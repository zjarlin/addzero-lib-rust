// Generated from OpenAPI spec. Do not edit by hand.
//! `FileAnnotation` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    FileAnnotationSource,
};

/// Annotation that references an uploaded file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileAnnotation {
    /// Type discriminator that is always `file` for this annotation.
    #[serde(rename = "type")]
    pub type_value: String,
    /// File attachment referenced by the annotation.
    pub source: FileAnnotationSource,
}
