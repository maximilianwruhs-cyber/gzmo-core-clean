//! GZMO Clean Core CLI
//!
//! Command-line interface for the clean GZMO architecture.

use gzmo_core_clean::cli::{CliArgs, CommandRunner, Repl};
use gzmo_core_clean::config::{load_from_file, generate_default_config, Config};
use std::path::Path;

#[tokio::main]
async fn main() {
    gzmo_core_clean::welcome();

    let args = CliArgs::parse();

    if args.verbose {
        println!("Verbose mode enabled");
    }

    // Load configuration
    let config = if let Some(config_path) = &args.config {
        println!("Loading configuration from: {}", config_path);
        match load_from_file(Path::new(config_path)) {
            Ok(cfg) => {
                println!("Configuration loaded successfully");
                Some(cfg)
            }
            Err(e) => {
                eprintln!("Warning: Failed to load config: {}", e);
                eprintln!("Using default configuration");
                Some(Config::default())
            }
        }
    } else {
        // Try default config locations
        let default_paths = ["config.toml", "gzmo.toml", "~/.config/gzmo/config.toml"];
        let mut found_config = None;
        for path in &default_paths {
            let expanded = if path.starts_with("~/") {
                if let Some(home) = dirs::home_dir() {
                    home.join(&path[2..]).to_string_lossy().to_string()
                } else {
                    path.to_string()
                }
            } else {
                path.to_string()
            };
            
            if Path::new(&expanded).exists() {
                println!("Loading configuration from: {}", expanded);
                if let Ok(cfg) = load_from_file(Path::new(&expanded)) {
                    found_config = Some(cfg);
                    break;
                }
            }
        }
        found_config
    };

    // Execute command if specified
    if let Some(cmd) = CommandRunner::parse(&args) {
        let runner = if let Some(cfg) = config {
            CommandRunner::with_config(cfg)
        } else {
            CommandRunner::new()
        };
        
        if let Err(e) = runner.execute(cmd).await {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    }

    // Start REPL if interactive
    if args.interactive {
        let repl = Repl::new();
        repl.run();
    }
}

/// Get home directory cross-platform
mod dirs {
    pub fn home_dir() -> Option<std::path::PathBuf> {
        #[cfg(unix)]
        {
            std::env::var("HOME").ok().map(std::path::PathBuf::from)
        }
        #[cfg(windows)]
        {
            std::env::var("USERPROFILE").ok().map(std::path::PathBuf::from)
        }
    }
}