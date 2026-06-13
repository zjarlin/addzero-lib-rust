use thiserror::Error;

pub type DriveCenterResult<T> = Result<T, DriveCenterError>;

#[derive(Debug, Error)]
pub enum DriveCenterError {
    #[error("missing drive-center database url")]
    MissingDatabaseUrl,
    #[error("drive path must not be blank")]
    BlankPath,
    #[error("drive action must not be blank")]
    BlankAction,
    #[error("toasty database error: {0}")]
    Toasty(#[from] toasty::Error),
    #[error(transparent)]
    Runtime(#[from] anyhow::Error),
}
