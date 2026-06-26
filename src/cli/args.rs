//! CLI Arguments
//!
//! Command-line argument parsing.

/// CLI arguments
#[derive(Debug, Clone)]
pub struct CliArgs {
    /// Configuration file path
    pub config: Option<String>,
    /// Command to run
    pub command: Option<String>,
    /// Arguments for the command
    pub command_args: Vec<String>,
    /// Enable verbose logging
    pub verbose: bool,
    /// Start interactive mode
    pub interactive: bool,
}

impl CliArgs {
    /// Parse from std::env::args
    pub fn parse() -> Self {
        let mut args = std::env::args().skip(1);
        let mut config = None;
        let mut verbose = false;
        let mut interactive = false;
        let mut command = None;
        let mut command_args = Vec::new();

        while let Some(arg) = args.next() {
            match arg.as_str() {
                "-c" | "--config" => {
                    config = args.next();
                }
                "-v" | "--verbose" => {
                    verbose = true;
                }
                "-i" | "--interactive" => {
                    interactive = true;
                }
                "-h" | "--help" => {
                    Self::print_help();
                    std::process::exit(0);
                }
                _ if arg.starts_with('-') => {
                    eprintln!("Unknown flag: {}", arg);
                }
                _ if command.is_none() => {
                    command = Some(arg);
                }
                _ => {
                    command_args.push(arg);
                }
            }
        }

        // Default to interactive if no command given
        if command.is_none() && !interactive {
            interactive = true;
        }

        Self {
            config,
            command,
            command_args,
            verbose,
            interactive,
        }
    }

    fn print_help() {
        println!(r#"GZMO Clean Core

USAGE:
    gzmo-clean [OPTIONS] [COMMAND] [ARGS...]

OPTIONS:
    -c, --config <FILE>     Configuration file path
    -v, --verbose           Enable verbose logging
    -i, --interactive       Start interactive mode (default)
    -h, --help              Print this help

COMMANDS:
    run                     Run the main loop
    pedagogy                Start tutoring session
    etl                     Run ETL batch job
    telemetry               Show telemetry dashboard
    self-improve            Run self-improving loop
    config <get|set>        Manage configuration

EXAMPLES:
    gzmo-clean --config my.toml run
    gzmo-clean -v pedagogy --subject calculus
"#);
    }
}

impl Default for CliArgs {
    fn default() -> Self {
        Self {
            config: None,
            command: None,
            command_args: vec![],
            verbose: false,
            interactive: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_are_sensible() {
        let args = CliArgs::default();
        assert!(args.interactive);
        assert!(!args.verbose);
    }
}