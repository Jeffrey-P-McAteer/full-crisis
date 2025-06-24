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
use clap::Parser;
use once_cell::sync::OnceCell;

/// cli-based console UI to play the game with
mod cli;
/// Utilities
mod err;
/// Game engine itself, responsible for game data and state changes
mod game;
/// iced-based native UI for all major platforms
mod gui;
/// Contains host info such as config folders, language, etc. Items which the user can change but the game will not.
mod system;

pub static SYS_CFG: OnceCell<system::SystemConfig> = OnceCell::new();
pub static CLI_ARGS: OnceCell<Args> = OnceCell::new();
pub static GAME: OnceCell<game::GameState> = OnceCell::new();

// TODO move beyond hello world
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    hide_console_iff_windows();

    // Store some globals for the cli + gui methods to reference
    let _ = CLI_ARGS.set(args.clone());
    let _ = SYS_CFG.set(system::SystemConfig::new());
    let _ = GAME.set(game::GameState::new());

    match args.command {
        Command::Gui => {
            // Iced wants to own the GUI thread and insists on using the main thread; so we let it.
            let r = iced::application(
                gui::GameWindow::new,
                gui::GameWindow::update,
                gui::GameWindow::view,
            )
            .theme(gui::GameWindow::theme)
            //.font(include_bytes!("../fonts/icons.ttf").as_slice())
            .default_font(iced::Font::MONOSPACE)
            .settings(gui::GameWindow::make_app_settings())
            .window(gui::GameWindow::make_window_settings())
            .run();

            if let Err(e) = r {
                eprintln!("[ Error in main() ] {}", e);
            }
        }
        Command::Cli => {
            // For compatability w/ iced runtime we also use tokio for the CLI routines
            let rt = tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .worker_threads(4)
                .build()
                .expect("Failed to build Tokio runtime");

            rt.block_on(async {
                let r = cli::run().await;

                // Just in-case cli borked real bad, restore the terminal a 2nd time before printing any errors.
                // Restore terminal
                if let Err(e) = crossterm::terminal::disable_raw_mode() {
                    eprintln!("{:?}", e);
                }

                if let Err(e) = r {
                    eprintln!("[ Error in main() ] {}", e);
                }
            });
        }
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

#[derive(Clone, Debug, clap::Parser)]
#[command(author, version, about)]
pub struct Args {
    #[arg(value_enum, default_value_t = Command::Gui)]
    command: Command,

    #[arg(short, action = clap::ArgAction::Count)]
    verbosity: u8,
}

#[derive(Clone, Debug, clap::ValueEnum)]
pub enum Command {
    Gui,
    Cli,
}

impl std::fmt::Display for Command {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Command::Gui => write!(f, "gui"),
            Command::Cli => write!(f, "cli"),
        }
    }
}
