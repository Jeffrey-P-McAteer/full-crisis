use crate::gui::types::GameMessage;
use crate::translations::{TranslationKey, t};
use iced::Task;

/// Common translation patterns to reduce verbosity
pub struct TranslationUtils;

impl TranslationUtils {
    pub fn translate(key: TranslationKey, language: &str) -> String {
        t(key, language)
    }
    
    pub fn translate_with_var(key: TranslationKey, language: &str, var_name: &str, value: &str) -> String {
        let base = t(key, language);
        base.replace(&format!("{{{}}}", var_name), value)
    }
}

/// Common audio operations
pub struct AudioUtils;

impl AudioUtils {
    pub fn handle_scene_audio(_audio_data: &[u8]) -> Task<GameMessage> {
        #[cfg(target_arch = "wasm32")]
        {
            if !_audio_data.is_empty() {
                crate::web_helpers::play_audio_from_bytes(_audio_data.to_vec());
            }
        }
        
        #[cfg(not(target_arch = "wasm32"))]
        {
            // Desktop audio handling through existing game systems
            // Audio is handled through the crisis loading system
        }
        
        Task::none()
    }
    
    pub fn start_menu_audio() {
        #[cfg(not(target_arch = "wasm32"))]
        {
            if let Some(audio_manager) = crate::AUDIO_MANAGER.get() {
                if let Ok(mut manager) = audio_manager.lock() {
                    let _ = manager.play_intro_chime_looped();
                }
            }
        }
    }
    
    pub fn stop_menu_audio() {
        #[cfg(not(target_arch = "wasm32"))]
        {
            if let Some(audio_manager) = crate::AUDIO_MANAGER.get() {
                if let Ok(mut manager) = audio_manager.lock() {
                    manager.stop_background_music();
                }
            }
        }
    }
}