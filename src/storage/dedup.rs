//! Duplicate Detection
//!
//! Simple binary duplicate detection: content hash and semantic similarity.
//! Replaces the complex 6-variant lifecycle enum with straightforward checks.

use std::collections::HashSet;

/// Simple binary duplicate detection
#[derive(Debug, Clone)]
pub struct DuplicateDetector {
    /// Threshold for semantic similarity (considered duplicate)
    similarity_threshold: f64,
    /// Stored hashes for exact matching
    content_hashes: HashSet<u64>,
    /// Recent embeddings for similarity comparison
    recent_embeddings: Vec<(String, Vec<f32>)>,
    /// Max recent embeddings to keep
    max_recent: usize,
}

/// Classification result
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FactRelation {
    /// Content hash matches exactly or semantic similarity >= threshold
    Duplicate,
    /// New unique content
    Novel,
}

impl FactRelation {
    /// Should this fact be stored
    pub fn should_store(&self) -> bool {
        matches!(self, Self::Novel)
    }

    /// Is this a duplicate
    pub fn is_duplicate(&self) -> bool {
        matches!(self, Self::Duplicate)
    }

    /// Is this novel (not a duplicate)
    pub fn is_novel(&self) -> bool {
        matches!(self, Self::Novel)
    }
}

impl DuplicateDetector {
    /// Create detector with default threshold
    pub fn new() -> Self {
        Self::with_threshold(0.95)
    }

    /// Create with custom threshold
    ///
    /// # Arguments
    /// * `similarity_threshold` - Cosine similarity above which items are considered duplicates
    pub fn with_threshold(similarity_threshold: f64) -> Self {
        Self {
            similarity_threshold: similarity_threshold.clamp(0.8, 0.99),
            content_hashes: HashSet::new(),
            recent_embeddings: Vec::with_capacity(1000),
            max_recent: 1000,
        }
    }

    /// Check if content is novel
    ///
    /// # Arguments
    /// * `content` - Text content to check
    /// * `embedding` - Optional embedding for semantic comparison
    pub fn check(&mut self, content: impl AsRef<str>, embedding: Option<&[f32]>) -> FactRelation {
        let content = content.as_ref();
        let hash = hash_content(content);

        // Exact match check
        if self.content_hashes.contains(&hash) {
            return FactRelation::Duplicate;
        }

        // Semantic similarity check (if embedding provided)
        if let Some(emb) = embedding {
            for (_, recent_emb) in &self.recent_embeddings {
                let sim = cosine_similarity(emb, recent_emb);
                if sim >= self.similarity_threshold {
                    return FactRelation::Duplicate;
                }
            }
        }

        // Store for future checks
        self.content_hashes.insert(hash);
        if let Some(emb) = embedding {
            self.recent_embeddings.push((content.to_string(), emb.to_vec()));
            if self.recent_embeddings.len() > self.max_recent {
                self.recent_embeddings.remove(0);
            }
        }

        FactRelation::Novel
    }

    /// Add known content without checking
    pub fn add_known(&mut self, content: impl AsRef<str>, embedding: Option<&[f32]>) {
        let content = content.as_ref();
        self.content_hashes.insert(hash_content(content));
        if let Some(emb) = embedding {
            self.recent_embeddings.push((content.to_string(), emb.to_vec()));
        }
    }

    /// Count of unique hashes seen
    pub fn unique_count(&self) -> usize {
        self.content_hashes.len()
    }

    /// Reset all tracked data
    pub fn clear(&mut self) {
        self.content_hashes.clear();
        self.recent_embeddings.clear();
    }
}

impl Default for DuplicateDetector {
    fn default() -> Self {
        Self::new()
    }
}

/// Hash content for exact matching
fn hash_content(content: &str) -> u64 {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut hasher = DefaultHasher::new();
    // Normalize: lowercase, trim whitespace
    content.to_lowercase().trim().hash(&mut hasher);
    hasher.finish()
}

/// Compute cosine similarity (re-export from vectors)
fn cosine_similarity(a: &[f32], b: &[f32]) -> f64 {
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
    fn identical_content_is_duplicate() {
        let mut detector = DuplicateDetector::new();
        let result = detector.check("hello world", None);
        assert!(result.is_novel());

        let result2 = detector.check("hello world", None);
        assert!(result2.is_duplicate());
    }

    #[test]
    fn case_insensitive_matching() {
        let mut detector = DuplicateDetector::new();
        detector.check("Hello World", None);

        let result = detector.check("hello world", None);
        assert!(result.is_duplicate());
    }

    #[test]
    fn whitespace_normalized() {
        let mut detector = DuplicateDetector::new();
        detector.check("hello world", None);

        let result = detector.check("  hello world  ", None);
        assert!(result.is_duplicate());
    }

    #[test]
    fn semantic_similarity_catches_near_duplicates() {
        let mut detector = DuplicateDetector::with_threshold(0.95);

        let emb1 = vec![1.0, 0.0, 0.0, 0.0];
        let emb2 = vec![0.99, 0.01, 0.0, 0.0]; // Very similar

        let result = detector.check("content A", Some(&emb1));
        assert!(result.is_novel());

        let result2 = detector.check("content B", Some(&emb2));
        assert!(result2.is_duplicate()); // Caught by similarity
    }

    #[test]
    fn dissimilar_content_is_novel() {
        let mut detector = DuplicateDetector::with_threshold(0.95);

        let emb1 = vec![1.0, 0.0, 0.0];
        let emb2 = vec![0.0, 1.0, 0.0]; // Orthogonal

        detector.check("content A", Some(&emb1));
        let result = detector.check("content B", Some(&emb2));

        assert!(result.is_novel());
    }

    #[test]
    fn unique_count_tracks_insertions() {
        let mut detector = DuplicateDetector::new();
        assert_eq!(detector.unique_count(), 0);

        detector.check("a", None);
        assert_eq!(detector.unique_count(), 1);

        detector.check("a", None); // Duplicate
        assert_eq!(detector.unique_count(), 1);

        detector.check("b", None);
        assert_eq!(detector.unique_count(), 2);
    }
}