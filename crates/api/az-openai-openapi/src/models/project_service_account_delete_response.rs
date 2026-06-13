// Generated from OpenAPI spec. Do not edit by hand.
//! `ProjectServiceAccountDeleteResponse` DTO.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectServiceAccountDeleteResponse {
    pub object: String,
    pub id: String,
    pub deleted: bool,
}
