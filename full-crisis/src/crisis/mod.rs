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
        _Interior_PlayableCrises::get(file_path)
    }
    
    pub fn iter() -> impl Iterator<Item = std::borrow::Cow<'static, str>> {
        _Interior_PlayableCrises::iter()
    }
}