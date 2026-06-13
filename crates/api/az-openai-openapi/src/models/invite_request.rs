// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `InviteRequest` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    InviteRequestProject,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InviteRequest {
    /// Send an email to this address
    pub email: String,
    /// `owner` or `reader`
    pub role: String,
    /// An array of projects to which membership is granted at the same time the org invite is accepted. If
    /// omitted, the user will be invited to the default project for compatibility with legacy behavior.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub projects: Option<Vec<InviteRequestProject>>,
}
