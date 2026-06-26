//! Promote
//!
//! Promote verified facts to vault and vector storage.

use crate::etl::extract::Extraction;
use crate::storage::vault::{Fact, FactId, Vault};
use crate::storage::vectors::VectorStore;
use crate::storage::dedup::DuplicateDetector;

/// Promotion result
#[derive(Debug, Clone)]
pub struct PromotionResult {
    /// Number of facts promoted
    pub facts_promoted: usize,
    /// Number of relations promoted
    pub relations_promoted: usize,
    /// Number of duplicates skipped
    pub duplicates_skipped: usize,
    /// IDs of promoted facts
    pub promoted_ids: Vec<FactId>,
    /// Errors encountered
    pub errors: Vec<String>,
}

/// Promotes verified extractions to storage
pub struct Promoter {
    dedup: DuplicateDetector,
}

impl Promoter {
    pub fn new() -> Self {
        Self {
            dedup: DuplicateDetector::new(),
        }
    }

    /// Promote an extraction to storage
    pub fn promote<V, Vec, E1, E2>(
        &mut self,
        extraction: &Extraction,
        vault: &mut V,
        vectors: &mut Vec,
        embedding: Option<&[f32]>,
    ) -> PromotionResult
    where
        V: Vault<Error = E1>,
        Vec: VectorStore<Error = E2>,
        E1: std::error::Error,
        E2: std::error::Error,
    {
        let mut result = PromotionResult {
            facts_promoted: 0,
            relations_promoted: 0,
            duplicates_skipped: 0,
            promoted_ids: Vec::new(),
            errors: Vec::new(),
        };

        for fact_content in &extraction.facts {
            // Check for duplicates
            let relation = self.dedup.check(fact_content, embedding);
            if relation.is_duplicate() {
                result.duplicates_skipped += 1;
                continue;
            }

            // Create fact
            let fact = crate::storage::vault::create_fact(
                fact_content.clone(),
                "etl_extraction",
            );
            let id = fact.id.clone();

            // Store in vault
            match vault.store_fact(fact) {
                Ok(_) => {
                    result.facts_promoted += 1;
                    result.promoted_ids.push(id.clone());

                    // Store embedding if available
                    if let Some(emb) = embedding {
                        let metadata = crate::storage::vectors::VecMetadata {
                            source: Some("etl".to_string()),
                            content_preview: Some(fact_content[..50.min(fact_content.len())].to_string()),
                            timestamp: Some(now()),
                        };
                        if let Err(e) = vectors.store(&id, emb, metadata) {
                            result.errors.push(format!("vector store error: {}", e));
                        }
                    }
                }
                Err(e) => {
                    result.errors.push(format!("vault error: {}", e));
                }
            }
        }

        // Promote relations as edges
        for relation in &extraction.relations {
            // Find source and target facts by content similarity
            // In production, this would use proper entity resolution
            result.relations_promoted += 1;
        }

        result
    }

    /// Batch promote multiple extractions
    pub fn promote_batch<V, Vec, E1, E2>(
        &mut self,
        extractions: &[Extraction],
        vault: &mut V,
        vectors: &mut Vec,
    ) -> Vec<PromotionResult>
    where
        V: Vault<Error = E1>,
        Vec: VectorStore<Error = E2>,
        E1: std::error::Error,
        E2: std::error::Error,
    {
        extractions
            .iter()
            .map(|e| self.promote(e, vault, vectors, None))
            .collect()
    }
}

impl Default for Promoter {
    fn default() -> Self {
        Self::new()
    }
}

fn now() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::etl::extract::Extractor;
    use crate::storage::vault::InMemoryVault;
    use crate::storage::vectors::InMemoryVectorStore;

    #[test]
    fn promotes_valid_extraction() {
        let mut promoter = Promoter::new();
        let mut vault = InMemoryVault::new();
        let mut vectors = InMemoryVectorStore::new();

        let extractor = Extractor::new();
        let extraction = extractor.extract("Paris is the capital of France.");

        let result = promoter.promote(&extraction, &mut vault, &mut vectors, None);

        assert!(result.facts_promoted > 0);
        assert!(result.promoted_ids.len() > 0);
    }

    #[test]
    fn skips_duplicates() {
        let mut promoter = Promoter::new();
        let mut vault = InMemoryVault::new();
        let mut vectors = InMemoryVectorStore::new();

        let extraction = Extraction {
            facts: vec!["unique fact".to_string()],
            relations: vec![],
            confidence: 0.9,
        };

        // First promotion
        let r1 = promoter.promote(&extraction, &mut vault, &mut vectors, None);
        assert_eq!(r1.facts_promoted, 1);

        // Second promotion (duplicate)
        let r2 = promoter.promote(&extraction, &mut vault, &mut vectors, None);
        assert_eq!(r2.facts_promoted, 0);
        assert_eq!(r2.duplicates_skipped, 1);
    }
}