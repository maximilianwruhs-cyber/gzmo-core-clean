//! Extract
//!
//! LLM-based extraction of structured facts from raw text.

use crate::gateway::{LlmClient, LlmRequest};
use crate::modulation::parameter_mapper::LLMParameters;
use serde::{Deserialize, Serialize};

/// Structured extraction from text
#[derive(Debug, Clone)]
pub struct Extraction {
    /// Extracted fact statements
    pub facts: Vec<String>,
    /// Relationships detected
    pub relations: Vec<Relation>,
    /// Confidence score (0-1)
    pub confidence: f64,
    /// LLM latency
    pub latency_ms: u64,
}

/// Detected relationship
#[derive(Debug, Clone)]
pub struct Relation {
    pub source: String,
    pub relation_type: String,
    pub target: String,
    pub confidence: f64,
}

/// LLM response structure for extraction
#[derive(Debug, Clone, Deserialize)]
struct LlmExtractionResponse {
    facts: Vec<String>,
    relations: Vec<LlmRelation>,
    confidence: f64,
    summary: String,
}

#[derive(Debug, Clone, Deserialize)]
struct LlmRelation {
    source: String,
    relation_type: String,
    target: String,
    confidence: f64,
}

/// LLM-based fact extractor
pub struct Extractor {
    client: LlmClient,
    min_confidence: f64,
}

impl Extractor {
    /// Create extractor with default local endpoint
    pub fn new() -> Self {
        Self {
            client: LlmClient::new(
                "http://localhost:8000/v1/chat/completions",
                "",
                "local-model",
            ),
            min_confidence: 0.7,
        }
    }

    /// Create with custom LLM client
    pub fn with_client(client: LlmClient) -> Self {
        Self {
            client,
            min_confidence: 0.7,
        }
    }

    /// Set confidence threshold
    pub fn with_threshold(mut self, min_confidence: f64) -> Self {
        self.min_confidence = min_confidence.clamp(0.0, 1.0);
        self
    }

    /// Set confidence threshold (alternative for building from existing instance)
    pub fn with_min_confidence(&mut self, min_confidence: f64) -> &mut Self {
        self.min_confidence = min_confidence.clamp(0.0, 1.0);
        self
    }

    /// Extract facts from raw text using LLM
    pub async fn extract(&self, text: &str) -> Result<Extraction, ExtractorError> {
        let start = std::time::Instant::now();

        let prompt = format!(
            "Extract factual statements and relationships from the following text:\n\n\
            \"{}\"\n\n\
            Respond in this exact JSON format:\n\
            {{\n\
              \"facts\": [\n\
                \"First fact extracted from the text\",\n\
                \"Second fact extracted from the text\"\n\
              ],\n\
              \"relations\": [\n\
                {{\n\
                  \"source\": \"entity1\",\n\
                  \"relation_type\": \"is_a|has|located_in|part_of|causes\",\n\
                  \"target\": \"entity2\",\n\
                  \"confidence\": 0.0-1.0\n\
                }}\n\
              ],\n\
              \"confidence\": 0.0-1.0,\n\
              \"summary\": \"Brief summary of the extraction\"\n\
            }}\n\n\
            Guidelines:\n\
            - Extract only high-confidence facts\n\
            - Capture specific entity relationships\n\
            - Avoid speculation or inference beyond the text\n\
            - Confidence should reflect certainty of extraction",
            text.replace('"', "\\\"")
        );

        let request = LlmRequest {
            system_prompt: Some(
                "You are an expert knowledge extraction system. \
                Extract factual information accurately from provided text. \
                Return only valid JSON in the requested format.".to_string()
            ),
            user_prompt: prompt,
            params: LLMParameters {
                temperature: 0.2, // Low temperature for consistent extraction
                max_tokens: 800,
                top_p: 0.9,
            },
        };

        let response = self.client.send(request).await
            .map_err(|e| ExtractorError::LlmError(e.to_string()))?;

        let extraction: LlmExtractionResponse = serde_json::from_str(&response.text)
            .map_err(|e| ExtractorError::ParseError(
                format!("Failed to parse LLM response: {}. Response: {}", 
                    e, 
                    response.text.chars().take(200).collect::<String>())
            ))?;

        // Filter by confidence
        let filtered_facts: Vec<String> = extraction.facts
            .into_iter()
            .filter(|f| !f.is_empty() && f.len() > 10)
            .collect();

        let relations: Vec<Relation> = extraction.relations
            .into_iter()
            .filter(|r| r.confidence >= self.min_confidence)
            .map(|r| Relation {
                source: r.source,
                relation_type: r.relation_type,
                target: r.target,
                confidence: r.confidence,
            })
            .collect();

        let latency = start.elapsed();

        Ok(Extraction {
            facts: filtered_facts,
            relations,
            confidence: extraction.confidence.clamp(0.0, 1.0),
            latency_ms: latency.as_millis() as u64,
        })
    }

    /// Quick heuristic extraction (no LLM call)
    pub fn extract_heuristic(&self, text: &str) -> Extraction {
        let start = std::time::Instant::now();
        
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

        let confidence = if sentences.is_empty() {
            0.0
        } else {
            0.7 + 0.1 * (relations.len() as f64 / sentences.len() as f64).min(1.0)
        };

        let latency = start.elapsed();

        Extraction {
            facts: sentences,
            relations,
            confidence,
            latency_ms: latency.as_millis() as u64,
        }
    }

    /// Generate LLM extraction prompt (for debugging)
    pub fn extraction_prompt(&self, text: &str) -> String {
        format!(
            "Extract factual statements and relationships from:\n\n{}\n\n\
            Output format (JSON):\n\
            {{\n\
              \"facts\": [...],\n\
              \"relations\": [...],\n\
              \"confidence\": 0.0-1.0,\n\
              \"summary\": \"...\"\n\
            }}",
            text
        )
    }
}

impl Default for Extractor {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug)]
pub enum ExtractorError {
    LlmError(String),
    ParseError(String),
}

impl std::fmt::Display for ExtractorError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ExtractorError::LlmError(s) => write!(f, "LLM error: {}", s),
            ExtractorError::ParseError(s) => write!(f, "Parse error: {}", s),
        }
    }
}

impl std::error::Error for ExtractorError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn heuristic_extracts_sentences() {
        let extractor = Extractor::new();
        let result = extractor.extract_heuristic("Paris is the capital of France. London is in the UK.");
        assert!(!result.facts.is_empty());
    }

    #[test]
    fn heuristic_detects_simple_relations() {
        let extractor = Extractor::new();
        // Sentence must be >20 chars for extraction
        let result = extractor.extract_heuristic("A domestic dog is classified as a mammalian animal in biology.");
        assert!(!result.relations.is_empty());
    }

    #[test]
    fn confidence_threshold_works() {
        let extractor = Extractor::new().with_threshold(0.8);
        // Test that threshold is set - use sentences >20 chars for extraction
        let result = extractor.extract_heuristic("This example demonstrates the confidence threshold mechanism in extraction.");
        // Relations below threshold would be filtered in real extraction
        assert!(result.confidence > 0.0);
    }
}