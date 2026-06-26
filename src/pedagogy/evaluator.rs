//! Student State Evaluator
//!
//! Evaluates student knowledge state using LLM analysis.
//! Replaces heuristic evaluation with actual language model assessment.

use crate::gateway::{LlmClient, LlmRequest};
use crate::modulation::parameter_mapper::LLMParameters;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

/// Student knowledge level
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum KnowledgeLevel {
    Novice,
    Developing,
    Proficient,
    Expert,
}

impl KnowledgeLevel {
    pub fn zpd_difficulty(&self) -> f64 {
        match self {
            KnowledgeLevel::Novice => 0.3,
            KnowledgeLevel::Developing => 0.5,
            KnowledgeLevel::Proficient => 0.7,
            KnowledgeLevel::Expert => 0.9,
        }
    }

    pub fn complexity(&self) -> &'static str {
        match self {
            KnowledgeLevel::Novice => "simple",
            KnowledgeLevel::Developing => "moderate",
            KnowledgeLevel::Proficient => "technical",
            KnowledgeLevel::Expert => "advanced",
        }
    }
}

impl std::str::FromStr for KnowledgeLevel {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "novice" | "beginner" => Ok(KnowledgeLevel::Novice),
            "developing" | "intermediate" => Ok(KnowledgeLevel::Developing),
            "proficient" | "advanced" => Ok(KnowledgeLevel::Proficient),
            "expert" | "master" => Ok(KnowledgeLevel::Expert),
            _ => Err(format!("unknown level: {}", s)),
        }
    }
}

/// LLM response structure for student evaluation
#[derive(Debug, Clone, Deserialize)]
struct LlmEvaluationResponse {
    level: String,
    misconceptions: Vec<String>,
    prerequisites_mastered: Vec<String>,
    prerequisite_gaps: Vec<String>,
    difficulty_target: f64,
    reasoning: String,
}

/// Evaluated student state
#[derive(Debug, Clone)]
pub struct StudentState {
    pub level: KnowledgeLevel,
    pub misconception_topics: Vec<String>,
    pub mastered_prerequisites: Vec<String>,
    pub prerequisite_gaps: Vec<String>,
    pub objectives: Vec<String>,
    pub difficulty_target: f64,
    pub evaluation_reasoning: String,
}

impl StudentState {
    pub fn from_level(level: KnowledgeLevel) -> Self {
        Self {
            level,
            misconception_topics: vec![],
            mastered_prerequisites: vec![],
            prerequisite_gaps: vec![],
            objectives: vec![],
            difficulty_target: level.zpd_difficulty(),
            evaluation_reasoning: String::new(),
        }
    }

    pub fn tutor_prompt(&self, subject: &str) -> String {
        let misconception_guidance = if self.misconception_topics.is_empty() {
            String::new()
        } else {
            format!(
                "Address these misconceptions: {}. ",
                self.misconception_topics.join(", ")
            )
        };

        let gap_guidance = if self.prerequisite_gaps.is_empty() {
            String::new()
        } else {
            format!(
                "Fill prerequisite gaps: {}. ",
                self.prerequisite_gaps.join(", ")
            )
        };

        format!(
            "You are a Socratic tutor teaching {}. \
            The student is at {} level. \
            Use {} language. \
            Target difficulty: {:.0}%. \
            {}\
            {}\
            Ask guiding questions. Do not lecture directly.",
            subject,
            format!("{:?}", self.level).to_lowercase(),
            self.level.complexity(),
            self.difficulty_target * 100.0,
            misconception_guidance,
            gap_guidance,
        )
    }
}

/// LLM-based student evaluator
pub struct StudentEvaluator {
    client: LlmClient,
}

impl StudentEvaluator {
    pub fn new(client: LlmClient) -> Self {
        Self { client }
    }

    pub fn with_endpoint(endpoint: &str, model: &str) -> Self {
        Self {
            client: LlmClient::new(endpoint, "", model),
        }
    }

