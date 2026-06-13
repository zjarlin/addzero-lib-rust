// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `PublicRoleListResource` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    Role,
};

/// Paginated list of roles available on an organization or project.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublicRoleListResource {
    /// Always `list`.
    pub object: String,
    /// Roles returned in the current page.
    pub data: Vec<Role>,
    /// Whether more roles are available when paginating.
    pub has_more: bool,
    /// Cursor to fetch the next page of results, or `null` when there are no additional roles.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub next: Option<String>,
}
