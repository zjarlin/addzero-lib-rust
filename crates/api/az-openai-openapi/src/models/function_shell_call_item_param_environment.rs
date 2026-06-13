// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `FunctionShellCallItemParamEnvironment` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    ContainerReferenceParam,
    LocalEnvironmentParam,
};

/// The environment to execute the shell commands in.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum FunctionShellCallItemParamEnvironment {
    LocalEnvironmentParam(LocalEnvironmentParam),
    ContainerReferenceParam(ContainerReferenceParam),
}
