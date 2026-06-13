// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `BatchErrors` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    BatchErrorsDataItem,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchErrors {
    /// The object type, which is always `list`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub object: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub data: Option<Vec<BatchErrorsDataItem>>,
}
