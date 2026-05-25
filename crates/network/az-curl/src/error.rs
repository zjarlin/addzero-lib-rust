use az_derive_aliases::{apply, error};

/// Result type used by `az-curl`.
pub type CurlResult<T> = Result<T, CurlError>;

/// Error type for curl parsing, request building, execution, and response decoding.
#[apply(error)]
pub enum CurlError {
    #[error("failed to tokenize curl command")]
    Tokenize,
    #[error("flag `{0}` requires a value")]
    MissingFlagValue(&'static str),
    #[error("curl command does not contain a URL")]
    MissingUrl,
    #[error("invalid HTTP method `{0}`")]
    InvalidMethod(String),
    #[error("invalid URL `{0}`")]
    InvalidUrl(String),
    #[error("invalid header expression `{0}`")]
    InvalidHeader(String),
    #[error("invalid form expression `{0}`")]
    InvalidFormField(String),
    #[error("failed to build request: {0}")]
    RequestBuild(#[source] reqwest::Error),
    #[error("failed to execute request: {0}")]
    Execute(#[source] reqwest::Error),
    #[error("response body is not valid UTF-8: {0}")]
    Utf8(#[from] std::string::FromUtf8Error),
}
