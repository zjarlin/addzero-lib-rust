// Generated from OpenAPI spec. Do not edit by hand.
//! `CreateChatCompletionRequest` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    ChatCompletionFunctions,
    ChatCompletionRequestMessage,
    ChatCompletionStreamOptions,
    ChatCompletionToolChoiceOption,
    CreateChatCompletionRequestAudio,
    CreateChatCompletionRequestFunctionCall,
    CreateChatCompletionRequestResponseFormat,
    CreateChatCompletionRequestTool,
    CreateChatCompletionRequestWebSearchOptions,
    Metadata,
    ModelIdsShared,
    ParallelToolCalls,
    PredictionContent,
    ReasoningEffort,
    ResponseModalities,
    ServiceTier,
    StopConfiguration,
    Verbosity,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateChatCompletionRequest {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub metadata: Option<Metadata>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub top_logprobs: Option<i32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub top_p: Option<f64>,
    /// This field is being replaced by `safety_identifier` and `prompt_cache_key`. Use `prompt_cache_key`
    /// instead to maintain caching optimizations. A stable identifier for your end-users. Used to boost
    /// cache hit rates by better bucketing similar requests and to help OpenAI detect and prevent abuse.
    /// [Learn more](/docs/guides/safety-best-practices#safety-identifiers).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub user: Option<String>,
    /// A stable identifier used to help detect users of your application that may be violating OpenAI's
    /// usage policies. The IDs should be a string that uniquely identifies each user, with a maximum length
    /// of 64 characters. We recommend hashing their username or email address, in order to avoid sending us
    /// any identifying information. [Learn more](/docs/guides/safety-best-practices#safety-identifiers).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub safety_identifier: Option<String>,
    /// Used by OpenAI to cache responses for similar requests to optimize your cache hit rates. Replaces
    /// the `user` field. [Learn more](/docs/guides/prompt-caching).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub prompt_cache_key: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub service_tier: Option<ServiceTier>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub prompt_cache_retention: Option<String>,
    /// A list of messages comprising the conversation so far. Depending on the [model](/docs/models) you
    /// use, different message types (modalities) are supported, like [text](/docs/guides/text-generation),
    /// [images](/docs/guides/vision), and [audio](/docs/guides/audio).
    pub messages: Vec<ChatCompletionRequestMessage>,
    /// Model ID used to generate the response, like `gpt-4o` or `o3`. OpenAI offers a wide range of models
    /// with different capabilities, performance characteristics, and price points. Refer to the [model
    /// guide](/docs/models) to browse and compare available models.
    pub model: ModelIdsShared,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub modalities: Option<ResponseModalities>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub verbosity: Option<Verbosity>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reasoning_effort: Option<ReasoningEffort>,
    /// An upper bound for the number of tokens that can be generated for a completion, including visible
    /// output tokens and [reasoning tokens](/docs/guides/reasoning).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_completion_tokens: Option<i32>,
    /// Number between -2.0 and 2.0. Positive values penalize new tokens based on their existing frequency
    /// in the text so far, decreasing the model's likelihood to repeat the same line verbatim.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub frequency_penalty: Option<f64>,
    /// Number between -2.0 and 2.0. Positive values penalize new tokens based on whether they appear in the
    /// text so far, increasing the model's likelihood to talk about new topics.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub presence_penalty: Option<f64>,
    /// This tool searches the web for relevant results to use in a response. Learn more about the [web
    /// search tool](/docs/guides/tools-web-search?api-mode=chat).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub web_search_options: Option<CreateChatCompletionRequestWebSearchOptions>,
    /// An object specifying the format that the model must output. Setting to `{ "type": "json_schema",
    /// "json_schema": {...} }` enables Structured Outputs which ensures the model will match your supplied
    /// JSON schema. Learn more in the [Structured Outputs guide](/docs/guides/structured-outputs). Setting
    /// to `{ "type": "json_object" }` enables the older JSON mode, which ensures the message the model
    /// generates is valid JSON. Using `json_schema` is preferred for models that support it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub response_format: Option<CreateChatCompletionRequestResponseFormat>,
    /// Parameters for audio output. Required when audio output is requested with `modalities: ["audio"]`.
    /// [Learn more](/docs/guides/audio).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub audio: Option<CreateChatCompletionRequestAudio>,
    /// Whether or not to store the output of this chat completion request for use in our [model
    /// distillation](/docs/guides/distillation) or [evals](/docs/guides/evals) products. Supports text and
    /// image inputs. Note: image inputs over 8MB will be dropped.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub store: Option<bool>,
    /// If set to true, the model response data will be streamed to the client as it is generated using
    /// [server-sent events](https://developer.mozilla.org/en-US/docs/Web/API/Server-
    /// sent_events/Using_server-sent_events#Event_stream_format). See the [Streaming section
    /// below](/docs/api-reference/chat/streaming) for more information, along with the [streaming
    /// responses](/docs/guides/streaming-responses) guide for more information on how to handle the
    /// streaming events.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stream: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stop: Option<StopConfiguration>,
    /// Modify the likelihood of specified tokens appearing in the completion. Accepts a JSON object that
    /// maps tokens (specified by their token ID in the tokenizer) to an associated bias value from -100 to
    /// 100. Mathematically, the bias is added to the logits generated by the model prior to sampling. The
    /// exact effect will vary per model, but values between -1 and 1 should decrease or increase likelihood
    /// of selection; values like -100 or 100 should result in a ban or exclusive selection of the relevant
    /// token.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub logit_bias: Option<std::collections::BTreeMap<String, i32>>,
    /// Whether to return log probabilities of the output tokens or not. If true, returns the log
    /// probabilities of each output token returned in the `content` of `message`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub logprobs: Option<bool>,
    /// The maximum number of [tokens](/tokenizer) that can be generated in the chat completion. This value
    /// can be used to control [costs](https://openai.com/api/pricing/) for text generated via API. This
    /// value is now deprecated in favor of `max_completion_tokens`, and is not compatible with [o-series
    /// models](/docs/guides/reasoning).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<i32>,
    /// How many chat completion choices to generate for each input message. Note that you will be charged
    /// based on the number of generated tokens across all of the choices. Keep `n` as `1` to minimize
    /// costs.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub n: Option<i32>,
    /// Configuration for a [Predicted Output](/docs/guides/predicted-outputs), which can greatly improve
    /// response times when large parts of the model response are known ahead of time. This is most common
    /// when you are regenerating a file with only minor changes to most of the content.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub prediction: Option<PredictionContent>,
    /// This feature is in Beta. If specified, our system will make a best effort to sample
    /// deterministically, such that repeated requests with the same `seed` and parameters should return the
    /// same result. Determinism is not guaranteed, and you should refer to the `system_fingerprint`
    /// response parameter to monitor changes in the backend.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub seed: Option<i32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stream_options: Option<ChatCompletionStreamOptions>,
    /// A list of tools the model may call. You can provide either [custom tools](/docs/guides/function-
    /// calling#custom-tools) or [function tools](/docs/guides/function-calling).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<CreateChatCompletionRequestTool>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tool_choice: Option<ChatCompletionToolChoiceOption>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parallel_tool_calls: Option<ParallelToolCalls>,
    /// Deprecated in favor of `tool_choice`. Controls which (if any) function is called by the model.
    /// `none` means the model will not call a function and instead generates a message. `auto` means the
    /// model can pick between generating a message or calling a function. Specifying a particular function
    /// via `{"name": "my_function"}` forces the model to call that function. `none` is the default when no
    /// functions are present. `auto` is the default if functions are present.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub function_call: Option<CreateChatCompletionRequestFunctionCall>,
    /// Deprecated in favor of `tools`. A list of functions the model may generate JSON inputs for.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub functions: Option<Vec<ChatCompletionFunctions>>,
}
