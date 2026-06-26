//! Built-in Skills
//!
//! Standard function implementations.

use crate::skills::registry::{Skill, SkillCategory, Parameter};

/// Register all built-in skills
pub fn register_all(registry: &mut crate::skills::registry::SkillRegistry) {
    register_io_skills(registry);
    register_text_skills(registry);
    register_calculation_skills(registry);
}

/// Register I/O skills
fn register_io_skills(registry: &mut crate::skills::registry::SkillRegistry) {
    let _ = registry.register(Skill {
        name: "echo".to_string(),
        description: "Echo the input message".to_string(),
        category: SkillCategory::IO,
        parameters: vec![
            Parameter {
                name: "message".to_string(),
                param_type: "string".to_string(),
                required: true,
                description: "Message to echo".to_string(),
            },
        ],
        handler: "builtin::echo".to_string(),
    });

    let _ = registry.register(Skill {
        name: "time".to_string(),
        description: "Get current timestamp".to_string(),
        category: SkillCategory::IO,
        parameters: vec![],
        handler: "builtin::time".to_string(),
    });

    let _ = registry.register(Skill {
        name: "read_file".to_string(),
        description: "Read file contents".to_string(),
        category: SkillCategory::IO,
        parameters: vec![
            Parameter {
                name: "path".to_string(),
                param_type: "string".to_string(),
                required: true,
                description: "File path".to_string(),
            },
        ],
        handler: "builtin::read_file".to_string(),
    });
}

/// Register text processing skills
fn register_text_skills(registry: &mut crate::skills::registry::SkillRegistry) {
    let _ = registry.register(Skill {
        name: "word_count".to_string(),
        description: "Count words in text".to_string(),
        category: SkillCategory::Text,
        parameters: vec![
            Parameter {
                name: "text".to_string(),
                param_type: "string".to_string(),
                required: true,
                description: "Text to count".to_string(),
            },
        ],
        handler: "builtin::word_count".to_string(),
    });

    let _ = registry.register(Skill {
        name: "to_lower".to_string(),
        description: "Convert text to lowercase".to_string(),
        category: SkillCategory::Text,
        parameters: vec![
            Parameter {
                name: "text".to_string(),
                param_type: "string".to_string(),
                required: true,
                description: "Text to convert".to_string(),
            },
        ],
        handler: "builtin::to_lower".to_string(),
    });
}

/// Register calculation skills
fn register_calculation_skills(registry: &mut crate::skills::registry::SkillRegistry) {
    let _ = registry.register(Skill {
        name: "add".to_string(),
        description: "Add two numbers".to_string(),
        category: SkillCategory::Calculation,
        parameters: vec![
            Parameter {
                name: "a".to_string(),
                param_type: "number".to_string(),
                required: true,
                description: "First number".to_string(),
            },
            Parameter {
                name: "b".to_string(),
                param_type: "number".to_string(),
                required: true,
                description: "Second number".to_string(),
            },
        ],
        handler: "builtin::add".to_string(),
    });
}

/// Built-in skill handlers
pub struct Builtins;

impl Builtins {
    /// Echo handler implementation
    pub fn echo(message: &str) -> String {
        message.to_string()
    }

    /// Word count implementation
    pub fn word_count(text: &str) -> usize {
        text.split_whitespace().count()
    }

    /// To lowercase implementation
    pub fn to_lower(text: &str) -> String {
        text.to_lowercase()
    }

    /// Add implementation
    pub fn add(a: f64, b: f64) -> f64 {
        a + b
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::skills::registry::SkillRegistry;

    #[test]
    fn registry_has_builtins() {
        let reg = SkillRegistry::default();
        assert!(reg.get("echo").is_some());
        assert!(reg.get("time").is_some());
        assert!(reg.get("word_count").is_some());
    }

    #[test]
    fn echo_works() {
        assert_eq!(Builtins::echo("hello"), "hello");
    }

    #[test]
    fn word_count_works() {
        assert_eq!(Builtins::word_count("hello world"), 2);
        assert_eq!(Builtins::word_count(""), 0);
    }

    #[test]
    fn to_lower_works() {
        assert_eq!(Builtins::to_lower("HELLO"), "hello");
    }

    #[test]
    fn add_works() {
        assert_eq!(Builtins::add(2.0, 3.0), 5.0);
    }
}