//! Repetition Detector
//!
//! Detects repetitive patterns in LLM output using n-gram analysis and
//! semantic similarity. Triggers exploration when patterns become stuck.

use std::collections::{HashMap, VecDeque};

/// Pattern state for feedback loop
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PatternState {
    /// Output is novel (no significant similarity to history)
    Novel,
    /// Output is similar to recent history
    Similar,
    /// Stuck in repetition (high similarity, low diversity)
    Stuck,
    /// Confirmed loop (repeating identical or near-identical content)
    Loop,
}

impl PatternState {
    /// Whether exploration should be increased
    pub fn needs_exploration(&self) -> bool {
        matches!(self, Self::Stuck | Self::Loop)
    }

    /// Level of exploration adjustment needed (0.0-1.0)
    pub fn exploration_level(&self) -> f64 {
        match self {
            Self::Novel => 0.0,
            Self::Similar => 0.2,
            Self::Stuck => 0.5,
            Self::Loop => 0.8,
        }
    }
}

/// Repetition detector using n-gram and Jaccard similarity
pub struct RepetitionDetector {
    /// Window of recent outputs
    history: VecDeque<String>,
    /// Maximum history size
    window_size: usize,
    /// Similarity threshold for "similar" classification
    similarity_threshold: f64,
    /// Similarity threshold for "stuck" classification
    stuck_threshold: f64,
    /// Consecutive similar outputs to trigger "stuck"
    stuck_consecutive: u32,
    /// Current consecutive similar count
    consecutive_similar: u32,
    /// State n-gram frequencies
    ngram_cache: HashMap<String, Vec<usize>>,
}

impl RepetitionDetector {
    /// Create new detector with default settings
    pub fn new() -> Self {
        Self::with_config(10, 0.75, 0.90, 3)
    }

    /// Create with custom configuration
    ///
    /// # Arguments
    /// * `window_size` - Number of outputs to keep in history
    /// * `similarity_threshold` - Jaccard threshold for "similar" (0.0-1.0)
    /// * `stuck_threshold` - Jaccard threshold for "stuck" (0.0-1.0)
    /// * `stuck_consecutive` - Consecutive similar outputs to trigger "stuck"
    pub fn with_config(
        window_size: usize,
        similarity_threshold: f64,
        stuck_threshold: f64,
        stuck_consecutive: u32,
    ) -> Self {
        Self {
            history: VecDeque::with_capacity(window_size),
            window_size,
            similarity_threshold,
            stuck_threshold,
            stuck_consecutive,
            consecutive_similar: 0,
            ngram_cache: HashMap::new(),
        }
    }

    /// Add output to history and return current pattern state
    pub fn add_output(&mut self, output: impl Into<String>) -> PatternState {
        let output = output.into();

        // Check similarity against recent history
        let max_similarity = self.max_similarity(&output);

        // Update consecutive counter
        if max_similarity >= self.similarity_threshold {
            self.consecutive_similar += 1;
        } else {
            self.consecutive_similar = 0;
        }

        // Add to history
        self.history.push_back(output.clone());
        while self.history.len() > self.window_size {
            self.history.pop_front();
        }

        // Cache n-grams for this output
        self.cache_ngrams(&output, self.history.len() - 1);

        // Determine state
        if max_similarity >= self.stuck_threshold {
            // Check if it's a loop (identical or near-identical)
            if max_similarity > 0.97 {
                PatternState::Loop
            } else {
                PatternState::Stuck
            }
        } else if max_similarity >= self.similarity_threshold {
            if self.consecutive_similar >= self.stuck_consecutive {
                PatternState::Stuck
            } else {
                PatternState::Similar
            }
        } else {
            PatternState::Novel
        }
    }

    /// Current pattern state based on recent additions
    pub fn current_state(&self) -> PatternState {
        if self.consecutive_similar >= self.stuck_consecutive * 2 {
            PatternState::Loop
        } else if self.consecutive_similar >= self.stuck_consecutive {
            PatternState::Stuck
        } else if self.consecutive_similar > 0 {
            PatternState::Similar
        } else {
            PatternState::Novel
        }
    }

