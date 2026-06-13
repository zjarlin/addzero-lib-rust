use thiserror::Error;

pub type LowcodeResult<T> = Result<T, LowcodeError>;

#[derive(Debug, Error)]
pub enum LowcodeError {
    #[error(
        "missing lowcode database url in config center namespace az-aio.dev key lowcode.database_url"
    )]
    MissingDatabaseUrl,
    #[error("invalid lowcode app id")]
    InvalidAppId,
    #[error("invalid lowcode page id")]
    InvalidPageId,
    #[error("config center error: {0}")]
    ConfigCenter(#[from] az_config_center_client::ConfigCenterError),
    #[error("toasty database error: {0}")]
    Toasty(#[from] toasty::Error),
    #[error("io error while {operation}: {source}")]
    Io {
        operation: &'static str,
        #[source]
        source: std::io::Error,
    },
    #[error("invalid socket address: {0}")]
    SocketAddr(#[from] std::net::AddrParseError),
}
