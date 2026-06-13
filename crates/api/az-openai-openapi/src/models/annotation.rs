// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `Annotation` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    ContainerFileCitationBody,
    FileCitationBody,
    FilePath,
    UrlCitationBody,
};

/// An annotation that applies to a span of output text.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Annotation {
    FileCitationBody(FileCitationBody),
    UrlCitationBody(UrlCitationBody),
    ContainerFileCitationBody(ContainerFileCitationBody),
    FilePath(FilePath),
}
