//! LLM Client
//!
//! HTTP client for connecting to llama.cpp server or OpenAI-compatible APIs.

use crate::modulation::parameter_mapper::LLMParameters;
use serde::{Deserialize, Serialize};
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

/// OpenAI-compatible API request body
#[derive(Serialize, Debug)]
struct ApiRequest {
    model: String,
    messages: Vec<Message>,
    temperature: f32,
    max_tokens: u32,
    top_p: f32,
}

#[derive(Serialize, Debug)]
struct Message {
    role: String,
    content: String,
}

/// OpenAI-compatible API response
#[derive(Deserialize, Debug)]
struct ApiResponse {
    choices: Vec<Choice>,
    usage: Option<Usage>,
}

#[derive(Deserialize, Debug)]
struct Choice {
    message: ResponseMessage,
}

#[derive(Deserialize, Debug)]
struct ResponseMessage {
    content: String,
}

#[derive(Deserialize, Debug)]
struct Usage {
    prompt_tokens: u32,
    completion_tokens: u32,
    total_tokens: u32,
}

/// LLM client with retry logic
#[derive(Clone)]
pub struct LlmClient {
    endpoint: String,
    api_key: String,
    model: String,
    max_retries: u32,
    timeout: Duration,
    http_client: std::sync::Arc<reqwest::Client>,
}

impl LlmClient {
    /// Create new client for llama.cpp or OpenAI-compatible endpoint
    pub fn new(endpoint: impl Into<String>, api_key: impl Into<String>, model: impl Into<String>) -> Self {
        let endpoint = endpoint.into();
        let api_key = api_key.into();
        let model = model.into();
        
        let http_client = std::sync::Arc::new(
            reqwest::Client::builder()
                .timeout(Duration::from_secs(30))
                .build()
                .expect("Failed to build HTTP client")
        );
        
        Self {
            endpoint,
            api_key,
            model,
            max_retries: 3,
            timeout: Duration::from_secs(30),
            http_client,
        }
    }

    /// Create client for local llama.cpp server on port 8000
    pub fn local_llamacpp(model: impl Into<String>) -> Self {
        Self::new(
            "http://localhost:8000/v1/chat/completions",
            "not-needed",
            model,
        )
    }

    /// Set max retries
    pub fn with_retries(mut self, retries: u32) -> Self {
        self.max_retries = retries;
        self
    }

    /// Set custom timeout
    pub fn with_timeout(mut self, seconds: u64) -> Self {
        self.timeout = Duration::from_secs(seconds);
        self
    }

    /// Send request to LLM with retry logic
    pub async fn send(&self, request: LlmRequest) -> Result<LlmResponse, LlmError> {
        let start = std::time::Instant::now();
        
        // Build messages array
        let mut messages = Vec::new();
        if let Some(system) = &request.system_prompt {
            messages.push(Message {
                role: "system".to_string(),
                content: system.clone(),
            });
        }
        messages.push(Message {
            role: "user".to_string(),
            content: request.user_prompt.clone(),
        });

        let api_request = ApiRequest {
            model: self.model.clone(),
            messages,
            temperature: request.params.temperature,
            max_tokens: request.params.max_tokens,
            top_p: request.params.top_p,
        };

        let mut last_error = None;
        
        for attempt in 0..self.max_retries {
            match self.send_single(&api_request).await {
                Ok(response) => {
                    return Ok(LlmResponse {
                        text: response.choices.get(0)
                            .map(|c| c.message.content.clone())
                            .unwrap_or_default(),
                        tokens_used: response.usage.map(|u| u.total_tokens).unwrap_or(0),
                        latency: start.elapsed(),
                        model: self.model.clone(),
                    });
                }
                Err(e) => {
                    last_error = Some(e);
                    if attempt < self.max_retries - 1 {
                        tokio::time::sleep(Duration::from_millis(100 * (attempt + 1) as u64)).await;
                    }
                }
            }
        }

        Err(last_error.unwrap_or_else(|| LlmError::RequestFailed("All retries exhausted".to_string())))
    }

    /// Send single HTTP request
    async fn send_single(&self, request: &ApiRequest) -> Result<ApiResponse, LlmError> {
        let mut req = (*self.http_client)
            .post(&self.endpoint)
            .header("Content-Type", "application/json");
        
        // Add auth header if API key is provided and not "not-needed"
        if !self.api_key.is_empty() && self.api_key != "not-needed" {
            req = req.header("Authorization", format!("Bearer {}", self.api_key));
        }

        let response = req
            .json(request)
            .send()
            .await
            .map_err(|e| LlmError::RequestFailed(e.to_string()))?;

        let status = response.status();
        
        if !status.is_success() {
            let text = response.text().await.unwrap_or_default();
            return Err(LlmError::RequestFailed(
                format!("HTTP {}: {}", status.as_u16(), text)
            ));
        }

        let api_response: ApiResponse = response
            .json()
            .await
            .map_err(|e| LlmError::InvalidResponse(format!("Failed to parse JSON: {}", e)))?;

        Ok(api_response)
    }

    /// Get endpoint URL
    pub fn endpoint(&self) -> &str {
        &self.endpoint
    }

    /// Get model name
    pub fn model(&self) -> &str {
        &self.model
    }
}

/// LLM error types
#[derive(Debug)]
pub enum LlmError {
    RequestFailed(String),
    Timeout,
    RateLimited,
    InvalidResponse(String),
}

impl std::fmt::Display for LlmError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LlmError::RequestFailed(s) => write!(f, "request failed: {}", s),
            LlmError::Timeout => write!(f, "request timeout"),
            LlmError::RateLimited => write!(f, "rate limited"),
            LlmError::InvalidResponse(s) => write!(f, "invalid response: {}", s),
        }
    }
}

impl std::error::Error for LlmError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn client_construction() {
        let client = LlmClient::local_llamacpp("llama3");
        assert_eq!(client.endpoint(), "http://localhost:8000/v1/chat/completions");
        assert_eq!(client.model(), "llama3");
    }

    #[test]
    fn request_building() {
        let request = LlmRequest {
            system_prompt: Some("You are helpful".to_string()),
            user_prompt: "Hello".to_string(),
            params: LLMParameters {
                temperature: 0.7,
                max_tokens: 100,
                top_p: 0.9,
            },
        };
        
        assert_eq!(request.user_prompt, "Hello");
        assert_eq!(request.params.temperature, 0.7);
    }
}