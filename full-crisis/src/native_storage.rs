use std::hash::Hasher;
use std::time::SystemTime;
use twox_hash::XxHash64;

fn attr_hash_name(name: &str) -> String {
  let mut hasher = XxHash64::with_seed(0);
  hasher.write(name.as_bytes());
  format!("{:016x}", hasher.finish())
}

pub fn get_attr(name: &str) -> Option<String> {
  if let Some(proj_dirs) = directories::ProjectDirs::from("com.jmcateer.full-crisis", "Full-Crisis",  "Full-Crisis") {
    let cache_dir = proj_dirs.cache_dir();
    if let Err(e) = std::fs::create_dir_all(cache_dir) {
      eprintln!("Error creating {:?}: {:?}", cache_dir, e);
    }
    let mut attr_file = cache_dir.to_path_buf();
    attr_file.push(attr_hash_name(name));
    if attr_file.exists() {
      match std::fs::read_to_string(&attr_file) {
        Ok(content) => return Some(content),
        Err(e) => {
          eprintln!("Error reading the attribute \"{}\" from {:?}: {:?}", name, &attr_file, e);
        }
      }
    }

  }
  None
}

pub fn set_attr(name: &str, value: &str) {
  if let Some(proj_dirs) = directories::ProjectDirs::from("com.jmcateer.full-crisis", "Full-Crisis",  "Full-Crisis") {
    let cache_dir = proj_dirs.cache_dir();
    if let Err(e) = std::fs::create_dir_all(cache_dir) {
      eprintln!("Error creating {:?}: {:?}", cache_dir, e);
    }
    let mut attr_file = cache_dir.to_path_buf();
    attr_file.push(attr_hash_name(name));

    if let Err(e) = std::fs::write(&attr_file, value) {
      eprintln!("Error writing the attribute \"{}\" to {:?}: {:?}", name, &attr_file, e);
    }

  }
}


pub fn time_now() -> SystemTime {
    SystemTime::now()
}

