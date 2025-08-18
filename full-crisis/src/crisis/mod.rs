use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use rand::prelude::*;

#[derive(rust_embed::Embed)]
#[folder = "$CARGO_MANIFEST_DIR/../playable-crises/"]
struct _Interior_PlayableCrises;

pub struct PlayableCrises;
impl PlayableCrises {
  pub fn get(file_path: &str) -> Option<rust_embed::EmbeddedFile> {
    _Interior_PlayableCrises::get(file_path)
  }
  pub fn iter() -> impl Iterator<Item = std::borrow::Cow<'static, str>> {
    _Interior_PlayableCrises::iter()
  }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrisisMetadata {
    pub id: String,
    pub version: String,
    pub author: String,
    pub description_key: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrisisCharacterNames {
    #[serde(flatten)]
    pub names: HashMap<String, Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrisisStory {
    pub starting_scene: String,
    pub default_language: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrisisMechanics {
    pub time_limit_minutes: u32,
    pub save_progress: bool,
    pub allow_restart: bool,
    pub track_decisions: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrisisConditions {
    pub variables: Option<Vec<String>>,
    pub choice_effects: Option<HashMap<String, HashMap<String, i32>>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrisisChoice {
    pub text: HashMap<String, String>,
    pub leads_to: String,
    pub subfolder: Option<String>,
    pub requires: Option<HashMap<String, i32>>,
    pub character_type: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrisisScene {
    pub text: HashMap<String, String>,
    #[serde(default)]
    pub choices: Vec<CrisisChoice>,
    pub continue_in_subfolder: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrisisDefinition {
    pub metadata: CrisisMetadata,
    pub name: HashMap<String, String>,
    pub description: HashMap<String, String>,
    pub character_names: CrisisCharacterNames,
    pub story: CrisisStory,
    pub mechanics: CrisisMechanics,
    pub conditions: CrisisConditions,
    pub scenes: HashMap<String, CrisisScene>,
}

#[derive(Debug, Clone)]
pub struct GameState {
    pub current_scene: String,
    pub character_name: String,
    pub character_type: Option<String>,
    pub variables: HashMap<String, i32>,
    pub language: String,
    pub crisis_id: String,
}

impl GameState {
    pub fn new(crisis_id: String, language: String) -> Self {
        Self {
            current_scene: String::new(),
            character_name: String::new(),
            character_type: None,
            variables: HashMap::new(),
            language,
            crisis_id,
        }
    }
}

pub fn get_crisis_names() -> Vec<String> {
    let mut names = vec![];
    
    for pc in PlayableCrises::iter() {
        let path = pc.as_ref();
        if path.ends_with("crisis.toml") {
            let crisis_name = path.replace("/crisis.toml", "").replace("_", " ");
            names.push(crisis_name);
        }
    }
    
    if names.is_empty() {
        names.push("No crises available".to_string());
    }
    
    names
}

pub fn get_saved_crisis_names() -> Vec<String> {
    vec!["TODO: Load saved games".to_string()]
}

pub fn load_crisis(crisis_name: &str) -> Result<CrisisDefinition, Box<dyn std::error::Error>> {
    let crisis_path = format!("{}/crisis.toml", crisis_name.replace(" ", "_"));
    
    if let Some(file) = PlayableCrises::get(&crisis_path) {
        let contents = std::str::from_utf8(file.data.as_ref())?;
        let crisis: CrisisDefinition = toml::from_str(contents)?;
        Ok(crisis)
    } else {
        Err(format!("Crisis '{}' not found", crisis_name).into())
    }
}

pub fn get_random_character_name(crisis: &CrisisDefinition, character_type: Option<&str>, language: &str) -> String {
    let mut rng = thread_rng();
    
    let name_key = match character_type {
        Some(ctype) => format!("{}_{}", ctype, language),
        None => format!("male_{}", language),
    };
    
    if let Some(names) = crisis.character_names.names.get(&name_key) {
        if let Some(name) = names.choose(&mut rng) {
            return name.clone();
        }
    }
    
    // Fallback to English male names
    if let Some(names) = crisis.character_names.names.get(&format!("male_{}", "eng")) {
        if let Some(name) = names.choose(&mut rng) {
            return name.clone();
        }
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

pub fn get_localized_text(text_map: &std::collections::HashMap<String, String>, language: &str) -> String {
    let fallback_chain = crate::language::get_language_fallback_chain(language);
    
    fallback_chain.iter()
        .find_map(|lang| text_map.get(lang))
        .unwrap_or(&"Missing text".to_string())
        .clone()
}


