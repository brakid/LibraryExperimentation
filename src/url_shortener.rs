use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use redis::{Client, Commands, Connection};
use std::collections::HashMap;
use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use std::sync::{Mutex, OnceLock};

trait Store: Send {
    fn get(&mut self, key: &str) -> Option<String>;
    fn set(&mut self, key: &str, value: &str);
}

struct RedisStore {
    conn: Connection,
}

impl RedisStore {
    fn new(url: &str) -> Result<Self, redis::RedisError> {
        let client = Client::open(url)?;
        let conn = client.get_connection()?;
        Ok(Self { conn })
    }
}

impl Store for RedisStore {
    fn get(&mut self, key: &str) -> Option<String> {
        self.conn.get(key).ok()
    }
    fn set(&mut self, key: &str, value: &str) {
        let _: Result<(), _> = self.conn.set_ex(key, value, TTL_SECS);
    }
}

struct MemoryStore {
    map: HashMap<String, String>,
}

impl MemoryStore {
    fn new() -> Self {
        Self {
            map: HashMap::new(),
        }
    }
}

impl Store for MemoryStore {
    fn get(&mut self, key: &str) -> Option<String> {
        self.map.get(key).cloned()
    }
    fn set(&mut self, key: &str, value: &str) {
        self.map.insert(key.to_string(), value.to_string());
    }
}

static STORE: OnceLock<Mutex<Option<Box<dyn Store + Send>>>> = OnceLock::new();

fn store_mutex() -> &'static Mutex<Option<Box<dyn Store + Send>>> {
    STORE.get_or_init(|| Mutex::new(None))
}

const ALPHANUM: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
const CODE_LEN: usize = 10;
const MAX_RETRIES: usize = 10;
const TTL_SECS: u64 = 30 * 24 * 60 * 60; // 30 days

static RNG: OnceLock<Mutex<Option<StdRng>>> = OnceLock::new();

fn rng_mutex() -> &'static Mutex<Option<StdRng>> {
    RNG.get_or_init(|| Mutex::new(None))
}

fn gen_code() -> String {
    let mut guard = rng_mutex().lock().unwrap();
    let rng = guard.get_or_insert_with(|| StdRng::seed_from_u64(0));
    (0..CODE_LEN)
        .map(|_| ALPHANUM[rng.gen_range(0..ALPHANUM.len())] as char)
        .collect()
}

fn to_c_string(s: String) -> *mut c_char {
    match CString::new(s) {
        Ok(c) => c.into_raw(),
        Err(_) => std::ptr::null_mut(),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn url_shortener_init_redis(url: *const c_char) -> i32 {
    if url.is_null() {
        return -1;
    }
    let url = unsafe { CStr::from_ptr(url) };
    let url = match url.to_str() {
        Ok(s) => s,
        Err(_) => return -2,
    };
    let store = match RedisStore::new(url) {
        Ok(s) => Box::new(s) as Box<dyn Store + Send>,
        Err(_) => return -3,
    };
    let mut guard = store_mutex().lock().unwrap();
    if guard.is_some() {
        return -4;
    }
    *guard = Some(store);
    0
}

#[unsafe(no_mangle)]
pub extern "C" fn url_shortener_init_memory() -> i32 {
    let store = Box::new(MemoryStore::new()) as Box<dyn Store + Send>;
    let mut guard = store_mutex().lock().unwrap();
    if guard.is_some() {
        return -4;
    }
    *guard = Some(store);
    0
}

#[unsafe(no_mangle)]
pub extern "C" fn url_shortener_seed(seed: u64) {
    let mut guard = rng_mutex().lock().unwrap();
    *guard = Some(StdRng::seed_from_u64(seed));
}

#[unsafe(no_mangle)]
pub extern "C" fn url_shortener_destroy() {
    let mutex = store_mutex();
    let mut guard = mutex.lock().unwrap();
    *guard = None;
}

#[unsafe(no_mangle)]
pub extern "C" fn shorten(url: *const c_char) -> *mut c_char {
    if url.is_null() {
        return std::ptr::null_mut();
    }
    let url = unsafe { CStr::from_ptr(url) };
    let url = match url.to_str() {
        Ok(s) => s.to_string(),
        Err(_) => return std::ptr::null_mut(),
    };

    let mutex = store_mutex();
    let mut guard = mutex.lock().unwrap();
    let store = match guard.as_mut() {
        Some(s) => s,
        None => return std::ptr::null_mut(),
    };

    let rev_key = format!("url:{}", url);
    if let Some(code) = store.get(&rev_key) {
        return to_c_string(code);
    }

    // TODO: propagate write errors (Store::set -> Result) so that writing to
    //       a read-only replica / unavailable Redis fails loudly instead of
    //       returning a code that was never persisted.
    for _ in 0..MAX_RETRIES {
        let code = gen_code();
        let code_key = format!("code:{}", code);
        if store.get(&code_key).is_some() {
            continue;
        }
        store.set(&code_key, &url);
        store.set(&rev_key, &code);
        return to_c_string(code);
    }

    std::ptr::null_mut()
}

#[unsafe(no_mangle)]
pub extern "C" fn resolve(code: *const c_char) -> *mut c_char {
    if code.is_null() {
        return std::ptr::null_mut();
    }
    let code = unsafe { CStr::from_ptr(code) };
    let code = match code.to_str() {
        Ok(s) => s.to_string(),
        Err(_) => return std::ptr::null_mut(),
    };

    let mutex = store_mutex();
    let mut guard = mutex.lock().unwrap();
    let store = match guard.as_mut() {
        Some(s) => s,
        None => return std::ptr::null_mut(),
    };

    let code_key = format!("code:{}", code);
    match store.get(&code_key) {
        Some(url) => to_c_string(url),
        None => std::ptr::null_mut(),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn free_string(ptr: *mut c_char) {
    if !ptr.is_null() {
        unsafe {
            let _ = CString::from_raw(ptr);
        }
    }
}