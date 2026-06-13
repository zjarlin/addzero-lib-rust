// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `ItemReferenceParam` DTO.

use serde::{Deserialize, Serialize};

/// An internal identifier for an item to reference.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ItemReferenceParam {
    #[serde(rename = "type", default, skip_serializing_if = "Option::is_none")]
    pub type_value: Option<String>,
    /// The ID of the item to reference.
    pub id: String,
}
