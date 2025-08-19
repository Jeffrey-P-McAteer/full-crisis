use crate::gui::types::*;

impl GameWindow {
    pub fn save_settings(&self) {
        let settings = GameSettings {
            game_save_folder: self.settings_game_save_folder.clone(),
            difficulty_level: self.settings_difficulty_level,
            autosave: self.settings_autosave,
            language: self.settings_language.clone(),
            last_username: self.new_game_player_name.clone(),
        };
        crate::storage::set_struct("game_settings", &settings);
    }

    pub fn load_settings() -> GameSettings {
        crate::storage::get_struct("game_settings").unwrap_or_default()
    }
}