use crate::hls::{ByteRange, KeyInfo, StreamInfo};

/// Represents the type of playlist being processed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlaylistType {
    Master,
    Media,
}

/// Represents pending context for the next URI.
#[derive(Debug, Clone)]
pub enum PendingContext {
    /// Next URI is a variant playlist.
    VariantStream(StreamInfo),
    /// Next URI is a segment.
    Segment,
}

/// Information about the current init segment (EXT-X-MAP).
#[derive(Debug, Clone)]
pub struct MapInfo {
    pub uri: String,
    pub byterange: Option<ByteRange>,
}

impl MapInfo {
    pub fn parse(line: &str) -> Option<Self> {
        let content = line.strip_prefix("#EXT-X-MAP:")?;

        let mut uri = None;
        let mut byterange = None;

        for attr in Self::parse_attributes(content) {
            if let Some((key, value)) = attr.split_once('=') {
                let key = key.trim().to_uppercase();
                let value = value.trim().trim_matches('"');

                match key.as_str() {
                    "URI" => uri = Some(value.to_string()),
                    "BYTERANGE" => byterange = ByteRange::parse(value).ok(),
                    _ => {}
                }
            }
        }

        uri.map(|uri| Self { uri, byterange })
    }

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
}

/// State maintained during stream processing.
#[derive(Debug, Clone)]
pub struct ProcessorState {
    /// Type of playlist being processed.
    pub playlist_type: Option<PlaylistType>,

    /// Current encryption context.
    pub current_key: Option<KeyInfo>,

    /// Current init segment info.
    pub current_map: Option<MapInfo>,

    /// Media sequence number from #EXT-X-MEDIA-SEQUENCE.
    pub media_sequence: u64,

    /// Current segment index (relative to media_sequence).
    pub segment_index: u64,

    /// Pending context for the next URI.
    pub pending_context: Option<PendingContext>,

    /// Current byte range for the next segment.
    pub current_byterange: Option<ByteRange>,

    /// Last byte range end offset (for continuation).
    pub last_byterange_end: Option<u64>,
}

impl ProcessorState {
    pub fn new() -> Self {
        Self {
            playlist_type: None,
            current_key: None,
            current_map: None,
            media_sequence: 0,
            segment_index: 0,
            pending_context: None,
            current_byterange: None,
            last_byterange_end: None,
        }
    }

    pub fn is_master_playlist(&self) -> bool {
        matches!(self.playlist_type, Some(PlaylistType::Master))
    }

    pub fn is_media_playlist(&self) -> bool {
        matches!(self.playlist_type, Some(PlaylistType::Media))
    }

    /// Calculate IV for current segment (from explicit IV or sequence number).
    pub fn current_iv(&self) -> [u8; 16] {
        self.current_key
            .as_ref()
            .and_then(|k| k.iv)
            .unwrap_or_else(|| self.derive_iv_from_sequence())
    }

    /// Derive IV from media sequence + segment index.
    fn derive_iv_from_sequence(&self) -> [u8; 16] {
        let seq = self.media_sequence + self.segment_index;
        let mut iv = [0u8; 16];
        iv[8..16].copy_from_slice(&seq.to_be_bytes());
        iv
    }

    /// Advance to the next segment.
    pub fn advance_segment(&mut self) {
        self.segment_index += 1;
        self.pending_context = None;

        // Update last byterange end for continuation
        if let Some(br) = &self.current_byterange {
            self.last_byterange_end = br.end_offset();
        }
        self.current_byterange = None;
    }

    pub fn set_pending_variant(&mut self, info: StreamInfo) {
        self.playlist_type = Some(PlaylistType::Master);
        self.pending_context = Some(PendingContext::VariantStream(info));
    }

    pub fn set_pending_segment(&mut self) {
        self.pending_context = Some(PendingContext::Segment);
    }

    pub fn take_pending(&mut self) -> Option<PendingContext> {
        self.pending_context.take()
    }

    pub fn update_key(&mut self, key: KeyInfo) {
        self.current_key = Some(key);
    }

    pub fn update_map(&mut self, map: MapInfo) {
        self.current_map = Some(map);
    }

    pub fn update_media_sequence(&mut self, seq: u64) {
        self.playlist_type = Some(PlaylistType::Media);
        self.media_sequence = seq;
    }

    pub fn set_byterange(&mut self, mut br: ByteRange) {
        // Handle continuation (no offset specified)
        if br.offset.is_none()
            && let Some(end) = self.last_byterange_end
        {
            br.offset = Some(end);
        }
        self.current_byterange = Some(br);
    }

    pub fn take_byterange(&mut self) -> Option<ByteRange> {
        self.current_byterange.take()
    }

    /// Reset key context (e.g., after #EXT-X-DISCONTINUITY).
    pub fn reset_key(&mut self) {
        self.current_key = None;
    }
}

impl Default for ProcessorState {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_derive_iv_from_sequence() {
        let mut state = ProcessorState::new();
        state.media_sequence = 100;
        state.segment_index = 5;

        let iv = state.current_iv();
        let expected_seq: u64 = 105;
        let mut expected = [0u8; 16];
        expected[8..16].copy_from_slice(&expected_seq.to_be_bytes());

        assert_eq!(iv, expected);
    }

    #[test]
    fn test_map_info_parse() {
        let line = r#"#EXT-X-MAP:URI="init.mp4",BYTERANGE="617@0""#;
        let map = MapInfo::parse(line).unwrap();
        assert_eq!(map.uri, "init.mp4");
        assert!(map.byterange.is_some());
        let br = map.byterange.unwrap();
        assert_eq!(br.length, 617);
        assert_eq!(br.offset, Some(0));
    }
}