    /// Calculate maximum Jaccard similarity to history
    fn max_similarity(&self, output: &str) -> f64 {
        if self.history.is_empty() {
            return 0.0;
        }

        let output_ngrams = extract_ngrams(output, 3);
        let output_set: std::collections::HashSet<_> = output_ngrams.iter().collect();

        let mut max_sim = 0.0;
        for (idx, hist) in self.history.iter().enumerate() {
            // Skip most recent (not yet cached)
            if idx == self.history.len() - 1 && self.ngram_cache.contains_key(hist) {
                continue;
            }

            let hist_ngrams = if let Some(cached) = self.ngram_cache.get(hist) {
                cached.clone()
            } else {
                extract_ngrams(hist, 3)
            };
            let hist_set: std::collections::HashSet<_> = hist_ngrams.iter().collect();

            if output_set.is_empty() || hist_set.is_empty() {
                continue;
            }

            let intersection: std::collections::HashSet<_> =
                output_set.intersection(&hist_set).collect();
            let union: std::collections::HashSet<_> =
                output_set.union(&hist_set).collect();

            let similarity = intersection.len() as f64 / union.len() as f64;
            max_sim = max_sim.max(similarity);
        }

        max_sim
    }

    /// Cache n-grams for an output
    fn cache_ngrams(&mut self, output: &str, _index: usize) {
        if !self.ngram_cache.contains_key(output) {
            let ngrams = extract_ngrams(output, 3);
            let hashes: Vec<_> = ngrams.iter().map(|g| hash_ngram(g)).collect();
            self.ngram_cache.insert(output.to_string(), hashes);
        }
    }

    /// Clear history and reset state
    pub fn reset(&mut self) {
        self.history.clear();
        self.consecutive_similar = 0;
        self.ngram_cache.clear();
    }

    /// Number of outputs in history
    pub fn history_len(&self) -> usize {
        self.history.len()
    }
}

impl Default for RepetitionDetector {
    fn default() -> Self {
        Self::new()
    }
}

/// Extract character n-grams from text
fn extract_ngrams(text: &str, n: usize) -> Vec<String> {
    let chars: Vec<char> = text.chars().collect();
    if chars.len() < n {
        return vec![text.to_string()];
    }

    let mut ngrams = Vec::with_capacity(chars.len() - n + 1);
    for i in 0..=chars.len() - n {
        let gram: String = chars[i..i + n].iter().collect();
        ngrams.push(gram);
    }
    ngrams
}

/// Hash n-gram for efficient comparison
fn hash_ngram(ngram: &str) -> usize {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut hasher = DefaultHasher::new();
    ngram.hash(&mut hasher);
    hasher.finish() as usize
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_history_is_novel() {
        let mut detector = RepetitionDetector::new();
        let state = detector.add_output("hello world");
        assert_eq!(state, PatternState::Novel);
    }

    #[test]
    fn identical_outputs_trigger_loop() {
        let mut detector = RepetitionDetector::new();
        detector.add_output("test output");
        detector.add_output("different text");
        detector.add_output("test output");
        let state = detector.add_output("test output");
        assert_eq!(state, PatternState::Loop);
    }

    #[test]
    fn similar_outputs_build_stuck() {
        let mut detector = RepetitionDetector::with_config(10, 0.6, 0.8, 2);
        detector.add_output("the quick brown fox jumps");
        detector.add_output("the quick brown fox jumps high");
        detector.add_output("the quick brown fox jumps over");
        let state = detector.add_output("the quick brown fox jumps again");
        assert!(state.needs_exploration());
    }

    #[test]
    fn novel_outputs_reset_consecutive() {
        let mut detector = RepetitionDetector::new();
        detector.add_output("first");
        detector.add_output("first similar");
        detector.add_output("first similar too");
        assert!(detector.current_state().needs_exploration());

        detector.add_output("completely different content here");
        assert!(!detector.current_state().needs_exploration());
    }

    #[test]
    fn exploration_levels_are_correct() {
        assert_eq!(PatternState::Novel.exploration_level(), 0.0);
        assert_eq!(PatternState::Similar.exploration_level(), 0.2);
        assert_eq!(PatternState::Stuck.exploration_level(), 0.5);
        assert_eq!(PatternState::Loop.exploration_level(), 0.8);
    }
}