// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `Tool` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    ApplyPatchToolParam,
    CodeInterpreterTool,
    ComputerTool,
    ComputerUsePreviewTool,
    CustomToolParam,
    FileSearchTool,
    FunctionShellToolParam,
    FunctionTool,
    ImageGenTool,
    LocalShellToolParam,
    MCPTool,
    NamespaceToolParam,
    ToolSearchToolParam,
    WebSearchPreviewTool,
    WebSearchTool,
};

/// A tool that can be used to generate a response.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Tool {
    FunctionTool(FunctionTool),
    FileSearchTool(FileSearchTool),
    ComputerTool(ComputerTool),
    ComputerUsePreviewTool(ComputerUsePreviewTool),
    WebSearchTool(WebSearchTool),
    MCPTool(MCPTool),
    CodeInterpreterTool(CodeInterpreterTool),
    ImageGenTool(ImageGenTool),
    LocalShellToolParam(LocalShellToolParam),
    FunctionShellToolParam(FunctionShellToolParam),
    CustomToolParam(CustomToolParam),
    NamespaceToolParam(NamespaceToolParam),
    ToolSearchToolParam(ToolSearchToolParam),
    WebSearchPreviewTool(WebSearchPreviewTool),
    ApplyPatchToolParam(ApplyPatchToolParam),
}
