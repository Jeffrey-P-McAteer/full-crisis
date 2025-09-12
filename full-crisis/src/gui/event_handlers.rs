use super::types::*;
use iced::{Task, Element};

#[cfg(target_arch = "wasm32")]
use once_cell::sync::OnceCell;

#[cfg(target_arch = "wasm32")]
static PLAY_BACKGROUND_AUDIO: OnceCell<Box<dyn Fn(&[u8]) + Send + Sync>> = OnceCell::new();

#[cfg(target_arch = "wasm32")]
static STOP_BACKGROUND_AUDIO: OnceCell<Box<dyn Fn() + Send + Sync>> = OnceCell::new();

#[cfg(target_arch = "wasm32")]
pub fn set_audio_callbacks(
    play_fn: Box<dyn Fn(&[u8]) + Send + Sync>,
    stop_fn: Box<dyn Fn() + Send + Sync>,
) {
    let _ = PLAY_BACKGROUND_AUDIO.set(play_fn);
    let _ = STOP_BACKGROUND_AUDIO.set(stop_fn);
}

#[cfg(target_arch = "wasm32")]
pub fn web_play_background_audio(audio_data: &[u8]) {
    if let Some(play_fn) = PLAY_BACKGROUND_AUDIO.get() {
        play_fn(audio_data);
    }
}

#[cfg(target_arch = "wasm32")]
pub fn web_stop_background_audio() {
    if let Some(stop_fn) = STOP_BACKGROUND_AUDIO.get() {
        stop_fn();
    }
}

#[cfg(not(target_arch = "wasm32"))]
use rodio::{Decoder, OutputStream, OutputStreamBuilder, Sink, Source};

#[cfg(not(target_arch = "wasm32"))]
#[derive(Default)]
struct DesktopAudioState {
    _stream: Option<OutputStream>,
    sink: Option<Sink>,
}

#[cfg(not(target_arch = "wasm32"))]
impl DesktopAudioState {
    fn stop_audio(&mut self) {
        if let Some(sink) = &self.sink {
            sink.stop();
        }
        self.sink = None;
    }
    
    fn play_audio(&mut self, audio_data: Vec<u8>) {
        use std::io::Cursor;
        
        // Stop any currently playing audio
        self.stop_audio();
        
        // Initialize audio output if needed
        if self._stream.is_none() {
            match OutputStreamBuilder::open_default_stream() {
                Ok(stream) => {
                    self._stream = Some(stream);
                }
                Err(e) => {
                    eprintln!("Failed to initialize audio output: {}", e);
                    return;
                }
            }
        }
        
        if let Some(stream) = &self._stream {
            // Create cursor from audio data
            let cursor = Cursor::new(audio_data);
            
            // Decode the audio
            match Decoder::new(cursor) {
                Ok(source) => {
                    // Create a new sink for this audio
                    let sink = Sink::connect_new(&stream.mixer());
                    
                    // Add the source to the sink with infinite looping
                    sink.append(source.repeat_infinite());
                    sink.play();
                    self.sink = Some(sink);
                }
                Err(e) => {
                    eprintln!("Failed to decode audio data: {}", e);
                }
            }
        }
    }
}

