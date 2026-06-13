use thiserror::Error;

pub type AssetHubResult<T> = Result<T, AssetHubError>;

#[derive(Debug, Error)]
pub enum AssetHubError {
    #[error("missing asset-hub database url")]
    MissingDatabaseUrl,
    #[error("asset title must not be blank")]
    BlankTitle,
    #[error("asset status must not be blank")]
    BlankStatus,
    #[error("toasty database error: {0}")]
    Toasty(#[from] toasty::Error),
    #[error(transparent)]
    Runtime(#[from] anyhow::Error),
}
