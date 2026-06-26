//! Metrics
//!
//! Metric collection for empirical validation.

use std::collections::HashMap;

/// Telemetry collector
#[derive(Debug, Clone)]
pub struct Telemetry {
    /// Parameter effectiveness: (param_value, quality_score) pairs
    parameter_correlations: HashMap<String, Vec<(f64, f64)>>,
    /// Stuck pattern count
    stuck_count: u64,
    /// Escape from stuck count
    escape_count: u64,
    /// Tokens consumed
    tokens_consumed: u64,
    /// API call count
    api_calls: u32,
    /// Total latency
    total_latency_ms: u64,
    /// Generation count
    generation_count: u64,
}

/// Metric snapshot at a point in time
#[derive(Debug, Clone)]
pub struct MetricSnapshot {
    pub timestamp: u64,
    pub temperature_diversity_correlation: f64,
    pub escape_rate: f64,
    /// Quality per cost unit (higher is better)
    pub efficiency: f64,
    pub avg_latency_ms: u64,
    pub total_tokens: u64,
    pub total_generations: u64,
}

impl Telemetry {
    pub fn new() -> Self {
        Self {
            parameter_correlations: HashMap::new(),
            stuck_count: 0,
            escape_count: 0,
            tokens_consumed: 0,
            api_calls: 0,
            total_latency_ms: 0,
            generation_count: 0,
        }
    }

    /// Record a parameter correlation
    pub fn record_correlation(&mut self, parameter: &str, value: f64, quality: f64) {
        self.parameter_correlations
            .entry(parameter.to_string())
            .or_default()
            .push((value, quality));
    }

    /// Record stuck event
    pub fn record_stuck(&mut self) {
        self.stuck_count += 1;
    }

    /// Record escape from stuck
    pub fn record_escape(&mut self) {
        self.escape_count += 1;
    }

    /// Record generation metrics
    pub fn record_generation(&mut self, tokens: u32, latency_ms: u64) {
        self.tokens_consumed += tokens as u64;
        self.total_latency_ms += latency_ms;
        self.generation_count += 1;
        self.api_calls += 1;
    }

    /// Calculate correlation between temperature and diversity
    pub fn temp_diversity_correlation(&self) -> f64 {
        self.calculate_correlation("temperature")
    }

    /// Calculate escape rate
    pub fn escape_rate(&self) -> f64 {
        if self.stuck_count == 0 {
            0.0
        } else {
            self.escape_count as f64 / self.stuck_count as f64
        }
    }

    /// Calculate cost efficiency (quality per cost)
    pub fn efficiency(&self) -> f64 {
        let total_cost = self.tokens_consumed as f64 * 0.001; // Approximate cost per 1K tokens
        let quality_sum: f64 = self
            .parameter_correlations
            .values()
            .flat_map(|v| v.iter().map(|(_, q)| q))
            .sum();

        if total_cost < 0.0001 {
            return 0.0;
        }

        quality_sum / total_cost
    }

    /// Calculate average latency
    pub fn avg_latency_ms(&self) -> u64 {
        if self.generation_count == 0 {
            0
        } else {
            self.total_latency_ms / self.generation_count
        }
    }

    /// Get snapshot of current metrics
    pub fn snapshot(&self) -> MetricSnapshot {
        MetricSnapshot {
            timestamp: now(),
            temperature_diversity_correlation: self.temp_diversity_correlation(),
            escape_rate: self.escape_rate(),
            efficiency: self.efficiency(),
            avg_latency_ms: self.avg_latency_ms(),
            total_tokens: self.tokens_consumed,
            total_generations: self.generation_count,
        }
    }

    /// Calculate correlation for a parameter
    fn calculate_correlation(&self, param: &str) -> f64 {
        let pairs = match self.parameter_correlations.get(param) {
            Some(p) if p.len() >= 2 => p,
            _ => return 0.0,
        };

        // Pearson correlation
        let n = pairs.len() as f64;
        let sum_x: f64 = pairs.iter().map(|(x, _)| x).sum();
        let sum_y: f64 = pairs.iter().map(|(_, y)| y).sum();
        let sum_xy: f64 = pairs.iter().map(|(x, y)| x * y).sum();
        let sum_x2: f64 = pairs.iter().map(|(x, _)| x * x).sum();
        let sum_y2: f64 = pairs.iter().map(|(_, y)| y * y).sum();

        let numerator = n * sum_xy - sum_x * sum_y;
        let denominator = ((n * sum_x2 - sum_x * sum_x) * (n * sum_y2 - sum_y * sum_y)).sqrt();

        if denominator == 0.0 {
            0.0
        } else {
            (numerator / denominator).clamp(-1.0, 1.0)
        }
    }
}

impl Default for Telemetry {
    fn default() -> Self {
        Self::new()
    }
}

fn now() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn telemetry_records_correlations() {
        let mut tel = Telemetry::new();
        for i in 0..10 {
            tel.record_correlation("temperature", i as f64 * 0.1, 0.5 + i as f64 * 0.05);
        }
        let corr = tel.temp_diversity_correlation();
        assert!(corr.abs() > 0.5, "correlation should be significant");
    }

    #[test]
    fn escape_rate_calculated_correctly() {
        let mut tel = Telemetry::new();
        assert_eq!(tel.escape_rate(), 0.0);

        tel.record_stuck();
        tel.record_stuck();
        assert_eq!(tel.escape_rate(), 0.0);

        tel.record_escape();
        assert_eq!(tel.escape_rate(), 0.5);

        tel.record_escape();
        assert_eq!(tel.escape_rate(), 1.0);
    }

    #[test]
    fn efficiency_decreases_with_cost() {
        let mut tel = Telemetry::new();

        // High quality, low cost
        tel.record_correlation("temp", 0.7, 0.9);
        let eff1 = tel.efficiency();

        // Low quality, same cost
        tel.record_correlation("temp", 0.7, 0.1);
        let eff2 = tel.efficiency();

        assert!(eff2 < eff1, "efficiency should decrease with lower quality");
    }

    #[test]
    fn snapshot_contains_all_fields() {
        let tel = Telemetry::new();
        let snap = tel.snapshot();

        assert!(snap.timestamp > 0);
        assert_eq!(snap.total_generations, 0);
    }
}