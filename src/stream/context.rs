use crate::{decrypt::DecryptionKey, Result};
use std::collections::HashMap;
use url::Url;

/// Context for transforming a playlist.
#[derive(Debug, Clone)]
pub struct TransformContext {
    /// Original manifest URL.
    pub original_url: Url,

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
        manifest_headers: Option<String>,
        segment_headers: Option<String>,
        manifest_headers_map: HashMap<String, String>,
        segment_headers_map: HashMap<String, String>,
        decryption_key: Option<DecryptionKey>,
        decrypt_enabled: bool,
    ) -> Self {
        Self {
            original_url,
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

    /// Build a relative URL for the /manifest endpoint.
    pub fn build_manifest_url(&self, target: &Url) -> String {
        let mut params = vec![format!("url={}", urlencoding::encode(target.as_str()))];

        if let Some(h) = &self.manifest_headers {
            params.push(format!("h={}", urlencoding::encode(h)));
        }
        if let Some(sh) = &self.segment_headers {
            params.push(format!("sh={}", urlencoding::encode(sh)));
        }
        if let Some(k) = &self.decryption_key {
            params.push(format!("k={}", urlencoding::encode(&k.to_string())));
        }
        if self.decrypt_enabled {
            params.push("decrypt=true".to_string());
        }

        format!("/manifest?{}", params.join("&"))
    }

    /// Build a relative URL for the /segment endpoint.
    pub fn build_segment_url(
        &self,
        target: &Url,
        method: &str,
        iv: &[u8; 16],
        byterange: Option<&crate::hls::ByteRange>,
        init_url: Option<&Url>,
        init_byterange: Option<&crate::hls::ByteRange>,
    ) -> String {
        // Extract extension from target URL path for player compatibility (e.g., ffplay requires .ts)
        let ext = target
            .path()
            .rsplit_once('/')
            .map(|(_, filename)| filename)
            .unwrap_or(target.path())
            .rsplit_once('.')
            .map(|(_, ext)| ext)
            .unwrap_or("ts");

        let mut params = vec![format!("url={}", urlencoding::encode(target.as_str()))];

        if let Some(sh) = &self.segment_headers {
            params.push(format!("h={}", urlencoding::encode(sh)));
        }
        if let Some(k) = &self.decryption_key {
            params.push(format!("k={}", urlencoding::encode(&k.to_string())));
        }

        params.push(format!("iv={}", hex::encode(iv)));
        params.push(format!("m={}", method));

        if let Some(br) = byterange {
            params.push(format!("br={}", urlencoding::encode(&br.to_query_param())));
        }
        if let Some(init) = init_url {
            params.push(format!("init={}", urlencoding::encode(init.as_str())));
        }
        if let Some(init_br) = init_byterange {
            params.push(format!(
                "init_br={}",
                urlencoding::encode(&init_br.to_query_param())
            ));
        }

        format!("/segment.{}?{}", ext, params.join("&"))
    }

    /// Check if we should intercept and decrypt segments with this key method.
    pub fn should_intercept(&self, requires_server_decrypt: bool) -> bool {
        self.decrypt_enabled && requires_server_decrypt && self.decryption_key.is_some()
    }
}
