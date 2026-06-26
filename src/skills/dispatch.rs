//! Skill Dispatcher
//!
//! Executes skill invocations with parameter validation and real execution.

use crate::skills::builtin::{execute, SkillResult};
use crate::skills::registry::{Skill, SkillRegistry};
use std::collections::HashMap;

/// Skill invocation
#[derive(Debug, Clone)]
pub struct Invocation {
    pub skill_name: String,
    pub parameters: HashMap<String, String>,
    pub request_id: String,
}

/// Invocation result with detailed output
#[derive(Debug, Clone)]
pub struct InvocationResult {
    pub success: bool,
    pub output: String,
    pub error: Option<String>,
    pub execution_time_ms: u64,
    pub skill_found: bool,
}

/// Skill dispatcher with real execution
pub struct Dispatcher {
    registry: SkillRegistry,
}

impl Dispatcher {
    /// Create new dispatcher with default registry
    pub fn new() -> Self {
        Self {
            registry: SkillRegistry::default(),
        }
    }

    /// Create with custom registry
    pub fn with_registry(registry: SkillRegistry) -> Self {
        Self { registry }
    }

    /// Dispatch a skill invocation with real execution
    pub async fn dispatch(&self, invocation: Invocation) -> InvocationResult {
        let start = std::time::Instant::now();

        // Step 1: Look up the skill
        let skill = match self.registry.get(&invocation.skill_name) {
            Some(s) => s,
            None => {
                return InvocationResult {
                    success: false,
                    output: String::new(),
                    error: Some(format!("Skill '{}' not found", invocation.skill_name)),
                    execution_time_ms: start.elapsed().as_millis() as u64,
                    skill_found: false,
                };
            }
        };

        // Step 2: Validate parameters
        match self.validate(&invocation, skill) {
            Ok(_) => {}
            Err(errors) => {
                return InvocationResult {
                    success: false,
                    output: String::new(),
                    error: Some(format!("Validation failed: {}", errors.join(", "))),
                    execution_time_ms: start.elapsed().as_millis() as u64,
                    skill_found: true,
                };
            }
        }

        // Step 3: Execute the skill
        let result = execute(&invocation.skill_name, &invocation.parameters);
        let execution_time = start.elapsed();

        match result {
            SkillResult::Success(output) => InvocationResult {
                success: true,
                output,
                error: None,
                execution_time_ms: execution_time.as_millis() as u64,
                skill_found: true,
            },
            SkillResult::Error(error) => InvocationResult {
                success: false,
                output: String::new(),
                error: Some(error),
                execution_time_ms: execution_time.as_millis() as u64,
                skill_found: true,
            },
        }
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

            // Validate parameter types
            if let Some(value) = invocation.parameters.get(&param.name) {
                match param.param_type.as_str() {
                    "number" => {
                        if value.parse::<f64>().is_err() {
                            errors.push(format!(
                                "parameter '{}' must be a number, got: {}",
                                param.name, value
                            ));
                        }
                    }
                    "array" => {
                        // Basic validation - check if it looks like a comma-separated list
                        if value.is_empty() {
                            errors.push(format!("parameter '{}' must not be empty", param.name));
                        }
                    }
                    "string" => {
                        // Strings are always valid
                    }
                    _ => {
                        // Unknown type - allow it
                    }
                }
            }
        }

        // Check for unknown parameters
        for param_name in invocation.parameters.keys() {
            if !skill.parameters.iter().any(|p| &p.name == param_name) {
                errors.push(format!("unknown parameter: {}", param_name));
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    /// Get list of available skills
    pub fn available_skills(&self) -> Vec<&str> {
        self.registry.list().iter().map(|s| s.name.as_str()).collect()
    }

    /// Check if a skill exists
    pub fn has_skill(&self, name: &str) -> bool {
        self.registry.get(name).is_some()
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
        assert!(result.skill_found);
        assert_eq!(result.output, "hello");
    }

    #[tokio::test]
    async fn dispatch_file_read() {
        let dispatcher = Dispatcher::new();
        let mut params = HashMap::new();
        params.insert("path".to_string(), "/tmp/nonexistent".to_string());

        let invocation = Invocation {
            skill_name: "read_file".to_string(),
            parameters: params,
            request_id: "test".to_string(),
        };

        let result = dispatcher.dispatch(invocation).await;
        assert!(!result.success); // File doesn't exist
        assert!(result.skill_found);
        assert!(result.error.is_some());
    }

    #[tokio::test]
    async fn unknown_skill_fails() {
        let dispatcher = Dispatcher::new();
        let invocation = Invocation {
            skill_name: "unknown_skill".to_string(),
            parameters: HashMap::new(),
            request_id: "test".to_string(),
        };

        let result = dispatcher.dispatch(invocation).await;
        assert!(!result.success);
        assert!(!result.skill_found);
        assert!(result.error.is_some());
    }

    #[tokio::test]
    async fn validation_catches_missing_param() {
        let dispatcher = Dispatcher::new();
        let invocation = Invocation {
            skill_name: "echo".to_string(),
            parameters: HashMap::new(), // Missing required 'message'
            request_id: "test".to_string(),
        };

        let result = dispatcher.dispatch(invocation).await;
        assert!(!result.success);
        assert!(result.error.is_some());
        assert!(result.error.as_ref().unwrap().contains("missing required"));
    }

    #[tokio::test]
    async fn dispatch_calculation() {
        let dispatcher = Dispatcher::new();
        let mut params = HashMap::new();
        params.insert("a".to_string(), "10".to_string());
        params.insert("b".to_string(), "20".to_string());

        let invocation = Invocation {
            skill_name: "add".to_string(),
            parameters: params,
            request_id: "test".to_string(),
        };

        let result = dispatcher.dispatch(invocation).await;
        assert!(result.success);
        assert_eq!(result.output, "30");
    }
}