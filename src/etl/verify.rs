//! Verify
//!
//! Confidence filtering and validation of extractions.

use crate::etl::extract::Extraction;

/// Verification result
#[derive(Debug, Clone)]
pub struct VerificationResult {
    /// Whether the extraction passed verification
    pub passed: bool,
    /// Adjusted confidence
    pub adjusted_confidence: f64,
    /// Reasons for rejection (if failed)
    pub rejection_reasons: Vec<String>,
    /// Flagged facts
    pub flagged_facts: Vec<String>,
}

/// Extraction verifier
pub struct Verifier {
    min_confidence: f64,
    min_facts: usize,
    max_fact_length: usize,
}

impl Verifier {
    pub fn new() -> Self {
        Self {
            min_confidence: 0.6,
            min_facts: 1,
            max_fact_length: 1000,
        }
    }

    pub fn with_config(min_confidence: f64, min_facts: usize) -> Self {
        Self {
            min_confidence: min_confidence.clamp(0.0, 1.0),
            min_facts,
            max_fact_length: 1000,
        }
    }

    /// Verify an extraction
    pub fn verify(&self, extraction: &Extraction) -> VerificationResult {
        let mut rejection_reasons = Vec::new();
        let mut flagged_facts = Vec::new();

        // Check confidence threshold
        if extraction.confidence < self.min_confidence {
            rejection_reasons.push(format!(
                "confidence {} below threshold {}",
                extraction.confidence, self.min_confidence
            ));
        }

        // Check minimum facts
        if extraction.facts.len() < self.min_facts {
            rejection_reasons.push(format!(
                "only {} facts, minimum is {}",
                extraction.facts.len(),
                self.min_facts
            ));
        }

        // Check fact quality
        for fact in &extraction.facts {
            if fact.len() > self.max_fact_length {
                flagged_facts.push(fact.clone());
                rejection_reasons.push("fact exceeds max length".to_string());
            }
            if self.is_vague(fact) {
                flagged_facts.push(fact.clone());
                rejection_reasons.push("vague or generic fact".to_string());
            }
        }

        // Adjust confidence based on flags
        let confidence_penalty = flagged_facts.len() as f64 * 0.1;
        let adjusted_confidence = (extraction.confidence - confidence_penalty).max(0.0);

        VerificationResult {
            passed: rejection_reasons.is_empty(),
            adjusted_confidence,
            rejection_reasons,
            flagged_facts,
        }
    }

    /// Check if a fact is too vague
    fn is_vague(&self, fact: &str) -> bool {
        let vague_phrases = ["something", "things", "stuff", "it depends"];
        let lower = fact.to_lowercase();
        vague_phrases.iter().any(|&p| lower.contains(p))
    }

    /// Batch verify multiple extractions
    pub fn verify_batch(&self, extractions: &[Extraction]) -> Vec<VerificationResult> {
        extractions.iter().map(|e| self.verify(e)).collect()
    }
}

impl Default for Verifier {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::etl::extract::Extractor;

    #[test]
    fn high_confidence_passes() {
        let verifier = Verifier::new();
        let extraction = Extraction {
            facts: vec!["Paris is capital of France".to_string()],
            relations: vec![],
            confidence: 0.9,
            latency_ms: 0,
        };
        let result = verifier.verify(&extraction);
        assert!(result.passed);
    }

    #[test]
    fn low_confidence_fails() {
        let verifier = Verifier::new();
        let extraction = Extraction {
            facts: vec!["test".to_string()],
            relations: vec![],
            confidence: 0.1,
            latency_ms: 0,
        };
        let result = verifier.verify(&extraction);
        assert!(!result.passed);
    }

    #[test]
    fn vague_facts_flagged() {
        let verifier = Verifier::new();
        let extraction = Extraction {
            facts: vec!["It depends on stuff".to_string()],
            relations: vec![],
            confidence: 0.8,
            latency_ms: 0,
        };
        let result = verifier.verify(&extraction);
        assert!(!result.passed);
        assert!(!result.flagged_facts.is_empty());
    }

    #[test]
    fn empty_facts_fail() {
        let verifier = Verifier::new();
        let extraction = Extraction {
            facts: vec![],
            relations: vec![],
            confidence: 0.9,
            latency_ms: 0,
        };
        let result = verifier.verify(&extraction);
        assert!(!result.passed);
    }
}