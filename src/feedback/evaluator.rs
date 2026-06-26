//! Output Evaluator
//!
//! Evaluates LLM output quality using measurable metrics:
//! - Lexical diversity (unique n-grams / total)
//! - Semantic novelty (hash similarity to recent outputs)
//! - Response latency
//! - Token count efficiency

use std::collections::HashSet;
use std::time::Duration;

/// Quality evaluation metrics
#[derive(Debug, Clone)]
pub struct QualityMetrics {
    /// Lexical diversity: unique 3-grams / total 3-grams
    pub diversity: f64,
    /// Semantic novelty: 1 - max_similarity_to_history
    pub novelty: f64,
    /// Response generation time in milliseconds
    pub latency_ms: u64,
    /// Total tokens (input + output)
    pub token_count: u32,
    /// Estimated success based on heuristics
    pub success: Option<bool>,
    /// Cache of n-gram hashes for efficiency
    ngram_cache: Vec<usize>,
}

impl QualityMetrics {
    /// Composite quality score (0-1)
    ///
    /// Weights: diversity 40%, novelty 40%, efficiency 20%
    pub fn score(&self) -> f64 {
        let efficiency = self.efficiency();
        self.diversity * 0.4 + self.novelty * 0.4 + efficiency * 0.2
    }

    /// Efficiency score based on latency and token count
    fn efficiency(&self) -> f64 {
        // Higher score for faster, cheaper responses
        let latency_score = 1.0 - (self.latency_ms as f64 / 5000.0).min(1.0);
        let token_score = 1.0 - (self.token_count as f64 / 4000.0).min(1.0);
        (latency_score + token_score) / 2.0
    }

    /// Cost per quality unit (lower is better)
    pub fn cost_per_quality(&self, token_cost_per_1k: f64) -> f64 {
        let cost = (self.token_count as f64 / 1000.0) * token_cost_per_1k;
        let quality = self.score().max(0.01);
        cost / quality
    }
}

/// Output quality evaluator
pub struct OutputEvaluator {
    /// N-gram size for diversity calculation
    ngram_size: usize,
    /// History of n-gram sets for novelty calculation
    history: Vec<Vec<usize>>,
    /// Maximum history size
    max_history: usize,
}

impl OutputEvaluator {
    /// Create evaluator with default settings
    pub fn new() -> Self {
        Self::with_config(3, 50)
    }

    /// Create with custom configuration
    ///
    /// # Arguments
    /// * `ngram_size` - Size of n-grams for diversity (3 = trigrams)
    /// * `max_history` - Number of outputs to keep for novelty comparison
    pub fn with_config(ngram_size: usize, max_history: usize) -> Self {
        Self {
            ngram_size,
            history: Vec::with_capacity(max_history),
            max_history,
        }
    }

    /// Evaluate output and return quality metrics
    ///
    /// # Arguments
    /// * `output` - The LLM output text
    /// * `latency` - Time taken to generate
    /// * `token_count` - Total tokens consumed
    pub fn evaluate(
        &mut self,
        output: impl AsRef<str>,
        latency: Duration,
        token_count: u32,
    ) -> QualityMetrics {
        let output = output.as_ref();

        // Calculate lexical diversity
        let diversity = self.calculate_diversity(output);

        // Calculate semantic novelty
        let novelty = self.calculate_novelty(output);

        // Heuristic success detection
        let success = self.detect_success(output);

        let metrics = QualityMetrics {
            diversity,
            novelty,
            latency_ms: latency.as_millis() as u64,
            token_count,
            success,
            ngram_cache: self.hash_ngrams(output),
        };

        // Add to history
        self.history.push(metrics.ngram_cache.clone());
        if self.history.len() > self.max_history {
            self.history.remove(0);
        }

        metrics
    }

    /// Calculate lexical diversity as unique n-grams / total n-grams
    fn calculate_diversity(&self, text: &str) -> f64 {
        let ngrams = extract_ngrams(text, self.ngram_size);
        if ngrams.is_empty() {
            return 0.0;
        }

        let unique: HashSet<_> = ngrams.iter().collect();
        unique.len() as f64 / ngrams.len() as f64
    }

