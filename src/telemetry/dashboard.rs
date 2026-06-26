//! Dashboard
//!
//! Real-time metric display.

use crate::telemetry::metrics::{Telemetry, MetricSnapshot};
use std::collections::VecDeque;

/// Dashboard event
#[derive(Debug, Clone)]
pub enum DashboardEvent {
    MetricUpdate(MetricSnapshot),
    Alert(String),
    Status(String),
}

/// Real-time dashboard
pub struct Dashboard {
    /// Recent snapshots for trending
    history: VecDeque<MetricSnapshot>,
    /// Max history to keep
    max_history: usize,
    /// Last event processed
    last_event: Option<DashboardEvent>,
}

impl Dashboard {
    pub fn new() -> Self {
        Self::with_capacity(60) // 1 minute at 1Hz
    }

    pub fn with_capacity(max_history: usize) -> Self {
        Self {
            history: VecDeque::with_capacity(max_history),
            max_history,
            last_event: None,
        }
    }

    /// Update dashboard with new telemetry
    pub fn update(&mut self, telemetry: &Telemetry) -> Vec<DashboardEvent> {
        let snapshot = telemetry.snapshot();
        let mut events = Vec::new();

        // Check for alerts
        if snapshot.escape_rate < 0.3 {
            events.push(DashboardEvent::Alert(
                format!("Low escape rate: {:.1}%", snapshot.escape_rate * 100.0)
            ));
        }

        if snapshot.avg_latency_ms > 2000 {
            events.push(DashboardEvent::Alert(
                format!("High latency: {}ms", snapshot.avg_latency_ms)
            ));
        }

        // Normal status
        events.push(DashboardEvent::Status(
            format!("Generations: {}, Efficiency: {:.1}",
                snapshot.total_generations,
                snapshot.efficiency
            )
        ));

        // Store snapshot
        self.history.push_back(snapshot);
        if self.history.len() > self.max_history {
            self.history.pop_front();
        }

        events.push(DashboardEvent::MetricUpdate(self.history.back().unwrap().clone()));
        self.last_event = events.last().cloned();

        events
    }

    /// Get current trend (increasing/decreasing/stable)
    pub fn trend(&self, metric: &str) -> &'static str {
        if self.history.len() < 2 {
            return "insufficient_data";
        }

        let recent: Vec<_> = self.history.iter().rev().take(5).collect();
        if recent.len() < 2 {
            return "insufficient_data";
        }

        // Simple linear trend
        let first = recent.first().unwrap();
        let last = recent.last().unwrap();

        match metric {
            "escape_rate" => {
                let diff = last.escape_rate - first.escape_rate;
                if diff.abs() < 0.05 {
                    "stable"
                } else if diff > 0.0 {
                    "improving"
                } else {
                    "declining"
                }
            }
            "efficiency" => {
                let diff = last.efficiency - first.efficiency;
                if diff.abs() < 1.0 {
                    "stable"
                } else if diff > 0.0 {
                    "improving"
                } else {
                    "declining"
                }
            }
            "latency" => {
                let diff = last.avg_latency_ms as f64 - first.avg_latency_ms as f64;
                if diff.abs() < 100.0 {
                    "stable"
                } else if diff < 0.0 {
                    "improving"
                } else {
                    "declining"
                }
            }
            _ => "unknown",
        }
    }

    /// Render simple text dashboard
    pub fn render(&self) -> String {
        if let Some(ref snap) = self.history.back() {
            format!(
                r#"=== GZMO Dashboard ===
Escape Rate: {:.1}% ({})
Efficiency: {:.1}
Avg Latency: {}ms ({})
Generations: {}
Total Tokens: {}
=====================
"#,
                snap.escape_rate * 100.0,
                self.trend("escape_rate"),
                snap.efficiency,
                snap.avg_latency_ms,
                self.trend("latency"),
                snap.total_generations,
                snap.total_tokens
            )
        } else {
            "No data available".to_string()
        }
    }

    /// Get history count
    pub fn history_len(&self) -> usize {
        self.history.len()
    }
}

impl Default for Dashboard {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dashboard_records_updates() {
        let mut dash = Dashboard::new();
        let mut tel = Telemetry::new();

        tel.record_generation(100, 500);
        dash.update(&tel);

        assert_eq!(dash.history_len(), 1);
    }

    #[test]
    fn low_escape_rate_triggers_alert() {
        let mut dash = Dashboard::new();
        let mut tel = Telemetry::new();

        // Many stuck, no escapes
        for _ in 0..10 {
            tel.record_stuck();
        }

        let events = dash.update(&tel);
        let has_alert = events.iter().any(|e| matches!(e, DashboardEvent::Alert(_)));
        assert!(has_alert);
    }

    #[test]
    fn trend_requires_history() {
        let dash = Dashboard::new();
        assert_eq!(dash.trend("escape_rate"), "insufficient_data");
    }

    #[test]
    fn render_outputs_dashboard() {
        let mut dash = Dashboard::new();
        let mut tel = Telemetry::new();
        tel.record_generation(100, 500);
        tel.record_escape();

        dash.update(&tel);
        let output = dash.render();

        assert!(output.contains("GZMO Dashboard"));
        assert!(output.contains("Escape Rate"));
    }
}