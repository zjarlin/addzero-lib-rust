// Generated from OpenAPI spec. Do not edit by hand.
//! Chatkit REST endpoint contract.

use async_trait::async_trait;

use crate::models::{
    ChatSessionResource,
    CreateChatSessionBody,
    DeletedThreadResource,
    OrderEnum,
    ThreadItemListResource,
    ThreadListResource,
    ThreadResource,
};

/// Chatkit REST endpoints.
#[async_trait]
pub trait OpenAiChatkitApi: Send + Sync {
    /// Error type returned by the application-layer implementation.
    type Error: std::error::Error + Send + Sync + 'static;

    /// Cancel an active ChatKit session and return its most recent metadata. Cancelling prevents new
    /// requests from using the issued client secret.
    ///
    /// REST: `POST /chatkit/sessions/{session_id}/cancel`.
    /// Path constant: [`OpenAiApiPath::CHATKIT_BY_SESSIONS_BY_SESSION_ID_BY_CANCEL`](crate::paths::OpenAiApiPath::CHATKIT_BY_SESSIONS_BY_SESSION_ID_BY_CANCEL).
    async fn cancel_chat_session_method(
        &self,
        session_id: String,
    ) -> Result<ChatSessionResource, Self::Error>;

    /// Create a ChatKit session.
    ///
    /// REST: `POST /chatkit/sessions`.
    /// Path constant: [`OpenAiApiPath::CHATKIT_BY_SESSIONS`](crate::paths::OpenAiApiPath::CHATKIT_BY_SESSIONS).
    async fn create_chat_session_method(
        &self,
        body: Option<CreateChatSessionBody>,
    ) -> Result<ChatSessionResource, Self::Error>;

    /// List items that belong to a ChatKit thread.
    ///
    /// REST: `GET /chatkit/threads/{thread_id}/items`.
    /// Path constant: [`OpenAiApiPath::CHATKIT_BY_THREADS_BY_THREAD_ID_BY_ITEMS`](crate::paths::OpenAiApiPath::CHATKIT_BY_THREADS_BY_THREAD_ID_BY_ITEMS).
    async fn list_thread_items_method(
        &self,
        thread_id: String,
        limit: Option<i32>,
        order: Option<OrderEnum>,
        after: Option<String>,
        before: Option<String>,
    ) -> Result<ThreadItemListResource, Self::Error>;

    /// Retrieve a ChatKit thread by its identifier.
    ///
    /// REST: `GET /chatkit/threads/{thread_id}`.
    /// Path constant: [`OpenAiApiPath::CHATKIT_BY_THREADS_BY_THREAD_ID`](crate::paths::OpenAiApiPath::CHATKIT_BY_THREADS_BY_THREAD_ID).
    async fn get_thread_method(&self, thread_id: String) -> Result<ThreadResource, Self::Error>;

    /// Delete a ChatKit thread along with its items and stored attachments.
    ///
    /// REST: `DELETE /chatkit/threads/{thread_id}`.
    /// Path constant: [`OpenAiApiPath::CHATKIT_BY_THREADS_BY_THREAD_ID`](crate::paths::OpenAiApiPath::CHATKIT_BY_THREADS_BY_THREAD_ID).
    async fn delete_thread_method(
        &self,
        thread_id: String,
    ) -> Result<DeletedThreadResource, Self::Error>;

    /// List ChatKit threads with optional pagination and user filters.
    ///
    /// REST: `GET /chatkit/threads`.
    /// Path constant: [`OpenAiApiPath::CHATKIT_BY_THREADS`](crate::paths::OpenAiApiPath::CHATKIT_BY_THREADS).
    async fn list_threads_method(
        &self,
        limit: Option<i32>,
        order: Option<OrderEnum>,
        after: Option<String>,
        before: Option<String>,
        user: Option<String>,
    ) -> Result<ThreadListResource, Self::Error>;
}