    /// Calculate novelty as 1 - max Jaccard similarity to history
    fn calculate_novelty(&self, text: &str) -> f64 {
        if self.history.is_empty() {
            return 1.0;
        }

        let ngrams = self.hash_ngrams(text);
        let ngram_set: HashSet<_> = ngrams.iter().copied().collect();

        let mut max_similarity = 0.0;
        for hist_ngrams in &self.history {
            let hist_set: HashSet<_> = hist_ngrams.iter().copied().collect();

            let intersection: HashSet<_> = ngram_set.intersection(&hist_set).collect();
            let union: HashSet<_> = ngram_set.union(&hist_set).collect();

            if union.is_empty() {
                continue;
            }

            let similarity = intersection.len() as f64 / union.len() as f64;
            max_similarity = max_similarity.max(similarity);
        }

        1.0 - max_similarity
    }

    /// Detect success based on heuristics
    fn detect_success(&self, output: &str) -> Option<bool> {
        // Simple heuristics
        if output.trim().is_empty() {
            return Some(false);
        }

        if output.len() < 10 {
            return Some(false);
        }

        // Error patterns
        let error_patterns = ["error:", "failed:", "unable to", "could not", "sorry"];
        let lower = output.to_lowercase();
        for pattern in &error_patterns {
            if lower.contains(pattern) {
                return Some(false);
            }
        }

        None // Unknown - no strong signal either way
    }

    /// Hash n-grams for efficient comparison
    fn hash_ngrams(&self, text: &str) -> Vec<usize> {
        extract_ngrams(text, self.ngram_size)
            .iter()
            .map(|g| hash_string(g))
            .collect()
    }

    /// Reset history
    pub fn reset(&mut self) {
        self.history.clear();
    }

    /// Get average diversity from history
    pub fn avg_diversity(&self) -> Option<f64> {
        if self.history.is_empty() {
            return None;
        }
        // We don't store diversity, so this is a placeholder
        // In production, store metrics in history instead of just n-grams
        Some(0.5) // Placeholder
    }
}

impl Default for OutputEvaluator {
    fn default() -> Self {
        Self::new()
    }
}

/// Extract character n-grams
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

/// Hash string for comparison
fn hash_string(s: &str) -> usize {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut hasher = DefaultHasher::new();
    s.hash(&mut hasher);
    hasher.finish() as usize
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn high_diversity_for_unique_text() {
        let mut eval = OutputEvaluator::new();
        let metrics = eval.evaluate("the quick brown fox jumps over", Duration::from_millis(100), 10);
        assert!(metrics.diversity > 0.7, "diversity {} too low", metrics.diversity);
    }

    #[test]
    fn low_diversity_for_repetitive_text() {
        let mut eval = OutputEvaluator::new();
        let metrics = eval.evaluate("aaaaaaaaaa", Duration::from_millis(100), 5);
        assert!(metrics.diversity < 0.3, "diversity {} too high", metrics.diversity);
    }

    #[test]
    fn novelty_decreases_for_similar_text() {
        let mut eval = OutputEvaluator::new();
        let m1 = eval.evaluate("the quick brown fox", Duration::from_millis(100), 5);
        assert_eq!(m1.novelty, 1.0);

        let m2 = eval.evaluate("the quick brown cat", Duration::from_millis(100), 5);
        assert!(m2.novelty < 1.0, "novelty {} should be less than 1.0", m2.novelty);
    }

    #[test]
    fn empty_output_is_failure() {
        let mut eval = OutputEvaluator::new();
        let metrics = eval.evaluate("", Duration::from_millis(100), 0);
        assert_eq!(metrics.success, Some(false));
    }

    #[test]
    fn score_combines_metrics() {
        let metrics = QualityMetrics {
            diversity: 0.8,
            novelty: 0.8,
            latency_ms: 1000,
            token_count: 1000,
            success: Some(true),
            ngram_cache: vec![],
        };

        let score = metrics.score();
        // Score should be diversity*0.4 + novelty*0.4 + efficiency*0.2
        // efficiency = (1 - 1000/5000)/2 + (1 - 1000/4000)/2 = 0.4 + 0.375 = 0.775
        let expected = 0.8 * 0.4 + 0.8 * 0.4 + 0.775 * 0.2;
        assert!((score - expected).abs() < 0.01);
    }
}