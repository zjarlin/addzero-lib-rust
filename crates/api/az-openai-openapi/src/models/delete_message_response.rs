// Generated from OpenAPI spec. Do not edit by hand.
//! `DeleteMessageResponse` DTO.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeleteMessageResponse {
    pub id: String,
    pub deleted: bool,
    pub object: String,
}
