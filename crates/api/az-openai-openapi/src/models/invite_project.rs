// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `InviteProject` DTO.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InviteProject {
    /// Project's public ID
    pub id: String,
    /// Project membership role
    pub role: String,
}
