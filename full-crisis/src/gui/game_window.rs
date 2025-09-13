use super::types::*;
use iced::{Element, Task, Theme, widget};

impl GameWindow {
    pub fn make_app_settings() -> iced::Settings {
        iced::Settings {
            ..Default::default()
        }
    }
    
    pub fn make_window_settings() -> iced::window::Settings {
        iced::window::Settings {
            resizable: true,
            decorations: true,
            fullscreen: true,
            ..Default::default()
        }
    }
    
    pub fn new() -> (Self, Task<GameMessage>) {
        let loaded_settings = Self::load_settings();
        let mut window = Self {
            os_theme: crate::OS_COLOR_THEME.get().unwrap_or(&crate::game::OSColorTheme::Light).clone(),
            game_state: crate::game::GameState::new(),
            new_game_player_name: loaded_settings.last_username,
            new_game_game_template: None,
            new_game_selected_description: None,
            continue_game_game_choice: None,
            continue_game_delete_confirmation: None,
            settings_game_crises_folder: loaded_settings.game_crises_folder,
            settings_difficulty_level: loaded_settings.difficulty_level,
            settings_autosave: loaded_settings.autosave,
            settings_language: loaded_settings.language,
            settings_font_scale: loaded_settings.font_scale,
            current_crisis: None,
            story_state: None,
            choice_text_inputs: std::collections::HashMap::new(),
            animation_frame_index: 0,
            current_background_audio: Vec::new(),
            focus_state: FocusState::new(),
        };
        
        // Initialize focus for main menu
        window.focus_state.set_focusable_elements(vec![
            FocusId::menu_button(0), // Continue Game  
            FocusId::menu_button(1), // New Game
            FocusId::menu_button(2), // Settings
            FocusId::menu_button(3), // Licenses
            FocusId::menu_button(4), // Quit Game
        ]);
        
        (
            window,
            Task::batch([
                widget::focus_next(),
            ]),
        )
    }

    pub fn view(&self) -> Element<'_, GameMessage> {
        #[cfg(not(target_arch = "wasm32"))]
        let start_time = std::time::Instant::now();
        
        let result = if let Ok(evt_loop_rguard) = self.game_state.active_event_loop.read() {
            match evt_loop_rguard.clone() {
                crate::game::ActiveEventLoop::WelcomeScreen(_welcome_screen_state) => {
                    // Start menu audio if not already playing
                    self.ensure_menu_audio_playing();
                    self.view_menu_screen()
                }
                crate::game::ActiveEventLoop::ActiveGame(_game_view) => {
                    self.view_game_screen()
                }
                crate::game::ActiveEventLoop::Exit => {
                    self.view_menu_screen()
                }
            }
        } else {
            self.view_menu_screen()
        };
        
        #[cfg(not(target_arch = "wasm32"))]
        {
            let elapsed = start_time.elapsed();
            let verbosity = crate::VERBOSITY.get().unwrap_or(&0);
            if *verbosity >= 3 {
                eprintln!("[TIMING] GameWindow::view() took {:?}", elapsed);
            } else if *verbosity >= 2 && elapsed.as_millis() > 10 {
                eprintln!("[TIMING] GameWindow::view() took {:?} (>10ms)", elapsed);
            }
        }
        
