use crate::{Error, Result};

/// Represents a byte range for partial content requests.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ByteRange {
    pub length: u64,
    pub offset: Option<u64>,
}

impl ByteRange {
    pub fn new(length: u64, offset: Option<u64>) -> Self {
        Self { length, offset }
    }

    /// Parse from "length@offset" or "length" format.
    pub fn parse(s: &str) -> Result<Self> {
        let s = s.trim();
        if let Some((len, off)) = s.split_once('@') {
            Ok(Self {
                length: len
                    .parse()
                    .map_err(|_| Error::InvalidByteRange(s.to_string()))?,
                offset: Some(
                    off.parse()
                        .map_err(|_| Error::InvalidByteRange(s.to_string()))?,
                ),
            })
        } else {
            Ok(Self {
                length: s
                    .parse()
                    .map_err(|_| Error::InvalidByteRange(s.to_string()))?,
                offset: None,
            })
        }
    }

    /// Parse from #EXT-X-BYTERANGE tag content.
    /// Format: #EXT-X-BYTERANGE:length[@offset]
    pub fn parse_from_tag(line: &str) -> Result<Self> {
        let value = line
            .strip_prefix("#EXT-X-BYTERANGE:")
            .ok_or_else(|| Error::InvalidByteRange(line.to_string()))?;
        Self::parse(value)
    }

    /// Convert to HTTP Range header value.
    pub fn to_range_header(&self) -> String {
        match self.offset {
            Some(offset) => format!("bytes={}-{}", offset, offset + self.length - 1),
            None => format!("bytes=0-{}", self.length - 1),
        }
    }

    /// Convert to query parameter format.
    pub fn to_query_param(&self) -> String {
        match self.offset {
            Some(offset) => format!("{}@{}", self.length, offset),
            None => self.length.to_string(),
        }
    }

    /// Calculate end offset.
    pub fn end_offset(&self) -> Option<u64> {
        self.offset.map(|o| o + self.length)
    }

    /// Update offset based on previous byte range (for continuation).
    pub fn with_continuation(&self, previous_end: u64) -> Self {
        Self {
            length: self.length,
            offset: Some(self.offset.unwrap_or(previous_end)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_with_offset() {
        let br = ByteRange::parse("1000@500").unwrap();
        assert_eq!(br.length, 1000);
        assert_eq!(br.offset, Some(500));
    }

    #[test]
    fn test_parse_without_offset() {
        let br = ByteRange::parse("1000").unwrap();
        assert_eq!(br.length, 1000);
        assert_eq!(br.offset, None);
    }

    #[test]
    fn test_to_range_header() {
        let br = ByteRange::new(1000, Some(500));
        assert_eq!(br.to_range_header(), "bytes=500-1499");
    }

    #[test]
    fn test_to_query_param() {
        let br = ByteRange::new(1000, Some(500));
        assert_eq!(br.to_query_param(), "1000@500");
    }
}
