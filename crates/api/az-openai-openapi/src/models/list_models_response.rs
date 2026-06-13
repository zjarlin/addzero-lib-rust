// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `ListModelsResponse` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    Model,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListModelsResponse {
    pub object: String,
    pub data: Vec<Model>,
}
