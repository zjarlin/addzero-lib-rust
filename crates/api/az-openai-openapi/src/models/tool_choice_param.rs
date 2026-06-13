// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `ToolChoiceParam` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    SpecificApplyPatchParam,
    SpecificFunctionShellParam,
    ToolChoiceAllowed,
    ToolChoiceCustom,
    ToolChoiceFunction,
    ToolChoiceMCP,
    ToolChoiceOptions,
    ToolChoiceTypes,
};

/// How the model should select which tool (or tools) to use when generating a response. See the `tools`
/// parameter to see how to specify which tools the model can call.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ToolChoiceParam {
    ToolChoiceOptions(ToolChoiceOptions),
    ToolChoiceAllowed(ToolChoiceAllowed),
    ToolChoiceTypes(ToolChoiceTypes),
    ToolChoiceFunction(ToolChoiceFunction),
    ToolChoiceMCP(ToolChoiceMCP),
    ToolChoiceCustom(ToolChoiceCustom),
    SpecificApplyPatchParam(SpecificApplyPatchParam),
    SpecificFunctionShellParam(SpecificFunctionShellParam),
}
