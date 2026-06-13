// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `FunctionShellCallEnvironment` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    ContainerReferenceResource,
    LocalEnvironmentResource,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum FunctionShellCallEnvironment {
    LocalEnvironmentResource(LocalEnvironmentResource),
    ContainerReferenceResource(ContainerReferenceResource),
}
