use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Failed to fetch URL: {url} - {reason}")]
    FetchFailed { url: String, reason: String },

    #[error("Fetch timeout for URL: {0}")]
    FetchTimeout(String),

    #[error("Invalid URL: {0}")]
    InvalidUrl(String),

    #[error("Invalid key format: {0}")]
    InvalidKeyFormat(String),

    #[error("Invalid key length: expected 16 bytes")]
    InvalidKeyLength,

    #[error("Single key required but multiple keys provided")]
    SingleKeyRequired,

    #[error("Multiple keys required but single key provided")]
    MultipleKeysRequired,

    #[error("Unsupported decryption method: {0}")]
    UnsupportedMethod(String),

    #[error("Unsupported method/format combination: {method} with {format}")]
    UnsupportedCombination { method: String, format: String },

    #[error("Decryption failed: {0}")]
    DecryptionFailed(String),

    #[error("Invalid header encoding: {0}")]
    InvalidHeaderEncoding(String),

    #[error("Invalid byte range format: {0}")]
    InvalidByteRange(String),

    #[error("Invalid IV format: {0}")]
    InvalidIv(String),

    #[error("Internal error: {0}")]
    Internal(String),
}

#[derive(Serialize)]
struct ErrorResponse {
    error: String,
    code: String,
}

impl Error {
    fn error_code(&self) -> &'static str {
        match self {
            Self::FetchFailed { .. } => "FETCH_FAILED",
            Self::FetchTimeout(_) => "FETCH_TIMEOUT",
            Self::InvalidUrl(_) => "INVALID_URL",
            Self::InvalidKeyFormat(_) => "INVALID_KEY_FORMAT",
            Self::InvalidKeyLength => "INVALID_KEY_LENGTH",
            Self::SingleKeyRequired => "SINGLE_KEY_REQUIRED",
            Self::MultipleKeysRequired => "MULTIPLE_KEYS_REQUIRED",
            Self::UnsupportedMethod(_) => "UNSUPPORTED_METHOD",
            Self::UnsupportedCombination { .. } => "UNSUPPORTED_COMBINATION",
            Self::DecryptionFailed(_) => "DECRYPTION_FAILED",
            Self::InvalidHeaderEncoding(_) => "INVALID_HEADER_ENCODING",
            Self::InvalidByteRange(_) => "INVALID_BYTE_RANGE",
            Self::InvalidIv(_) => "INVALID_IV",
            Self::Internal(_) => "INTERNAL_ERROR",
        }
    }

    fn status_code(&self) -> StatusCode {
        match self {
            Self::FetchFailed { .. } => StatusCode::BAD_GATEWAY,
            Self::FetchTimeout(_) => StatusCode::GATEWAY_TIMEOUT,
            Self::InvalidUrl(_)
            | Self::InvalidKeyFormat(_)
            | Self::InvalidKeyLength
            | Self::SingleKeyRequired
            | Self::MultipleKeysRequired
            | Self::InvalidHeaderEncoding(_)
            | Self::InvalidByteRange(_)
            | Self::InvalidIv(_) => StatusCode::BAD_REQUEST,
            Self::UnsupportedMethod(_) | Self::UnsupportedCombination { .. } => {
                StatusCode::NOT_IMPLEMENTED
            }
            Self::DecryptionFailed(_) | Self::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        let status = self.status_code();
        let body = ErrorResponse {
            error: self.to_string(),
            code: self.error_code().to_string(),
        };
        (status, Json(body)).into_response()
    }
}

impl From<url::ParseError> for Error {
    fn from(e: url::ParseError) -> Self {
        Self::InvalidUrl(e.to_string())
    }
}

impl From<reqwest::Error> for Error {
    fn from(e: reqwest::Error) -> Self {
        if e.is_timeout() {
            Self::FetchTimeout(e.url().map(|u| u.to_string()).unwrap_or_default())
        } else {
            Self::FetchFailed {
                url: e.url().map(|u| u.to_string()).unwrap_or_default(),
                reason: e.to_string(),
            }
        }
    }
}

impl From<hex::FromHexError> for Error {
    fn from(e: hex::FromHexError) -> Self {
        Self::InvalidKeyFormat(e.to_string())
    }
}
