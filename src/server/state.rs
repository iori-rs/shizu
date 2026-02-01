use crate::{cache::InitSegmentCache, proxy::ProxyClient};
use std::sync::Arc;

use super::signature::SigningKey;

/// Shared application state.
#[derive(Clone)]
pub struct AppState {
    pub client: ProxyClient,
    pub init_cache: Arc<InitSegmentCache>,
    pub signing_key: SigningKey,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            client: ProxyClient::new(),
            init_cache: Arc::new(InitSegmentCache::new(100)),
            signing_key: SigningKey::from_env(),
        }
    }

    /// Verify that a URL has a valid signature.
    pub fn verify_signature(&self, url: &str, signature: Option<&str>) -> bool {
        self.signing_key.verify(url, signature)
    }

    /// Sign a URL and return the signature.
    pub fn sign_url(&self, url: &str) -> String {
        self.signing_key.sign(url)
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}
