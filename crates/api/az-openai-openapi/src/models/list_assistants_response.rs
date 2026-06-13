// Generated from OpenAPI spec. Do not edit by hand.
//! `ListAssistantsResponse` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    AssistantObject,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListAssistantsResponse {
    pub object: String,
    pub data: Vec<AssistantObject>,
    pub first_id: String,
    pub last_id: String,
    pub has_more: bool,
}
