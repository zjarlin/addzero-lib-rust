// Generated from OpenAPI spec. Do not edit by hand.
//! `FunctionShellToolParamEnvironment` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    ContainerAutoParam,
    ContainerReferenceParam,
    LocalEnvironmentParam,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum FunctionShellToolParamEnvironment {
    ContainerAutoParam(ContainerAutoParam),
    LocalEnvironmentParam(LocalEnvironmentParam),
    ContainerReferenceParam(ContainerReferenceParam),
}
