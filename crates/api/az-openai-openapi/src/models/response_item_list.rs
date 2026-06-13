// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `ResponseItemList` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    ItemResource,
};

/// A list of Response items.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseItemList {
    /// The type of object returned, must be `list`.
    pub object: String,
    /// A list of items used to generate this response.
    pub data: Vec<ItemResource>,
    /// Whether there are more items available.
    pub has_more: bool,
    /// The ID of the first item in the list.
    pub first_id: String,
    /// The ID of the last item in the list.
    pub last_id: String,
}
