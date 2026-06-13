// Generated from OpenAPI spec. Do not edit by hand.
//! `DeletedSkillResource` DTO.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeletedSkillResource {
    pub object: String,
    pub deleted: bool,
    pub id: String,
}
