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

        if let Some((value, expiration)) = guard.get(&key)
            && Instant::now() < *expiration
        {
            return Some(value.clone());
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

    /// Check if the cache is empty.
    pub async fn is_empty(&self) -> bool {
        let guard = self.cache.read().await;
        guard.is_empty()
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

        cache
            .insert("instance1", "model1", value.clone(), 300)
            .await;
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

        cache
            .insert("instance1", "model1", value1.clone(), 300)
            .await;
        cache
            .insert("instance1", "model2", value2.clone(), 300)
            .await;

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
        assert_eq!(
            cache.get("instance1", "model2").await,
            Some(serde_json::json!({"test": "data2"}))
        );
    }

    #[tokio::test]
    async fn test_cache_multiple_instances() {
        let cache = MetadataCache::new();
        let value1 = serde_json::json!({"instance": 1});
        let value2 = serde_json::json!({"instance": 2});

        cache
            .insert("instance1", "model", value1.clone(), 300)
            .await;
        cache
            .insert("instance2", "model", value2.clone(), 300)
            .await;

        assert_eq!(cache.get("instance1", "model").await, Some(value1));
        assert_eq!(cache.get("instance2", "model").await, Some(value2));
        assert_eq!(cache.len().await, 2);
    }

    #[tokio::test]
    async fn test_cache_same_model_different_instances() {
        let cache = MetadataCache::new();
        let value = serde_json::json!({"data": "test"});

        // Same model name, different instances
        cache
            .insert("prod", "res.partner", value.clone(), 300)
            .await;
        cache
            .insert("staging", "res.partner", value.clone(), 300)
            .await;

        assert_eq!(cache.get("prod", "res.partner").await, Some(value.clone()));
        assert_eq!(
            cache.get("staging", "res.partner").await,
            Some(value.clone())
        );
    }

    #[tokio::test]
    async fn test_cache_overwrite_existing() {
        let cache = MetadataCache::new();
        let value1 = serde_json::json!({"version": 1});
        let value2 = serde_json::json!({"version": 2});

        cache
            .insert("instance1", "model1", value1.clone(), 300)
            .await;
        assert_eq!(cache.get("instance1", "model1").await, Some(value1));

        cache
            .insert("instance1", "model1", value2.clone(), 300)
            .await;
        assert_eq!(cache.get("instance1", "model1").await, Some(value2));
    }

    #[tokio::test]
    async fn test_cache_large_json_value() {
        let cache = MetadataCache::new();
        let large_value = serde_json::json!({
            "fields": {
                "name": {"type": "char", "string": "Name"},
                "email": {"type": "char", "string": "Email"},
                "active": {"type": "boolean", "string": "Active"},
                "phone": {"type": "char", "string": "Phone"},
            },
            "metadata": {
                "description": "A test model with multiple fields"
            }
        });

        cache
            .insert("instance1", "model1", large_value.clone(), 300)
            .await;
        assert_eq!(cache.get("instance1", "model1").await, Some(large_value));
    }

    #[tokio::test]
    async fn test_cache_default_constructor() {
        let cache = MetadataCache::default();
        assert_eq!(cache.len().await, 0);
    }

    #[tokio::test]
    async fn test_cache_clone_independent() {
        let cache1 = MetadataCache::new();
        let cache2 = cache1.clone();
        let value = serde_json::json!({"test": "data"});

        cache1
            .insert("instance1", "model1", value.clone(), 300)
            .await;

        // Both should have the value (they share the same Arc)
        assert_eq!(cache1.get("instance1", "model1").await, Some(value.clone()));
        assert_eq!(cache2.get("instance1", "model1").await, Some(value));
    }

    #[tokio::test]
    async fn test_cache_special_characters_in_keys() {
        let cache = MetadataCache::new();
        let value = serde_json::json!({"test": "data"});

        // Test with special characters in instance and model names
        cache
            .insert("prod-db_v2", "module.model.subtype", value.clone(), 300)
            .await;
        assert_eq!(
            cache.get("prod-db_v2", "module.model.subtype").await,
            Some(value)
        );
    }

    #[tokio::test]
    async fn test_cache_zero_ttl() {
        let cache = MetadataCache::new();
        let value = serde_json::json!({"test": "data"});

        // Insert with 0 second TTL - should expire immediately
        cache.insert("instance1", "model1", value, 0).await;

        // A tiny sleep to ensure time passes
        tokio::time::sleep(Duration::from_millis(1)).await;

        // Should be expired
        assert_eq!(cache.get("instance1", "model1").await, None);
    }

    #[tokio::test]
    async fn test_cache_concurrent_reads() {
        let cache = MetadataCache::new();
        let value = serde_json::json!({"test": "data"});

        cache
            .insert("instance1", "model1", value.clone(), 300)
            .await;

        let cache1 = cache.clone();
        let cache2 = cache.clone();
        let cache3 = cache.clone();

        let handles = vec![
            tokio::spawn(async move { cache1.get("instance1", "model1").await }),
            tokio::spawn(async move { cache2.get("instance1", "model1").await }),
            tokio::spawn(async move { cache3.get("instance1", "model1").await }),
        ];

        for handle in handles {
            let result = handle.await.unwrap();
            assert_eq!(result, Some(value.clone()));
        }
    }

    #[tokio::test]
    async fn test_cache_concurrent_writes() {
        let cache = MetadataCache::new();

        let handles: Vec<_> = (0..5)
            .map(|i| {
                let cache_clone = cache.clone();
                tokio::spawn(async move {
                    let value = serde_json::json!({"id": i});
                    cache_clone
                        .insert("instance", &format!("model{}", i), value, 300)
                        .await;
                })
            })
            .collect();

        for handle in handles {
            handle.await.unwrap();
        }

        assert_eq!(cache.len().await, 5);
    }

    #[tokio::test]
    async fn test_cache_json_null_value() {
        let cache = MetadataCache::new();
        let null_value = serde_json::json!(null);

        cache
            .insert("instance1", "model1", null_value.clone(), 300)
            .await;
        assert_eq!(cache.get("instance1", "model1").await, Some(null_value));
    }

    #[tokio::test]
    async fn test_cache_empty_json_object() {
        let cache = MetadataCache::new();
        let empty_value = serde_json::json!({});

        cache
            .insert("instance1", "model1", empty_value.clone(), 300)
            .await;
        assert_eq!(cache.get("instance1", "model1").await, Some(empty_value));
    }
}
