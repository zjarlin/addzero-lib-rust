use thiserror::Error;

pub type ConfigCenterResult<T> = Result<T, ConfigCenterError>;

#[derive(Debug, Error)]
pub enum ConfigCenterError {
    #[error("missing config-center database url")]
    MissingDatabaseUrl,
    #[error("config key must not be blank")]
    BlankKey,
    #[error("config value must not be blank")]
    BlankValue,
    #[error("toasty database error: {0}")]
    Toasty(#[from] toasty::Error),
    #[error(transparent)]
    Runtime(#[from] anyhow::Error),
}
