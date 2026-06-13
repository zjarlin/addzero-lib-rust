use thiserror::Error;

pub type EdgeGatewayResult<T> = Result<T, EdgeGatewayError>;

#[derive(Debug, Error)]
pub enum EdgeGatewayError {
    #[error("missing edge-gateway database url")]
    MissingDatabaseUrl,
    #[error("gateway flow name must not be blank")]
    BlankFlowName,
    #[error("gateway flow route must not be blank")]
    BlankRoute,
    #[error("toasty database error: {0}")]
    Toasty(#[from] toasty::Error),
    #[error(transparent)]
    Runtime(#[from] anyhow::Error),
}
