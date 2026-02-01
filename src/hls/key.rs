/// Represents an HLS encryption method.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KeyMethod {
    None,
    Aes128,
    SampleAes,
    SampleAesCtr,
    SampleAesCenc,
    Unknown(String),
}

impl KeyMethod {
    /// Parse from EXT-X-KEY METHOD attribute value.
    pub fn parse(s: &str) -> Self {
        match s.to_uppercase().as_str() {
            "NONE" => Self::None,
            "AES-128" => Self::Aes128,
            "SAMPLE-AES" => Self::SampleAes,
            "SAMPLE-AES-CTR" => Self::SampleAesCtr,
            "SAMPLE-AES-CENC" => Self::SampleAesCenc,
            other => Self::Unknown(other.to_string()),
        }
    }

    /// Returns true if this method requires server-side decryption.
    /// AES-128 is NEVER included - clients handle it natively.
    pub fn requires_server_decrypt(&self) -> bool {
        matches!(
            self,
            Self::SampleAes | Self::SampleAesCtr | Self::SampleAesCenc
        )
    }

    /// Returns true if clients can handle this encryption natively.
    pub fn is_client_supported(&self) -> bool {
        matches!(self, Self::None | Self::Aes128)
    }

    /// Convert to segment endpoint method parameter.
    pub fn to_segment_param(&self) -> Option<&'static str> {
        match self {
            Self::SampleAes => Some("ssa"),
            Self::SampleAesCtr => Some("ssa-ctr"),
            Self::SampleAesCenc => Some("cenc"),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &str {
        match self {
            Self::None => "NONE",
            Self::Aes128 => "AES-128",
            Self::SampleAes => "SAMPLE-AES",
            Self::SampleAesCtr => "SAMPLE-AES-CTR",
            Self::SampleAesCenc => "SAMPLE-AES-CENC",
            Self::Unknown(s) => s.as_str(),
        }
    }
}

/// Represents parsed key information from #EXT-X-KEY tag.
#[derive(Debug, Clone)]
pub struct KeyInfo {
    pub method: KeyMethod,
    pub uri: Option<String>,
    pub iv: Option<[u8; 16]>,
    pub keyformat: Option<String>,
    pub keyformatversions: Option<String>,
}

impl KeyInfo {
    /// Parse from #EXT-X-KEY tag line.
    pub fn parse(line: &str) -> Option<Self> {
        let content = line.strip_prefix("#EXT-X-KEY:")?;

        let mut method = KeyMethod::None;
        let mut uri = None;
        let mut iv = None;
        let mut keyformat = None;
        let mut keyformatversions = None;

        // Simple attribute parser
        for attr in Self::parse_attributes(content) {
            let (key, value) = attr.split_once('=')?;
            let key = key.trim().to_uppercase();
            let value = value.trim().trim_matches('"');

            match key.as_str() {
                "METHOD" => method = KeyMethod::parse(value),
                "URI" => uri = Some(value.to_string()),
                "IV" => iv = Self::parse_iv(value),
                "KEYFORMAT" => keyformat = Some(value.to_string()),
                "KEYFORMATVERSIONS" => keyformatversions = Some(value.to_string()),
                _ => {}
            }
        }

        Some(Self {
            method,
            uri,
            iv,
            keyformat,
            keyformatversions,
        })
    }

    /// Parse IV from hex string (with or without 0x prefix).
    fn parse_iv(s: &str) -> Option<[u8; 16]> {
        let s = s
            .strip_prefix("0x")
            .or_else(|| s.strip_prefix("0X"))
            .unwrap_or(s);
        let bytes = hex::decode(s).ok()?;
        if bytes.len() == 16 {
            let mut arr = [0u8; 16];
            arr.copy_from_slice(&bytes);
            Some(arr)
        } else {
            None
        }
    }

    /// Simple attribute parser that handles quoted values.
    fn parse_attributes(s: &str) -> Vec<&str> {
        let mut attrs = Vec::new();
        let mut start = 0;
        let mut in_quotes = false;

        for (i, c) in s.char_indices() {
            match c {
                '"' => in_quotes = !in_quotes,
                ',' if !in_quotes => {
                    attrs.push(s[start..i].trim());
                    start = i + 1;
                }
                _ => {}
            }
        }

        if start < s.len() {
            attrs.push(s[start..].trim());
        }

        attrs
    }

    /// Check if this key requires server-side decryption.
    pub fn requires_server_decrypt(&self) -> bool {
        self.method.requires_server_decrypt()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_key_method_parse() {
        assert_eq!(KeyMethod::parse("AES-128"), KeyMethod::Aes128);
        assert_eq!(KeyMethod::parse("SAMPLE-AES"), KeyMethod::SampleAes);
        assert_eq!(KeyMethod::parse("aes-128"), KeyMethod::Aes128);
    }

    #[test]
    fn test_key_method_requires_decrypt() {
        assert!(!KeyMethod::None.requires_server_decrypt());
        assert!(!KeyMethod::Aes128.requires_server_decrypt());
        assert!(KeyMethod::SampleAes.requires_server_decrypt());
        assert!(KeyMethod::SampleAesCenc.requires_server_decrypt());
    }

    #[test]
    fn test_key_info_parse() {
        let line = r#"#EXT-X-KEY:METHOD=SAMPLE-AES,URI="https://example.com/key",IV=0x00000000000000000000000000000001"#;
        let info = KeyInfo::parse(line).unwrap();
        assert_eq!(info.method, KeyMethod::SampleAes);
        assert_eq!(info.uri, Some("https://example.com/key".to_string()));
        assert!(info.iv.is_some());
    }
}
