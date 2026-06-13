// Generated from OpenAPI spec. Do not edit by hand.
//! `CreateContainerFileBody` DTO.

use serde::{Deserialize, Serialize};

use crate::bodies::{
    OpenAiBinaryBody,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateContainerFileBody {
    /// Name of the file to create.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub file_id: Option<String>,
    /// The File object (not file name) to be uploaded.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub file: Option<OpenAiBinaryBody>,
}
