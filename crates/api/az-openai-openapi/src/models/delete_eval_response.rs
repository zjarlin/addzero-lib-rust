// Generated from OpenAPI spec. Do not edit by hand.
//! `DeleteEvalResponse` DTO.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeleteEvalResponse {
    pub object: String,
    pub deleted: bool,
    pub eval_id: String,
}
