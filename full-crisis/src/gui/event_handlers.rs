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
            GameMessage::Nop => {
                eprintln!("Recieved a GameMessage::Nop");
                Task::none()
            },
            GameMessage::NavigateUp => {
                self.handle_navigate_up()
            }
            GameMessage::NavigateDown => {
                self.handle_navigate_down()
            }
            GameMessage::NavigateLeft => {
                self.handle_navigate_left()
            }
            GameMessage::NavigateRight => {
                self.handle_navigate_right()
            }
            GameMessage::NavigateTab => {
                self.handle_navigate_tab()
            }
            GameMessage::NavigateEnter => {
                self.handle_navigate_enter()
            }
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
                            // Load background audio for the current scene
                            self.load_scene_background_audio(&crisis, &loaded_story_state.current_scene);
                            
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
        self.current_background_audio.clear(); // Stop any playing audio
        self.update_audio_playback(); // Update audio playback to stop audio
        
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
    
    fn handle_navigate_up(&mut self) -> Task<GameMessage> {
        // Only handle navigation if we're in the menu screen
        if let Ok(evt_loop_val) = self.game_state.active_event_loop.try_read() {
            if let crate::game::ActiveEventLoop::WelcomeScreen(_) = *evt_loop_val {
                if self.menu_right_panel_focused {
                    // Navigation within right panel is context-specific
                    // For now, we'll handle this in the specific UI builders
                } else {
                    // Navigate up through menu buttons
                    if self.menu_focused_button > 0 {
                        self.menu_focused_button -= 1;
                    }
                }
            }
        }
        Task::none()
    }
    
    fn handle_navigate_down(&mut self) -> Task<GameMessage> {
        // Only handle navigation if we're in the menu screen
        if let Ok(evt_loop_val) = self.game_state.active_event_loop.try_read() {
            if let crate::game::ActiveEventLoop::WelcomeScreen(_) = *evt_loop_val {
                if self.menu_right_panel_focused {
                    // Navigation within right panel is context-specific
                } else {
                    // Navigate down through menu buttons (5 total: Continue, New, Settings, Licenses, Quit)
                    if self.menu_focused_button < 4 {
                        self.menu_focused_button += 1;
                    }
                }
            }
        }
        Task::none()
    }
    
    fn handle_navigate_left(&mut self) -> Task<GameMessage> {
        // Move focus to the left (from right panel to menu buttons)
        if let Ok(evt_loop_val) = self.game_state.active_event_loop.try_read() {
            if let crate::game::ActiveEventLoop::WelcomeScreen(_) = *evt_loop_val {
                if self.menu_right_panel_focused {
                    self.menu_right_panel_focused = false;
                    // Return focus to the menu based on the current view
                    match &*evt_loop_val {
                        crate::game::ActiveEventLoop::WelcomeScreen(crate::game::WelcomeScreenView::ContinueGame) => {
                            self.menu_focused_button = 0;
                        }
                        crate::game::ActiveEventLoop::WelcomeScreen(crate::game::WelcomeScreenView::NewGame) => {
                            self.menu_focused_button = 1;
                        }
                        crate::game::ActiveEventLoop::WelcomeScreen(crate::game::WelcomeScreenView::Settings) => {
                            self.menu_focused_button = 2;
                        }
                        crate::game::ActiveEventLoop::WelcomeScreen(crate::game::WelcomeScreenView::Licenses) => {
                            self.menu_focused_button = 3;
                        }
                        _ => {}
                    }
                }
            }
        }
        Task::none()
    }
    
    fn handle_navigate_right(&mut self) -> Task<GameMessage> {
        // Move focus to the right (from menu buttons to right panel)
        if let Ok(evt_loop_val) = self.game_state.active_event_loop.try_read() {
            if let crate::game::ActiveEventLoop::WelcomeScreen(ws_area) = &*evt_loop_val {
                if !self.menu_right_panel_focused {
                    // Only move to right panel if there's content there
                    match ws_area {
                        crate::game::WelcomeScreenView::NewGame => {
                            self.menu_right_panel_focused = true;
                            // Focus the player name text input in new game
                            return iced::widget::text_input::focus(
                                iced::widget::text_input::Id::new("new_game_player_name")
                            );
                        }
                        crate::game::WelcomeScreenView::ContinueGame |
                        crate::game::WelcomeScreenView::Settings |
                        crate::game::WelcomeScreenView::Licenses => {
                            self.menu_right_panel_focused = true;
                            // Focus the first interactive element in the right panel
                            return iced::widget::focus_next();
                        }
                        _ => {}
                    }
                }
            }
        }
        Task::none()
    }
    
    fn handle_navigate_tab(&mut self) -> Task<GameMessage> {
        // Tab navigation for pick_list cycling
        if self.menu_right_panel_focused && self.pick_list_expanded {
            // This will be handled by the pick_list widget itself
            return iced::widget::focus_next();
        }
        Task::none()
    }
    
    fn handle_navigate_enter(&mut self) -> Task<GameMessage> {
        // Enter key handling
        let is_menu_screen = if let Ok(evt_loop_val) = self.game_state.active_event_loop.try_read() {
            matches!(*evt_loop_val, crate::game::ActiveEventLoop::WelcomeScreen(_))
        } else {
            false
        };
        
        if is_menu_screen {
            if !self.menu_right_panel_focused {
                // Trigger the button action based on focused button
                return match self.menu_focused_button {
                    0 => self.update(GameMessage::Menu_ContinueGameRequested),
                    1 => self.update(GameMessage::Menu_NewGameRequested),
                    2 => self.update(GameMessage::Menu_SettingsRequested),
                    3 => self.update(GameMessage::Menu_LicensesRequested),
                    4 => self.update(GameMessage::QuitGameRequested),
                    _ => Task::none(),
                };
            } else {
                // In the right panel, Enter should submit forms or toggle pick_lists
                // The pick_list widget itself handles Enter when focused
                // For now, we'll let the default behavior handle this
                return Task::none();
            }
        }
        Task::none()
    }
}
