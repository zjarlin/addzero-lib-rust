// Generated from OpenAPI spec. Do not edit by hand.
//! `ModifyThreadRequest` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    Metadata,
    ModifyThreadRequestToolResources,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModifyThreadRequest {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tool_resources: Option<ModifyThreadRequestToolResources>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub metadata: Option<Metadata>,
}
