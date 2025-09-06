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
        (
            Self {
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
                menu_focused_button: 0,
                menu_right_panel_focused: false,
                pick_list_expanded: false,
            },
            Task::batch([
                widget::focus_next(),
            ]),
        )
    }

    pub fn view(&self) -> Element<'_, GameMessage> {
        #[cfg(not(target_arch = "wasm32"))]
        let start_time = std::time::Instant::now();
        
        let main_content = if let Ok(evt_loop_rguard) = self.game_state.active_event_loop.read() {
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
        
        // Add debug overlay if verbosity is enabled
        let verbosity = crate::VERBOSITY.get().unwrap_or(&0);
        if *verbosity > 0 {
            let debug_overlay = self.create_debug_overlay();
            iced::widget::stack![
                main_content,
                debug_overlay
            ]
            .width(iced::Length::Fill)
            .height(iced::Length::Fill)
            .into()
        } else {
            main_content
        }
    }

    pub fn theme(&self) -> Theme {
        match self.os_theme {
            crate::game::OSColorTheme::Light => Theme::Light,
            crate::game::OSColorTheme::Dark => Theme::Dark,
        }
    }

    pub fn subscription(&self) -> iced::Subscription<GameMessage> {
        use iced::keyboard;
        use iced::Subscription;
        
        let mut subscriptions = vec![];
        
        // Animation timer for active game
        if self.current_crisis.is_some() && self.story_state.is_some() {
            subscriptions.push(
                iced::time::every(std::time::Duration::from_millis(500))
                    .map(|_| GameMessage::Game_AnimationTick)
            );
        }
        
        // Keyboard events subscription
        subscriptions.push(
            keyboard::on_key_press(|key, modifiers| {
                match key {
                    keyboard::Key::Named(keyboard::key::Named::ArrowUp) => Some(GameMessage::NavigateUp),
                    keyboard::Key::Named(keyboard::key::Named::ArrowDown) => Some(GameMessage::NavigateDown),
                    keyboard::Key::Named(keyboard::key::Named::ArrowLeft) => Some(GameMessage::NavigateLeft),
                    keyboard::Key::Named(keyboard::key::Named::ArrowRight) => Some(GameMessage::NavigateRight),
                    keyboard::Key::Named(keyboard::key::Named::Tab) => {
                        if modifiers.shift() {
                            Some(GameMessage::NavigateShiftTab)
                        } else {
                            Some(GameMessage::NavigateTab)
                        }
                    },
                    keyboard::Key::Named(keyboard::key::Named::Enter) => Some(GameMessage::NavigateEnter),
                    keyboard::Key::Named(keyboard::key::Named::Escape) => Some(GameMessage::NavigateEscape),
                    _ => None,
                }
            })
        );
        
        Subscription::batch(subscriptions)
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
    
    fn create_debug_overlay(&self) -> Element<'_, GameMessage> {
        use iced::widget::{container, text, Column, Row, Space};
        use iced::{alignment, Color, Length};
        
        // Generate focus debug text
        let focus_info = self.get_focus_debug_info();
        
        let debug_text = text(focus_info)
            .size(self.font_size_small())
            .color(Color::from_rgb(1.0, 0.0, 0.0)); // Red text
        
        // Create a container that positions the text in the lower-right
        let debug_container = container(debug_text)
            .padding(10)
            .style(|_theme: &Theme| {
                iced::widget::container::Style {
                    background: Some(iced::Background::Color(Color::from_rgba(0.0, 0.0, 0.0, 0.7))), // Semi-transparent black background
                    border: iced::border::rounded(4),
                    ..Default::default()
                }
            });
        
        // Use columns and rows with spacers to position in lower-right
        Column::new()
            .push(Space::with_height(Length::Fill))
            .push(
                Row::new()
                    .push(Space::with_width(Length::Fill))
                    .push(debug_container)
            )
            .into()
    }
    
    fn get_focus_debug_info(&self) -> String {
        let mut info = String::new();
        
        // Add menu focus information
        if let Ok(evt_loop_val) = self.game_state.active_event_loop.try_read() {
            if let crate::game::ActiveEventLoop::WelcomeScreen(ws_view) = &*evt_loop_val {
                info.push_str("Menu Focus:\n");
                
                if self.menu_right_panel_focused {
                    info.push_str(&format!("Panel: {:?}\n", ws_view));
                    info.push_str("Location: Right Panel\n");
                    if self.pick_list_expanded {
                        info.push_str("Pick List: Expanded\n");
                    }
                } else {
                    let button_name = match self.menu_focused_button {
                        0 => "Continue Game",
                        1 => "New Game",
                        2 => "Settings",
                        3 => "Licenses",
                        4 => "Quit Game",
                        _ => "Unknown",
                    };
                    info.push_str(&format!("Button: {} (index: {})\n", button_name, self.menu_focused_button));
                    info.push_str("Location: Left Menu\n");
                }
            } else {
                info.push_str("Focus: Game Screen\n");
            }
        }
        
        info
    }
}