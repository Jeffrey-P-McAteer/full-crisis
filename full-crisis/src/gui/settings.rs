use crate::gui::types::*;

impl GameWindow {
    pub fn save_settings(&self) {
        let settings = GameSettings {
            game_save_folder: self.settings_game_save_folder.clone(),
            difficulty_level: self.settings_difficulty_level,
            autosave: self.settings_autosave,
        };
        crate::native_storage::set_struct("game_settings", &settings);
    }

    pub fn load_settings() -> GameSettings {
        crate::native_storage::get_struct("game_settings").unwrap_or_default()
    }
}