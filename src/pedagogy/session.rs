//! Pedagogy Session Manager
//!
//! Manages a complete learning session with LLM-based state tracking,
//! evaluation, and response generation.

use crate::gateway::LlmClient;
use crate::pedagogy::evaluator::{EvaluatorError, KnowledgeLevel, StudentEvaluator, StudentState};
use crate::pedagogy::tutor::{SocraticTutor, TutorError, TutorResponse};
use std::time::{Duration, Instant};

/// A complete learning session with LLM integration
pub struct Session {
    pub id: String,
    pub subject: String,
    pub student_id: String,
    pub state: SessionState,
    pub interactions: Vec<Interaction>,
    pub created_at: Instant,
    pub max_interactions: usize,
    evaluator: StudentEvaluator,
    tutor: SocraticTutor,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionState {
    Active,
    Paused,
    Completed,
    Abandoned,
}

#[derive(Debug, Clone)]
pub struct Interaction {
    pub seq: usize,
    pub student_input: String,
    pub tutor_response: String,
    pub student_state: StudentState,
    pub response_meta: ResponseMeta,
    pub timestamp: Instant,
}

#[derive(Debug, Clone)]
pub struct ResponseMeta {
    pub difficulty: f64,
    pub had_question: bool,
    pub cognitive_load: u8,
    pub latency_ms: u64,
    pub llm_calls: u32,
}

#[derive(Debug, Clone)]
pub struct SessionConfig {
    pub subject: String,
    pub student_id: String,
    pub max_interactions: usize,
    pub initial_level: KnowledgeLevel,
    pub llm_endpoint: String,
    pub llm_model: String,
}

impl Default for SessionConfig {
    fn default() -> Self {
        Self {
            subject: "general".to_string(),
            student_id: "anonymous".to_string(),
            max_interactions: 20,
            initial_level: KnowledgeLevel::Developing,
            llm_endpoint: "http://localhost:8000/v1/chat/completions".to_string(),
            llm_model: "local-model".to_string(),
        }
    }
}

/// Error types for session operations
#[derive(Debug)]
pub enum SessionError {
    EvaluatorError(EvaluatorError),
    TutorError(TutorError),
    SessionEnded,
}

impl std::fmt::Display for SessionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SessionError::EvaluatorError(e) => write!(f, "Evaluator error: {}", e),
            SessionError::TutorError(e) => write!(f, "Tutor error: {}", e),
            SessionError::SessionEnded => write!(f, "Session has ended"),
        }
    }
}

impl std::error::Error for SessionError {}

impl From<EvaluatorError> for SessionError {
    fn from(e: EvaluatorError) -> Self {
        SessionError::EvaluatorError(e)
    }
}

impl From<TutorError> for SessionError {
    fn from(e: TutorError) -> Self {
        SessionError::TutorError(e)
    }
}

impl Session {
    /// Create new session with LLM clients
    pub fn new(config: SessionConfig) -> Self {
        let client = LlmClient::new(&config.llm_endpoint, "", &config.llm_model);
        
        Self {
            id: generate_session_id(),
            subject: config.subject,
            student_id: config.student_id,
            state: SessionState::Active,
            interactions: Vec::with_capacity(config.max_interactions),
            created_at: Instant::now(),
            max_interactions: config.max_interactions,
            evaluator: StudentEvaluator::new(client.clone()),
            tutor: SocraticTutor::new(client),
        }
    }

    /// Create with custom LLM client
    pub fn with_client(config: SessionConfig, client: LlmClient) -> Self {
        Self {
            id: generate_session_id(),
            subject: config.subject,
            student_id: config.student_id,
            state: SessionState::Active,
            interactions: Vec::with_capacity(config.max_interactions),
            created_at: Instant::now(),
            max_interactions: config.max_interactions,
            evaluator: StudentEvaluator::new(client.clone()),
            tutor: SocraticTutor::new(client),
        }
    }

