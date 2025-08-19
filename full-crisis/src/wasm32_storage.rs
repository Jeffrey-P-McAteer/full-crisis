
use wasm_bindgen::prelude::*;
use std::time::{SystemTime, Duration, UNIX_EPOCH};
use serde::{Serialize, Deserialize};

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

pub fn get_struct<T>(name: &str) -> Option<T> 
where 
    T: for<'de> Deserialize<'de>,
{
    if let Some(content) = get_attr(name) {
        if !content.is_empty() {
            match serde_json::from_str::<T>(&content) {
                Ok(data) => return Some(data),
                Err(e) => {
                    web_sys::console::log_1(&format!("Error deserializing struct \"{}\": {:?}", name, e).into());
                }
            }
        }
    }
    None
}

pub fn set_struct<T>(name: &str, value: &T) 
where 
    T: Serialize,
{
    match serde_json::to_string(value) {
        Ok(serialized) => {
            set_attr(name, &serialized);
        },
        Err(e) => {
            web_sys::console::log_1(&format!("Error serializing struct \"{}\": {:?}", name, e).into());
        }
    }
}

pub fn time_now() -> SystemTime {
    let timestamp_ms = js_get_timestamp();
    let timestamp_secs = (timestamp_ms / 1000.0) as u64;
    let timestamp_nanos = ((timestamp_ms % 1000.0) * 1_000_000.0) as u32;
    UNIX_EPOCH + Duration::new(timestamp_secs, timestamp_nanos)
}

