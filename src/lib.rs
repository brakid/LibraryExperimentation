mod fibonacci;
mod url_shortener;

// Re-export FFI symbols are already #[no_mangle] in submodules,
// but ensure modules are compiled into the crate.
pub use fibonacci::*;
pub use url_shortener::*;