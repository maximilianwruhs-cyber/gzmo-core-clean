//! Gateway Module
//!
//! LLM API abstraction with routing and caching.

pub mod client;
pub mod routing;
pub mod cache;

pub use client::{LlmClient, LlmRequest, LlmResponse, LlmError};
pub use routing::{Router, ModelPreference};
pub use cache::{ResponseCache, CacheEntry};
