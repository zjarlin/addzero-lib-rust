// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `GroupRoleAssignment` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    Group,
    Role,
};

/// Role assignment linking a group to a role.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupRoleAssignment {
    /// Always `group.role`.
    pub object: String,
    pub group: Group,
    pub role: Role,
}
