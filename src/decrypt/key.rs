use crate::{Error, Result};
use std::collections::HashMap;

/// Represents decryption key(s) provided by the user.
#[derive(Debug, Clone)]
pub enum DecryptionKey {
    /// Single 16-byte key for SAMPLE-AES.
    Single([u8; 16]),
    /// Multiple kid:key pairs for CENC.
    Multi(HashMap<String, [u8; 16]>),
}

impl DecryptionKey {
    /// Parse from query parameter string.
    ///
    /// Formats:
    /// - Single key: `0123456789abcdef0123456789abcdef` (32 hex chars)
    /// - Multi key: `kid1:key1,kid2:key2` (each 32 hex chars)
    pub fn parse(s: &str) -> Result<Self> {
        let s = s.trim();

        if s.contains(':') {
            let mut keys = HashMap::new();
            for pair in s.split(',') {
                let pair = pair.trim();
                let (kid, key) = pair
                    .split_once(':')
                    .ok_or_else(|| Error::InvalidKeyFormat(pair.to_string()))?;
                keys.insert(kid.to_string(), Self::parse_hex_key(key)?);
            }
            Ok(Self::Multi(keys))
        } else {
            Ok(Self::Single(Self::parse_hex_key(s)?))
        }
    }

    /// Parse a 16-byte key from hex string.
    fn parse_hex_key(s: &str) -> Result<[u8; 16]> {
        let s = s.trim();
        let bytes = hex::decode(s)?;
        bytes.try_into().map_err(|_| Error::InvalidKeyLength)
    }

    /// Check if this is a single key.
    pub fn is_single(&self) -> bool {
        matches!(self, Self::Single(_))
    }

    /// Check if this is a multi-key.
    pub fn is_multi(&self) -> bool {
        matches!(self, Self::Multi(_))
    }

    /// Get as single key reference.
    pub fn as_single(&self) -> Option<&[u8; 16]> {
        match self {
            Self::Single(k) => Some(k),
            _ => None,
        }
    }

    /// Get key for a specific KID.
    pub fn get_key_for_kid(&self, kid: &str) -> Option<&[u8; 16]> {
        match self {
            Self::Multi(keys) => keys.get(kid),
            Self::Single(k) => Some(k),
        }
    }

    /// Convert to format expected by mp4decrypt crate (kid -> key as hex strings).
    pub fn to_mp4decrypt_keys(&self) -> Result<HashMap<String, String>> {
        match self {
            Self::Single(k) => {
                // For single key, use empty kid
                let mut map = HashMap::new();
                map.insert(String::new(), hex::encode(k));
                Ok(map)
            }
            Self::Multi(keys) => Ok(keys
                .iter()
                .map(|(kid, key)| (kid.clone(), hex::encode(key)))
                .collect()),
        }
    }

    /// Require single key or error.
    pub fn require_single(&self) -> Result<&[u8; 16]> {
        self.as_single().ok_or(Error::SingleKeyRequired)
    }
}

impl std::fmt::Display for DecryptionKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Single(k) => write!(f, "{}", hex::encode(k)),
            Self::Multi(keys) => {
                let pairs: Vec<_> = keys
                    .iter()
                    .map(|(kid, key)| format!("{}:{}", kid, hex::encode(key)))
                    .collect();
                write!(f, "{}", pairs.join(","))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_single() {
        let key = DecryptionKey::parse("0123456789abcdef0123456789abcdef").unwrap();
        assert!(key.is_single());
    }

    #[test]
    fn test_parse_multi() {
        let key = DecryptionKey::parse(
            "00000000000000000000000000000001:0123456789abcdef0123456789abcdef,00000000000000000000000000000002:fedcba9876543210fedcba9876543210"
        ).unwrap();
        assert!(key.is_multi());
    }

    #[test]
    fn test_parse_invalid_length() {
        let result = DecryptionKey::parse("0123456789abcdef");
        assert!(result.is_err());
    }

    #[test]
    fn test_display_roundtrip() {
        let original = "0123456789abcdef0123456789abcdef";
        let key = DecryptionKey::parse(original).unwrap();
        assert_eq!(key.to_string(), original);
    }
}
