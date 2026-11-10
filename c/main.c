#include <fibonacci.h>
#include <stdio.h>

int main() {
    printf("Call Rust Library\n");

    uint64_t n = 50;

    printf("Cached: %lld\n", fibonacci(n));
    printf("Uncached: %lld\n", uncached_fibonacci(n));
    return 0;
}