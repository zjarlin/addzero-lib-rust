// Generated from OpenAPI spec. Do not edit by hand.
//! `CreateConversationItemsRequest` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    InputItem,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateConversationItemsRequest {
    /// The items to add to the conversation. You may add up to 20 items at a time.
    pub items: Vec<InputItem>,
}
