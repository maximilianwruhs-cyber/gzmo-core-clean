//! Extract
//!
//! LLM-based extraction of structured facts from raw text.

/// Structured extraction from text
#[derive(Debug, Clone)]
pub struct Extraction {
    /// Extracted fact statements
    pub facts: Vec<String>,
    /// Relationships detected
    pub relations: Vec<Relation>,
    /// Confidence score (0-1)
    pub confidence: f64,
}

/// Detected relationship
#[derive(Debug, Clone)]
pub struct Relation {
    pub source: String,
    pub relation_type: String,
    pub target: String,
    pub confidence: f64,
}

/// LLM-based fact extractor
pub struct Extractor {
    min_confidence: f64,
}

impl Extractor {
    pub fn new() -> Self {
        Self { min_confidence: 0.7 }
    }

    pub fn with_threshold(min_confidence: f64) -> Self {
        Self {
            min_confidence: min_confidence.clamp(0.0, 1.0),
        }
    }

    /// Extract facts from raw text
    ///
    /// In production, this calls an LLM. Here we provide a heuristic placeholder.
    pub fn extract(&self, text: &str) -> Extraction {
        let sentences: Vec<_> = text
            .split(|c| c == '.' || c == '!' || c == '?')
            .map(|s| s.trim())
            .filter(|s| !s.is_empty() && s.len() > 20)
            .map(|s| s.to_string())
            .collect();

        // Simple relation detection: "X is Y" patterns
        let mut relations = Vec::new();
        for sent in &sentences {
            if let Some((a, b)) = sent.split_once(" is ") {
                let a = a.split_whitespace().last().unwrap_or(a);
                let b = b.split_whitespace().next().unwrap_or(b);
                relations.push(Relation {
                    source: a.to_string(),
                    relation_type: "is".to_string(),
                    target: b.to_string(),
                    confidence: 0.8,
                });
            }
        }

        // Confidence based on extraction quality
        let confidence = if sentences.is_empty() {
            0.0
        } else {
            0.7 + 0.1 * (relations.len() as f64 / sentences.len() as f64).min(1.0)
        };

        Extraction {
            facts: sentences,
            relations,
            confidence,
        }
    }

    /// Generate LLM extraction prompt
    pub fn extraction_prompt(&self, text: &str) -> String {
        format!(
            "Extract factual statements and relationships from:\n\n{}\n\n\
            Output format:\n\
            FACTS:\n\
            - [fact 1]\n\
            - [fact 2]\n\n\
            RELATIONS:\n\
            - [entity1] --[relation]--> [entity2]\n\n\
            Only include high-confidence extractions.",
            text
        )
    }
}

impl Default for Extractor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_sentences() {
        let extractor = Extractor::new();
        let result = extractor.extract("Paris is the capital of France. London is in the UK.");
        assert!(!result.facts.is_empty());
    }

    #[test]
    fn detects_simple_relations() {
        let extractor = Extractor::new();
        let result = extractor.extract("A dog is an animal.");
        assert!(!result.relations.is_empty());
        assert!(result.relations[0].relation_type == "is");
    }

    #[test]
    fn confidence_based_on_content() {
        let extractor = Extractor::new();
        let good = extractor.extract("Paris is capital. London is capital. Berlin is capital.");
        let bad = extractor.extract("hi");
        assert!(good.confidence > bad.confidence);
    }
}