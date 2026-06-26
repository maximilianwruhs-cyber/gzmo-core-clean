//! Pedagogy Session Manager
//!
//! Manages a complete learning session with state tracking,
//! evaluation, and response generation.

use crate::pedagogy::evaluator::{KnowledgeLevel, StudentEvaluator, StudentState};
use crate::pedagogy::tutor::{SocraticTutor, TutorResponse};
use std::time::{Duration, Instant};

/// A complete learning session
pub struct Session {
    /// Session unique identifier
    pub id: String,
    /// Subject/topic being learned
    pub subject: String,
    /// Student identifier
    pub student_id: String,
    /// Session state
    pub state: SessionState,
    /// Interaction history
    pub interactions: Vec<Interaction>,
    /// Created timestamp
    pub created_at: Instant,
    /// Maximum interactions before session ends
    pub max_interactions: usize,
}

/// Session state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionState {
    Active,
    Paused,
    Completed,
    Abandoned,
}

/// Single student-tutor interaction
#[derive(Debug, Clone)]
pub struct Interaction {
    /// Interaction sequence number
    pub seq: usize,
    /// Student input
    pub student_input: String,
    /// Tutor response
    pub tutor_response: String,
    /// Student state at time of evaluation
    pub student_state: StudentState,
    /// Response quality metrics
    pub response_meta: ResponseMeta,
    /// Timestamp
    pub timestamp: Instant,
}

/// Response metadata
#[derive(Debug, Clone)]
pub struct ResponseMeta {
    /// Target difficulty applied
    pub difficulty: f64,
    /// Whether response contained a question
    pub had_question: bool,
    /// Estimated cognitive load
    pub cognitive_load: u8,
    /// Response generation latency
    pub latency_ms: u64,
}

/// Session configuration
#[derive(Debug, Clone)]
pub struct SessionConfig {
    pub subject: String,
    pub student_id: String,
    pub max_interactions: usize,
    pub initial_level: KnowledgeLevel,
}

impl Session {
    /// Create new session
    pub fn new(config: SessionConfig) -> Self {
        Self {
            id: generate_session_id(),
            subject: config.subject,
            student_id: config.student_id,
            state: SessionState::Active,
            interactions: Vec::with_capacity(config.max_interactions),
            created_at: Instant::now(),
            max_interactions: config.max_interactions,
        }
    }

    /// Process a student message and return tutor response
    ///
    /// # Arguments
    /// * `student_input` - The student's message
    pub fn interact(&mut self, student_input: impl Into<String>) -> SessionResult {
        if self.state != SessionState::Active {
            return SessionResult::SessionEnded;
        }

        let input = student_input.into();
        let start = Instant::now();

        // Evaluate student state
        let evaluator = StudentEvaluator::new();
        let previous = self.interactions.last().map(|i| i.student_input.as_str());
        let student_state = evaluator.evaluate(&input, &self.subject, previous);

        // Generate tutor response
        let tutor = SocraticTutor::new();
        let response = tutor.respond(&input, &student_state, &self.subject);
        let latency = start.elapsed();

        // Record interaction
        let interaction = Interaction {
            seq: self.interactions.len() + 1,
            student_input: input,
            tutor_response: response.text.clone(),
            student_state: student_state.clone(),
            response_meta: ResponseMeta {
                difficulty: response.difficulty,
                had_question: response.has_question,
                cognitive_load: response.cognitive_load,
                latency_ms: latency.as_millis() as u64,
            },
            timestamp: Instant::now(),
        };

        self.interactions.push(interaction);

        // Check if session should end
        if self.interactions.len() >= self.max_interactions {
            self.state = SessionState::Completed;
        }

        SessionResult::Response {
            text: response.text,
            meta: response.into_meta(latency),
            remaining_interactions: self.max_interactions - self.interactions.len(),
        }
    }

    /// Get current student state estimate
    pub fn current_state(&self) -> Option<&StudentState> {
        self.interactions.last().map(|i| &i.student_state)
    }

