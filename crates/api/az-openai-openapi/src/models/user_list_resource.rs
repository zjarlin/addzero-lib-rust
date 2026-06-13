// Generated from OpenAPI spec. Do not edit by hand.
//! `UserListResource` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    GroupUser,
};

/// Paginated list of user objects returned when inspecting group membership.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserListResource {
    /// Always `list`.
    pub object: String,
    /// Users in the current page.
    pub data: Vec<GroupUser>,
    /// Whether more users are available when paginating.
    pub has_more: bool,
    /// Cursor to fetch the next page of results, or `null` when no further users are available.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub next: Option<String>,
}
