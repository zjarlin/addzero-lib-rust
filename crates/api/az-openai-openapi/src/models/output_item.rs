// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `OutputItem` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    ApplyPatchToolCall,
    ApplyPatchToolCallOutput,
    CodeInterpreterToolCall,
    CompactionBody,
    ComputerToolCall,
    ComputerToolCallOutputResource,
    CustomToolCall,
    CustomToolCallOutputResource,
    FileSearchToolCall,
    FunctionShellCall,
    FunctionShellCallOutput,
    FunctionToolCall,
    FunctionToolCallOutputResource,
    ImageGenToolCall,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum OutputItem {
    OutputMessage(OutputMessage),
    FileSearchToolCall(FileSearchToolCall),
    FunctionToolCall(FunctionToolCall),
    FunctionToolCallOutputResource(FunctionToolCallOutputResource),
    WebSearchToolCall(WebSearchToolCall),
    ComputerToolCall(ComputerToolCall),
    ComputerToolCallOutputResource(ComputerToolCallOutputResource),
    ReasoningItem(ReasoningItem),
    ToolSearchCall(ToolSearchCall),
    ToolSearchOutput(ToolSearchOutput),
    CompactionBody(CompactionBody),
    ImageGenToolCall(ImageGenToolCall),
    CodeInterpreterToolCall(CodeInterpreterToolCall),
    LocalShellToolCall(LocalShellToolCall),
    LocalShellToolCallOutput(LocalShellToolCallOutput),
    FunctionShellCall(FunctionShellCall),
    FunctionShellCallOutput(FunctionShellCallOutput),
    ApplyPatchToolCall(ApplyPatchToolCall),
    ApplyPatchToolCallOutput(ApplyPatchToolCallOutput),
    MCPToolCall(MCPToolCall),
    MCPListTools(MCPListTools),
    MCPApprovalRequest(MCPApprovalRequest),
    MCPApprovalResponseResource(MCPApprovalResponseResource),
    CustomToolCall(CustomToolCall),
    CustomToolCallOutputResource(CustomToolCallOutputResource),
}
