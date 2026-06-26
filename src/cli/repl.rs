//! REPL
//!
//! Interactive command loop.

/// REPL commands
#[derive(Debug, Clone)]
pub enum ReplCommand {
    /// Run a skill
    Skill { name: String, args: Vec<String> },
    /// Show status
    Status,
    /// Show telemetry
    Telemetry,
    /// Start pedagogy session
    Learn { subject: String },
    /// Help
    Help,
    /// Quit
    Quit,
    /// Unknown command
    Unknown(String),
}

/// Read-eval-print loop
pub struct Repl;

impl Repl {
    pub fn new() -> Self {
        Self
    }

    /// Run the REPL
    pub fn run(&self) {
        println!("GZMO Clean Core v0.1.0");
        println!("Type 'help' for commands, 'quit' to exit.");
        println!();

        loop {
            print!("gzmo> ");
            let _ = std::io::Write::flush(&mut std::io::stdout());

            let mut input = String::new();
            match std::io::stdin().read_line(&mut input) {
                Ok(_) => {
                    let cmd = Self::parse(&input);
                    if let ReplCommand::Quit = cmd {
                        println!("Goodbye!");
                        break;
                    }
                    self.execute(cmd);
                }
                Err(e) => {
                    eprintln!("Error reading input: {}", e);
                }
            }
        }
    }

    /// Parse input into command
    pub fn parse(input: &str) -> ReplCommand {
        let parts: Vec<_> = input.trim().split_whitespace().collect();
        if parts.is_empty() {
            return ReplCommand::Unknown("empty".to_string());
        }

        match parts[0] {
            "quit" | "exit" | "q" => ReplCommand::Quit,
            "help" | "h" | "?" => ReplCommand::Help,
            "status" | "s" => ReplCommand::Status,
            "telemetry" | "t" => ReplCommand::Telemetry,
            "learn" | "l" => {
                let subject = parts.get(1).map(|&s| s.to_string()).unwrap_or_default();
                ReplCommand::Learn { subject }
            }
            "skill" => {
                let name = parts.get(1).map(|&s| s.to_string()).unwrap_or_default();
                let args = parts[2..].iter().map(|&s| s.to_string()).collect();
                ReplCommand::Skill { name, args }
            }
            _ => ReplCommand::Unknown(parts.join(" ")),
        }
    }

    /// Execute a command
    fn execute(&self, cmd: ReplCommand) {
        match cmd {
            ReplCommand::Help => {
                println!("Commands:");
                println!("  help, h, ?       - Show this help");
                println!("  status, s          - Show system status");
                println!("  telemetry, t       - Show telemetry");
                println!("  learn <subject>, l - Start learning session");
                println!("  skill <name> [...] - Run a skill");
                println!("  quit, q            - Exit");
            }
            ReplCommand::Status => {
                println!("Status: Running");
                println!("  Module: Clean Core");
                println!("  Version: 0.1.0");
            }
            ReplCommand::Telemetry => {
                println!("Telemetry: [Enable telemetry module]");
            }
            ReplCommand::Learn { subject } => {
                if subject.is_empty() {
                    println!("Usage: learn <subject>");
                } else {
                    println!("Starting learning session: {}", subject);
                }
            }
            ReplCommand::Skill { name, args } => {
                println!("Running skill: {} with args {:?}", name, args);
            }
            ReplCommand::Unknown(cmd) => {
                println!("Unknown command: {}", cmd);
                println!("Type 'help' for available commands.");
            }
            ReplCommand::Quit => unreachable!(),
        }
    }
}

impl Default for Repl {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_quit_commands() {
        assert!(matches!(Repl::parse("quit"), ReplCommand::Quit));
        assert!(matches!(Repl::parse("exit"), ReplCommand::Quit));
        assert!(matches!(Repl::parse("q"), ReplCommand::Quit));
    }

    #[test]
    fn parses_help_commands() {
        assert!(matches!(Repl::parse("help"), ReplCommand::Help));
        assert!(matches!(Repl::parse("h"), ReplCommand::Help));
        assert!(matches!(Repl::parse("?"), ReplCommand::Help));
    }

    #[test]
    fn parses_learn_with_subject() {
        match Repl::parse("learn rust") {
            ReplCommand::Learn { subject } => {
                assert_eq!(subject, "rust");
            }
            _ => panic!("Expected Learn command"),
        }
    }

    #[test]
    fn parses_skill_with_args() {
        match Repl::parse("skill echo hello world") {
            ReplCommand::Skill { name, args } => {
                assert_eq!(name, "echo");
                assert_eq!(args, vec!["hello", "world"]);
            }
            _ => panic!("Expected Skill command"),
        }
    }

    #[test]
    fn parses_unknown_command() {
        match Repl::parse("unknown stuff") {
            ReplCommand::Unknown(s) => {
                assert_eq!(s, "unknown stuff");
            }
            _ => panic!("Expected Unknown command"),
        }
    }
}