pub mod types;
pub mod operations;

pub use types::*;
pub use operations::*;

#[derive(rust_embed::Embed)]
#[folder = "$CARGO_MANIFEST_DIR/../playable-crises/"]
struct _Interior_PlayableCrises;

pub struct PlayableCrises;
impl PlayableCrises {
    pub fn get(file_path: &str) -> Option<rust_embed::EmbeddedFile> {
        // Try to get from embedded files first
        if let Some(embedded_file) = _Interior_PlayableCrises::get(file_path) {
            return Some(embedded_file);
        }
        
        // On non-WASM platforms, also check the crises folder
        #[cfg(not(target_arch = "wasm32"))]
        {
            Self::get_from_crises_folder(file_path)
        }
        
        #[cfg(target_arch = "wasm32")]
        None
    }
    
    pub fn iter() -> Box<dyn Iterator<Item = std::borrow::Cow<'static, str>>> {
        let mut all_files = std::collections::HashSet::new();
        
        // Add embedded files
        for file in _Interior_PlayableCrises::iter() {
            all_files.insert(file);
        }
        
        // On non-WASM platforms, also add files from crises folder
        #[cfg(not(target_arch = "wasm32"))]
        {
            if let Some(folder_files) = Self::iter_crises_folder() {
                for file in folder_files {
                    all_files.insert(file);
                }
            }
        }
        
        Box::new(all_files.into_iter())
    }
    
    #[cfg(not(target_arch = "wasm32"))]
    fn get_from_crises_folder(file_path: &str) -> Option<rust_embed::EmbeddedFile> {
        let settings = crate::gui::GameWindow::load_settings();
        let crises_folder = std::path::Path::new(&settings.game_crises_folder);
        let full_path = crises_folder.join(file_path);
        
        if let Ok(data) = std::fs::read(full_path) {
            // Create a fake EmbeddedFile-like structure
            Some(rust_embed::EmbeddedFile {
                data: std::borrow::Cow::Owned(data),
                metadata: rust_embed::Metadata::__rust_embed_new(
                    [0; 32], // hash
                    None, // last_modified
                    None, // created
                ),
            })
        } else {
            None
        }
    }
    
    #[cfg(not(target_arch = "wasm32"))]
    fn iter_crises_folder() -> Option<Vec<std::borrow::Cow<'static, str>>> {
        let settings = crate::gui::GameWindow::load_settings();
        let crises_folder = std::path::Path::new(&settings.game_crises_folder);
        
        if !crises_folder.exists() {
            return None;
        }
        
        let mut files = Vec::new();
        if let Ok(entries) = Self::walk_directory(crises_folder, crises_folder) {
            files.extend(entries);
        }
        
        Some(files)
    }
    
    #[cfg(not(target_arch = "wasm32"))]
    fn walk_directory(base_path: &std::path::Path, current_path: &std::path::Path) -> Result<Vec<std::borrow::Cow<'static, str>>, std::io::Error> {
        let mut files = Vec::new();
        
        if let Ok(entries) = std::fs::read_dir(current_path) {
            for entry in entries {
                let entry = entry?;
                let path = entry.path();
                
                if path.is_dir() {
                    // Recursively walk subdirectories
                    files.extend(Self::walk_directory(base_path, &path)?);
                } else if path.is_file() {
                    // Convert absolute path to relative path from base
                    if let Ok(relative_path) = path.strip_prefix(base_path) {
                        let path_str = relative_path.to_string_lossy().replace('\\', "/");
                        files.push(std::borrow::Cow::Owned(path_str));
                    }
                }
            }
        }
        
        Ok(files)
    }
}
