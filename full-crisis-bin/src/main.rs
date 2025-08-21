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

use clap::Parser;
use once_cell::sync::OnceCell;

/// cli-based console UI to play the game with
mod cli;

pub static CLI_ARGS: OnceCell<Args> = OnceCell::new();

// TODO move beyond hello world
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    hide_console_iff_windows();

    // Store some globals for the cli + gui methods to reference
    let _ = CLI_ARGS.set(args.clone());
    let _ = full_crisis::VERBOSITY.set(args.verbosity);

    full_crisis::init_global_vars();
    // Use dark_light to detect OS theme
    let _ = full_crisis::OS_COLOR_THEME.set(match dark_light::detect() {
        Ok(dark_light::Mode::Dark) => full_crisis::game::OSColorTheme::Dark,
        Ok(dark_light::Mode::Light) => full_crisis::game::OSColorTheme::Light,
        _ => dark_light_fallback_theme_detections(),
    });

    if args.verbosity > 0 {
        eprintln!("OS color theme = {:?} because dark_light::detect() = {:?}", full_crisis::OS_COLOR_THEME.get(), dark_light::detect());
    }

    match args.command {
        Command::Gui => {
            // Iced wants to own the GUI thread and insists on using the main thread; so we let it.
            let r = iced::application(
                full_crisis::gui::GameWindow::new,
                full_crisis::gui::GameWindow::update,
                full_crisis::gui::GameWindow::view,
            )
            .subscription(full_crisis::gui::GameWindow::subscription)
            .theme(full_crisis::gui::GameWindow::theme)
            //.font(include_bytes!("../fonts/icons.ttf").as_slice())
            .default_font(iced::Font::MONOSPACE)
            .settings(full_crisis::gui::GameWindow::make_app_settings())
            .window(full_crisis::gui::GameWindow::make_window_settings())
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
        Command::Test => {
            match run_crisis_tests(args.verbosity) {
                Ok(_) => println!("All crisis tests completed."),
                Err(e) => {
                    eprintln!("Test error: {}", e);
                    std::process::exit(1);
                }
            }
        }
    }

    Ok(())
}

fn run_crisis_tests(verbosity: u8) -> Result<(), Box<dyn std::error::Error>> {
    use full_crisis::crisis::operations::*;
    use full_crisis::crisis::PlayableCrises;
    use std::collections::HashSet;
    
    println!("Starting crisis validation tests...\n");
    
    for pc in PlayableCrises::iter() {
        let path = pc.as_ref();
        if path.ends_with("crisis.toml") {
            let folder_name = path.replace("/crisis.toml", "");
            println!("Testing crisis: {}", folder_name);
            
            match load_crisis(&folder_name) {
                Ok(crisis) => {
                    validate_crisis(&crisis, &folder_name, verbosity)?;
                }
                Err(e) => {
                    println!("❌ Failed to load crisis {}: {}", folder_name, e);
                }
            }
            println!();
        }
    }
    
    Ok(())
}

fn validate_file_exists(scene_name: &str, file_path: &str, file_type: &str) -> Option<String> {
    use full_crisis::crisis::PlayableCrises;
    
    if PlayableCrises::get(file_path).is_none() {
        Some(format!("Scene '{}': {} file not found: {}", scene_name, file_type, file_path))
    } else {
        None
    }
}

