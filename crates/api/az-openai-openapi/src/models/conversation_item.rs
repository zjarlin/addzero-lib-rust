// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `ConversationItem` DTO.

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
    FunctionToolCallOutputResource,
    FunctionToolCallResource,
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

/// A single item within a conversation. The set of possible types are the same as the `output` type of
/// a [Response object](/docs/api-reference/responses/object#responses/object-output).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ConversationItem {
    Message(Message),
    FunctionToolCallResource(FunctionToolCallResource),
    FunctionToolCallOutputResource(FunctionToolCallOutputResource),
    FileSearchToolCall(FileSearchToolCall),
    WebSearchToolCall(WebSearchToolCall),
    ImageGenToolCall(ImageGenToolCall),
    ComputerToolCall(ComputerToolCall),
    ComputerToolCallOutputResource(ComputerToolCallOutputResource),
    ToolSearchCall(ToolSearchCall),
    ToolSearchOutput(ToolSearchOutput),
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
