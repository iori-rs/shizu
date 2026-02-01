use super::{LineType, ProcessorState, TransformContext, TransformRule};

/// Rule for rewriting #EXT-X-MEDIA tags with URI attributes.
pub struct MediaTagProxyRule;

impl TransformRule for MediaTagProxyRule {
    fn matches(
        &self,
        line_type: &LineType,
        _state: &ProcessorState,
        _context: &TransformContext,
    ) -> bool {
        *line_type == LineType::ExtXMedia
    }

    fn transform(
        &self,
        line: &str,
        _state: &mut ProcessorState,
        context: &TransformContext,
    ) -> Vec<String> {
        // Parse and rewrite URI attribute if present
        let Some(content) = line.strip_prefix("#EXT-X-MEDIA:") else {
            return vec![line.to_string()];
        };

        // Find URI attribute and rewrite it
        let mut result = String::from("#EXT-X-MEDIA:");
        let mut first = true;

        for attr in Self::parse_attributes(content) {
            if !first {
                result.push(',');
            }
            first = false;

            if let Some((key, value)) = attr.split_once('=') {
                let key_upper = key.trim().to_uppercase();
                if key_upper == "URI" {
                    // Rewrite URI
                    let uri = value.trim().trim_matches('"');
                    if let Ok(resolved) = context.resolve_url(uri) {
                        let proxied = context.build_manifest_url(&resolved);
                        result.push_str(&format!("URI=\"{}\"", proxied));
                        continue;
                    }
                }
            }
            result.push_str(attr);
        }

        vec![result]
    }
}

impl MediaTagProxyRule {
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
    fn test_matches_ext_x_media() {
        let rule = MediaTagProxyRule;
        let context = create_test_context();
        let state = ProcessorState::new();

        assert!(rule.matches(&LineType::ExtXMedia, &state, &context));
        assert!(!rule.matches(&LineType::ExtXStreamInf, &state, &context));
    }

    #[test]
    fn test_rewrite_uri() {
        let rule = MediaTagProxyRule;
        let context = create_test_context();
        let mut state = ProcessorState::new();

        let line =
            r#"#EXT-X-MEDIA:TYPE=AUDIO,GROUP-ID="audio",NAME="English",URI="audio/playlist.m3u8""#;
        let result = rule.transform(line, &mut state, &context);

        assert_eq!(result.len(), 1);
        assert!(result[0].contains("/manifest?"));
    }
}
