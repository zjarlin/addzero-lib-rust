// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `Invite` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    InviteProject,
};

/// Represents an individual `invite` to the organization.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Invite {
    /// The object type, which is always `organization.invite`
    pub object: String,
    /// The identifier, which can be referenced in API endpoints
    pub id: String,
    /// The email address of the individual to whom the invite was sent
    pub email: String,
    /// `owner` or `reader`
    pub role: String,
    /// `accepted`,`expired`, or `pending`
    pub status: String,
    /// The Unix timestamp (in seconds) of when the invite was sent.
    pub created_at: i64,
    /// The Unix timestamp (in seconds) of when the invite expires.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<i64>,
    /// The Unix timestamp (in seconds) of when the invite was accepted.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub accepted_at: Option<i64>,
    /// The projects that were granted membership upon acceptance of the invite.
    pub projects: Vec<InviteProject>,
}
