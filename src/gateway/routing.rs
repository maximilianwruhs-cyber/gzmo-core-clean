//! Router
//!
//! Multi-model fallback routing.

use crate::gateway::client::{LlmClient, LlmRequest, LlmResponse};

/// Model preference for routing
#[derive(Debug, Clone)]
pub struct ModelPreference {
    pub provider: String,
    pub model: String,
    pub priority: u32, // Lower = higher priority
}

/// Request router with fallback
pub struct Router {
    routes: Vec<(ModelPreference, LlmClient)>,
    current: usize,
}

impl Router {
    pub fn new() -> Self {
        Self {
            routes: Vec::new(),
            current: 0,
        }
    }

    /// Add a route
    pub fn add_route(&mut self, pref: ModelPreference, client: LlmClient) {
        self.routes.push((pref, client));
        self.routes.sort_by_key(|(p, _)| p.priority);
    }

    /// Send request with fallback
    pub async fn send(&mut self, request: LlmRequest) -> Result<LlmResponse, RouterError> {
        for (pref, client) in &self.routes {
            match client.send(request.clone()).await {
                Ok(response) => return Ok(response),
                Err(e) => {
                    eprintln!("Route {} failed: {}, trying next...", pref.model, e);
                    continue;
                }
            }
        }

        Err(RouterError::AllRoutesFailed)
    }

    /// Get route count
    pub fn route_count(&self) -> usize {
        self.routes.len()
    }
}

impl Default for Router {
    fn default() -> Self {
        Self::new()
    }
}

/// Router error
#[derive(Debug)]
pub enum RouterError {
    AllRoutesFailed,
    NoRoutesConfigured,
}

impl std::fmt::Display for RouterError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RouterError::AllRoutesFailed => write!(f, "all routes failed"),
            RouterError::NoRoutesConfigured => write!(f, "no routes configured"),
        }
    }
}

impl std::error::Error for RouterError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn single_route_works() {
        let mut router = Router::new();
        router.add_route(
            ModelPreference {
                provider: "test".to_string(),
                model: "model".to_string(),
                priority: 1,
            },
            LlmClient::new("http://test", "key", "model"),
        );

        let request = LlmRequest {
            system_prompt: None,
            user_prompt: "test".to_string(),
            params: crate::modulation::parameter_mapper::LLMParameters {
                temperature: 0.7,
                max_tokens: 100,
                top_p: 0.9,
            },
        };

        let response = router.send(request).await.unwrap();
        assert!(!response.text.is_empty());
    }

    #[test]
    fn route_count_tracks_routes() {
        let mut router = Router::new();
        assert_eq!(router.route_count(), 0);

        router.add_route(
            ModelPreference {
                provider: "test".to_string(),
                model: "model".to_string(),
                priority: 1,
            },
            LlmClient::new("http://test", "key", "model"),
        );

        assert_eq!(router.route_count(), 1);
    }
}