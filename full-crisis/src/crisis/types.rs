use serde::{Serialize, Deserialize, Deserializer};
use serde::de::{self, Visitor};
use std::collections::HashMap;
use std::fmt;

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
    pub text_input: Option<CrisisTextInput>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrisisTextInput {
    pub variable_name: String,
    pub input_type: TextInputType,
    pub placeholder: Option<HashMap<String, String>>,
    pub min_length: Option<usize>,
    pub max_length: Option<usize>,
    pub min_value: Option<i32>,
    pub max_value: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TextInputType {
    Text,
    Number,
}

#[derive(Debug, Clone, Serialize)]
#[serde(untagged)]
pub enum SpeakingCharacterImage {
    Single(String),
    Animation(Vec<String>),
}

impl<'de> serde::Deserialize<'de> for SpeakingCharacterImage {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_any(SpeakingCharacterImageVisitor)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrisisScene {
    pub text: HashMap<String, String>,
    #[serde(default)]
    pub choices: Vec<CrisisChoice>,
    pub continue_in_subfolder: Option<String>,
    pub background_image: Option<String>,
    #[serde(default)]
    pub speaking_character_image: Option<SpeakingCharacterImage>,
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
    pub text_inputs: HashMap<String, String>,
    pub language: String,
    pub crisis_id: String,
    pub template_name: String,
}

impl GameState {
    pub fn new(crisis_id: String, language: String, template_name: String) -> Self {
        Self {
            current_scene: String::new(),
            character_name: String::new(),
            character_type: None,
            variables: HashMap::new(),
            text_inputs: HashMap::new(),
            language,
            crisis_id,
            template_name,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SavedGame {
    pub save_name: String,
    pub crisis_name: String,
    pub character_name: String,
    pub current_scene: String,
    pub variables: HashMap<String, i32>,
    #[serde(default)]
    pub text_inputs: HashMap<String, String>,
    pub character_type: Option<String>,
    pub language: String,
    pub save_timestamp: String,
    pub template_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SavedGames {
    pub saves: Vec<SavedGame>,
}

impl SavedGames {
    pub fn add_save(&mut self, save: SavedGame) {
        // Remove any existing save with the same name
        self.saves.retain(|s| s.save_name != save.save_name);
        // Add the new save
        self.saves.push(save);
        // Sort by timestamp (newest first)
        self.saves.sort_by(|a, b| b.save_timestamp.cmp(&a.save_timestamp));
    }
    
    pub fn get_save_names(&self) -> Vec<String> {
        self.saves.iter().map(|s| {
            // Convert timestamp to readable format
            let readable_time = if let Ok(timestamp) = s.save_timestamp.parse::<u64>() {
                let datetime = std::time::UNIX_EPOCH + std::time::Duration::from_secs(timestamp);
                format!("{:?}", datetime).split_whitespace().take(2).collect::<Vec<_>>().join(" ")
            } else {
                s.save_timestamp.clone()
            };
            format!("{} - {} ({})", s.save_name, s.crisis_name, readable_time)
        }).collect()
    }
    
    pub fn get_save_by_display_name(&self, display_name: &str) -> Option<&SavedGame> {
        // Extract save name from display format "SaveName - CrisisName (Date)"
        if let Some(save_name) = display_name.split(" - ").next() {
            self.saves.iter().find(|s| s.save_name == save_name)
        } else {
            None
        }
    }

    pub fn delete_save_by_display_name(&mut self, display_name: &str) -> bool {
        // Extract save name from display format "SaveName - CrisisName (Date)"
        if let Some(save_name) = display_name.split(" - ").next() {
            let original_len = self.saves.len();
            self.saves.retain(|s| s.save_name != save_name);
            self.saves.len() != original_len
        } else {
            false
        }
    }
}

struct SpeakingCharacterImageVisitor;

impl<'de> Visitor<'de> for SpeakingCharacterImageVisitor {
    type Value = SpeakingCharacterImage;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a string or an array of strings")
    }

    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(SpeakingCharacterImage::Single(value.to_string()))
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: de::SeqAccess<'de>,
    {
        let mut images = Vec::new();
        while let Some(image) = seq.next_element::<String>()? {
            images.push(image);
        }
        if images.is_empty() {
            Err(de::Error::custom("Image array cannot be empty"))
        } else {
            Ok(SpeakingCharacterImage::Animation(images))
        }
    }
}

