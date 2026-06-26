//! Vector Storage
//!
//! Semantic search via Qdrant integration (placeholder).
//! Provides embedding storage and cosine similarity search.

/// Vector storage trait
///
/// Abstraction over Qdrant or other vector databases.
pub trait VectorStore: Send + Sync {
    type Error: std::error::Error;

    /// Store an embedding
    fn store(&mut self, id: &str, embedding: &[f32], metadata: VecMetadata) -> Result<(), Self::Error>;

    /// Search for similar vectors
    fn search(&self, query: &[f32], limit: usize) -> Result<Vec<SearchResult>, Self::Error>;

    /// Delete an embedding
    fn delete(&mut self, id: &str) -> Result<bool, Self::Error>;

    /// Get count of stored vectors
    fn count(&self) -> Result<usize, Self::Error>;
}

/// Metadata for a vector entry
#[derive(Debug, Clone, Default)]
pub struct VecMetadata {
    pub source: Option<String>,
    pub content_preview: Option<String>,
    pub timestamp: Option<u64>,
}

/// Search result
#[derive(Debug, Clone)]
pub struct SearchResult {
    pub id: String,
    pub score: f64,
    pub metadata: VecMetadata,
}

/// In-memory vector store for testing
pub struct InMemoryVectorStore {
    vectors: std::collections::HashMap<String, VecEntry>,
}

struct VecEntry {
    embedding: Vec<f32>,
    metadata: VecMetadata,
}

impl InMemoryVectorStore {
    pub fn new() -> Self {
        Self {
            vectors: std::collections::HashMap::new(),
        }
    }
}

#[derive(Debug)]
pub struct VecError(String);

impl std::fmt::Display for VecError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for VecError {}

impl VectorStore for InMemoryVectorStore {
    type Error = VecError;

    fn store(&mut self, id: &str, embedding: &[f32], metadata: VecMetadata) -> Result<(), Self::Error> {
        self.vectors.insert(
            id.to_string(),
            VecEntry {
                embedding: embedding.to_vec(),
                metadata,
            },
        );
        Ok(())
    }

    fn search(&self, query: &[f32], limit: usize) -> Result<Vec<SearchResult>, Self::Error> {
        let mut results: Vec<_> = self
            .vectors
            .iter()
            .map(|(id, entry)| {
                let score = cosine_similarity(query, &entry.embedding);
                SearchResult {
                    id: id.clone(),
                    score,
                    metadata: entry.metadata.clone(),
                }
            })
            .collect();

        results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());
        results.truncate(limit);
        Ok(results)
    }

    fn delete(&mut self, id: &str) -> Result<bool, Self::Error> {
        Ok(self.vectors.remove(id).is_some())
    }

    fn count(&self) -> Result<usize, Self::Error> {
        Ok(self.vectors.len())
    }
}

/// Compute cosine similarity between two vectors
pub fn cosine_similarity(a: &[f32], b: &[f32]) -> f64 {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identical_vectors_have_similarity_1() {
        let a = vec![1.0, 0.0, 0.0];
        let b = vec![1.0, 0.0, 0.0];
        assert!((cosine_similarity(&a, &b) - 1.0).abs() < 0.001);
    }

    #[test]
    fn orthogonal_vectors_have_similarity_0() {
        let a = vec![1.0, 0.0];
        let b = vec![0.0, 1.0];
        assert!(cosine_similarity(&a, &b).abs() < 0.001);
    }

    #[test]
    fn search_returns_sorted_results() {
        let mut store = InMemoryVectorStore::new();

        // Three vectors at different angles
        store.store("a", &[1.0, 0.0, 0.0], VecMetadata::default()).unwrap();
        store.store("b", &[0.7, 0.7, 0.0], VecMetadata::default()).unwrap();
        store.store("c", &[0.0, 1.0, 0.0], VecMetadata::default()).unwrap();

        // Query along x-axis
        let results = store.search(&[1.0, 0.0, 0.0], 2).unwrap();

        assert_eq!(results.len(), 2);
        assert_eq!(results[0].id, "a");
        assert!(results[0].score > results[1].score);
    }
}