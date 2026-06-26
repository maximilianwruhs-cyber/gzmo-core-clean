//! Vault Storage
//!
//! Structured SQLite storage for facts, edges, and relationships.
//! Replaces the theatrical 4-layer memory with direct storage.

use rusqlite::OptionalExtension;
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

/// SQLite-backed vault storage (thread-safe)
pub struct SqliteVault {
    conn: std::sync::Mutex<rusqlite::Connection>,
}

impl SqliteVault {
    /// Open or create SQLite database at given path
    pub fn open(path: &str) -> Result<Self, VaultError> {
        let conn = rusqlite::Connection::open(path)
            .map_err(|e| VaultError(format!("Failed to open database: {}", e)))?;
        
        let mut vault = Self { 
            conn: std::sync::Mutex::new(conn) 
        };
        vault.init_schema()?;
        Ok(vault)
    }

    /// Initialize database schema
    fn init_schema(&mut self) -> Result<(), VaultError> {
        let conn = self.conn.lock().unwrap();
        conn.execute_batch(
            r#"
            CREATE TABLE IF NOT EXISTS facts (
                id TEXT PRIMARY KEY,
                content TEXT NOT NULL,
                source TEXT NOT NULL,
                confidence REAL NOT NULL,
                created_at INTEGER NOT NULL,
                updated_at INTEGER NOT NULL,
                metadata TEXT
            );

            CREATE INDEX IF NOT EXISTS idx_facts_source ON facts(source);
            CREATE INDEX IF NOT EXISTS idx_facts_confidence ON facts(confidence);

            CREATE TABLE IF NOT EXISTS edges (
                id TEXT PRIMARY KEY,
                source_id TEXT NOT NULL,
                target_id TEXT NOT NULL,
                relation_type TEXT NOT NULL,
                confidence REAL NOT NULL,
                created_at INTEGER NOT NULL,
                FOREIGN KEY (source_id) REFERENCES facts(id),
                FOREIGN KEY (target_id) REFERENCES facts(id)
            );

            CREATE INDEX IF NOT EXISTS idx_edges_source ON edges(source_id);
            CREATE INDEX IF NOT EXISTS idx_edges_target ON edges(target_id);
            "#
        ).map_err(|e| VaultError(format!("Failed to init schema: {}", e)))?;
        
        Ok(())
    }

    /// Serialize metadata HashMap to JSON string
    fn metadata_to_json(metadata: &HashMap<String, String>) -> String {
        serde_json::to_string(metadata).unwrap_or_else(|_| "{}".to_string())
    }

    /// Deserialize metadata JSON string to HashMap
    fn json_to_metadata(json: &str) -> HashMap<String, String> {
        serde_json::from_str(json).unwrap_or_default()
    }

    /// Convert database row to Fact
    fn row_to_fact(row: &rusqlite::Row) -> Result<Fact, rusqlite::Error> {
        Ok(Fact {
            id: row.get(0)?,
            content: row.get(1)?,
            source: row.get(2)?,
            confidence: row.get(3)?,
            created_at: row.get(4)?,
            updated_at: row.get(5)?,
            embedding: None, // Not stored in SQLite
            metadata: Self::json_to_metadata(&row.get::<_, String>(6).unwrap_or_default()),
        })
    }

    /// Convert database row to Edge
    fn row_to_edge(row: &rusqlite::Row) -> Result<Edge, rusqlite::Error> {
        Ok(Edge {
            id: row.get(0)?,
            source_id: row.get(1)?,
            target_id: row.get(2)?,
            relation_type: row.get(3)?,
            confidence: row.get(4)?,
            created_at: row.get(5)?,
        })
    }
}

impl Vault for SqliteVault {
    type Error = VaultError;

