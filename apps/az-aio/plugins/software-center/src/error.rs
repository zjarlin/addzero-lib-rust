use thiserror::Error;

pub type SoftwareCenterResult<T> = Result<T, SoftwareCenterError>;

#[derive(Debug, Error)]
pub enum SoftwareCenterError {
    #[error("missing software-center database url")]
    MissingDatabaseUrl,
    #[error("software package name must not be blank")]
    BlankPackageName,
    #[error("software package source path must not be blank")]
    BlankSourcePath,
    #[error("toasty database error: {0}")]
    Toasty(#[from] toasty::Error),
    #[error(transparent)]
    Runtime(#[from] anyhow::Error),
}
