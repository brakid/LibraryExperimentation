use lru::LruCache;
use std::num::NonZeroUsize;
use std::sync::{Mutex, OnceLock};

static CACHE: OnceLock<Mutex<LruCache<u64, u64>>> = OnceLock::new();

fn get_cache() -> &'static Mutex<LruCache<u64, u64>> {
    CACHE.get_or_init(|| Mutex::new(LruCache::new(NonZeroUsize::new(1000).unwrap())))
}

#[unsafe(no_mangle)]
pub extern "C" fn fibonacci(n: u64) -> u64 {
    if n <= 1 {
        return n;
    }

    {
        let mut cache = get_cache().lock().unwrap();
        if let Some(&value) = cache.get(&n) {
            return value;
        }
    }

    let value = fibonacci(n - 1) + fibonacci(n - 2);
    {
        let mut cache = get_cache().lock().unwrap();
        cache.put(n, value);
    }

    value
}

#[unsafe(no_mangle)]
pub extern "C" fn uncached_fibonacci(n: u64) -> u64 {
    if n <= 1 {
        return n;
    }

    uncached_fibonacci(n - 1) + uncached_fibonacci(n - 2)
}