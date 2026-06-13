// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `UserRoleAssignment` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    Role,
    User,
};

/// Role assignment linking a user to a role.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserRoleAssignment {
    /// Always `user.role`.
    pub object: String,
    pub user: User,
    pub role: Role,
}
