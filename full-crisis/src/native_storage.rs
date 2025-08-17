use std::hash::Hasher;
use twox_hash::XxHash64;
use serde::{Serialize, Deserialize};

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

pub fn get_struct<T>(name: &str) -> Option<T> 
where 
  T: for<'de> Deserialize<'de>,
{
  if let Some(proj_dirs) = directories::ProjectDirs::from("com.jmcateer.full-crisis", "Full-Crisis",  "Full-Crisis") {
    let cache_dir = proj_dirs.cache_dir();
    if let Err(e) = std::fs::create_dir_all(cache_dir) {
      eprintln!("Error creating {:?}: {:?}", cache_dir, e);
    }
    let mut attr_file = cache_dir.to_path_buf();
    attr_file.push(attr_hash_name(name));
    if attr_file.exists() {
      match std::fs::read_to_string(&attr_file) {
        Ok(content) => {
          match serde_json::from_str::<T>(&content) {
            Ok(data) => return Some(data),
            Err(e) => {
              eprintln!("Error deserializing struct \"{}\" from {:?}: {:?}", name, &attr_file, e);
            }
          }
        },
        Err(e) => {
          eprintln!("Error reading the struct \"{}\" from {:?}: {:?}", name, &attr_file, e);
        }
      }
    }
  }
  None
}

pub fn set_struct<T>(name: &str, value: &T) 
where 
  T: Serialize,
{
  if let Some(proj_dirs) = directories::ProjectDirs::from("com.jmcateer.full-crisis", "Full-Crisis",  "Full-Crisis") {
    let cache_dir = proj_dirs.cache_dir();
    if let Err(e) = std::fs::create_dir_all(cache_dir) {
      eprintln!("Error creating {:?}: {:?}", cache_dir, e);
    }
    let mut attr_file = cache_dir.to_path_buf();
    attr_file.push(attr_hash_name(name));

    match serde_json::to_string(value) {
      Ok(serialized) => {
        if let Err(e) = std::fs::write(&attr_file, serialized) {
          eprintln!("Error writing the struct \"{}\" to {:?}: {:?}", name, &attr_file, e);
        }
      },
      Err(e) => {
        eprintln!("Error serializing struct \"{}\": {:?}", name, e);
      }
    }
  }
}

