//! LLM Client
//!
//! HTTP client with retries and error handling.

use crate::modulation::parameter_mapper::LLMParameters;
use std::time::Duration;

/// LLM request
#[derive(Debug, Clone)]
pub struct LlmRequest {
    pub system_prompt: Option<String>,
    pub user_prompt: String,
    pub params: LLMParameters,
}

/// LLM response
#[derive(Debug, Clone)]
pub struct LlmResponse {
    pub text: String,
    pub tokens_used: u32,
    pub latency: Duration,
    pub model: String,
}

/// LLM client
pub struct LlmClient {
    endpoint: String,
    api_key: String,
    model: String,
    max_retries: u32,
    timeout: Duration,
}

impl LlmClient {
    pub fn new(endpoint: impl Into<String>, api_key: impl Into<String>, model: impl Into<String>) -> Self {
        Self {
            endpoint: endpoint.into(),
            api_key: api_key.into(),
            model: model.into(),
            max_retries: 3,
            timeout: Duration::from_secs(30),
        }
    }

    /// Send request to LLM
    ///
    /// In production: actual HTTP call with retry logic
    pub async fn send(&self, request: LlmRequest) -> Result<LlmResponse, LlmError> {
        // Placeholder implementation
        let start = std::time::Instant::now();

        // Simulate latency
        tokio::time::sleep(Duration::from_millis(100)).await;

        Ok(LlmResponse {
            text: format!("[Response to: {}]", request.user_prompt.chars().take(50).collect::<String>()),
            tokens_used: 100,
            latency: start.elapsed(),
            model: self.model.clone(),
        })
    }

    /// Set max retries
    pub fn with_retries(mut self, retries: u32) -> Self {
        self.max_retries = retries;
        self
    }
}

/// LLM error
#[derive(Debug)]
pub enum LlmError {
    RequestFailed(String),
    Timeout,
    RateLimited,
    InvalidResponse,
}

impl std::fmt::Display for LlmError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LlmError::RequestFailed(s) => write!(f, "request failed: {}", s),
            LlmError::Timeout => write!(f, "request timeout"),
            LlmError::RateLimited => write!(f, "rate limited"),
            LlmError::InvalidResponse => write!(f, "invalid response"),
        }
    }
}

impl std::error::Error for LlmError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn client_returns_response() {
        let client = LlmClient::new("http://test", "key", "model");
        let request = LlmRequest {
            system_prompt: None,
            user_prompt: "test prompt".to_string(),
            params: LLMParameters {
                temperature: 0.7,
                max_tokens: 100,
                top_p: 0.9,
            },
        };

        let response = client.send(request).await.unwrap();
        assert!(!response.text.is_empty());
    }
}