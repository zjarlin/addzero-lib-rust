// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `RealtimeConversationItem` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    RealtimeConversationItemFunctionCall,
    RealtimeConversationItemFunctionCallOutput,
    RealtimeConversationItemMessageAssistant,
    RealtimeConversationItemMessageSystem,
    RealtimeConversationItemMessageUser,
    RealtimeMCPApprovalRequest,
    RealtimeMCPApprovalResponse,
    RealtimeMCPListTools,
    RealtimeMCPToolCall,
};

/// A single item within a Realtime conversation.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum RealtimeConversationItem {
    RealtimeConversationItemMessageSystem(RealtimeConversationItemMessageSystem),
    RealtimeConversationItemMessageUser(RealtimeConversationItemMessageUser),
    RealtimeConversationItemMessageAssistant(RealtimeConversationItemMessageAssistant),
    RealtimeConversationItemFunctionCall(RealtimeConversationItemFunctionCall),
    RealtimeConversationItemFunctionCallOutput(RealtimeConversationItemFunctionCallOutput),
    RealtimeMCPApprovalResponse(RealtimeMCPApprovalResponse),
    RealtimeMCPListTools(RealtimeMCPListTools),
    RealtimeMCPToolCall(RealtimeMCPToolCall),
    RealtimeMCPApprovalRequest(RealtimeMCPApprovalRequest),
}
