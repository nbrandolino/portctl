// Config file (~/.config/portctl/config.toml) load/save helpers
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
    match std::env::var("HOME") {
        Ok(home) => PathBuf::from(home).join(".config").join("portctl").join("config.toml"),
        Err(_) => {
            eprintln!("Error: $HOME is not set; cannot determine config file location");
            std::process::exit(1);
        }
    }
}

impl Config {
    pub fn load() -> Self {
        let mut cfg = Self::load_file();

        // Environment variables override the config file (useful for containers)
        if let Ok(url) = std::env::var("PORTCTL_URL") {
            if !url.is_empty() {
                cfg.portainer_url = Some(url);
            }
        }
        if let Ok(token) = std::env::var("PORTCTL_TOKEN") {
            if !token.is_empty() {
                cfg.api_token = Some(token);
            }
        }

        cfg
    }

    // Load only the config file, ignoring environment overrides.
    // Used by `set-url`/`set-token` so env values are never persisted to disk.
    pub fn load_file() -> Self {
        let path = config_path();
        if !path.exists() {
            return Config::default();
        }
        let contents = match fs::read_to_string(&path) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("Warning: could not read config file '{}': {e}", path.display());
                return Config::default();
            }
        };
        match toml::from_str(&contents) {
            Ok(c) => c,
            Err(e) => {
                eprintln!("Warning: config file '{}' is invalid and will be ignored: {e}", path.display());
                Config::default()
            }
        }
    }

    pub fn save(&self) {
        let path = config_path();
        if let Some(dir) = path.parent() {
            ensure_config_dir_exists(dir);
        }
        let contents = toml::to_string(self).unwrap_or_default();

        // The config file holds the API token in plaintext, so restrict it to the
        // owner (0600). On Unix, create it with those permissions from the start so
        // the token is never briefly readable by others.
        let write_result = {
            #[cfg(unix)]
            {
                use std::io::Write;
                use std::os::unix::fs::OpenOptionsExt;
                fs::OpenOptions::new()
                    .write(true)
                    .create(true)
                    .truncate(true)
                    .mode(0o600)
                    .open(&path)
                    .and_then(|mut f| f.write_all(contents.as_bytes()))
            }
            #[cfg(not(unix))]
            {
                fs::write(&path, contents)
            }
        };
        if let Err(e) = write_result {
            eprintln!("Error: failed to write config file '{}': {e}", path.display());
            std::process::exit(1);
        }

        // An existing file keeps its old mode through O_CREAT, so tighten it explicitly.
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            if let Err(e) = fs::set_permissions(&path, fs::Permissions::from_mode(0o600)) {
                eprintln!("Warning: failed to restrict permissions on config file '{}': {e}", path.display());
            }
        }
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
