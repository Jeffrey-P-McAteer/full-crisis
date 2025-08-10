
#[derive(rust_embed::Embed)]
#[folder = "../playable-crises/"]
struct _Interior_PlayableCrises;

pub struct PlayableCrises;
impl PlayableCrises {
  pub fn get(file_path: &str) -> Option<rust_embed::EmbeddedFile> {
    // TODO release mode FS override (IF exists)
    _Interior_PlayableCrises::get(file_path)
  }
  pub fn iter() -> impl Iterator<Item = std::borrow::Cow<'static, str>> {
    // TODO release mode FS override (IF exists)
    _Interior_PlayableCrises::iter()
  }
}

// TODO better interface -_-
pub fn get_crisis_names() -> Vec<String> {
  let mut names = vec![];
  for pc in PlayableCrises::iter() {
    names.push(pc.to_string());
  }
  return names;
}


pub fn get_saved_crisis_names() -> Vec<String> {
  let mut names = vec![];
  names.push("TODO read saved game file data".to_string());
  return names;
}


