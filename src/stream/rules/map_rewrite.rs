use super::{LineType, ProcessorState, TransformContext, TransformRule};

/// Rule for rewriting #EXT-X-MAP tags.
pub struct MapTagRewriteRule;

impl TransformRule for MapTagRewriteRule {
    fn matches(
        &self,
        line_type: &LineType,
        state: &ProcessorState,
        context: &TransformContext,
    ) -> bool {
        if *line_type != LineType::ExtXMap {
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
        let Some(ref map_info) = state.current_map else {
            return vec![line.to_string()];
        };

        let Some(ref key) = state.current_key else {
            return vec![line.to_string()];
        };

        let Some(method) = key.method.to_segment_param() else {
            return vec![line.to_string()];
        };

        // Resolve the init segment URL
        let resolved = match context.resolve_url(&map_info.uri) {
            Ok(url) => url,
            Err(_) => return vec![line.to_string()],
        };

        let iv = state.current_iv();

        // Build segment URL for init segment
        let proxied = context.build_segment_url(
            &resolved,
            method,
            &iv,
            map_info.byterange.as_ref(),
            None, // No nested init
            None,
        );

        // Rebuild the #EXT-X-MAP tag with proxied URI
        let mut result = String::from("#EXT-X-MAP:URI=\"");
        result.push_str(&proxied);
        result.push('"');

        // Note: we don't include BYTERANGE in output since it's encoded in the proxied URL

        vec![result]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::decrypt::DecryptionKey;
    use crate::hls::{KeyInfo, KeyMethod};
    use crate::stream::state::MapInfo;
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
        )
    }

    #[test]
    fn test_rewrites_map_when_drm() {
        let rule = MapTagRewriteRule;
        let context = create_context_with_decrypt();

        let mut state = ProcessorState::new();
        state.update_key(KeyInfo {
            method: KeyMethod::SampleAes,
            uri: Some("https://key.server/key".to_string()),
            iv: None,
            keyformat: None,
            keyformatversions: None,
        });
        state.update_map(MapInfo {
            uri: "init.mp4".to_string(),
            byterange: None,
        });

        let result = rule.transform(r#"#EXT-X-MAP:URI="init.mp4""#, &mut state, &context);

        assert_eq!(result.len(), 1);
        assert!(result[0].starts_with("#EXT-X-MAP:URI=\"/segment.mp4?"));
    }
}
