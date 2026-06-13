// Generated from OpenAPI spec. Do not edit by hand.
//! `ThreadItem` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    AssistantMessageItem,
    ClientToolCallItem,
    TaskGroupItem,
    TaskItem,
    UserMessageItem,
    WidgetMessageItem,
};

/// The thread item
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ThreadItem {
    UserMessageItem(UserMessageItem),
    AssistantMessageItem(AssistantMessageItem),
    WidgetMessageItem(WidgetMessageItem),
    ClientToolCallItem(ClientToolCallItem),
    TaskItem(TaskItem),
    TaskGroupItem(TaskGroupItem),
}
