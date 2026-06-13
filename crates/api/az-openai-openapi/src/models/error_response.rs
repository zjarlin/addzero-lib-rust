// Generated from OpenAPI spec. Do not edit by hand.
//! `ErrorResponse` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    Error,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorResponse {
    pub error: Error,
}
