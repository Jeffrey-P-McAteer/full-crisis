use rust_embed::RustEmbed;
use std::io::Cursor;

#[cfg(not(target_arch = "wasm32"))]
use rodio::{Decoder, OutputStreamBuilder, Sink};

#[derive(RustEmbed)]
#[folder = "$AUDIO_BUILD_DIR"]
pub struct AudioAssets;

pub struct AudioManager {
    #[cfg(not(target_arch = "wasm32"))]
    _stream: rodio::OutputStream,
    #[cfg(not(target_arch = "wasm32"))]
    background_sink: Option<Sink>,
    #[cfg(target_arch = "wasm32")]
    is_playing: bool,
}

impl std::fmt::Debug for AudioManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AudioManager")
            .field("background_sink_active", &self.is_background_playing())
            .finish()
    }
}

impl AudioManager {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        #[cfg(not(target_arch = "wasm32"))]
        {
            let _stream = OutputStreamBuilder::open_default_stream()
                .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)?;
            Ok(AudioManager {
                _stream,
                background_sink: None,
            })
        }
        
        #[cfg(target_arch = "wasm32")]
        {
            Ok(AudioManager {
                is_playing: false,
            })
        }
    }
    
    pub fn play_intro_chime_looped(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        #[cfg(not(target_arch = "wasm32"))]
        {
            if let Some(audio_data) = AudioAssets::get("intro_chime.wav") {
                // Stop any existing background music
                self.stop_background_music();
                
                // Create a new sink for the background music
                let sink = Sink::connect_new(&self._stream.mixer());
                
                // Clone the data to avoid lifetime issues
                let audio_bytes = audio_data.data.to_vec();
                
                // Add the audio multiple times for looping effect
                for _ in 0..100 {  // Loop 100 times (should be enough for menu time)
                    let cursor = Cursor::new(audio_bytes.clone());
                    if let Ok(source) = Decoder::new(cursor) {
                        sink.append(source);
                    }
                }
                
                sink.set_volume(0.2);  // Quieter for background
                self.background_sink = Some(sink);
            }
        }
        
        #[cfg(target_arch = "wasm32")]
        {
            if let Some(audio_data) = AudioAssets::get("intro_chime.wav") {
                // Use the web audio callback system
                crate::gui::event_handlers::web_play_background_audio(&audio_data.data);
                self.is_playing = true;
            }
        }
        
        Ok(())
    }
    
    pub fn stop_background_music(&mut self) {
        #[cfg(not(target_arch = "wasm32"))]
        {
            if let Some(sink) = self.background_sink.take() {
                sink.stop();
            }
        }
        
        #[cfg(target_arch = "wasm32")]
        {
            // Use the web audio callback system
            crate::gui::event_handlers::web_stop_background_audio();
            self.is_playing = false;
        }
    }
    
    pub fn play_sound_effect(&self, filename: &str) -> Result<(), Box<dyn std::error::Error>> {
        #[cfg(not(target_arch = "wasm32"))]
        {
            if let Some(audio_data) = AudioAssets::get(filename) {
                let audio_bytes = audio_data.data.to_vec();
                let cursor = Cursor::new(audio_bytes);
                let source = Decoder::new(cursor)?;
                let sink = Sink::connect_new(&self._stream.mixer());
                sink.append(source);
                sink.set_volume(0.5);
                sink.detach(); // Let it play independently
            }
        }
        
        Ok(())
    }
    
    pub fn is_background_playing(&self) -> bool {
        #[cfg(not(target_arch = "wasm32"))]
        {
            self.background_sink.as_ref().map_or(false, |sink| !sink.empty())
        }
        
        #[cfg(target_arch = "wasm32")]
        {
            self.is_playing
        }
    }
}

impl Default for AudioManager {
    fn default() -> Self {
        Self::new().unwrap_or_else(|_| {
            #[cfg(not(target_arch = "wasm32"))]
            {
                // Fallback with dummy values if audio initialization fails
                if let Ok(_stream) = OutputStreamBuilder::open_default_stream() {
                    AudioManager {
                        _stream,
                        background_sink: None,
                    }
                } else {
                    // If even fallback fails, create a minimal struct
                    panic!("Cannot initialize audio system");
                }
            }
            
            #[cfg(target_arch = "wasm32")]
            {
                AudioManager {
                    is_playing: false,
                }
            }
        })
    }
}