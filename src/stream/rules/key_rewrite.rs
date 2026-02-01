use super::{LineType, ProcessorState, TransformContext, TransformRule};

/// Rule for rewriting or removing #EXT-X-KEY tags.
pub struct KeyTagRewriteRule;

impl TransformRule for KeyTagRewriteRule {
    fn matches(
        &self,
        line_type: &LineType,
        state: &ProcessorState,
        context: &TransformContext,
    ) -> bool {
        if *line_type != LineType::ExtXKey {
            return false;
        }

        // Only handle if we're intercepting DRM
        if let Some(ref key) = state.current_key {
            context.should_intercept(key.requires_server_decrypt())
        } else {
            false
        }
    }

    fn transform(
        &self,
        _line: &str,
        state: &mut ProcessorState,
        _context: &TransformContext,
    ) -> Vec<String> {
        // When we intercept DRM, we remove the KEY tag from output
        // because the /segment endpoint will handle decryption

        // Check if it's a DRM method we're handling
        if let Some(ref key) = state.current_key
            && key.requires_server_decrypt()
        {
            // Remove the KEY tag - segments will be decrypted by server
            return vec![];
        }

        // For AES-128 or other methods, this rule shouldn't match
        // but if it does, passthrough
        vec![]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::decrypt::DecryptionKey;
    use crate::hls::{KeyInfo, KeyMethod};
    use std::collections::HashMap;
    use url::Url;

    fn create_context_with_decrypt() -> TransformContext {
        TransformContext::new(
            Url::parse("https://cdn.example.com/playlist.m3u8").unwrap(),
            Url::parse("http://localhost:8080").unwrap(),
            None,
            None,
            HashMap::new(),
            HashMap::new(),
            Some(DecryptionKey::parse("0123456789abcdef0123456789abcdef").unwrap()),
            true,
        )
    }

    #[test]
    fn test_matches_drm_key() {
        let rule = KeyTagRewriteRule;
        let context = create_context_with_decrypt();

        let mut state = ProcessorState::new();
        state.update_key(KeyInfo {
            method: KeyMethod::SampleAes,
            uri: Some("https://key.server/key".to_string()),
            iv: None,
            keyformat: None,
            keyformatversions: None,
        });

        assert!(rule.matches(&LineType::ExtXKey, &state, &context));
    }

    #[test]
    fn test_does_not_match_aes128() {
        let rule = KeyTagRewriteRule;
        let context = create_context_with_decrypt();

        let mut state = ProcessorState::new();
        state.update_key(KeyInfo {
            method: KeyMethod::Aes128,
            uri: Some("https://key.server/key".to_string()),
            iv: None,
            keyformat: None,
            keyformatversions: None,
        });

        // AES-128 should NOT be intercepted
        assert!(!rule.matches(&LineType::ExtXKey, &state, &context));
    }

    #[test]
    fn test_removes_drm_key_tag() {
        let rule = KeyTagRewriteRule;
        let context = create_context_with_decrypt();

        let mut state = ProcessorState::new();
        state.update_key(KeyInfo {
            method: KeyMethod::SampleAes,
            uri: Some("https://key.server/key".to_string()),
            iv: None,
            keyformat: None,
            keyformatversions: None,
        });

        let result = rule.transform(
            r#"#EXT-X-KEY:METHOD=SAMPLE-AES,URI="https://key.server/key""#,
            &mut state,
            &context,
        );

        assert!(result.is_empty());
    }
}
