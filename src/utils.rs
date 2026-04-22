use std::fs;
use std::io::{self, Write};
use std::path::Path;

pub fn ensure_config_dir_exists(config_path: &Path) {
    if !config_path.exists() {
        fs::create_dir_all(config_path)
            .expect("Failed to create config directory");
    }
}

pub fn confirm(prompt: &str) -> bool {
    eprint!("{prompt} [y/N] ");
    io::stderr().flush().ok();
    let mut input = String::new();
    io::stdin().read_line(&mut input).ok();
    matches!(input.trim().to_lowercase().as_str(), "y" | "yes")
}
