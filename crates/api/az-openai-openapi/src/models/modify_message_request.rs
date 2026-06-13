// Generated from OpenAPI spec. Do not edit by hand.
//! `ModifyMessageRequest` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    Metadata,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModifyMessageRequest {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub metadata: Option<Metadata>,
}
