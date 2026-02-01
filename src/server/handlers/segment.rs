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

    // Fetch init segment if needed (for fMP4)
    let init_data = if let Some(ref init_url) = params.init {
        let init_byterange = params
            .init_br
            .as_ref()
            .map(|br| ByteRange::parse(br))
            .transpose()?;

        Some(
            state
                .init_cache
                .get_or_fetch(init_url, &headers, init_byterange.as_ref(), &state.client)
                .await?,
        )
    } else {
        None
    };

    // Fetch segment
    let segment_data = state
        .client
        .fetch(&params.url, Some(&headers), byterange.as_ref())
        .await?;

    tracing::debug!(
        "Fetched segment: {} bytes, format: {:?}",
        segment_data.len(),
        format
    );

    // Create decryptor and decrypt
    let decryptor = SegmentDecryptor::new(method, key, iv);
    let decrypted = decryptor.decrypt(segment_data, init_data, format).await?;

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
