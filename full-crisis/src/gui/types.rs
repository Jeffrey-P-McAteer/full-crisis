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
    pub game_crises_folder: String,
    pub difficulty_level: DifficultyLevel,
    pub autosave: bool,
    pub language: String,
    pub last_username: String,
    pub font_scale: f32,
}

impl Default for GameSettings {
    fn default() -> Self {
        Self {
            game_crises_folder: String::from("./crises"),
            difficulty_level: DifficultyLevel::Medium,
            autosave: true,
            language: crate::language::detect_system_language(),
            last_username: String::new(),
            font_scale: 1.0,
        }
    }
}

pub struct GameWindow {
    pub os_theme: crate::game::OSColorTheme,
    pub game_state: crate::game::GameState,
    pub new_game_player_name: String,
    pub new_game_game_template: Option<String>,
    pub new_game_selected_description: Option<String>,
    pub continue_game_game_choice: Option<String>,
    pub continue_game_delete_confirmation: Option<String>,
    pub settings_game_crises_folder: String,
    pub settings_difficulty_level: DifficultyLevel,
    pub settings_autosave: bool,
    pub settings_language: String,
    pub settings_font_scale: f32,
    pub current_crisis: Option<crate::crisis::CrisisDefinition>,
    pub story_state: Option<crate::crisis::GameState>,
    pub choice_text_inputs: std::collections::HashMap<usize, String>, // Track text input values by choice index
    pub animation_frame_index: usize, // Current frame index for character animation
    pub current_background_audio: Vec<u8>, // Current background audio data to play
    pub menu_focused_button: usize, // Track which menu button is focused (0-4 for the 5 main buttons)
    pub menu_right_panel_focused: bool, // Track if focus is on the right panel
    pub pick_list_expanded: bool, // Track if a pick list is currently expanded
}

#[derive(Debug, Clone)]
pub enum GameMessage {
    Nop,
    
    // Keyboard navigation
    NavigateUp,
    NavigateDown,
    NavigateLeft,
    NavigateRight,
    NavigateTab,
    NavigateShiftTab,
    NavigateEnter,
    NavigateEscape,
    
    // Menu actions
    Menu_NewGameRequested,
    Menu_NewGamePlayerNameAltered(String),
    Menu_NewGameTemplateChoiceAltered(String),
    Menu_NewGameStartClicked,
    
    Menu_ContinueGameRequested,
    Menu_ContinueGameChoiceAltered(String),
    Menu_ContinueGameStartClicked,
    Menu_ContinueGameDeleteRequested(String),
    Menu_ContinueGameDeleteConfirmed(String),
    
    Menu_SettingsRequested,
    Menu_SettingsGameCrisesFolderChanged(String),
    Menu_SettingsOpenCrisesFolder,
    Menu_SettingsDifficultyLevelChanged(DifficultyLevel),
    Menu_SettingsAutosaveToggled(bool),
    Menu_SettingsLanguageChanged(String),
    Menu_SettingsFontScaleChanged(f32),
    
    Menu_LicensesRequested,
    QuitGameRequested,
    
    // Game actions
    Game_ChoiceSelected(usize),
    Game_TextInputChanged(usize, String), // (choice_index, input_value)
    Game_TextInputSubmitted(usize, String), // (choice_index, input_value)
    Game_RestartRequested,
    Game_SaveAndQuitRequested,
    Game_QuitWithoutSaveRequested,
    Game_AnimationTick, // Timer message for character animation
}

