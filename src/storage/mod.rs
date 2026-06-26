//! Storage Module
//!
//! Two-layer storage: raw input and curated facts.
//!
//! Components:
//! - `vault`: Structured SQLite storage for facts and edges
//! - `vectors`: Qdrant semantic search integration
//! - `dedup`: Binary duplicate detection (Duplicate/Novel)

pub mod vault;
pub mod vectors;
pub mod dedup;

pub use vault::{Fact, FactId, Edge, FactQuery, Vault, InMemoryVault, SqliteVault, create_fact};
pub use vectors::{VectorStore, VecMetadata, SearchResult, InMemoryVectorStore, cosine_similarity};
pub use dedup::{DuplicateDetector, FactRelation};
