//! Socratic Tutor
//!
//! Generates Socratic responses using LLM to create authentic, adaptive tutoring.
//! Replaces hardcoded templates with actual language model generation.

use crate::gateway::{LlmClient, LlmRequest};
use crate::modulation::parameter_mapper::LLMParameters;
use crate::pedagogy::evaluator::StudentState;

/// LLM-powered Socratic tutor
pub struct SocraticTutor {
    client: LlmClient,
}

/// Generated tutor output with metadata
#[derive(Debug, Clone)]
pub struct TutorResponse {
    pub text: String,
    pub difficulty: f64,
    pub has_question: bool,
    pub cognitive_load: u8,
    pub latency_ms: u64,
}

impl SocraticTutor {
    /// Create new tutor with LLM client
    pub fn new(client: LlmClient) -> Self {
        Self { client }
    }

    /// Create tutor with default local endpoint
    pub fn with_endpoint(endpoint: &str, model: &str) -> Self {
        Self {
            client: LlmClient::new(endpoint, "", model),
        }
    }

    /// Generate Socratic response using LLM
    pub async fn respond(
        &self,
        student_input: &str,
        state: &StudentState,
        subject: &str,
    ) -> Result<TutorResponse, TutorError> {
        let start = std::time::Instant::now();

        let system_prompt = self.build_system_prompt(state, subject);
        let user_prompt = self.build_user_prompt(student_input, state);

        // Adjust temperature based on student level
        let temperature = match state.level {
            crate::pedagogy::evaluator::KnowledgeLevel::Novice => 0.4,
            crate::pedagogy::evaluator::KnowledgeLevel::Developing => 0.5,
            crate::pedagogy::evaluator::KnowledgeLevel::Proficient => 0.6,
            crate::pedagogy::evaluator::KnowledgeLevel::Expert => 0.7,
        };

        let request = LlmRequest {
            system_prompt: Some(system_prompt),
            user_prompt,
            params: LLMParameters {
                temperature,
                max_tokens: 300,
                top_p: 0.9,
            },
        };

        let response = self.client.send(request).await
            .map_err(|e| TutorError::LlmError(e.to_string()))?;

        let text = response.text.trim().to_string();
        let has_question = text.contains('?');
        let cognitive_load = self.estimate_cognitive_load(&text, state.difficulty_target);

        Ok(TutorResponse {
            text,
            difficulty: state.difficulty_target,
            has_question,
            cognitive_load,
            latency_ms: response.latency.as_millis() as u64,
        })
    }

    /// Build system prompt from student state
    fn build_system_prompt(&self, state: &StudentState, subject: &str) -> String {
        let mut prompt = format!(
            "You are an expert Socratic tutor teaching {}. \
            The student is at {:?} level. \
            Use {} language appropriate for their level. \
            Target difficulty: {:.0}%.",
            subject,
            state.level,
            format!("{:?}", state.level).to_lowercase(),
            state.difficulty_target * 100.0,
        );

        if !state.misconception_topics.is_empty() {
            prompt.push_str(&format!(
                "\n\nAddress these student misconceptions through questioning: {}. \
                Do not state the misconception directly. Use questions to help them discover the error.",
                state.misconception_topics.join(", ")
            ));
        }

        if !state.prerequisite_gaps.is_empty() {
            prompt.push_str(&format!(
                "\n\nThe student has gaps in: {}. \
    Guide them to recognize what they need to understand first.",
                state.prerequisite_gaps.join(", ")
            ));
        }

        prompt.push_str(&format!(
            "\n\nGuidelines:\n\
            - Ask guiding questions, never give direct answers\n\
            - Help the student discover knowledge through inquiry\n\
            - If they made an error, use counterexamples or probing questions\n\
            - Respond in 2-4 sentences\n\
            - End with a question that advances their thinking"
        ));

        prompt
    }

