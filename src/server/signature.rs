//! URL signature verification to prevent SSRF attacks.
//!
//! This module provides HMAC-SHA256 based URL signing and verification.
//! Only URLs signed with the server's secret key can be fetched.
//!
//! If no signing key is configured, signature validation is bypassed
//! (with a warning logged at startup).

use hmac::{Hmac, Mac};
use sha2::Sha256;
use std::sync::Arc;

type HmacSha256 = Hmac<Sha256>;

/// Optional signing key for URL verification.
/// When `None`, signature validation is bypassed.
#[derive(Clone)]
pub struct SigningKey {
    key: Option<Arc<[u8]>>,
}

impl std::fmt::Debug for SigningKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SigningKey")
            .field(
                "key",
                &if self.key.is_some() {
                    "[REDACTED]"
                } else {
                    "[DISABLED]"
                },
            )
            .finish()
    }
}

impl SigningKey {
    /// Create a new signing key from bytes.
    pub fn new(key: impl Into<Vec<u8>>) -> Self {
        Self {
            key: Some(key.into().into()),
        }
    }

    /// Create a disabled signing key (bypasses validation).
    pub fn disabled() -> Self {
        Self { key: None }
    }

    /// Create a test signing key (for testing only).
    #[cfg(test)]
    pub fn test_key() -> Self {
        Self::new(b"test-signing-key-for-tests".to_vec())
    }

    /// Create a signing key from the SHIZU_SIGNING_KEY environment variable.
    /// If not set, returns a disabled key and logs a warning.
    pub fn from_env() -> Self {
        if let Ok(key) = std::env::var("SHIZU_SIGNING_KEY") {
            if key.is_empty() {
                tracing::warn!("SHIZU_SIGNING_KEY is empty, signature validation is DISABLED");
                tracing::warn!("This server is vulnerable to SSRF attacks!");
                return Self::disabled();
            }
            // Try to decode as hex first, fall back to using the string as bytes
            let key_bytes = hex::decode(&key).unwrap_or_else(|_| key.into_bytes());
            tracing::info!("URL signature validation is enabled");
            Self::new(key_bytes)
        } else {
            tracing::warn!("SHIZU_SIGNING_KEY is not set, signature validation is DISABLED");
            tracing::warn!("This server is vulnerable to SSRF attacks!");
            tracing::warn!(
                "Set SHIZU_SIGNING_KEY environment variable to enable signature validation"
            );
            Self::disabled()
        }
    }

    /// Check if signature validation is enabled.
    pub fn is_enabled(&self) -> bool {
        self.key.is_some()
    }

    /// Sign a URL and return the signature as a hex string.
    /// Returns an empty string if signing is disabled.
    pub fn sign(&self, url: &str) -> String {
        let Some(key) = &self.key else {
            return String::new();
        };

        let mut mac = HmacSha256::new_from_slice(key).expect("HMAC can take key of any size");
        mac.update(url.as_bytes());
        let result = mac.finalize();
        hex::encode(result.into_bytes())
    }

    /// Verify a URL signature.
    /// Returns `true` if signing is disabled (bypass mode).
    pub fn verify(&self, url: &str, signature: Option<&str>) -> bool {
        let Some(key) = &self.key else {
            // Signing disabled, bypass validation
            return true;
        };

        let Some(sig) = signature else {
            // Signing enabled but no signature provided
            return false;
        };

        let Ok(sig_bytes) = hex::decode(sig) else {
            return false;
        };

        let mut mac = HmacSha256::new_from_slice(key).expect("HMAC can take key of any size");
        mac.update(url.as_bytes());

        mac.verify_slice(&sig_bytes).is_ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sign_and_verify() {
        let key = SigningKey::new(b"test-secret-key".to_vec());
        let url = "https://example.com/manifest.m3u8";

        let signature = key.sign(url);
        assert!(key.verify(url, Some(&signature)));
    }

    #[test]
    fn test_verify_invalid_signature() {
        let key = SigningKey::new(b"test-secret-key".to_vec());
        let url = "https://example.com/manifest.m3u8";

        assert!(!key.verify(url, Some("invalid-signature")));
    }

    #[test]
    fn test_verify_missing_signature() {
        let key = SigningKey::new(b"test-secret-key".to_vec());
        let url = "https://example.com/manifest.m3u8";

        assert!(!key.verify(url, None));
    }

    #[test]
    fn test_verify_wrong_url() {
        let key = SigningKey::new(b"test-secret-key".to_vec());
        let url1 = "https://example.com/manifest.m3u8";
        let url2 = "https://evil.com/manifest.m3u8";

        let signature = key.sign(url1);
        assert!(!key.verify(url2, Some(&signature)));
    }

    #[test]
    fn test_different_keys_produce_different_signatures() {
        let key1 = SigningKey::new(b"key1".to_vec());
        let key2 = SigningKey::new(b"key2".to_vec());
        let url = "https://example.com/manifest.m3u8";

        let sig1 = key1.sign(url);
        let sig2 = key2.sign(url);

        assert_ne!(sig1, sig2);
        assert!(!key2.verify(url, Some(&sig1)));
    }

    #[test]
    fn test_disabled_key_bypasses_validation() {
        let key = SigningKey::disabled();
        let url = "https://example.com/manifest.m3u8";

        // Disabled key always returns true for verify
        assert!(key.verify(url, Some("any-signature")));
        assert!(key.verify(url, None));

        // Sign returns empty string when disabled
        assert_eq!(key.sign(url), "");
    }

    #[test]
    fn test_is_enabled() {
        let enabled = SigningKey::new(b"key".to_vec());
        let disabled = SigningKey::disabled();

        assert!(enabled.is_enabled());
        assert!(!disabled.is_enabled());
    }
}
