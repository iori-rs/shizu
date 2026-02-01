use super::{LineType, ProcessorState, TransformContext, TransformRule};
use crate::stream::state::PendingContext;

/// Rule for rewriting segment URLs to go through /segment when decrypting.
pub struct SegmentUrlProxyRule;

impl TransformRule for SegmentUrlProxyRule {
    fn matches(
        &self,
        line_type: &LineType,
        state: &ProcessorState,
        context: &TransformContext,
    ) -> bool {
        // Only match URIs in media playlists that are segments
        if *line_type != LineType::Uri {
            return false;
        }

        // Must be a segment (after #EXTINF)
        if !matches!(state.pending_context, Some(PendingContext::Segment)) {
            return false;
        }

        // Only rewrite if we're intercepting DRM
        if let Some(ref key) = state.current_key {
            context.should_intercept(key.requires_server_decrypt())
        } else {
            false
        }
    }

    fn transform(
        &self,
        line: &str,
        state: &mut ProcessorState,
        context: &TransformContext,
    ) -> Vec<String> {
        let line = line.trim();

        let Some(ref key) = state.current_key else {
            return vec![line.to_string()];
        };

        let Some(method) = key.method.to_segment_param() else {
            return vec![line.to_string()];
        };

        // Resolve relative URL
        let resolved = match context.resolve_url(line) {
            Ok(url) => url,
            Err(_) => return vec![line.to_string()],
        };

        let iv = state.current_iv();
        let byterange = state.current_byterange.as_ref();

        // Get init segment info if present
        let (init_url, init_byterange) = if let Some(ref map) = state.current_map {
            let init_resolved = context.resolve_url(&map.uri).ok();
            (init_resolved, map.byterange.as_ref())
        } else {
            (None, None)
        };

        // Build proxied segment URL
        let proxied = context.build_segment_url(
            &resolved,
            method,
            &iv,
            byterange,
            init_url.as_ref(),
            init_byterange,
        );

        vec![proxied]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::decrypt::DecryptionKey;
    use crate::hls::{KeyInfo, KeyMethod};
    use crate::server::SigningKey;
    use std::collections::HashMap;
    use url::Url;

    fn create_context_with_decrypt() -> TransformContext {
        TransformContext::new(
            Url::parse("https://cdn.example.com/playlist.m3u8").unwrap(),
            None,
            None,
            HashMap::new(),
            HashMap::new(),
            Some(DecryptionKey::parse("0123456789abcdef0123456789abcdef").unwrap()),
            true,
            SigningKey::test_key(),
        )
    }

    #[test]
    fn test_matches_drm_segment() {
        let rule = SegmentUrlProxyRule;
        let context = create_context_with_decrypt();

        let mut state = ProcessorState::new();
        state.update_media_sequence(0);
        state.set_pending_segment();
        state.update_key(KeyInfo {
            method: KeyMethod::SampleAes,
            uri: Some("https://key.server/key".to_string()),
            iv: None,
            keyformat: None,
            keyformatversions: None,
        });

        assert!(rule.matches(&LineType::Uri, &state, &context));
    }

    #[test]
    fn test_does_not_match_aes128_segment() {
        let rule = SegmentUrlProxyRule;
        let context = create_context_with_decrypt();

        let mut state = ProcessorState::new();
        state.update_media_sequence(0);
        state.set_pending_segment();
        state.update_key(KeyInfo {
            method: KeyMethod::Aes128,
            uri: Some("https://key.server/key".to_string()),
            iv: None,
            keyformat: None,
            keyformatversions: None,
        });

        // AES-128 should NOT be intercepted
        assert!(!rule.matches(&LineType::Uri, &state, &context));
    }

    #[test]
    fn test_rewrites_segment_url() {
        let rule = SegmentUrlProxyRule;
        let context = create_context_with_decrypt();

        let mut state = ProcessorState::new();
        state.update_media_sequence(0);
        state.set_pending_segment();
        state.update_key(KeyInfo {
            method: KeyMethod::SampleAes,
            uri: Some("https://key.server/key".to_string()),
            iv: None,
            keyformat: None,
            keyformatversions: None,
        });

        let result = rule.transform("segment001.ts", &mut state, &context);

        assert_eq!(result.len(), 1);
        assert!(result[0].starts_with("/segment.ts?"));
        assert!(result[0].contains("m=ssa"));
    }
}
