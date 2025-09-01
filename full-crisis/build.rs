use std::env;
use std::fs;
use std::path::Path;
use std::process::Command;

fn main() {
    println!("cargo:rerun-if-changed=../audio/");
    
    let out_dir = env::var("OUT_DIR").unwrap();
    let audio_build_dir = Path::new(&out_dir).join("audio");
    
    // Create audio build directory
    fs::create_dir_all(&audio_build_dir).expect("Failed to create audio build directory");
    
    // Generate intro chime
    let intro_chime_path = audio_build_dir.join("intro_chime.wav");
    let audio_script_path = Path::new("../audio/generate_intro_chime.py");
    
    // Check if the script exists
    if !audio_script_path.exists() {
        println!("cargo:warning=Audio script not found at {:?}, skipping audio generation", audio_script_path);
        return;
    }
    
    println!("Running audio generation script...");
    let status = Command::new("uv")
        .args(&["run", "generate_intro_chime.py", intro_chime_path.to_str().unwrap()])
        .current_dir("../audio")
        .status();
    
    match status {
        Ok(status) if status.success() => {
            println!("Audio generation completed successfully");
        }
        Ok(status) => {
            println!("cargo:warning=Audio generation script failed with exit code: {:?}", status.code());
        }
        Err(e) => {
            println!("cargo:warning=Failed to execute audio generation script: {}", e);
        }
    }
    
    // Set environment variable for the embed macro
    println!("cargo:rustc-env=AUDIO_BUILD_DIR={}", audio_build_dir.display());
    
    println!("Audio files generated successfully");
}