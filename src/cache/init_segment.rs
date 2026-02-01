use bytes::Bytes;
use lru::LruCache;
use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
    num::NonZeroUsize,
    sync::Mutex,
    collections::HashMap,
};

use crate::{hls::ByteRange, proxy::ProxyClient, Result};

/// Cache key for init segments.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct CacheKey {
    url: String,
    headers_hash: u64,
    byterange: Option<(u64, Option<u64>)>,
}

impl CacheKey {
    fn new(url: &str, headers: &HashMap<String, String>, byterange: Option<&ByteRange>) -> Self {
        let mut hasher = DefaultHasher::new();
        for (k, v) in headers {
            k.hash(&mut hasher);
            v.hash(&mut hasher);
        }

        Self {
            url: url.to_string(),
            headers_hash: hasher.finish(),
            byterange: byterange.map(|br| (br.length, br.offset)),
        }
    }
}

/// LRU cache for init segments.
pub struct InitSegmentCache {
    cache: Mutex<LruCache<CacheKey, Bytes>>,
}

impl InitSegmentCache {
    pub fn new(max_entries: usize) -> Self {
        Self {
            cache: Mutex::new(LruCache::new(
                NonZeroUsize::new(max_entries).expect("max_entries must be > 0"),
            )),
        }
    }

    /// Get init segment from cache or fetch from URL.
    pub async fn get_or_fetch(
        &self,
        url: &str,
        headers: &HashMap<String, String>,
        byterange: Option<&ByteRange>,
        client: &ProxyClient,
    ) -> Result<Bytes> {
        let key = CacheKey::new(url, headers, byterange);

        // Check cache first
        {
            let mut cache = self.cache.lock().unwrap();
            if let Some(cached) = cache.get(&key) {
                tracing::debug!("Init segment cache hit: {}", url);
                return Ok(cached.clone());
            }
        }

        // Fetch from URL
        tracing::debug!("Init segment cache miss, fetching: {}", url);
        let bytes = client.fetch(url, Some(headers), byterange).await?;

        // Store in cache
        {
            let mut cache = self.cache.lock().unwrap();
            cache.put(key, bytes.clone());
        }

        Ok(bytes)
    }

    /// Clear the cache.
    pub fn clear(&self) {
        let mut cache = self.cache.lock().unwrap();
        cache.clear();
    }

    /// Get current cache size.
    pub fn len(&self) -> usize {
        let cache = self.cache.lock().unwrap();
        cache.len()
    }

    /// Check if cache is empty.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl Default for InitSegmentCache {
    fn default() -> Self {
        Self::new(100)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_key_hash() {
        let mut headers1 = HashMap::new();
        headers1.insert("Authorization".to_string(), "Bearer token".to_string());

        let mut headers2 = HashMap::new();
        headers2.insert("Authorization".to_string(), "Bearer token".to_string());

        let key1 = CacheKey::new("https://example.com/init.mp4", &headers1, None);
        let key2 = CacheKey::new("https://example.com/init.mp4", &headers2, None);

        // Same headers should produce same key
        assert_eq!(key1, key2);
    }
}
