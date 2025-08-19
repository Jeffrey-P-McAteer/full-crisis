
#![allow(unreachable_patterns, non_camel_case_types)]

pub mod types;
pub mod settings;
pub mod ui_builders;
use types::*;

// Re-export key types for public use
pub use types::{GameWindow, GameMessage, DifficultyLevel, GameSettings};

use iced::widget::Space;
use iced::widget::Column;
use iced::widget::Row;
use iced::widget::{
    self, Container, Image, button, center_x, column, container, horizontal_space,
    pick_list, row, text, toggler, text_input,
};
use iced::{Center, Element, Length, Task, Theme};
// Immutable global data
const SPLASH_PNG_BYTES: &[u8] = include_bytes!("../../../icon/full-crisis-splash.transparent.png");





impl GameWindow {

    pub fn make_app_settings() -> iced::Settings {
        iced::Settings {
            ..Default::default()
        }
    }
    pub fn make_window_settings() -> iced::window::Settings {
        iced::window::Settings {
            resizable: true,
            decorations: true, // TODO other way
            fullscreen: true,
            ..Default::default()
        }
    }
    pub fn new() -> (Self, Task<GameMessage>) {
        let loaded_settings = Self::load_settings();
        (

            Self {
                os_theme: crate::OS_COLOR_THEME.get().unwrap_or(&crate::game::OSColorTheme::Light).clone(),
                game_state: crate::game::GameState::new(),
                new_game_player_name: loaded_settings.last_username,
                new_game_game_template: None,
                new_game_selected_description: None,
                continue_game_game_choice: None,
                continue_game_delete_confirmation: None,
                settings_game_save_folder: loaded_settings.game_save_folder,
                settings_difficulty_level: loaded_settings.difficulty_level,
                settings_autosave: loaded_settings.autosave,
                settings_language: loaded_settings.language,
                current_crisis: None,
                story_state: None,
                choice_text_inputs: std::collections::HashMap::new(),
            },
            Task::batch([
                widget::focus_next(),
            ]),
        )
    }

