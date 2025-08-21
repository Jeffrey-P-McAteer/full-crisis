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
        eprintln!("load_crisis: Attempting to load crisis '{}' from path '{}'", crisis_name, crisis_path);
    }
    
    if let Some(file) = PlayableCrises::get(&crisis_path) {
        if *verbosity > 0 {
            eprintln!("load_crisis: Found embedded file, size {} bytes", file.data.len());
        }
        
        let contents = std::str::from_utf8(file.data.as_ref())?;
        if *verbosity > 1 {
            eprintln!("load_crisis: File contents preview (first 200 chars): {}", 
                &contents[..std::cmp::min(200, contents.len())]);
        }
        
        match toml::from_str::<CrisisDefinition>(contents) {
            Ok(mut crisis) => {
                if *verbosity > 0 {
                    eprintln!("load_crisis: Successfully parsed TOML, crisis id: {}", crisis.metadata.id);
                    eprintln!("load_crisis: Starting scene: {}", crisis.story.starting_scene);
                    eprintln!("load_crisis: Character name keys: {:?}", crisis.character_names.names.keys().collect::<Vec<_>>());
                }
                
                // Load scene files from scenes/ directory
                match load_crisis_scenes(crisis_name) {
                    Ok(scenes) => {
                        if *verbosity > 0 {
                            eprintln!("load_crisis: Loaded {} scenes from files", scenes.len());
                        }
                        
                        // Merge scenes from files with any inline scenes (files take precedence)
                        for (scene_name, scene) in scenes {
                            crisis.scenes.insert(scene_name, scene);
                        }
                    }
                    Err(e) => {
                        if *verbosity > 0 {
                            eprintln!("load_crisis: Scene loading failed: {}", e);
                        }
                        return Err(e);
                    }
                }
                
                Ok(crisis)
            }
            Err(e) => {
                if *verbosity > 0 {
                    eprintln!("load_crisis: TOML parsing failed: {}", e);
                }
                Err(e.into())
            }
        }
    } else {
        if *verbosity > 0 {
            eprintln!("load_crisis: File not found. Available embedded files:");
            for path in PlayableCrises::iter() {
                eprintln!("  - {}", path);
            }
        }
        Err(format!("Crisis '{}' not found", crisis_name).into())
    }
}

fn load_crisis_scenes(crisis_name: &str) -> Result<HashMap<String, CrisisScene>, Box<dyn std::error::Error>> {
    let verbosity = crate::VERBOSITY.get().unwrap_or(&0);
    let mut scenes = HashMap::new();
    let crisis_folder = crisis_name.replace(" ", "_");
    
    // Look for scene files in the scenes/ subdirectory
    for file_path in PlayableCrises::iter() {
        let path = file_path.as_ref();
        let scenes_prefix = format!("{}/scenes/", crisis_folder);
        
        if path.starts_with(&scenes_prefix) && path.ends_with(".toml") {
            let scene_name = path
                .strip_prefix(&scenes_prefix)
                .unwrap()
                .strip_suffix(".toml")
                .unwrap()
                .to_string();
            
            if let Some(file) = PlayableCrises::get(path) {
                let contents = std::str::from_utf8(file.data.as_ref())?;
                
                // Parse TOML into a flexible Value first, then convert to CrisisScene
                match toml::from_str::<toml::Value>(contents) {
                    Ok(toml_value) => {
                        match parse_scene_from_toml_value(&toml_value) {
                            Ok(scene) => {
                                if *verbosity > 1 {
                                    eprintln!("load_crisis_scenes: Loaded scene '{}' from '{}'", scene_name, path);
                                }
                                scenes.insert(scene_name, scene);
                            }
                            Err(e) => {
                                if *verbosity > 0 {
                                    eprintln!("load_crisis_scenes: Failed to convert scene '{}': {}", path, e);
                                }
                                return Err(format!("Failed to convert scene file '{}': {}", path, e).into());
                            }
                        }
                    }
                    Err(e) => {
                        if *verbosity > 0 {
                            eprintln!("load_crisis_scenes: Failed to parse scene '{}': {}", path, e);
                        }
                        return Err(format!("Failed to parse scene file '{}': {}", path, e).into());
                    }
                }
            }
        }
    }
    
    Ok(scenes)
}

