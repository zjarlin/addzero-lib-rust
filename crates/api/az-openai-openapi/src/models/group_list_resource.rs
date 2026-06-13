// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `GroupListResource` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    GroupResponse,
};

/// Paginated list of organization groups.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupListResource {
    /// Always `list`.
    pub object: String,
    /// Groups returned in the current page.
    pub data: Vec<GroupResponse>,
    /// Whether additional groups are available when paginating.
    pub has_more: bool,
    /// Cursor to fetch the next page of results, or `null` if there are no more results.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub next: Option<String>,
}
