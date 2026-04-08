use std::fs;
use std::path::Path;

pub fn ensure_config_dir_exists(config_path: &Path) {
    if !config_path.exists() {
        fs::create_dir_all(config_path)
            .expect("Failed to create config directory");
    }
}