        result
    }

    pub fn theme(&self) -> Theme {
        match self.os_theme {
            crate::game::OSColorTheme::Light => Theme::Light,
            crate::game::OSColorTheme::Dark => Theme::Dark,
        }
    }

    pub fn subscription(&self) -> iced::Subscription<GameMessage> {
        let mut subscriptions = vec![];
        
        // Keyboard events for focus navigation
        subscriptions.push(
            iced::event::listen_with(|event, status, _window| {
                if let iced::Event::Keyboard(iced::keyboard::Event::KeyPressed { 
                    key, modifiers, ..
                }) = event {
                    // Process all keyboard events (ignore status for now to debug)
                    match key.as_ref() {
                        iced::keyboard::Key::Named(iced::keyboard::key::Named::ArrowUp) => {
                            eprintln!("DEBUG: ArrowUp pressed");
                            Some(GameMessage::Focus_NavigateUp)
                        }
                        iced::keyboard::Key::Named(iced::keyboard::key::Named::ArrowDown) => {
                            Some(GameMessage::Focus_NavigateDown)
                        }
                        iced::keyboard::Key::Named(iced::keyboard::key::Named::ArrowLeft) => {
                            Some(GameMessage::Focus_NavigateLeft)
                        }
                        iced::keyboard::Key::Named(iced::keyboard::key::Named::ArrowRight) => {
                            Some(GameMessage::Focus_NavigateRight)
                        }
                        iced::keyboard::Key::Named(iced::keyboard::key::Named::Enter) => {
                            Some(GameMessage::Focus_Activate)
                        }
                        iced::keyboard::Key::Named(iced::keyboard::key::Named::Tab) => {
                            eprintln!("DEBUG: Tab pressed, shift: {}", modifiers.shift());
                            if modifiers.shift() {
                                Some(GameMessage::Focus_ShiftTabInteract)
                            } else {
                                Some(GameMessage::Focus_TabInteract)
                            }
                        }
                        _ => None
                    }
                } else {
                    None
                }
            })
        );
        
        if self.current_crisis.is_some() && self.story_state.is_some() {
            // Only run animation timer when in active game
            subscriptions.push(
                iced::time::every(std::time::Duration::from_millis(500))
                    .map(|_| GameMessage::Game_AnimationTick)
            );
        }
        
        // Controller input polling - poll every 100ms for UI navigation
        subscriptions.push(
            iced::time::every(std::time::Duration::from_millis(100))
                .map(|_| GameMessage::Controller_PollInput)
        );
        
        iced::Subscription::batch(subscriptions)
    }
    
    #[cfg(not(target_arch = "wasm32"))]
    pub fn get_settings_storage_path() -> String {
        use directories::ProjectDirs;
        
        if let Some(proj_dirs) = ProjectDirs::from("com.jmcateer.full-crisis", "Full-Crisis", "Full-Crisis") {
            if let Some(cache_dir) = proj_dirs.cache_dir().to_str() {
                cache_dir.to_string()
            } else {
                "Unable to get cache directory path".to_string()
            }
        } else {
            "Unable to get project directories".to_string()
        }
    }
    
    // Font scaling helper functions
    pub fn scaled_font_size(&self, base_size: f32) -> f32 {
        let scaled = base_size * self.settings_font_scale;
        // Ensure minimum readable size and maximum reasonable size
        scaled.max(8.0).min(200.0)
    }
    
    pub fn font_size_small(&self) -> f32 {
        self.scaled_font_size(16.0)
    }
    
    pub fn font_size_base(&self) -> f32 {
        self.scaled_font_size(22.0)
    }
    
    pub fn font_size_large(&self) -> f32 {
        self.scaled_font_size(28.0)
    }
    
    pub fn ensure_menu_audio_playing(&self) {
        if let Some(audio_manager) = crate::AUDIO_MANAGER.get() {
            if let Ok(mut manager) = audio_manager.lock() {
                if !manager.is_background_playing() {
                    let _ = manager.play_intro_chime_looped();
                }
            }
        }
    }
    
    pub fn start_menu_audio(&self) {
        if let Some(audio_manager) = crate::AUDIO_MANAGER.get() {
            if let Ok(mut manager) = audio_manager.lock() {
                let _ = manager.play_intro_chime_looped();
            }
        }
    }
    
    pub fn stop_menu_audio(&self) {
        if let Some(audio_manager) = crate::AUDIO_MANAGER.get() {
            if let Ok(mut manager) = audio_manager.lock() {
                manager.stop_background_music();
            }
        }
    }
}
