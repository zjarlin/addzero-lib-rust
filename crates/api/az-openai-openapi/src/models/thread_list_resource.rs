// Generated from OpenAPI spec. Do not edit by hand.
//! `ThreadListResource` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    ThreadResource,
};

/// A paginated list of ChatKit threads.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThreadListResource {
    /// The type of object returned, must be `list`.
    pub object: String,
    /// A list of items
    pub data: Vec<ThreadResource>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub first_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_id: Option<String>,
    /// Whether there are more items available.
    pub has_more: bool,
}
