//! Clean GZMO Core
//!
//! From-scratch architecture without theatrical language.
//!
//! This crate provides a clean implementation of the GZMO ecosystem with honest,
//! descriptive naming and clear abstractions.
//!
//! # Module Structure
//!
//! - `modulation`: Deterministic parameter generation (was "chaos")
//! - `feedback`: Closed-loop optimization with repetition detection
//! - `pedagogy`: Simplified 2-agent Socratic tutoring
//! - `storage`: 2-layer storage (raw + curated)
//! - `etl`: Nightly batch processing
//! - `skills`: Function registry with dispatch
//! - `gateway`: LLM API abstraction
//! - `config`: Explicit configuration with validation
//! - `telemetry`: Metric collection and export
//! - `cli`: Command-line interface
//!
//! # Design Principles
//!
//! 1. **No Theatrical Language**: Every name describes actual behavior
//! 2. **Explicit Over Implicit**: All parameters visible and configurable
//! 3. **Measurable Outcomes**: Every decision has attached metrics
//! 4. **Minimal Indirection**: Direct data flow, no theatrical layers
//!
//! # Example
//!
//! ```rust
//! use gzmo_core_clean::modulation::StateGenerator;
//!
//! let mut gen = StateGenerator::new(0.506);
//! let (x, y, z) = gen.step();
//! let temp = gen.map_to_range(0.3, 1.2);
//! ```

pub mod modulation;
pub mod feedback;
pub mod pedagogy;
pub mod storage;
pub mod etl;
pub mod skills;
pub mod gateway;
pub mod config;
pub mod telemetry;
pub mod cli;

// Re-export commonly used types
pub use modulation::{StateGenerator, ParameterMapper, LLMParameters, AdaptiveTempo};
pub use feedback::{
    RepetitionDetector, OutputEvaluator, StrategyLearner, PatternState,
    SelfImprovingLoop, CycleResult, LoopStats, LearningStrategy
};
pub use pedagogy::{StudentEvaluator, SocraticTutor, Session, SessionConfig, KnowledgeLevel, StudentState};
pub use storage::{Vault, VectorStore, DuplicateDetector, Fact, FactRelation, SqliteVault};
pub use gateway::{LlmClient, LlmRequest, LlmResponse};
pub use config::{Config, load_from_file, save_to_file, generate_default_config};
pub use skills::{SkillRegistry, Dispatcher, Invocation, InvocationResult, SkillResult, execute};

/// Version of this crate
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Print welcome banner
pub fn welcome() {
    println!(
        r#"╔═══════════════════════════════════════╗
║   GZMO Clean Core v{}              ║
║   From-scratch architecture            ║
║   Zero theatrical language               ║
╚═══════════════════════════════════════╝"#,
        VERSION
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn version_is_defined() {
        assert!(!VERSION.is_empty());
    }
}