    /// Evaluate student input using LLM
    pub async fn evaluate(
        &self,
        student_input: &str,
        subject: &str,
        previous_interaction: Option<&str>,
    ) -> Result<StudentState, EvaluatorError> {
        let context = previous_interaction.map(|p| format!("\nPrevious interaction:\n{}", p)).unwrap_or_default();
        
        let prompt = format!(
            "Analyze this student input about {}:{}\n\n\
            Student input: \"{}\"\n\n\
            Evaluate and respond in this exact JSON format:\n\
            {{\n\
              \"level\": \"novice|developing|proficient|expert\",\n\
              \"misconceptions\": [\"topic1\", \"topic2\"],\n\
              \"prerequisites_mastered\": [\"prereq1\", \"prereq2\"],\n\
              \"prerequisite_gaps\": [\"gap1\", \"gap2\"],\n\
              \"difficulty_target\": 0.0-1.0,\n\
              \"reasoning\": \"brief explanation\"\n\
            }}\n\n\
            Assessment criteria:\n\
            - novice: basic questions, confusion markers\n\
            - developing: partial understanding, some technical terms\n\
            - proficient: correct technical usage, reasoning shown\n\
            - expert: sophisticated analysis, connections made",
            subject,
            context,
            student_input.replace('"', "\\\"")
        );

        let request = LlmRequest {
            system_prompt: Some(
                "You are an expert educational assessor. Analyze student input objectively. \
                Return only valid JSON matching the requested format.".to_string()
            ),
            user_prompt: prompt,
            params: LLMParameters {
                temperature: 0.3,
                max_tokens: 500,
                top_p: 0.9,
            },
        };

        let response = self.client.send(request).await
            .map_err(|e| EvaluatorError::LlmError(e.to_string()))?;

        let evaluation: LlmEvaluationResponse = serde_json::from_str(&response.text)
            .map_err(|e| EvaluatorError::ParseError(format!("Failed to parse LLM response: {}. Response: {}", e, response.text.chars().take(200).collect::<String>())))?;

        let level = KnowledgeLevel::from_str(&evaluation.level)
            .map_err(|e| EvaluatorError::InvalidLevel(e))?;

        Ok(StudentState {
            level,
            misconception_topics: evaluation.misconceptions,
            mastered_prerequisites: evaluation.prerequisites_mastered,
            prerequisite_gaps: evaluation.prerequisite_gaps,
            objectives: vec![format!("Progress in {}", subject)],
            difficulty_target: evaluation.difficulty_target.clamp(0.1, 0.95),
            evaluation_reasoning: evaluation.reasoning,
        })
    }

    /// Quick heuristic evaluation (no LLM call) for fallback
    pub fn evaluate_heuristic(&self, student_input: &str, subject: &str) -> StudentState {
        let input_lower = student_input.to_lowercase();

        let technical_indicators = ["therefore", "consequently", "specifically", "in contrast"];
        let has_technical = technical_indicators.iter().any(|&w| input_lower.contains(w));

        let confusion_indicators = ["i don't understand", "confused", "unclear", "wait"];
        let has_confusion = confusion_indicators.iter().any(|&w| input_lower.contains(w));

        let novice_indicators = ["what is", "how do", "what does", "basic"];
        let has_novice = novice_indicators.iter().any(|&w| input_lower.contains(w));

        let level = if has_confusion {
            KnowledgeLevel::Novice
        } else if has_novice {
            KnowledgeLevel::Developing
        } else if has_technical {
            KnowledgeLevel::Proficient
        } else {
            KnowledgeLevel::Developing
        };

        let mut state = StudentState::from_level(level);
        state.evaluation_reasoning = "heuristic fallback".to_string();
        
        if input_lower.contains("always") || input_lower.contains("never") {
            state.misconception_topics.push("overgeneralization".to_string());
        }
        
        if has_novice {
            state.prerequisite_gaps.push(format!("{} fundamentals", subject));
        }

        state
    }
}

impl Default for StudentEvaluator {
    fn default() -> Self {
        Self::with_endpoint("http://localhost:8000/v1/chat/completions", "local-model")
    }
}

#[derive(Debug)]
pub enum EvaluatorError {
    LlmError(String),
    ParseError(String),
    InvalidLevel(String),
}

impl std::fmt::Display for EvaluatorError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EvaluatorError::LlmError(s) => write!(f, "LLM error: {}", s),
            EvaluatorError::ParseError(s) => write!(f, "Parse error: {}", s),
            EvaluatorError::InvalidLevel(s) => write!(f, "Invalid level: {}", s),
        }
    }
}

impl std::error::Error for EvaluatorError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tutor_prompt_includes_misconceptions() {
        let mut state = StudentState::from_level(KnowledgeLevel::Developing);
        state.misconception_topics.push("overgeneralization".to_string());
        let prompt = state.tutor_prompt("math");
        assert!(prompt.contains("overgeneralization"));
    }

    #[test]
    fn heuristic_evaluation_works() {
        let eval = StudentEvaluator::default();
        let state = eval.evaluate_heuristic("what is calculus?", "calculus");
        // "what is" pattern triggers novice indicators -> Developing level
        assert_eq!(state.level, KnowledgeLevel::Developing);
        assert!(state.prerequisite_gaps.contains(&"calculus fundamentals".to_string()));
    }

    #[test]
    fn zpd_increases_with_level() {
        assert!(KnowledgeLevel::Novice.zpd_difficulty() < KnowledgeLevel::Expert.zpd_difficulty());
    }
}