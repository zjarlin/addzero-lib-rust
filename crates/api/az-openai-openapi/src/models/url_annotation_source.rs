// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `UrlAnnotationSource` DTO.

use serde::{Deserialize, Serialize};

/// URL backing an annotation entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UrlAnnotationSource {
    /// Type discriminator that is always `url`.
    #[serde(rename = "type")]
    pub type_value: String,
    /// URL referenced by the annotation.
    pub url: String,
}
