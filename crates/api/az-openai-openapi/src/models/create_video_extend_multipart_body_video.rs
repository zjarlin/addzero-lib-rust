// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `CreateVideoExtendMultipartBodyVideo` DTO.

use serde::{Deserialize, Serialize};

use crate::bodies::{
    OpenAiBinaryBody,
};

use crate::models::{
    VideoReferenceInputParam,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum CreateVideoExtendMultipartBodyVideo {
    VideoReferenceInputParam(VideoReferenceInputParam),
    String(OpenAiBinaryBody),
}
