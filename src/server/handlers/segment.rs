use axum::{
    body::Body,
    extract::{Path, Query, State},
    http::header,
    response::{IntoResponse, Response},
};

use crate::{
    Error, Result,
    decrypt::{DecryptionKey, SegmentDecryptMethod, SegmentDecryptor},
    hls::{ByteRange, SegmentFormat},
    proxy::HeaderCodec,
    server::{params::SegmentParams, state::AppState},
};

/// Handle GET /segment requests.
pub async fn handle_segment(
    State(state): State<AppState>,
    path: Path<String>,
    Query(params): Query<SegmentParams>,
) -> Result<Response> {
    tracing::info!("Segment request: {}", params.url);

    // Verify URL signature to prevent SSRF attacks
    if !state.verify_signature(&params.url, params.sig.as_deref()) {
        tracing::warn!("Invalid signature for URL: {}", params.url);
        return Err(Error::InvalidSignature);
    }

    // Parse decryption method
    let method = SegmentDecryptMethod::parse(&params.m)?;

    // Parse decryption key
    let key = DecryptionKey::parse(&params.k)?;

    // Parse IV (default to zeros if not provided)
    let iv = parse_iv(params.iv.as_deref())?;

    // Decode headers
    let headers = HeaderCodec::decode_optional(params.h.as_deref())?;

    // Parse byte range
    let byterange = params
        .br
        .as_ref()
        .map(|br| ByteRange::parse(br))
        .transpose()?;

    // Determine segment format from path extension or URL
    let format = SegmentFormat::from_extension(&path)?;

    // Create decryptor
    let decryptor = SegmentDecryptor::new(method, key, iv);

    // Fetch init segment if needed (for fMP4), caching both raw and decrypted forms
    let (init_data, init_decrypted) = if let Some(ref init_url) = params.init {
        let init_byterange = params
            .init_br
            .as_ref()
            .map(|br| ByteRange::parse(br))
            .transpose()?;

        let (raw, decrypted) = state
            .init_cache
            .get_or_fetch_with_decrypted(
                init_url,
                &headers,
                init_byterange.as_ref(),
                &params.k,
                &state.client,
                |raw| decryptor.decrypt_init(raw),
            )
            .await?;

        (Some(raw), Some(decrypted))
    } else {
        (None, None)
    };

    // Fetch segment, reusing init cache if the URL was already fetched as an init segment
    let segment_data = match state.init_cache.get(&params.url, &headers, byterange.as_ref()) {
        Some(cached) => {
            tracing::debug!("Segment served from init cache: {}", params.url);
            cached
        }
        None => {
            state
                .client
                .fetch(&params.url, Some(&headers), byterange.as_ref())
                .await?
        }
    };

    tracing::debug!(
        "Fetched segment: {} bytes, format: {:?}",
        segment_data.len(),
        format
    );

    let decrypted = decryptor
        .decrypt(segment_data, init_data, init_decrypted, format)
        .await?;

    tracing::debug!("Decrypted segment: {} bytes", decrypted.len());

    // Return response with appropriate content type
    Ok((
        [(header::CONTENT_TYPE, format.content_type())],
        Body::from(decrypted),
    )
        .into_response())
}

/// Parse IV from hex string or return default zeros.
fn parse_iv(iv_str: Option<&str>) -> Result<[u8; 16]> {
    match iv_str {
        Some(s) if !s.is_empty() => {
            let s = s
                .strip_prefix("0x")
                .or_else(|| s.strip_prefix("0X"))
                .unwrap_or(s);
            let bytes = hex::decode(s).map_err(|e| Error::InvalidIv(e.to_string()))?;
            if bytes.len() != 16 {
                return Err(Error::InvalidIv(format!(
                    "Expected 16 bytes, got {}",
                    bytes.len()
                )));
            }
            let mut iv = [0u8; 16];
            iv.copy_from_slice(&bytes);
            Ok(iv)
        }
        _ => Ok([0u8; 16]),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_iv_with_prefix() {
        let iv = parse_iv(Some("0x00000000000000000000000000000001")).unwrap();
        assert_eq!(iv[15], 1);
    }

    #[test]
    fn test_parse_iv_without_prefix() {
        let iv = parse_iv(Some("00000000000000000000000000000001")).unwrap();
        assert_eq!(iv[15], 1);
    }

    #[test]
    fn test_parse_iv_none() {
        let iv = parse_iv(None).unwrap();
        assert_eq!(iv, [0u8; 16]);
    }
}
