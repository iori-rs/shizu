use crate::{Error, Result};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use std::collections::HashMap;

/// Codec for encoding/decoding headers as base64url JSON.
pub struct HeaderCodec;

impl HeaderCodec {
    /// Decode headers from base64url-encoded JSON string.
    pub fn decode(encoded: &str) -> Result<HashMap<String, String>> {
        let json_bytes = URL_SAFE_NO_PAD
            .decode(encoded)
            .map_err(|e| Error::InvalidHeaderEncoding(e.to_string()))?;

        serde_json::from_slice(&json_bytes).map_err(|e| Error::InvalidHeaderEncoding(e.to_string()))
    }

    /// Encode headers to base64url-encoded JSON string.
    pub fn encode(headers: &HashMap<String, String>) -> Result<String> {
        let json =
            serde_json::to_vec(headers).map_err(|e| Error::InvalidHeaderEncoding(e.to_string()))?;
        Ok(URL_SAFE_NO_PAD.encode(&json))
    }

    /// Decode headers from optional parameter, returning empty map if None.
    pub fn decode_optional(encoded: Option<&str>) -> Result<HashMap<String, String>> {
        match encoded {
            Some(s) if !s.is_empty() => Self::decode(s),
            _ => Ok(HashMap::new()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_decode_roundtrip() {
        let mut headers = HashMap::new();
        headers.insert("Authorization".to_string(), "Bearer token123".to_string());
        headers.insert("Cookie".to_string(), "session=abc".to_string());

        let encoded = HeaderCodec::encode(&headers).unwrap();
        let decoded = HeaderCodec::decode(&encoded).unwrap();

        assert_eq!(headers, decoded);
    }

    #[test]
    fn test_decode_optional_none() {
        let result = HeaderCodec::decode_optional(None).unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn test_decode_optional_empty() {
        let result = HeaderCodec::decode_optional(Some("")).unwrap();
        assert!(result.is_empty());
    }
}