    /// Build user prompt with context
    fn build_user_prompt(&self, student_input: &str, state: &StudentState) -> String {
        let context = if !state.evaluation_reasoning.is_empty() {
            format!("\n\nAssessment context: {}", state.evaluation_reasoning)
        } else {
            String::new()
        };

        format!(
            "Student says: \"{}\"{}\n\nRespond with a Socratic question that guides their thinking.",
            student_input.replace('"', "\\\""),
            context
        )
    }

    /// Estimate cognitive load based on text characteristics
    fn estimate_cognitive_load(&self, text: &str, zpd: f64) -> u8 {
        let sentences = text.matches('.').count() + text.matches('?').count();
        let words = text.split_whitespace().count();
        let question_count = text.matches('?').count();

        // Base load from complexity
        let base_load = if words < 20 {
            1
        } else if words < 40 {
            2
        } else if words < 60 {
            3
        } else if words < 80 {
            4
        } else {
            5
        };

        // Adjust for questions (more questions = higher engagement load)
        let question_factor = question_count.min(2);

        // Adjust for ZPD match
        let zpd_adjustment = if zpd > 0.7 { 1 } else { 0 };

        ((base_load + question_factor + zpd_adjustment).min(5)).max(1) as u8
    }

    /// Quick fallback response when LLM unavailable
    pub fn respond_fallback(
        &self,
        student_input: &str,
        state: &StudentState,
        subject: &str,
    ) -> TutorResponse {
        let templates = [
            format!(
                "Let's think about {} together. What do you already know about this topic?",
                subject
            ),
            format!(
                "That's an interesting point about {}. What makes you think that?",
                subject
            ),
            format!(
                "Good question about {}. Before we dive in, what do you think the key concepts are?",
                subject
            ),
            format!(
                "I see you're working through {}. What would happen if we looked at it from a different angle?",
                subject
            ),
        ];

        // Select based on student level
        let index = match state.level {
            crate::pedagogy::evaluator::KnowledgeLevel::Novice => 0,
            crate::pedagogy::evaluator::KnowledgeLevel::Developing => 1,
            crate::pedagogy::evaluator::KnowledgeLevel::Proficient => 2,
            crate::pedagogy::evaluator::KnowledgeLevel::Expert => 3,
        };

        let text = templates[index % templates.len()].clone();

        TutorResponse {
            text,
            difficulty: state.difficulty_target,
            has_question: true,
            cognitive_load: 2,
            latency_ms: 0,
        }
    }
}

impl Default for SocraticTutor {
    fn default() -> Self {
        Self::with_endpoint("http://localhost:8000/v1/chat/completions", "local-model")
    }
}

#[derive(Debug)]
pub enum TutorError {
    LlmError(String),
}

impl std::fmt::Display for TutorError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TutorError::LlmError(s) => write!(f, "LLM error: {}", s),
        }
    }
}

impl std::error::Error for TutorError {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pedagogy::evaluator::{KnowledgeLevel, StudentState};

    #[test]
    fn fallback_generates_question() {
        let tutor = SocraticTutor::default();
        let state = StudentState::from_level(KnowledgeLevel::Novice);
        let response = tutor.respond_fallback("what is gravity?", &state, "physics");
        assert!(response.has_question);
        assert!(!response.text.is_empty());
    }

    #[test]
    fn cognitive_load_based_on_length() {
        let tutor = SocraticTutor::default();
        let state = StudentState::from_level(KnowledgeLevel::Developing);

        let short = tutor.respond_fallback("test", &state, "math");
        assert!(short.cognitive_load <= 2);
    }

    #[test]
    fn system_prompt_includes_misconceptions() {
        let tutor = SocraticTutor::default();
        let mut state = StudentState::from_level(KnowledgeLevel::Developing);
        state.misconception_topics.push("test_error".to_string());

        let prompt = tutor.build_system_prompt(&state, "math");
        assert!(prompt.contains("test_error"));
    }
}