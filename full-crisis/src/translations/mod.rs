pub mod keys;
pub mod data;

pub use keys::TranslationKey;
pub use data::Translation;

use std::collections::HashMap;

/// Central translation manager
pub struct TranslationManager {
    translations: HashMap<TranslationKey, HashMap<String, String>>,
}

impl TranslationManager {
    pub fn new() -> Self {
        let mut manager = Self {
            translations: HashMap::new(),
        };
        manager.load_builtin_translations();
        manager
    }
    
    /// Get translated text for a key in the specified language
    pub fn get(&self, key: TranslationKey, language: &str) -> String {
        let fallback_chain = crate::language::get_language_fallback_chain(language);
        
        if let Some(translations) = self.translations.get(&key) {
            for lang in fallback_chain {
                if let Some(text) = translations.get(&lang) {
                    return text.clone();
                }
            }
        }
        
        // Final fallback - return the key name as a readable string
        format!("{:?}", key)
    }
    
    /// Get translated text with variable substitution
    pub fn get_with_vars(&self, key: TranslationKey, language: &str, vars: &HashMap<String, String>) -> String {
        let mut text = self.get(key, language);
        
        for (var_name, var_value) in vars {
            text = text.replace(&format!("{{{}}}", var_name), var_value);
        }
        
        text
    }
    
    /// Load all builtin translations
    fn load_builtin_translations(&mut self) {
        let translations = data::get_builtin_translations();
        
        // Load translations into the manager
        for translation in translations {
            self.translations.insert(translation.key, translation.translations);
        }
    }
}

/// Global translation manager instance
static TRANSLATION_MANAGER: once_cell::sync::Lazy<TranslationManager> = once_cell::sync::Lazy::new(|| {
    TranslationManager::new()
});

/// Get the global translation manager
pub fn get_translation_manager() -> &'static TranslationManager {
    &TRANSLATION_MANAGER
}

/// Convenience function to get translated text
pub fn t(key: TranslationKey, language: &str) -> String {
    get_translation_manager().get(key, language)
}

/// Convenience function to get translated text with variables
pub fn t_vars(key: TranslationKey, language: &str, vars: &HashMap<String, String>) -> String {
    get_translation_manager().get_with_vars(key, language, vars)
}