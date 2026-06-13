// Generated from OpenAPI spec. Do not edit by hand.
//! `UpdateConversationBody` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    Metadata,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateConversationBody {
    /// Set of 16 key-value pairs that can be attached to an object. This can be useful for storing
    /// additional information about the object in a structured format, and querying for objects via API or
    /// the dashboard. Keys are strings with a maximum length of 64 characters. Values are strings with a
    /// maximum length of 512 characters.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub metadata: Option<Metadata>,
}
