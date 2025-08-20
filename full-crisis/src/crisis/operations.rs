use super::types::*;
use super::PlayableCrises;
use std::collections::HashMap;
use rand::prelude::*;

pub fn get_crisis_names() -> Vec<String> {
    get_crisis_names_localized("eng")
}

pub fn get_crisis_names_localized(language: &str) -> Vec<String> {
    let mut names = vec![];
    
    for pc in PlayableCrises::iter() {
        let path = pc.as_ref();
        if path.ends_with("crisis.toml") {
            let folder_name = path.replace("/crisis.toml", "");
            match load_crisis(&folder_name) {
                Ok(crisis) => {
                    let localized_name = get_localized_text(&crisis.name, language);
                    names.push(localized_name);
                }
                Err(_) => {
                    // Fallback to folder name with underscores replaced
                    let crisis_name = folder_name.replace("_", " ");
                    names.push(crisis_name);
                }
            }
        }
    }
    
    if names.is_empty() {
        names.push("No crises available".to_string());
    }
    
    names
}

pub fn get_crisis_info_by_display_name(display_name: &str, language: &str) -> Option<(String, String)> {
    for pc in PlayableCrises::iter() {
        let path = pc.as_ref();
        if path.ends_with("crisis.toml") {
            let folder_name = path.replace("/crisis.toml", "");
            if let Ok(crisis) = load_crisis(&folder_name) {
                let localized_name = get_localized_text(&crisis.name, language);
                if localized_name == display_name {
                    let description = get_localized_text(&crisis.description, language);
                    return Some((folder_name, description));
                }
            }
        }
    }
    None
}

pub fn get_template_name_from_display_name(display_name: &str) -> String {
    // Try to find the folder name by matching display name across all languages
    for pc in PlayableCrises::iter() {
        let path = pc.as_ref();
        if path.ends_with("crisis.toml") {
            let folder_name = path.replace("/crisis.toml", "");
            if let Ok(crisis) = load_crisis(&folder_name) {
                // Check if display name matches any localized version
                for (_, localized_name) in &crisis.name {
                    if localized_name == display_name {
                        return folder_name;
                    }
                }
            }
        }
    }
    
    // Fallback to old behavior if no match found
    display_name.replace(" ", "_")
}

pub fn get_saved_games() -> SavedGames {
    if let Some(content) = crate::storage::get_attr("saved_games") {
        serde_json::from_str(&content).unwrap_or_default()
    } else {
        SavedGames::default()
    }
}

pub fn save_games(saved_games: &SavedGames) {
    if let Ok(serialized) = serde_json::to_string(saved_games) {
        crate::storage::set_attr("saved_games", &serialized);
    }
}

pub fn get_saved_crisis_names() -> Vec<String> {
    let saved_games = get_saved_games();
    if saved_games.saves.is_empty() {
        vec!["No saved games found".to_string()]
    } else {
        saved_games.get_save_names()
    }
}

pub fn save_current_game(
    story_state: &GameState, 
    template_name: &str, 
    save_name: Option<String>
) -> Result<String, String> {
    let mut saved_games = get_saved_games();
    
    // Load the crisis to get the human-readable name
    let human_readable_name = match load_crisis(template_name) {
        Ok(crisis) => get_localized_text(&crisis.name, &story_state.language),
        Err(_) => template_name.replace("_", " "), // Fallback to template name
    };
    
    // Generate save name if not provided
    let timestamp = crate::storage::time_now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let save_name = save_name.unwrap_or_else(|| {
        format!("{}-{}", story_state.character_name, timestamp)
    });
    
    let saved_game = SavedGame {
        save_name: save_name.clone(),
        crisis_name: human_readable_name,
        character_name: story_state.character_name.clone(),
        current_scene: story_state.current_scene.clone(),
        variables: story_state.variables.clone(),
        text_inputs: story_state.text_inputs.clone(),
        character_type: story_state.character_type.clone(),
        language: story_state.language.clone(),
        save_timestamp: format!("{}", timestamp),
        template_name: template_name.to_string(),
    };
    
    saved_games.add_save(saved_game);
    save_games(&saved_games);
    
    Ok(save_name)
}

pub fn load_saved_game(display_name: &str) -> Result<GameState, String> {
    let saved_games = get_saved_games();
    
    if let Some(saved_game) = saved_games.get_save_by_display_name(display_name) {
        // Use template_name if available, otherwise fall back to crisis_name
        let template_name = if saved_game.template_name.is_empty() {
            saved_game.crisis_name.clone()
        } else {
            saved_game.template_name.clone()
        };
        
        let mut game_state = GameState::new(
            saved_game.crisis_name.clone(),
            saved_game.language.clone(),
            template_name.clone(),
        );
        
        game_state.character_name = saved_game.character_name.clone();
        game_state.current_scene = saved_game.current_scene.clone();
        game_state.variables = saved_game.variables.clone();
        game_state.text_inputs = saved_game.text_inputs.clone();
        game_state.character_type = saved_game.character_type.clone();
        
        Ok(game_state)
    } else {
        Err(format!("Saved game '{}' not found", display_name))
    }
}

pub fn delete_saved_game(display_name: &str) -> Result<(), String> {
    let mut saved_games = get_saved_games();
    
    if saved_games.delete_save_by_display_name(display_name) {
        save_games(&saved_games);
        Ok(())
    } else {
        Err(format!("Saved game '{}' not found", display_name))
    }
}

