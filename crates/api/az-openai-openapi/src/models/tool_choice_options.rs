// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `ToolChoiceOptions` DTO.


/// Controls which (if any) tool is called by the model. `none` means the model will not call any tool
/// and instead generates a message. `auto` means the model can pick between generating a message or
/// calling one or more tools. `required` means the model must call one or more tools.
pub type ToolChoiceOptions = String;
