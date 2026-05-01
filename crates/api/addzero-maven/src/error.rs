use reqwest::header::{InvalidHeaderName, InvalidHeaderValue};
use thiserror::Error;

pub type CreatesResult<T> = Result<T, CreatesError>;

#[derive(Debug, Error)]
pub enum CreatesError {
    #[error("invalid config: {0}")]
    InvalidConfig(String),
    #[error("invalid base url `{0}`")]
    InvalidBaseUrl(String),
    #[error("invalid request path `{0}`")]
    InvalidPath(String),
    #[error("invalid header name `{name}`: {source}")]
    InvalidHeaderName {
        name: String,
        #[source]
        source: InvalidHeaderName,
    },
    #[error("invalid header value for `{name}`: {source}")]
    InvalidHeaderValue {
        name: String,
        #[source]
        source: InvalidHeaderValue,
    },
    #[error("request failed: {0}")]
    Transport(#[from] reqwest::Error),
    #[error("failed to parse json payload: {0}")]
    Json(#[from] serde_json::Error),
    #[error("request to `{url}` returned HTTP {status}: {body}")]
    HttpStatus {
        url: String,
        status: u16,
        body: String,
    },
    #[error("signature error: {0}")]
    Signature(String),
    #[error("invalid response: {0}")]
    InvalidResponse(String),
}

impl From<addzero_music::MusicError> for CreatesError {
    fn from(value: addzero_music::MusicError) -> Self {
        match value {
            addzero_music::MusicError::InvalidConfig(message) => Self::InvalidConfig(message),
            addzero_music::MusicError::InvalidBaseUrl(url) => Self::InvalidBaseUrl(url),
            addzero_music::MusicError::InvalidPath(path) => Self::InvalidPath(path),
            addzero_music::MusicError::InvalidHeaderName { name, source } => {
                Self::InvalidHeaderName { name, source }
            }
            addzero_music::MusicError::InvalidHeaderValue { name, source } => {
                Self::InvalidHeaderValue { name, source }
            }
            addzero_music::MusicError::Transport(error) => Self::Transport(error),
            addzero_music::MusicError::Json(error) => Self::Json(error),
            addzero_music::MusicError::HttpStatus { url, status, body } => {
                Self::HttpStatus { url, status, body }
            }
            addzero_music::MusicError::InvalidResponse(message) => Self::InvalidResponse(message),
        }
    }
}
