// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `RealtimeClientEvent` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    RealtimeClientEventConversationItemCreate,
    RealtimeClientEventConversationItemDelete,
    RealtimeClientEventConversationItemRetrieve,
    RealtimeClientEventConversationItemTruncate,
    RealtimeClientEventInputAudioBufferAppend,
    RealtimeClientEventInputAudioBufferClear,
    RealtimeClientEventInputAudioBufferCommit,
    RealtimeClientEventOutputAudioBufferClear,
    RealtimeClientEventResponseCancel,
    RealtimeClientEventResponseCreate,
    RealtimeClientEventSessionUpdate,
};

/// A realtime client event.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum RealtimeClientEvent {
    RealtimeClientEventConversationItemCreate(RealtimeClientEventConversationItemCreate),
    RealtimeClientEventConversationItemDelete(RealtimeClientEventConversationItemDelete),
    RealtimeClientEventConversationItemRetrieve(RealtimeClientEventConversationItemRetrieve),
    RealtimeClientEventConversationItemTruncate(RealtimeClientEventConversationItemTruncate),
    RealtimeClientEventInputAudioBufferAppend(RealtimeClientEventInputAudioBufferAppend),
    RealtimeClientEventInputAudioBufferClear(RealtimeClientEventInputAudioBufferClear),
    RealtimeClientEventOutputAudioBufferClear(RealtimeClientEventOutputAudioBufferClear),
    RealtimeClientEventInputAudioBufferCommit(RealtimeClientEventInputAudioBufferCommit),
    RealtimeClientEventResponseCancel(RealtimeClientEventResponseCancel),
    RealtimeClientEventResponseCreate(RealtimeClientEventResponseCreate),
    RealtimeClientEventSessionUpdate(RealtimeClientEventSessionUpdate),
}
