//! Config Module
//!
//! Configuration loading and validation.

pub mod validation;
pub mod defaults;

use std::collections::HashMap;

/// Configuration
#[derive(Debug, Clone)]
pub struct Config {
    /// Modulation parameters
    pub modulation: ModulationConfig,
    /// Feedback loop settings
    pub feedback: FeedbackConfig,
    /// Storage settings
    pub storage: StorageConfig,
    /// ETL settings
    pub etl: EtlConfig,
    /// Pedagogy settings
    pub pedagogy: PedagogyConfig,
    /// Telemetry settings
    pub telemetry: TelemetryConfig,
    /// LLM gateway settings
    pub gateway: GatewayConfig,
    /// Extra settings
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
            extra: HashMap::new(),
        }
    }
}

/// Modulation configuration
#[derive(Debug, Clone)]
pub struct ModulationConfig {
    pub sigma: f64,
    pub rho: f64,
    pub beta: f64,
    pub temp_min: f32,
    pub temp_max: f32,
    pub tokens_min: u32,
    pub tokens_max: u32,
    pub tempo_min_ms: u64,
    pub tempo_max_ms: u64,
}

impl Default for ModulationConfig {
    fn default() -> Self {
        Self {
            sigma: 10.0,
            rho: 28.0,
            beta: 8.0 / 3.0,
            temp_min: 0.3,
            temp_max: 1.2,
            tokens_min: 256,
            tokens_max: 2048,
            tempo_min_ms: 50,
            tempo_max_ms: 5000,
        }
    }
}

/// Feedback loop configuration
#[derive(Debug, Clone)]
pub struct FeedbackConfig {
    pub history_window: usize,
    pub similarity_threshold: f64,
    pub exploration_boost: f32,
}

impl Default for FeedbackConfig {
    fn default() -> Self {
        Self {
            history_window: 10,
            similarity_threshold: 0.85,
            exploration_boost: 0.5,
        }
    }
}

/// Storage configuration
#[derive(Debug, Clone)]
pub struct StorageConfig {
    pub vault_path: String,
    pub qdrant_url: String,
    pub dedup_threshold: f64,
}

impl Default for StorageConfig {
    fn default() -> Self {
        Self {
            vault_path: "data/vault.db".to_string(),
            qdrant_url: "http://localhost:6333".to_string(),
            dedup_threshold: 0.95,
        }
    }
}

/// ETL configuration
#[derive(Debug, Clone)]
pub struct EtlConfig {
    pub cron_hour: u8,
    pub cron_minute: u8,
    pub min_confidence: f64,
}

impl Default for EtlConfig {
    fn default() -> Self {
        Self {
            cron_hour: 1,
            cron_minute: 0,
            min_confidence: 0.8,
        }
    }
}

/// Pedagogy configuration
#[derive(Debug, Clone)]
pub struct PedagogyConfig {
    pub evaluator_temp: f32,
    pub tutor_temp: f32,
}

impl Default for PedagogyConfig {
    fn default() -> Self {
        Self {
            evaluator_temp: 0.4,
            tutor_temp: 0.6,
        }
    }
}

/// Telemetry configuration
#[derive(Debug, Clone)]
pub struct TelemetryConfig {
    pub enabled: bool,
    pub export_interval_sec: u64,
}

impl Default for TelemetryConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            export_interval_sec: 60,
        }
    }
}

/// Gateway configuration
#[derive(Debug, Clone)]
pub struct GatewayConfig {
    pub timeout_seconds: u64,
    pub max_retries: u32,
}

impl Default for GatewayConfig {
    fn default() -> Self {
        Self {
            timeout_seconds: 30,
            max_retries: 3,
        }
    }
}

/// Load configuration from file
pub fn load_from_file(path: &std::path::Path) -> Result<Config, ConfigError> {
    let content = std::fs::read_to_string(path)?;
    parse_config(&content)
}

/// Parse configuration from TOML string
pub fn parse_config(content: &str) -> Result<Config, ConfigError> {
    // Placeholder: In production, use proper TOML parsing
    Ok(Config::default())
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
