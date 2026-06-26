//! Strategy Learner
//!
//! Learns optimal LLM parameters from parameter-outcome pairs.
//! Records experiences and recommends parameters based on past success.

use std::collections::HashMap;

/// A recorded experience: parameters -> outcome
#[derive(Debug, Clone)]
pub struct Experience {
    /// Temperature setting used
    pub temperature: f32,
    /// Max tokens setting used
    pub max_tokens: u32,
    /// Quality score received
    pub quality_score: f64,
    /// Whether the task succeeded
    pub success: bool,
    /// Task type/category
    pub task_type: String,
    /// Timestamp
    pub timestamp: u64,
}

/// Learning strategy for parameter selection
#[derive(Debug, Clone, Copy)]
pub enum LearningStrategy {
    /// Select parameters that worked best on average
    BestAverage,
    /// Select parameters that maximized success rate
    MaximizeSuccess,
    /// Select parameters that maximized diversity
    MaximizeDiversity,
    /// Balance exploration vs exploitation (epsilon-greedy)
    EpsilonGreedy { epsilon: f64 },
}

/// Strategy learner that optimizes parameters from experience
pub struct StrategyLearner {
    /// Recorded experiences by task type
    experiences: HashMap<String, Vec<Experience>>,
    /// Maximum experiences to keep per task type
    max_experiences: usize,
    /// Learning rate for updating recommendations
    learning_rate: f64,
    /// Current recommended parameters by task type
    recommendations: HashMap<String, ParameterRecommendation>,
    /// Strategy in use
    strategy: LearningStrategy,
}

/// Recommended parameters with confidence
#[derive(Debug, Clone)]
pub struct ParameterRecommendation {
    pub temperature: f32,
    pub max_tokens: u32,
    pub confidence: f64,
    pub sample_count: usize,
}

impl StrategyLearner {
    /// Create new learner with default settings
    pub fn new() -> Self {
        Self::with_config(1000, 0.1, LearningStrategy::EpsilonGreedy { epsilon: 0.2 })
    }

    /// Create with custom configuration
    pub fn with_config(
        max_experiences: usize,
        learning_rate: f64,
        strategy: LearningStrategy,
    ) -> Self {
        Self {
            experiences: HashMap::new(),
            max_experiences,
            learning_rate: learning_rate.clamp(0.01, 1.0),
            recommendations: HashMap::new(),
            strategy,
        }
    }

    /// Record a new experience
    pub fn record(&mut self, experience: Experience) {
        let task_type = experience.task_type.clone();

        // Add to experiences
        let list = self.experiences.entry(task_type.clone()).or_default();
        list.push(experience);

        // Trim if over limit
        if list.len() > self.max_experiences {
            list.remove(0);
        }

        // Update recommendations for this task type
        self.update_recommendation(&task_type);
    }

    /// Get recommended parameters for a task type
    ///
    /// Returns None if no experiences recorded yet
    pub fn recommend(&self, task_type: &str) -> Option<ParameterRecommendation> {
        self.recommendations.get(task_type).cloned()
    }

    /// Get recommendation with exploration noise
    ///
    /// Useful for epsilon-greedy exploration
    pub fn recommend_with_noise(
        &self,
        task_type: &str,
        noise_temperature: f32,
        noise_tokens: u32,
    ) -> Option<ParameterRecommendation> {
        let base = self.recommend(task_type)?;

        // Add random noise within bounds
        let temp_noise = (rand::random::<f32>() - 0.5) * 2.0 * noise_temperature;
        let token_noise = (rand::random::<f32>() - 0.5) * 2.0 * noise_tokens as f32;

        Some(ParameterRecommendation {
            temperature: (base.temperature + temp_noise).clamp(0.0, 2.0),
            max_tokens: (base.max_tokens as i32 + token_noise as i32).max(1) as u32,
            confidence: base.confidence * 0.8, // Reduced confidence with noise
            sample_count: base.sample_count,
        })
    }

    /// Update recommendation based on experiences
    fn update_recommendation(&mut self, task_type: &str) {
        let experiences = match self.experiences.get(task_type) {
            Some(e) if !e.is_empty() => e,
            _ => return,
        };

        let rec = match self.strategy {
            LearningStrategy::BestAverage => self.compute_best_average(experiences),
            LearningStrategy::MaximizeSuccess => self.compute_maximize_success(experiences),
            LearningStrategy::MaximizeDiversity => self.compute_maximize_diversity(experiences),
            LearningStrategy::EpsilonGreedy { epsilon } => {
                if rand::random::<f64>() < epsilon {
                    // Random exploration
                    ParameterRecommendation {
                        temperature: 0.3 + rand::random::<f32>() * 1.0,
                        max_tokens: 256 + rand::random::<u32>() % 1792,
                        confidence: 0.1,
                        sample_count: experiences.len(),
                    }
                } else {
                    self.compute_best_average(experiences)
                }
            }
        };

        self.recommendations.insert(task_type.to_string(), rec);
    }

