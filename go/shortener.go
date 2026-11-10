package main

/*
#cgo CFLAGS: -I..
#cgo LDFLAGS: -L../target/release -llibrary
#include "url_shortener.h"
#include <stdlib.h>
*/
import "C"
import (
	"log"
	"unsafe"
)

func main() {
	C.url_shortener_seed(42)

	if C.url_shortener_init_memory() != 0 {
		log.Fatalf("Init failed")
	}
	defer C.url_shortener_destroy()

	url := C.CString("https://hagen-schupp.me")
	defer C.free(unsafe.Pointer(url))
	log.Printf("Url: %s\n", C.GoString(url))

	shortenedUrl := C.shorten(url)
	defer C.free_string(shortenedUrl)
	log.Printf("Shortened: %s\n", C.GoString(shortenedUrl))

	reversedUrl := C.resolve(shortenedUrl)
	defer C.free_string(reversedUrl)
	log.Printf("Reversed: %s\n", C.GoString(reversedUrl))
}
