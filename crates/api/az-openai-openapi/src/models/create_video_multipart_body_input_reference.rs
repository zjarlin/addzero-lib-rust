// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `CreateVideoMultipartBodyInputReference` DTO.

use serde::{Deserialize, Serialize};

use crate::bodies::{
    OpenAiBinaryBody,
};

use crate::models::{
    ImageRefParam2,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum CreateVideoMultipartBodyInputReference {
    String(OpenAiBinaryBody),
    ImageRefParam2(ImageRefParam2),
}
