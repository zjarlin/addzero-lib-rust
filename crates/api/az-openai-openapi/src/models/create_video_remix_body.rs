// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `CreateVideoRemixBody` DTO.

use serde::{Deserialize, Serialize};

/// Parameters for remixing an existing generated video.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateVideoRemixBody {
    /// Updated text prompt that directs the remix generation.
    pub prompt: String,
}
