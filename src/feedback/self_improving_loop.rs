//! Self-Improving Loop
//!
//! Closed-loop parameter optimization that actually adjusts generation
//! parameters based on measured outcomes.

use crate::feedback::{
    detector::{PatternState, RepetitionDetector},
    evaluator::{OutputEvaluator, QualityMetrics},
    learner::{Experience, LearningStrategy, StrategyLearner},
};
use crate::gateway::{LlmClient, LlmRequest};
use crate::modulation::parameter_mapper::{LLMParameters, ParameterMapper};
use crate::modulation::StateGenerator;

/// Complete self-improving generation loop
pub struct SelfImprovingLoop {
    /// State generator for parameter modulation
    generator: StateGenerator,
    /// Maps generator state to LLM parameters
    mapper: ParameterMapper,
    /// Detects repetitive/stuck patterns
    detector: RepetitionDetector,
    /// Evaluates output quality
    evaluator: OutputEvaluator,
    /// Learns optimal parameters from experience
    learner: StrategyLearner,
    /// Current parameters being used
    current_params: LLMParameters,
    /// LLM client for generation
    client: LlmClient,
    /// Task type for learning
    task_type: String,
    /// Whether to use learned recommendations
    use_learning: bool,
    /// Generation counter
    generation_count: u64,
    /// Successful escapes from stuck patterns
    escape_count: u64,
}

/// Result of one generation cycle
#[derive(Debug, Clone)]
pub struct CycleResult {
    pub output: String,
    pub params: LLMParameters,
    pub pattern_state: PatternState,
    pub quality: QualityMetrics,
    pub latency_ms: u64,
    pub adjusted: bool,
}

/// Statistics from the self-improving loop
#[derive(Debug, Clone)]
pub struct LoopStats {
    pub total_generations: u64,
    pub escape_count: u64,
    pub stuck_count: u64,
    pub avg_quality: f64,
    pub current_temperature: f32,
    pub learning_recommendations: usize,
}

impl SelfImprovingLoop {
    /// Create new self-improving loop with default configuration
    pub fn new(client: LlmClient, task_type: impl Into<String>) -> Self {
        let generator = StateGenerator::new(0.506);
        let mapper = ParameterMapper::default();
        
        Self {
            generator,
            mapper: mapper.clone(),
            detector: RepetitionDetector::new(),
            evaluator: OutputEvaluator::new(),
            learner: StrategyLearner::with_config(
                1000,
                0.1,
                LearningStrategy::EpsilonGreedy { epsilon: 0.2 },
            ),
            current_params: mapper.map_state(&StateGenerator::new(0.506)),
            client,
            task_type: task_type.into(),
            use_learning: true,
            generation_count: 0,
            escape_count: 0,
        }
    }

    /// Create with custom configuration
    pub fn with_config(
        client: LlmClient,
        task_type: impl Into<String>,
        detector_window: usize,
        learning_strategy: LearningStrategy,
    ) -> Self {
        let generator = StateGenerator::new(0.506);
        let mapper = ParameterMapper::default();
        
        Self {
            generator,
            mapper: mapper.clone(),
            detector: RepetitionDetector::with_config(detector_window, 0.75, 0.90, 3),
            evaluator: OutputEvaluator::new(),
            learner: StrategyLearner::with_config(1000, 0.1, learning_strategy),
            current_params: mapper.map_state(&StateGenerator::new(0.506)),
            client,
            task_type: task_type.into(),
            use_learning: true,
            generation_count: 0,
            escape_count: 0,
        }
    }

