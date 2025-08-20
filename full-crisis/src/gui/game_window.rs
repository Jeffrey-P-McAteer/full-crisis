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

    pub fn view(&self) -> Element<'_, GameMessage> {
        #[cfg(not(target_arch = "wasm32"))]
        let start_time = std::time::Instant::now();
        
        let result = if let Ok(evt_loop_rguard) = self.game_state.active_event_loop.read() {
            match evt_loop_rguard.clone() {
                crate::game::ActiveEventLoop::WelcomeScreen(_welcome_screen_state) => {
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
}