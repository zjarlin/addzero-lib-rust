// Generated from OpenAPI spec. Do not edit by hand.
//! `DeletedSkillVersionResource` DTO.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeletedSkillVersionResource {
    pub object: String,
    pub deleted: bool,
    pub id: String,
    /// The deleted skill version.
    pub version: String,
}
