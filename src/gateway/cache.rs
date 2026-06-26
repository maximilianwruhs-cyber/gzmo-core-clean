//! Response Cache
//!
//! Simple LRU cache for LLM responses.

use crate::gateway::client::LlmResponse;
use std::collections::HashMap;

/// Cached entry
#[derive(Debug, Clone)]
pub struct CacheEntry {
    pub response: LlmResponse,
    pub hits: u64,
    pub created_at: u64,
}

/// LRU response cache
pub struct ResponseCache {
    entries: HashMap<String, CacheEntry>,
    max_size: usize,
    ttl_seconds: u64,
}

impl ResponseCache {
    pub fn new(max_size: usize, ttl_seconds: u64) -> Self {
        Self {
            entries: HashMap::with_capacity(max_size),
            max_size,
            ttl_seconds,
        }
    }

    /// Get cached response if available and not expired
    pub fn get(&mut self, key: &str) -> Option<LlmResponse> {
        let now = now();

        if let Some(entry) = self.entries.get_mut(key) {
            if now - entry.created_at > self.ttl_seconds {
                // Expired
                self.entries.remove(key);
                return None;
            }

            entry.hits += 1;
            return Some(entry.response.clone());
        }

        None
    }
    /// Store response in cache
    pub fn put(&mut self, key: impl Into<String>, response: LlmResponse) {
        let key = key.into();

        // Evict oldest if at capacity
        if self.entries.len() >= self.max_size && !self.entries.contains_key(&key) {
            let oldest = self
                .entries
                .iter()
                .min_by_key(|(_, e)| e.created_at)
                .map(|(k, _)| k.clone());
            if let Some(k) = oldest {
                self.entries.remove(&k);
            }
        }

        self.entries.insert(
            key,
            CacheEntry {
                response,
                hits: 0,
                created_at: now(),
            },
        );
    }

    /// Clear all entries
    pub fn clear(&mut self) {
        self.entries.clear();
    }

    /// Get cache size
    pub fn size(&self) -> usize {
        self.entries.len()
    }

    /// Get total hits
    pub fn total_hits(&self) -> u64 {
        self.entries.values().map(|e| e.hits).sum()
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
    use std::time::Duration;

    fn make_response() -> LlmResponse {
        LlmResponse {
            text: "test".to_string(),
            tokens_used: 10,
            latency: Duration::from_millis(100),
            model: "test".to_string(),
        }
    }

    #[test]
    fn cache_stores_and_retrieves() {
        let mut cache = ResponseCache::new(100, 60);
        cache.put("key", make_response());

        let retrieved = cache.get("key");
        assert!(retrieved.is_some());
    }

    #[test]
    fn cache_returns_none_for_missing() {
        let mut cache = ResponseCache::new(100, 60);
        let retrieved = cache.get("missing");
        assert!(retrieved.is_none());
    }

    #[test]
    fn cache_tracks_hits() {
        let mut cache = ResponseCache::new(100, 60);
        cache.put("key", make_response());

        cache.get("key");
        cache.get("key");

        assert_eq!(cache.total_hits(), 2);
    }

    #[test]
    fn cache_respects_max_size() {
        let mut cache = ResponseCache::new(2, 60);
        cache.put("a", make_response());
        cache.put("b", make_response());
        cache.put("c", make_response()); // Should evict oldest

        assert_eq!(cache.size(), 2);
    }
}