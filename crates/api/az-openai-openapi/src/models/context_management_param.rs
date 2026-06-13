// Generated from OpenAPI spec. Do not edit by hand.
//! `ContextManagementParam` DTO.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextManagementParam {
    /// The context management entry type. Currently only 'compaction' is supported.
    #[serde(rename = "type")]
    pub type_value: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub compact_threshold: Option<i32>,
}
