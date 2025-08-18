use serde::{Serialize, Deserialize};
use iced::{Center, Element, Fill, Length, Task, Theme};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DifficultyLevel {
    Easy,
    Medium,
    Hard,
}

impl std::fmt::Display for DifficultyLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DifficultyLevel::Easy => write!(f, "Easy"),
            DifficultyLevel::Medium => write!(f, "Medium"),
            DifficultyLevel::Hard => write!(f, "Hard"),
        }
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
    pub game_text_input: String,
    pub player_cash: i32,
    pub player_health: i32,
    pub player_popularity: i32,
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
    Game_TextInputChanged(String),
    Game_TextInputSubmitted,
    Game_ChoiceSelected(usize),
    Game_RestartRequested,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NewGame_Type {
    Type_A, Type_B, Type_C,
}

impl std::fmt::Display for NewGame_Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NewGame_Type::Type_A => write!(f, "Type A"),
            NewGame_Type::Type_B => write!(f, "Type B"),
            NewGame_Type::Type_C => write!(f, "type C"),
        }
    }
}

impl NewGame_Type {
    pub const ALL: [NewGame_Type; 3] = [
        NewGame_Type::Type_A,
        NewGame_Type::Type_B,
        NewGame_Type::Type_C,
    ];
}