fn validate_crisis(
    crisis: &full_crisis::crisis::CrisisDefinition, 
    _folder_name: &str,
    verbosity: u8
) -> Result<(), Box<dyn std::error::Error>> {
    use full_crisis::crisis::PlayableCrises;
    use std::collections::HashSet;
    
    let mut warnings = Vec::new();
    let mut scene_names = HashSet::new();
    let mut referenced_scenes = HashSet::new();
    
    // Collect all scene names
    for scene_name in crisis.scenes.keys() {
        scene_names.insert(scene_name.clone());
    }
    
    // Add starting scene to referenced scenes
    referenced_scenes.insert(crisis.story.starting_scene.clone());
    
    // Validate each scene
    for (scene_name, scene) in &crisis.scenes {
        println!("  Testing scene: {}", scene_name);
        
        // Check background image
        if let Some(bg_img) = &scene.background_image {
            if let Some(warning) = validate_file_exists(scene_name, bg_img, "Background image") {
                warnings.push(warning);
            }
        }
        
        // Check background audio
        if let Some(bg_audio) = &scene.background_audio {
            if let Some(warning) = validate_file_exists(scene_name, bg_audio, "Background audio") {
                warnings.push(warning);
            }
        } else {
            warnings.push(format!("Scene '{}': No background_audio defined", scene_name));
        }
        
        // Check speaking character image
        if let Some(char_img) = &scene.speaking_character_image {
            match char_img {
                full_crisis::crisis::SpeakingCharacterImage::Single(img_path) => {
                    if let Some(warning) = validate_file_exists(scene_name, img_path, "Character image") {
                        warnings.push(warning);
                    }
                }
                full_crisis::crisis::SpeakingCharacterImage::Animation(img_paths) => {
                    if img_paths.is_empty() {
                        warnings.push(format!("Scene '{}': Empty animation array for speaking_character_image", scene_name));
                    } else {
                        for (i, img_path) in img_paths.iter().enumerate() {
                            if let Some(_) = validate_file_exists(scene_name, img_path, &format!("Animation frame {}", i)) {
                                warnings.push(format!("Scene '{}': Animation frame {} not found: {}", scene_name, i, img_path));
                            }
                        }
                    }
                }
            }
        }
        
        // Check choice destinations and collect referenced scenes
        for (i, choice) in scene.choices.iter().enumerate() {
            let leads_to = &choice.leads_to;
            referenced_scenes.insert(leads_to.clone());
            if !scene_names.contains(leads_to) {
                warnings.push(format!("Scene '{}' choice {}: References non-existent scene '{}'", scene_name, i, leads_to));
            }
        }
    }
    
    // Check for disconnected scenes
    let disconnected: Vec<_> = scene_names.difference(&referenced_scenes).collect();
    if !disconnected.is_empty() {
        println!("  ⚠️  Disconnected scenes (not reachable from starting scene):");
        for scene in &disconnected {
            println!("    - {}", scene);
        }
    }
    
    // Print warnings
    if !warnings.is_empty() {
        println!("  ⚠️  Warnings:");
        for warning in &warnings {
            println!("    - {}", warning);
        }
    }
    
    if warnings.is_empty() && disconnected.is_empty() {
        println!("  ✅ All validations passed");
    }
    
    if verbosity > 0 {
        println!("  📊 Statistics:");
        println!("    - Total scenes: {}", scene_names.len());
        println!("    - Referenced scenes: {}", referenced_scenes.len());
        println!("    - Starting scene: {}", crisis.story.starting_scene);
    }
    
    Ok(())
}

fn dark_light_fallback_theme_detections() -> full_crisis::game::OSColorTheme {
    #[cfg(target_os = "linux")]
    {
        // If gimp is installed, read ~/.config/GIMP/3.0/theme.css to see if there is an import to a file /usr/share/gimp/3.0/themes/Default/gimp-dark.css
        match file_contains(&[".config", "GIMP", "3.0", "theme.css",], "themes/Default/gimp-dark.css") {
            Some(true) => return full_crisis::game::OSColorTheme::Dark,
            Some(false) => return full_crisis::game::OSColorTheme::Light,
            _ => { }
        }
    }

    // TODO collect OS heuristics here.

    full_crisis::game::OSColorTheme::Light
}

fn file_contains(home_dir_path_frags: &[&str], content: &str) -> Option<bool> {
    if let Ok(Some(mut home_dir)) = homedir::my_home() {
        for path_frag in home_dir_path_frags.iter() {
            home_dir.push(path_frag);
        }
        let file_path = home_dir;
        if file_path.exists() {
            if let Ok(file_contents) = std::fs::read_to_string(&file_path) {
                return Some(file_contents.contains(content))
            }
        }
    }
    None
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
    Test,
}

impl std::fmt::Display for Command {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Command::Gui => write!(f, "gui"),
            Command::Cli => write!(f, "cli"),
            Command::Test => write!(f, "test"),
        }
    }
}
