//! Built-in Skills
//!
//! Standard function implementations with actual execution handlers.

use crate::skills::registry::{Skill, SkillCategory, Parameter};
use std::collections::HashMap;

/// Register all built-in skills
pub fn register_all(registry: &mut crate::skills::registry::SkillRegistry) {
    register_io_skills(registry);
    register_text_skills(registry);
    register_calculation_skills(registry);
    register_system_skills(registry);
}

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

    let _ = registry.register(Skill {
        name: "write_file".to_string(),
        description: "Write content to file".to_string(),
        category: SkillCategory::IO,
        parameters: vec![
            Parameter {
                name: "path".to_string(),
                param_type: "string".to_string(),
                required: true,
                description: "File path".to_string(),
            },
            Parameter {
                name: "content".to_string(),
                param_type: "string".to_string(),
                required: true,
                description: "Content to write".to_string(),
            },
        ],
        handler: "builtin::write_file".to_string(),
    });
}

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

    let _ = registry.register(Skill {
        name: "to_upper".to_string(),
        description: "Convert text to uppercase".to_string(),
        category: SkillCategory::Text,
        parameters: vec![
            Parameter {
                name: "text".to_string(),
                param_type: "string".to_string(),
                required: true,
                description: "Text to convert".to_string(),
            },
        ],
        handler: "builtin::to_upper".to_string(),
    });

    let _ = registry.register(Skill {
        name: "extract_ngrams".to_string(),
        description: "Extract n-grams from text".to_string(),
        category: SkillCategory::Text,
        parameters: vec![
            Parameter {
                name: "text".to_string(),
                param_type: "string".to_string(),
                required: true,
                description: "Text to process".to_string(),
            },
            Parameter {
                name: "n".to_string(),
                param_type: "number".to_string(),
                required: false,
                description: "N-gram size (default 3)".to_string(),
            },
        ],
        handler: "builtin::extract_ngrams".to_string(),
    });
}

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

    let _ = registry.register(Skill {
        name: "multiply".to_string(),
        description: "Multiply two numbers".to_string(),
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
        handler: "builtin::multiply".to_string(),
    });

    let _ = registry.register(Skill {
        name: "cosine_similarity".to_string(),
        description: "Calculate cosine similarity between two vectors".to_string(),
        category: SkillCategory::Calculation,
        parameters: vec![
            Parameter {
                name: "vec1".to_string(),
                param_type: "array".to_string(),
                required: true,
                description: "First vector (comma-separated)".to_string(),
            },
            Parameter {
                name: "vec2".to_string(),
                param_type: "array".to_string(),
                required: true,
                description: "Second vector (comma-separated)".to_string(),
            },
        ],
        handler: "builtin::cosine_similarity".to_string(),
    });
}

fn register_system_skills(registry: &mut crate::skills::registry::SkillRegistry) {
    let _ = registry.register(Skill {
        name: "system_info".to_string(),
        description: "Get system information".to_string(),
        category: SkillCategory::System,
        parameters: vec![],
        handler: "builtin::system_info".to_string(),
    });

    let _ = registry.register(Skill {
        name: "hash_string".to_string(),
        description: "Compute hash of string".to_string(),
        category: SkillCategory::System,
        parameters: vec![
            Parameter {
                name: "input".to_string(),
                param_type: "string".to_string(),
                required: true,
                description: "String to hash".to_string(),
            },
        ],
        handler: "builtin::hash_string".to_string(),
    });
}

/// Skill execution result
#[derive(Debug, Clone)]
pub enum SkillResult {
    Success(String),
    Error(String),
}

