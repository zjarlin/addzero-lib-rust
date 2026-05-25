use az_derive_aliases::{apply, error_eq};

/// Driver-local conversion errors before they are mapped to `toasty_core::Error`.
#[apply(error_eq)]
pub(crate) enum GitDbDriverError {
    #[error("unsupported gitdb/toasty value conversion: {0}")]
    UnsupportedValue(String),

    #[error("invalid gitdb result: {0}")]
    InvalidResult(String),
}
