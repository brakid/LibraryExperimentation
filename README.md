# URL Shortener (Rust, C ABI)

A small URL-shortening library exposed through a C ABI (`cdylib`/`staticlib`).
URLs are mapped to 10-character alphanumeric codes, backed by either an
in-memory store or a real Redis instance, selectable at runtime.

## Build

```sh
cargo build            # debug
cargo build --release   # release
```

Produces `target/{debug,release}/liburl_shortener.{a,dylib,so}` plus the
header `url_shortener.h`.

## Setup / usage (C)

```c
#include "url_shortener.h"

int main(void) {
    // Pick ONE backend before calling shorten/resolve:
    url_shortener_init_memory();            // in-memory
    // url_shortener_init_redis("redis://127.0.0.1/");

    // Optional: deterministic codes for tests
    url_shortener_seed(42);

    char *code = shorten("https://example.com/some/long/path");
    // code -> "aB3xY9qLm2" (example)

    char *url = resolve(code);
    // url -> "https://example.com/some/long/path"

    free_string(code);
    free_string(url);

    url_shortener_destroy();  // releases the Redis connection (if any)
    return 0;
}
```

## Setup / usage (Go via cgo)

```go
// #cgo LDFLAGS: -L. -lurl_shortener
// #include <stdlib.h>
// #include "url_shortener.h"
import "C"
import "unsafe"

func shortenURL(u string) string {
    c := C.CString(u)
    defer C.free(unsafe.Pointer(c))
    res := C.shorten(c)
    if res == nil { return "" }
    defer C.free_string(res)
    return C.GoString(res)
}
```

## Redis persistence & replicas

Redis is not durable by default on every write. This project's `docker-compose.yml`
enables AOF with `appendfsync everysec` (fsync once per second, up to ~1s loss
on crash). For stronger durability use `--appendfsync always` (slower); for
weaker, `--appendfsync no`.

Data is stored in a named Docker volume (`redis_data`) so it survives container
restarts.

A read-only replica is included (`redis-replica`, port 6380). Redis replicas are
read-only by default (`replica-read-only yes`), so they serve reads but reject
writes. The library connects to a single instance via `url_shortener_init_redis`;
point it at the primary for writes. Replicas are useful for scaling reads or
failover, but the library does not (yet) route reads to replicas automatically.

## Multithreading

The library is **safe to use from multiple threads**:

- The global store is a `OnceLock<Mutex<Option<Box<dyn Store + Send>>>>`; all
  `shorten`/`resolve`/`destroy` calls lock it, so concurrent access is
  serialized and data races are prevented.
- The RNG is likewise guarded by a `Mutex`.
- Redis connections are not shared across threads without synchronization; the
  `Mutex` around the store ensures a single connection is used at a time.

Trade-off: locking serializes operations, so under high concurrency a single
global lock may become a bottleneck. For higher throughput, prefer the
in-memory backend (no network round-trip) or shard by running multiple
processes.

## Notes

- `shorten` is idempotent: shortening the same URL twice returns the same code.
- Records expire after 30 days in Redis (key TTL). The in-memory store keeps entries indefinitely.
- Codes are 10 alphanumeric characters, generated randomly with collision
  retries (up to 10).
- Strings returned by `shorten`/`resolve` are heap-allocated by Rust and must
  be freed with `free_string`, not `free`.
- The store must be initialized exactly once; calling an init function twice
  returns `-4`.
