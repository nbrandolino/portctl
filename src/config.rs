use crate::utils::ensure_config_dir_exists;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Config {
    pub portainer_url: Option<String>,
    pub api_token: Option<String>,
}

fn config_path() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    PathBuf::from(home).join(".config").join("portctl").join("config.toml")
}

impl Config {
    pub fn load() -> Self {
        let path = config_path();
        if !path.exists() {
            return Config::default();
        }
        let contents = fs::read_to_string(&path).unwrap_or_default();
        toml::from_str(&contents).unwrap_or_default()
    }

    pub fn save(&self) {
        let path = config_path();
        ensure_config_dir_exists(path.parent().expect("config path has no parent directory"));
        let contents = toml::to_string(self).expect("Failed to serialize config");
        fs::write(&path, contents).expect("Failed to write config file");
    }

    pub fn set_url(&mut self, url: String) {
        self.portainer_url = Some(url);
        self.save();
    }

    pub fn set_token(&mut self, token: String) {
        self.api_token = Some(token);
        self.save();
    }
}
