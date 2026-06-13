// Generated from OpenAPI spec. Do not edit by hand.
//! `VectorStoreFileContentResponseDataItem` DTO.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorStoreFileContentResponseDataItem {
    /// The content type (currently only `"text"`)
    #[serde(rename = "type", default, skip_serializing_if = "Option::is_none")]
    pub type_value: Option<String>,
    /// The text content
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
}