pub fn load_crisis(crisis_name: &str) -> Result<CrisisDefinition, Box<dyn std::error::Error>> {
    let verbosity = crate::VERBOSITY.get().unwrap_or(&0);
    let crisis_path = format!("{}/crisis.toml", crisis_name.replace(" ", "_"));
    
    if *verbosity > 0 {
        eprintln!("[VERBOSE] load_crisis: Attempting to load crisis '{}' from path '{}'", crisis_name, crisis_path);
    }
    
    if let Some(file) = PlayableCrises::get(&crisis_path) {
        if *verbosity > 0 {
            eprintln!("[VERBOSE] load_crisis: Found embedded file, size {} bytes", file.data.len());
        }
        
        let contents = std::str::from_utf8(file.data.as_ref())?;
        if *verbosity > 1 {
            eprintln!("[VERBOSE] load_crisis: File contents preview (first 200 chars): {}", 
                &contents[..std::cmp::min(200, contents.len())]);
        }
        
        match toml::from_str::<CrisisDefinition>(contents) {
            Ok(crisis) => {
                if *verbosity > 0 {
                    eprintln!("[VERBOSE] load_crisis: Successfully parsed TOML, crisis id: {}", crisis.metadata.id);
                    eprintln!("[VERBOSE] load_crisis: Starting scene: {}", crisis.story.starting_scene);
                    eprintln!("[VERBOSE] load_crisis: Character name keys: {:?}", crisis.character_names.names.keys().collect::<Vec<_>>());
                }
                Ok(crisis)
            }
            Err(e) => {
                if *verbosity > 0 {
                    eprintln!("[VERBOSE] load_crisis: TOML parsing failed: {}", e);
                }
                Err(e.into())
            }
        }
    } else {
        if *verbosity > 0 {
            eprintln!("[VERBOSE] load_crisis: File not found. Available embedded files:");
            for path in PlayableCrises::iter() {
                eprintln!("  - {}", path);
            }
        }
        Err(format!("Crisis '{}' not found", crisis_name).into())
    }
}

pub fn get_random_character_name(crisis: &CrisisDefinition, character_type: Option<&str>, language: &str) -> String {
    let mut rng = thread_rng();
    let verbosity = crate::VERBOSITY.get().unwrap_or(&0);
    
    if *verbosity > 1 {
        eprintln!("[VERBOSE] get_random_character_name: character_type={:?}, language={}", character_type, language);
        eprintln!("[VERBOSE] get_random_character_name: available keys={:?}", crisis.character_names.names.keys().collect::<Vec<_>>());
    }
    
    // Try different naming patterns based on character_type
    let possible_keys = match character_type {
        Some(ctype) => vec![
            format!("{}_male_{}", ctype, language),
            format!("{}_female_{}", ctype, language),
            format!("{}_{}", ctype, language),
            format!("male_{}", language),
        ],
        None => vec![
            format!("male_{}", language),
            format!("female_{}", language),
        ],
    };
    
    if *verbosity > 1 {
        eprintln!("[VERBOSE] get_random_character_name: trying keys={:?}", possible_keys);
    }
    
    // Try each possible key
    for name_key in &possible_keys {
        if let Some(names) = crisis.character_names.names.get(name_key) {
            if let Some(name) = names.choose(&mut rng) {
                if *verbosity > 1 {
                    eprintln!("[VERBOSE] get_random_character_name: found name '{}' using key '{}'", name, name_key);
                }
                return name.clone();
            }
        }
    }
    
    if *verbosity > 1 {
        eprintln!("[VERBOSE] get_random_character_name: no standard keys worked, trying any key with language");
    }
    
    // If no standard keys worked, try to find ANY available character names
    for (key, names) in &crisis.character_names.names {
        if key.contains(language) && !names.is_empty() {
            if let Some(name) = names.choose(&mut rng) {
                if *verbosity > 1 {
                    eprintln!("[VERBOSE] get_random_character_name: found name '{}' using fallback key '{}'", name, key);
                }
                return name.clone();
            }
        }
    }
    
    if *verbosity > 1 {
        eprintln!("[VERBOSE] get_random_character_name: no language-specific keys worked, trying any key");
    }
    
    // Absolute final fallback - try any names at all
    for (key, names) in &crisis.character_names.names {
        if !names.is_empty() {
            if let Some(name) = names.choose(&mut rng) {
                if *verbosity > 1 {
                    eprintln!("[VERBOSE] get_random_character_name: found name '{}' using absolute fallback key '{}'", name, key);
                }
                return name.clone();
            }
        }
    }
    
    if *verbosity > 0 {
        eprintln!("[VERBOSE] get_random_character_name: all fallbacks failed, using 'Player'");
    }
    
    "Player".to_string()
}

pub fn get_scene_text(scene: &CrisisScene, language: &str, character_name: &str) -> String {
    let fallback_chain = crate::language::get_language_fallback_chain(language);
    
    let text = fallback_chain.iter()
        .find_map(|lang| scene.text.get(lang))
        .unwrap_or(&"Missing text".to_string())
        .clone();
    
    text.replace("{character_name}", character_name)
}

pub fn get_scene_text_with_substitutions(scene: &CrisisScene, language: &str, character_name: &str, text_inputs: &HashMap<String, String>) -> String {
    let fallback_chain = crate::language::get_language_fallback_chain(language);
    
    let mut text = fallback_chain.iter()
        .find_map(|lang| scene.text.get(lang))
        .unwrap_or(&"Missing text".to_string())
        .clone();
    
    // Replace character name
    text = text.replace("{character_name}", character_name);
    
    // Replace text input variables
    for (variable_name, value) in text_inputs {
        let placeholder = format!("{{{}}}", variable_name);
        text = text.replace(&placeholder, value);
    }
    
    text
}

pub fn get_localized_text(text_map: &std::collections::HashMap<String, String>, language: &str) -> String {
    let fallback_chain = crate::language::get_language_fallback_chain(language);
    
    fallback_chain.iter()
        .find_map(|lang| text_map.get(lang))
        .unwrap_or(&"Missing text".to_string())
        .clone()
}