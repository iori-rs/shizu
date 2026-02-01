use super::{LineType, ProcessorState, TransformContext, TransformRule};
use crate::stream::state::PendingContext;

/// Rule for rewriting variant playlist URLs to go through /manifest.
pub struct VariantUrlProxyRule;

impl TransformRule for VariantUrlProxyRule {
    fn matches(
        &self,
        line_type: &LineType,
        state: &ProcessorState,
        _context: &TransformContext,
    ) -> bool {
        // Match URIs that follow #EXT-X-STREAM-INF
        *line_type == LineType::Uri
            && matches!(
                state.pending_context,
                Some(PendingContext::VariantStream(_))
            )
    }

    fn transform(
        &self,
        line: &str,
        _state: &mut ProcessorState,
        context: &TransformContext,
    ) -> Vec<String> {
        let line = line.trim();

        // Resolve relative URL
        let resolved = match context.resolve_url(line) {
            Ok(url) => url,
            Err(_) => return vec![line.to_string()],
        };

        // Build proxied manifest URL
        let proxied = context.build_manifest_url(&resolved);

        vec![proxied]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hls::StreamInfo;
    use std::collections::HashMap;
    use url::Url;

    fn create_test_context() -> TransformContext {
        TransformContext::new(
            Url::parse("https://cdn.example.com/master.m3u8").unwrap(),
            None,
            None,
            HashMap::new(),
            HashMap::new(),
            None,
            false,
        )
    }

    #[test]
    fn test_matches_variant_uri() {
        let rule = VariantUrlProxyRule;
        let context = create_test_context();

        let mut state = ProcessorState::new();
        state.set_pending_variant(StreamInfo::default());

        assert!(rule.matches(&LineType::Uri, &state, &context));
    }

    #[test]
    fn test_does_not_match_segment_uri() {
        let rule = VariantUrlProxyRule;
        let context = create_test_context();

        let mut state = ProcessorState::new();
        state.set_pending_segment();

        assert!(!rule.matches(&LineType::Uri, &state, &context));
    }

    #[test]
    fn test_transform_relative_url() {
        let rule = VariantUrlProxyRule;
        let context = create_test_context();
        let mut state = ProcessorState::new();

        let result = rule.transform("720p/playlist.m3u8", &mut state, &context);

        assert_eq!(result.len(), 1);
        assert!(result[0].starts_with("/manifest?"));
        assert!(result[0].contains("url=https%3A%2F%2Fcdn.example.com%2F720p%2Fplaylist.m3u8"));
    }
}
