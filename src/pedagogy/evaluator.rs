//! Student State Evaluator
//!
//! Evaluates student knowledge state in a single LLM call.
//! Simplified from 4-agent system (Diagnoser, Planner, Affective, Tutor)
//! to 1-agent state assessment.

/// Student knowledge level
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KnowledgeLevel {
    /// No prior exposure
    Novice,
    /// Some familiarity, gaps remain
    Developing,
    /// Solid understanding
    Proficient,
    /// Mastery
    Expert,
}

impl KnowledgeLevel {
    /// Target zone difficulty for this level
    pub fn zpd_difficulty(&self) -> f64 {
        match self {
            KnowledgeLevel::Novice => 0.3,
            KnowledgeLevel::Developing => 0.5,
            KnowledgeLevel::Proficient => 0.7,
            KnowledgeLevel::Expert => 0.9,
        }
    }

    /// Student-appropriate language complexity
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

/// Evaluated student state
#[derive(Debug, Clone)]
pub struct StudentState {
    /// Estimated knowledge level
    pub level: KnowledgeLevel,
    /// Specific topics where student shows confusion
    pub misconception_topics: Vec<String>,
    /// Prerequisites the student has mastered
    pub mastered_prerequisites: Vec<String>,
    /// Prerequisite gaps to address
    pub prerequisite_gaps: Vec<String>,
    /// Suggested learning objectives
    pub objectives: Vec<String>,
    /// Appropriate difficulty level (0-1)
    pub difficulty_target: f64,
}

impl StudentState {
    /// Create state from a simple knowledge level estimate
    pub fn from_level(level: KnowledgeLevel) -> Self {
        Self {
            level,
            misconception_topics: vec![],
            mastered_prerequisites: vec![],
            prerequisite_gaps: vec![],
            objectives: vec![],
            difficulty_target: level.zpd_difficulty(),
        }
    }

    /// Generate system prompt for tutor based on this state
    pub fn tutor_prompt(&self, subject: &str) -> String {
        format!(
            "You are a Socratic tutor teaching {}. \
            The student is at {} level. \
            Use {} language. \
            Target difficulty: {:.0}%. \
            Ask guiding questions, do not lecture directly.",
            subject,
            format!("{:?}", self.level).to_lowercase(),
            self.level.complexity(),
            self.difficulty_target * 100.0,
        )
    }
}

/// Evaluates student input to determine state
pub struct StudentEvaluator;

impl StudentEvaluator {
    /// Create new evaluator
    pub fn new() -> Self {
        Self
    }

    /// Evaluate student input via LLM
    ///
    /// # Arguments
    /// * `student_input` - The student's response or initial message
    /// * `subject` - Topic being learned
    /// * `previous_interaction` - Optional previous interaction for context
    ///
    /// Returns a structured StudentState
    pub fn evaluate(&self, student_input: &str, subject: &str, _previous_interaction: Option<&str>) -> StudentState {
        // Simple heuristic evaluation (in production, use LLM)
        let input_lower = student_input.to_lowercase();

        // Check for confident technical language
        let technical_indicators = ["therefore", "consequently", "specifically", "in contrast"];
        let has_technical = technical_indicators.iter().any(|&w| input_lower.contains(w));

        // Check for confusion markers
        let confusion_indicators = ["i don't understand", "confused", "unclear", "wait"];
        let has_confusion = confusion_indicators.iter().any(|&w| input_lower.contains(w));

        // Check for novice markers
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

        StudentState {
            level,
            misconception_topics: self.extract_misconceptions(&input_lower),
            mastered_prerequisites: vec![],
            prerequisite_gaps: if has_novice {
                vec![format!("{} fundamentals", subject)]
            } else {
                vec![]
            },
            objectives: vec![format!("Master {}", subject)],
            difficulty_target: level.zpd_difficulty(),
        }
    }

    /// Extract potential misconception topics from input
    fn extract_misconceptions(&self, input: &str) -> Vec<String> {
        let mut topics = Vec::new();

        if input.contains("always") || input.contains("never") {
            topics.push("overgeneralization".to_string());
        }
        if input.contains("because") && !input.contains("because of") {
            // Check for causal reasoning issues
            if input.matches("because").count() > 1 {
                topics.push("causal chain complexity".to_string());
            }
        }

        topics
    }

    /// Generate the LLM prompt for evaluation
    pub fn evaluation_prompt(&self, student_input: &str, subject: &str) -> String {
        format!(
            "Evaluate the following student input about {}:\n\n\
            Input: {}\n\n\
            Assess:\n\
            1. Knowledge level (novice/developing/proficient/expert)\n\
            2. Specific misconceptions or gaps\n\
            3. Prerequisite mastery status\n\
            4. Appropriate difficulty target (0-1)\n\n\
            Format: JSON with fields: level, misconceptions, prerequisites_mastered, prerequisite_gaps, difficulty_target",
            subject,
            student_input,
        )
    }
}

impl Default for StudentEvaluator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn novice_detected_from_basic_question() {
        let eval = StudentEvaluator::new();
        let state = eval.evaluate("what is a neural network?", "machine learning", None);
        assert_eq!(state.level, KnowledgeLevel::Novice);
    }

    #[test]
    fn confusion_detected() {
        let eval = StudentEvaluator::new();
        let state = eval.evaluate("I'm confused about this", "math", None);
        assert_eq!(state.level, KnowledgeLevel::Novice);
        assert!(state.difficulty_target < 0.4);
    }

    #[test]
    fn technical_language_suggests_proficiency() {
        let eval = StudentEvaluator::new();
        let state = eval.evaluate(
            "The derivative represents the instantaneous rate of change. Specifically, it measures the slope of the tangent line.",
            "calculus",
            None,
        );
        assert!(matches!(state.level, KnowledgeLevel::Proficient | KnowledgeLevel::Expert));
    }

    #[test]
    fn misconception_overgeneralization_detected() {
        let eval = StudentEvaluator::new();
        let state = eval.evaluate("Neural networks always work better", "machine learning", None);
        assert!(state.misconception_topics.contains(&"overgeneralization".to_string()));
    }

    #[test]
    fn zpd_increases_with_level() {
        assert!(KnowledgeLevel::Novice.zpd_difficulty() < KnowledgeLevel::Expert.zpd_difficulty());
    }

    #[test]
    fn tutor_prompt_includes_level() {
        let state = StudentState::from_level(KnowledgeLevel::Developing);
        let prompt = state.tutor_prompt("geometry");
        assert!(prompt.contains("developing"));
        assert!(prompt.contains("moderate"));
    }
}