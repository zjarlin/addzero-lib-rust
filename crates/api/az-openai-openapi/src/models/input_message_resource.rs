// Generated from OpenAPI spec. Do not edit by hand.
//! `InputMessageResource` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    InputMessageContentList,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InputMessageResource {
    /// The type of the message input. Always set to `message`.
    #[serde(rename = "type", default, skip_serializing_if = "Option::is_none")]
    pub type_value: Option<String>,
    /// The role of the message input. One of `user`, `system`, or `developer`.
    pub role: String,
    /// The status of item. One of `in_progress`, `completed`, or `incomplete`. Populated when items are
    /// returned via API.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
    pub content: InputMessageContentList,
    /// The unique ID of the message input.
    pub id: String,
}