    /// End the session
    pub fn end(&mut self) {
        self.state = SessionState::Completed;
    }

    /// Session duration
    pub fn duration(&self) -> Duration {
        self.created_at.elapsed()
    }

    /// Summary statistics
    pub fn stats(&self) -> SessionStats {
        let total = self.interactions.len();
        if total == 0 {
            return SessionStats::default();
        }

        let questions = self.interactions.iter().filter(|i| i.response_meta.had_question).count();
        let avg_load: u64 = self.interactions.iter().map(|i| i.response_meta.cognitive_load as u64).sum::<u64>() / total as u64;
        let avg_difficulty: f64 = self.interactions.iter().map(|i| i.response_meta.difficulty).sum::<f64>() / total as f64;
        let avg_latency: u64 = self.interactions.iter().map(|i| i.response_meta.latency_ms).sum::<u64>() / total as u64;

        SessionStats {
            total_interactions: total,
            tutor_questions: questions,
            avg_cognitive_load: avg_load as u8,
            avg_difficulty,
            avg_latency_ms: avg_latency,
            duration_seconds: self.duration().as_secs(),
        }
    }
}

/// Result of a session interaction
#[derive(Debug)]
pub enum SessionResult {
    Response {
        text: String,
        meta: ResponseMeta,
        remaining_interactions: usize,
    },
    SessionEnded,
}

/// Session summary statistics
#[derive(Debug, Clone)]
pub struct SessionStats {
    pub total_interactions: usize,
    pub tutor_questions: usize,
    pub avg_cognitive_load: u8,
    pub avg_difficulty: f64,
    pub avg_latency_ms: u64,
    pub duration_seconds: u64,
}

impl Default for SessionStats {
    fn default() -> Self {
        Self {
            total_interactions: 0,
            tutor_questions: 0,
            avg_cognitive_load: 0,
            avg_difficulty: 0.0,
            avg_latency_ms: 0,
            duration_seconds: 0,
        }
    }
}

impl TutorResponse {
    fn into_meta(self, latency: Duration) -> ResponseMeta {
        ResponseMeta {
            difficulty: self.difficulty,
            had_question: self.has_question,
            cognitive_load: self.cognitive_load,
            latency_ms: latency.as_millis() as u64,
        }
    }
}

fn generate_session_id() -> String {
    use std::time::SystemTime;
    let now = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    format!("sess_{}", now)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_session() -> Session {
        Session::new(SessionConfig {
            subject: "geometry".to_string(),
            student_id: "stu_123".to_string(),
            max_interactions: 5,
            initial_level: KnowledgeLevel::Novice,
        })
    }

    #[test]
    fn session_creates_with_id() {
        let session = make_session();
        assert!(session.id.starts_with("sess_"));
        assert_eq!(session.subject, "geometry");
    }

    #[test]
    fn interact_returns_response() {
        let mut session = make_session();
        let result = session.interact("hello");

        match result {
            SessionResult::Response { text, remaining_interactions, .. } => {
                assert!(!text.is_empty());
                assert_eq!(remaining_interactions, 4);
            }
            SessionResult::SessionEnded => panic!("session should be active"),
        }
    }

    #[test]
    fn session_ends_after_max_interactions() {
        let mut session = make_session();

        for _ in 0..5 {
            session.interact("question");
        }

        assert_eq!(session.state, SessionState::Completed);

        let result = session.interact("extra");
        assert!(matches!(result, SessionResult::SessionEnded));
    }

    #[test]
    fn stats_calculate_correctly() {
        let mut session = make_session();
        session.interact("q1");
        session.interact("q2");

        let stats = session.stats();
        assert_eq!(stats.total_interactions, 2);
    }

    #[test]
    fn end_session_manually() {
        let mut session = make_session();
        session.interact("test");
        session.end();

        assert_eq!(session.state, SessionState::Completed);
    }
}