fn parse_scene_from_toml_value(value: &toml::Value) -> Result<CrisisScene, Box<dyn std::error::Error>> {
    let table = value.as_table().ok_or("Scene must be a table")?;
    
    // Parse text field (required)
    let text = table.get("text")
        .and_then(|v| v.as_table())
        .map(|t| {
            t.iter()
                .map(|(k, v)| (k.clone(), v.as_str().unwrap_or("").to_string()))
                .collect()
        })
        .unwrap_or_default();
    
    // Parse choices (optional, default empty)
    let choices = table.get("choices")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|choice_val| {
                    choice_val.as_table().and_then(|choice_table| {
                        // Parse choice text
                        let choice_text = choice_table.get("text")
                            .and_then(|v| v.as_table())
                            .map(|t| {
                                t.iter()
                                    .map(|(k, v)| (k.clone(), v.as_str().unwrap_or("").to_string()))
                                    .collect()
                            })
                            .unwrap_or_default();
                        
                        // Parse leads_to (required)
                        let leads_to = choice_table.get("leads_to")
                            .and_then(|v| v.as_str())
                            .map(|s| s.to_string())?;
                        
                        // Parse optional fields
                        let subfolder = choice_table.get("subfolder")
                            .and_then(|v| v.as_str())
                            .map(|s| s.to_string());
                        
                        let requires = choice_table.get("requires")
                            .and_then(|v| v.as_table())
                            .map(|t| {
                                t.iter()
                                    .filter_map(|(k, v)| {
                                        v.as_integer().map(|i| (k.clone(), i as i32))
                                    })
                                    .collect()
                            });
                        
                        let character_type = choice_table.get("character_type")
                            .and_then(|v| v.as_str())
                            .map(|s| s.to_string());
                        
                        Some(CrisisChoice {
                            text: choice_text,
                            leads_to,
                            subfolder,
                            requires,
                            character_type,
                            text_input: None, // TODO: Implement if needed
                        })
                    })
                })
                .collect()
        })
        .unwrap_or_default();
    
    // Parse optional fields
    let continue_in_subfolder = table.get("continue_in_subfolder")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());
    
    let background_image = table.get("background_image")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());
    
    // Parse speaking_character_image with flexible handling
    let speaking_character_image = table.get("speaking_character_image")
        .map(|v| {
            match v {
                toml::Value::String(s) => Some(SpeakingCharacterImage::Single(s.clone())),
                toml::Value::Array(arr) => {
                    let images: Vec<String> = arr.iter()
                        .filter_map(|item| item.as_str().map(|s| s.to_string()))
                        .collect();
                    if images.is_empty() {
                        None
                    } else {
                        Some(SpeakingCharacterImage::Animation(images))
                    }
                }
                _ => None,
            }
        })
        .flatten();
    
    Ok(CrisisScene {
        text,
        choices,
        continue_in_subfolder,
        background_image,
        speaking_character_image,
        background_audio: None,
    })
}

pub fn get_random_character_name(crisis: &CrisisDefinition, character_type: Option<&str>, language: &str) -> String {
    let mut rng = thread_rng();
    let verbosity = crate::VERBOSITY.get().unwrap_or(&0);
    
    if *verbosity > 1 {
        eprintln!("get_random_character_name: character_type={:?}, language={}", character_type, language);
        eprintln!("get_random_character_name: available keys={:?}", crisis.character_names.names.keys().collect::<Vec<_>>());
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
        eprintln!("get_random_character_name: trying keys={:?}", possible_keys);
    }
    
    // Try each possible key
    for name_key in &possible_keys {
        if let Some(names) = crisis.character_names.names.get(name_key) {
            if let Some(name) = names.choose(&mut rng) {
                if *verbosity > 1 {
                    eprintln!("get_random_character_name: found name '{}' using key '{}'", name, name_key);
                }
                return name.clone();
            }
        }
    }
    
    if *verbosity > 1 {
        eprintln!("get_random_character_name: no standard keys worked, trying any key with language");
    }
    
    // If no standard keys worked, try to find ANY available character names
    for (key, names) in &crisis.character_names.names {
        if key.contains(language) && !names.is_empty() {
            if let Some(name) = names.choose(&mut rng) {
                if *verbosity > 1 {
                    eprintln!("get_random_character_name: found name '{}' using fallback key '{}'", name, key);
                }
                return name.clone();
            }
        }
    }
    
    if *verbosity > 1 {
        eprintln!("get_random_character_name: no language-specific keys worked, trying any key");
    }
    
    // Absolute final fallback - try any names at all
    for (key, names) in &crisis.character_names.names {
        if !names.is_empty() {
            if let Some(name) = names.choose(&mut rng) {
                if *verbosity > 1 {
                    eprintln!("get_random_character_name: found name '{}' using absolute fallback key '{}'", name, key);
                }
                return name.clone();
            }
        }
    }
    
    if *verbosity > 0 {
        eprintln!("get_random_character_name: all fallbacks failed, using 'Player'");
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