impl GameWindow {
    pub fn update(&mut self, message: GameMessage) -> Task<GameMessage> {
        #[cfg(not(target_arch = "wasm32"))]
        let start_time = std::time::Instant::now();
        
        let result = match message {
            GameMessage::Menu_NewGameRequested => {
                if let Ok(mut evt_loop_wguard) = self.game_state.active_event_loop.write() {
                    *evt_loop_wguard = crate::game::ActiveEventLoop::WelcomeScreen(crate::game::WelcomeScreenView::NewGame);
                }
                self.update_focus_for_new_game_screen();
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
                // Stop menu music when starting game
                self.stop_menu_audio();
                self.handle_new_game_start()
            }
            GameMessage::Menu_ContinueGameRequested => {
                if let Ok(mut evt_loop_wguard) = self.game_state.active_event_loop.write() {
                    *evt_loop_wguard = crate::game::ActiveEventLoop::WelcomeScreen(crate::game::WelcomeScreenView::ContinueGame);
                }
                self.update_focus_for_continue_game_screen();
                Task::none()
            }
            GameMessage::Menu_ContinueGameChoiceAltered(saved_game_name) => {
                self.continue_game_game_choice = Some(saved_game_name);
                Task::none()
            }
            GameMessage::Menu_ContinueGameStartClicked => {
                // Stop menu music when starting game
                self.stop_menu_audio();
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
                self.update_focus_for_settings_screen();
                Task::none()
            }
            GameMessage::Menu_LicensesRequested => {
                if let Ok(mut evt_loop_wguard) = self.game_state.active_event_loop.write() {
                    *evt_loop_wguard = crate::game::ActiveEventLoop::WelcomeScreen(crate::game::WelcomeScreenView::Licenses);
                }
                Task::none()
            }
            GameMessage::Menu_SettingsGameCrisesFolderChanged(folder_path) => {
                eprintln!("Settings: Game Crises Folder changed to: {}", folder_path);
                self.settings_game_crises_folder = folder_path;
                self.save_settings();
                Task::none()
            }
            GameMessage::Menu_SettingsOpenCrisesFolder => {
                eprintln!("Settings: Open Crises Folder requested");
                #[cfg(not(target_arch = "wasm32"))]
                {
                    Self::open_crises_folder(&self.settings_game_crises_folder);
                }
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
            GameMessage::Menu_SettingsFontScaleChanged(scale) => {
                eprintln!("Settings: Font Scale changed to: {}", scale);
                self.settings_font_scale = scale.max(0.1).min(10.0); // Clamp to valid range
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
            GameMessage::Focus_NavigateUp => {
                self.focus_state.navigate_up();
                Task::none()
            }
            GameMessage::Focus_NavigateDown => {
                self.focus_state.navigate_down();
                Task::none()
            }
            GameMessage::Focus_NavigateLeft => {
                self.focus_state.navigate_left();
                Task::none()
            }
            GameMessage::Focus_NavigateRight => {
                self.focus_state.navigate_right();
                Task::none()
            }
            GameMessage::Focus_Activate => {
                self.handle_focus_activation()
            }
            GameMessage::Focus_TabInteract => {
                self.handle_tab_interaction(false)
            }
            GameMessage::Focus_ShiftTabInteract => {
                self.handle_tab_interaction(true)
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
                    
                    // Load background audio for the starting scene
                    self.load_scene_background_audio(&crisis, &crisis.story.starting_scene);
                    
                    // Get number of choices before moving crisis
                    let num_choices = crisis.scenes.get(&crisis.story.starting_scene)
                        .map(|scene| scene.choices.len())
                        .unwrap_or(0);
                    
                    self.current_crisis = Some(crisis);
                    self.story_state = Some(story_state);
                    
                    if let Ok(mut evt_loop_wguard) = self.game_state.active_event_loop.write() {
                        *evt_loop_wguard = crate::game::ActiveEventLoop::ActiveGame(crate::game::GameView::StoryScene);
                    }
                    
                    // Set up focus for game screen
                    self.update_focus_for_game_screen(num_choices);
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

    // WARNING: Long Function
    fn handle_continue_game_start(&mut self) -> Task<GameMessage> {
        let verbosity = crate::VERBOSITY.get().unwrap_or(&0);
        
        if let Some(ref saved_game_name) = self.continue_game_game_choice {
            match crate::crisis::load_saved_game(saved_game_name) {
                Ok(loaded_story_state) => {
                    match crate::crisis::load_crisis(&loaded_story_state.template_name) {
                        Ok(crisis) => {
                            // Load background audio for the current scene
                            self.load_scene_background_audio(&crisis, &loaded_story_state.current_scene);
                            
                            // Set up focus for game screen before moving values
                            let num_choices = crisis.scenes.get(&loaded_story_state.current_scene)
                                .map(|scene| scene.choices.len())
                                .unwrap_or(0);
                            
                            self.current_crisis = Some(crisis);
                            self.story_state = Some(loaded_story_state);
                            
                            if let Ok(mut evt_loop_wguard) = self.game_state.active_event_loop.write() {
                                *evt_loop_wguard = crate::game::ActiveEventLoop::ActiveGame(crate::game::GameView::StoryScene);
                            }
                            
                            self.update_focus_for_game_screen(num_choices);
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
        
        // Update focus to include/exclude confirmation dialog buttons
        self.update_focus_for_continue_game_screen();
        
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
        
        // Update focus to remove confirmation dialog buttons
        self.update_focus_for_continue_game_screen();
        
        Task::none()
    }

    fn handle_choice_selection(&mut self, choice_index: usize) -> Task<GameMessage> {
        // Clone the needed data to avoid borrowing issues
        let leads_to = if let (Some(crisis), Some(story_state)) = (&self.current_crisis, &self.story_state) {
            if let Some(current_scene) = crisis.scenes.get(&story_state.current_scene) {
                if let Some(choice) = current_scene.choices.get(choice_index) {
                    Some(choice.leads_to.clone())
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            None
        };
        
        if let Some(leads_to) = leads_to {
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
            
            // Load background audio for the new scene after updating the state
            if let Some(crisis) = self.current_crisis.clone() {
                self.load_scene_background_audio(&crisis, &leads_to);
                
                // Update focus for the new scene's choices
                if let Some(scene) = crisis.scenes.get(&leads_to) {
                    self.update_focus_for_game_screen(scene.choices.len());
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
                    
                    // Update focus for the new scene's choices
                    if let Some(scene) = crisis.scenes.get(&choice.leads_to) {
                        self.update_focus_for_game_screen(scene.choices.len());
                    }
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
        self.current_background_audio.clear(); // Stop any playing audio
        self.update_audio_playback(); // Update audio playback to stop audio
        
        // Update focus for main menu
        self.update_focus_for_main_menu();
        
        // Start menu audio when returning to menu
        self.start_menu_audio();
    }
    
    fn load_scene_background_audio(&mut self, crisis: &crate::crisis::CrisisDefinition, scene_name: &str) {
        if let Some(scene) = crisis.scenes.get(scene_name) {
            if let Some(ref audio_path) = scene.background_audio {
                if let Some(audio_file) = crate::crisis::PlayableCrises::get(audio_path) {
                    self.current_background_audio = audio_file.data.as_ref().to_vec();
                } else {
                    // Audio file not found, clear current audio to stop playing
                    self.current_background_audio.clear();
                }
            } else {
                // No background audio defined, clear current audio to stop playing
                self.current_background_audio.clear();
            }
        } else {
            // Scene not found, clear current audio to stop playing
            self.current_background_audio.clear();
        }
        
        // Update audio playback
        self.update_audio_playback();
    }
    
    #[cfg(target_arch = "wasm32")]
    fn update_audio_playback(&self) {
        if self.current_background_audio.is_empty() {
            if let Some(stop_fn) = STOP_BACKGROUND_AUDIO.get() {
                stop_fn();
            }
        } else {
            if let Some(play_fn) = PLAY_BACKGROUND_AUDIO.get() {
                play_fn(&self.current_background_audio);
            }
        }
    }
    
    #[cfg(not(target_arch = "wasm32"))]
    fn update_audio_playback(&self) {
        use std::io::Cursor;
        use std::sync::{Arc, Mutex};
        use std::thread;
        
        static DESKTOP_AUDIO_STATE: once_cell::sync::Lazy<Arc<Mutex<DesktopAudioState>>> = 
            once_cell::sync::Lazy::new(|| Arc::new(Mutex::new(DesktopAudioState::default())));
        
        let audio_state = DESKTOP_AUDIO_STATE.clone();
        let current_audio = self.current_background_audio.clone();
        
        thread::spawn(move || {
            if let Ok(mut state) = audio_state.lock() {
                if current_audio.is_empty() {
                    // Stop current audio playback
                    state.stop_audio();
                } else {
                    // Start new audio playback
                    state.play_audio(current_audio);
                }
            }
        });
    }
    
    #[cfg(not(target_arch = "wasm32"))]
    fn open_crises_folder(folder_path: &str) {
        use std::path::Path;
        use std::process::Command;
        
        let path = Path::new(folder_path);
        
        if !path.exists() {
            // Try to create the directory
            if let Err(e) = std::fs::create_dir_all(path) {
                eprintln!("Failed to create crises folder '{}': {}", folder_path, e);
                return;
            }
            eprintln!("Created crises folder: {}", folder_path);
        }
        
        // Try to open the folder in the system file manager
        let result = if cfg!(target_os = "windows") {
            Command::new("explorer").arg(folder_path).spawn()
        } else if cfg!(target_os = "macos") {
            Command::new("open").arg(folder_path).spawn()
        } else {
            // Linux and other Unix-like systems
            Command::new("xdg-open").arg(folder_path).spawn()
        };
        
        match result {
            Ok(_) => eprintln!("Opened crises folder: {}", folder_path),
            Err(e) => eprintln!("Failed to open crises folder '{}': {}", folder_path, e),
        }
    }
    
    fn handle_focus_activation(&mut self) -> Task<GameMessage> {
        if let Some(current_focus) = self.focus_state.current_focus {
            match (current_focus.0, current_focus.1) {
                // Menu buttons
                ("menu", 0) => Task::done(GameMessage::Menu_ContinueGameRequested), 
                ("menu", 1) => Task::done(GameMessage::Menu_NewGameRequested),
                ("menu", 2) => Task::done(GameMessage::Menu_SettingsRequested),
                ("menu", 3) => Task::done(GameMessage::Menu_LicensesRequested),
                ("menu", 4) => Task::done(GameMessage::QuitGameRequested),
                
                // New game elements
                ("newgame_button", 0) => Task::done(GameMessage::Menu_NewGameStartClicked),
                // Note: text inputs and pick lists handle their own focus/activation
                
                // Continue game elements
                ("continue_button", 0) => Task::done(GameMessage::Menu_ContinueGameStartClicked),
                ("continue_button", 1) => {
                    if let Some(ref game_name) = self.continue_game_game_choice {
                        Task::done(GameMessage::Menu_ContinueGameDeleteRequested(game_name.clone()))
                    } else {
                        Task::none()
                    }
                }
                
                // Continue game confirmation dialog
                ("continue_confirm", 0) => {
                    // Confirm delete button
                    if let Some(ref game_name) = self.continue_game_delete_confirmation {
                        Task::done(GameMessage::Menu_ContinueGameDeleteConfirmed(game_name.clone()))
                    } else {
                        Task::none()
                    }
                }
                ("continue_confirm", 1) => {
                    // Cancel button - clear the confirmation
                    Task::done(GameMessage::Menu_ContinueGameDeleteRequested("".to_string()))
                }
                
                // Settings elements
                ("settings_button", 0) => Task::done(GameMessage::Menu_SettingsOpenCrisesFolder),
                // Note: toggles can be activated with Enter key
                ("settings_toggle", 0) => Task::done(GameMessage::Menu_SettingsAutosaveToggled(!self.settings_autosave)),
                
                // Game control buttons
                ("control", 0) => Task::done(GameMessage::Game_SaveAndQuitRequested),
                ("control", 1) => Task::done(GameMessage::Game_QuitWithoutSaveRequested),
                
                // Game choice buttons
                ("choice", index) => Task::done(GameMessage::Game_ChoiceSelected(index)),
                
                _ => Task::none(),
            }
        } else {
            Task::none()
        }
    }
    
    fn update_focus_for_main_menu(&mut self) {
        let menu_elements = vec![
            FocusId::menu_button(0), // Continue Game  
            FocusId::menu_button(1), // New Game
            FocusId::menu_button(2), // Settings
            FocusId::menu_button(3), // Licenses
            FocusId::menu_button(4), // Quit Game
        ];
        self.focus_state.set_focusable_elements(menu_elements);
    }
    
    fn update_focus_for_new_game_screen(&mut self) {
        let elements = vec![
            // Left panel
            FocusId::menu_button(0), // Continue Game  
            FocusId::menu_button(1), // New Game
            FocusId::menu_button(2), // Settings
            FocusId::menu_button(3), // Licenses
            FocusId::menu_button(4), // Quit Game
            
            // Right panel - new game elements
            FocusId::new_game_input(0),  // Player name input
            FocusId::new_game_input(1),  // Game template picker
            FocusId::new_game_button(0), // Go button
        ];
        self.focus_state.set_focusable_elements(elements);
    }
    
    fn update_focus_for_continue_game_screen(&mut self) {
        let mut elements = vec![
            // Left panel
            FocusId::menu_button(0), // Continue Game  
            FocusId::menu_button(1), // New Game
            FocusId::menu_button(2), // Settings
            FocusId::menu_button(3), // Licenses
            FocusId::menu_button(4), // Quit Game
            
            // Right panel - continue game elements
            FocusId::continue_game_input(0),  // Saved games picker
            FocusId::continue_game_button(0), // Play button
            FocusId::continue_game_button(1), // Delete button
        ];
        
        // Add confirmation dialog buttons if delete confirmation is active
        if self.continue_game_delete_confirmation.is_some() {
            elements.push(FocusId::continue_game_confirm(0)); // Confirm Delete button
            elements.push(FocusId::continue_game_confirm(1)); // Cancel button
        }
        
        self.focus_state.set_focusable_elements(elements);
    }
    
    fn update_focus_for_settings_screen(&mut self) {
        let mut elements = vec![
            // Left panel
            FocusId::menu_button(0), // Continue Game  
            FocusId::menu_button(1), // New Game
            FocusId::menu_button(2), // Settings
            FocusId::menu_button(3), // Licenses
            FocusId::menu_button(4), // Quit Game
            
            // Right panel - settings elements
            FocusId::settings_input(0),   // Game crises folder
            FocusId::settings_picker(0),  // Difficulty level
            FocusId::settings_toggle(0),  // Autosave toggle
            FocusId::settings_picker(1),  // Language picker
            FocusId::settings_slider(0),  // Font scale slider
        ];
        
        // Add Open Folder button on non-wasm32 platforms
        #[cfg(not(target_arch = "wasm32"))]
        {
            elements.insert(6, FocusId::settings_button(0)); // Open folder button after folder input
        }
        
        self.focus_state.set_focusable_elements(elements);
    }
    
    fn handle_tab_interaction(&mut self, reverse: bool) -> Task<GameMessage> {
        use crate::gui::types::TabInteractionResult;
        
        match self.focus_state.handle_tab_interact(reverse) {
            TabInteractionResult::TextInputToggled(_focus_id, _is_focused) => {
                // Text input focus is handled by the focus state itself
                Task::none()
            }
            
            TabInteractionResult::PickListCycle(focus_id, current_index, is_reverse) => {
                // Handle pick list cycling based on the specific pick list
                match (focus_id.0, focus_id.1) {
                    ("continue_input", 0) => {
                        // Saved games picker - cycle through saved games
                        let saved_games = crate::crisis::get_saved_crisis_names();
                        if !saved_games.is_empty() {
                            let next_index = if is_reverse {
                                if current_index == 0 { saved_games.len() - 1 } else { current_index - 1 }
                            } else {
                                (current_index + 1) % saved_games.len()
                            };
                            self.focus_state.pick_list_selection_index.insert(focus_id, next_index);
                            if let Some(game_name) = saved_games.get(next_index) {
                                return Task::done(GameMessage::Menu_ContinueGameChoiceAltered(game_name.clone()));
                            }
                        }
                        Task::none()
                    }
                    ("newgame_input", 1) => {
                        // Game template picker - cycle through available templates
                        let crisis_names = crate::crisis::get_crisis_names_localized(&self.settings_language);
                        if !crisis_names.is_empty() {
                            let next_index = if is_reverse {
                                if current_index == 0 { crisis_names.len() - 1 } else { current_index - 1 }
                            } else {
                                (current_index + 1) % crisis_names.len()
                            };
                            self.focus_state.pick_list_selection_index.insert(focus_id, next_index);
                            if let Some(template_name) = crisis_names.get(next_index) {
                                return Task::done(GameMessage::Menu_NewGameTemplateChoiceAltered(template_name.clone()));
                            }
                        }
                        Task::none()
                    }
                    ("settings_picker", 0) => {
                        // Difficulty picker - cycle through difficulty levels
                        use crate::gui::DifficultyLevel;
                        let difficulties = &DifficultyLevel::ALL;
                        let next_index = if is_reverse {
                            if current_index == 0 { difficulties.len() - 1 } else { current_index - 1 }
                        } else {
                            (current_index + 1) % difficulties.len()
                        };
                        self.focus_state.pick_list_selection_index.insert(focus_id, next_index);
                        Task::done(GameMessage::Menu_SettingsDifficultyLevelChanged(difficulties[next_index]))
                    }
                    ("settings_picker", 1) => {
                        // Language picker - cycle through available languages
                        let languages = crate::language::get_available_languages();
                        if !languages.is_empty() {
                            let next_index = if is_reverse {
                                if current_index == 0 { languages.len() - 1 } else { current_index - 1 }
                            } else {
                                (current_index + 1) % languages.len()
                            };
                            self.focus_state.pick_list_selection_index.insert(focus_id, next_index);
                            if let Some((lang_code, _)) = languages.get(next_index) {
                                return Task::done(GameMessage::Menu_SettingsLanguageChanged(lang_code.clone()));
                            }
                        }
                        Task::none()
                    }
                    _ => Task::none()
                }
            }
            
            TabInteractionResult::SliderChanged(focus_id, new_value) => {
                // Handle slider value changes
                match (focus_id.0, focus_id.1) {
                    ("settings_slider", 0) => {
                        // Font scale slider
                        Task::done(GameMessage::Menu_SettingsFontScaleChanged(new_value))
                    }
                    _ => Task::none()
                }
            }
            
            TabInteractionResult::ToggleFlipped(focus_id) => {
                // Handle toggle state changes
                match (focus_id.0, focus_id.1) {
                    ("settings_toggle", 0) => {
                        // Autosave toggle
                        Task::done(GameMessage::Menu_SettingsAutosaveToggled(!self.settings_autosave))
                    }
                    _ => Task::none()
                }
            }
            
            TabInteractionResult::ButtonActivated(_focus_id) => {
                // Same as Enter key activation
                self.handle_focus_activation()
            }
            
            TabInteractionResult::None => Task::none(),
        }
    }
    
    fn update_focus_for_game_screen(&mut self, num_choices: usize) {
        let mut elements = vec![
            FocusId("control", 0), // Save and quit
            FocusId("control", 1), // Quit without save
        ];
        
        // Add choice buttons
        for i in 0..num_choices {
            elements.push(FocusId("choice", i));
        }
        
        self.focus_state.set_focusable_elements(elements);
    }
    
    // Helper methods to break down large functions
    
    fn validate_new_game_inputs(&self) -> bool {
        self.new_game_game_template.is_some()
    }
    
    fn get_character_name_for_new_game(&self, crisis: &crate::crisis::CrisisDefinition) -> String {
        if self.new_game_player_name.is_empty() {
            crate::crisis::get_random_character_name(crisis, None, &self.settings_language)
        } else {
            self.new_game_player_name.clone()
        }
    }
    
    fn initialize_game_state(&self, crisis: &crate::crisis::CrisisDefinition, template_name: &str, character_name: String) -> crate::crisis::GameState {
        let user_language = self.settings_language.clone();
        let mut story_state = crate::crisis::GameState::new(
            crisis.metadata.id.clone(),
            user_language.clone(),
            template_name.to_string(),
        );
        story_state.current_scene = crisis.story.starting_scene.clone();
        story_state.character_name = character_name;
        story_state
    }
    
    fn setup_game_session(&mut self, crisis: crate::crisis::CrisisDefinition, story_state: crate::crisis::GameState) -> Task<GameMessage> {
        self.current_crisis = Some(crisis.clone());
        self.story_state = Some(story_state.clone());
        self.choice_text_inputs.clear();
        self.animation_frame_index = 0;

        if let Ok(mut rguard) = self.game_state.active_event_loop.write() {
            *rguard = crate::game::ActiveEventLoop::ActiveGame(crate::game::GameView::StoryScene);
        }

        // Load background audio
        self.load_scene_background_audio(&crisis, &story_state.current_scene);
        
        Task::none()
    }
}
