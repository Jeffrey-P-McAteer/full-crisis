use wasm_bindgen::prelude::*;

#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen(start)]
pub fn start() -> Result<(), JsValue> {
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

