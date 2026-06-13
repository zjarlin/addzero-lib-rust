// Generated from OpenAPI spec. Do not edit by hand.
//! `VideoReferenceInputParam` DTO.

use serde::{Deserialize, Serialize};

/// Reference to the completed video.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VideoReferenceInputParam {
    /// The identifier of the completed video.
    pub id: String,
}
