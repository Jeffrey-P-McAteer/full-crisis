use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DifficultyLevel {
    Easy,
    Medium,
    Hard,
}

impl DifficultyLevel {
    pub fn to_translated_string(&self, language: &str) -> String {
        match self {
            DifficultyLevel::Easy => crate::translations::t(crate::translations::TranslationKey::Easy, language),
            DifficultyLevel::Medium => crate::translations::t(crate::translations::TranslationKey::Medium, language),
            DifficultyLevel::Hard => crate::translations::t(crate::translations::TranslationKey::Hard, language),
        }
    }
}

impl std::fmt::Display for DifficultyLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Fallback to English for Display trait
        write!(f, "{}", self.to_translated_string("eng"))
    }
}

impl DifficultyLevel {
    pub const ALL: [DifficultyLevel; 3] = [
        DifficultyLevel::Easy,
        DifficultyLevel::Medium,
        DifficultyLevel::Hard,
    ];
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameSettings {
    pub game_save_folder: String,
    pub difficulty_level: DifficultyLevel,
    pub autosave: bool,
    pub language: String,
}

impl Default for GameSettings {
    fn default() -> Self {
        Self {
            game_save_folder: String::from("./saves"),
            difficulty_level: DifficultyLevel::Medium,
            autosave: true,
            language: crate::language::detect_system_language(),
        }
    }
}

pub struct GameWindow {
    pub os_theme: crate::game::OSColorTheme,
    pub game_state: crate::game::GameState,
    pub new_game_player_name: String,
    pub new_game_game_template: Option<String>,
    pub continue_game_game_choice: Option<String>,
    pub settings_game_save_folder: String,
    pub settings_difficulty_level: DifficultyLevel,
    pub settings_autosave: bool,
    pub settings_language: String,
    pub current_crisis: Option<crate::crisis::CrisisDefinition>,
    pub story_state: Option<crate::crisis::GameState>,
}

#[derive(Debug, Clone)]
pub enum GameMessage {
    Nop,
    Menu_NewGameRequested,
        Menu_NewGamePlayerNameAltered(String),
        Menu_NewGameTemplateChoiceAltered(String),
        Menu_NewGameStartClicked,
    Menu_ContinueGameRequested,
        Menu_ContinueGameChoiceAltered(String),
    Menu_SettingsRequested,
        Menu_SettingsGameSaveFolderChanged(String),
        Menu_SettingsDifficultyLevelChanged(DifficultyLevel),
        Menu_SettingsAutosaveToggled(bool),
        Menu_SettingsLanguageChanged(String),
    QuitGameRequested,
    Game_ChoiceSelected(usize),
    Game_RestartRequested,
}

