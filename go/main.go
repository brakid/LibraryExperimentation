package main

/*
#cgo CFLAGS: -I..
#cgo LDFLAGS: -L../target/release -lfibonacci
#include "fibonacci.h"
*/
import "C"
import (
	"fmt"
	"time"
)

func main() {
	fmt.Printf("Call Rust Library\n")
	var n C.ulonglong = 50

	start := time.Now()
	result := C.fibonacci(n)
	duration := time.Since(start)
	fmt.Printf("Cached Fibonacci: %v - %v\n", result, duration)

	start = time.Now()
	result = C.uncached_fibonacci(n)
	duration = time.Since(start)
	fmt.Printf("Cached Fibonacci: %v - %v\n", result, duration)
}
