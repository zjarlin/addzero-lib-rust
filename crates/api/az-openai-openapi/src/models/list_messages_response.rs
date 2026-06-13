// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `ListMessagesResponse` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    MessageObject,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListMessagesResponse {
    pub object: String,
    pub data: Vec<MessageObject>,
    pub first_id: String,
    pub last_id: String,
    pub has_more: bool,
}
