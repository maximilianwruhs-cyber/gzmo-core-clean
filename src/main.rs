//! GZMO Clean Core CLI
//!
//! Command-line interface for the clean GZMO architecture.

use gzmo_core_clean::cli::{CliArgs, CommandRunner, Repl};

#[tokio::main]
async fn main() {
    gzmo_core_clean::welcome();

    let args = CliArgs::parse();

    if args.verbose {
        println!("Verbose mode enabled");
    }

    // Load configuration if specified
    if let Some(config_path) = &args.config {
        println!("Loading configuration from: {}", config_path);
    }

    // Execute command if specified
    if let Some(cmd) = CommandRunner::parse(&args) {
        let runner = CommandRunner::new();
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