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
    use std::io::IsTerminal;
    if !io::stdin().is_terminal() {
        eprintln!("Error: confirmation required but stdin is not a terminal. Use -y/--yes to skip.");
        std::process::exit(1);
    }
    eprint!("{prompt} [y/N] ");
    io::stderr().flush().ok();
    let mut input = String::new();
    io::stdin().read_line(&mut input).ok();
    matches!(input.trim().to_lowercase().as_str(), "y" | "yes")
}
