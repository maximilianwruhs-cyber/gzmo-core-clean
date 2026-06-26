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

pub use evaluator::{KnowledgeLevel, StudentEvaluator, StudentState};
pub use tutor::{SocraticTutor, TutorResponse};
pub use session::{Session, SessionConfig, SessionState, Interaction, SessionStats, SessionResult};
