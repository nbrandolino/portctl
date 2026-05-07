use std::fs;
use std::io::{self, Read, Write};
use std::path::Path;

pub fn ensure_config_dir_exists(config_path: &Path) {
    if !config_path.exists() {
        if let Err(e) = fs::create_dir_all(config_path) {
            eprintln!("Error: failed to create config directory '{}': {e}", config_path.display());
            std::process::exit(1);
        }
    }
}

/// Reads a Docker multiplexed stream (8-byte header + payload per chunk) and
/// writes stdout chunks to stdout, stderr chunks (type 2) to stderr.
pub fn pipe_docker_stream(mut stream: impl Read) {
    let mut header = [0u8; 8];
    loop {
        match stream.read_exact(&mut header) {
            Ok(()) => {}
            Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => break,
            Err(e) => {
                eprintln!("Error reading stream: {e}");
                std::process::exit(1);
            }
        }

        let payload_len = u32::from_be_bytes([header[4], header[5], header[6], header[7]]) as usize;
        const MAX_PAYLOAD: usize = 100 * 1024 * 1024;
        if payload_len > MAX_PAYLOAD {
            eprintln!("Error: stream payload too large ({payload_len} bytes), aborting.");
            std::process::exit(1);
        }
        let mut payload = vec![0u8; payload_len];

        match stream.read_exact(&mut payload) {
            Ok(()) => {}
            Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => break,
            Err(e) => {
                eprintln!("Error reading stream: {e}");
                std::process::exit(1);
            }
        }

        if let Ok(text) = std::str::from_utf8(&payload) {
            if header[0] == 2 {
                eprint!("{text}");
            } else {
                print!("{text}");
            }
        }
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
    if let Err(e) = io::stdin().read_line(&mut input) {
        eprintln!("Error: failed to read input: {e}");
        std::process::exit(1);
    }
    matches!(input.trim().to_lowercase().as_str(), "y" | "yes")
}
