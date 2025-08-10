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


#[cfg(target_arch = "wasm32")]
mod wasm32_storage;
#[cfg(not(target_arch = "wasm32"))]
mod native_storage;

#[cfg(target_arch = "wasm32")]
use wasm32_storage as storage;
#[cfg(not(target_arch = "wasm32"))]
use native_storage as storage;



pub static GAME: OnceCell<game::GameState> = OnceCell::new();
pub static OS_COLOR_THEME: OnceCell<game::OSColorTheme> = OnceCell::new();

pub fn init_global_vars() {

  // TODO likely do not want to do this now, push down to web query of state or FS read op in bin
  if let Err(e) = GAME.set(game::GameState::new()) {
    println!("{:?}", e);
  }

  // Cannot assign to OS_COLOR_THEME in any reasonable manner

  // Increment run number every time we start; TODO put more than proof-of-concept data here!
  let last_run_val = crate::storage::get_attr("run-times").unwrap_or_else(|| "0".to_string());
  let last_run_num = last_run_val.parse::<i32>().unwrap_or(0);
  crate::storage::set_attr("run-times", &format!("{}", last_run_num+1))

}
