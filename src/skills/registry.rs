//! Skill Registry
//!
//! Function registration with metadata.

use std::collections::HashMap;

/// Registered skill
#[derive(Debug, Clone)]
pub struct Skill {
    pub name: String,
    pub description: String,
    pub category: SkillCategory,
    pub parameters: Vec<Parameter>,
    pub handler: String, // In production: function pointer
}

/// Skill category for organization
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SkillCategory {
    System,
    IO,
    Data,
    Web,
    Calculation,
    Text,
}

/// Parameter definition
#[derive(Debug, Clone)]
pub struct Parameter {
    pub name: String,
    pub param_type: String,
    pub required: bool,
    pub description: String,
}

/// Registry of available skills
pub struct SkillRegistry {
    skills: HashMap<String, Skill>,
}

impl SkillRegistry {
    pub fn new() -> Self {
        Self {
            skills: HashMap::new(),
        }
    }

    /// Register a skill
    pub fn register(&mut self, skill: Skill) -> Result<(), SkillError> {
        if self.skills.contains_key(&skill.name) {
            return Err(SkillError::Duplicate(skill.name.clone()));
        }
        self.skills.insert(skill.name.clone(), skill);
        Ok(())
    }

    /// Get a skill by name
    pub fn get(&self, name: &str) -> Option<&Skill> {
        self.skills.get(name)
    }

    /// List all skills
    pub fn list(&self) -> Vec<&Skill> {
        self.skills.values().collect()
    }

    /// List skills by category
    pub fn list_by_category(&self, category: SkillCategory) -> Vec<&Skill> {
        self.skills
            .values()
            .filter(|s| s.category == category)
            .collect()
    }

    /// Unregister a skill
    pub fn unregister(&mut self, name: &str) -> Option<Skill> {
        self.skills.remove(name)
    }

    /// Count of registered skills
    pub fn count(&self) -> usize {
        self.skills.len()
    }
}

impl Default for SkillRegistry {
    fn default() -> Self {
        let mut reg = Self::new();
        // Register built-in skills
        crate::skills::builtin::register_all(&mut reg);
        reg
    }
}

/// Registry error
#[derive(Debug)]
pub enum SkillError {
    Duplicate(String),
    NotFound(String),
}

impl std::fmt::Display for SkillError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SkillError::Duplicate(name) => write!(f, "skill '{}' already registered", name),
            SkillError::NotFound(name) => write!(f, "skill '{}' not found", name),
        }
    }
}

impl std::error::Error for SkillError {}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_skill(name: &str) -> Skill {
        Skill {
            name: name.to_string(),
            description: "test".to_string(),
            category: SkillCategory::System,
            parameters: vec![],
            handler: "test_handler".to_string(),
        }
    }

    #[test]
    fn register_and_retrieve() {
        let mut reg = SkillRegistry::new();
        reg.register(make_skill("test_skill")).unwrap();

        let skill = reg.get("test_skill");
        assert!(skill.is_some());
        assert_eq!(skill.unwrap().name, "test_skill");
    }

    #[test]
    fn duplicate_registration_fails() {
        let mut reg = SkillRegistry::new();
        reg.register(make_skill("skill")).unwrap();
        let result = reg.register(make_skill("skill"));
        assert!(result.is_err());
    }

    #[test]
    fn category_filtering_works() {
        let mut reg = SkillRegistry::new();
        let mut s1 = make_skill("s1");
        s1.category = SkillCategory::IO;
        reg.register(s1).unwrap();

        let io_skills = reg.list_by_category(SkillCategory::IO);
        assert_eq!(io_skills.len(), 1);

        let calc_skills = reg.list_by_category(SkillCategory::Calculation);
        assert!(calc_skills.is_empty());
    }
}