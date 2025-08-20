use super::types::*;
use iced::{Task, Element};

impl GameWindow {
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
                let template_name = crate::crisis::get_template_name_from_display_name(&game_template);
                self.new_game_game_template = Some(template_name);
                
                if let Some((_, description)) = crate::crisis::get_crisis_info_by_display_name(&game_template, &self.settings_language) {
                    self.new_game_selected_description = Some(description);
                } else {
                    self.new_game_selected_description = None;
                }
                
                Task::none()
            }
            GameMessage::Menu_NewGameStartClicked => {
                self.handle_new_game_start()
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
                self.handle_continue_game_start()
            }
            GameMessage::Menu_ContinueGameDeleteRequested(game_name) => {
                self.handle_delete_game_request(game_name)
            }
            GameMessage::Menu_ContinueGameDeleteConfirmed(game_name) => {
                self.handle_delete_game_confirmed(game_name)
            }
            GameMessage::Menu_SettingsRequested => {
                if let Ok(mut evt_loop_wguard) = self.game_state.active_event_loop.write() {
                    *evt_loop_wguard = crate::game::ActiveEventLoop::WelcomeScreen(crate::game::WelcomeScreenView::Settings);
                }
                Task::none()
            }
            GameMessage::Menu_LicensesRequested => {
                if let Ok(mut evt_loop_wguard) = self.game_state.active_event_loop.write() {
                    *evt_loop_wguard = crate::game::ActiveEventLoop::WelcomeScreen(crate::game::WelcomeScreenView::Licenses);
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
                crate::quit_game_gui()
            }
            GameMessage::Game_ChoiceSelected(choice_index) => {
                self.handle_choice_selection(choice_index)
            }
            GameMessage::Game_TextInputChanged(choice_index, value) => {
                self.choice_text_inputs.insert(choice_index, value);
                Task::none()
            }
            GameMessage::Game_TextInputSubmitted(choice_index, value) => {
                self.handle_text_input_submission(choice_index, value)
            }
            GameMessage::Game_RestartRequested => {
                self.handle_game_restart()
            }
            GameMessage::Game_SaveAndQuitRequested => {
                self.handle_save_and_quit()
            }
            GameMessage::Game_QuitWithoutSaveRequested => {
                self.handle_quit_without_save()
            }
            GameMessage::Game_AnimationTick => {
                self.animation_frame_index = self.animation_frame_index.wrapping_add(1);
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
                eprintln!("[TIMING] GameWindow::update() took {:?}", elapsed);
            } else if *verbosity >= 2 && elapsed.as_millis() > 10 {
                eprintln!("[TIMING] GameWindow::update() took {:?} (>10ms)", elapsed);
            }
        }
        
        result
    }

    fn handle_new_game_start(&mut self) -> Task<GameMessage> {
        let verbosity = crate::VERBOSITY.get().unwrap_or(&0);
        if *verbosity > 0 {
            eprintln!("Menu_NewGameStartClicked: template_name={:?}", self.new_game_game_template);
        }
        
        if let Some(ref template_name) = self.new_game_game_template {
            match crate::crisis::load_crisis(template_name) {
                Ok(crisis) => {
                    if *verbosity > 0 {
                        eprintln!("Menu_NewGameStartClicked: Crisis loaded successfully");
                    }
                    
                    let character_name = if self.new_game_player_name.is_empty() {
                        crate::crisis::get_random_character_name(&crisis, None, &self.settings_language)
                    } else {
                        self.new_game_player_name.clone()
                    };
                    
                    let user_language = self.settings_language.clone();
                    let mut story_state = crate::crisis::GameState::new(
                        crisis.metadata.id.clone(),
                        user_language.clone(),
                        template_name.clone(),
                    );
                    story_state.current_scene = crisis.story.starting_scene.clone();
                    story_state.character_name = character_name;
                    
                    self.current_crisis = Some(crisis);
                    self.story_state = Some(story_state);
                    
                    if let Ok(mut evt_loop_wguard) = self.game_state.active_event_loop.write() {
                        *evt_loop_wguard = crate::game::ActiveEventLoop::ActiveGame(crate::game::GameView::StoryScene);
                    }
                }
                Err(e) => {
                    if *verbosity > 0 {
                        eprintln!("Menu_NewGameStartClicked: Failed to load crisis: {}", e);
                    }
                }
            }
        }
        Task::none()
    }

    fn handle_continue_game_start(&mut self) -> Task<GameMessage> {
        let verbosity = crate::VERBOSITY.get().unwrap_or(&0);
        
        if let Some(ref saved_game_name) = self.continue_game_game_choice {
            match crate::crisis::load_saved_game(saved_game_name) {
                Ok(loaded_story_state) => {
                    match crate::crisis::load_crisis(&loaded_story_state.template_name) {
                        Ok(crisis) => {
                            self.current_crisis = Some(crisis);
                            self.story_state = Some(loaded_story_state);
                            
                            if let Ok(mut evt_loop_wguard) = self.game_state.active_event_loop.write() {
                                *evt_loop_wguard = crate::game::ActiveEventLoop::ActiveGame(crate::game::GameView::StoryScene);
                            }
                        }
                        Err(e) => {
                            if *verbosity > 0 {
                                eprintln!("Failed to load crisis: {}", e);
                            }
                        }
                    }
                }
                Err(e) => {
                    if *verbosity > 0 {
                        eprintln!("Failed to load saved game: {}", e);
                    }
                }
            }
        }
        Task::none()
    }

    fn handle_delete_game_request(&mut self, game_name: String) -> Task<GameMessage> {
        if game_name.is_empty() {
            self.continue_game_delete_confirmation = None;
        } else {
            self.continue_game_delete_confirmation = Some(game_name);
        }
        Task::none()
    }

    fn handle_delete_game_confirmed(&mut self, game_name: String) -> Task<GameMessage> {
        let verbosity = crate::VERBOSITY.get().unwrap_or(&0);
        
        match crate::crisis::delete_saved_game(&game_name) {
            Ok(()) => {
                if let Some(ref selected_game) = self.continue_game_game_choice {
                    if selected_game == &game_name {
                        self.continue_game_game_choice = None;
                    }
                }
            }
            Err(e) => {
                if *verbosity > 0 {
                    eprintln!("Failed to delete saved game '{}': {}", game_name, e);
                }
            }
        }
        
        self.continue_game_delete_confirmation = None;
        Task::none()
    }

    fn handle_choice_selection(&mut self, choice_index: usize) -> Task<GameMessage> {
        if let (Some(crisis), Some(story_state)) = (&self.current_crisis, &mut self.story_state) {
            if let Some(current_scene) = crisis.scenes.get(&story_state.current_scene) {
                if let Some(choice) = current_scene.choices.get(choice_index) {
                    if let Some(ref choice_effects) = crisis.conditions.choice_effects {
                        if let Some(effects) = choice_effects.get(&choice.leads_to) {
                            for (var, value) in effects {
                                *story_state.variables.entry(var.clone()).or_insert(0) += value;
                            }
                        }
                    }
                    
                    story_state.current_scene = choice.leads_to.clone();
                    
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

    fn handle_text_input_submission(&mut self, choice_index: usize, value: String) -> Task<GameMessage> {
        if let (Some(crisis), Some(story_state)) = (&self.current_crisis, &mut self.story_state) {
            if let Some(current_scene) = crisis.scenes.get(&story_state.current_scene) {
                if let Some(choice) = current_scene.choices.get(choice_index) {
                    if let Some(ref text_input) = choice.text_input {
                        let validated_value = match text_input.input_type {
                            crate::crisis::TextInputType::Text => {
                                if let Some(min_len) = text_input.min_length {
                                    if value.len() < min_len {
                                        return Task::none();
                                    }
                                }
                                if let Some(max_len) = text_input.max_length {
                                    if value.len() > max_len {
                                        return Task::none();
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
                                    return Task::none();
                                }
                            }
                        };
                        
                        story_state.text_inputs.insert(text_input.variable_name.clone(), validated_value);
                    }
                    
                    if let Some(ref choice_effects) = crisis.conditions.choice_effects {
                        if let Some(effects) = choice_effects.get(&choice.leads_to) {
                            for (var, value) in effects {
                                *story_state.variables.entry(var.clone()).or_insert(0) += value;
                            }
                        }
                    }
                    
                    story_state.current_scene = choice.leads_to.clone();
                    
                    if let Some(ref char_type) = choice.character_type {
                        story_state.character_type = Some(char_type.clone());
                        story_state.character_name = crate::crisis::get_random_character_name(
                            crisis, 
                            Some(char_type), 
                            &story_state.language
                        );
                    }
                    
                    self.choice_text_inputs.clear();
                }
            }
        }
        Task::none()
    }

    fn handle_game_restart(&mut self) -> Task<GameMessage> {
        if let Ok(mut evt_loop_wguard) = self.game_state.active_event_loop.write() {
            *evt_loop_wguard = crate::game::ActiveEventLoop::WelcomeScreen(crate::game::WelcomeScreenView::Empty);
        }
        self.current_crisis = None;
        self.story_state = None;
        self.new_game_game_template = None;
        self.new_game_selected_description = None;
        Task::none()
    }

    fn handle_save_and_quit(&mut self) -> Task<GameMessage> {
        let verbosity = crate::VERBOSITY.get().unwrap_or(&0);
        
        if let (Some(story_state), Some(_crisis)) = (&self.story_state, &self.current_crisis) {
            match crate::crisis::save_current_game(story_state, &story_state.template_name, None) {
                Ok(save_name) => {
                    if *verbosity > 0 {
                        eprintln!("Game saved as: {}", save_name);
                    }
                }
                Err(e) => {
                    if *verbosity > 0 {
                        eprintln!("Failed to save game: {}", e);
                    }
                }
            }
        }
        
        self.clear_game_state();
        Task::none()
    }

    fn handle_quit_without_save(&mut self) -> Task<GameMessage> {
        let verbosity = crate::VERBOSITY.get().unwrap_or(&0);
        if *verbosity > 0 {
            eprintln!("Game_QuitWithoutSaveRequested");
        }
        
        self.clear_game_state();
        Task::none()
    }

    fn clear_game_state(&mut self) {
        if let Ok(mut evt_loop_wguard) = self.game_state.active_event_loop.write() {
            *evt_loop_wguard = crate::game::ActiveEventLoop::WelcomeScreen(crate::game::WelcomeScreenView::Empty);
        }
        
        self.current_crisis = None;
        self.story_state = None;
        self.new_game_game_template = None;
        self.new_game_selected_description = None;
        self.choice_text_inputs.clear();
        self.animation_frame_index = 0;
    }
}