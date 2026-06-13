// Generated from OpenAPI spec. Do not edit by hand.
//! `InputMessage` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    InputMessageContentList,
};

/// A message input to the model with a role indicating instruction following hierarchy. Instructions
/// given with the `developer` or `system` role take precedence over instructions given with the `user`
/// role.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InputMessage {
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
}
