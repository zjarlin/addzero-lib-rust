// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `CreateVideoEditMultipartBodyVideo` DTO.

use serde::{Deserialize, Serialize};

use crate::bodies::{
    OpenAiBinaryBody,
};

use crate::models::{
    VideoReferenceInputParam,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum CreateVideoEditMultipartBodyVideo {
    String(OpenAiBinaryBody),
    VideoReferenceInputParam(VideoReferenceInputParam),
}
