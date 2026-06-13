// Generated from OpenAPI spec. Do not edit by hand.
//! `CreateModerationResponse` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    CreateModerationResponseResult,
};

/// Represents if a given text input is potentially harmful.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateModerationResponse {
    /// The unique identifier for the moderation request.
    pub id: String,
    /// The model used to generate the moderation results.
    pub model: String,
    /// A list of moderation objects.
    pub results: Vec<CreateModerationResponseResult>,
}
