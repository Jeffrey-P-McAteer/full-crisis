
use wasm_bindgen::prelude::*;
use std::time::{SystemTime, Duration, UNIX_EPOCH};

// Expose a JS function to Rust using wasm-bindgen
#[wasm_bindgen(inline_js = "
export function js_get_attr(name) {
    return localStorage.getItem(name) || '';
}
export function js_set_attr(name, value) {
    localStorage.setItem(name, value);
}
export function js_get_timestamp() {
    return Date.now();
}
")]
unsafe extern "C" {
    pub fn js_get_attr(name: &str) -> String;
    pub fn js_set_attr(name: &str, value: &str);
    pub fn js_get_timestamp() -> f64;
}

pub fn get_attr(name: &str) -> Option<String> {
    let result = js_get_attr(name);
    if result.is_empty() {
        None
    } else {
        Some(result)
    }
}

pub fn set_attr(name: &str, value: &str) {
    js_set_attr(name, value)
}


pub fn time_now() -> SystemTime {
    let timestamp_ms = js_get_timestamp();
    let timestamp_secs = (timestamp_ms / 1000.0) as u64;
    let timestamp_nanos = ((timestamp_ms % 1000.0) * 1_000_000.0) as u32;
    UNIX_EPOCH + Duration::new(timestamp_secs, timestamp_nanos)
}

