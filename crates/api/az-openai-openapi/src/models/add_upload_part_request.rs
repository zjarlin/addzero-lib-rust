// Generated from OpenAPI spec. Do not edit by hand.
//! `AddUploadPartRequest` DTO.

use serde::{Deserialize, Serialize};

use crate::bodies::{
    OpenAiBinaryBody,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddUploadPartRequest {
    /// The chunk of bytes for this Part.
    pub data: OpenAiBinaryBody,
}
