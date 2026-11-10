#ifndef _FIBONACCI
#define _FIBONACCI

#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

uint64_t fibonacci(uint64_t n);
uint64_t uncached_fibonacci(uint64_t n);

#ifdef __cplusplus
}
#endif

#endif // _FIBONACCI