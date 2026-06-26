//! Commands
//!
//! Command execution.

use crate::cli::args::CliArgs;

/// Commands
#[derive(Debug, Clone)]
pub enum Command {
    Run,
    Pedagogy { subject: Option<String> },
    Etl,
    Telemetry,
    SelfImprove,
    Config { action: ConfigAction },
}

#[derive(Debug, Clone)]
pub enum ConfigAction {
    Get { key: String },
    Set { key: String, value: String },
}

/// Command runner
pub struct CommandRunner;

impl CommandRunner {
    pub fn new() -> Self {
        Self
    }

    /// Parse command from CLI args
    pub fn parse(args: &CliArgs) -> Option<Command> {
        let cmd = args.command.as_ref()?;

        Some(match cmd.as_str() {
            "run" => Command::Run,
            "pedagogy" => Command::Pedagogy {
                subject: args.command_args.get(0).cloned(),
            },
            "etl" => Command::Etl,
            "telemetry" => Command::Telemetry,
            "self-improve" => Command::SelfImprove,
            "config" => {
                let action = match args.command_args.get(0).map(|s| s.as_str()) {
                    Some("get") => ConfigAction::Get {
                        key: args.command_args.get(1).cloned().unwrap_or_default(),
                    },
                    Some("set") => ConfigAction::Set {
                        key: args.command_args.get(1).cloned().unwrap_or_default(),
                        value: args.command_args.get(2).cloned().unwrap_or_default(),
                    },
                    _ => ConfigAction::Get { key: String::new() },
                };
                Command::Config { action }
            }
            _ => return None,
        })
    }

    /// Execute a command
    pub async fn execute(&self, cmd: Command) -> Result<(), CommandError> {
        match cmd {
            Command::Run => {
                println!("Running main loop...");
                // In production: start the main modulation loop
                Ok(())
            }
            Command::Pedagogy { subject } => {
                let subj = subject.unwrap_or_else(|| "general".to_string());
                println!("Starting pedagogy session: {}", subj);
                // In production: start tutoring session
                Ok(())
            }
            Command::Etl => {
                println!("Running ETL batch job...");
                // In production: run extraction/verification/promotion
                Ok(())
            }
            Command::Telemetry => {
                println!("Starting telemetry dashboard...");
                // In production: show real-time dashboard
                Ok(())
            }
            Command::SelfImprove => {
                println!("Running self-improving loop...");
                // In production: start feedback/learning loop
                Ok(())
            }
            Command::Config { action } => {
                match action {
                    ConfigAction::Get { key } => {
                        println!("Config key: {} = [not implemented]", key);
                    }
                    ConfigAction::Set { key, value } => {
                        println!("Config key: {} = {}", key, value);
                    }
                }
                Ok(())
            }
        }
    }
}

impl Default for CommandRunner {
    fn default() -> Self {
        Self::new()
    }
}

/// Command error
#[derive(Debug)]
pub enum CommandError {
    Execution(String),
    NotFound(String),
}

impl std::fmt::Display for CommandError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CommandError::Execution(s) => write!(f, "execution error: {}", s),
            CommandError::NotFound(s) => write!(f, "command not found: {}", s),
        }
    }
}

impl std::error::Error for CommandError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_run_command() {
        let args = CliArgs {
            command: Some("run".to_string()),
            ..Default::default()
        };
        let cmd = CommandRunner::parse(&args);
        assert!(matches!(cmd, Some(Command::Run)));
    }

    #[test]
    fn parses_pedagogy_with_subject() {
        let args = CliArgs {
            command: Some("pedagogy".to_string()),
            command_args: vec!["rust".to_string()],
            ..Default::default()
        };
        let cmd = CommandRunner::parse(&args);
        match cmd {
            Some(Command::Pedagogy { subject: Some(s) }) => {
                assert_eq!(s, "rust");
            }
            _ => panic!("Expected Pedagogy command"),
        }
    }

    #[test]
    fn parses_config_get() {
        let args = CliArgs {
            command: Some("config".to_string()),
            command_args: vec!["get".to_string(), "modulation.temp_min".to_string()],
            ..Default::default()
        };
        let cmd = CommandRunner::parse(&args);
        assert!(matches!(cmd, Some(Command::Config { .. })));
    }
}