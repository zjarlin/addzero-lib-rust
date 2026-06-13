// Generated from OpenAPI spec. Do not edit by hand.
//! `InviteRequestProject` DTO.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InviteRequestProject {
    /// Project's public ID
    pub id: String,
    /// Project membership role
    pub role: String,
}
