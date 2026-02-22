use bytes::Bytes;
use lru::LruCache;
use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
    num::NonZeroUsize,
    sync::Mutex,
    collections::HashMap,
};

use crate::{Result, hls::ByteRange, proxy::ProxyClient};

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
    decrypted_cache: Mutex<LruCache<u64, Bytes>>,
}

impl InitSegmentCache {
    pub fn new(max_entries: usize) -> Self {
        let cap = NonZeroUsize::new(max_entries).expect("max_entries must be > 0");
        Self {
            cache: Mutex::new(LruCache::new(cap)),
            decrypted_cache: Mutex::new(LruCache::new(cap)),
        }
    }

    /// Look up a URL in the cache without fetching.
    pub fn get(
        &self,
        url: &str,
        headers: &HashMap<String, String>,
        byterange: Option<&ByteRange>,
    ) -> Option<Bytes> {
        let key = CacheKey::new(url, headers, byterange);
        self.cache.lock().unwrap().get(&key).cloned()
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

    /// Fetch the init segment (caching the raw bytes) and also return its decrypted form
    /// (caching that too), in a single call.
    pub async fn get_or_fetch_with_decrypted(
        &self,
        url: &str,
        headers: &HashMap<String, String>,
        byterange: Option<&ByteRange>,
        key_str: &str,
        client: &ProxyClient,
        decrypt: impl FnOnce(&Bytes) -> Result<Bytes>,
    ) -> Result<(Bytes, Bytes)> {
        let raw = self.get_or_fetch(url, headers, byterange, client).await?;
        let decrypted = self.get_or_compute_decrypted_init(&raw, key_str, || decrypt(&raw))?;
        Ok((raw, decrypted))
    }

    /// Get decrypted init segment from cache, or compute and store it.
    pub fn get_or_compute_decrypted_init(
        &self,
        raw_init: &Bytes,
        key_str: &str,
        compute: impl FnOnce() -> Result<Bytes>,
    ) -> Result<Bytes> {
        let hash = {
            let mut hasher = DefaultHasher::new();
            raw_init.as_ref().hash(&mut hasher);
            key_str.hash(&mut hasher);
            hasher.finish()
        };

        {
            let mut cache = self.decrypted_cache.lock().unwrap();
            if let Some(cached) = cache.get(&hash) {
                tracing::debug!("Decrypted init segment cache hit");
                return Ok(cached.clone());
            }
        }

        let decrypted = compute()?;

        {
            let mut cache = self.decrypted_cache.lock().unwrap();
            cache.put(hash, decrypted.clone());
        }

        Ok(decrypted)
    }

    /// Clear the cache.
    pub fn clear(&self) {
        self.cache.lock().unwrap().clear();
        self.decrypted_cache.lock().unwrap().clear();
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
