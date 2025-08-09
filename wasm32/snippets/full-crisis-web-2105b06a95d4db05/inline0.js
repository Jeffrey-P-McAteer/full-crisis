
export function os_prefers_dark() {
    return window.matchMedia && window.matchMedia('(prefers-color-scheme: dark)').matches;
}
