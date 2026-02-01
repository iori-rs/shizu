use crate::{decrypt::DecryptionKey, Result};
use std::collections::HashMap;
use url::Url;

/// Context for transforming a playlist.
#[derive(Debug, Clone)]
pub struct TransformContext {
    /// Original manifest URL.
    pub original_url: Url,

    /// Base URL for this server (for rewriting URLs).
    pub server_base_url: Url,

    /// Headers for fetching manifests (base64-encoded).
    pub manifest_headers: Option<String>,

    /// Headers for fetching segments (base64-encoded).
    pub segment_headers: Option<String>,

    /// Decoded manifest headers.
    pub manifest_headers_map: HashMap<String, String>,

    /// Decoded segment headers.
    pub segment_headers_map: HashMap<String, String>,

    /// Decryption key(s) if provided.
    pub decryption_key: Option<DecryptionKey>,

    /// Whether to decrypt DRM segments.
    pub decrypt_enabled: bool,
}

impl TransformContext {
    pub fn new(
        original_url: Url,
        server_base_url: Url,
        manifest_headers: Option<String>,
        segment_headers: Option<String>,
        manifest_headers_map: HashMap<String, String>,
        segment_headers_map: HashMap<String, String>,
        decryption_key: Option<DecryptionKey>,
        decrypt_enabled: bool,
    ) -> Self {
        Self {
            original_url,
            server_base_url,
            manifest_headers,
            segment_headers,
            manifest_headers_map,
            segment_headers_map,
            decryption_key,
            decrypt_enabled,
        }
    }

    /// Resolve a relative URL against the original manifest URL.
    pub fn resolve_url(&self, relative: &str) -> Result<Url> {
        self.original_url.join(relative).map_err(Into::into)
    }

    /// Build a URL for the /manifest endpoint.
    pub fn build_manifest_url(&self, target: &Url) -> Url {
        let mut url = self.server_base_url.join("/manifest").unwrap();
        {
            let mut query = url.query_pairs_mut();
            query.append_pair("url", target.as_str());

            if let Some(h) = &self.manifest_headers {
                query.append_pair("h", h);
            }
            if let Some(sh) = &self.segment_headers {
                query.append_pair("sh", sh);
            }
            if let Some(k) = &self.decryption_key {
                query.append_pair("k", &k.to_string());
            }
            if self.decrypt_enabled {
                query.append_pair("decrypt", "true");
            }
        }
        url
    }

    /// Build a URL for the /segment endpoint.
    pub fn build_segment_url(
        &self,
        target: &Url,
        method: &str,
        iv: &[u8; 16],
        byterange: Option<&crate::hls::ByteRange>,
        init_url: Option<&Url>,
        init_byterange: Option<&crate::hls::ByteRange>,
    ) -> Url {
        // Extract extension from target URL path for player compatibility (e.g., ffplay requires .ts)
        let ext = target
            .path()
            .rsplit_once('/')
            .map(|(_, filename)| filename)
            .unwrap_or(target.path())
            .rsplit_once('.')
            .map(|(_, ext)| ext)
            .unwrap_or("ts");
        let endpoint = format!("/segment.{}", ext);
        let mut url = self.server_base_url.join(&endpoint).unwrap();
        {
            let mut query = url.query_pairs_mut();
            query.append_pair("url", target.as_str());

            if let Some(sh) = &self.segment_headers {
                query.append_pair("h", sh);
            }
            if let Some(k) = &self.decryption_key {
                query.append_pair("k", &k.to_string());
            }

            query.append_pair("iv", &hex::encode(iv));
            query.append_pair("m", method);

            if let Some(br) = byterange {
                query.append_pair("br", &br.to_query_param());
            }
            if let Some(init) = init_url {
                query.append_pair("init", init.as_str());
            }
            if let Some(init_br) = init_byterange {
                query.append_pair("init_br", &init_br.to_query_param());
            }
        }
        url
    }

    /// Check if we should intercept and decrypt segments with this key method.
    pub fn should_intercept(&self, requires_server_decrypt: bool) -> bool {
        self.decrypt_enabled && requires_server_decrypt && self.decryption_key.is_some()
    }
}
