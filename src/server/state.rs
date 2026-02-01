use crate::{cache::InitSegmentCache, proxy::ProxyClient};
use std::sync::Arc;

/// Shared application state.
#[derive(Clone)]
pub struct AppState {
    pub client: ProxyClient,
    pub init_cache: Arc<InitSegmentCache>,
    pub server_base_url: url::Url,
}

impl AppState {
    pub fn new(server_base_url: url::Url) -> Self {
        Self {
            client: ProxyClient::new(),
            init_cache: Arc::new(InitSegmentCache::new(100)),
            server_base_url,
        }
    }
}
