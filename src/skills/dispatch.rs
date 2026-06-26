//! Skill Dispatcher
//!
//! Executes skill invocations with parameter validation.

use crate::skills::registry::{Skill, SkillCategory, SkillError};
use std::collections::HashMap;

/// Skill invocation
#[derive(Debug, Clone)]
pub struct Invocation {
    pub skill_name: String,
    pub parameters: HashMap<String, String>,
    pub request_id: String,
}

/// Invocation result
#[derive(Debug, Clone)]
pub struct InvocationResult {
    pub success: bool,
    pub output: String,
    pub error: Option<String>,
    pub execution_time_ms: u64,
}

/// Skill dispatcher
pub struct Dispatcher {
    // In production: reference to registry + async executor
}

impl Dispatcher {
    pub fn new() -> Self {
        Self {}
    }

    /// Dispatch a skill invocation
    ///
    /// In production: lookup skill, validate params, execute
    pub async fn dispatch(&self, invocation: Invocation) -> InvocationResult {
        let start = std::time::Instant::now();

        // Placeholder execution
        let result = match invocation.skill_name.as_str() {
            "echo" => InvocationResult {
                success: true,
                output: invocation
                    .parameters
                    .get("message")
                    .cloned()
                    .unwrap_or_default(),
                error: None,
                execution_time_ms: start.elapsed().as_millis() as u64,
            },
            "time" => InvocationResult {
                success: true,
                output: format!(
                    "{}",
                    std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_secs()
                ),
                error: None,
                execution_time_ms: start.elapsed().as_millis() as u64,
            },
            _ => InvocationResult {
                success: false,
                output: String::new(),
                error: Some(format!("unknown skill: {}", invocation.skill_name)),
                execution_time_ms: start.elapsed().as_millis() as u64,
            },
        };

        result
    }

    /// Batch dispatch multiple invocations
    pub async fn dispatch_batch(&self, invocations: Vec<Invocation>) -> Vec<InvocationResult> {
        let mut results = Vec::with_capacity(invocations.len());
        for inv in invocations {
            results.push(self.dispatch(inv).await);
        }
        results
    }

    /// Validate invocation against skill definition
    pub fn validate(
        &self,
        invocation: &Invocation,
        skill: &Skill,
    ) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();

        for param in &skill.parameters {
            if param.required && !invocation.parameters.contains_key(&param.name) {
                errors.push(format!("missing required parameter: {}", param.name));
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

impl Default for Dispatcher {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn dispatch_echo_skill() {
        let dispatcher = Dispatcher::new();
        let mut params = HashMap::new();
        params.insert("message".to_string(), "hello".to_string());

        let invocation = Invocation {
            skill_name: "echo".to_string(),
            parameters: params,
            request_id: "test".to_string(),
        };

        let result = dispatcher.dispatch(invocation).await;
        assert!(result.success);
        assert_eq!(result.output, "hello");
    }

    #[tokio::test]
    async fn unknown_skill_fails() {
        let dispatcher = Dispatcher::new();
        let invocation = Invocation {
            skill_name: "unknown".to_string(),
            parameters: HashMap::new(),
            request_id: "test".to_string(),
        };

        let result = dispatcher.dispatch(invocation).await;
        assert!(!result.success);
        assert!(result.error.is_some());
    }
}