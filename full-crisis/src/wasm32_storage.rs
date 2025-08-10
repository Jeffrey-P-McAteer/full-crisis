
// Expose a JS function to Rust using wasm-bindgen
#[wasm_bindgen(inline_js = "
export function js_get_attr(name) {
    return localStorage.getItem(name) || '';
}
export function js_set_attr(name, value) {
    localStorage.setItem(name, value);
}
")]
extern "C" {
    pub fn js_get_attr(&str) -> String;
    pub fn js_set_attr(&str, &str);
}

pub fn get_attr(name: &str) -> Option<String> {
  js_get_attr(name)
}

pub fn set_attr(name: &str, value: &str) {
  js_set_attr(name, value)
}

