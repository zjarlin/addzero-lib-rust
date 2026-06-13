// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `ResponseOutputTextAnnotation` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    FileAnnotation,
    UrlAnnotation,
};

/// Annotation object describing a cited source.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ResponseOutputTextAnnotation {
    FileAnnotation(FileAnnotation),
    UrlAnnotation(UrlAnnotation),
}
