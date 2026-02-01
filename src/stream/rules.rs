pub mod key_rewrite;
pub mod map_rewrite;
pub mod media_proxy;
pub mod segment_proxy;
pub mod variant_proxy;

use super::{classifier::LineType, context::TransformContext, state::ProcessorState};

pub use key_rewrite::KeyTagRewriteRule;
pub use map_rewrite::MapTagRewriteRule;
pub use media_proxy::MediaTagProxyRule;
pub use segment_proxy::SegmentUrlProxyRule;
pub use variant_proxy::VariantUrlProxyRule;

/// Trait for transform rules.
pub trait TransformRule: Send + Sync {
    /// Check if this rule should be applied.
    fn matches(&self, line_type: &LineType, state: &ProcessorState, context: &TransformContext)
        -> bool;

    /// Transform the line.
    fn transform(
        &self,
        line: &str,
        state: &mut ProcessorState,
        context: &TransformContext,
    ) -> Vec<String>;
}

/// Create default set of transform rules.
pub fn default_rules() -> Vec<Box<dyn TransformRule>> {
    vec![
        Box::new(VariantUrlProxyRule),
        Box::new(MediaTagProxyRule),
        Box::new(KeyTagRewriteRule),
        Box::new(MapTagRewriteRule),
        Box::new(SegmentUrlProxyRule),
    ]
}