/// Execute a built-in skill
pub fn execute(skill_name: &str, params: &HashMap<String, String>) -> SkillResult {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    match skill_name {
        "echo" => {
            params.get("message")
                .map(|m| SkillResult::Success(m.clone()))
                .unwrap_or_else(|| SkillResult::Error("Missing 'message' parameter".to_string()))
        }

        "time" => {
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();
            SkillResult::Success(now.to_string())
        }

        "read_file" => {
            match params.get("path") {
                Some(path) => {
                    match std::fs::read_to_string(path) {
                        Ok(content) => SkillResult::Success(content),
                        Err(e) => SkillResult::Error(format!("Failed to read file: {}", e)),
                    }
                }
                None => SkillResult::Error("Missing 'path' parameter".to_string()),
            }
        }

        "write_file" => {
            let path = params.get("path");
            let content = params.get("content");
            
            match (path, content) {
                (Some(p), Some(c)) => {
                    match std::fs::write(p, c) {
                        Ok(_) => SkillResult::Success(format!("Wrote {} bytes to {}", c.len(), p)),
                        Err(e) => SkillResult::Error(format!("Failed to write file: {}", e)),
                    }
                }
                _ => SkillResult::Error("Missing 'path' or 'content' parameter".to_string()),
            }
        }

        "word_count" => {
            params.get("text")
                .map(|t| SkillResult::Success(t.split_whitespace().count().to_string()))
                .unwrap_or_else(|| SkillResult::Error("Missing 'text' parameter".to_string()))
        }

        "to_lower" => {
            params.get("text")
                .map(|t| SkillResult::Success(t.to_lowercase()))
                .unwrap_or_else(|| SkillResult::Error("Missing 'text' parameter".to_string()))
        }

        "to_upper" => {
            params.get("text")
                .map(|t| SkillResult::Success(t.to_uppercase()))
                .unwrap_or_else(|| SkillResult::Error("Missing 'text' parameter".to_string()))
        }

        "extract_ngrams" => {
            match params.get("text") {
                Some(text) => {
                    let n: usize = params.get("n")
                        .and_then(|n| n.parse().ok())
                        .unwrap_or(3);
                    
                    let chars: Vec<char> = text.chars().collect();
                    if chars.len() < n {
                        return SkillResult::Success(text.to_string());
                    }

                    let ngrams: Vec<String> = (0..=chars.len() - n)
                        .map(|i| chars[i..i + n].iter().collect())
                        .collect();

                    SkillResult::Success(ngrams.join(", "))
                }
                None => SkillResult::Error("Missing 'text' parameter".to_string()),
            }
        }

        "add" => {
            let a = params.get("a").and_then(|v| v.parse::<f64>().ok());
            let b = params.get("b").and_then(|v| v.parse::<f64>().ok());
            
            match (a, b) {
                (Some(x), Some(y)) => SkillResult::Success((x + y).to_string()),
                _ => SkillResult::Error("Invalid or missing 'a' or 'b' parameters".to_string()),
            }
        }

        "multiply" => {
            let a = params.get("a").and_then(|v| v.parse::<f64>().ok());
            let b = params.get("b").and_then(|v| v.parse::<f64>().ok());
            
            match (a, b) {
                (Some(x), Some(y)) => SkillResult::Success((x * y).to_string()),
                _ => SkillResult::Error("Invalid or missing 'a' or 'b' parameters".to_string()),
            }
        }

        "cosine_similarity" => {
            let vec1 = params.get("vec1")
                .map(|v| parse_vector(v))
                .unwrap_or_default();
            let vec2 = params.get("vec2")
                .map(|v| parse_vector(v))
                .unwrap_or_default();

            if vec1.is_empty() || vec2.is_empty() {
                return SkillResult::Error("Invalid or missing vectors".to_string());
            }

            if vec1.len() != vec2.len() {
                return SkillResult::Error("Vectors must have same dimension".to_string());
            }

            let similarity = compute_cosine_similarity(&vec1, &vec2);
            SkillResult::Success(similarity.to_string())
        }

        "system_info" => {
            let info = format!(
                "OS: {}\nArch: {}\nCores: {}",
                std::env::consts::OS,
                std::env::consts::ARCH,
                std::thread::available_parallelism()
                    .map(|n| n.get())
                    .unwrap_or(1)
            );
            SkillResult::Success(info)
        }

        "hash_string" => {
            match params.get("input") {
                Some(input) => {
                    let mut hasher = DefaultHasher::new();
                    input.hash(&mut hasher);
                    let hash = hasher.finish();
                    SkillResult::Success(hash.to_string())
                }
                None => SkillResult::Error("Missing 'input' parameter".to_string()),
            }
        }

        _ => SkillResult::Error(format!("Unknown skill: {}", skill_name)),
    }
}

/// Parse a vector from comma-separated string
fn parse_vector(s: &str) -> Vec<f32> {
    s.split(',')
        .filter_map(|v| v.trim().parse::<f32>().ok())
        .collect()
}

/// Compute cosine similarity between two vectors
fn compute_cosine_similarity(a: &[f32], b: &[f32]) -> f64 {
    if a.len() != b.len() || a.is_empty() {
        return 0.0;
    }

    let dot: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();

    if norm_a == 0.0 || norm_b == 0.0 {
        return 0.0;
    }

    (dot / (norm_a * norm_b)) as f64
}

/// Built-in skill handlers (legacy structure)
pub struct Builtins;

impl Builtins {
    pub fn echo(message: &str) -> String {
        message.to_string()
    }

    pub fn word_count(text: &str) -> usize {
        text.split_whitespace().count()
    }

    pub fn to_lower(text: &str) -> String {
        text.to_lowercase()
    }

    pub fn to_upper(text: &str) -> String {
        text.to_uppercase()
    }

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
        assert!(reg.get("system_info").is_some());
    }

    #[test]
    fn execute_echo() {
        let mut params = HashMap::new();
        params.insert("message".to_string(), "hello".to_string());
        
        match execute("echo", &params) {
            SkillResult::Success(result) => assert_eq!(result, "hello"),
            _ => panic!("Echo failed"),
        }
    }

    #[test]
    fn execute_word_count() {
        let mut params = HashMap::new();
        params.insert("text".to_string(), "hello world".to_string());
        
        match execute("word_count", &params) {
            SkillResult::Success(result) => assert_eq!(result, "2"),
            _ => panic!("Word count failed"),
        }
    }

    #[test]
    fn execute_add() {
        let mut params = HashMap::new();
        params.insert("a".to_string(), "2.5".to_string());
        params.insert("b".to_string(), "3.5".to_string());
        
        match execute("add", &params) {
            SkillResult::Success(result) => assert_eq!(result, "6"),
            _ => panic!("Add failed"),
        }
    }

    #[test]
    fn execute_unknown_skill() {
        let params = HashMap::new();
        
        match execute("unknown_skill", &params) {
            SkillResult::Error(_) => (), // Expected
            _ => panic!("Should error on unknown skill"),
        }
    }

    #[test]
    fn cosine_similarity_computed() {
        let mut params = HashMap::new();
        params.insert("vec1".to_string(), "1.0, 0.0, 0.0".to_string());
        params.insert("vec2".to_string(), "1.0, 0.0, 0.0".to_string());
        
        match execute("cosine_similarity", &params) {
            SkillResult::Success(result) => {
                let sim: f64 = result.parse().unwrap();
                assert!((sim - 1.0).abs() < 0.001);
            }
            _ => panic!("Cosine similarity failed"),
        }
    }
}