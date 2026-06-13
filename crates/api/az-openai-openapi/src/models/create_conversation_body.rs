// Generated from OpenAPI spec. Do not edit by hand.
//! `CreateConversationBody` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    InputItem,
    Metadata,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateConversationBody {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub metadata: Option<Metadata>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub items: Option<Vec<InputItem>>,
}
