// Generated from OpenAPI spec. Do not edit by hand.
//! `ProjectApiKeyDeleteResponse` DTO.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectApiKeyDeleteResponse {
    pub object: String,
    pub id: String,
    pub deleted: bool,
}
