// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `UrlAnnotation` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    UrlAnnotationSource,
};

/// Annotation that references a URL.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UrlAnnotation {
    /// Type discriminator that is always `url` for this annotation.
    #[serde(rename = "type")]
    pub type_value: String,
    /// URL referenced by the annotation.
    pub source: UrlAnnotationSource,
}
