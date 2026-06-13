// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `VectorStoreFileObjectLastError` DTO.

use serde::{Deserialize, Serialize};

/// The last error associated with this vector store file. Will be `null` if there are no errors.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorStoreFileObjectLastError {
    /// One of `server_error`, `unsupported_file`, or `invalid_file`.
    pub code: String,
    /// A human-readable description of the error.
    pub message: String,
}
