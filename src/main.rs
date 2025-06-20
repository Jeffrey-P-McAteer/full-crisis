#![allow(unused_imports)]
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

/// iced-based native UI for all major platforms
mod gui;
/// Game engine itself, responsible for game data and state changes
mod game;
/// cli-based console UI to play the game with
mod cli;
/// Utilities
mod err;

// TODO move beyond hello world
fn main() -> Result<(), Box<dyn std::error::Error>> {
    hide_console_iff_windows();

    // TODO read+report folder where game files will be searched from
    if let Some(proj_dir_obj) = directories::ProjectDirs::from("com.jmcateer", "FullCrisis",  "FullCrisis") {
        eprintln!("proj_dir_obj.config_local_dir() = {:?}", proj_dir_obj.config_local_dir());
        eprintln!("proj_dir_obj.data_dir() = {:?}", proj_dir_obj.data_dir());
    }

    if let Some(locale_bcp_47) = sys_locale::get_locale() {
        eprintln!("locale_bcp_47 = {:?}", locale_bcp_47);
        // Go from the first 2 chars, which are ISO-639 2-letter language codes, and get the ISO-639 3-letter code.0
        if let Some(lang_639) = rust_iso639::from_code_1(&locale_bcp_47[..2]) {
            eprintln!("lang_639.code_3 = {:?}", lang_639.code_3);
        }
    }

    // Iced wants to own the GUI thread and insists on using the main thread; so we let it.
    let r = iced::application(gui::GameWindow::new, gui::GameWindow::update, gui::GameWindow::view)
          .theme(gui::GameWindow::theme)
          //.font(include_bytes!("../fonts/icons.ttf").as_slice())
          .default_font(iced::Font::MONOSPACE)
          .run();

    if let Err(e) = r {
        eprintln!("[ Error in main() ] {}", e);
    }

    Ok(())
}



fn hide_console_iff_windows() {
    #[cfg(target_os = "windows")]
    {
        if let Ok(val) = std::env::var("NO_CONSOLE_DETATCH") {
            if val.contains("y") || val.contains("Y") || val.contains("1") {
                return;
            }
        }
        // Check if we are run from the console or just launched with explorer.exe
        let mut console_proc_list_buff: Vec<u32> = vec![0; 16];
        let num_procs = unsafe {
            winapi::um::wincon::GetConsoleProcessList(console_proc_list_buff.as_mut_ptr(), 16)
        };
        //eprintln!("num_procs={:?}", num_procs);
        if num_procs == 1 || num_procs == 2 {
            // We were launched from explorer.exe, detatch the console
            unsafe { winapi::um::wincon::FreeConsole() };
        }
        // Otherwise do nothing, we want console messages when run from the console.
    }
}



