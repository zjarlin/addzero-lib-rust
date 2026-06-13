// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `ItemField` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    ApplyPatchToolCall,
    ApplyPatchToolCallOutput,
    CodeInterpreterToolCall,
    CompactionBody,
    ComputerToolCall,
    ComputerToolCallOutputResource,
    CustomToolCall,
    CustomToolCallOutput,
    FileSearchToolCall,
    FunctionShellCall,
    FunctionShellCallOutput,
    FunctionToolCall,
    FunctionToolCallOutput,
    ImageGenToolCall,
    LocalShellToolCall,
    LocalShellToolCallOutput,
    MCPApprovalRequest,
    MCPApprovalResponseResource,
    MCPListTools,
    MCPToolCall,
    Message,
    ReasoningItem,
    ToolSearchCall,
    ToolSearchOutput,
    WebSearchToolCall,
};

/// An item representing a message, tool call, tool output, reasoning, or other response element.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ItemField {
    Message(Message),
    FunctionToolCall(FunctionToolCall),
    ToolSearchCall(ToolSearchCall),
    ToolSearchOutput(ToolSearchOutput),
    FunctionToolCallOutput(FunctionToolCallOutput),
    FileSearchToolCall(FileSearchToolCall),
    WebSearchToolCall(WebSearchToolCall),
    ImageGenToolCall(ImageGenToolCall),
    ComputerToolCall(ComputerToolCall),
    ComputerToolCallOutputResource(ComputerToolCallOutputResource),
    ReasoningItem(ReasoningItem),
    CompactionBody(CompactionBody),
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
    CustomToolCall(CustomToolCall),
    CustomToolCallOutput(CustomToolCallOutput),
}
