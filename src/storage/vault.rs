//! Vault Storage
//!
//! Structured SQLite storage for facts, edges, and relationships.
//! Replaces the theatrical 4-layer memory with direct storage.

use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

/// Unique identifier for facts
pub type FactId = String;

/// A stored fact
#[derive(Debug, Clone)]
pub struct Fact {
    pub id: FactId,
    pub content: String,
    pub embedding: Option<Vec<f32>>,
    pub source: String,
    pub confidence: f64,
    pub created_at: u64,
    pub updated_at: u64,
    pub metadata: HashMap<String, String>,
}

/// Relationship between facts
#[derive(Debug, Clone)]
pub struct Edge {
    pub id: String,
    pub source_id: FactId,
    pub target_id: FactId,
    pub relation_type: String,
    pub confidence: f64,
    pub created_at: u64,
}

/// Query for facts
#[derive(Debug, Clone)]
pub struct FactQuery {
    pub content_contains: Option<String>,
    pub source_equals: Option<String>,
    pub min_confidence: f64,
    pub limit: usize,
}

impl Default for FactQuery {
    fn default() -> Self {
        Self {
            content_contains: None,
            source_equals: None,
            min_confidence: 0.0,
            limit: 100,
        }
    }
}

/// Vault storage interface
///
/// This is a trait for database abstraction. Concrete implementations
/// would use SQLite, PostgreSQL, or other backends.
pub trait Vault: Send + Sync {
    type Error: std::error::Error;

    /// Store a new fact
    fn store_fact(&mut self, fact: Fact) -> Result<FactId, Self::Error>;

    /// Retrieve a fact by ID
    fn get_fact(&self, id: &FactId) -> Result<Option<Fact>, Self::Error>;

    /// Update an existing fact
    fn update_fact(&mut self, fact: Fact) -> Result<(), Self::Error>;

    /// Delete a fact
    fn delete_fact(&mut self, id: &FactId) -> Result<bool, Self::Error>;

    /// Query facts
    fn query_facts(&self, query: FactQuery) -> Result<Vec<Fact>, Self::Error>;

    /// Create an edge between facts
    fn create_edge(&mut self, edge: Edge) -> Result<String, Self::Error>;

    /// Get edges from a source fact
    fn get_edges_from(&self, source_id: &FactId) -> Result<Vec<Edge>, Self::Error>;

    /// Get edges to a target fact
    fn get_edges_to(&self, target_id: &FactId) -> Result<Vec<Edge>, Self::Error>;

    /// Delete an edge
    fn delete_edge(&mut self, id: &str) -> Result<bool, Self::Error>;

    /// Get fact count
    fn fact_count(&self) -> Result<usize, Self::Error>;

    /// Get edge count
    fn edge_count(&self) -> Result<usize, Self::Error>;
}

/// In-memory vault for testing
pub struct InMemoryVault {
    facts: HashMap<FactId, Fact>,
    edges: HashMap<String, Edge>,
}

impl InMemoryVault {
    pub fn new() -> Self {
        Self {
            facts: HashMap::new(),
            edges: HashMap::new(),
        }
    }
}

#[derive(Debug)]
pub struct VaultError(String);

impl std::fmt::Display for VaultError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for VaultError {}

impl Vault for InMemoryVault {
    type Error = VaultError;

    fn store_fact(&mut self, fact: Fact) -> Result<FactId, Self::Error> {
        let id = fact.id.clone();
        self.facts.insert(id.clone(), fact);
        Ok(id)
    }

    fn get_fact(&self, id: &FactId) -> Result<Option<Fact>, Self::Error> {
        Ok(self.facts.get(id).cloned())
    }

    fn update_fact(&mut self, fact: Fact) -> Result<(), Self::Error> {
        if !self.facts.contains_key(&fact.id) {
            return Err(VaultError(format!("fact not found: {}", fact.id)));
        }
        self.facts.insert(fact.id.clone(), fact);
        Ok(())
    }

    fn delete_fact(&mut self, id: &FactId) -> Result<bool, Self::Error> {
        Ok(self.facts.remove(id).is_some())
    }

    fn query_facts(&self, query: FactQuery) -> Result<Vec<Fact>, Self::Error> {
        let mut results: Vec<_> = self
            .facts
            .values()
            .filter(|f| f.confidence >= query.min_confidence)
            .filter(|f| {
                query.content_contains.as_ref().map_or(true, |pat| {
                    f.content.to_lowercase().contains(&pat.to_lowercase())
                })
            })
            .filter(|f| {
                query.source_equals.as_ref().map_or(true, |src| f.source == *src)
            })
            .cloned()
            .collect();

        results.truncate(query.limit);
        Ok(results)
    }

    fn create_edge(&mut self, edge: Edge) -> Result<String, Self::Error> {
        let id = edge.id.clone();
        self.edges.insert(id.clone(), edge);
        Ok(id)
    }

    fn get_edges_from(&self, source_id: &FactId) -> Result<Vec<Edge>, Self::Error> {
        Ok(self
            .edges
            .values()
            .filter(|e| e.source_id == *source_id)
            .cloned()
            .collect())
    }

    fn get_edges_to(&self, target_id: &FactId) -> Result<Vec<Edge>, Self::Error> {
        Ok(self
            .edges
            .values()
            .filter(|e| e.target_id == *target_id)
            .cloned()
            .collect())
    }

    fn delete_edge(&mut self, id: &str) -> Result<bool, Self::Error> {
        Ok(self.edges.remove(id).is_some())
    }

    fn fact_count(&self) -> Result<usize, Self::Error> {
        Ok(self.facts.len())
    }

    fn edge_count(&self) -> Result<usize, Self::Error> {
        Ok(self.edges.len())
    }
}

/// Create a new fact with current timestamp
pub fn create_fact(content: impl Into<String>, source: impl Into<String>) -> Fact {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    let content = content.into();
    let id = format!("fact_{}_{}", now, hash_string(&content));

    Fact {
        id,
        content,
        embedding: None,
        source: source.into(),
        confidence: 1.0,
        created_at: now,
        updated_at: now,
        metadata: HashMap::new(),
    }
}

fn hash_string(s: &str) -> u64 {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut hasher = DefaultHasher::new();
    s.hash(&mut hasher);
    hasher.finish()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn store_and_retrieve_fact() {
        let mut vault = InMemoryVault::new();
        let fact = create_fact("test content", "test");
        let id = fact.id.clone();

        vault.store_fact(fact).unwrap();
        let retrieved = vault.get_fact(&id).unwrap();

        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().content, "test content");
    }

    #[test]
    fn query_by_content() {
        let mut vault = InMemoryVault::new();
        vault.store_fact(create_fact("hello world", "source1")).unwrap();
        vault.store_fact(create_fact("goodbye world", "source2")).unwrap();

        let query = FactQuery {
            content_contains: Some("hello".to_string()),
            ..Default::default()
        };
        let results = vault.query_facts(query).unwrap();

        assert_eq!(results.len(), 1);
        assert!(results[0].content.contains("hello"));
    }

    #[test]
    fn confidence_filter_works() {
        let mut vault = InMemoryVault::new();
        let mut fact = create_fact("low confidence", "test");
        fact.confidence = 0.3;
        vault.store_fact(fact).unwrap();

        let mut fact2 = create_fact("high confidence", "test");
        fact2.confidence = 0.9;
        vault.store_fact(fact2).unwrap();

        let query = FactQuery {
            min_confidence: 0.5,
            ..Default::default()
        };
        let results = vault.query_facts(query).unwrap();

        assert_eq!(results.len(), 1);
        assert!(results[0].content.contains("high"));
    }
}