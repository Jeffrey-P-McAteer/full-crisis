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
    pub focus_state: FocusState, // Focus tracking state
}

#[derive(Debug, Clone)]
pub enum GameMessage {
    Nop,
    
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
    
    // Focus system messages
    Focus_NavigateUp,
    Focus_NavigateDown,
    Focus_NavigateLeft,
    Focus_NavigateRight,
    Focus_Activate, // Enter key or similar
}

// Focus system types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FocusId(pub &'static str, pub usize); // (category, index)

impl FocusId {
    // Left panel - menu buttons
    pub const fn menu_button(index: usize) -> Self {
        Self("menu", index)
    }
    
    // Right panel - new game elements
    pub const fn new_game_input(index: usize) -> Self {
        Self("newgame_input", index)
    }
    
    pub const fn new_game_button(index: usize) -> Self {
        Self("newgame_button", index)
    }
    
    // Right panel - continue game elements  
    pub const fn continue_game_input(index: usize) -> Self {
        Self("continue_input", index)
    }
    
    pub const fn continue_game_button(index: usize) -> Self {
        Self("continue_button", index)
    }
    
    // Continue game confirmation dialog
    pub const fn continue_game_confirm(index: usize) -> Self {
        Self("continue_confirm", index)
    }
    
    // Right panel - settings elements
    pub const fn settings_input(index: usize) -> Self {
        Self("settings_input", index)
    }
    
    pub const fn settings_button(index: usize) -> Self {
        Self("settings_button", index)
    }
    
    pub const fn settings_picker(index: usize) -> Self {
        Self("settings_picker", index)
    }
    
    pub const fn settings_toggle(index: usize) -> Self {
        Self("settings_toggle", index)
    }
    
    pub const fn settings_slider(index: usize) -> Self {
        Self("settings_slider", index)
    }
    
    // Game screen elements
    pub const fn game_choice(index: usize) -> Self {
        Self("choice", index)
    }
    
    pub const fn game_control(index: usize) -> Self {
        Self("control", index)
    }
}

#[derive(Debug, Clone)]
pub struct FocusState {
    pub current_focus: Option<FocusId>,
    pub focusable_elements: Vec<FocusId>,
    pub enabled: bool,
}

impl Default for FocusState {
    fn default() -> Self {
        Self {
            current_focus: None,
            focusable_elements: Vec::new(),
            enabled: true,
        }
    }
}

impl FocusState {
    pub fn new() -> Self {
        Self::default()
    }
    
    pub fn set_focusable_elements(&mut self, elements: Vec<FocusId>) {
        self.focusable_elements = elements.clone();
        if self.current_focus.is_none() && !elements.is_empty() {
            self.current_focus = Some(elements[0]);
        }
        if let Some(current) = self.current_focus {
            if !elements.contains(&current) {
                self.current_focus = elements.first().copied();
            }
        }
    }
    
    pub fn navigate_up(&mut self) {
        self.navigate_vertical(-1);
    }
    
    pub fn navigate_down(&mut self) {
        self.navigate_vertical(1);
    }
    
    pub fn navigate_left(&mut self) {
        self.navigate_horizontal(-1);
    }
    
    pub fn navigate_right(&mut self) {
        self.navigate_horizontal(1);
    }
    
    fn navigate_vertical(&mut self, direction: i32) {
        if self.focusable_elements.is_empty() { return; }
        
        let current_idx = if let Some(current) = self.current_focus {
            self.focusable_elements.iter().position(|&id| id == current)
                .unwrap_or(0)
        } else {
            0
        };
        
        let new_idx = if direction > 0 {
            (current_idx + 1) % self.focusable_elements.len()
        } else {
            if current_idx == 0 {
                self.focusable_elements.len() - 1
            } else {
                current_idx - 1
            }
        };
        
        self.current_focus = Some(self.focusable_elements[new_idx]);
    }
    
    fn navigate_horizontal(&mut self, direction: i32) {
        if self.focusable_elements.is_empty() { return; }
        
        let current_focus = if let Some(current) = self.current_focus {
            current
        } else {
            self.current_focus = Some(self.focusable_elements[0]);
            return;
        };
        
        // Check if we're in left panel (menu buttons) or right panel (other controls)
        let is_in_left_panel = current_focus.0 == "menu";
        
        if direction > 0 { // Moving right
            if is_in_left_panel {
                // Move from left panel to right panel - find first right panel element
                let first_right_element = self.focusable_elements.iter()
                    .find(|&element| element.0 != "menu");
                if let Some(&element) = first_right_element {
                    self.current_focus = Some(element);
                }
            } else {
                // Already in right panel, navigate within right panel
                self.navigate_within_panel(direction);
            }
        } else { // Moving left  
            if is_in_left_panel {
                // Already in left panel, navigate within left panel
                self.navigate_within_panel(direction);
            } else {
                // Move from right panel to left panel - find first left panel element
                let first_left_element = self.focusable_elements.iter()
                    .find(|&element| element.0 == "menu");
                if let Some(&element) = first_left_element {
                    self.current_focus = Some(element);
                }
            }
        }
    }
    
    fn navigate_within_panel(&mut self, direction: i32) {
        if let Some(current) = self.current_focus {
            let panel = current.0;
            
            // Get all elements in the same panel
            let panel_elements: Vec<FocusId> = self.focusable_elements.iter()
                .filter(|&element| element.0 == panel)
                .copied()
                .collect();
            
            if panel_elements.is_empty() { return; }
            
            let current_idx = panel_elements.iter().position(|&id| id == current)
                .unwrap_or(0);
            
            let new_idx = if direction > 0 {
                (current_idx + 1) % panel_elements.len()
            } else {
                if current_idx == 0 {
                    panel_elements.len() - 1
                } else {
                    current_idx - 1
                }
            };
            
            self.current_focus = Some(panel_elements[new_idx]);
        }
    }
    
    pub fn is_focused(&self, id: FocusId) -> bool {
        self.enabled && self.current_focus == Some(id)
    }
}

