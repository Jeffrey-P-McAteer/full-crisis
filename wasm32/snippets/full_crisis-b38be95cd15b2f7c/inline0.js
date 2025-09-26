
export function js_get_attr(name) {
    return localStorage.getItem(name) || '';
}
export function js_set_attr(name, value) {
    localStorage.setItem(name, value);
}
export function js_get_timestamp() {
    return Date.now();
}
