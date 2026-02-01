use crate::{cache::InitSegmentCache, proxy::ProxyClient};
use std::sync::Arc;

/// Shared application state.
#[derive(Clone)]
pub struct AppState {
    pub client: ProxyClient,
    pub init_cache: Arc<InitSegmentCache>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            client: ProxyClient::new(),
            init_cache: Arc::new(InitSegmentCache::new(100)),
        }
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}
