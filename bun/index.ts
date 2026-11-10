import { dlopen, FFIType, suffix } from 'bun:ffi';

const { symbols: { fibonacci, uncached_fibonacci }} = dlopen(`../target/release/liblibrary.${suffix}`, {
    fibonacci: {
        args: [FFIType.u64],
        returns: FFIType.u64
    },
    uncached_fibonacci: {
        args: [FFIType.u64],
        returns: FFIType.u64
    },
});

console.log(`Call Rust Library`);
let start = Bun.nanoseconds();
let result = fibonacci(50);
let end = Bun.nanoseconds();
console.log(`Cached Fibonacci: ${result} - ${end - start}ns`);
start = Bun.nanoseconds();
result = uncached_fibonacci(50);
end = Bun.nanoseconds();
console.log(`Uncached Fibonacci: ${result} - ${end - start}ns`);