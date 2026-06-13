// Generated from OpenAPI spec. Do not edit by hand.
//! `FileAnnotationSource` DTO.

use serde::{Deserialize, Serialize};

/// Attachment source referenced by an annotation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileAnnotationSource {
    /// Type discriminator that is always `file`.
    #[serde(rename = "type")]
    pub type_value: String,
    /// Filename referenced by the annotation.
    pub filename: String,
}
