use super::{
    classifier::{LineClassifier, LineType},
    context::TransformContext,
    rules::TransformRule,
    state::{MapInfo, ProcessorState},
};
use crate::hls::{ByteRange, KeyInfo, StreamInfo};

/// Stream-based M3U8 processor.
pub struct StreamProcessor {
    state: ProcessorState,
    context: TransformContext,
    rules: Vec<Box<dyn TransformRule>>,
}

impl StreamProcessor {
    pub fn new(context: TransformContext, rules: Vec<Box<dyn TransformRule>>) -> Self {
        Self {
            state: ProcessorState::new(),
            context,
            rules,
        }
    }

    /// Process entire playlist content and return transformed content.
    pub fn process(&mut self, input: &str) -> String {
        let mut output = Vec::new();

        for line in input.lines() {
            let transformed = self.process_line(line);
            output.extend(transformed);
        }

        output.join("\n")
    }

    /// Process a single line and return transformed line(s).
    pub fn process_line(&mut self, line: &str) -> Vec<String> {
        let line_type = LineClassifier::classify(line);

        // Update state based on line type
        self.update_state(&line_type, line);

        // Find first matching rule and apply it
        for rule in &self.rules {
            if rule.matches(&line_type, &self.state, &self.context) {
                let result = rule.transform(line, &mut self.state, &self.context);

                // Post-transform state updates (e.g., advance segment after URI)
                self.post_transform_update(&line_type);

                return result;
            }
        }

        // Post-transform state updates
        self.post_transform_update(&line_type);

        // Default: passthrough
        vec![line.to_string()]
    }

    /// Update state based on line type (before transform).
    fn update_state(&mut self, line_type: &LineType, line: &str) {
        match line_type {
            LineType::ExtXStreamInf => {
                let info = StreamInfo::parse(line);
                self.state.set_pending_variant(info);
            }
            LineType::ExtXMediaSequence => {
                if let Some(seq) = Self::parse_sequence(line) {
                    self.state.update_media_sequence(seq);
                }
            }
            LineType::ExtXKey => {
                if let Some(key) = KeyInfo::parse(line) {
                    self.state.update_key(key);
                }
            }
            LineType::ExtXMap => {
                if let Some(map) = MapInfo::parse(line) {
                    self.state.update_map(map);
                }
            }
            LineType::ExtInf => {
                self.state.set_pending_segment();
            }
            LineType::ExtXByteRange => {
                if let Ok(br) = ByteRange::parse_from_tag(line) {
                    self.state.set_byterange(br);
                }
            }
            LineType::ExtXDiscontinuity => {
                // Key context may change after discontinuity
                // We don't reset here as key should persist until explicitly changed
            }
            _ => {}
        }
    }

    /// Post-transform state updates.
    fn post_transform_update(&mut self, line_type: &LineType) {
        // After processing a URI that was a segment, advance the segment counter
        if *line_type == LineType::Uri {
            if matches!(
                self.state.pending_context,
                Some(super::state::PendingContext::Segment) | None
            ) && self.state.is_media_playlist()
            {
                self.state.advance_segment();
            } else {
                // Clear pending for variants
                self.state.pending_context = None;
            }
        }
    }

    fn parse_sequence(line: &str) -> Option<u64> {
        line.strip_prefix("#EXT-X-MEDIA-SEQUENCE:")
            .and_then(|s| s.trim().parse().ok())
    }

    /// Get current state (for inspection/testing).
    pub fn state(&self) -> &ProcessorState {
        &self.state
    }

    /// Get context (for inspection/testing).
    pub fn context(&self) -> &TransformContext {
        &self.context
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use url::Url;

    fn create_test_context() -> TransformContext {
        TransformContext::new(
            Url::parse("https://example.com/playlist.m3u8").unwrap(),
            None,
            None,
            HashMap::new(),
            HashMap::new(),
            None,
            false,
        )
    }

    #[test]
    fn test_passthrough_simple_playlist() {
        let context = create_test_context();
        let mut processor = StreamProcessor::new(context, vec![]);

        let input = "#EXTM3U\n#EXT-X-VERSION:3\n#EXTINF:6.0,\nsegment.ts";
        let output = processor.process(input);

        assert_eq!(output, input);
    }
}