    /// Generate with self-improving parameter adjustment
    pub async fn generate(&mut self, prompt: &str) -> Result<CycleResult, SelfImprovingError> {
        let start = std::time::Instant::now();
        
        // Step 1: Check current pattern state
        let pattern_state = self.detector.current_state();
        let was_stuck = pattern_state.needs_exploration();
        
        // Step 2: Adjust parameters based on state and learning
        let adjusted = self.adjust_parameters(&pattern_state);
        
        // Step 3: Generate with adjusted parameters
        let request = LlmRequest {
            system_prompt: None,
            user_prompt: prompt.to_string(),
            params: self.current_params,
        };
        
        let response = self.client.send(request).await
            .map_err(|e| SelfImprovingError::LlmError(e.to_string()))?;
        
        let latency = start.elapsed();
        
        // Step 4: Detect patterns in output
        let new_state = self.detector.add_output(&response.text);
        
        // Step 5: Evaluate quality
        let quality = self.evaluator.evaluate(
            &response.text,
            latency,
            response.tokens_used,
        );
        
        // Step 6: Record experience for learning
        if self.use_learning {
            let success = !new_state.needs_exploration();
            let experience = Experience {
                temperature: self.current_params.temperature,
                max_tokens: self.current_params.max_tokens,
                quality_score: quality.score(),
                success,
                task_type: self.task_type.clone(),
                timestamp: self.generation_count,
            };
            self.learner.record(experience);
        }
        
        // Step 7: Track escapes
        if was_stuck && !new_state.needs_exploration() {
            self.escape_count += 1;
        }
        
        self.generation_count += 1;
        
        Ok(CycleResult {
            output: response.text,
            params: self.current_params,
            pattern_state: new_state,
            quality,
            latency_ms: latency.as_millis() as u64,
            adjusted,
        })
    }

    /// Adjust parameters based on pattern state and learning
    fn adjust_parameters(&mut self, state: &PatternState) -> bool {
        let mut adjusted = false;
        
        // Get base parameters from generator
        self.generator.step();
        let base_params = self.mapper.map_state(&self.generator);
        
        // Apply learning-based recommendation if available
        if self.use_learning {
            if let Some(rec) = self.learner.recommend(&self.task_type) {
                // Blend learned parameters with generator (70% learned, 30% generator)
                self.current_params.temperature = 
                    rec.temperature * 0.7 + base_params.temperature * 0.3;
                self.current_params.max_tokens = 
                    ((rec.max_tokens as f32 * 0.7) + (base_params.max_tokens as f32 * 0.3)) as u32;
                adjusted = true;
            } else {
                self.current_params = base_params;
            }
        } else {
            self.current_params = base_params;
        }
        
        // Apply exploration boost if stuck
        if state.needs_exploration() {
            let boost = state.exploration_level() as f32;
            self.current_params.temperature = 
                (self.current_params.temperature + boost * 0.5).min(1.5);
            self.current_params.max_tokens = 
                (self.current_params.max_tokens as f32 * 1.2).min(4096.0) as u32;
            adjusted = true;
        }
        
        adjusted
    }

    /// Get current loop statistics
    pub fn stats(&self) -> LoopStats {
        let stuck_count = self.detector.current_state().needs_exploration() as u64;
        
        LoopStats {
            total_generations: self.generation_count,
            escape_count: self.escape_count,
            stuck_count,
            avg_quality: 0.0, // Would need to track running average
            current_temperature: self.current_params.temperature,
            learning_recommendations: self.learner.stats(&self.task_type)
                .map(|s| s.total_samples)
                .unwrap_or(0),
        }
    }

    /// Get current recommendation from learner
    pub fn current_recommendation(&self) -> Option<crate::feedback::learner::ParameterRecommendation> {
        self.learner.recommend(&self.task_type)
    }

    /// Reset all learning and detection state
    pub fn reset(&mut self) {
        self.detector.reset();
        self.evaluator.reset();
        self.learner.reset();
        self.generation_count = 0;
        self.escape_count = 0;
    }

    /// Enable/disable learning
    pub fn set_learning(&mut self, enabled: bool) {
        self.use_learning = enabled;
    }
}

#[derive(Debug)]
pub enum SelfImprovingError {
    LlmError(String),
    PatternError(String),
}

impl std::fmt::Display for SelfImprovingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SelfImprovingError::LlmError(s) => write!(f, "LLM error: {}", s),
            SelfImprovingError::PatternError(s) => write!(f, "Pattern error: {}", s),
        }
    }
}

impl std::error::Error for SelfImprovingError {}

#[cfg(test)]
mod tests {
    use super::*;

    // Note: These tests would need a mock LLM client to run properly
    // For now they just verify the structure compiles

    #[test]
    fn loop_stats_track_generations() {
        // This is a structural test - real tests need mock client
        let stats = LoopStats {
            total_generations: 10,
            escape_count: 2,
            stuck_count: 1,
            avg_quality: 0.75,
            current_temperature: 0.7,
            learning_recommendations: 5,
        };
        
        assert_eq!(stats.total_generations, 10);
        assert_eq!(stats.escape_count, 2);
    }
}