import { dlopen, CString, FFIType, suffix, ptr } from 'bun:ffi';
import { exit } from 'process';

const { symbols: { url_shortener_seed, url_shortener_destroy, url_shortener_init_redis, resolve, free_string } } = dlopen(`../target/release/liblibrary.${suffix}`, {
    url_shortener_seed: {
        args: [FFIType.u64],
        returns: FFIType.void
    },
    url_shortener_destroy: {
        args: [],
        returns: FFIType.void
    },
    url_shortener_init_redis: {
        args: [FFIType.ptr],
        returns: FFIType.i32
    },
    resolve: {
        args: [FFIType.ptr],
        returns: FFIType.ptr
    },
    free_string: {
        args: [FFIType.ptr],
        returns: FFIType.void
    }
});

url_shortener_seed(42);
const redisUrl = "redis://127.0.0.1/";
if (url_shortener_init_redis(ptr(Buffer.from(redisUrl))) != 0) {
    console.log("Error init");
    exit(1);
}

const shortened = "ghnZCZt0IA";
const urlPtr = resolve(ptr(Buffer.from(shortened)));
const code = new CString(urlPtr).toString();
console.log(code);

free_string(urlPtr);

url_shortener_destroy();