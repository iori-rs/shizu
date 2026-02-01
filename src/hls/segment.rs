use crate::{Error, Result};

/// Represents the format of a media segment.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SegmentFormat {
    MpegTS,
    Mp4,
    Aac,
    Unknown,
}

impl SegmentFormat {
    /// Parse from format parameter string.
    pub fn parse(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "ts" => Self::MpegTS,
            "mp4" | "m4s" | "m4f" | "cmfv" | "cmfa" => Self::Mp4,
            "aac" | "m4a" => Self::Aac,
            _ => Self::Unknown,
        }
    }

    /// Detect format from file extension.
    /// Returns an error for unknown extensions.
    pub fn from_extension(ext: &str) -> Result<Self> {
        match ext.to_lowercase().as_str() {
            "ts" => Ok(Self::MpegTS),
            "mp4" | "m4s" | "m4f" | "cmfv" | "cmfa" => Ok(Self::Mp4),
            "aac" | "m4a" => Ok(Self::Aac),
            _ => Err(Error::UnknownSegmentFormat(ext.to_string())),
        }
    }

    /// Detect format from URL/filename.
    pub fn from_url(url: &str) -> Self {
        let path = url.split('?').next().unwrap_or(url);
        let ext = path.rsplit('.').next().unwrap_or("");

        match ext.to_lowercase().as_str() {
            "ts" => Self::MpegTS,
            "mp4" | "m4s" | "m4f" | "cmfv" | "cmfa" => Self::Mp4,
            "aac" | "m4a" => Self::Aac,
            _ => Self::Unknown,
        }
    }

    /// Detect format from content bytes (magic bytes).
    pub fn from_bytes(data: &[u8]) -> Self {
        if data.len() < 4 {
            return Self::Unknown;
        }

        // MPEG-TS sync byte
        if data[0] == 0x47 {
            return Self::MpegTS;
        }

        // ftyp box (MP4/fMP4)
        if data.len() >= 8 && &data[4..8] == b"ftyp" {
            return Self::Mp4;
        }

        // AAC ADTS sync word
        if data.len() >= 2 && data[0] == 0xFF && (data[1] & 0xF0) == 0xF0 {
            return Self::Aac;
        }

        Self::Unknown
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::MpegTS => "ts",
            Self::Mp4 => "mp4",
            Self::Aac => "aac",
            Self::Unknown => "unknown",
        }
    }

    /// Get the appropriate Content-Type header value.
    pub fn content_type(&self) -> &'static str {
        match self {
            Self::MpegTS => "video/mp2t",
            Self::Mp4 => "video/mp4",
            Self::Aac => "audio/aac",
            Self::Unknown => "application/octet-stream",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse() {
        assert_eq!(SegmentFormat::parse("ts"), SegmentFormat::MpegTS);
        assert_eq!(SegmentFormat::parse("mp4"), SegmentFormat::Mp4);
        assert_eq!(SegmentFormat::parse("m4s"), SegmentFormat::Mp4);
    }

    #[test]
    fn test_from_url() {
        assert_eq!(
            SegmentFormat::from_url("https://example.com/segment.ts"),
            SegmentFormat::MpegTS
        );
        assert_eq!(
            SegmentFormat::from_url("https://example.com/segment.m4s?token=abc"),
            SegmentFormat::Mp4
        );
    }

    #[test]
    fn test_from_bytes() {
        assert_eq!(
            SegmentFormat::from_bytes(&[0x47, 0x00, 0x00, 0x00]),
            SegmentFormat::MpegTS
        );
        assert_eq!(
            SegmentFormat::from_bytes(&[0x00, 0x00, 0x00, 0x20, b'f', b't', b'y', b'p']),
            SegmentFormat::Mp4
        );
    }
}
