use crate::{hls::ByteRange, Result};
use bytes::Bytes;
use reqwest::Client;
use std::collections::HashMap;

/// HTTP client for proxying requests to upstream servers.
#[derive(Clone)]
pub struct ProxyClient {
    client: Client,
}

impl ProxyClient {
    pub fn new() -> Self {
        Self {
            client: Client::builder()
                .timeout(std::time::Duration::from_secs(30))
                .build()
                .expect("Failed to create HTTP client"),
        }
    }

    pub fn with_client(client: Client) -> Self {
        Self { client }
    }

    /// Fetch content from a URL with optional headers and byte range.
    pub async fn fetch(
        &self,
        url: &str,
        headers: Option<&HashMap<String, String>>,
        byterange: Option<&ByteRange>,
    ) -> Result<Bytes> {
        let mut request = self.client.get(url);

        // Apply custom headers
        if let Some(headers) = headers {
            for (key, value) in headers {
                request = request.header(key.as_str(), value.as_str());
            }
        }

        // Apply byte range header
        if let Some(br) = byterange {
            request = request.header("Range", br.to_range_header());
        }

        let response = request.send().await?;
        let status = response.status();

        if !status.is_success() && status.as_u16() != 206 {
            return Err(crate::Error::FetchFailed {
                url: url.to_string(),
                reason: format!("HTTP {}", status),
            });
        }

        Ok(response.bytes().await?)
    }

    /// Fetch content and return as string.
    pub async fn fetch_text(
        &self,
        url: &str,
        headers: Option<&HashMap<String, String>>,
    ) -> Result<String> {
        let bytes = self.fetch(url, headers, None).await?;
        String::from_utf8(bytes.to_vec()).map_err(|e| crate::Error::FetchFailed {
            url: url.to_string(),
            reason: format!("Invalid UTF-8: {}", e),
        })
    }
}

impl Default for ProxyClient {
    fn default() -> Self {
        Self::new()
    }
}
