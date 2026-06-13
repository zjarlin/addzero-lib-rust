// Generated from OpenAPI spec. Do not edit by hand.
//! `DeletedConversation` DTO.

use serde::{Deserialize, Serialize};

/// The deleted conversation object
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeletedConversation {
    pub object: String,
    pub deleted: bool,
    pub id: String,
}