    pub fn update(&mut self, message: GameMessage) -> Task<GameMessage> {
        #[cfg(not(target_arch = "wasm32"))]
        let start_time = std::time::Instant::now();
        
        let result = match message {
            GameMessage::Menu_NewGameRequested => {
                if let Ok(mut evt_loop_wguard) = self.game_state.active_event_loop.write() {
                    *evt_loop_wguard = crate::game::ActiveEventLoop::WelcomeScreen(crate::game::WelcomeScreenView::NewGame);
                }
                Task::none()
            }
            GameMessage::Menu_NewGamePlayerNameAltered(name) => {
                self.new_game_player_name = name;
                self.save_settings();
                Task::none()
            }
            GameMessage::Menu_NewGameTemplateChoiceAltered(game_template) => {
                // Convert display name to template name (folder path)
                let template_name = crate::crisis::get_template_name_from_display_name(&game_template);
                self.new_game_game_template = Some(template_name);
                
                // Get the description for the selected crisis
                if let Some((_, description)) = crate::crisis::get_crisis_info_by_display_name(&game_template, &self.settings_language) {
                    self.new_game_selected_description = Some(description);
                } else {
                    self.new_game_selected_description = None;
                }
                
                Task::none()
            }
            GameMessage::Menu_NewGameStartClicked => {
                let verbosity = crate::VERBOSITY.get().unwrap_or(&0);
                if *verbosity > 0 {
                    eprintln!("[VERBOSE] Menu_NewGameStartClicked: template_name={:?}", self.new_game_game_template);
                }
                
                if let Some(ref template_name) = self.new_game_game_template {
                    match crate::crisis::load_crisis(template_name) {
                        Ok(crisis) => {
                            if *verbosity > 0 {
                                eprintln!("[VERBOSE] Menu_NewGameStartClicked: Crisis loaded successfully");
                            }
                            
                            let character_name = if self.new_game_player_name.is_empty() {
                                crate::crisis::get_random_character_name(&crisis, None, &self.settings_language)
                            } else {
                                self.new_game_player_name.clone()
                            };
                            
                            if *verbosity > 0 {
                                eprintln!("[VERBOSE] Menu_NewGameStartClicked: Character name: {}", character_name);
                            }
                            
                            let user_language = self.settings_language.clone();
                            let mut story_state = crate::crisis::GameState::new(
                                crisis.metadata.id.clone(),
                                user_language.clone(),
                                template_name.clone(),
                            );
                            story_state.current_scene = crisis.story.starting_scene.clone();
                            story_state.character_name = character_name;
                            
                            if *verbosity > 0 {
                                eprintln!("[VERBOSE] Menu_NewGameStartClicked: Starting scene: {}", story_state.current_scene);
                            }
                            
                            self.current_crisis = Some(crisis);
                            self.story_state = Some(story_state);
                            
                            if let Ok(mut evt_loop_wguard) = self.game_state.active_event_loop.write() {
                                *evt_loop_wguard = crate::game::ActiveEventLoop::ActiveGame(crate::game::GameView::StoryScene);
                                if *verbosity > 0 {
                                    eprintln!("[VERBOSE] Menu_NewGameStartClicked: Successfully switched to ActiveGame state");
                                }
                            } else {
                                if *verbosity > 0 {
                                    eprintln!("[VERBOSE] Menu_NewGameStartClicked: Failed to acquire event loop write lock");
                                }
                            }
                        }
                        Err(e) => {
                            if *verbosity > 0 {
                                eprintln!("[VERBOSE] Menu_NewGameStartClicked: Failed to load crisis: {}", e);
                            }
                        }
                    }
                } else {
                    if *verbosity > 0 {
                        eprintln!("[VERBOSE] Menu_NewGameStartClicked: No template selected");
                    }
                }
                Task::none()
            }

            GameMessage::Menu_ContinueGameRequested => {
                if let Ok(mut evt_loop_wguard) = self.game_state.active_event_loop.write() {
                    *evt_loop_wguard = crate::game::ActiveEventLoop::WelcomeScreen(crate::game::WelcomeScreenView::ContinueGame);
                }
                Task::none()
            }
            GameMessage::Menu_ContinueGameChoiceAltered(saved_game_name) => {
                self.continue_game_game_choice = Some(saved_game_name);
                Task::none()
            }
            GameMessage::Menu_ContinueGameStartClicked => {
                let verbosity = crate::VERBOSITY.get().unwrap_or(&0);
                if *verbosity > 0 {
                    eprintln!("[VERBOSE] Menu_ContinueGameStartClicked: saved_game={:?}", self.continue_game_game_choice);
                }
                
                if let Some(ref saved_game_name) = self.continue_game_game_choice {
                    match crate::crisis::load_saved_game(saved_game_name) {
                        Ok(loaded_story_state) => {
                            if *verbosity > 0 {
                                eprintln!("[VERBOSE] Loaded saved game: {}", saved_game_name);
                                eprintln!("[VERBOSE] Crisis: {}, Scene: {}, Character: {}", 
                                    loaded_story_state.crisis_id,
                                    loaded_story_state.current_scene,
                                    loaded_story_state.character_name);
                            }
                            
                            // Load the crisis definition using template_name
                            match crate::crisis::load_crisis(&loaded_story_state.template_name) {
                                Ok(crisis) => {
                                    if *verbosity > 0 {
                                        eprintln!("[VERBOSE] Crisis loaded successfully");
                                    }
                                    
                                    // Set up the game state
                                    self.current_crisis = Some(crisis);
                                    self.story_state = Some(loaded_story_state);
                                    
                                    // Switch to game view
                                    if let Ok(mut evt_loop_wguard) = self.game_state.active_event_loop.write() {
                                        *evt_loop_wguard = crate::game::ActiveEventLoop::ActiveGame(crate::game::GameView::StoryScene);
                                        if *verbosity > 0 {
                                            eprintln!("[VERBOSE] Successfully switched to loaded game");
                                        }
                                    } else {
                                        if *verbosity > 0 {
                                            eprintln!("[VERBOSE] Failed to acquire event loop write lock");
                                        }
                                    }
                                }
                                Err(e) => {
                                    if *verbosity > 0 {
                                        eprintln!("[VERBOSE] Failed to load crisis '{}': {}", loaded_story_state.crisis_id, e);
                                    }
                                }
                            }
                        }
                        Err(e) => {
                            if *verbosity > 0 {
                                eprintln!("[VERBOSE] Failed to load saved game '{}': {}", saved_game_name, e);
                            }
                        }
                    }
                } else {
                    if *verbosity > 0 {
                        eprintln!("[VERBOSE] No saved game selected");
                    }
                }
                
                Task::none()
            }
            GameMessage::Menu_ContinueGameDeleteRequested(game_name) => {
                let verbosity = crate::VERBOSITY.get().unwrap_or(&0);
                if *verbosity > 0 {
                    eprintln!("[VERBOSE] Menu_ContinueGameDeleteRequested: game_name={:?}", game_name);
                }
                
                if game_name.is_empty() {
                    // Cancel deletion
                    self.continue_game_delete_confirmation = None;
                } else {
                    // Show confirmation dialog
                    self.continue_game_delete_confirmation = Some(game_name);
                }
                Task::none()
            }
            GameMessage::Menu_ContinueGameDeleteConfirmed(game_name) => {
                let verbosity = crate::VERBOSITY.get().unwrap_or(&0);
                if *verbosity > 0 {
                    eprintln!("[VERBOSE] Menu_ContinueGameDeleteConfirmed: game_name={:?}", game_name);
                }
                
                // Perform the deletion
                match crate::crisis::delete_saved_game(&game_name) {
                    Ok(()) => {
                        if *verbosity > 0 {
                            eprintln!("[VERBOSE] Successfully deleted saved game: {}", game_name);
                        }
                        // Clear the selected game if it was the one being deleted
                        if let Some(ref selected_game) = self.continue_game_game_choice {
                            if selected_game == &game_name {
                                self.continue_game_game_choice = None;
                            }
                        }
                    }
                    Err(e) => {
                        if *verbosity > 0 {
                            eprintln!("[VERBOSE] Failed to delete saved game '{}': {}", game_name, e);
                        }
                    }
                }
                
                // Clear confirmation dialog
                self.continue_game_delete_confirmation = None;
                Task::none()
            }

            GameMessage::Menu_SettingsRequested => {
                if let Ok(mut evt_loop_wguard) = self.game_state.active_event_loop.write() {
                    *evt_loop_wguard = crate::game::ActiveEventLoop::WelcomeScreen(crate::game::WelcomeScreenView::Settings);
                }
                Task::none()
            }
            GameMessage::Menu_SettingsGameSaveFolderChanged(folder_path) => {
                eprintln!("Settings: Game Save Folder changed to: {}", folder_path);
                self.settings_game_save_folder = folder_path;
                self.save_settings();
                Task::none()
            }
            GameMessage::Menu_SettingsDifficultyLevelChanged(difficulty) => {
                eprintln!("Settings: Difficulty Level changed to: {:?}", difficulty);
                self.settings_difficulty_level = difficulty;
                self.save_settings();
                Task::none()
            }
            GameMessage::Menu_SettingsAutosaveToggled(enabled) => {
                eprintln!("Settings: Autosave toggled to: {}", enabled);
                self.settings_autosave = enabled;
                self.save_settings();
                Task::none()
            }
            GameMessage::Menu_SettingsLanguageChanged(language) => {
                eprintln!("Settings: Language changed to: {}", language);
                self.settings_language = language;
                self.save_settings();
                Task::none()
            }
            GameMessage::QuitGameRequested => {
                // TODO and game-save requirements should go here
                crate::quit_game_gui()
            }
            GameMessage::Game_ChoiceSelected(choice_index) => {
                if let (Some(crisis), Some(story_state)) = (&self.current_crisis, &mut self.story_state) {
                    if let Some(current_scene) = crisis.scenes.get(&story_state.current_scene) {
                        if let Some(choice) = current_scene.choices.get(choice_index) {
                            // Apply choice effects
                            if let Some(ref choice_effects) = crisis.conditions.choice_effects {
                                if let Some(effects) = choice_effects.get(&choice.leads_to) {
                                    for (var, value) in effects {
                                        *story_state.variables.entry(var.clone()).or_insert(0) += value;
                                    }
                                }
                            }
                            
                            // Move to next scene
                            story_state.current_scene = choice.leads_to.clone();
                            
                            // Handle character type selection
                            if let Some(ref char_type) = choice.character_type {
                                story_state.character_type = Some(char_type.clone());
                                story_state.character_name = crate::crisis::get_random_character_name(
                                    crisis, 
                                    Some(char_type), 
                                    &story_state.language
                                );
                            }
                        }
                    }
                }
                Task::none()
            }
            GameMessage::Game_TextInputChanged(choice_index, value) => {
                self.choice_text_inputs.insert(choice_index, value);
                Task::none()
            }
            GameMessage::Game_TextInputSubmitted(choice_index, value) => {
                let verbosity = crate::VERBOSITY.get().unwrap_or(&0);
                
                if let (Some(crisis), Some(story_state)) = (&self.current_crisis, &mut self.story_state) {
                    if let Some(current_scene) = crisis.scenes.get(&story_state.current_scene) {
                        if let Some(choice) = current_scene.choices.get(choice_index) {
                            if let Some(ref text_input) = choice.text_input {
                                // Validate and store the input
                                let validated_value = match text_input.input_type {
                                    crate::crisis::TextInputType::Text => {
                                        if let Some(min_len) = text_input.min_length {
                                            if value.len() < min_len {
                                                return Task::none(); // Don't proceed if validation fails
                                            }
                                        }
                                        if let Some(max_len) = text_input.max_length {
                                            if value.len() > max_len {
                                                return Task::none(); // Don't proceed if validation fails
                                            }
                                        }
                                        value.clone()
                                    }
                                    crate::crisis::TextInputType::Number => {
                                        if let Ok(num) = value.parse::<i32>() {
                                            if let Some(min_val) = text_input.min_value {
                                                if num < min_val {
                                                    return Task::none();
                                                }
                                            }
                                            if let Some(max_val) = text_input.max_value {
                                                if num > max_val {
                                                    return Task::none();
                                                }
                                            }
                                            value.clone()
                                        } else {
                                            return Task::none(); // Invalid number
                                        }
                                    }
                                };
                                
                                // Store the text input in the game state
                                story_state.text_inputs.insert(text_input.variable_name.clone(), validated_value);
                                
                                if *verbosity > 0 {
                                    eprintln!("[VERBOSE] Text input submitted: {} = {}", text_input.variable_name, value);
                                }
                            }
                            
                            // Apply choice effects (same as regular choice)
                            if let Some(ref choice_effects) = crisis.conditions.choice_effects {
                                if let Some(effects) = choice_effects.get(&choice.leads_to) {
                                    for (var, value) in effects {
                                        *story_state.variables.entry(var.clone()).or_insert(0) += value;
                                    }
                                }
                            }
                            
                            // Move to next scene
                            story_state.current_scene = choice.leads_to.clone();
                            
                            // Handle character type selection
                            if let Some(ref char_type) = choice.character_type {
                                story_state.character_type = Some(char_type.clone());
                                story_state.character_name = crate::crisis::get_random_character_name(
                                    crisis, 
                                    Some(char_type), 
                                    &story_state.language
                                );
                            }
                            
                            // Clear text inputs for the new scene
                            self.choice_text_inputs.clear();
                        }
                    }
                }
                
                Task::none()
            }
            GameMessage::Game_RestartRequested => {
                if let Ok(mut evt_loop_wguard) = self.game_state.active_event_loop.write() {
                    *evt_loop_wguard = crate::game::ActiveEventLoop::WelcomeScreen(crate::game::WelcomeScreenView::Empty);
                }
                self.current_crisis = None;
                self.story_state = None;
                // Clear new game selections when returning to menu
                self.new_game_game_template = None;
                self.new_game_selected_description = None;
                Task::none()
            }
            GameMessage::Game_SaveAndQuitRequested => {
                let verbosity = crate::VERBOSITY.get().unwrap_or(&0);
                if *verbosity > 0 {
                    eprintln!("[VERBOSE] Game_SaveAndQuitRequested");
                }
                
                if let (Some(story_state), Some(_crisis)) = (&self.story_state, &self.current_crisis) {
                    // Use the template_name from the GameState
                    match crate::crisis::save_current_game(story_state, &story_state.template_name, None) {
                        Ok(save_name) => {
                            if *verbosity > 0 {
                                eprintln!("[VERBOSE] Game saved as: {}", save_name);
                            }
                        }
                        Err(e) => {
                            if *verbosity > 0 {
                                eprintln!("[VERBOSE] Failed to save game: {}", e);
                            }
                        }
                    }
                }
                
                // Return to main menu
                if let Ok(mut evt_loop_wguard) = self.game_state.active_event_loop.write() {
                    *evt_loop_wguard = crate::game::ActiveEventLoop::WelcomeScreen(crate::game::WelcomeScreenView::Empty);
                }
                
                // Clear game state
                self.current_crisis = None;
                self.story_state = None;
                self.new_game_game_template = None;
                self.new_game_selected_description = None;
                self.choice_text_inputs.clear();
                
                Task::none()
            }
            GameMessage::Game_QuitWithoutSaveRequested => {
                let verbosity = crate::VERBOSITY.get().unwrap_or(&0);
                if *verbosity > 0 {
                    eprintln!("[VERBOSE] Game_QuitWithoutSaveRequested");
                }
                
                // Return to main menu without saving
                if let Ok(mut evt_loop_wguard) = self.game_state.active_event_loop.write() {
                    *evt_loop_wguard = crate::game::ActiveEventLoop::WelcomeScreen(crate::game::WelcomeScreenView::Empty);
                }
                
                // Clear game state
                self.current_crisis = None;
                self.story_state = None;
                self.new_game_game_template = None;
                self.new_game_selected_description = None;
                self.choice_text_inputs.clear();
                
                Task::none()
            }
            GameMessage::Nop => {
                eprintln!("Recieved a GameMessage::Nop");
                Task::none()
            },
            _ => Task::none(),
        };
        
        #[cfg(not(target_arch = "wasm32"))]
        {
            let elapsed = start_time.elapsed();
            let verbosity = crate::VERBOSITY.get().unwrap_or(&0);
            if *verbosity >= 3 {
                // -vvv: Show all timings
                eprintln!("[TIMING] GameWindow::update() took {:?}", elapsed);
            } else if *verbosity >= 2 && elapsed.as_millis() > 10 {
                // -vv: Show only timings > 10ms
                eprintln!("[TIMING] GameWindow::update() took {:?} (>10ms)", elapsed);
            }
        }
        
        result
    }

