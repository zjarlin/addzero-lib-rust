// Generated from OpenAPI spec. Do not edit by hand.
//! `InviteDeleteResponse` DTO.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InviteDeleteResponse {
    /// The object type, which is always `organization.invite.deleted`
    pub object: String,
    pub id: String,
    pub deleted: bool,
}
