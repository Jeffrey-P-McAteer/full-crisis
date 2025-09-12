#![allow(unreachable_patterns, non_camel_case_types)]

pub mod types;
pub mod settings;
pub mod ui_builders;
pub mod event_handlers;
pub mod game_window;
pub mod views;
pub mod styles;
pub mod helpers;
pub mod builders;

use types::*;

// Re-export key types for public use
pub use types::{GameWindow, GameMessage, DifficultyLevel, GameSettings};
pub use styles::*;