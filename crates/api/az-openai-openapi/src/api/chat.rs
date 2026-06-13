// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! Chat REST endpoint contract.

use async_trait::async_trait;

use crate::models::{
    ChatCompletionDeleted,
    ChatCompletionList,
    ChatCompletionMessageList,
    CreateChatCompletionRequest,
    CreateChatCompletionResponse,
    Metadata,
    UpdateChatCompletionRequest,
};

/// Chat REST endpoints.
#[async_trait]
pub trait OpenAiChatApi: Send + Sync {
    /// Error type returned by the application-layer implementation.
    type Error: std::error::Error + Send + Sync + 'static;

    /// List stored Chat Completions. Only Chat Completions that have been stored with the `store` parameter
    /// set to `true` will be returned.
    ///
    /// REST: `GET /chat/completions`.
    /// Path constant: [`OpenAiApiPath::CHAT_BY_COMPLETIONS`](crate::paths::OpenAiApiPath::CHAT_BY_COMPLETIONS).
    async fn list_chat_completions(
        &self,
        model: Option<String>,
        metadata: Option<Metadata>,
        after: Option<String>,
        limit: Option<i32>,
        order: Option<String>,
    ) -> Result<ChatCompletionList, Self::Error>;

    /// **Starting a new project?** We recommend trying [Responses](/docs/api-reference/responses) to take
    /// advantage of the latest OpenAI platform features. Compare [Chat Completions with
    /// Responses](/docs/guides/responses-vs-chat-completions?api-mode=responses). --- Creates a model
    /// response for the given chat conversation. Learn more in the [text generation](/docs/guides/text-
    /// generation), [vision](/docs/guides/vision), and [audio](/docs/guides/audio) guides. Parameter
    /// support can differ depending on the model used to generate the response, particularly for newer
    /// reasoning models. Parameters that are only supported for reasoning models are noted below. For the
    /// current state of unsupported parameters in reasoning models, [refer to the reasoning
    /// guide](/docs/guides/reasoning). Returns a chat completion object, or a streamed sequence of chat
    /// completion chunk objects if the request is streamed.
    ///
    /// REST: `POST /chat/completions`.
    /// Path constant: [`OpenAiApiPath::CHAT_BY_COMPLETIONS`](crate::paths::OpenAiApiPath::CHAT_BY_COMPLETIONS).
    async fn create_chat_completion(
        &self,
        body: CreateChatCompletionRequest,
    ) -> Result<CreateChatCompletionResponse, Self::Error>;

    /// Get a stored chat completion. Only Chat Completions that have been created with the `store`
    /// parameter set to `true` will be returned.
    ///
    /// REST: `GET /chat/completions/{completion_id}`.
    /// Path constant: [`OpenAiApiPath::CHAT_BY_COMPLETIONS_BY_COMPLETION_ID`](crate::paths::OpenAiApiPath::CHAT_BY_COMPLETIONS_BY_COMPLETION_ID).
    async fn get_chat_completion(
        &self,
        completion_id: String,
    ) -> Result<CreateChatCompletionResponse, Self::Error>;

    /// Modify a stored chat completion. Only Chat Completions that have been created with the `store`
    /// parameter set to `true` can be modified. Currently, the only supported modification is to update the
    /// `metadata` field.
    ///
    /// REST: `POST /chat/completions/{completion_id}`.
    /// Path constant: [`OpenAiApiPath::CHAT_BY_COMPLETIONS_BY_COMPLETION_ID`](crate::paths::OpenAiApiPath::CHAT_BY_COMPLETIONS_BY_COMPLETION_ID).
    async fn update_chat_completion(
        &self,
        completion_id: String,
        body: UpdateChatCompletionRequest,
    ) -> Result<CreateChatCompletionResponse, Self::Error>;

    /// Delete a stored chat completion. Only Chat Completions that have been created with the `store`
    /// parameter set to `true` can be deleted.
    ///
    /// REST: `DELETE /chat/completions/{completion_id}`.
    /// Path constant: [`OpenAiApiPath::CHAT_BY_COMPLETIONS_BY_COMPLETION_ID`](crate::paths::OpenAiApiPath::CHAT_BY_COMPLETIONS_BY_COMPLETION_ID).
    async fn delete_chat_completion(
        &self,
        completion_id: String,
    ) -> Result<ChatCompletionDeleted, Self::Error>;

    /// Get the messages in a stored chat completion. Only Chat Completions that have been created with the
    /// `store` parameter set to `true` will be returned.
    ///
    /// REST: `GET /chat/completions/{completion_id}/messages`.
    /// Path constant: [`OpenAiApiPath::CHAT_BY_COMPLETIONS_BY_COMPLETION_ID_BY_MESSAGES`](crate::paths::OpenAiApiPath::CHAT_BY_COMPLETIONS_BY_COMPLETION_ID_BY_MESSAGES).
    async fn get_chat_completion_messages(
        &self,
        completion_id: String,
        after: Option<String>,
        limit: Option<i32>,
        order: Option<String>,
    ) -> Result<ChatCompletionMessageList, Self::Error>;
}
