use wasm_bindgen::prelude::*;

// Responsible for 2.5mb less WASM code generated, TODO measure performance impact
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen(start)]
pub fn start() -> Result<(), JsValue> {
    #[cfg(target_arch = "wasm32")]
    {
        console_error_panic_hook::set_once();

        full_crisis::init_global_vars();

        // Initialize audio callbacks
        full_crisis::set_audio_callbacks(
            Box::new(|bytes: &[u8]| {
                play_background_audio(bytes);
            }),
            Box::new(|| {
                stop_background_audio();
            }),
        );

        // Detect browser theme + store in variable
        let _ = full_crisis::OS_COLOR_THEME.set(
            if os_prefers_dark() { full_crisis::game::OSColorTheme::Dark }
            else                 { full_crisis::game::OSColorTheme::Light }
        );
    }

    // Iced wants to own the GUI thread and insists on using the main thread; so we let it.
    iced::application(
        full_crisis::gui::GameWindow::new,
        full_crisis::gui::GameWindow::update,
        full_crisis::gui::GameWindow::view,
    )
    .subscription(full_crisis::gui::GameWindow::subscription)
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

// Global audio context for background audio playback
const context = new AudioContext();
let currentAudioSource = null;

// Play background audio from a Vec<u8> (Uint8Array)
export function play_background_audio(bytes) {
    try {
        // Stop any currently playing audio
        if (currentAudioSource) {
            currentAudioSource.stop();
            currentAudioSource = null;
        }
        
        // If empty array, just stop audio and return
        if (!bytes || bytes.length === 0) {
            return;
        }
        
        // Create buffer and copy the bytes
        const buffer = context.createBuffer(2, bytes.length, context.sampleRate);
        const arrayBuffer = new ArrayBuffer(bytes.length);
        const view = new Uint8Array(arrayBuffer);
        view.set(new Uint8Array(bytes), 0);
        
        // Decode the audio data
        context.decodeAudioData(arrayBuffer.slice(0), function(decodedBuffer) {
            // Create and configure the audio source
            const source = context.createBufferSource();
            source.buffer = decodedBuffer;
            source.loop = true; // Loop the audio
            source.connect(context.destination);
            
            // Play the audio
            source.start();
            currentAudioSource = source;
        }, function(error) {
            console.error('Error decoding audio data:', error);
        });
    } catch (error) {
        console.error('Error in play_background_audio:', error);
    }
}

// Stop any currently playing background audio
export function stop_background_audio() {
    if (currentAudioSource) {
        currentAudioSource.stop();
        currentAudioSource = null;
    }
}
")]
extern "C" {
    pub fn os_prefers_dark() -> bool;
    pub fn browser_go_back();
    pub fn play_background_audio(bytes: &[u8]);
    pub fn stop_background_audio();
}

