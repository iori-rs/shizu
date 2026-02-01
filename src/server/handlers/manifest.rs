use axum::{
    extract::{Query, State},
    http::header,
    response::{IntoResponse, Response},
};

use crate::{
    Result,
    decrypt::DecryptionKey,
    proxy::HeaderCodec,
    server::{params::ManifestParams, state::AppState},
    stream::{StreamProcessor, TransformContext, rules},
};

/// Handle GET /manifest requests.
pub async fn handle_manifest(
    State(state): State<AppState>,
    Query(params): Query<ManifestParams>,
) -> Result<Response> {
    tracing::info!("Manifest request: {}", params.url);

    // Parse original URL
    let original_url = url::Url::parse(&params.url)?;

    // Decode headers
    let manifest_headers = HeaderCodec::decode_optional(params.h.as_deref())?;
    let segment_headers = HeaderCodec::decode_optional(params.sh.as_deref())?;

    // Parse decryption key if provided
    let decryption_key = params
        .k
        .as_ref()
        .map(|k| DecryptionKey::parse(k))
        .transpose()?;

    let decrypt_enabled = params.decrypt.unwrap_or(false);

    // Fetch the manifest
    let content = state
        .client
        .fetch_text(&params.url, Some(&manifest_headers))
        .await?;

    // Create transform context
    let context = TransformContext::new(
        original_url,
        params.h.clone(),
        params.sh.clone(),
        manifest_headers,
        segment_headers,
        decryption_key,
        decrypt_enabled,
    );

    // Create processor with default rules
    let rules = rules::default_rules();
    let mut processor = StreamProcessor::new(context, rules);

    // Process the manifest
    let transformed = processor.process(&content);

    tracing::debug!("Transformed manifest:\n{}", transformed);

    // Return response with appropriate content type
    Ok((
        [(header::CONTENT_TYPE, "application/vnd.apple.mpegurl")],
        transformed,
    )
        .into_response())
}
