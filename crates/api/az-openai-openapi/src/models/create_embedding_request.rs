// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `CreateEmbeddingRequest` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    CreateEmbeddingRequestInput,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateEmbeddingRequest {
    /// Input text to embed, encoded as a string or array of tokens. To embed multiple inputs in a single
    /// request, pass an array of strings or array of token arrays. The input must not exceed the max input
    /// tokens for the model (8192 tokens for all embedding models), cannot be an empty string, and any
    /// array must be 2048 dimensions or less. [Example Python
    /// code](https://cookbook.openai.com/examples/how_to_count_tokens_with_tiktoken) for counting tokens.
    /// In addition to the per-input token limit, all embedding models enforce a maximum of 300,000 tokens
    /// summed across all inputs in a single request.
    pub input: CreateEmbeddingRequestInput,
    /// ID of the model to use. You can use the [List models](/docs/api-reference/models/list) API to see
    /// all of your available models, or see our [Model overview](/docs/models) for descriptions of them.
    pub model: String,
    /// The format to return the embeddings in. Can be either `float` or
    /// [`base64`](https://pypi.org/project/pybase64/).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub encoding_format: Option<String>,
    /// The number of dimensions the resulting output embeddings should have. Only supported in `text-
    /// embedding-3` and later models.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dimensions: Option<i32>,
    /// A unique identifier representing your end-user, which can help OpenAI to monitor and detect abuse.
    /// [Learn more](/docs/guides/safety-best-practices#end-user-ids).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub user: Option<String>,
}
