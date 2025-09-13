#![allow(unused_imports,dead_code,non_camel_case_types)]
/**
 *  full-crisis - An emergency-response simulator videogame
 *  Copyright (C) 2025  Jeffrey McAteer <jeffrey@jmcateer.com>
 *
 *  This program is free software; you can redistribute it and/or modify
 *  it under the terms of the GNU General Public License as published by
 *  the Free Software Foundation; version 2 of the License ONLY.
 *
 *  This program is distributed in the hope that it will be useful,
 *  but WITHOUT ANY WARRANTY; without even the implied warranty of
 *  MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 *  GNU General Public License for more details.
 *
 *  You should have received a copy of the GNU General Public License along
 *  with this program; if not, write to the Free Software Foundation, Inc.,
 *  51 Franklin Street, Fifth Floor, Boston, MA 02110-1301 USA.
 */

use once_cell::sync::OnceCell;

/// Utilities
pub mod err;
/// Game engine itself, responsible for game data and state changes
pub mod game;
/// iced-based native UI for all major platforms
pub mod gui;
/// Contains the actual crisis file data and structures read by other modules
pub mod crisis;
/// Language detection and management utilities
pub mod language;
/// Central translation system for GUI elements
pub mod translations;
/// Audio management and embedded audio assets
pub mod main_audio;
/// Game controller input abstraction for cross-platform support
pub mod input;


#[cfg(target_arch = "wasm32")]
mod wasm32_storage;
#[cfg(not(target_arch = "wasm32"))]
mod native_storage;

#[cfg(target_arch = "wasm32")]
use wasm32_storage as internal_storage;
#[cfg(not(target_arch = "wasm32"))]
use native_storage as internal_storage;

// Platform-specific time implementations
#[cfg(target_arch = "wasm32")]
mod wasm32_time;
#[cfg(not(target_arch = "wasm32"))]
mod native_time;

#[cfg(target_arch = "wasm32")]
use wasm32_time as internal_time;
#[cfg(not(target_arch = "wasm32"))]
use native_time as internal_time;



pub static GAME: OnceCell<game::GameState> = OnceCell::new();
pub static OS_COLOR_THEME: OnceCell<game::OSColorTheme> = OnceCell::new();
pub static VERBOSITY: OnceCell<u8> = OnceCell::new();
pub static AUDIO_MANAGER: OnceCell<std::sync::Mutex<main_audio::AudioManager>> = OnceCell::new();
pub static CONTROLLER_MANAGER: OnceCell<std::sync::Mutex<Box<dyn input::ControllerManager + Send>>> = OnceCell::new();

// Re-export WASM audio callback setter
#[cfg(target_arch = "wasm32")]
pub use gui::event_handlers::set_audio_callbacks;

pub fn init_global_vars() {

  // TODO likely do not want to do this now, push down to web query of state or FS read op in bin
  if let Err(e) = GAME.set(game::GameState::new()) {
    println!("{:?}", e);
  }

  // Initialize audio manager
  if let Ok(audio_manager) = main_audio::AudioManager::new() {
    if let Err(e) = AUDIO_MANAGER.set(std::sync::Mutex::new(audio_manager)) {
      eprintln!("Failed to initialize audio manager: {:?}", e);
    }
  }

  // Initialize controller manager
  let controller_manager = input::create_controller_manager();
  if let Err(_e) = CONTROLLER_MANAGER.set(std::sync::Mutex::new(controller_manager)) {
    eprintln!("Failed to initialize controller manager");
  }

  // Cannot assign to OS_COLOR_THEME in any reasonable manner

}

/// Public storage functions for external use
pub mod storage {
    pub fn get_attr(name: &str) -> Option<String> {
        super::internal_storage::get_attr(name)
    }
    
    pub fn set_attr(name: &str, value: &str) {
        super::internal_storage::set_attr(name, value)
    }
}

/// Public time functions for cross-platform time management
pub mod time {
    use std::time::Duration;
    
    pub use super::internal_time::PlatformInstant;
    
    /// Get the current instant
    pub fn now() -> PlatformInstant {
        PlatformInstant::now()
    }
    
    /// Create a duration from milliseconds
    pub fn duration_from_millis(millis: u64) -> Duration {
        Duration::from_millis(millis)
    }
    
    /// Create a duration from seconds
    pub fn duration_from_secs(secs: u64) -> Duration {
        Duration::from_secs(secs)
    }
}

/// Quit the game application
#[cfg(target_arch = "wasm32")]
pub fn quit_game() {
    use wasm_bindgen::prelude::*;
    
    #[wasm_bindgen]
    extern "C" {
        #[wasm_bindgen(js_name = "history.back")]
        fn history_back();
    }
    
    // Navigate back in browser history
    history_back();
}

#[cfg(not(target_arch = "wasm32"))]
pub fn quit_game() -> iced::Task<iced::advanced::graphics::core::event::Status> {
    // On native platforms, close window and exit
    iced::window::get_latest().and_then(iced::window::close).chain(iced::exit())
}

// Helper function for GUI that returns the correct Task type
pub fn quit_game_gui<T: Send + 'static>() -> iced::Task<T> {
    #[cfg(target_arch = "wasm32")]
    {
        quit_game();
        iced::Task::none()
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        quit_game().map(|_| unreachable!())
    }
}
