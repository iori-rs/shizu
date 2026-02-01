/// Represents parsed stream information from #EXT-X-STREAM-INF tag.
#[derive(Debug, Clone, Default)]
pub struct StreamInfo {
    pub bandwidth: Option<u64>,
    pub average_bandwidth: Option<u64>,
    pub resolution: Option<(u32, u32)>,
    pub codecs: Option<String>,
    pub frame_rate: Option<f64>,
    pub audio: Option<String>,
    pub video: Option<String>,
    pub subtitles: Option<String>,
    pub closed_captions: Option<String>,
}

impl StreamInfo {
    /// Parse from #EXT-X-STREAM-INF tag line.
    pub fn parse(line: &str) -> Self {
        let content = match line.strip_prefix("#EXT-X-STREAM-INF:") {
            Some(c) => c,
            None => return Self::default(),
        };

        let mut info = Self::default();

        for attr in Self::parse_attributes(content) {
            if let Some((key, value)) = attr.split_once('=') {
                let key = key.trim().to_uppercase();
                let value = value.trim().trim_matches('"');

                match key.as_str() {
                    "BANDWIDTH" => info.bandwidth = value.parse().ok(),
                    "AVERAGE-BANDWIDTH" => info.average_bandwidth = value.parse().ok(),
                    "RESOLUTION" => info.resolution = Self::parse_resolution(value),
                    "CODECS" => info.codecs = Some(value.to_string()),
                    "FRAME-RATE" => info.frame_rate = value.parse().ok(),
                    "AUDIO" => info.audio = Some(value.to_string()),
                    "VIDEO" => info.video = Some(value.to_string()),
                    "SUBTITLES" => info.subtitles = Some(value.to_string()),
                    "CLOSED-CAPTIONS" => info.closed_captions = Some(value.to_string()),
                    _ => {}
                }
            }
        }

        info
    }

    fn parse_resolution(s: &str) -> Option<(u32, u32)> {
        let (w, h) = s.split_once('x')?;
        Some((w.parse().ok()?, h.parse().ok()?))
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse() {
        let line = r#"#EXT-X-STREAM-INF:BANDWIDTH=1000000,RESOLUTION=1280x720,CODECS="avc1.64001f,mp4a.40.2""#;
        let info = StreamInfo::parse(line);
        assert_eq!(info.bandwidth, Some(1000000));
        assert_eq!(info.resolution, Some((1280, 720)));
        assert_eq!(info.codecs, Some("avc1.64001f,mp4a.40.2".to_string()));
    }
}
