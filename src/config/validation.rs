//! Config Validation
//!
//! Validates configuration values against acceptable ranges.

use crate::config::{Config, ModulationConfig, FeedbackConfig};

/// Validation error
#[derive(Debug, Clone)]
pub struct ValidationError {
    pub field: String,
    pub value: String,
    pub reason: String,
}

/// Validation result
pub type ValidationResult = Result<(), Vec<ValidationError>>;

/// Validate full configuration
pub fn validate(config: &Config) -> ValidationResult {
    let mut errors = Vec::new();

    validate_modulation(&config.modulation, &mut errors);
    validate_feedback(&config.feedback, &mut errors);

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

/// Validate modulation config
fn validate_modulation(cfg: &ModulationConfig, errors: &mut Vec<ValidationError>) {
    if cfg.temp_min < 0.0 || cfg.temp_max > 2.0 || cfg.temp_min >= cfg.temp_max {
        errors.push(ValidationError {
            field: "modulation.temp_range".to_string(),
            value: format!("{} - {}", cfg.temp_min, cfg.temp_max),
            reason: "temperature range must be within [0, 2]".to_string(),
        });
    }

    if cfg.tokens_min == 0 || cfg.tokens_min >= cfg.tokens_max {
        errors.push(ValidationError {
            field: "modulation.tokens_range".to_string(),
            value: format!("{} - {}", cfg.tokens_min, cfg.tokens_max),
            reason: "tokens range invalid".to_string(),
        });
    }

    if cfg.tempo_min_ms == 0 || cfg.tempo_min_ms >= cfg.tempo_max_ms {
        errors.push(ValidationError {
            field: "modulation.tempo_range".to_string(),
            value: format!("{} - {}", cfg.tempo_min_ms, cfg.tempo_max_ms),
            reason: "tempo range invalid".to_string(),
        });
    }
}

/// Validate feedback config
fn validate_feedback(cfg: &FeedbackConfig, errors: &mut Vec<ValidationError>) {
    if cfg.similarity_threshold < 0.5 || cfg.similarity_threshold > 1.0 {
        errors.push(ValidationError {
            field: "feedback.similarity_threshold".to_string(),
            value: cfg.similarity_threshold.to_string(),
            reason: "must be in [0.5, 1.0]".to_string(),
        });
    }

    if cfg.exploration_boost < 0.0 || cfg.exploration_boost > 1.0 {
        errors.push(ValidationError {
            field: "feedback.exploration_boost".to_string(),
            value: cfg.exploration_boost.to_string(),
            reason: "must be in [0, 1]".to_string(),
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;

    #[test]
    fn valid_config_passes() {
        let config = Config::default();
        let result = validate(&config);
        assert!(result.is_ok());
    }

    #[test]
    fn invalid_temp_range_fails() {
        let mut config = Config::default();
        config.modulation.temp_min = 1.5;
        config.modulation.temp_max = 0.5; // inverted

        let result = validate(&config);
        assert!(result.is_err());
    }

    #[test]
    fn invalid_similarity_threshold_fails() {
        let mut config = Config::default();
        config.feedback.similarity_threshold = 2.0; // out of range

        let result = validate(&config);
        assert!(result.is_err());
    }
}