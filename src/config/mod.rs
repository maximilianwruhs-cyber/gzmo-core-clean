//! Config Module
//!
//! Configuration loading and validation with TOML support.

pub mod validation;
pub mod defaults;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Root configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub modulation: ModulationConfig,
    #[serde(default)]
    pub feedback: FeedbackConfig,
    #[serde(default)]
    pub storage: StorageConfig,
    #[serde(default)]
    pub etl: EtlConfig,
    #[serde(default)]
    pub pedagogy: PedagogyConfig,
    #[serde(default)]
    pub telemetry: TelemetryConfig,
    #[serde(default)]
    pub gateway: GatewayConfig,
    #[serde(default)]
    pub llm: LlmConfig,
    #[serde(flatten)]
    pub extra: HashMap<String, toml::Value>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            modulation: ModulationConfig::default(),
            feedback: FeedbackConfig::default(),
            storage: StorageConfig::default(),
            etl: EtlConfig::default(),
            pedagogy: PedagogyConfig::default(),
            telemetry: TelemetryConfig::default(),
            gateway: GatewayConfig::default(),
            llm: LlmConfig::default(),
            extra: HashMap::new(),
        }
    }
}

/// Modulation configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModulationConfig {
    #[serde(default = "default_sigma")]
    pub sigma: f64,
    #[serde(default = "default_rho")]
    pub rho: f64,
    #[serde(default = "default_beta")]
    pub beta: f64,
    #[serde(default = "default_temp_min")]
    pub temp_min: f32,
    #[serde(default = "default_temp_max")]
    pub temp_max: f32,
    #[serde(default = "default_tokens_min")]
    pub tokens_min: u32,
    #[serde(default = "default_tokens_max")]
    pub tokens_max: u32,
    #[serde(default = "default_tempo_min_ms")]
    pub tempo_min_ms: u64,
    #[serde(default = "default_tempo_max_ms")]
    pub tempo_max_ms: u64,
}

impl Default for ModulationConfig {
    fn default() -> Self {
        Self {
            sigma: default_sigma(),
            rho: default_rho(),
            beta: default_beta(),
            temp_min: default_temp_min(),
            temp_max: default_temp_max(),
            tokens_min: default_tokens_min(),
            tokens_max: default_tokens_max(),
            tempo_min_ms: default_tempo_min_ms(),
            tempo_max_ms: default_tempo_max_ms(),
        }
    }
}

fn default_sigma() -> f64 { 10.0 }
fn default_rho() -> f64 { 28.0 }
fn default_beta() -> f64 { 8.0 / 3.0 }
fn default_temp_min() -> f32 { 0.3 }
fn default_temp_max() -> f32 { 1.2 }
fn default_tokens_min() -> u32 { 256 }
fn default_tokens_max() -> u32 { 2048 }
fn default_tempo_min_ms() -> u64 { 50 }
fn default_tempo_max_ms() -> u64 { 5000 }

/// Feedback loop configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeedbackConfig {
    #[serde(default = "default_history_window")]
    pub history_window: usize,
    #[serde(default = "default_similarity_threshold")]
    pub similarity_threshold: f64,
    #[serde(default = "default_exploration_boost")]
    pub exploration_boost: f32,
}

impl Default for FeedbackConfig {
    fn default() -> Self {
        Self {
            history_window: default_history_window(),
            similarity_threshold: default_similarity_threshold(),
            exploration_boost: default_exploration_boost(),
        }
    }
}

fn default_history_window() -> usize { 10 }
fn default_similarity_threshold() -> f64 { 0.85 }
fn default_exploration_boost() -> f32 { 0.5 }

/// Storage configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageConfig {
    #[serde(default = "default_vault_path")]
    pub vault_path: String,
    #[serde(default = "default_qdrant_url")]
    pub qdrant_url: String,
    #[serde(default = "default_dedup_threshold")]
    pub dedup_threshold: f64,
}

impl Default for StorageConfig {
    fn default() -> Self {
        Self {
            vault_path: default_vault_path(),
            qdrant_url: default_qdrant_url(),
            dedup_threshold: default_dedup_threshold(),
        }
    }
}

fn default_vault_path() -> String { "data/vault.db".to_string() }
fn default_qdrant_url() -> String { "http://localhost:6333".to_string() }
fn default_dedup_threshold() -> f64 { 0.95 }

/// ETL configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EtlConfig {
    #[serde(default = "default_cron_hour")]
    pub cron_hour: u8,
    #[serde(default = "default_cron_minute")]
    pub cron_minute: u8,
    #[serde(default = "default_min_confidence")]
    pub min_confidence: f64,
}

impl Default for EtlConfig {
    fn default() -> Self {
        Self {
            cron_hour: default_cron_hour(),
            cron_minute: default_cron_minute(),
            min_confidence: default_min_confidence(),
        }
    }
}

fn default_cron_hour() -> u8 { 1 }
fn default_cron_minute() -> u8 { 0 }
fn default_min_confidence() -> f64 { 0.8 }

