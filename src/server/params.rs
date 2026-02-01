use serde::Deserialize;

/// Query parameters for the /manifest endpoint.
#[derive(Debug, Deserialize)]
pub struct ManifestParams {
    /// URL of the M3U8 manifest.
    pub url: String,

    /// Base64url-encoded JSON headers for manifest fetch.
    #[serde(default)]
    pub h: Option<String>,

    /// Base64url-encoded JSON headers for segment fetch.
    #[serde(default)]
    pub sh: Option<String>,

    /// Decryption key(s) in hex format.
    #[serde(default)]
    pub k: Option<String>,

    /// Whether to decrypt DRM segments.
    #[serde(default)]
    pub decrypt: Option<bool>,

    /// HMAC-SHA256 signature of the URL (hex encoded).
    /// Required when SHIZU_SIGNING_KEY is set to prevent SSRF attacks.
    #[serde(default)]
    pub sig: Option<String>,
}

/// Query parameters for the /segment endpoint.
#[derive(Debug, Deserialize)]
pub struct SegmentParams {
    /// URL of the segment.
    pub url: String,

    /// Base64url-encoded JSON headers.
    #[serde(default)]
    pub h: Option<String>,

    /// Decryption key in hex format.
    pub k: String,

    /// IV in hex format.
    #[serde(default)]
    pub iv: Option<String>,

    /// Decryption method: ssa, ssa-ctr, cenc.
    pub m: String,

    /// Init segment URL (for fMP4).
    #[serde(default)]
    pub init: Option<String>,

    /// Byte range: length@offset.
    #[serde(default)]
    pub br: Option<String>,

    /// Init segment byte range.
    #[serde(default)]
    pub init_br: Option<String>,

    /// HMAC-SHA256 signature of the URL (hex encoded).
    /// Required when SHIZU_SIGNING_KEY is set to prevent SSRF attacks.
    #[serde(default)]
    pub sig: Option<String>,
}