    fn store_fact(&mut self, fact: Fact) -> Result<FactId, Self::Error> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO facts (id, content, source, confidence, created_at, updated_at, metadata)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
             ON CONFLICT(id) DO UPDATE SET
                content=excluded.content,
                source=excluded.source,
                confidence=excluded.confidence,
                updated_at=excluded.updated_at,
                metadata=excluded.metadata",
            rusqlite::params![
                &fact.id,
                &fact.content,
                &fact.source,
                fact.confidence,
                fact.created_at as i64,
                fact.updated_at as i64,
                Self::metadata_to_json(&fact.metadata)
            ],
        ).map_err(|e| VaultError(format!("Failed to store fact: {}", e)))?;
        
        Ok(fact.id)
    }

    fn get_fact(&self, id: &FactId) -> Result<Option<Fact>, Self::Error> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, content, source, confidence, created_at, updated_at, metadata
             FROM facts WHERE id = ?1"
        ).map_err(|e| VaultError(format!("Failed to prepare query: {}", e)))?;

        let fact = stmt.query_row(rusqlite::params![id], Self::row_to_fact)
            .optional()
            .map_err(|e| VaultError(format!("Failed to query fact: {}", e)))?;
        
        Ok(fact)
    }

    fn update_fact(&mut self, fact: Fact) -> Result<(), Self::Error> {
        let conn = self.conn.lock().unwrap();
        let updated = conn.execute(
            "UPDATE facts SET
                content = ?2,
                source = ?3,
                confidence = ?4,
                updated_at = ?5,
                metadata = ?6
             WHERE id = ?1",
            rusqlite::params![
                &fact.id,
                &fact.content,
                &fact.source,
                fact.confidence,
                fact.updated_at as i64,
                Self::metadata_to_json(&fact.metadata)
            ],
        ).map_err(|e| VaultError(format!("Failed to update fact: {}", e)))?;

        if updated == 0 {
            return Err(VaultError(format!("Fact not found: {}", fact.id)));
        }
        
        Ok(())
    }

    fn delete_fact(&mut self, id: &FactId) -> Result<bool, Self::Error> {
        let conn = self.conn.lock().unwrap();
        // Delete associated edges first
        conn.execute(
            "DELETE FROM edges WHERE source_id = ?1 OR target_id = ?1",
            rusqlite::params![id],
        ).map_err(|e| VaultError(format!("Failed to delete edges: {}", e)))?;

        let deleted = conn.execute(
            "DELETE FROM facts WHERE id = ?1",
            rusqlite::params![id],
        ).map_err(|e| VaultError(format!("Failed to delete fact: {}", e)))?;
        
        Ok(deleted > 0)
    }

    fn query_facts(&self, query: FactQuery) -> Result<Vec<Fact>, Self::Error> {
        let conn = self.conn.lock().unwrap();
        let mut sql = String::from(
            "SELECT id, content, source, confidence, created_at, updated_at, metadata
             FROM facts WHERE confidence >= ?1"
        );
        
        let mut params: Vec<rusqlite::types::Value> = vec![query.min_confidence.into()];
        
        if query.content_contains.is_some() {
            sql.push_str(" AND content LIKE ?2");
            let pattern = format!("%{}%", query.content_contains.unwrap());
            params.push(pattern.into());
        }
        
        if query.source_equals.is_some() {
            let param_num = params.len() + 1;
            sql.push_str(&format!(" AND source = ?{}", param_num));
            params.push(query.source_equals.unwrap().into());
        }
        
        sql.push_str(" ORDER BY updated_at DESC LIMIT ?");
        params.push((query.limit as i64).into());

        let mut stmt = conn.prepare(&sql)
            .map_err(|e| VaultError(format!("Failed to prepare query: {}", e)))?;

        let fact_iter = stmt.query_map(
            rusqlite::params_from_iter(params.iter()),
            Self::row_to_fact
        ).map_err(|e| VaultError(format!("Failed to execute query: {}", e)))?;

        let mut facts = Vec::new();
        for fact in fact_iter {
            facts.push(fact.map_err(|e| VaultError(format!("Row error: {}", e)))?);
        }
        
        Ok(facts)
    }

    fn create_edge(&mut self, edge: Edge) -> Result<String, Self::Error> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO edges (id, source_id, target_id, relation_type, confidence, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)
             ON CONFLICT(id) DO UPDATE SET
                relation_type=excluded.relation_type,
                confidence=excluded.confidence",
            rusqlite::params![
                &edge.id,
                &edge.source_id,
                &edge.target_id,
                &edge.relation_type,
                edge.confidence,
                edge.created_at as i64,
            ],
        ).map_err(|e| VaultError(format!("Failed to create edge: {}", e)))?;
        
        Ok(edge.id)
    }

    fn get_edges_from(&self, source_id: &FactId) -> Result<Vec<Edge>, Self::Error> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, source_id, target_id, relation_type, confidence, created_at
             FROM edges WHERE source_id = ?1"
        ).map_err(|e| VaultError(format!("Failed to prepare query: {}", e)))?;

        let edge_iter = stmt.query_map(rusqlite::params![source_id], Self::row_to_edge)
            .map_err(|e| VaultError(format!("Failed to execute query: {}", e)))?;

        let mut edges = Vec::new();
        for edge in edge_iter {
            edges.push(edge.map_err(|e| VaultError(format!("Row error: {}", e)))?);
        }
        
        Ok(edges)
    }

    fn get_edges_to(&self, target_id: &FactId) -> Result<Vec<Edge>, Self::Error> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, source_id, target_id, relation_type, confidence, created_at
             FROM edges WHERE target_id = ?1"
        ).map_err(|e| VaultError(format!("Failed to prepare query: {}", e)))?;

        let edge_iter = stmt.query_map(rusqlite::params![target_id], Self::row_to_edge)
            .map_err(|e| VaultError(format!("Failed to execute query: {}", e)))?;

        let mut edges = Vec::new();
        for edge in edge_iter {
            edges.push(edge.map_err(|e| VaultError(format!("Row error: {}", e)))?);
        }
        
        Ok(edges)
    }

    fn delete_edge(&mut self, id: &str) -> Result<bool, Self::Error> {
        let conn = self.conn.lock().unwrap();
        let deleted = conn.execute(
            "DELETE FROM edges WHERE id = ?1",
            rusqlite::params![id],
        ).map_err(|e| VaultError(format!("Failed to delete edge: {}", e)))?;
        
        Ok(deleted > 0)
    }

    fn fact_count(&self) -> Result<usize, Self::Error> {
        let conn = self.conn.lock().unwrap();
        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM facts",
            [],
            |row| row.get(0),
        ).map_err(|e| VaultError(format!("Failed to count facts: {}", e)))?;
        
        Ok(count as usize)
    }

    fn edge_count(&self) -> Result<usize, Self::Error> {
        let conn = self.conn.lock().unwrap();
        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM edges",
            [],
            |row| row.get(0),
        ).map_err(|e| VaultError(format!("Failed to count edges: {}", e)))?;
        
        Ok(count as usize)
    }
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

    #[test]
    fn sqlite_vault_works() {
        use std::path::Path;
        
        let temp_path = "/tmp/test_vault.db";
        // Clean up any existing test db
        let _ = std::fs::remove_file(temp_path);
        
        let mut vault = SqliteVault::open(temp_path).unwrap();
        
        // Store a fact
        let fact = create_fact("sqlite test content", "test_source");
        let id = fact.id.clone();
        vault.store_fact(fact).unwrap();
        
        // Retrieve it
        let retrieved = vault.get_fact(&id).unwrap();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().content, "sqlite test content");
        
        // Check count
        assert_eq!(vault.fact_count().unwrap(), 1);
        
        // Query
        let query = FactQuery {
            content_contains: Some("sqlite".to_string()),
            ..Default::default()
        };
        let results = vault.query_facts(query).unwrap();
        assert_eq!(results.len(), 1);
        
        // Clean up
        let _ = std::fs::remove_file(temp_path);
    }
}