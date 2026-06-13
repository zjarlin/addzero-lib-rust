// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `ErrorEvent` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    Error,
};

/// Occurs when an [error](/docs/guides/error-codes#api-errors) occurs. This can happen due to an
/// internal server error or a timeout.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorEvent {
    pub event: String,
    pub data: Error,
}
