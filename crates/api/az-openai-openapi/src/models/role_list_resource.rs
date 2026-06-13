// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `RoleListResource` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    AssignedRoleDetails,
};

/// Paginated list of roles assigned to a principal.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoleListResource {
    /// Always `list`.
    pub object: String,
    /// Role assignments returned in the current page.
    pub data: Vec<AssignedRoleDetails>,
    /// Whether additional assignments are available when paginating.
    pub has_more: bool,
    /// Cursor to fetch the next page of results, or `null` when there are no more assignments.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub next: Option<String>,
}
