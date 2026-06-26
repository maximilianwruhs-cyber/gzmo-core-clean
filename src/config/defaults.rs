//! Default Values
//!
//! All default values in one place for documentation.

/// Returns the default configuration as TOML
pub fn default_config_toml() -> &'static str {
    r#"[modulation]
sigma = 10.0
rho = 28.0
beta = 2.667
temp_min = 0.3
temp_max = 1.2
tokens_min = 256
tokens_max = 2048
tempo_min_ms = 50
tempo_max_ms = 5000

[feedback]
history_window = 10
similarity_threshold = 0.85
exploration_boost = 0.5

[storage]
vault_path = "data/vault.db"
qdrant_url = "http://localhost:6333"
dedup_threshold = 0.95

[etl]
cron_hour = 1
cron_minute = 0
min_confidence = 0.8

[pedagogy]
evaluator_temp = 0.4
tutor_temp = 0.6

[telemetry]
enabled = true
export_interval_sec = 60

[gateway]
timeout_seconds = 30
max_retries = 3
"#
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_is_valid_toml() {
        let toml = default_config_toml();
        // Basic sanity check
        assert!(toml.contains("[modulation]"));
        assert!(toml.contains("[feedback]"));
        assert!(toml.contains("[storage]"));
    }
}