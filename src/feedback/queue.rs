//! Parameter Mutation Queue
//!
//! Delayed parameter adjustment queue. Mutations are queued and applied
//! after a configurable delay to prevent oscillation.
//! Replaces "Thought Cabinet" with honest queue terminology.

use std::collections::VecDeque;
use std::time::{Duration, Instant};

/// Request to modify a parameter
#[derive(Debug, Clone)]
pub struct ParameterRequest {
    /// Category of parameter to modify
    pub target: ParameterTarget,
    /// Amount to adjust
    pub delta: f64,
    /// Delay before application (in ticks)
    pub delay_ticks: u64,
    /// Request timestamp
    pub created_at: Instant,
    /// Priority (higher = applied first)
    pub priority: u8,
}

/// Parameter target for adjustment
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ParameterTarget {
    /// LLM temperature
    Temperature,
    /// Max tokens limit
    MaxTokens,
    /// Top-p sampling threshold
    TopP,
    /// Lorenz rho parameter (affects attractor topology)
    LorenzRho,
    /// Lorenz sigma parameter
    LorenzSigma,
    /// Tempo interval
    TempoInterval,
}

/// Queue for delayed parameter mutations
pub struct ParameterMutationQueue {
    /// Pending mutations waiting for delay
    pending: VecDeque<QueuedMutation>,
    /// Applied mutations (kept for history)
    applied: Vec<AppliedMutation>,
    /// Maximum pending queue size
    max_pending: usize,
    /// Maximum applied history size
    max_applied: usize,
    /// Current tick counter
    tick: u64,
}

/// Mutation in queue awaiting application
#[derive(Debug, Clone)]
struct QueuedMutation {
    /// The underlying request
    request: ParameterRequest,
    /// Target tick for application
    target_tick: u64,
    /// Whether this mutation has been batch-merged
    merged: bool,
}

/// Mutation that has been applied
#[derive(Debug, Clone)]
pub struct AppliedMutation {
    pub target: ParameterTarget,
    pub delta: f64,
    pub applied_at_tick: u64,
    pub queued_for_ticks: u64,
    pub priority: u8,
    pub batch_id: Option<u32>,
}

/// Result of queue operations
#[derive(Debug)]
pub struct QueueResult {
    pub mutations_applied: Vec<AppliedMutation>,
    pub pending_count: usize,
    pub dropped_count: usize,
}

impl ParameterMutationQueue {
    /// Create new queue with default capacity
    pub fn new() -> Self {
        Self::with_capacity(100, 1000)
    }

    /// Create with custom capacity
    pub fn with_capacity(max_pending: usize, max_applied: usize) -> Self {
        Self {
            pending: VecDeque::with_capacity(max_pending),
            applied: Vec::with_capacity(max_applied),
            max_pending,
            max_applied,
            tick: 0,
        }
    }

    /// Queue a mutation for delayed application
    ///
    /// Returns false if queue is full and mutation was dropped
    pub fn try_queue(&mut self, request: ParameterRequest) -> bool {
        if self.pending.len() >= self.max_pending {
            return false;
        }

        let target_tick = self.tick + request.delay_ticks;
        let mutation = QueuedMutation {
            request,
            target_tick,
            merged: false,
        };

        // Insert in priority order
        let insert_pos = self.pending
            .iter()
            .position(|m| m.request.priority < mutation.request.priority)
            .unwrap_or(self.pending.len());
        self.pending.insert(insert_pos, mutation);

        true
    }

    /// Advance tick and apply ready mutations
    pub fn tick(&mut self) -> QueueResult {
        self.tick += 1;

        // Collect mutations ready for application
        let ready: Vec<_> = self.pending
            .iter()
            .enumerate()
            .filter(|(_, m)| m.target_tick <= self.tick && !m.merged)
            .map(|(i, _)| i)
            .collect();

        let mut applied = Vec::with_capacity(ready.len());
        let mut dropped = 0;

        // Apply in priority order, batching similar targets
        let mut batch_targets: std::collections::HashMap<ParameterTarget, f64> =
            std::collections::HashMap::new();

        for idx in ready {
            if let Some(mut mutation) = self.pending.get_mut(idx) {
                // Check if we can batch with existing
                if let Some(existing) = batch_targets.get(&mutation.request.target) {
                    // Merge: add deltas
                    batch_targets.insert(mutation.request.target, existing + mutation.request.delta);
                    mutation.merged = true;
                } else {
                    batch_targets.insert(mutation.request.target, mutation.request.delta);
                }
            }
        }

        // Create applied mutations from batches
        for (target, total_delta) in batch_targets {
            let applied_mut = AppliedMutation {
                target,
                delta: total_delta,
                applied_at_tick: self.tick,
                queued_for_ticks: 0, // Will be filled from original
                priority: 5,         // Average priority
                batch_id: Some(self.tick as u32),
            };
            applied.push(applied_mut);

            // Add to history
            self.applied.push(applied_mut.clone());
        }

        // Remove applied/merged from pending
        self.pending.retain(|m| !m.merged && m.target_tick > self.tick);

        // Trim history if over limit
        if self.applied.len() > self.max_applied {
            let to_remove = self.applied.len() - self.max_applied;
            self.applied.drain(0..to_remove);
        }

        // Count dropped (still full after processing)
        dropped = if self.pending.len() >= self.max_pending { 1 } else { 0 };

        QueueResult {
            mutations_applied: applied,
            pending_count: self.pending.len(),
            dropped_count: dropped,
        }
    }

