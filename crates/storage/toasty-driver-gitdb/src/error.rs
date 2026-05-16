use thiserror::Error;

/// Driver-local conversion errors before they are mapped to `toasty_core::Error`.
#[derive(Debug, Error)]
pub(crate) enum GitDbDriverError {
    #[error("unsupported gitdb/toasty value conversion: {0}")]
    UnsupportedValue(String),

    #[error("invalid gitdb result: {0}")]
    InvalidResult(String),
}
