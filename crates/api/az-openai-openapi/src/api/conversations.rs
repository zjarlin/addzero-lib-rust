// Generated from OpenAPI spec. Do not edit by hand.
//! Conversations REST endpoint contract.

use async_trait::async_trait;

use crate::models::{
    ConversationItem,
    ConversationItemList,
    ConversationResource,
    CreateConversationBody,
    CreateConversationItemsRequest,
    DeletedConversationResource,
    IncludeEnum,
    UpdateConversationBody,
};

/// Conversations REST endpoints.
#[async_trait]
pub trait OpenAiConversationsApi: Send + Sync {
    /// Error type returned by the application-layer implementation.
    type Error: std::error::Error + Send + Sync + 'static;

    /// List all items for a conversation with the given ID.
    ///
    /// REST: `GET /conversations/{conversation_id}/items`.
    /// Path constant: [`OpenAiApiPath::CONVERSATIONS_BY_CONVERSATION_ID_BY_ITEMS`](crate::paths::OpenAiApiPath::CONVERSATIONS_BY_CONVERSATION_ID_BY_ITEMS).
    async fn list_conversation_items(
        &self,
        conversation_id: String,
        limit: Option<i32>,
        order: Option<String>,
        after: Option<String>,
        include: Option<Vec<IncludeEnum>>,
    ) -> Result<ConversationItemList, Self::Error>;

    /// Create items in a conversation with the given ID.
    ///
    /// REST: `POST /conversations/{conversation_id}/items`.
    /// Path constant: [`OpenAiApiPath::CONVERSATIONS_BY_CONVERSATION_ID_BY_ITEMS`](crate::paths::OpenAiApiPath::CONVERSATIONS_BY_CONVERSATION_ID_BY_ITEMS).
    async fn create_conversation_items(
        &self,
        conversation_id: String,
        include: Option<Vec<IncludeEnum>>,
        body: CreateConversationItemsRequest,
    ) -> Result<ConversationItemList, Self::Error>;

    /// Get a single item from a conversation with the given IDs.
    ///
    /// REST: `GET /conversations/{conversation_id}/items/{item_id}`.
    /// Path constant: [`OpenAiApiPath::CONVERSATIONS_BY_CONVERSATION_ID_BY_ITEMS_BY_ITEM_ID`](crate::paths::OpenAiApiPath::CONVERSATIONS_BY_CONVERSATION_ID_BY_ITEMS_BY_ITEM_ID).
    async fn get_conversation_item(
        &self,
        conversation_id: String,
        item_id: String,
        include: Option<Vec<IncludeEnum>>,
    ) -> Result<ConversationItem, Self::Error>;

    /// Delete an item from a conversation with the given IDs.
    ///
    /// REST: `DELETE /conversations/{conversation_id}/items/{item_id}`.
    /// Path constant: [`OpenAiApiPath::CONVERSATIONS_BY_CONVERSATION_ID_BY_ITEMS_BY_ITEM_ID`](crate::paths::OpenAiApiPath::CONVERSATIONS_BY_CONVERSATION_ID_BY_ITEMS_BY_ITEM_ID).
    async fn delete_conversation_item(
        &self,
        conversation_id: String,
        item_id: String,
    ) -> Result<ConversationResource, Self::Error>;

    /// Create a conversation.
    ///
    /// REST: `POST /conversations`.
    /// Path constant: [`OpenAiApiPath::CONVERSATIONS`](crate::paths::OpenAiApiPath::CONVERSATIONS).
    async fn create_conversation(
        &self,
        body: Option<CreateConversationBody>,
    ) -> Result<ConversationResource, Self::Error>;

    /// Get a conversation
    ///
    /// REST: `GET /conversations/{conversation_id}`.
    /// Path constant: [`OpenAiApiPath::CONVERSATIONS_BY_CONVERSATION_ID`](crate::paths::OpenAiApiPath::CONVERSATIONS_BY_CONVERSATION_ID).
    async fn get_conversation(
        &self,
        conversation_id: String,
    ) -> Result<ConversationResource, Self::Error>;

    /// Update a conversation
    ///
    /// REST: `POST /conversations/{conversation_id}`.
    /// Path constant: [`OpenAiApiPath::CONVERSATIONS_BY_CONVERSATION_ID`](crate::paths::OpenAiApiPath::CONVERSATIONS_BY_CONVERSATION_ID).
    async fn update_conversation(
        &self,
        conversation_id: String,
        body: Option<UpdateConversationBody>,
    ) -> Result<ConversationResource, Self::Error>;

    /// Delete a conversation. Items in the conversation will not be deleted.
    ///
    /// REST: `DELETE /conversations/{conversation_id}`.
    /// Path constant: [`OpenAiApiPath::CONVERSATIONS_BY_CONVERSATION_ID`](crate::paths::OpenAiApiPath::CONVERSATIONS_BY_CONVERSATION_ID).
    async fn delete_conversation(
        &self,
        conversation_id: String,
    ) -> Result<DeletedConversationResource, Self::Error>;
}