    pub fn view(&self) -> Element<'_, GameMessage> {
        #[cfg(not(target_arch = "wasm32"))]
        let start_time = std::time::Instant::now();
        
        let result = if let Ok(evt_loop_rguard) = self.game_state.active_event_loop.read() {
            match evt_loop_rguard.clone() {
                crate::game::ActiveEventLoop::WelcomeScreen(_welcome_screen_state) => {
                    self.view_menu_screen()
                }
                crate::game::ActiveEventLoop::ActiveGame(_game_view_state) => {
                    self.view_game_screen()
                }
                crate::game::ActiveEventLoop::Exit => {
                    text("Exiting...").into()
                }
            }
        }
        else {
            text("Error, cannot read game_state.active_event_loop").into()
        };
        
        #[cfg(not(target_arch = "wasm32"))]
        {
            let elapsed = start_time.elapsed();
            let verbosity = crate::VERBOSITY.get().unwrap_or(&0);
            if *verbosity >= 3 {
                // -vvv: Show all timings
                eprintln!("[TIMING] GameWindow::view() took {:?}", elapsed);
            } else if *verbosity >= 2 && elapsed.as_millis() > 10 {
                // -vv: Show only timings > 10ms
                eprintln!("[TIMING] GameWindow::view() took {:?} (>10ms)", elapsed);
            }
        }
        
