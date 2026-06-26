//! Adaptive Tempo
//!
//! Workload-adaptive timing that adjusts tick intervals based on queue depth.
//! Replaces fixed "174 BPM heartbeat" with response to actual system state.

use std::time::Duration;

/// Adaptive tick interval controller
///
/// Adjusts tempo (tick interval) based on:
/// - Queue depth (workload pressure)
/// - Average latency (system responsiveness)
/// - Capacity level (resource availability)
pub struct AdaptiveTempo {
    /// Current interval between ticks
    current_ms: u64,
    /// Minimum interval under heavy load
    min_ms: u64,
    /// Maximum interval when idle
    max_ms: u64,
    /// Smoothing factor for gradual changes
    smoothing: f64,
    /// Recent workload measurements
    history: VecDeque<WorkloadSample>,
    /// History window size
    window_size: usize,
}

use std::collections::VecDeque;

/// Workload measurement sample
#[derive(Debug, Clone, Copy)]
struct WorkloadSample {
    queue_depth: usize,
    avg_latency_ms: u64,
    timestamp_ms: u64,
}

impl AdaptiveTempo {
    /// Create new adaptive tempo controller
    ///
    /// # Arguments
    /// * `min_ms` - Minimum interval (heavy load)
    /// * `max_ms` - Maximum interval (idle)
    /// * `smoothing` - Change smoothing factor (0.0-1.0)
    pub fn new(min_ms: u64, max_ms: u64, smoothing: f64) -> Self {
        Self {
            current_ms: max_ms,
            min_ms,
            max_ms,
            smoothing: smoothing.clamp(0.0, 1.0),
            history: VecDeque::with_capacity(10),
            window_size: 10,
        }
    }

    /// Default tempo: 50-5000ms range with 0.1 smoothing
    pub fn default() -> Self {
        Self::new(50, 5000, 0.1)
    }

    /// Current tick interval
    pub fn interval(&self) -> Duration {
        Duration::from_millis(self.current_ms)
    }

    /// Current interval in milliseconds
    pub fn interval_ms(&self) -> u64 {
        self.current_ms
    }

    /// Update tempo based on current workload
    ///
    /// The target interval is computed from queue depth:
    /// - High queue depth -> shorter interval (process faster)
    /// - Low queue depth -> longer interval (save resources)
    pub fn update(&mut self, queue_depth: usize, avg_latency_ms: u64) {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;

        // Add sample to history
        self.history.push_back(WorkloadSample {
            queue_depth,
            avg_latency_ms,
            timestamp_ms: now,
        });

        // Trim history to window size
        while self.history.len() > self.window_size {
            self.history.pop_front();
        }

        // Calculate target interval based on average queue depth
        let avg_depth = if self.history.is_empty() {
            0.0
        } else {
            self.history.iter().map(|s| s.queue_depth as f64).sum::<f64>()
                / self.history.len() as f64
        };

        // Map average depth to interval: high depth -> low interval
        // Using inverse mapping: interval = max - (depth / threshold) * (max - min)
        let threshold = 100.0; // Depth at which we hit minimum interval
        let load_ratio = (avg_depth / threshold).clamp(0.0, 1.0);
        let target_ms = self.max_ms as f64 - load_ratio * (self.max_ms - self.min_ms) as f64;

        // Apply smoothing
        let new_ms = self.smoothing * target_ms + (1.0 - self.smoothing) * self.current_ms as f64;
        self.current_ms = new_ms as u64;
    }

    /// Force interval change (bypasses smoothing)
    pub fn set_interval(&mut self, ms: u64) {
        self.current_ms = ms.clamp(self.min_ms, self.max_ms);
    }

    /// Get current load estimate
    pub fn current_load(&self) -> f64 {
        if self.history.is_empty() {
            return 0.0;
        }
        let avg_depth = self.history.iter().map(|s| s.queue_depth as f64).sum::<f64>()
            / self.history.len() as f64;
        let threshold = 100.0;
        (avg_depth / threshold).clamp(0.0, 1.0)
    }

    /// Reset history and return to idle interval
    pub fn reset(&mut self) {
        self.history.clear();
        self.current_ms = self.max_ms;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tempo_stays_within_bounds() {
        let mut tempo = AdaptiveTempo::default();
        for i in 0..100 {
            tempo.update(i * 10, 100);
            let ms = tempo.interval_ms();
            assert!(ms >= 50 && ms <= 5000, "interval {} out of bounds", ms);
        }
    }

    #[test]
    fn high_queue_depth_reduces_interval() {
        let mut tempo = AdaptiveTempo::new(100, 1000, 0.5);
        tempo.update(0, 100);
        let idle_interval = tempo.interval_ms();

        // High queue depth should reduce interval
        for _ in 0..10 {
            tempo.update(500, 200);
        }
        let loaded_interval = tempo.interval_ms();

        assert!(
            loaded_interval < idle_interval,
            "loaded ({}) should be less than idle ({})",
            loaded_interval,
            idle_interval
        );
    }

    #[test]
    fn smoothing_prevents_sudden_changes() {
        let mut tempo = AdaptiveTempo::new(100, 1000, 0.1);
        tempo.update(0, 100);
        let initial = tempo.interval_ms();

        // Single high reading should not change much due to smoothing
        tempo.update(1000, 200);
        let after_one = tempo.interval_ms();

        let change_ratio = (initial as f64 - after_one as f64) / initial as f64;
        assert!(
            change_ratio < 0.2,
            "change ratio {} too high for smoothing=0.1",
            change_ratio
        );
    }
}