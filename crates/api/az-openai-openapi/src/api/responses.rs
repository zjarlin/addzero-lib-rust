// Generated from OpenAPI spec. Do not edit by hand.
//! Responses REST endpoint contract.

use async_trait::async_trait;

use crate::models::{
    CompactResource,
    CompactResponseMethodPublicBody,
    CreateResponse,
    IncludeEnum,
    Response,
    ResponseItemList,
    TokenCountsBody,
    TokenCountsResource,
};

/// Responses REST endpoints.
#[async_trait]
pub trait OpenAiResponsesApi: Send + Sync {
    /// Error type returned by the application-layer implementation.
    type Error: std::error::Error + Send + Sync + 'static;

    /// Creates a model response. Provide [text](/docs/guides/text) or [image](/docs/guides/images) inputs
    /// to generate [text](/docs/guides/text) or [JSON](/docs/guides/structured-outputs) outputs. Have the
    /// model call your own [custom code](/docs/guides/function-calling) or use built-in
    /// [tools](/docs/guides/tools) like [web search](/docs/guides/tools-web-search) or [file
    /// search](/docs/guides/tools-file-search) to use your own data as input for the model's response.
    ///
    /// REST: `POST /responses`.
    /// Path constant: [`OpenAiApiPath::RESPONSES`](crate::paths::OpenAiApiPath::RESPONSES).
    async fn create_response(&self, body: CreateResponse) -> Result<Response, Self::Error>;

    /// Retrieves a model response with the given ID.
    ///
    /// REST: `GET /responses/{response_id}`.
    /// Path constant: [`OpenAiApiPath::RESPONSES_BY_RESPONSE_ID`](crate::paths::OpenAiApiPath::RESPONSES_BY_RESPONSE_ID).
    async fn get_response(
        &self,
        response_id: String,
        include: Option<Vec<IncludeEnum>>,
        stream: Option<bool>,
        starting_after: Option<i32>,
        include_obfuscation: Option<bool>,
    ) -> Result<Response, Self::Error>;

    /// Deletes a model response with the given ID.
    ///
    /// REST: `DELETE /responses/{response_id}`.
    /// Path constant: [`OpenAiApiPath::RESPONSES_BY_RESPONSE_ID`](crate::paths::OpenAiApiPath::RESPONSES_BY_RESPONSE_ID).
    async fn delete_response(&self, response_id: String) -> Result<(), Self::Error>;

    /// Cancels a model response with the given ID. Only responses created with the `background` parameter
    /// set to `true` can be cancelled. [Learn more](/docs/guides/background).
    ///
    /// REST: `POST /responses/{response_id}/cancel`.
    /// Path constant: [`OpenAiApiPath::RESPONSES_BY_RESPONSE_ID_BY_CANCEL`](crate::paths::OpenAiApiPath::RESPONSES_BY_RESPONSE_ID_BY_CANCEL).
    async fn cancel_response(&self, response_id: String) -> Result<Response, Self::Error>;

    /// Returns a list of input items for a given response.
    ///
    /// REST: `GET /responses/{response_id}/input_items`.
    /// Path constant: [`OpenAiApiPath::RESPONSES_BY_RESPONSE_ID_BY_INPUT_ITEMS`](crate::paths::OpenAiApiPath::RESPONSES_BY_RESPONSE_ID_BY_INPUT_ITEMS).
    async fn list_input_items(
        &self,
        response_id: String,
        limit: Option<i32>,
        order: Option<String>,
        after: Option<String>,
        include: Option<Vec<IncludeEnum>>,
    ) -> Result<ResponseItemList, Self::Error>;

    /// Returns input token counts of the request. Returns an object with `object` set to
    /// `response.input_tokens` and an `input_tokens` count.
    ///
    /// REST: `POST /responses/input_tokens`.
    /// Path constant: [`OpenAiApiPath::RESPONSES_BY_INPUT_TOKENS`](crate::paths::OpenAiApiPath::RESPONSES_BY_INPUT_TOKENS).
    async fn get_input_token_counts(
        &self,
        body: Option<TokenCountsBody>,
    ) -> Result<TokenCountsResource, Self::Error>;

    /// Compact a conversation. Returns a compacted response object. Learn when and how to compact long-
    /// running conversations in the [conversation state guide](/docs/guides/conversation-state#managing-
    /// the-context-window). For ZDR-compatible compaction details, see [Compaction
    /// (advanced)](/docs/guides/conversation-state#compaction-advanced).
    ///
    /// REST: `POST /responses/compact`.
    /// Path constant: [`OpenAiApiPath::RESPONSES_BY_COMPACT`](crate::paths::OpenAiApiPath::RESPONSES_BY_COMPACT).
    async fn compact_conversation(
        &self,
        body: Option<CompactResponseMethodPublicBody>,
    ) -> Result<CompactResource, Self::Error>;
}
