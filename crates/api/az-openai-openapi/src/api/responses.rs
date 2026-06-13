//! Responses REST endpoint contract.

use async_trait::async_trait;

use crate::bodies::*;

/// Responses REST endpoints.
#[async_trait]
pub trait OpenAiResponsesApi: Send + Sync {
    /// Error type returned by the application-layer implementation.
    type Error: std::error::Error + Send + Sync + 'static;
    /// Creates a model response. Provide [text](/docs/guides/text) or [image](/docs/guides/images) inputs to generate [text](/docs/guides/text) or [JSON](/docs/guides/structured-outputs) outputs. Have the model call your own [custom code](/docs/guides/function-calling) or use built-in [tools](/docs/guides/tools) like [web search](/docs/guides/tools-web-search) or [file search](/docs/guides/tools-file-search) to use your own data as input for the model's response.
    ///
    /// REST: `POST /responses`.
    /// Path constant: [`OpenAiApiPath::RESPONSES`](crate::paths::OpenAiApiPath::RESPONSES).
    async fn create_response(
        &self,
        body: OpenAiRequestBody,
    ) -> Result<OpenAiResponseBody, Self::Error>;

    /// Compact a conversation. Returns a compacted response object. Learn when and how to compact long-running conversations in the [conversation state guide](/docs/guides/conversation-state#managing-the-context-window). For ZDR-compatible compaction details, see [Compaction (advanced)](/docs/guides/conversation-state#compaction-advanced).
    ///
    /// REST: `POST /responses/compact`.
    /// Path constant: [`OpenAiApiPath::RESPONSES_BY_COMPACT`](crate::paths::OpenAiApiPath::RESPONSES_BY_COMPACT).
    async fn compact_conversation(
        &self,
        body: Option<OpenAiRequestBody>,
    ) -> Result<OpenAiResponseBody, Self::Error>;

    /// Returns input token counts of the request. Returns an object with `object` set to `response.input_tokens` and an `input_tokens` count.
    ///
    /// REST: `POST /responses/input_tokens`.
    /// Path constant: [`OpenAiApiPath::RESPONSES_BY_INPUT_TOKENS`](crate::paths::OpenAiApiPath::RESPONSES_BY_INPUT_TOKENS).
    async fn get_input_token_counts(
        &self,
        body: Option<OpenAiRequestBody>,
    ) -> Result<OpenAiResponseBody, Self::Error>;

    /// Deletes a model response with the given ID.
    ///
    /// REST: `DELETE /responses/{response_id}`.
    /// Path constant: [`OpenAiApiPath::RESPONSES_BY_RESPONSE_ID`](crate::paths::OpenAiApiPath::RESPONSES_BY_RESPONSE_ID).
    async fn delete_response(&self, response_id: String)
    -> Result<OpenAiResponseBody, Self::Error>;

    /// Retrieves a model response with the given ID.
    ///
    /// REST: `GET /responses/{response_id}`.
    /// Path constant: [`OpenAiApiPath::RESPONSES_BY_RESPONSE_ID`](crate::paths::OpenAiApiPath::RESPONSES_BY_RESPONSE_ID).
    async fn get_response(
        &self,
        response_id: String,
        include: Option<Vec<String>>,
        stream: Option<bool>,
        starting_after: Option<i64>,
        include_obfuscation: Option<bool>,
    ) -> Result<OpenAiResponseBody, Self::Error>;

    /// Cancels a model response with the given ID. Only responses created with the `background` parameter set to `true` can be cancelled. [Learn more](/docs/guides/background).
    ///
    /// REST: `POST /responses/{response_id}/cancel`.
    /// Path constant: [`OpenAiApiPath::RESPONSES_BY_RESPONSE_ID_BY_CANCEL`](crate::paths::OpenAiApiPath::RESPONSES_BY_RESPONSE_ID_BY_CANCEL).
    async fn cancel_response(&self, response_id: String)
    -> Result<OpenAiResponseBody, Self::Error>;

    /// Returns a list of input items for a given response.
    ///
    /// REST: `GET /responses/{response_id}/input_items`.
    /// Path constant: [`OpenAiApiPath::RESPONSES_BY_RESPONSE_ID_BY_INPUT_ITEMS`](crate::paths::OpenAiApiPath::RESPONSES_BY_RESPONSE_ID_BY_INPUT_ITEMS).
    async fn list_input_items(
        &self,
        response_id: String,
        limit: Option<i64>,
        order: Option<String>,
        after: Option<String>,
        include: Option<Vec<String>>,
    ) -> Result<OpenAiResponseBody, Self::Error>;
}
