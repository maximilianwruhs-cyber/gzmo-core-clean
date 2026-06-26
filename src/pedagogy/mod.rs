//! Pedagogy Module
//!
//! Simplified 2-agent system for Socratic tutoring.
//!
//! Components:
//! - `evaluator`: Student state assessment (was 4-agent Diagnoser/Planner/Affective)
//! - `tutor`: Socratic response generation
//! - `session`: Complete session management

pub mod evaluator;
pub mod tutor;
pub mod session;

pub use evaluator::{KnowledgeLevel, StudentEvaluator, StudentState, EvaluatorError};
pub use tutor::{SocraticTutor, TutorResponse, TutorError};
pub use session::{Session, SessionConfig, SessionState, Interaction, SessionStats, SessionResult, SessionError, ResponseMeta};
