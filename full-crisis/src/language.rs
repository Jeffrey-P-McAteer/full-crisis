use isolang::Language;

/// Available languages in the game with their display names
pub fn get_available_languages() -> Vec<(String, String)> {
    vec![
        ("eng".to_string(), "English".to_string()),
        ("spa".to_string(), "Español".to_string()),
        ("fra".to_string(), "Français".to_string()),
        ("deu".to_string(), "Deutsch".to_string()),
        ("ita".to_string(), "Italiano".to_string()),
        ("por".to_string(), "Português".to_string()),
        ("rus".to_string(), "Русский".to_string()),
        ("jpn".to_string(), "日本語".to_string()),
        ("kor".to_string(), "한국어".to_string()),
        ("zho".to_string(), "中文".to_string()),
    ]
}

/// Convert ISO 639-1 (2-letter) to ISO 639-3 (3-letter) language code
pub fn convert_language_code(input: &str) -> String {
    if input.len() == 2 {
        // Try to convert 2-letter to 3-letter
        if let Some(lang) = Language::from_639_1(input) {
            return lang.to_639_3().to_string();
        }
    } else if input.len() == 3 {
        // Validate 3-letter code
        if Language::from_639_3(input).is_some() {
            return input.to_string();
        }
    }
    
    // Fallback to English
    "eng".to_string()
}

/// Detect system language based on platform
pub fn detect_system_language() -> String {
    #[cfg(target_arch = "wasm32")]
    {
        detect_browser_language()
    }
    
    #[cfg(not(target_arch = "wasm32"))]
    {
        detect_os_language()
    }
}

#[cfg(target_arch = "wasm32")]
fn detect_browser_language() -> String {
    use wasm_bindgen::JsCast;
    use web_sys::{window, Navigator};
    
    if let Some(window) = window() {
        let navigator = window.navigator();
        
        // Try to get language from navigator.language
        if let Some(language) = navigator.language() {
            let lang_code = parse_language_tag(&language);
            let converted = convert_language_code(&lang_code);
            eprintln!("Browser language detected: {} -> {}", language, converted);
            return converted;
        }
        
        // Try to get languages from navigator.languages
        if let Ok(languages) = navigator.languages() {
            if languages.length() > 0 {
                if let Some(first_lang) = languages.get(0).as_string() {
                    let lang_code = parse_language_tag(&first_lang);
                    let converted = convert_language_code(&lang_code);
                    eprintln!("Browser languages[0] detected: {} -> {}", first_lang, converted);
                    return converted;
                }
            }
        }
    }
    
    eprintln!("Browser language detection failed, defaulting to English");
    "eng".to_string()
}

#[cfg(not(target_arch = "wasm32"))]
fn detect_os_language() -> String {
    if let Some(locale) = sys_locale::get_locale() {
        let lang_code = parse_language_tag(&locale);
        let converted = convert_language_code(&lang_code);
        eprintln!("OS locale detected: {} -> {}", locale, converted);
        return converted;
    }
    
    eprintln!("OS locale detection failed, defaulting to English");
    "eng".to_string()
}

/// Parse language tag (e.g., "en-US", "es-ES") to get just the language part
fn parse_language_tag(tag: &str) -> String {
    tag.split('-').next().unwrap_or("en").to_lowercase()
}

/// Get display name for a language code
pub fn get_language_display_name(code: &str) -> String {
    get_available_languages()
        .into_iter()
        .find(|(lang_code, _)| lang_code == code)
        .map(|(_, display_name)| display_name)
        .unwrap_or_else(|| {
            // Try to use isolang for unknown codes
            if let Some(lang) = Language::from_639_3(code) {
                lang.to_name().to_string()
            } else {
                format!("Unknown ({})", code)
            }
        })
}

/// Check if a language code is supported
pub fn is_language_supported(code: &str) -> bool {
    get_available_languages()
        .iter()
        .any(|(lang_code, _)| lang_code == code)
}

/// Get fallback language chain for a given language
pub fn get_language_fallback_chain(primary_language: &str) -> Vec<String> {
    let mut chain = vec![primary_language.to_string()];
    
    // Add English as fallback if not already primary
    if primary_language != "eng" {
        chain.push("eng".to_string());
    }
    
    chain
}