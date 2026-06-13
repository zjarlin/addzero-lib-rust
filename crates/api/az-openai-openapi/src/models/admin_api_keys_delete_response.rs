// Generated from OpenAPI spec. Do not edit by hand.
//! `AdminApiKeysDeleteResponse` DTO.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdminApiKeysDeleteResponse {
    pub id: String,
    pub object: String,
    pub deleted: bool,
}
