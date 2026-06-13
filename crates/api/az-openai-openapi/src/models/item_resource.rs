// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `ItemResource` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    ApplyPatchToolCall,
    ApplyPatchToolCallOutput,
    CodeInterpreterToolCall,
    CompactionBody,
    ComputerToolCall,
    ComputerToolCallOutputResource,
    CustomToolCallOutputResource,
    CustomToolCallResource,
    FileSearchToolCall,
    FunctionShellCall,
    FunctionShellCallOutput,
    FunctionToolCallOutputResource,
    FunctionToolCallResource,
    ImageGenToolCall,
    InputMessageResource,
    LocalShellToolCall,
    LocalShellToolCallOutput,
    MCPApprovalRequest,
    MCPApprovalResponseResource,
    MCPListTools,
    MCPToolCall,
    OutputMessage,
    ReasoningItem,
    ToolSearchCall,
    ToolSearchOutput,
    WebSearchToolCall,
};

/// Content item used to generate a response.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ItemResource {
    InputMessageResource(InputMessageResource),
    OutputMessage(OutputMessage),
    FileSearchToolCall(FileSearchToolCall),
    ComputerToolCall(ComputerToolCall),
    ComputerToolCallOutputResource(ComputerToolCallOutputResource),
    WebSearchToolCall(WebSearchToolCall),
    FunctionToolCallResource(FunctionToolCallResource),
    FunctionToolCallOutputResource(FunctionToolCallOutputResource),
    ToolSearchCall(ToolSearchCall),
    ToolSearchOutput(ToolSearchOutput),
    ReasoningItem(ReasoningItem),
    CompactionBody(CompactionBody),
    ImageGenToolCall(ImageGenToolCall),
    CodeInterpreterToolCall(CodeInterpreterToolCall),
    LocalShellToolCall(LocalShellToolCall),
    LocalShellToolCallOutput(LocalShellToolCallOutput),
    FunctionShellCall(FunctionShellCall),
    FunctionShellCallOutput(FunctionShellCallOutput),
    ApplyPatchToolCall(ApplyPatchToolCall),
    ApplyPatchToolCallOutput(ApplyPatchToolCallOutput),
    MCPListTools(MCPListTools),
    MCPApprovalRequest(MCPApprovalRequest),
    MCPApprovalResponseResource(MCPApprovalResponseResource),
    MCPToolCall(MCPToolCall),
    CustomToolCallResource(CustomToolCallResource),
    CustomToolCallOutputResource(CustomToolCallOutputResource),
}
