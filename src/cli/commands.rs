//! Commands
//!
//! Command execution with full component wiring.

use crate::cli::args::CliArgs;
use crate::config::{Config, generate_default_config};
use crate::modulation::{StateGenerator, ParameterMapper, AdaptiveTempo};
use crate::feedback::{RepetitionDetector, PatternState};
use crate::telemetry::Telemetry;
use crate::gateway::{LlmClient, LlmRequest};
use crate::storage::{SqliteVault, Vault};
use std::path::Path;
use tokio::signal;

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
    Init { path: String },
}

/// Command runner
pub struct CommandRunner {
    config: Option<Config>,
}

impl CommandRunner {
    pub fn new() -> Self {
        Self { config: None }
    }

    pub fn with_config(config: Config) -> Self {
        Self { config: Some(config) }
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
                    Some("init") => ConfigAction::Init {
                        path: args.command_args.get(1).cloned().unwrap_or_else(|| "config.toml".to_string()),
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
                self.run_main_loop().await
            }
            Command::Pedagogy { subject } => {
                self.run_pedagogy(subject).await
            }
            Command::Etl => {
                self.run_etl().await
            }
            Command::Telemetry => {
                self.show_telemetry().await
            }
            Command::SelfImprove => {
                self.run_self_improve().await
            }
            Command::Config { action } => {
                self.handle_config(action).await
            }
        }
    }

    /// Run the main modulation loop
    async fn run_main_loop(&self) -> Result<(), CommandError> {
        println!("Starting GZMO modulation loop...");
        
        // Load or use default config
        let config = self.config.clone().unwrap_or_default();
        
        // Initialize components
        let mut generator = StateGenerator::new(0.506);
        let mapper = ParameterMapper::with_ranges(
            config.modulation.temp_min,
            config.modulation.temp_max,
            config.modulation.tokens_min,
            config.modulation.tokens_max,
            0.7, 0.95, // top_p min/max
        );
        
        let mut detector = RepetitionDetector::with_config(
            config.feedback.history_window,
            config.feedback.similarity_threshold,
            0.90, // stuck threshold
            3,    // consecutive before stuck
        );
        
        let mut telemetry = Telemetry::new();
        let mut tempo = AdaptiveTempo::new(
            config.modulation.tempo_min_ms,
            config.modulation.tempo_max_ms,
            0.1, // smoothing
        );
        
        // Initialize LLM client
        let client = LlmClient::new(
            &config.llm.endpoint,
            &config.llm.api_key,
            &config.llm.model,
        ).with_retries(config.gateway.max_retries)
         .with_timeout(config.gateway.timeout_seconds);
        
        println!("Connected to LLM at: {}", config.llm.endpoint);
        println!("Model: {}", config.llm.model);
        println!("Press Ctrl+C to stop\n");
        
        // Initialize SQLite vault
        let vault_path = &config.storage.vault_path;
        std::fs::create_dir_all(std::path::Path::new(vault_path).parent().unwrap_or(Path::new(".")))
            .map_err(|e| CommandError::Execution(format!("Failed to create data dir: {}", e)))?;
        
        let mut vault = SqliteVault::open(vault_path)
            .map_err(|e| CommandError::Execution(format!("Failed to open vault: {}", e)))?;
        
        println!("Vault initialized: {} facts, {} edges", 
            vault.fact_count().unwrap_or(0),
            vault.edge_count().unwrap_or(0)
        );
        
        // Setup shutdown handler
        let shutdown = signal::ctrl_c();
        tokio::pin!(shutdown);
        
        let mut iteration = 0;
        let prompt = "Generate a creative response about artificial intelligence and learning.";
        
        loop {
            tokio::select! {
                _ = &mut shutdown => {
                    println!("\nShutdown signal received. Stopping...");
                    break;
                }
                _ = async {
                    // Step 1: Advance state generator
                    generator.step();
                    
                    // Step 2: Check if stuck and adjust parameters
                    let pattern_state = detector.current_state();
                    let needs_exploration = pattern_state.needs_exploration();
                    
                    // Step 3: Map state to LLM parameters
                    let mut params = mapper.map_state(&generator);
                    
                    // Apply exploration boost if stuck
                    if needs_exploration {
                        let boost = config.feedback.exploration_boost;
                        params.temperature = (params.temperature + boost).min(config.modulation.temp_max);
                        println!("[Iteration {}] Detected repetition, boosting temperature to {:.2}", 
                            iteration, params.temperature);
                    }
                    
                    // Step 4: Build LLM request
                    let request = LlmRequest {
                        system_prompt: Some("You are a helpful AI assistant.".to_string()),
                        user_prompt: prompt.to_string(),
                        params: params.clone(),
                    };
                    
                    // Step 5: Send to LLM
                    match client.send(request).await {
                        Ok(response) => {
                            // Record telemetry
                            telemetry.record_generation(response.tokens_used, response.latency.as_millis() as u64);
                            
                            // Add output to detector
                            let new_state = detector.add_output(&response.text);
                            
                            // Log results
                            println!("[Iter {}] Temp: {:.2}, Tokens: {}, Latency: {}ms, State: {:?}",
                                iteration,
                                params.temperature,
                                response.tokens_used,
                                response.latency.as_millis(),
                                new_state
                            );
                            
                            // Store in vault if novel
                            if new_state == PatternState::Novel {
                                let fact = crate::storage::create_fact(
                                    response.text.chars().take(200).collect::<String>(),
                                    "llm_generation"
                                );
                                let _ = vault.store_fact(fact);
                            }
                        }
                        Err(e) => {
                            eprintln!("[Iter {}] LLM error: {}", iteration, e);
                        }
                    }
                    
                    // Step 6: Update tempo based on iteration
                    tempo.update(iteration % 100, 500);
                    
                    // Step 7: Sleep before next iteration
                    tokio::time::sleep(tempo.interval()).await;
                    
                    iteration += 1;
                } => {}
            }
        }
        
        // Print final stats
        let snapshot = telemetry.snapshot();
        println!("\n=== Final Statistics ===");
        println!("Total generations: {}", snapshot.total_generations);
        println!("Average latency: {}ms", snapshot.avg_latency_ms);
        println!("Total tokens: {}", snapshot.total_tokens);
        println!("Vault facts: {}", vault.fact_count().unwrap_or(0));
        
        Ok(())
    }

    /// Run pedagogy session
    async fn run_pedagogy(&self, subject: Option<String>) -> Result<(), CommandError> {
        use crate::pedagogy::{Session, SessionConfig, SessionResult};
        use crate::pedagogy::evaluator::KnowledgeLevel;
        
        let subject = subject.unwrap_or_else(|| "general".to_string());
        println!("Starting pedagogy session: {}", subject);
        
        let config = self.config.clone().unwrap_or_default();
        
        let mut session = Session::new(SessionConfig {
            subject: subject.clone(),
            student_id: "cli_user".to_string(),
            max_interactions: 20,
            initial_level: KnowledgeLevel::Developing,
            llm_endpoint: config.llm.endpoint.clone(),
            llm_model: config.llm.model.clone(),
        });
        
        // Demo interaction with async/await
        let input = "What is this topic about?";
        
        match session.interact(input).await {
            Ok(SessionResult::Response { text, meta, remaining_interactions }) => {
                println!("\nStudent: {}", input);
                println!("Tutor: {}", text);
                println!("  [Difficulty: {:.0}%, Load: {}, Remaining: {}]\n",
                    meta.difficulty * 100.0,
                    meta.cognitive_load,
                    remaining_interactions
                );
            }
            Ok(SessionResult::SessionEnded) => {
                println!("Session completed.");
            }
            Err(e) => {
                println!("Session error: {}", e);
            }
        }
        
        // Show sync fallback example
        println!("Using sync fallback...");
        let result = session.interact_sync("Tell me more.");
        match result {
            SessionResult::Response { text, .. } => {
                println!("Student: Tell me more.");
                println!("Tutor: {}\n", text);
            }
            _ => {}
        }
        
        let stats = session.stats();
        println!("Session Stats:");
        println!("  Total interactions: {}", stats.total_interactions);
        println!("  Average difficulty: {:.0}%", stats.avg_difficulty * 100.0);
        
        Ok(())
    }

    /// Run ETL pipeline
    async fn run_etl(&self) -> Result<(), CommandError> {
        use crate::etl::{Extractor, Verifier, Promoter};
        
        println!("Running ETL pipeline...");
        
        let config = self.config.clone().unwrap_or_default();
        
        // Create extractor with LLM client
        let client = LlmClient::new(
            &config.llm.endpoint,
            &config.llm.api_key,
            &config.llm.model,
        );
        let extractor = Extractor::with_client(client)
            .with_threshold(config.etl.min_confidence);
        
        let verifier = Verifier::with_config(config.etl.min_confidence, 1);
        let _promoter = Promoter::new();
        
        // Create vault
        let vault = SqliteVault::open(&config.storage.vault_path)
            .map_err(|e| CommandError::Execution(format!("Failed to open vault: {}", e)))?;
        
        // Test extraction with LLM (async)
        let text = "Paris is the capital of France. London is in the UK. Berlin is in Germany.";
        
        match extractor.extract(text).await {
            Ok(extraction) => {
                println!("Extracted {} facts (LLM)", extraction.facts.len());
                
                let verification = verifier.verify(&extraction);
                
                if verification.passed {
                    println!("Verification passed");
                    println!("ETL complete: {} facts processed, latency: {}ms", 
                        extraction.facts.len(), extraction.latency_ms);
                } else {
                    println!("Verification failed: {}", 
                        verification.rejection_reasons.join(", "));
                }
            }
            Err(e) => {
                println!("LLM extraction failed ({}), using heuristic fallback...", e);
                
                // Fallback to heuristic
                let extraction = extractor.extract_heuristic(text);
                println!("Extracted {} facts (heuristic)", extraction.facts.len());
                
                let verification = verifier.verify(&extraction);
                if verification.passed {
                    println!("Verification passed (heuristic)");
                }
            }
        }
        
        let _ = vault.fact_count(); // Keep vault in scope
        
        Ok(())
    }

    /// Show telemetry dashboard
    async fn show_telemetry(&self) -> Result<(), CommandError> {
        use crate::telemetry::{Telemetry, Dashboard, MetricExporter, exporter::ExportFormat};
        
        println!("Telemetry Dashboard");
        println!("===================\n");
        
        let mut telemetry = Telemetry::new();
        let mut dashboard = Dashboard::new();
        
        // Simulate some data
        for i in 0..5 {
            telemetry.record_generation(100 + i * 20, (500 + i * 50) as u64);
            telemetry.record_correlation("temperature", 0.7 + i as f64 * 0.05, 0.8);
        }
        
        // Update dashboard
        let _events = dashboard.update(&telemetry);
        
        // Print dashboard
        println!("{}", dashboard.render());
        
        // Export metrics
        let exporter = MetricExporter::default();
        let snapshot = telemetry.snapshot();
        
        println!("\n=== JSON Export ===");
        println!("{}", exporter.export(&snapshot, ExportFormat::Json));
        
        Ok(())
    }

    /// Run self-improvement loop
    async fn run_self_improve(&self) -> Result<(), CommandError> {
        use crate::feedback::{RepetitionDetector, OutputEvaluator, StrategyLearner, LearningStrategy, Experience};
        use crate::feedback::detector::PatternState;
        
        println!("Self-Improvement Loop Analysis");
        println!("==============================\n");
        
        let mut detector = RepetitionDetector::new();
        let mut evaluator = OutputEvaluator::new();
        let mut learner = StrategyLearner::with_config(1000, 0.1, LearningStrategy::BestAverage);
        
        // Simulate some outputs
        let outputs = vec![
            "First unique output about cats.",
            "Another unique output about dogs.",
            "Similar output about cats.",
            "Similar output about cats.",
            "Breakthrough new output about birds.",
        ];
        
        for (i, output) in outputs.iter().enumerate() {
            let state = detector.add_output(*output);
            let metrics = evaluator.evaluate(output, std::time::Duration::from_millis(500), 100);
            
            // Record experience
            let experience = Experience {
                temperature: 0.7 + i as f32 * 0.05,
                max_tokens: 500,
                quality_score: metrics.score(),
                success: metrics.success.unwrap_or(true),
                task_type: "test".to_string(),
                timestamp: i as u64,
            };
            
            learner.record(experience);
            
            println!("Output {}: State={:?}, Score={:.2}", i, state, metrics.score());
        }
        
        // Get recommendation
        if let Some(rec) = learner.recommend("test") {
            println!("\nRecommended temperature: {:.2} (confidence: {:.2})", 
                rec.temperature, rec.confidence);
        }
        
        Ok(())
    }

    /// Handle config commands
    async fn handle_config(&self, action: ConfigAction) -> Result<(), CommandError> {
        match action {
            ConfigAction::Get { key } => {
                if key.is_empty() {
                    // Show all config
                    let config = self.config.clone().unwrap_or_default();
                    println!("Current Configuration:");
                    println!("  LLM Endpoint: {}", config.llm.endpoint);
                    println!("  LLM Model: {}", config.llm.model);
                    println!("  Temperature Range: {:.2} - {:.2}", 
                        config.modulation.temp_min, config.modulation.temp_max);
                    println!("  Tokens Range: {} - {}", 
                        config.modulation.tokens_min, config.modulation.tokens_max);
                    println!("  Vault Path: {}", config.storage.vault_path);
                    println!("  History Window: {}", config.feedback.history_window);
                } else {
                    println!("Config key: {} = [use 'config init' to see all values]", key);
                }
            }
            ConfigAction::Set { key, value } => {
                println!("Setting config: {} = {} (requires config file edit)", key, value);
            }
            ConfigAction::Init { path } => {
                generate_default_config(Path::new(&path))
                    .map_err(|e| CommandError::Execution(format!("Failed to create config: {}", e)))?;
                println!("Created default config at: {}", path);
            }
        }
        Ok(())
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
    fn parses_config_init() {
        let args = CliArgs {
            command: Some("config".to_string()),
            command_args: vec!["init".to_string(), "/tmp/test.toml".to_string()],
            ..Default::default()
        };
        let cmd = CommandRunner::parse(&args);
        assert!(matches!(cmd, Some(Command::Config { action: ConfigAction::Init { .. } })));
    }
}