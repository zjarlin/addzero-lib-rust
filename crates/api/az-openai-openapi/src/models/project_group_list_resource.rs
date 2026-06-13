// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `ProjectGroupListResource` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    ProjectGroup,
};

/// Paginated list of groups that have access to a project.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectGroupListResource {
    /// Always `list`.
    pub object: String,
    /// Project group memberships returned in the current page.
    pub data: Vec<ProjectGroup>,
    /// Whether additional project group memberships are available.
    pub has_more: bool,
    /// Cursor to fetch the next page of results, or `null` when there are no more results.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub next: Option<String>,
}
