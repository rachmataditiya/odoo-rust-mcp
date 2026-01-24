use std::collections::HashMap;
use std::time::{Duration, Instant};

use serde_json::Value;
use tokio::sync::RwLock;

/// Cache key: (instance_name, model_name)
type CacheKey = (String, String);

/// Cache entry: (cached_value, expiration_time)
type CacheEntry = (Value, Instant);

/// TTL-based in-memory metadata cache for fields_get results.
/// Uses RwLock for efficient read-heavy workloads.
#[derive(Clone)]
pub struct MetadataCache {
    cache: std::sync::Arc<RwLock<HashMap<CacheKey, CacheEntry>>>,
}

impl MetadataCache {
    /// Create a new empty metadata cache.
    pub fn new() -> Self {
        Self {
            cache: std::sync::Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Get a cached value if it exists and hasn't expired.
    ///
    /// Returns `Some(value)` if cache hit and not expired,
    /// `None` if cache miss or expired.
    pub async fn get(&self, instance: &str, model: &str) -> Option<Value> {
        let key = (instance.to_string(), model.to_string());
        let guard = self.cache.read().await;

        if let Some((value, expiration)) = guard.get(&key) {
            if Instant::now() < *expiration {
                return Some(value.clone());
            }
        }
        None
    }

    /// Insert a value into the cache with TTL.
    ///
    /// `ttl_secs` specifies the time-to-live in seconds.
    pub async fn insert(&self, instance: &str, model: &str, value: Value, ttl_secs: u64) {
        let key = (instance.to_string(), model.to_string());
        let expiration = Instant::now() + Duration::from_secs(ttl_secs);

        let mut guard = self.cache.write().await;
        guard.insert(key, (value, expiration));
    }

    /// Clear expired entries from the cache.
    /// This is optional cleanup; expired entries are still checked on access.
    pub async fn clear_expired(&self) {
        let mut guard = self.cache.write().await;
        let now = Instant::now();
        guard.retain(|_, (_, expiration)| now < *expiration);
    }

    /// Clear all cache entries.
    pub async fn clear_all(&self) {
        let mut guard = self.cache.write().await;
        guard.clear();
    }

    /// Get the current number of entries in the cache (including expired).
    pub async fn len(&self) -> usize {
        let guard = self.cache.read().await;
        guard.len()
    }
}

impl Default for MetadataCache {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_cache_insert_and_get() {
        let cache = MetadataCache::new();
        let value = serde_json::json!({"test": "data"});

        cache.insert("instance1", "model1", value.clone(), 300).await;
        let retrieved = cache.get("instance1", "model1").await;

        assert_eq!(retrieved, Some(value));
    }

    #[tokio::test]
    async fn test_cache_miss() {
        let cache = MetadataCache::new();
        let retrieved = cache.get("instance1", "model1").await;

        assert_eq!(retrieved, None);
    }

    #[tokio::test]
    async fn test_cache_expiration() {
        let cache = MetadataCache::new();
        let value = serde_json::json!({"test": "data"});

        // Insert with 1 second TTL
        cache.insert("instance1", "model1", value.clone(), 1).await;

        // Should be available immediately
        assert_eq!(cache.get("instance1", "model1").await, Some(value.clone()));

        // Wait for expiration
        tokio::time::sleep(Duration::from_secs(2)).await;

        // Should be expired
        assert_eq!(cache.get("instance1", "model1").await, None);
    }

    #[tokio::test]
    async fn test_different_keys() {
        let cache = MetadataCache::new();
        let value1 = serde_json::json!({"test": "data1"});
        let value2 = serde_json::json!({"test": "data2"});

        cache.insert("instance1", "model1", value1.clone(), 300).await;
        cache.insert("instance1", "model2", value2.clone(), 300).await;

        assert_eq!(cache.get("instance1", "model1").await, Some(value1));
        assert_eq!(cache.get("instance1", "model2").await, Some(value2));
    }

    #[tokio::test]
    async fn test_cache_clear_all() {
        let cache = MetadataCache::new();
        let value = serde_json::json!({"test": "data"});

        cache.insert("instance1", "model1", value, 300).await;
        assert_eq!(cache.len().await, 1);

        cache.clear_all().await;
        assert_eq!(cache.len().await, 0);
    }

    #[tokio::test]
    async fn test_cache_clear_expired() {
        let cache = MetadataCache::new();
        let value1 = serde_json::json!({"test": "data1"});
        let value2 = serde_json::json!({"test": "data2"});

        // Insert with short TTL
        cache.insert("instance1", "model1", value1, 1).await;
        // Insert with long TTL
        cache.insert("instance1", "model2", value2, 300).await;

        assert_eq!(cache.len().await, 2);

        // Wait for first to expire
        tokio::time::sleep(Duration::from_secs(2)).await;

        // Clear expired entries
        cache.clear_expired().await;

        // Should only have the non-expired one
        assert_eq!(cache.len().await, 1);
        assert_eq!(cache.get("instance1", "model2").await, Some(serde_json::json!({"test": "data2"})));
    }
}
