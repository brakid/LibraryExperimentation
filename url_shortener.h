#ifndef URL_SHORTENER_H
#define URL_SHORTENER_H

#ifdef __cplusplus
extern "C" {
#endif

/* Initialize the global store with an in-memory backend.
 * Returns 0 on success, non-zero on error (e.g. already initialized). */
int url_shortener_init_memory(void);

/* Tear down the global store, releasing the Redis connection (if any).
 * After this, init_* must be called again before shorten/resolve. */
void url_shortener_destroy(void);

/* Seed the internal RNG for deterministic code generation (useful for tests).
 * Pass any u64; calling with the same seed yields the same sequence. */
void url_shortener_seed(unsigned long long seed);

/* Initialize the global store with a Redis backend at the given URL
 * (e.g. "redis://127.0.0.1/"). Returns 0 on success, non-zero on error. */
int url_shortener_init_redis(const char *url);

/* Shorten `url` into a 10-character alphanumeric code.
 * Returns a heap-allocated string owned by the caller (free with free_string),
 * or NULL on failure / if the store was not initialized. */
char *shorten(const char *url);

/* Resolve a previously shortened `code` back to its original URL.
 * Returns a heap-allocated string owned by the caller (free with free_string),
 * or NULL if not found. */
char *resolve(const char *code);

/* Free a string returned by shorten/resolve. */
void free_string(char *ptr);

#ifdef __cplusplus
}
#endif

#endif /* URL_SHORTENER_H */