    /// Compute recommendation maximizing average score
    fn compute_best_average(&self, experiences: &[Experience]) -> ParameterRecommendation {
        // Bin by rounded parameters
        let mut bins: HashMap<(i32, i32), Vec<f64>> = HashMap::new();

        for exp in experiences {
            // Round to bins
            let temp_bin = (exp.temperature * 10.0).round() as i32;
            let token_bin = ((exp.max_tokens / 100) as i32) * 100;
            let bin = (temp_bin, token_bin);

            bins.entry(bin).or_default().push(exp.quality_score);
        }

        // Find bin with highest average
        let mut best_bin = (3, 1000); // default
        let mut best_avg = 0.0;
        let mut max_samples = 0;

        for (bin, scores) in &bins {
            let avg = scores.iter().sum::<f64>() / scores.len() as f64;
            if avg > best_avg || (avg == best_avg && scores.len() > max_samples) {
                best_avg = avg;
                best_bin = *bin;
                max_samples = scores.len();
            }
        }

        ParameterRecommendation {
            temperature: best_bin.0 as f32 / 10.0,
            max_tokens: best_bin.1 as u32,
            confidence: (best_avg * (max_samples as f64).sqrt() / 10.0).min(1.0),
            sample_count: experiences.len(),
        }
    }

    /// Compute recommendation maximizing success rate
    fn compute_maximize_success(&self, experiences: &[Experience]) -> ParameterRecommendation {
        let mut bins: HashMap<(i32, i32), (u32, u32)> = HashMap::new(); // (successes, total)

        for exp in experiences {
            let temp_bin = (exp.temperature * 10.0).round() as i32;
            let token_bin = ((exp.max_tokens / 100) as i32) * 100;
            let bin = (temp_bin, token_bin);

            let (successes, total) = bins.entry(bin).or_insert((0, 0));
            if exp.success {
                *successes += 1;
            }
            *total += 1;
        }

        let mut best_bin = (3, 1000);
        let mut best_rate = 0.0;

        for (bin, (successes, total)) in bins {
            if total == 0 {
                continue;
            }
            let rate = successes as f64 / total as f64;
            if rate > best_rate {
                best_rate = rate;
                best_bin = bin;
            }
        }

        ParameterRecommendation {
            temperature: best_bin.0 as f32 / 10.0,
            max_tokens: best_bin.1 as u32,
            confidence: best_rate,
            sample_count: experiences.len(),
        }
    }

    /// Compute recommendation maximizing diversity (quality score proxy)
    fn compute_maximize_diversity(&self, experiences: &[Experience]) -> ParameterRecommendation {
        // For diversity, we prefer higher temperatures
        let avg_temp = experiences.iter().map(|e| e.temperature).sum::<f32>() / experiences.len() as f32;
        let avg_tokens = experiences.iter().map(|e| e.max_tokens).sum::<u32>() / experiences.len() as u32;

        // Slight bias toward higher temperature
        ParameterRecommendation {
            temperature: (avg_temp + 0.2).min(1.2),
            max_tokens: avg_tokens,
            confidence: 0.5,
            sample_count: experiences.len(),
        }
    }

    /// Get statistics for a task type
    pub fn stats(&self, task_type: &str) -> Option<LearnerStats> {
        let experiences = self.experiences.get(task_type)?;
        if experiences.is_empty() {
            return None;
        }

        let total = experiences.len();
        let successes = experiences.iter().filter(|e| e.success).count();
        let avg_score = experiences.iter().map(|e| e.quality_score).sum::<f64>() / total as f64;
        let avg_temp = experiences.iter().map(|e| e.temperature).sum::<f32>() / total as f32;

        Some(LearnerStats {
            total_samples: total,
            success_rate: successes as f64 / total as f64,
            avg_quality_score: avg_score,
            avg_temperature: avg_temp,
        })
    }

    /// Clear all experiences and recommendations
    pub fn reset(&mut self) {
        self.experiences.clear();
        self.recommendations.clear();
    }
}

/// Statistics for a task type
#[derive(Debug, Clone)]
pub struct LearnerStats {
    pub total_samples: usize,
    pub success_rate: f64,
    pub avg_quality_score: f64,
    pub avg_temperature: f64,
}

impl Default for StrategyLearner {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_exp(temp: f32, tokens: u32, score: f64, success: bool) -> Experience {
        Experience {
            temperature: temp,
            max_tokens: tokens,
            quality_score: score,
            success,
            task_type: "test".to_string(),
            timestamp: 0,
        }
    }

    #[test]
    fn learner_recommends_after_experiences() {
        let mut learner = StrategyLearner::new();

        // Add good experiences with temp=0.7
        for _ in 0..5 {
            learner.record(make_exp(0.7, 1000, 0.8, true));
        }

        // Add worse experiences with temp=0.3
        for _ in 0..5 {
            learner.record(make_exp(0.3, 1000, 0.4, false));
        }

        let rec = learner.recommend("test").unwrap();
        assert!((rec.temperature - 0.7).abs() < 0.2, "should recommend ~0.7");
    }

    #[test]
    fn maximize_success_strategy_works() {
        let mut learner = StrategyLearner::with_config(100, 0.1, LearningStrategy::MaximizeSuccess);

        // High temp: 100% failure
        for _ in 0..5 {
            learner.record(make_exp(1.2, 500, 0.3, false));
        }

        // Medium temp: 100% success
        for _ in 0..5 {
            learner.record(make_exp(0.5, 500, 0.6, true));
        }

        let rec = learner.recommend("test").unwrap();
        assert!((rec.temperature - 0.5).abs() < 0.2, "should prefer successful temp");
        assert!(rec.confidence > 0.8, "confidence should be high");
    }
}