    /// Preview mutations that would be applied on next tick
    ///
    /// Returns targets and their cumulative deltas if applied now
    pub fn pending_preview(&self) -> Vec<(ParameterTarget, f64)> {
        let mut previews: std::collections::HashMap<ParameterTarget, f64> =
            std::collections::HashMap::new();

        for m in &self.pending {
            if m.target_tick <= self.tick + 1 {
                let entry = previews.entry(m.request.target).or_insert(0.0);
                *entry += m.request.delta;
            }
        }

        previews.into_iter().collect()
    }

    /// Apply all pending mutations immediately (force)
    pub fn apply_all(&mut self) -> Vec<AppliedMutation> {
        let mut applied = Vec::with_capacity(self.pending.len());

        while let Some(queued) = self.pending.pop_front() {
            let mut app = AppliedMutation {
                target: queued.request.target,
                delta: queued.request.delta,
                applied_at_tick: self.tick,
                queued_for_ticks: self.tick - (queued.target_tick - queued.request.delay_ticks),
                priority: queued.request.priority,
                batch_id: None,
            };
            applied.push(app.clone());
            self.applied.push(app);
        }

        applied
    }

    /// Clear all pending mutations
    pub fn clear_pending(&mut self) {
        self.pending.clear();
    }

    /// Get count of pending mutations
    pub fn pending_count(&self) -> usize {
        self.pending.len()
    }

    /// Get count of applied mutations in history
    pub fn applied_count(&self) -> usize {
        self.applied.len()
    }

    /// Get recent applied mutations
    pub fn recent_applied(&self, n: usize) -> Vec<&AppliedMutation> {
        self.applied.iter().rev().take(n).collect()
    }

    /// Current tick number
    pub fn current_tick(&self) -> u64 {
        self.tick
    }
}

impl Default for ParameterMutationQueue {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_request(target: ParameterTarget, delta: f64, delay: u64) -> ParameterRequest {
        ParameterRequest {
            target,
            delta,
            delay_ticks: delay,
            created_at: Instant::now(),
            priority: 5,
        }
    }

    #[test]
    fn mutation_queues_and_applies_after_delay() {
        let mut queue = ParameterMutationQueue::new();

        // Queue with 2-tick delay
        queue.try_queue(make_request(ParameterTarget::Temperature, 0.1, 2));
        assert_eq!(queue.pending_count(), 1);

        // After 1 tick: still pending
        queue.tick();
        assert_eq!(queue.pending_count(), 1);

        // After 2 ticks: still pending
        queue.tick();
        assert_eq!(queue.pending_count(), 1);

        // After 3 ticks: applied
        let result = queue.tick();
        assert_eq!(result.mutations_applied.len(), 1);
        assert_eq!(result.mutations_applied[0].target, ParameterTarget::Temperature);
        assert_eq!(queue.pending_count(), 0);
    }

    #[test]
    fn priority_affects_application_order() {
        let mut queue = ParameterMutationQueue::new();

        // Queue low priority first, then high priority
        let mut low = make_request(ParameterTarget::Temperature, 0.1, 0);
        low.priority = 1;
        queue.try_queue(low);

        let mut high = make_request(ParameterTarget::MaxTokens, 100.0, 0);
        high.priority = 10;
        queue.try_queue(high);

        // Both should apply on next tick
        let result = queue.tick();

        // High priority should be first
        assert_eq!(result.mutations_applied[0].target, ParameterTarget::MaxTokens);
    }

    #[test]
    fn queue_drops_when_full() {
        let mut queue = ParameterMutationQueue::with_capacity(2, 100);

        // Fill queue
        queue.try_queue(make_request(ParameterTarget::Temperature, 0.1, 100));
        queue.try_queue(make_request(ParameterTarget::Temperature, 0.2, 100));

        // Third should be dropped
        assert!(!queue.try_queue(make_request(ParameterTarget::Temperature, 0.3, 100)));
    }

    #[test]
    fn apply_all_bypasses_delay() {
        let mut queue = ParameterMutationQueue::new();
        queue.try_queue(make_request(ParameterTarget::Temperature, 0.1, 10));
        queue.try_queue(make_request(ParameterTarget::MaxTokens, 100.0, 10));

        let applied = queue.apply_all();
        assert_eq!(applied.len(), 2);
        assert_eq!(queue.pending_count(), 0);
        assert_eq!(queue.applied_count(), 2);
    }

    #[test]
    fn pending_preview_shows_ready_mutations() {
        let mut queue = ParameterMutationQueue::new();
        queue.try_queue(make_request(ParameterTarget::Temperature, 0.1, 1));

        // Before tick: preview should be empty
        let preview = queue.pending_preview();
        assert!(preview.is_empty());

        // After tick: preview should show
        queue.tick();
        let preview = queue.pending_preview();
        assert!(!preview.is_empty());
    }
}