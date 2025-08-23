use crate::gui::types::*;

impl GameWindow {
    pub fn save_settings(&self) {
        let settings = GameSettings {
            game_crises_folder: self.settings_game_crises_folder.clone(),
            difficulty_level: self.settings_difficulty_level,
            autosave: self.settings_autosave,
            language: self.settings_language.clone(),
            last_username: self.new_game_player_name.clone(),
            font_scale: self.settings_font_scale,
        };
        if let Ok(serialized) = serde_json::to_string(&settings) {
            crate::internal_storage::set_attr("game_settings", &serialized);
        }
    }

    pub fn load_settings() -> GameSettings {
        if let Some(content) = crate::internal_storage::get_attr("game_settings") {
            serde_json::from_str(&content).unwrap_or_default()
        } else {
            GameSettings::default()
        }
    }
}