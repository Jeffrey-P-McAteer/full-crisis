#![allow(unused_imports,dead_code)]
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
/// Contains host info such as config folders, language, etc. Items which the user can change but the game will not.

pub static GAME: OnceCell<game::GameState> = OnceCell::new();

pub fn init_global_vars() {
  if let Err(e) = GAME.set(game::GameState::new()) {
    println!("{:?}", e);
  }
}

