// Generated from OpenAPI spec. Do not edit by hand.
//! `DeletedConversationResource` DTO.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeletedConversationResource {
    pub object: String,
    pub deleted: bool,
    pub id: String,
}
