use wasm_bindgen::prelude::*;

// Responsible for 2.5mb less WASM code generated, TODO measure performance impact
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen(start)]
pub fn start() -> Result<(), JsValue> {
    console_error_panic_hook::set_once();

    full_crisis::init_global_vars();

    // Detect browser theme + store in variable
    let _ = full_crisis::OS_COLOR_THEME.set(
        if os_prefers_dark() { full_crisis::game::OSColorTheme::Dark }
        else                 { full_crisis::game::OSColorTheme::Light }
    );


    // Iced wants to own the GUI thread and insists on using the main thread; so we let it.
    iced::application(
        full_crisis::gui::GameWindow::new,
        full_crisis::gui::GameWindow::update,
        full_crisis::gui::GameWindow::view,
    )
    .theme(full_crisis::gui::GameWindow::theme)
    //.font(include_bytes!("../fonts/icons.ttf").as_slice())
    .default_font(iced::Font::MONOSPACE)
    .settings(full_crisis::gui::GameWindow::make_app_settings())
    .window(full_crisis::gui::GameWindow::make_window_settings())
    .run()
    .map_err(|e| JsValue::from_str(&format!("{:?}", e)))
}


// Expose a JS function to Rust using wasm-bindgen
#[wasm_bindgen(inline_js = "
export function os_prefers_dark() {
    return window.matchMedia && window.matchMedia('(prefers-color-scheme: dark)').matches;
}

export function browser_go_back() {
    window.history.back();
}
")]
extern "C" {
    pub fn os_prefers_dark() -> bool;
    pub fn browser_go_back();
}

