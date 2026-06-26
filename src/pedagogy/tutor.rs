//! Socratic Tutor
//!
//! Generates Socratic responses based on student state.
//! Uses guiding questions rather than direct answers to promote learning.

use crate::pedagogy::evaluator::{KnowledgeLevel, StudentState};

/// Tutor response generator
pub struct SocraticTutor;

/// Generated tutor output
#[derive(Debug, Clone)]
pub struct TutorResponse {
    /// Response text for the student
    pub text: String,
    /// Target difficulty applied (0-1)
    pub difficulty: f64,
    /// Whether this response contains a question
    pub has_question: bool,
    /// Estimated cognitive load (1-5)
    pub cognitive_load: u8,
}

impl SocraticTutor {
    /// Create new tutor
    pub fn new() -> Self {
        Self
    }

    /// Generate a Socratic response
    ///
    /// # Arguments
    /// * `student_input` - The student's message
    /// * `state` - Current evaluated student state
    /// * `subject` - Topic being learned
    pub fn respond(&self, student_input: &str, state: &StudentState, subject: &str) -> TutorResponse {
        let complexity = state.level.complexity();
        let zpd = state.difficulty_target;

        // Check if student asked a direct question
        let is_question = student_input.ends_with('?')
            || student_input.to_lowercase().starts_with("what")
            || student_input.to_lowercase().starts_with("how");

        let text = if is_question {
            self.respond_to_question(student_input, state, subject, complexity)
        } else {
            self.respond_to_statement(student_input, state, subject, complexity)
        };

        let cognitive_load = self.estimate_cognitive_load(&text, zpd);

        TutorResponse {
            text,
            difficulty: zpd,
            has_question: self.contains_question_marker(&text),
            cognitive_load,
        }
    }

    /// Respond to a student question
    fn respond_to_question(
        &self,
        question: &str,
        state: &StudentState,
        subject: &str,
        complexity: &str,
    ) -> String {
        // Socratic response: don't answer directly
        let question_lower = question.to_lowercase();

        if question_lower.contains("why") || question_lower.contains("how") {
            // Process questions
            format!(
                "That's an interesting {} question about {}. \
                Before we explore that, what do you think are the key factors involved? \
                Consider what you already know about {}.",
                complexity,
                subject,
                state.mastered_prerequisites.first().unwrap_or(&subject.to_string()),
            )
        } else if question_lower.contains("what is") || question_lower.contains("what are") {
            // Definition questions
            format!(
                "To understand {}, let's break it down. \
                What components do you think {} involves? \
                What happens if we remove one of those components?",
                subject,
                subject,
            )
        } else {
            // General questions
            format!(
                "Good question. Let's think through this together using {} reasoning. \
                What would happen if we approached this differently? \
                What constraints or assumptions are we making?",
                complexity,
            )
        }
    }

    /// Respond to a student statement
    fn respond_to_statement(
        &self,
        statement: &str,
        state: &StudentState,
        subject: &str,
        complexity: &str,
    ) -> String {
        // Check for misconceptions
        if !state.misconception_topics.is_empty() {
            return format!(
                "I notice you're thinking about {}. \
                Let's examine this more carefully. \
                Can you think of a counterexample or exception to what you've stated? \
                What would {} look like in that case?",
                state.misconception_topics[0],
                subject,
            );
        }

        // Check for gaps
        if !state.prerequisite_gaps.is_empty() {
            return format!(
                "Before we go deeper, let's make sure we have the foundation. \
                You mentioned understanding {}, but how does that connect to {}? \
                Can you explain that relationship?",
                subject,
                state.prerequisite_gaps[0],
            );
        }

        // Default Socratic probe
        format!(
            "You mentioned: \"{}\". \
            Let's explore that with {} detail. \
            What evidence supports your conclusion? \
            How would you explain this to someone at a beginner level?",
            if statement.len() > 50 {
                &statement[..50]
            } else {
                statement
            },
            complexity,
        )
    }

    /// Check if text contains a question
    fn contains_question_marker(&self, text: &str) -> bool {
        text.contains('?')
    }

    /// Estimate cognitive load (1-5) based on complexity and length
    fn estimate_cognitive_load(&self, text: &str, _zpd: f64) -> u8 {
        let sentences = text.matches('.').count() + text.matches('?').count();
        let words = text.split_whitespace().count();
        let avg_words_per_sentence = if sentences > 0 {
            words / sentences
        } else {
            words
        };

        match avg_words_per_sentence {
            0..=10 => 1,
            11..=20 => 2,
            21..=30 => 3,
            31..=40 => 4,
            _ => 5,
        }
    }

    /// Generate the LLM system prompt for this tutor
    pub fn system_prompt(&self, state: &StudentState, subject: &str) -> String {
        state.tutor_prompt(subject)
    }

    /// Generate the LLM user prompt
    pub fn user_prompt(&self, student_input: &str) -> String {
        format!(
            "Student input: {}\n\n\
            Respond with a Socratic question that guides the student toward understanding. \
            Do not give the answer directly. \
            If the student made an error, use a counterexample or probing question to help them discover it.",
            student_input
        )
    }
}

impl Default for SocraticTutor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pedagogy::evaluator::{KnowledgeLevel, StudentState};

    #[test]
    fn question_gets_socratic_response() {
        let tutor = SocraticTutor::new();
        let state = StudentState::from_level(KnowledgeLevel::Novice);
        let response = tutor.respond("what is gravity?", &state, "physics");
        assert!(response.has_question);
        assert!(!response.text.contains("Gravity is")); // No direct definition
    }

    #[test]
    fn misconception_gets_probing_response() {
        let tutor = SocraticTutor::new();
        let state = StudentState {
            level: KnowledgeLevel::Developing,
            misconception_topics: vec!["overgeneralization".to_string()],
            mastered_prerequisites: vec![],
            prerequisite_gaps: vec![],
            objectives: vec![],
            difficulty_target: 0.5,
        };
        let response = tutor.respond("All functions are linear", &state, "math");
        assert!(response.text.contains("counterexample") || response.text.contains("examine"));
    }

    #[test]
    fn system_prompt_includes_level() {
        let tutor = SocraticTutor::new();
        let state = StudentState::from_level(KnowledgeLevel::Expert);
        let prompt = tutor.system_prompt(&state, "calculus");
        assert!(prompt.contains("expert"));
    }

    #[test]
    fn higher_complexity_increases_load() {
        let tutor = SocraticTutor::new();

        let novice = StudentState::from_level(KnowledgeLevel::Novice);
        let r1 = tutor.respond("test", &novice, "subject");

        let expert = StudentState::from_level(KnowledgeLevel::Expert);
        let r2 = tutor.respond("test", &expert, "subject");

        // Expert responses typically longer/higher load
        assert!(r2.text.len() >= r1.text.len() || r2.cognitive_load >= r1.cognitive_load);
    }

    #[test]
    fn why_question_gets_process_probe() {
        let tutor = SocraticTutor::new();
        let state = StudentState::from_level(KnowledgeLevel::Developing);
        let response = tutor.respond("why does photosynthesis work?", &state, "biology");
        assert!(response.text.contains("factors") || response.text.contains("think"));
    }
}