/// Pedagogy configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PedagogyConfig {
    #[serde(default = "default_evaluator_temp")]
    pub evaluator_temp: f32,
    #[serde(default = "default_tutor_temp")]
    pub tutor_temp: f32,
}

impl Default for PedagogyConfig {
    fn default() -> Self {
        Self {
            evaluator_temp: default_evaluator_temp(),
            tutor_temp: default_tutor_temp(),
        }
    }
}

fn default_evaluator_temp() -> f32 { 0.4 }
fn default_tutor_temp() -> f32 { 0.6 }

/// Telemetry configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelemetryConfig {
    #[serde(default = "default_telemetry_enabled")]
    pub enabled: bool,
    #[serde(default = "default_export_interval_sec")]
    pub export_interval_sec: u64,
}

impl Default for TelemetryConfig {
    fn default() -> Self {
        Self {
            enabled: default_telemetry_enabled(),
            export_interval_sec: default_export_interval_sec(),
        }
    }
}

fn default_telemetry_enabled() -> bool { true }
fn default_export_interval_sec() -> u64 { 60 }

/// Gateway configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GatewayConfig {
    #[serde(default = "default_timeout_seconds")]
    pub timeout_seconds: u64,
    #[serde(default = "default_max_retries")]
    pub max_retries: u32,
}

impl Default for GatewayConfig {
    fn default() -> Self {
        Self {
            timeout_seconds: default_timeout_seconds(),
            max_retries: default_max_retries(),
        }
    }
}

fn default_timeout_seconds() -> u64 { 30 }
fn default_max_retries() -> u32 { 3 }

/// LLM endpoint configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmConfig {
    #[serde(default = "default_llm_endpoint")]
    pub endpoint: String,
    #[serde(default)]
    pub api_key: String,
    #[serde(default = "default_llm_model")]
    pub model: String,
}

impl Default for LlmConfig {
    fn default() -> Self {
        Self {
            endpoint: default_llm_endpoint(),
            api_key: String::new(),
            model: default_llm_model(),
        }
    }
}

fn default_llm_endpoint() -> String { "http://localhost:8000/v1/chat/completions".to_string() }
fn default_llm_model() -> String { "local-model".to_string() }

/// Load configuration from TOML file
pub fn load_from_file(path: &std::path::Path) -> Result<Config, ConfigError> {
    let content = std::fs::read_to_string(path)?;
    parse_config(&content)
}

/// Parse configuration from TOML string
pub fn parse_config(content: &str) -> Result<Config, ConfigError> {
    let config: Config = toml::from_str(content)
        .map_err(|e| ConfigError::Parse(format!("TOML parse error: {}", e)))?;
    
    // Validate the parsed config
    validation::validate(&config)
        .map_err(|errors| ConfigError::Validation(
            errors.iter().map(|e| format!("{}: {}", e.field, e.reason)).collect::<Vec<_>>().join(", ")
        ))?;
    
    Ok(config)
}

/// Save configuration to TOML file
pub fn save_to_file(config: &Config, path: &std::path::Path) -> Result<(), ConfigError> {
    let content = toml::to_string_pretty(config)
        .map_err(|e| ConfigError::Parse(format!("TOML serialization error: {}", e)))?;
    std::fs::write(path, content)?;
    Ok(())
}

/// Generate default configuration file
pub fn generate_default_config(path: &std::path::Path) -> Result<(), ConfigError> {
    let config = Config::default();
    save_to_file(&config, path)
}

/// Configuration error
#[derive(Debug)]
pub enum ConfigError {
    Io(std::io::Error),
    Parse(String),
    Validation(String),
}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConfigError::Io(e) => write!(f, "IO error: {}", e),
            ConfigError::Parse(s) => write!(f, "parse error: {}", s),
            ConfigError::Validation(s) => write!(f, "validation error: {}", s),
        }
    }
}

impl std::error::Error for ConfigError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            ConfigError::Io(e) => Some(e),
            _ => None,
        }
    }
}

impl From<std::io::Error> for ConfigError {
    fn from(e: std::io::Error) -> Self {
        ConfigError::Io(e)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_valid_toml() {
        let toml = r#"
[modulation]
temp_min = 0.5
temp_max = 1.0

[llm]
endpoint = "http://localhost:8080/v1/chat/completions"
model = "llama3"
"#;
        
        let config = parse_config(toml).unwrap();
        assert_eq!(config.modulation.temp_min, 0.5);
        assert_eq!(config.modulation.temp_max, 1.0);
        assert_eq!(config.llm.endpoint, "http://localhost:8080/v1/chat/completions");
        assert_eq!(config.llm.model, "llama3");
        // Defaults preserved for other fields
        assert_eq!(config.modulation.sigma, 10.0);
    }

    #[test]
    fn defaults_when_empty() {
        let config = parse_config("").unwrap();
        assert_eq!(config.modulation.temp_min, 0.3);
        assert_eq!(config.feedback.history_window, 10);
    }
}