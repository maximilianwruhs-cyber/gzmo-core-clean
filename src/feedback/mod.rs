//! Feedback Module
//!
//! Closed-loop optimization for LLM parameter modulation.
//!
//! Components:
//! - `detector`: Repetition detection using n-gram and Jaccard similarity
//! - `evaluator`: Output quality metrics (diversity, novelty, latency)
//! - `learner`: Strategy optimization from parameter-outcome pairs
//! - `queue`: Delayed parameter mutation queue (was "Thought Cabinet")

pub mod detector;
pub mod evaluator;
pub mod learner;
pub mod queue;

pub use detector::{PatternState, RepetitionDetector};
pub use evaluator::{QualityMetrics, OutputEvaluator};
pub use learner::{Experience, LearningStrategy, StrategyLearner, ParameterRecommendation, LearnerStats};
pub use queue::{ParameterRequest, ParameterTarget, ParameterMutationQueue, AppliedMutation, QueueResult};
