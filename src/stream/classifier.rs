/// Represents the type of a line in an M3U8 playlist.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LineType {
    Empty,
    ExtM3U,
    ExtXStreamInf,
    ExtXMedia,
    ExtXKey,
    ExtXMap,
    ExtXMediaSequence,
    ExtInf,
    ExtXByteRange,
    ExtXIFrameStreamInf,
    ExtXDiscontinuity,
    ExtXDiscontinuitySequence,
    ExtXEndList,
    ExtXTargetDuration,
    ExtXPlaylistType,
    ExtXVersion,
    UnknownExtTag,
    Comment,
    Uri,
}

impl LineType {
    pub fn is_tag(&self) -> bool {
        !matches!(self, Self::Empty | Self::Uri | Self::Comment)
    }

    pub fn is_uri(&self) -> bool {
        matches!(self, Self::Uri)
    }

    pub fn affects_state(&self) -> bool {
        matches!(
            self,
            Self::ExtXKey
                | Self::ExtXMediaSequence
                | Self::ExtXByteRange
                | Self::ExtInf
                | Self::ExtXStreamInf
                | Self::ExtXMap
                | Self::ExtXDiscontinuity
                | Self::ExtXDiscontinuitySequence
        )
    }

    pub fn signals_next_uri_is_variant(&self) -> bool {
        matches!(self, Self::ExtXStreamInf)
    }

    pub fn signals_next_uri_is_segment(&self) -> bool {
        matches!(self, Self::ExtInf)
    }
}

/// Classifier for M3U8 lines.
pub struct LineClassifier;

impl LineClassifier {
    /// Classify a line from an M3U8 playlist.
    pub fn classify(line: &str) -> LineType {
        let line = line.trim();

        if line.is_empty() {
            return LineType::Empty;
        }

        if !line.starts_with('#') {
            return LineType::Uri;
        }

        // Fast prefix matching for known tags
        if line.starts_with("#EXTM3U") {
            LineType::ExtM3U
        } else if line.starts_with("#EXT-X-STREAM-INF:") {
            LineType::ExtXStreamInf
        } else if line.starts_with("#EXT-X-MEDIA:") {
            LineType::ExtXMedia
        } else if line.starts_with("#EXT-X-KEY:") {
            LineType::ExtXKey
        } else if line.starts_with("#EXT-X-MAP:") {
            LineType::ExtXMap
        } else if line.starts_with("#EXT-X-MEDIA-SEQUENCE:") {
            LineType::ExtXMediaSequence
        } else if line.starts_with("#EXTINF:") {
            LineType::ExtInf
        } else if line.starts_with("#EXT-X-BYTERANGE:") {
            LineType::ExtXByteRange
        } else if line.starts_with("#EXT-X-I-FRAME-STREAM-INF:") {
            LineType::ExtXIFrameStreamInf
        } else if line.starts_with("#EXT-X-DISCONTINUITY-SEQUENCE:") {
            LineType::ExtXDiscontinuitySequence
        } else if line.starts_with("#EXT-X-DISCONTINUITY") {
            LineType::ExtXDiscontinuity
        } else if line.starts_with("#EXT-X-ENDLIST") {
            LineType::ExtXEndList
        } else if line.starts_with("#EXT-X-TARGETDURATION:") {
            LineType::ExtXTargetDuration
        } else if line.starts_with("#EXT-X-PLAYLIST-TYPE:") {
            LineType::ExtXPlaylistType
        } else if line.starts_with("#EXT-X-VERSION:") {
            LineType::ExtXVersion
        } else if line.starts_with("#EXT") {
            LineType::UnknownExtTag
        } else {
            LineType::Comment
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_classify_extm3u() {
        assert_eq!(LineClassifier::classify("#EXTM3U"), LineType::ExtM3U);
    }

    #[test]
    fn test_classify_stream_inf() {
        assert_eq!(
            LineClassifier::classify("#EXT-X-STREAM-INF:BANDWIDTH=1000000"),
            LineType::ExtXStreamInf
        );
    }

    #[test]
    fn test_classify_uri() {
        assert_eq!(
            LineClassifier::classify("https://example.com/playlist.m3u8"),
            LineType::Uri
        );
        assert_eq!(LineClassifier::classify("segment001.ts"), LineType::Uri);
    }

    #[test]
    fn test_classify_comment() {
        assert_eq!(
            LineClassifier::classify("# This is a comment"),
            LineType::Comment
        );
    }

    #[test]
    fn test_classify_unknown_ext() {
        assert_eq!(
            LineClassifier::classify("#EXT-X-CUSTOM-TAG:value"),
            LineType::UnknownExtTag
        );
    }

    #[test]
    fn test_classify_empty() {
        assert_eq!(LineClassifier::classify(""), LineType::Empty);
        assert_eq!(LineClassifier::classify("  "), LineType::Empty);
    }
}
