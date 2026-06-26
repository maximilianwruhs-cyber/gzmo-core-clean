//! Modulation Module
//!
//! Deterministic parameter generation using Lorenz ODEs.
//!
//! Components:
//! - `state_generator`: Lorenz ODE solver for smooth sequences
//! - `parameter_mapper`: Maps state to LLM parameter ranges
//! - `tempo`: Adaptive timing based on workload

pub mod state_generator;
pub mod parameter_mapper;
pub mod tempo;

pub use state_generator::{StateGenerator, SecondaryGenerator};
pub use parameter_mapper::{ParameterMapper, LLMParameters, ParameterError};
pub use tempo::AdaptiveTempo;
