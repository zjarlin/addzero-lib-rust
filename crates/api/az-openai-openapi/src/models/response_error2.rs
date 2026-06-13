// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `ResponseError2` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    ResponseErrorCode,
};

/// An error object returned when the model fails to generate a Response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseError2 {
    pub code: ResponseErrorCode,
    /// A human-readable description of the error.
    pub message: String,
}
