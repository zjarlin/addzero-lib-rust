use thiserror::Error;

pub type LineCrdtResult<T> = Result<T, LineCrdtError>;

#[derive(Debug, Error)]
pub enum LineCrdtError {
    #[error("line index {index} is out of bounds for {line_count} lines")]
    LineIndexOutOfBounds { index: usize, line_count: usize },

    #[error("line content must not contain '\\n'")]
    LineContainsNewline,

    #[error("invalid CRDT version cursor: {reason}")]
    InvalidVersion { reason: String },

    #[error("{operation} failed: {reason}")]
    Engine {
        operation: &'static str,
        reason: String,
    },
}

pub(crate) fn engine_error(operation: &'static str, error: impl std::fmt::Debug) -> LineCrdtError {
    LineCrdtError::Engine {
        operation,
        reason: format!("{error:?}"),
    }
}