    /// Process student message with LLM evaluation and tutoring
    pub async fn interact(&mut self, student_input: impl Into<String>) -> Result<SessionResult, SessionError> {
        if self.state != SessionState::Active {
            return Err(SessionError::SessionEnded);
        }

        let input = student_input.into();
        let start = Instant::now();
        let mut llm_calls = 0;

        // Step 1: LLM-based student evaluation
        let previous = self.interactions.last().map(|i| i.student_input.clone());
        let student_state = match self.evaluator.evaluate(&input, &self.subject, previous.as_deref()).await {
            Ok(state) => {
                llm_calls += 1;
                state
            }
            Err(_) => {
                // Fallback to heuristic if LLM fails
                self.evaluator.evaluate_heuristic(&input, &self.subject)
            }
        };

        // Step 2: LLM-based tutor response
        let response = match self.tutor.respond(&input, &student_state, &self.subject).await {
            Ok(resp) => {
                llm_calls += 1;
                resp
            }
            Err(_) => {
                // Fallback if LLM fails
                self.tutor.respond_fallback(&input, &student_state, &self.subject)
            }
        };

        let total_latency = start.elapsed();

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
                latency_ms: total_latency.as_millis() as u64,
                llm_calls,
            },
            timestamp: Instant::now(),
        };

        self.interactions.push(interaction);

        // Check if session should end
        if self.interactions.len() >= self.max_interactions {
            self.state = SessionState::Completed;
        }

        Ok(SessionResult::Response {
            text: response.text,
            meta: ResponseMeta {
                difficulty: response.difficulty,
                had_question: response.has_question,
                cognitive_load: response.cognitive_load,
                latency_ms: total_latency.as_millis() as u64,
                llm_calls,
            },
            remaining_interactions: self.max_interactions.saturating_sub(self.interactions.len()),
        })
    }

    /// Quick interaction without async (uses fallbacks)
    pub fn interact_sync(&mut self, student_input: impl Into<String>) -> SessionResult {
        if self.state != SessionState::Active {
            return SessionResult::SessionEnded;
        }

        let input = student_input.into();
        let start = Instant::now();

        // Use heuristic evaluation
        let student_state = self.evaluator.evaluate_heuristic(&input, &self.subject);

        // Use fallback tutor response
        let response = self.tutor.respond_fallback(&input, &student_state, &self.subject);

        let latency = start.elapsed();

        let interaction = Interaction {
            seq: self.interactions.len() + 1,
            student_input: input,
            tutor_response: response.text.clone(),
            student_state,
            response_meta: ResponseMeta {
                difficulty: response.difficulty,
                had_question: response.has_question,
                cognitive_load: response.cognitive_load,
                latency_ms: latency.as_millis() as u64,
                llm_calls: 0,
            },
            timestamp: Instant::now(),
        };

        self.interactions.push(interaction);

        if self.interactions.len() >= self.max_interactions {
            self.state = SessionState::Completed;
        }

        SessionResult::Response {
            text: response.text,
            meta: ResponseMeta {
                difficulty: response.difficulty,
                had_question: response.has_question,
                cognitive_load: response.cognitive_load,
                latency_ms: latency.as_millis() as u64,
                llm_calls: 0,
            },
            remaining_interactions: self.max_interactions.saturating_sub(self.interactions.len()),
        }
    }

    pub fn current_state(&self) -> Option<&StudentState> {
        self.interactions.last().map(|i| &i.student_state)
    }

    pub fn end(&mut self) {
        self.state = SessionState::Completed;
    }

    pub fn duration(&self) -> Duration {
        self.created_at.elapsed()
    }

    pub fn stats(&self) -> SessionStats {
        let total = self.interactions.len();
        if total == 0 {
            return SessionStats::default();
        }

        let questions = self.interactions.iter().filter(|i| i.response_meta.had_question).count();
        let avg_load: u64 = self.interactions.iter().map(|i| i.response_meta.cognitive_load as u64).sum::<u64>() / total as u64;
        let avg_difficulty: f64 = self.interactions.iter().map(|i| i.response_meta.difficulty).sum::<f64>() / total as f64;
        let avg_latency: u64 = self.interactions.iter().map(|i| i.response_meta.latency_ms).sum::<u64>() / total as u64;
        let total_llm_calls: u32 = self.interactions.iter().map(|i| i.response_meta.llm_calls).sum();

        SessionStats {
            total_interactions: total,
            tutor_questions: questions,
            avg_cognitive_load: avg_load as u8,
            avg_difficulty,
            avg_latency_ms: avg_latency,
            duration_seconds: self.duration().as_secs(),
            total_llm_calls,
        }
    }
}

#[derive(Debug)]
pub enum SessionResult {
    Response {
        text: String,
        meta: ResponseMeta,
        remaining_interactions: usize,
    },
    SessionEnded,
}

#[derive(Debug, Clone)]
pub struct SessionStats {
    pub total_interactions: usize,
    pub tutor_questions: usize,
    pub avg_cognitive_load: u8,
    pub avg_difficulty: f64,
    pub avg_latency_ms: u64,
    pub duration_seconds: u64,
    pub total_llm_calls: u32,
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
            total_llm_calls: 0,
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
            llm_endpoint: "http://localhost:8000/v1/chat/completions".to_string(),
            llm_model: "local-model".to_string(),
        })
    }

    #[test]
    fn session_creates_with_id() {
        let session = make_session();
        assert!(session.id.starts_with("sess_"));
        assert_eq!(session.subject, "geometry");
    }

    #[test]
    fn interact_sync_returns_response() {
        let mut session = make_session();
        let result = session.interact_sync("hello");

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
            session.interact_sync("question");
        }

        assert_eq!(session.state, SessionState::Completed);

        let result = session.interact_sync("extra");
        assert!(matches!(result, SessionResult::SessionEnded));
    }

    #[test]
    fn stats_calculate_correctly() {
        let mut session = make_session();
        session.interact_sync("q1");
        session.interact_sync("q2");

        let stats = session.stats();
        assert_eq!(stats.total_interactions, 2);
        assert_eq!(stats.total_llm_calls, 0); // Sync uses fallbacks
    }
}