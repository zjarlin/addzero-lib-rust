// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `InviteListResponse` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    Invite,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InviteListResponse {
    /// The object type, which is always `list`
    pub object: String,
    pub data: Vec<Invite>,
    /// The first `invite_id` in the retrieved `list`
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub first_id: Option<String>,
    /// The last `invite_id` in the retrieved `list`
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_id: Option<String>,
    /// The `has_more` property is used for pagination to indicate there are additional results.
    pub has_more: bool,
}