        result
    }

    pub fn theme(&self) -> Theme {
        if self.os_theme == crate::game::OSColorTheme::Dark {
            Theme::Dark
        }
        else {
            Theme::Light
        }
    }

    pub fn view_menu_screen(&self) -> Element<'_, GameMessage> {
        let splash_handle = iced::widget::image::Handle::from_bytes(SPLASH_PNG_BYTES);
        let splash_img = Image::<iced::widget::image::Handle>::new(splash_handle)
            .width(Length::Fill)
            .height(Length::Fill);
        let background = Container::<GameMessage, Theme, iced::Renderer>::new(splash_img)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x(Length::Fill)
            .center_y(Length::Fill);

        let app_version = env!("CARGO_PKG_VERSION");

        let user_language = &self.settings_language;
        let buttons = column![
            button(text(crate::translations::t(crate::translations::TranslationKey::ContinueGame, user_language)))
                .on_press(GameMessage::Menu_ContinueGameRequested)
                .width(Length::Fill),
            button(text(crate::translations::t(crate::translations::TranslationKey::NewGame, user_language)))
                .on_press(GameMessage::Menu_NewGameRequested)
                .width(Length::Fill),
            button(text(crate::translations::t(crate::translations::TranslationKey::Settings, user_language)))
                .on_press(GameMessage::Menu_SettingsRequested)
                .width(Length::Fill),
            button(text(crate::translations::t(crate::translations::TranslationKey::QuitGame, user_language)))
                .on_press(GameMessage::QuitGameRequested)
                .width(Length::Fill),
            text(format!("Version {}", app_version))
                .width(Length::Fill),
        ]
        .spacing(10)
        .width(240)
        .padding(10)
        .align_x(Center)
        .width(Length::Fixed(186.0f32));

        let right_panel = container(self.build_menu_screen_right_ui())
            .width(Length::Fixed(760.0f32))
            .align_x(Center)
            .center_y(iced::Length::Shrink);


        let foreground_content = row![buttons, right_panel]
            .height(Length::Fill)
            .width(Length::Shrink);

        let foreground_content = Column::new()
            .height(Length::Fill)
            .width(Length::Shrink)
            .align_x(iced::alignment::Horizontal::Center)
            .push(Space::with_height(Length::Fill))
            .push(
                foreground_content
            )
            .push(Space::with_height(Length::Fill));

        let foreground_content = Row::new()
            .height(Length::Fill)
            .width(Length::Shrink)
            .align_y(iced::alignment::Vertical::Center)
            .push(Space::with_height(Length::Fill))
            .push(
                foreground_content
            )
            .push(Space::with_height(Length::Fill));


        iced::widget::stack![
            background,
            Container::new(foreground_content)
                .width(Length::Fill)
                .height(Length::Fill)
                .align_x(Center)
                .align_y(Center)
        ]
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
    }





    pub fn view_game_screen(&self) -> Element<'_, GameMessage> {
        if let (Some(crisis), Some(story_state)) = (&self.current_crisis, &self.story_state) {
            self.render_story_scene(crisis, story_state)
        } else {
            container(
                column![
                    text(crate::translations::t(crate::translations::TranslationKey::LoadingCrisis, &self.settings_language)).size(20),
                    button(text(crate::translations::t(crate::translations::TranslationKey::ReturnToMenu, &self.settings_language)))
                        .on_press(GameMessage::Game_RestartRequested)
                        .padding(10)
                ]
                .spacing(20)
                .align_x(Center)
            )
            .center_x(Length::Fill)
            .center_y(Length::Fill)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
        }
    }
    
    fn render_story_scene(&self, crisis: &crate::crisis::CrisisDefinition, story_state: &crate::crisis::GameState) -> Element<'_, GameMessage> {
        if let Some(current_scene) = crisis.scenes.get(&story_state.current_scene) {
            // Track image loading errors
            let mut error_messages = Vec::new();
            
            // Create background image if specified
            let background_layer = if let Some(ref bg_path) = current_scene.background_image {
                if let Some(bg_file) = crate::crisis::PlayableCrises::get(bg_path) {
                    let bg_handle = iced::widget::image::Handle::from_bytes(bg_file.data.as_ref().to_vec());
                    let bg_img = Image::<iced::widget::image::Handle>::new(bg_handle)
                        .width(Length::Fill)
                        .height(Length::Fill)
                        .content_fit(iced::ContentFit::Cover);
                    
                    // Add semi-transparent overlay for better text readability
                    let overlay = Container::<GameMessage, Theme, iced::Renderer>::new(
                        iced::widget::Space::with_width(Length::Fill)
                    )
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .style(|_theme: &Theme| {
                        iced::widget::container::Style {
                            background: Some(iced::Color::from_rgba(0.0, 0.0, 0.0, 0.3).into()),
                            ..iced::widget::container::Style::default()
                        }
                    });
                    
                    Some(iced::widget::stack![
                        Container::<GameMessage, Theme, iced::Renderer>::new(bg_img)
                            .width(Length::Fill)
                            .height(Length::Fill),
                        overlay
                    ]
                    .width(Length::Fill)
                    .height(Length::Fill))
                } else {
                    error_messages.push(format!("Scene '{}': Background image file not found: {}", story_state.current_scene, bg_path));
                    None
                }
            } else {
                error_messages.push(format!("Scene '{}': No background_image defined", story_state.current_scene));
                None
            };

            // Game data at center-top
            let title = crate::crisis::get_localized_text(&crisis.name, &story_state.language);
            let mut vars = std::collections::HashMap::new();
            vars.insert("character_name".to_string(), story_state.character_name.clone());
            let character_info = text(
                crate::translations::t_vars(crate::translations::TranslationKey::PlayingAs, &story_state.language, &vars)
            )
                .size(16)
                .color(iced::Color::from_rgb(0.6, 0.6, 0.6));
            
            let mut variables_text = String::new();
            if !story_state.variables.is_empty() {
                variables_text = format!("Variables: {:?}", story_state.variables);
            }
            
            // Save and Quit buttons for upper-left
            let save_button = button(
                    text(crate::translations::t(crate::translations::TranslationKey::SaveAndQuit, &story_state.language))
                        .align_x(Center)
                )
                .on_press(GameMessage::Game_SaveAndQuitRequested)
                .padding(8)
                .width(Length::Fixed(140.0));
                
            let quit_button = button(
                    text(crate::translations::t(crate::translations::TranslationKey::Quit, &story_state.language))
                        .align_x(Center)
                )
                .on_press(GameMessage::Game_QuitWithoutSaveRequested)
                .padding(8)
                .width(Length::Fixed(60.0));
                
            let control_buttons = row![save_button, quit_button]
                .spacing(10);

            // Top row with buttons on left and title/info in center
            let top_row = row![
                container(control_buttons)
                    .align_x(iced::alignment::Horizontal::Left),
                container(
                    column![
                        text(title.clone()).size(24).align_x(Center),
                        character_info.align_x(Center),
                        if !variables_text.is_empty() {
                            text(variables_text.clone()).size(12).color(iced::Color::from_rgb(0.5, 0.5, 0.5))
                        } else {
                            text("")
                        }
                    ]
                    .spacing(5)
                    .align_x(Center)
                )
                .width(Length::Fill)
                .align_x(Center),
                // Right side spacer to balance the layout
                container(Space::with_width(Length::Fixed(190.0)))
            ]
            .width(Length::Fill)
            .align_y(iced::alignment::Vertical::Top);
            
            let top_data = container(top_row)
                .width(Length::Fill)
                .padding(20);

            // Story text and choices in left column (60% width)
            let scene_text = crate::crisis::get_scene_text_with_substitutions(current_scene, &story_state.language, &story_state.character_name, &story_state.text_inputs);
            let story_text_display = container(
                container(
                    text(scene_text.clone())
                        .size(18)
                        .wrapping(iced::widget::text::Wrapping::Word)
                )
                .padding(20)
                .style(move |theme: &Theme| {
                    let palette = theme.extended_palette();
                    iced::widget::container::Style {
                        background: Some(palette.background.weak.color.into()),
                        border: iced::border::rounded(8)
                            .color(palette.primary.weak.color)
                            .width(1),
                        ..iced::widget::container::Style::default()
                    }
                })
            )
            .width(Length::Fill)
            .padding(10);

            // User choices below story text
            let mut choices_column = column![].spacing(10);
            
            if current_scene.choices.is_empty() {
                choices_column = choices_column.push(
                    column![
                        text(crate::translations::t(crate::translations::TranslationKey::End, &story_state.language)).size(20),
                        button(text(crate::translations::t(crate::translations::TranslationKey::ReturnToMenu, &story_state.language)))
                            .on_press(GameMessage::Game_RestartRequested)
                            .padding(10)
                            .width(Length::Fill)
                    ]
                    .spacing(10)
                );
            } else {
                choices_column = choices_column.push(
                    text(crate::translations::t(crate::translations::TranslationKey::WhatDoYouChoose, &story_state.language))
                        .size(16)
                );
                
                for (index, choice) in current_scene.choices.iter().enumerate() {
                    let choice_text = crate::crisis::get_localized_text(&choice.text, &story_state.language);
                    
                    let mut available = true;
                    if let Some(ref requirements) = choice.requires {
                        for (var, required_value) in requirements {
                            if let Some(current_value) = story_state.variables.get(var) {
                                if current_value < required_value {
                                    available = false;
                                    break;
                                }
                            } else {
                                available = false;
                                break;
                            }
                        }
                    }
                    
                    // Check if this choice has text input
                    if let Some(ref text_input) = choice.text_input {
                        let placeholder_text = if let Some(ref placeholder) = text_input.placeholder {
                            crate::crisis::get_localized_text(placeholder, &story_state.language)
                        } else {
                            match text_input.input_type {
                                crate::crisis::TextInputType::Text => "Enter text...".to_string(),
                                crate::crisis::TextInputType::Number => "Enter number...".to_string(),
                            }
                        };
                        
                        let input_value = self.choice_text_inputs.get(&index).cloned().unwrap_or_default();
                        
                        let text_input_widget = iced::widget::text_input(&placeholder_text, &input_value)
                            .on_input(move |value| GameMessage::Game_TextInputChanged(index, value))
                            .on_submit(GameMessage::Game_TextInputSubmitted(index, input_value.clone()))
                            .padding(8)
                            .width(Length::Fill);
                        
                        let submit_button = if available && !input_value.is_empty() {
                            button(text(choice_text.clone()))
                                .on_press(GameMessage::Game_TextInputSubmitted(index, input_value.clone()))
                                .padding(10)
                                .width(Length::Fixed(120.0))
                        } else {
                            button(text(choice_text.clone()))
                                .padding(10)
                                .width(Length::Fixed(120.0))
                                .style(move |theme: &Theme, _status| {
                                    let palette = theme.extended_palette();
                                    iced::widget::button::Style {
                                        background: Some(palette.background.weak.color.into()),
                                        text_color: palette.background.strong.text,
                                        border: iced::border::rounded(4)
                                            .color(palette.background.strong.color)
                                            .width(1),
                                        ..iced::widget::button::Style::default()
                                    }
                                })
                        };
                        
                        let input_row = row![text_input_widget, submit_button]
                            .spacing(10)
                            .align_y(iced::alignment::Vertical::Center);
                        
                        choices_column = choices_column.push(input_row);
                    } else {
                        // Regular choice button (existing behavior)
                        let choice_button = if available {
                            button(text(choice_text.clone()))
                                .on_press(GameMessage::Game_ChoiceSelected(index))
                                .padding(10)
                                .width(Length::Fill)
                        } else {
                            button(text(format!("{} {}", choice_text, 
                                crate::translations::t(crate::translations::TranslationKey::RequirementsNotMet, &story_state.language))))
                                .padding(10)
                                .width(Length::Fill)
                                .style(move |theme: &Theme, _status| {
                                    let palette = theme.extended_palette();
                                    iced::widget::button::Style {
                                        background: Some(palette.background.weak.color.into()),
                                        text_color: palette.background.strong.text,
                                        border: iced::border::rounded(4)
                                            .color(palette.background.strong.color)
                                            .width(1),
                                        ..iced::widget::button::Style::default()
                                    }
                                })
                        };
                        
                        choices_column = choices_column.push(choice_button);
                    }
                }
            }

            // Left column: story text above choices (60% width), anchored to bottom
            let left_column = column![
                Space::with_height(Length::Fill), // Expanding spacer to push content to bottom
                story_text_display,
                container(choices_column).padding(20)
            ]
            .spacing(10)
            .width(Length::FillPortion(60))
            .height(Length::Fill);

            // Speaking character at right (40% width), anchored to bottom
            let character_display = if let Some(ref char_path) = current_scene.speaking_character_image {
                if let Some(char_file) = crate::crisis::PlayableCrises::get(char_path) {
                    let char_handle = iced::widget::image::Handle::from_bytes(char_file.data.as_ref().to_vec());
                    let char_img = Image::<iced::widget::image::Handle>::new(char_handle)
                        .width(Length::Fill)
                        .height(Length::Fixed(300.0));
                    container(char_img)
                        .width(Length::Fill)
                        .padding(20)
                        .align_x(Center)
                } else {
                    error_messages.push(format!("Scene '{}': Character image file not found: {}", story_state.current_scene, char_path));
                    container(Space::with_width(Length::Fill))
                }
            } else {
                error_messages.push(format!("Scene '{}': No speaking_character_image defined", story_state.current_scene));
                container(Space::with_width(Length::Fill))
            };

            let right_column = column![
                Space::with_height(Length::Fill), // Expanding spacer to push character to bottom
                character_display
            ]
            .width(Length::FillPortion(40))
            .height(Length::Fill);

            // Bottom row with left column (story+choices) and right column (character)
            let bottom_row = row![left_column, right_column]
                .width(Length::Fill)
                .height(Length::Fill);

            // Create main layout with optional error display
            let main_layout = if !error_messages.is_empty() {
                let error_text = error_messages.join("; ");
                let error_display = container(
                    text(format!("Image Loading Errors: {}", error_text))
                        .size(12)
                        .color(iced::Color::from_rgb(0.8, 0.2, 0.2))
                        .wrapping(iced::widget::text::Wrapping::Word)
                )
                .width(Length::Fill)
                .padding(10)
                .style(move |_theme: &Theme| {
                    iced::widget::container::Style {
                        background: Some(iced::Color::from_rgba(0.8, 0.2, 0.2, 0.1).into()),
                        border: iced::border::rounded(4)
                            .color(iced::Color::from_rgb(0.8, 0.2, 0.2))
                            .width(1),
                        ..iced::widget::container::Style::default()
                    }
                });

                column![
                    top_data,
                    bottom_row,
                    error_display
                ]
                .width(Length::Fill)
                .height(Length::Fill)
            } else {
                column![
                    top_data,
                    bottom_row
                ]
                .width(Length::Fill)
                .height(Length::Fill)
            };

            // Stack background and foreground if background exists
            if let Some(background) = background_layer {
                iced::widget::stack![
                    background,
                    Container::new(main_layout)
                        .width(Length::Fill)
                        .height(Length::Fill)
                ]
                .width(Length::Fill)
                .height(Length::Fill)
                .into()
            } else {
                Container::new(main_layout)
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .into()
            }
        } else {
            container(
                column![
                    text(format!("{} '{}' {}", 
                        crate::translations::t(crate::translations::TranslationKey::SceneNotFound, &story_state.language).replace("!", ""),
                        story_state.current_scene,
                        "!"
                    )).size(20),
                    button(text(crate::translations::t(crate::translations::TranslationKey::ReturnToMenu, &story_state.language)))
                        .on_press(GameMessage::Game_RestartRequested)
                        .padding(10)
                ]
                .spacing(20)
                .align_x(Center)
            )
            .center_x(Length::Fill)
            .center_y(Length::Fill)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
        }
    }


}


pub fn menu_right_box_style(theme: &Theme) -> iced::widget::container::Style {
    let palette = theme.extended_palette();

    iced::widget::container::Style {
        background: Some(palette.background.weak.color.into()),
        border: iced::border::rounded(12)
            .color(palette.primary.weak.color)
            .width(2),
        ..iced::widget::container::Style::default()
    }
}




