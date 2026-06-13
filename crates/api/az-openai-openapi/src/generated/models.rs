use super::bodies::*;
///Indicates that a thread is active.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct ActiveStatus {
    ///Status discriminator that is always `active`.
    #[serde(rename = "type")]
    pub type_value: String,
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct AddUploadPartRequest {
    ///The chunk of bytes for this Part.
    pub data: OpenAiBinaryBody,
}
///Represents an individual Admin API key in an org.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct AdminApiKey {
    ///The Unix timestamp (in seconds) of when the API key was created
    pub created_at: i64,
    ///The identifier, which can be referenced in API endpoints
    pub id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_used_at: ::std::option::Option<i64>,
    ///The name of the API key
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: ::std::option::Option<String>,
    ///The object type, which is always `organization.admin_api_key`
    pub object: String,
    pub owner: AdminApiKeyOwner,
    ///The redacted value of the API key
    pub redacted_value: String,
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct AdminApiKeyCreateResponse {
    ///The Unix timestamp (in seconds) of when the API key was created
    pub created_at: i64,
    ///The identifier, which can be referenced in API endpoints
    pub id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_used_at: ::std::option::Option<i64>,
    ///The name of the API key
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: ::std::option::Option<String>,
    ///The object type, which is always `organization.admin_api_key`
    pub object: String,
    pub owner: AdminApiKeyCreateResponseOwner,
    ///The redacted value of the API key
    pub redacted_value: String,
    ///The value of the API key. Only shown on create.
    pub value: String,
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct AdminApiKeyCreateResponseOwner {
    ///The Unix timestamp (in seconds) of when the user was created
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub created_at: ::std::option::Option<i64>,
    ///The identifier, which can be referenced in API endpoints
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: ::std::option::Option<String>,
    ///The name of the user
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: ::std::option::Option<String>,
    ///The object type, which is always organization.user
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub object: ::std::option::Option<String>,
    ///Always `owner`
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub role: ::std::option::Option<String>,
    ///Always `user`
    #[serde(rename = "type", default, skip_serializing_if = "Option::is_none")]
    pub type_value: ::std::option::Option<String>,
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct AdminApiKeyOwner {
    ///The Unix timestamp (in seconds) of when the user was created
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub created_at: ::std::option::Option<i64>,
    ///The identifier, which can be referenced in API endpoints
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: ::std::option::Option<String>,
    ///The name of the user
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: ::std::option::Option<String>,
    ///The object type, which is always organization.user
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub object: ::std::option::Option<String>,
    ///Always `owner`
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub role: ::std::option::Option<String>,
    ///Always `user`
    #[serde(rename = "type", default, skip_serializing_if = "Option::is_none")]
    pub type_value: ::std::option::Option<String>,
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct AdminApiKeysCreateRequest {
    pub name: String,
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct AdminApiKeysDeleteResponse {
    pub deleted: bool,
    pub id: String,
    pub object: String,
}
///An annotation that applies to a span of output text.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
#[serde(untagged)]
pub enum Annotation {
    FileCitationBody(FileCitationBody),
    UrlCitationBody(UrlCitationBody),
    ContainerFileCitationBody(ContainerFileCitationBody),
    FilePath(FilePath),
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct ApiKeyList {
    pub data: Vec<AdminApiKey>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub first_id: ::std::option::Option<String>,
    pub has_more: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_id: ::std::option::Option<String>,
    pub object: String,
}
pub type ApplyPatchCallOutputStatus = String;
///Outcome values reported for apply_patch tool call outputs.
pub type ApplyPatchCallOutputStatusParam = String;
pub type ApplyPatchCallStatus = String;
///Status values reported for apply_patch tool calls.
pub type ApplyPatchCallStatusParam = String;
///Instruction describing how to create a file via the apply_patch tool.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct ApplyPatchCreateFileOperation {
    ///Diff to apply.
    pub diff: String,
    ///Path of the file to create.
    pub path: String,
    ///Create a new file with the provided diff.
    #[serde(rename = "type")]
    pub type_value: String,
}
///Instruction for creating a new file via the apply_patch tool.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct ApplyPatchCreateFileOperationParam {
    ///Unified diff content to apply when creating the file.
    pub diff: String,
    ///Path of the file to create relative to the workspace root.
    pub path: String,
    ///The operation type. Always `create_file`.
    #[serde(rename = "type")]
    pub type_value: String,
}
///Instruction describing how to delete a file via the apply_patch tool.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct ApplyPatchDeleteFileOperation {
    ///Path of the file to delete.
    pub path: String,
    ///Delete the specified file.
    #[serde(rename = "type")]
    pub type_value: String,
}
///Instruction for deleting an existing file via the apply_patch tool.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct ApplyPatchDeleteFileOperationParam {
    ///Path of the file to delete relative to the workspace root.
    pub path: String,
    ///The operation type. Always `delete_file`.
    #[serde(rename = "type")]
    pub type_value: String,
}
///One of the create_file, delete_file, or update_file operations supplied to the apply_patch tool.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
#[serde(untagged)]
pub enum ApplyPatchOperationParam {
    ApplyPatchCreateFileOperationParam(ApplyPatchCreateFileOperationParam),
    ApplyPatchDeleteFileOperationParam(ApplyPatchDeleteFileOperationParam),
    ApplyPatchUpdateFileOperationParam(ApplyPatchUpdateFileOperationParam),
}
///A tool call that applies file diffs by creating, deleting, or updating files.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct ApplyPatchToolCall {
    ///The unique ID of the apply patch tool call generated by the model.
    pub call_id: String,
    ///The ID of the entity that created this tool call.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub created_by: ::std::option::Option<String>,
    ///The unique ID of the apply patch tool call. Populated when this item is returned via API.
    pub id: String,
    ///One of the create_file, delete_file, or update_file operations applied via apply_patch.
    pub operation: ApplyPatchToolCallOperation,
    ///The status of the apply patch tool call. One of `in_progress` or `completed`.
    pub status: ApplyPatchCallStatus,
    ///The type of the item. Always `apply_patch_call`.
    #[serde(rename = "type")]
    pub type_value: String,
}
///A tool call representing a request to create, delete, or update files using diff patches.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct ApplyPatchToolCallItemParam {
    ///The unique ID of the apply patch tool call generated by the model.
    pub call_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: ::std::option::Option<String>,
    ///The specific create, delete, or update instruction for the apply_patch tool call.
    pub operation: ApplyPatchOperationParam,
    ///The status of the apply patch tool call. One of `in_progress` or `completed`.
    pub status: ApplyPatchCallStatusParam,
    ///The type of the item. Always `apply_patch_call`.
    #[serde(rename = "type")]
    pub type_value: String,
}
///One of the create_file, delete_file, or update_file operations applied via apply_patch.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
#[serde(untagged)]
pub enum ApplyPatchToolCallOperation {
    ApplyPatchCreateFileOperation(ApplyPatchCreateFileOperation),
    ApplyPatchDeleteFileOperation(ApplyPatchDeleteFileOperation),
    ApplyPatchUpdateFileOperation(ApplyPatchUpdateFileOperation),
}
///The output emitted by an apply patch tool call.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct ApplyPatchToolCallOutput {
    ///The unique ID of the apply patch tool call generated by the model.
    pub call_id: String,
    ///The ID of the entity that created this tool call output.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub created_by: ::std::option::Option<String>,
    ///The unique ID of the apply patch tool call output. Populated when this item is returned via API.
    pub id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub output: ::std::option::Option<String>,
    ///The status of the apply patch tool call output. One of `completed` or `failed`.
    pub status: ApplyPatchCallOutputStatus,
    ///The type of the item. Always `apply_patch_call_output`.
    #[serde(rename = "type")]
    pub type_value: String,
}
///The streamed output emitted by an apply patch tool call.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct ApplyPatchToolCallOutputItemParam {
    ///The unique ID of the apply patch tool call generated by the model.
    pub call_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: ::std::option::Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub output: ::std::option::Option<String>,
    ///The status of the apply patch tool call output. One of `completed` or `failed`.
    pub status: ApplyPatchCallOutputStatusParam,
    ///The type of the item. Always `apply_patch_call_output`.
    #[serde(rename = "type")]
    pub type_value: String,
}
///Allows the assistant to create, delete, or update files using unified diffs.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct ApplyPatchToolParam {
    ///The type of the tool. Always `apply_patch`.
    #[serde(rename = "type")]
    pub type_value: String,
}
///Instruction describing how to update a file via the apply_patch tool.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct ApplyPatchUpdateFileOperation {
    ///Diff to apply.
    pub diff: String,
    ///Path of the file to update.
    pub path: String,
    ///Update an existing file with the provided diff.
    #[serde(rename = "type")]
    pub type_value: String,
}
///Instruction for updating an existing file via the apply_patch tool.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct ApplyPatchUpdateFileOperationParam {
    ///Unified diff content to apply to the existing file.
    pub diff: String,
    ///Path of the file to update relative to the workspace root.
    pub path: String,
    ///The operation type. Always `update_file`.
    #[serde(rename = "type")]
    pub type_value: String,
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct ApproximateLocation {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub city: ::std::option::Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub country: ::std::option::Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub region: ::std::option::Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub timezone: ::std::option::Option<String>,
    ///The type of location approximation. Always `approximate`.
    #[serde(rename = "type")]
    pub type_value: String,
}
///Detailed information about a role assignment entry returned when listing assignments.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct AssignedRoleDetails {
    ///When the role was created.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub created_at: ::std::option::Option<i64>,
    ///Identifier of the actor who created the role.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub created_by: ::std::option::Option<String>,
    ///User details for the actor that created the role, when available.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub created_by_user_obj: ::std::option::Option<OpenAiJsonValue>,
    ///Description of the role.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: ::std::option::Option<String>,
    ///Identifier for the role.
    pub id: String,
    ///Arbitrary metadata stored on the role.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub metadata: ::std::option::Option<OpenAiJsonValue>,
    ///Name of the role.
    pub name: String,
    ///Permissions associated with the role.
    pub permissions: Vec<String>,
    ///Whether the role is predefined by OpenAI.
    pub predefined_role: bool,
    ///Resource type the role applies to.
    pub resource_type: String,
    ///When the role was last updated.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub updated_at: ::std::option::Option<i64>,
}
///Assistant-authored message within a thread.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct AssistantMessageItem {
    ///Ordered assistant response segments.
    pub content: Vec<ResponseOutputText>,
    ///Unix timestamp (in seconds) for when the item was created.
    pub created_at: i64,
    ///Identifier of the thread item.
    pub id: String,
    ///Type discriminator that is always `chatkit.thread_item`.
    pub object: String,
    ///Identifier of the parent thread.
    pub thread_id: String,
    ///Type discriminator that is always `chatkit.assistant_message`.
    #[serde(rename = "type")]
    pub type_value: String,
}
///Represents an `assistant` that can call the model and use tools.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct AssistantObject {
    ///The Unix timestamp (in seconds) for when the assistant was created.
    pub created_at: i64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: ::std::option::Option<String>,
    ///The identifier, which can be referenced in API endpoints.
    pub id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub instructions: ::std::option::Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub metadata: ::std::option::Option<Metadata>,
    ///ID of the model to use. You can use the [List models](/docs/api-reference/models/list) API to see all of your available models, or see our [Model overview](/docs/models) for descriptions of them.
    pub model: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: ::std::option::Option<String>,
    ///The object type, which is always `assistant`.
    pub object: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub response_format: ::std::option::Option<AssistantsApiResponseFormatOption>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub temperature: ::std::option::Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tool_resources: ::std::option::Option<AssistantObjectToolResources>,
    ///A list of tool enabled on the assistant. There can be a maximum of 128 tools per assistant. Tools can be of types `code_interpreter`, `file_search`, or `function`.
    pub tools: Vec<AssistantObjectTool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub top_p: ::std::option::Option<f64>,
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
#[serde(untagged)]
pub enum AssistantObjectTool {
    AssistantToolsCode(AssistantToolsCode),
    AssistantToolsFileSearch(AssistantToolsFileSearch),
    AssistantToolsFunction(AssistantToolsFunction),
}
///A set of resources that are used by the assistant's tools. The resources are specific to the type of tool. For example, the `code_interpreter` tool requires a list of file IDs, while the `file_search` tool requires a list of vector store IDs.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct AssistantObjectToolResources {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub code_interpreter: ::std::option::Option<
        AssistantObjectToolResourcesCodeInterpreter,
    >,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub file_search: ::std::option::Option<AssistantObjectToolResourcesFileSearch>,
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct AssistantObjectToolResourcesCodeInterpreter {
    ///A list of [file](/docs/api-reference/files) IDs made available to the `code_interpreter`` tool. There can be a maximum of 20 files associated with the tool.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub file_ids: ::std::option::Option<Vec<String>>,
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct AssistantObjectToolResourcesFileSearch {
    ///The ID of the [vector store](/docs/api-reference/vector-stores/object) attached to this assistant. There can be a maximum of 1 vector store attached to the assistant.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub vector_store_ids: ::std::option::Option<Vec<String>>,
}
///Represents an event emitted when streaming a Run. Each event in a server-sent events stream has an `event` and `data` property: ``` event: thread.created data: {"id": "thread_123", "object": "thread", ...} ``` We emit events whenever a new object is created, transitions to a new state, or is being streamed in parts (deltas). For example, we emit `thread.run.created` when a new run is created, `thread.run.completed` when a run completes, and so on. When an Assistant chooses to create a message during a run, we emit a `thread.message.created event`, a `thread.message.in_progress` event, many `thread.message.delta` events, and finally a `thread.message.completed` event. We may add additional events over time, so we recommend handling unknown events gracefully in your code. See the [Assistants API quickstart](/docs/assistants/overview) to learn how to integrate the Assistants API with streaming.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
#[serde(untagged)]
pub enum AssistantStreamEvent {
    ThreadStreamEvent(ThreadStreamEvent),
    RunStreamEvent(RunStreamEvent),
    RunStepStreamEvent(RunStepStreamEvent),
    MessageStreamEvent(MessageStreamEvent),
    ErrorEvent(ErrorEvent),
    DoneEvent(DoneEvent),
}
pub type AssistantSupportedModels = String;
///Code interpreter tool
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct AssistantToolsCode {
    ///The type of tool being defined: `code_interpreter`
    #[serde(rename = "type")]
    pub type_value: String,
}
///FileSearch tool
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct AssistantToolsFileSearch {
    ///Overrides for the file search tool.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub file_search: ::std::option::Option<AssistantToolsFileSearchFileSearch>,
    ///The type of tool being defined: `file_search`
    #[serde(rename = "type")]
    pub type_value: String,
}
///Overrides for the file search tool.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct AssistantToolsFileSearchFileSearch {
    ///The maximum number of results the file search tool should output. The default is 20 for `gpt-4*` models and 5 for `gpt-3.5-turbo`. This number should be between 1 and 50 inclusive. Note that the file search tool may output fewer than `max_num_results` results. See the [file search tool documentation](/docs/assistants/tools/file-search#customizing-file-search-settings) for more information.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_num_results: ::std::option::Option<i32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ranking_options: ::std::option::Option<FileSearchRankingOptions>,
}
///FileSearch tool
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct AssistantToolsFileSearchTypeOnly {
    ///The type of tool being defined: `file_search`
    #[serde(rename = "type")]
    pub type_value: String,
}
///Function tool
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct AssistantToolsFunction {
    pub function: FunctionObject,
    ///The type of tool being defined: `function`
    #[serde(rename = "type")]
    pub type_value: String,
}
///Specifies the format that the model must output. Compatible with [GPT-4o](/docs/models#gpt-4o), [GPT-4 Turbo](/docs/models#gpt-4-turbo-and-gpt-4), and all GPT-3.5 Turbo models since `gpt-3.5-turbo-1106`. Setting to `{ "type": "json_schema", "json_schema": {...} }` enables Structured Outputs which ensures the model will match your supplied JSON schema. Learn more in the [Structured Outputs guide](/docs/guides/structured-outputs). Setting to `{ "type": "json_object" }` enables JSON mode, which ensures the message the model generates is valid JSON. **Important:** when using JSON mode, you **must** also instruct the model to produce JSON yourself via a system or user message. Without this, the model may generate an unending stream of whitespace until the generation reaches the token limit, resulting in a long-running and seemingly "stuck" request. Also note that the message content may be partially cut off if `finish_reason="length"`, which indicates the generation exceeded `max_tokens` or the conversation exceeded the max context length.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
#[serde(untagged)]
pub enum AssistantsApiResponseFormatOption {
    Auto(String),
    ResponseFormatText(ResponseFormatText),
    ResponseFormatJsonObject(ResponseFormatJsonObject),
    ResponseFormatJsonSchema(ResponseFormatJsonSchema),
}
///Controls which (if any) tool is called by the model. `none` means the model will not call any tools and instead generates a message. `auto` is the default value and means the model can pick between generating a message or calling one or more tools. `required` means the model must call one or more tools before responding to the user. Specifying a particular tool like `{"type": "file_search"}` or `{"type": "function", "function": {"name": "my_function"}}` forces the model to call that tool.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
#[serde(untagged)]
pub enum AssistantsApiToolChoiceOption {
    String(String),
    AssistantsNamedToolChoice(AssistantsNamedToolChoice),
}
///Specifies a tool the model should use. Use to force the model to call a specific tool.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct AssistantsNamedToolChoice {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub function: ::std::option::Option<AssistantsNamedToolChoiceFunction>,
    ///The type of the tool. If type is `function`, the function name must be set
    #[serde(rename = "type")]
    pub type_value: String,
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct AssistantsNamedToolChoiceFunction {
    ///The name of the function to call.
    pub name: String,
}
///Attachment metadata included on thread items.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct Attachment {
    ///Identifier for the attachment.
    pub id: String,
    ///MIME type of the attachment.
    pub mime_type: String,
    ///Original display name for the attachment.
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub preview_url: ::std::option::Option<String>,
    ///Attachment discriminator.
    #[serde(rename = "type")]
    pub type_value: AttachmentType,
}
pub type AttachmentType = String;
///The format of the output, in one of these options: `json`, `text`, `srt`, `verbose_json`, `vtt`, or `diarized_json`. For `gpt-4o-transcribe` and `gpt-4o-mini-transcribe`, the only supported format is `json`. For `gpt-4o-transcribe-diarize`, the supported formats are `json`, `text`, and `diarized_json`, with `diarized_json` required to receive speaker annotations.
pub type AudioResponseFormat = String;
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct AudioTranscription {
    ///Controls how long the model waits before emitting transcription text. Higher values can improve transcription accuracy at the cost of latency. Only supported with `gpt-realtime-whisper` in GA Realtime sessions.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub delay: ::std::option::Option<String>,
    ///The language of the input audio. Supplying the input language in [ISO-639-1](https://en.wikipedia.org/wiki/List_of_ISO_639-1_codes) (e.g. `en`) format will improve accuracy and latency.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub language: ::std::option::Option<String>,
    ///The model to use for transcription. Current options are `whisper-1`, `gpt-4o-mini-transcribe`, `gpt-4o-mini-transcribe-2025-12-15`, `gpt-4o-transcribe`, `gpt-4o-transcribe-diarize`, and `gpt-realtime-whisper`. Use `gpt-4o-transcribe-diarize` when you need diarization with speaker labels.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model: ::std::option::Option<String>,
    ///An optional text to guide the model's style or continue a previous audio segment. For `whisper-1`, the [prompt is a list of keywords](/docs/guides/speech-to-text#prompting). For `gpt-4o-transcribe` models (excluding `gpt-4o-transcribe-diarize`), the prompt is a free text string, for example "expect words related to technology". Prompt is not supported with `gpt-realtime-whisper` in GA Realtime sessions.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub prompt: ::std::option::Option<String>,
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct AudioTranscriptionResponse {
    ///The language of the input audio.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub language: ::std::option::Option<String>,
    ///The model used for transcription. Current options are `whisper-1`, `gpt-4o-mini-transcribe`, `gpt-4o-mini-transcribe-2025-12-15`, `gpt-4o-transcribe`, `gpt-4o-transcribe-diarize`, and `gpt-realtime-whisper`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model: ::std::option::Option<String>,
    ///The prompt configured for input audio transcription, when present.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub prompt: ::std::option::Option<String>,
}
///A log of a user action or configuration change within this organization.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct AuditLog {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub actor: ::std::option::Option<AuditLogActor>,
    ///The details for events with this `type`.
    #[serde(
        rename = "api_key.created",
        default,
        skip_serializing_if = "Option::is_none",
    )]
    pub api_key_created: ::std::option::Option<AuditLogApiKeyCreated>,
    ///The details for events with this `type`.
    #[serde(
        rename = "api_key.deleted",
        default,
        skip_serializing_if = "Option::is_none",
    )]
    pub api_key_deleted: ::std::option::Option<AuditLogApiKeyDeleted>,
    ///The details for events with this `type`.
    #[serde(
        rename = "api_key.updated",
        default,
        skip_serializing_if = "Option::is_none",
    )]
    pub api_key_updated: ::std::option::Option<AuditLogApiKeyUpdated>,
    ///The details for events with this `type`.
    #[serde(
        rename = "certificate.created",
        default,
        skip_serializing_if = "Option::is_none",
    )]
    pub certificate_created: ::std::option::Option<AuditLogCertificateCreated>,
    ///The details for events with this `type`.
    #[serde(
        rename = "certificate.deleted",
        default,
        skip_serializing_if = "Option::is_none",
    )]
    pub certificate_deleted: ::std::option::Option<AuditLogCertificateDeleted>,
    ///The details for events with this `type`.
    #[serde(
        rename = "certificate.updated",
        default,
        skip_serializing_if = "Option::is_none",
    )]
    pub certificate_updated: ::std::option::Option<AuditLogCertificateUpdated>,
    ///The details for events with this `type`.
    #[serde(
        rename = "certificates.activated",
        default,
        skip_serializing_if = "Option::is_none",
    )]
    pub certificates_activated: ::std::option::Option<AuditLogCertificatesActivated>,
    ///The details for events with this `type`.
    #[serde(
        rename = "certificates.deactivated",
        default,
        skip_serializing_if = "Option::is_none",
    )]
    pub certificates_deactivated: ::std::option::Option<AuditLogCertificatesDeactivated>,
    ///The project and fine-tuned model checkpoint that the checkpoint permission was created for.
    #[serde(
        rename = "checkpoint.permission.created",
        default,
        skip_serializing_if = "Option::is_none",
    )]
    pub checkpoint_permission_created: ::std::option::Option<
        AuditLogCheckpointPermissionCreated,
    >,
    ///The details for events with this `type`.
    #[serde(
        rename = "checkpoint.permission.deleted",
        default,
        skip_serializing_if = "Option::is_none",
    )]
    pub checkpoint_permission_deleted: ::std::option::Option<
        AuditLogCheckpointPermissionDeleted,
    >,
    ///The Unix timestamp (in seconds) of the event.
    pub effective_at: i64,
    ///The details for events with this `type`.
    #[serde(
        rename = "external_key.registered",
        default,
        skip_serializing_if = "Option::is_none",
    )]
    pub external_key_registered: ::std::option::Option<AuditLogExternalKeyRegistered>,
    ///The details for events with this `type`.
    #[serde(
        rename = "external_key.removed",
        default,
        skip_serializing_if = "Option::is_none",
    )]
    pub external_key_removed: ::std::option::Option<AuditLogExternalKeyRemoved>,
    ///The details for events with this `type`.
    #[serde(rename = "group.created", default, skip_serializing_if = "Option::is_none")]
    pub group_created: ::std::option::Option<AuditLogGroupCreated>,
    ///The details for events with this `type`.
    #[serde(rename = "group.deleted", default, skip_serializing_if = "Option::is_none")]
    pub group_deleted: ::std::option::Option<AuditLogGroupDeleted>,
    ///The details for events with this `type`.
    #[serde(rename = "group.updated", default, skip_serializing_if = "Option::is_none")]
    pub group_updated: ::std::option::Option<AuditLogGroupUpdated>,
    ///The ID of this log.
    pub id: String,
    ///The details for events with this `type`.
    #[serde(
        rename = "invite.accepted",
        default,
        skip_serializing_if = "Option::is_none",
    )]
    pub invite_accepted: ::std::option::Option<AuditLogInviteAccepted>,
    ///The details for events with this `type`.
    #[serde(rename = "invite.deleted", default, skip_serializing_if = "Option::is_none")]
    pub invite_deleted: ::std::option::Option<AuditLogInviteDeleted>,
    ///The details for events with this `type`.
    #[serde(rename = "invite.sent", default, skip_serializing_if = "Option::is_none")]
    pub invite_sent: ::std::option::Option<AuditLogInviteSent>,
    ///The details for events with this `type`.
    #[serde(
        rename = "ip_allowlist.config.activated",
        default,
        skip_serializing_if = "Option::is_none",
    )]
    pub ip_allowlist_config_activated: ::std::option::Option<
        AuditLogIpAllowlistConfigActivated,
    >,
    ///The details for events with this `type`.
    #[serde(
        rename = "ip_allowlist.config.deactivated",
        default,
        skip_serializing_if = "Option::is_none",
    )]
    pub ip_allowlist_config_deactivated: ::std::option::Option<
        AuditLogIpAllowlistConfigDeactivated,
    >,
    ///The details for events with this `type`.
    #[serde(
        rename = "ip_allowlist.created",
        default,
        skip_serializing_if = "Option::is_none",
    )]
    pub ip_allowlist_created: ::std::option::Option<AuditLogIpAllowlistCreated>,
    ///The details for events with this `type`.
    #[serde(
        rename = "ip_allowlist.deleted",
        default,
        skip_serializing_if = "Option::is_none",
    )]
    pub ip_allowlist_deleted: ::std::option::Option<AuditLogIpAllowlistDeleted>,
    ///The details for events with this `type`.
    #[serde(
        rename = "ip_allowlist.updated",
        default,
        skip_serializing_if = "Option::is_none",
    )]
    pub ip_allowlist_updated: ::std::option::Option<AuditLogIpAllowlistUpdated>,
    ///The details for events with this `type`.
    #[serde(rename = "login.failed", default, skip_serializing_if = "Option::is_none")]
    pub login_failed: ::std::option::Option<AuditLogLoginFailed>,
    ///This event has no additional fields beyond the standard audit log attributes.
    #[serde(
        rename = "login.succeeded",
        default,
        skip_serializing_if = "Option::is_none",
    )]
    pub login_succeeded: ::std::option::Option<OpenAiJsonValue>,
    ///The details for events with this `type`.
    #[serde(rename = "logout.failed", default, skip_serializing_if = "Option::is_none")]
    pub logout_failed: ::std::option::Option<AuditLogLogoutFailed>,
    ///This event has no additional fields beyond the standard audit log attributes.
    #[serde(
        rename = "logout.succeeded",
        default,
        skip_serializing_if = "Option::is_none",
    )]
    pub logout_succeeded: ::std::option::Option<OpenAiJsonValue>,
    ///The details for events with this `type`.
    #[serde(
        rename = "organization.updated",
        default,
        skip_serializing_if = "Option::is_none",
    )]
    pub organization_updated: ::std::option::Option<AuditLogOrganizationUpdated>,
    ///The project that the action was scoped to. Absent for actions not scoped to projects. Note that any admin actions taken via Admin API keys are associated with the default project.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub project: ::std::option::Option<AuditLogProject>,
    ///The details for events with this `type`.
    #[serde(
        rename = "project.archived",
        default,
        skip_serializing_if = "Option::is_none",
    )]
    pub project_archived: ::std::option::Option<AuditLogProjectArchived>,
    ///The details for events with this `type`.
    #[serde(
        rename = "project.created",
        default,
        skip_serializing_if = "Option::is_none",
    )]
    pub project_created: ::std::option::Option<AuditLogProjectCreated>,
    ///The details for events with this `type`.
    #[serde(
        rename = "project.deleted",
        default,
        skip_serializing_if = "Option::is_none",
    )]
    pub project_deleted: ::std::option::Option<AuditLogProjectDeleted>,
    ///The details for events with this `type`.
    #[serde(
        rename = "project.updated",
        default,
        skip_serializing_if = "Option::is_none",
    )]
    pub project_updated: ::std::option::Option<AuditLogProjectUpdated>,
    ///The details for events with this `type`.
    #[serde(
        rename = "rate_limit.deleted",
        default,
        skip_serializing_if = "Option::is_none",
    )]
    pub rate_limit_deleted: ::std::option::Option<AuditLogRateLimitDeleted>,
    ///The details for events with this `type`.
    #[serde(
        rename = "rate_limit.updated",
        default,
        skip_serializing_if = "Option::is_none",
    )]
    pub rate_limit_updated: ::std::option::Option<AuditLogRateLimitUpdated>,
    ///The details for events with this `type`.
    #[serde(
        rename = "role.assignment.created",
        default,
        skip_serializing_if = "Option::is_none",
    )]
    pub role_assignment_created: ::std::option::Option<AuditLogRoleAssignmentCreated>,
    ///The details for events with this `type`.
    #[serde(
        rename = "role.assignment.deleted",
        default,
        skip_serializing_if = "Option::is_none",
    )]
    pub role_assignment_deleted: ::std::option::Option<AuditLogRoleAssignmentDeleted>,
    ///The details for events with this `type`.
    #[serde(rename = "role.created", default, skip_serializing_if = "Option::is_none")]
    pub role_created: ::std::option::Option<AuditLogRoleCreated>,
    ///The details for events with this `type`.
    #[serde(rename = "role.deleted", default, skip_serializing_if = "Option::is_none")]
    pub role_deleted: ::std::option::Option<AuditLogRoleDeleted>,
    ///The details for events with this `type`.
    #[serde(rename = "role.updated", default, skip_serializing_if = "Option::is_none")]
    pub role_updated: ::std::option::Option<AuditLogRoleUpdated>,
    ///The details for events with this `type`.
    #[serde(rename = "scim.disabled", default, skip_serializing_if = "Option::is_none")]
    pub scim_disabled: ::std::option::Option<AuditLogScimDisabled>,
    ///The details for events with this `type`.
    #[serde(rename = "scim.enabled", default, skip_serializing_if = "Option::is_none")]
    pub scim_enabled: ::std::option::Option<AuditLogScimEnabled>,
    ///The details for events with this `type`.
    #[serde(
        rename = "service_account.created",
        default,
        skip_serializing_if = "Option::is_none",
    )]
    pub service_account_created: ::std::option::Option<AuditLogServiceAccountCreated>,
    ///The details for events with this `type`.
    #[serde(
        rename = "service_account.deleted",
        default,
        skip_serializing_if = "Option::is_none",
    )]
    pub service_account_deleted: ::std::option::Option<AuditLogServiceAccountDeleted>,
    ///The details for events with this `type`.
    #[serde(
        rename = "service_account.updated",
        default,
        skip_serializing_if = "Option::is_none",
    )]
    pub service_account_updated: ::std::option::Option<AuditLogServiceAccountUpdated>,
    #[serde(rename = "type")]
    pub type_value: AuditLogEventType,
    ///The details for events with this `type`.
    #[serde(rename = "user.added", default, skip_serializing_if = "Option::is_none")]
    pub user_added: ::std::option::Option<AuditLogUserAdded>,
    ///The details for events with this `type`.
    #[serde(rename = "user.deleted", default, skip_serializing_if = "Option::is_none")]
    pub user_deleted: ::std::option::Option<AuditLogUserDeleted>,
    ///The details for events with this `type`.
    #[serde(rename = "user.updated", default, skip_serializing_if = "Option::is_none")]
    pub user_updated: ::std::option::Option<AuditLogUserUpdated>,
}
///The actor who performed the audit logged action.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct AuditLogActor {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub api_key: ::std::option::Option<AuditLogActorApiKey>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub session: ::std::option::Option<AuditLogActorSession>,
    ///The type of actor. Is either `session` or `api_key`.
    #[serde(rename = "type", default, skip_serializing_if = "Option::is_none")]
    pub type_value: ::std::option::Option<String>,
}
///The API Key used to perform the audit logged action.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct AuditLogActorApiKey {
    ///The tracking id of the API key.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: ::std::option::Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub service_account: ::std::option::Option<AuditLogActorServiceAccount>,
    ///The type of API key. Can be either `user` or `service_account`.
    #[serde(rename = "type", default, skip_serializing_if = "Option::is_none")]
    pub type_value: ::std::option::Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub user: ::std::option::Option<AuditLogActorUser>,
}
///The service account that performed the audit logged action.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct AuditLogActorServiceAccount {
    ///The service account id.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: ::std::option::Option<String>,
}
///The session in which the audit logged action was performed.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct AuditLogActorSession {
    ///The IP address from which the action was performed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ip_address: ::std::option::Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub user: ::std::option::Option<AuditLogActorUser>,
}
///The user who performed the audit logged action.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct AuditLogActorUser {
    ///The user email.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub email: ::std::option::Option<String>,
    ///The user id.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: ::std::option::Option<String>,
}
///The details for events with this `type`.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct AuditLogApiKeyCreated {
    ///The payload used to create the API key.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub data: ::std::option::Option<AuditLogApiKeyCreatedData>,
    ///The tracking ID of the API key.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: ::std::option::Option<String>,
}
///The payload used to create the API key.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct AuditLogApiKeyCreatedData {
    ///A list of scopes allowed for the API key, e.g. `["api.model.request"]`
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scopes: ::std::option::Option<Vec<String>>,
}
///The details for events with this `type`.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct AuditLogApiKeyDeleted {
    ///The tracking ID of the API key.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: ::std::option::Option<String>,
}
///The details for events with this `type`.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct AuditLogApiKeyUpdated {
    ///The payload used to update the API key.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub changes_requested: ::std::option::Option<AuditLogApiKeyUpdatedChangesRequested>,
    ///The tracking ID of the API key.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: ::std::option::Option<String>,
}
///The payload used to update the API key.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct AuditLogApiKeyUpdatedChangesRequested {
    ///A list of scopes allowed for the API key, e.g. `["api.model.request"]`
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scopes: ::std::option::Option<Vec<String>>,
}
///The details for events with this `type`.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct AuditLogCertificateCreated {
    ///The certificate ID.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: ::std::option::Option<String>,
    ///The name of the certificate.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: ::std::option::Option<String>,
}
///The details for events with this `type`.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct AuditLogCertificateDeleted {
    ///The certificate content in PEM format.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub certificate: ::std::option::Option<String>,
    ///The certificate ID.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: ::std::option::Option<String>,
    ///The name of the certificate.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: ::std::option::Option<String>,
}
///The details for events with this `type`.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct AuditLogCertificateUpdated {
    ///The certificate ID.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: ::std::option::Option<String>,
    ///The name of the certificate.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: ::std::option::Option<String>,
}
///The details for events with this `type`.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct AuditLogCertificatesActivated {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub certificates: ::std::option::Option<
        Vec<AuditLogCertificatesActivatedCertificate>,
    >,
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct AuditLogCertificatesActivatedCertificate {
    ///The certificate ID.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: ::std::option::Option<String>,
    ///The name of the certificate.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: ::std::option::Option<String>,
}
///The details for events with this `type`.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct AuditLogCertificatesDeactivated {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub certificates: ::std::option::Option<
        Vec<AuditLogCertificatesDeactivatedCertificate>,
    >,
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct AuditLogCertificatesDeactivatedCertificate {
    ///The certificate ID.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: ::std::option::Option<String>,
    ///The name of the certificate.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: ::std::option::Option<String>,
}
///The project and fine-tuned model checkpoint that the checkpoint permission was created for.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct AuditLogCheckpointPermissionCreated {
    ///The payload used to create the checkpoint permission.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub data: ::std::option::Option<AuditLogCheckpointPermissionCreatedData>,
    ///The ID of the checkpoint permission.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: ::std::option::Option<String>,
}
///The payload used to create the checkpoint permission.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct AuditLogCheckpointPermissionCreatedData {
    ///The ID of the fine-tuned model checkpoint.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fine_tuned_model_checkpoint: ::std::option::Option<String>,
    ///The ID of the project that the checkpoint permission was created for.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub project_id: ::std::option::Option<String>,
}
///The details for events with this `type`.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct AuditLogCheckpointPermissionDeleted {
    ///The ID of the checkpoint permission.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: ::std::option::Option<String>,
}
///The event type.
pub type AuditLogEventType = String;
///The details for events with this `type`.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct AuditLogExternalKeyRegistered {
    ///The configuration for the external key.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub data: ::std::option::Option<OpenAiJsonValue>,
    ///The ID of the external key configuration.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: ::std::option::Option<String>,
}
///The details for events with this `type`.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct AuditLogExternalKeyRemoved {
    ///The ID of the external key configuration.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: ::std::option::Option<String>,
}
///The details for events with this `type`.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct AuditLogGroupCreated {
    ///Information about the created group.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub data: ::std::option::Option<AuditLogGroupCreatedData>,
    ///The ID of the group.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: ::std::option::Option<String>,
}
///Information about the created group.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct AuditLogGroupCreatedData {
    ///The group name.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub group_name: ::std::option::Option<String>,
}
///The details for events with this `type`.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct AuditLogGroupDeleted {
    ///The ID of the group.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: ::std::option::Option<String>,
}
///The details for events with this `type`.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct AuditLogGroupUpdated {
    ///The payload used to update the group.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub changes_requested: ::std::option::Option<AuditLogGroupUpdatedChangesRequested>,
    ///The ID of the group.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: ::std::option::Option<String>,
}
///The payload used to update the group.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct AuditLogGroupUpdatedChangesRequested {
    ///The updated group name.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub group_name: ::std::option::Option<String>,
}
///The details for events with this `type`.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct AuditLogInviteAccepted {
    ///The ID of the invite.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: ::std::option::Option<String>,
}
///The details for events with this `type`.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct AuditLogInviteDeleted {
    ///The ID of the invite.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: ::std::option::Option<String>,
}
///The details for events with this `type`.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct AuditLogInviteSent {
    ///The payload used to create the invite.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub data: ::std::option::Option<AuditLogInviteSentData>,
    ///The ID of the invite.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: ::std::option::Option<String>,
}
///The payload used to create the invite.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct AuditLogInviteSentData {
    ///The email invited to the organization.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub email: ::std::option::Option<String>,
    ///The role the email was invited to be. Is either `owner` or `member`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub role: ::std::option::Option<String>,
}
///The details for events with this `type`.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct AuditLogIpAllowlistConfigActivated {
    ///The configurations that were activated.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub configs: ::std::option::Option<Vec<AuditLogIpAllowlistConfigActivatedConfig>>,
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct AuditLogIpAllowlistConfigActivatedConfig {
    ///The ID of the IP allowlist configuration.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: ::std::option::Option<String>,
    ///The name of the IP allowlist configuration.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: ::std::option::Option<String>,
}
///The details for events with this `type`.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct AuditLogIpAllowlistConfigDeactivated {
    ///The configurations that were deactivated.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub configs: ::std::option::Option<Vec<AuditLogIpAllowlistConfigDeactivatedConfig>>,
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct AuditLogIpAllowlistConfigDeactivatedConfig {
    ///The ID of the IP allowlist configuration.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: ::std::option::Option<String>,
    ///The name of the IP allowlist configuration.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: ::std::option::Option<String>,
}
///The details for events with this `type`.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct AuditLogIpAllowlistCreated {
    ///The IP addresses or CIDR ranges included in the configuration.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub allowed_ips: ::std::option::Option<Vec<String>>,
    ///The ID of the IP allowlist configuration.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: ::std::option::Option<String>,
    ///The name of the IP allowlist configuration.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: ::std::option::Option<String>,
}
///The details for events with this `type`.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct AuditLogIpAllowlistDeleted {
    ///The IP addresses or CIDR ranges that were in the configuration.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub allowed_ips: ::std::option::Option<Vec<String>>,
    ///The ID of the IP allowlist configuration.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: ::std::option::Option<String>,
    ///The name of the IP allowlist configuration.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: ::std::option::Option<String>,
}
///The details for events with this `type`.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct AuditLogIpAllowlistUpdated {
    ///The updated set of IP addresses or CIDR ranges in the configuration.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub allowed_ips: ::std::option::Option<Vec<String>>,
    ///The ID of the IP allowlist configuration.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: ::std::option::Option<String>,
}
///The details for events with this `type`.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct AuditLogLoginFailed {
    ///The error code of the failure.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error_code: ::std::option::Option<String>,
    ///The error message of the failure.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error_message: ::std::option::Option<String>,
}
///The details for events with this `type`.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct AuditLogLogoutFailed {
    ///The error code of the failure.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error_code: ::std::option::Option<String>,
    ///The error message of the failure.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error_message: ::std::option::Option<String>,
}
///The details for events with this `type`.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct AuditLogOrganizationUpdated {
    ///The payload used to update the organization settings.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub changes_requested: ::std::option::Option<
        AuditLogOrganizationUpdatedChangesRequested,
    >,
    ///The organization ID.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: ::std::option::Option<String>,
}
///The payload used to update the organization settings.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct AuditLogOrganizationUpdatedChangesRequested {
    ///How your organization logs data from supported API calls. One of `disabled`, `enabled_per_call`, `enabled_for_all_projects`, or `enabled_for_selected_projects`
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub api_call_logging: ::std::option::Option<String>,
    ///The list of project ids if api_call_logging is set to `enabled_for_selected_projects`
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub api_call_logging_project_ids: ::std::option::Option<String>,
    ///The organization description.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: ::std::option::Option<String>,
    ///The organization name.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: ::std::option::Option<String>,
    ///Visibility of the threads page which shows messages created with the Assistants API and Playground. One of `ANY_ROLE`, `OWNERS`, or `NONE`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub threads_ui_visibility: ::std::option::Option<String>,
    ///The organization title.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: ::std::option::Option<String>,
    ///Visibility of the usage dashboard which shows activity and costs for your organization. One of `ANY_ROLE` or `OWNERS`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub usage_dashboard_visibility: ::std::option::Option<String>,
}
///The project that the action was scoped to. Absent for actions not scoped to projects. Note that any admin actions taken via Admin API keys are associated with the default project.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct AuditLogProject {
    ///The project ID.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: ::std::option::Option<String>,
    ///The project title.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: ::std::option::Option<String>,
}
///The details for events with this `type`.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct AuditLogProjectArchived {
    ///The project ID.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: ::std::option::Option<String>,
}
///The details for events with this `type`.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct AuditLogProjectCreated {
    ///The payload used to create the project.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub data: ::std::option::Option<AuditLogProjectCreatedData>,
    ///The project ID.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: ::std::option::Option<String>,
}
///The payload used to create the project.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct AuditLogProjectCreatedData {
    ///The project name.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: ::std::option::Option<String>,
    ///The title of the project as seen on the dashboard.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: ::std::option::Option<String>,
}
///The details for events with this `type`.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct AuditLogProjectDeleted {
    ///The project ID.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: ::std::option::Option<String>,
}
///The details for events with this `type`.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct AuditLogProjectUpdated {
    ///The payload used to update the project.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub changes_requested: ::std::option::Option<AuditLogProjectUpdatedChangesRequested>,
    ///The project ID.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: ::std::option::Option<String>,
}
///The payload used to update the project.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct AuditLogProjectUpdatedChangesRequested {
    ///The title of the project as seen on the dashboard.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: ::std::option::Option<String>,
}
///The details for events with this `type`.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct AuditLogRateLimitDeleted {
    ///The rate limit ID
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: ::std::option::Option<String>,
}
///The details for events with this `type`.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct AuditLogRateLimitUpdated {
    ///The payload used to update the rate limits.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub changes_requested: ::std::option::Option<
        AuditLogRateLimitUpdatedChangesRequested,
    >,
    ///The rate limit ID
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: ::std::option::Option<String>,
}
///The payload used to update the rate limits.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct AuditLogRateLimitUpdatedChangesRequested {
    ///The maximum batch input tokens per day. Only relevant for certain models.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub batch_1_day_max_input_tokens: ::std::option::Option<i32>,
    ///The maximum audio megabytes per minute. Only relevant for certain models.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_audio_megabytes_per_1_minute: ::std::option::Option<i32>,
    ///The maximum images per minute. Only relevant for certain models.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_images_per_1_minute: ::std::option::Option<i32>,
    ///The maximum requests per day. Only relevant for certain models.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_requests_per_1_day: ::std::option::Option<i32>,
    ///The maximum requests per minute.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_requests_per_1_minute: ::std::option::Option<i32>,
    ///The maximum tokens per minute.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_tokens_per_1_minute: ::std::option::Option<i32>,
}
///The details for events with this `type`.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct AuditLogRoleAssignmentCreated {
    ///The identifier of the role assignment.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: ::std::option::Option<String>,
    ///The principal (user or group) that received the role.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub principal_id: ::std::option::Option<String>,
    ///The type of principal (user or group) that received the role.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub principal_type: ::std::option::Option<String>,
    ///The resource the role assignment is scoped to.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resource_id: ::std::option::Option<String>,
    ///The type of resource the role assignment is scoped to.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resource_type: ::std::option::Option<String>,
}
///The details for events with this `type`.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct AuditLogRoleAssignmentDeleted {
    ///The identifier of the role assignment.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: ::std::option::Option<String>,
    ///The principal (user or group) that had the role removed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub principal_id: ::std::option::Option<String>,
    ///The type of principal (user or group) that had the role removed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub principal_type: ::std::option::Option<String>,
    ///The resource the role assignment was scoped to.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resource_id: ::std::option::Option<String>,
    ///The type of resource the role assignment was scoped to.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resource_type: ::std::option::Option<String>,
}
///The details for events with this `type`.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct AuditLogRoleCreated {
    ///The role ID.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: ::std::option::Option<String>,
    ///The permissions granted by the role.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub permissions: ::std::option::Option<Vec<String>>,
    ///The resource the role is scoped to.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resource_id: ::std::option::Option<String>,
    ///The type of resource the role belongs to.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resource_type: ::std::option::Option<String>,
    ///The name of the role.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub role_name: ::std::option::Option<String>,
}
///The details for events with this `type`.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct AuditLogRoleDeleted {
    ///The role ID.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: ::std::option::Option<String>,
}
///The details for events with this `type`.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct AuditLogRoleUpdated {
    ///The payload used to update the role.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub changes_requested: ::std::option::Option<AuditLogRoleUpdatedChangesRequested>,
    ///The role ID.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: ::std::option::Option<String>,
}
///The payload used to update the role.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct AuditLogRoleUpdatedChangesRequested {
    ///The updated role description, when provided.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: ::std::option::Option<String>,
    ///Additional metadata stored on the role.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub metadata: ::std::option::Option<OpenAiJsonValue>,
    ///The permissions added to the role.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub permissions_added: ::std::option::Option<Vec<String>>,
    ///The permissions removed from the role.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub permissions_removed: ::std::option::Option<Vec<String>>,
    ///The resource the role is scoped to.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resource_id: ::std::option::Option<String>,
    ///The type of resource the role belongs to.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resource_type: ::std::option::Option<String>,
    ///The updated role name, when provided.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub role_name: ::std::option::Option<String>,
}
///The details for events with this `type`.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct AuditLogScimDisabled {
    ///The ID of the SCIM was disabled for.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: ::std::option::Option<String>,
}
///The details for events with this `type`.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct AuditLogScimEnabled {
    ///The ID of the SCIM was enabled for.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: ::std::option::Option<String>,
}
///The details for events with this `type`.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct AuditLogServiceAccountCreated {
    ///The payload used to create the service account.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub data: ::std::option::Option<AuditLogServiceAccountCreatedData>,
    ///The service account ID.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: ::std::option::Option<String>,
}
///The payload used to create the service account.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct AuditLogServiceAccountCreatedData {
    ///The role of the service account. Is either `owner` or `member`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub role: ::std::option::Option<String>,
}
///The details for events with this `type`.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct AuditLogServiceAccountDeleted {
    ///The service account ID.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: ::std::option::Option<String>,
}
///The details for events with this `type`.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct AuditLogServiceAccountUpdated {
    ///The payload used to updated the service account.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub changes_requested: ::std::option::Option<
        AuditLogServiceAccountUpdatedChangesRequested,
    >,
    ///The service account ID.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: ::std::option::Option<String>,
}
///The payload used to updated the service account.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct AuditLogServiceAccountUpdatedChangesRequested {
    ///The role of the service account. Is either `owner` or `member`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub role: ::std::option::Option<String>,
}
///The details for events with this `type`.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct AuditLogUserAdded {
    ///The payload used to add the user to the project.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub data: ::std::option::Option<AuditLogUserAddedData>,
    ///The user ID.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: ::std::option::Option<String>,
}
///The payload used to add the user to the project.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct AuditLogUserAddedData {
    ///The role of the user. Is either `owner` or `member`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub role: ::std::option::Option<String>,
}
///The details for events with this `type`.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct AuditLogUserDeleted {
    ///The user ID.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: ::std::option::Option<String>,
}
///The details for events with this `type`.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct AuditLogUserUpdated {
    ///The payload used to update the user.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub changes_requested: ::std::option::Option<AuditLogUserUpdatedChangesRequested>,
    ///The project ID.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: ::std::option::Option<String>,
}
///The payload used to update the user.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct AuditLogUserUpdatedChangesRequested {
    ///The role of the user. Is either `owner` or `member`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub role: ::std::option::Option<String>,
}
///The default strategy. This strategy currently uses a `max_chunk_size_tokens` of `800` and `chunk_overlap_tokens` of `400`.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct AutoChunkingStrategyRequestParam {
    ///Always `auto`.
    #[serde(rename = "type")]
    pub type_value: String,
}
///Configuration for a code interpreter container. Optionally specify the IDs of the files to run the code on.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct AutoCodeInterpreterToolParam {
    ///An optional list of uploaded files to make available to your code.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub file_ids: ::std::option::Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub memory_limit: ::std::option::Option<ContainerMemoryLimit>,
    ///Network access policy for the container.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub network_policy: ::std::option::Option<AutoCodeInterpreterToolParamNetworkPolicy>,
    ///Always `auto`.
    #[serde(rename = "type")]
    pub type_value: String,
}
///Network access policy for the container.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
#[serde(untagged)]
pub enum AutoCodeInterpreterToolParamNetworkPolicy {
    ContainerNetworkPolicyDisabledParam(ContainerNetworkPolicyDisabledParam),
    ContainerNetworkPolicyAllowlistParam(ContainerNetworkPolicyAllowlistParam),
}
///Controls whether ChatKit automatically generates thread titles.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct AutomaticThreadTitlingParam {
    ///Enable automatic thread title generation. Defaults to true.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub enabled: ::std::option::Option<bool>,
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct Batch {
    ///The Unix timestamp (in seconds) for when the batch was cancelled.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cancelled_at: ::std::option::Option<i64>,
    ///The Unix timestamp (in seconds) for when the batch started cancelling.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cancelling_at: ::std::option::Option<i64>,
    ///The Unix timestamp (in seconds) for when the batch was completed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub completed_at: ::std::option::Option<i64>,
    ///The time frame within which the batch should be processed.
    pub completion_window: String,
    ///The Unix timestamp (in seconds) for when the batch was created.
    pub created_at: i64,
    ///The OpenAI API endpoint used by the batch.
    pub endpoint: String,
    ///The ID of the file containing the outputs of requests with errors.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error_file_id: ::std::option::Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub errors: ::std::option::Option<BatchErrors>,
    ///The Unix timestamp (in seconds) for when the batch expired.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expired_at: ::std::option::Option<i64>,
    ///The Unix timestamp (in seconds) for when the batch will expire.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expires_at: ::std::option::Option<i64>,
    ///The Unix timestamp (in seconds) for when the batch failed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub failed_at: ::std::option::Option<i64>,
    ///The Unix timestamp (in seconds) for when the batch started finalizing.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub finalizing_at: ::std::option::Option<i64>,
    pub id: String,
    ///The Unix timestamp (in seconds) for when the batch started processing.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub in_progress_at: ::std::option::Option<i64>,
    ///The ID of the input file for the batch.
    pub input_file_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub metadata: ::std::option::Option<Metadata>,
    ///Model ID used to process the batch, like `gpt-5-2025-08-07`. OpenAI offers a wide range of models with different capabilities, performance characteristics, and price points. Refer to the [model guide](/docs/models) to browse and compare available models.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model: ::std::option::Option<String>,
    ///The object type, which is always `batch`.
    pub object: String,
    ///The ID of the file containing the outputs of successfully executed requests.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub output_file_id: ::std::option::Option<String>,
    ///The request counts for different statuses within the batch.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub request_counts: ::std::option::Option<BatchRequestCounts>,
    ///The current status of the batch.
    pub status: String,
    ///Represents token usage details including input tokens, output tokens, a breakdown of output tokens, and the total tokens used. Only populated on batches created after September 7, 2025.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub usage: ::std::option::Option<BatchUsage>,
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct BatchErrors {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub data: ::std::option::Option<Vec<BatchErrorsDataItem>>,
    ///The object type, which is always `list`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub object: ::std::option::Option<String>,
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct BatchErrorsDataItem {
    ///An error code identifying the error type.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub code: ::std::option::Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub line: ::std::option::Option<i32>,
    ///A human-readable message providing more details about the error.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub message: ::std::option::Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub param: ::std::option::Option<String>,
}
///The expiration policy for the output and/or error file that are generated for a batch.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct BatchFileExpirationAfter {
    ///Anchor timestamp after which the expiration policy applies. Supported anchors: `created_at`. Note that the anchor is the file creation time, not the time the batch is created.
    pub anchor: String,
    ///The number of seconds after the anchor time that the file will expire. Must be between 3600 (1 hour) and 2592000 (30 days).
    pub seconds: i64,
}
///The request counts for different statuses within the batch.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct BatchRequestCounts {
    ///Number of requests that have been completed successfully.
    pub completed: i32,
    ///Number of requests that have failed.
    pub failed: i32,
    ///Total number of requests in the batch.
    pub total: i32,
}
///Represents token usage details including input tokens, output tokens, a breakdown of output tokens, and the total tokens used. Only populated on batches created after September 7, 2025.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct BatchUsage {
    ///The number of input tokens.
    pub input_tokens: i32,
    ///A detailed breakdown of the input tokens.
    pub input_tokens_details: BatchUsageInputTokensDetails,
    ///The number of output tokens.
    pub output_tokens: i32,
    ///A detailed breakdown of the output tokens.
    pub output_tokens_details: BatchUsageOutputTokensDetails,
    ///The total number of tokens used.
    pub total_tokens: i32,
}
///A detailed breakdown of the input tokens.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct BatchUsageInputTokensDetails {
    ///The number of tokens that were retrieved from the cache. [More on prompt caching](/docs/guides/prompt-caching).
    pub cached_tokens: i32,
}
///A detailed breakdown of the output tokens.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct BatchUsageOutputTokensDetails {
    ///The number of reasoning tokens.
    pub reasoning_tokens: i32,
}
///Represents an individual `certificate` uploaded to the organization.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct Certificate {
    ///Whether the certificate is currently active at the specified scope. Not returned when getting details for a specific certificate.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub active: ::std::option::Option<bool>,
    pub certificate_details: CertificateCertificateDetails,
    ///The Unix timestamp (in seconds) of when the certificate was uploaded.
    pub created_at: i64,
    ///The identifier, which can be referenced in API endpoints
    pub id: String,
    ///The name of the certificate.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: ::std::option::Option<String>,
    ///The object type. - If creating, updating, or getting a specific certificate, the object type is `certificate`. - If listing, activating, or deactivating certificates for the organization, the object type is `organization.certificate`. - If listing, activating, or deactivating certificates for a project, the object type is `organization.project.certificate`.
    pub object: String,
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct CertificateCertificateDetails {
    ///The content of the certificate in PEM format.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub content: ::std::option::Option<String>,
    ///The Unix timestamp (in seconds) of when the certificate expires.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expires_at: ::std::option::Option<i64>,
    ///The Unix timestamp (in seconds) of when the certificate becomes valid.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub valid_at: ::std::option::Option<i64>,
}
///Constrains the tools available to the model to a pre-defined set.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct ChatCompletionAllowedTools {
    ///Constrains the tools available to the model to a pre-defined set. `auto` allows the model to pick from among the allowed tools and generate a message. `required` requires the model to call one or more of the allowed tools.
    pub mode: String,
    ///A list of tool definitions that the model should be allowed to call. For the Chat Completions API, the list of tool definitions might look like: ```json [ { "type": "function", "function": { "name": "get_weather" } }, { "type": "function", "function": { "name": "get_time" } } ] ```
    pub tools: Vec<OpenAiJsonValue>,
}
///Constrains the tools available to the model to a pre-defined set.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct ChatCompletionAllowedToolsChoice {
    pub allowed_tools: ChatCompletionAllowedTools,
    ///Allowed tool configuration type. Always `allowed_tools`.
    #[serde(rename = "type")]
    pub type_value: String,
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct ChatCompletionDeleted {
    ///Whether the chat completion was deleted.
    pub deleted: bool,
    ///The ID of the chat completion that was deleted.
    pub id: String,
    ///The type of object being deleted.
    pub object: String,
}
///Specifying a particular function via `{"name": "my_function"}` forces the model to call that function.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct ChatCompletionFunctionCallOption {
    ///The name of the function to call.
    pub name: String,
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct ChatCompletionFunctions {
    ///A description of what the function does, used by the model to choose when and how to call the function.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: ::std::option::Option<String>,
    ///The name of the function to be called. Must be a-z, A-Z, 0-9, or contain underscores and dashes, with a maximum length of 64.
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parameters: ::std::option::Option<FunctionParameters>,
}
///An object representing a list of Chat Completions.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct ChatCompletionList {
    ///An array of chat completion objects.
    pub data: Vec<CreateChatCompletionResponse>,
    ///The identifier of the first chat completion in the data array.
    pub first_id: String,
    ///Indicates whether there are more Chat Completions available.
    pub has_more: bool,
    ///The identifier of the last chat completion in the data array.
    pub last_id: String,
    ///The type of this object. It is always set to "list".
    pub object: String,
}
///A call to a custom tool created by the model.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct ChatCompletionMessageCustomToolCall {
    ///The custom tool that the model called.
    pub custom: ChatCompletionMessageCustomToolCallCustom,
    ///The ID of the tool call.
    pub id: String,
    ///The type of the tool. Always `custom`.
    #[serde(rename = "type")]
    pub type_value: String,
}
///The custom tool that the model called.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct ChatCompletionMessageCustomToolCallCustom {
    ///The input for the custom tool call generated by the model.
    pub input: String,
    ///The name of the custom tool to call.
    pub name: String,
}
///An object representing a list of chat completion messages.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct ChatCompletionMessageList {
    ///An array of chat completion message objects.
    pub data: Vec<ChatCompletionMessageListDataItem>,
    ///The identifier of the first chat message in the data array.
    pub first_id: String,
    ///Indicates whether there are more chat messages available.
    pub has_more: bool,
    ///The identifier of the last chat message in the data array.
    pub last_id: String,
    ///The type of this object. It is always set to "list".
    pub object: String,
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct ChatCompletionMessageListDataItem {
    ///Annotations for the message, when applicable, as when using the [web search tool](/docs/guides/tools-web-search?api-mode=chat).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub annotations: ::std::option::Option<
        Vec<ChatCompletionMessageListDataItemAnnotation>,
    >,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub audio: ::std::option::Option<ChatCompletionMessageListDataItemAudio>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub content: ::std::option::Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub content_parts: ::std::option::Option<
        Vec<ChatCompletionMessageListDataItemContentPart>,
    >,
    ///Deprecated and replaced by `tool_calls`. The name and arguments of a function that should be called, as generated by the model.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub function_call: ::std::option::Option<
        ChatCompletionMessageListDataItemFunctionCall,
    >,
    ///The identifier of the chat message.
    pub id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub refusal: ::std::option::Option<String>,
    ///The role of the author of this message.
    pub role: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tool_calls: ::std::option::Option<ChatCompletionMessageToolCalls>,
}
///A URL citation when using web search.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct ChatCompletionMessageListDataItemAnnotation {
    ///The type of the URL citation. Always `url_citation`.
    #[serde(rename = "type")]
    pub type_value: String,
    ///A URL citation when using web search.
    pub url_citation: ChatCompletionMessageListDataItemAnnotationUrlCitation,
}
///A URL citation when using web search.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct ChatCompletionMessageListDataItemAnnotationUrlCitation {
    ///The index of the last character of the URL citation in the message.
    pub end_index: i32,
    ///The index of the first character of the URL citation in the message.
    pub start_index: i32,
    ///The title of the web resource.
    pub title: String,
    ///The URL of the web resource.
    pub url: String,
}
///If the audio output modality is requested, this object contains data about the audio response from the model. [Learn more](/docs/guides/audio).
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct ChatCompletionMessageListDataItemAudio {
    ///Base64 encoded audio bytes generated by the model, in the format specified in the request.
    pub data: String,
    ///The Unix timestamp (in seconds) for when this audio response will no longer be accessible on the server for use in multi-turn conversations.
    pub expires_at: i64,
    ///Unique identifier for this audio response.
    pub id: String,
    ///Transcript of the audio generated by the model.
    pub transcript: String,
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
#[serde(untagged)]
pub enum ChatCompletionMessageListDataItemContentPart {
    ChatCompletionRequestMessageContentPartText(
        ChatCompletionRequestMessageContentPartText,
    ),
    ChatCompletionRequestMessageContentPartImage(
        ChatCompletionRequestMessageContentPartImage,
    ),
}
///Deprecated and replaced by `tool_calls`. The name and arguments of a function that should be called, as generated by the model.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct ChatCompletionMessageListDataItemFunctionCall {
    ///The arguments to call the function with, as generated by the model in JSON format. Note that the model does not always generate valid JSON, and may hallucinate parameters not defined by your function schema. Validate the arguments in your code before calling your function.
    pub arguments: String,
    ///The name of the function to call.
    pub name: String,
}
///A call to a function tool created by the model.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct ChatCompletionMessageToolCall {
    ///The function that the model called.
    pub function: ChatCompletionMessageToolCallFunction,
    ///The ID of the tool call.
    pub id: String,
    ///The type of the tool. Currently, only `function` is supported.
    #[serde(rename = "type")]
    pub type_value: String,
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
#[serde(untagged)]
pub enum ChatCompletionMessageToolCall2 {
    ChatCompletionMessageToolCall(ChatCompletionMessageToolCall),
    ChatCompletionMessageCustomToolCall(ChatCompletionMessageCustomToolCall),
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct ChatCompletionMessageToolCallChunk {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub function: ::std::option::Option<ChatCompletionMessageToolCallChunkFunction>,
    ///The ID of the tool call.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: ::std::option::Option<String>,
    pub index: i32,
    ///The type of the tool. Currently, only `function` is supported.
    #[serde(rename = "type", default, skip_serializing_if = "Option::is_none")]
    pub type_value: ::std::option::Option<String>,
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct ChatCompletionMessageToolCallChunkFunction {
    ///The arguments to call the function with, as generated by the model in JSON format. Note that the model does not always generate valid JSON, and may hallucinate parameters not defined by your function schema. Validate the arguments in your code before calling your function.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub arguments: ::std::option::Option<String>,
    ///The name of the function to call.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: ::std::option::Option<String>,
}
///The function that the model called.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct ChatCompletionMessageToolCallFunction {
    ///The arguments to call the function with, as generated by the model in JSON format. Note that the model does not always generate valid JSON, and may hallucinate parameters not defined by your function schema. Validate the arguments in your code before calling your function.
    pub arguments: String,
    ///The name of the function to call.
    pub name: String,
}
///The tool calls generated by the model, such as function calls.
pub type ChatCompletionMessageToolCalls = Vec<ChatCompletionMessageToolCall2>;
pub type ChatCompletionModalities = Vec<String>;
///Specifies a tool the model should use. Use to force the model to call a specific function.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct ChatCompletionNamedToolChoice {
    pub function: ChatCompletionNamedToolChoiceFunction,
    ///For function calling, the type is always `function`.
    #[serde(rename = "type")]
    pub type_value: String,
}
///Specifies a tool the model should use. Use to force the model to call a specific custom tool.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct ChatCompletionNamedToolChoiceCustom {
    pub custom: ChatCompletionNamedToolChoiceCustomCustom,
    ///For custom tool calling, the type is always `custom`.
    #[serde(rename = "type")]
    pub type_value: String,
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct ChatCompletionNamedToolChoiceCustomCustom {
    ///The name of the custom tool to call.
    pub name: String,
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct ChatCompletionNamedToolChoiceFunction {
    ///The name of the function to call.
    pub name: String,
}
///Messages sent by the model in response to user messages.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct ChatCompletionRequestAssistantMessage {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub audio: ::std::option::Option<ChatCompletionRequestAssistantMessageAudio>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub content: ::std::option::Option<ChatCompletionRequestAssistantMessageContent>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub function_call: ::std::option::Option<
        ChatCompletionRequestAssistantMessageFunctionCall,
    >,
    ///An optional name for the participant. Provides the model information to differentiate between participants of the same role.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: ::std::option::Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub refusal: ::std::option::Option<String>,
    ///The role of the messages author, in this case `assistant`.
    pub role: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tool_calls: ::std::option::Option<ChatCompletionMessageToolCalls>,
}
///Data about a previous audio response from the model. [Learn more](/docs/guides/audio).
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct ChatCompletionRequestAssistantMessageAudio {
    ///Unique identifier for a previous audio response from the model.
    pub id: String,
}
///The contents of the assistant message. Required unless `tool_calls` or `function_call` is specified.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
#[serde(untagged)]
pub enum ChatCompletionRequestAssistantMessageContent {
    TextContent(String),
    ArrayOfContentParts(Vec<ChatCompletionRequestAssistantMessageContentPart>),
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
#[serde(untagged)]
pub enum ChatCompletionRequestAssistantMessageContentPart {
    ChatCompletionRequestMessageContentPartText(
        ChatCompletionRequestMessageContentPartText,
    ),
    ChatCompletionRequestMessageContentPartRefusal(
        ChatCompletionRequestMessageContentPartRefusal,
    ),
}
///Deprecated and replaced by `tool_calls`. The name and arguments of a function that should be called, as generated by the model.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct ChatCompletionRequestAssistantMessageFunctionCall {
    ///The arguments to call the function with, as generated by the model in JSON format. Note that the model does not always generate valid JSON, and may hallucinate parameters not defined by your function schema. Validate the arguments in your code before calling your function.
    pub arguments: String,
    ///The name of the function to call.
    pub name: String,
}
///Developer-provided instructions that the model should follow, regardless of messages sent by the user. With o1 models and newer, `developer` messages replace the previous `system` messages.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct ChatCompletionRequestDeveloperMessage {
    ///The contents of the developer message.
    pub content: ChatCompletionRequestDeveloperMessageContent,
    ///An optional name for the participant. Provides the model information to differentiate between participants of the same role.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: ::std::option::Option<String>,
    ///The role of the messages author, in this case `developer`.
    pub role: String,
}
///The contents of the developer message.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
#[serde(untagged)]
pub enum ChatCompletionRequestDeveloperMessageContent {
    TextContent(String),
    ArrayOfContentParts(Vec<ChatCompletionRequestMessageContentPartText>),
}
///Function message
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct ChatCompletionRequestFunctionMessage {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub content: ::std::option::Option<String>,
    ///The name of the function to call.
    pub name: String,
    ///The role of the messages author, in this case `function`.
    pub role: String,
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
#[serde(untagged)]
pub enum ChatCompletionRequestMessage {
    ChatCompletionRequestDeveloperMessage(ChatCompletionRequestDeveloperMessage),
    ChatCompletionRequestSystemMessage(ChatCompletionRequestSystemMessage),
    ChatCompletionRequestUserMessage(ChatCompletionRequestUserMessage),
    ChatCompletionRequestAssistantMessage(ChatCompletionRequestAssistantMessage),
    ChatCompletionRequestToolMessage(ChatCompletionRequestToolMessage),
    ChatCompletionRequestFunctionMessage(ChatCompletionRequestFunctionMessage),
}
///Learn about [audio inputs](/docs/guides/audio).
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct ChatCompletionRequestMessageContentPartAudio {
    pub input_audio: ChatCompletionRequestMessageContentPartAudioInputAudio,
    ///The type of the content part. Always `input_audio`.
    #[serde(rename = "type")]
    pub type_value: String,
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct ChatCompletionRequestMessageContentPartAudioInputAudio {
    ///Base64 encoded audio data.
    pub data: String,
    ///The format of the encoded audio data. Currently supports "wav" and "mp3".
    pub format: String,
}
///Learn about [file inputs](/docs/guides/text) for text generation.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct ChatCompletionRequestMessageContentPartFile {
    pub file: ChatCompletionRequestMessageContentPartFileFile,
    ///The type of the content part. Always `file`.
    #[serde(rename = "type")]
    pub type_value: String,
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct ChatCompletionRequestMessageContentPartFileFile {
    ///The base64 encoded file data, used when passing the file to the model as a string.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub file_data: ::std::option::Option<String>,
    ///The ID of an uploaded file to use as input.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub file_id: ::std::option::Option<String>,
    ///The name of the file, used when passing the file to the model as a string.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub filename: ::std::option::Option<String>,
}
///Learn about [image inputs](/docs/guides/vision).
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct ChatCompletionRequestMessageContentPartImage {
    pub image_url: ChatCompletionRequestMessageContentPartImageImageUrl,
    ///The type of the content part.
    #[serde(rename = "type")]
    pub type_value: String,
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct ChatCompletionRequestMessageContentPartImageImageUrl {
    ///Specifies the detail level of the image. Learn more in the [Vision guide](/docs/guides/vision#low-or-high-fidelity-image-understanding).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub detail: ::std::option::Option<String>,
    ///Either a URL of the image or the base64 encoded image data.
    pub url: String,
}
///Refusal content part
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct ChatCompletionRequestMessageContentPartRefusal {
    ///The refusal message generated by the model.
    pub refusal: String,
    ///The type of the content part.
    #[serde(rename = "type")]
    pub type_value: String,
}
///Learn about [text inputs](/docs/guides/text-generation).
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct ChatCompletionRequestMessageContentPartText {
    ///The text content.
    pub text: String,
    ///The type of the content part.
    #[serde(rename = "type")]
    pub type_value: String,
}
///Developer-provided instructions that the model should follow, regardless of messages sent by the user. With o1 models and newer, use `developer` messages for this purpose instead.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct ChatCompletionRequestSystemMessage {
    ///The contents of the system message.
    pub content: ChatCompletionRequestSystemMessageContent,
    ///An optional name for the participant. Provides the model information to differentiate between participants of the same role.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: ::std::option::Option<String>,
    ///The role of the messages author, in this case `system`.
    pub role: String,
}
///The contents of the system message.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
#[serde(untagged)]
pub enum ChatCompletionRequestSystemMessageContent {
    TextContent(String),
    ArrayOfContentParts(Vec<ChatCompletionRequestSystemMessageContentPart>),
}
pub type ChatCompletionRequestSystemMessageContentPart = ChatCompletionRequestMessageContentPartText;
///Tool message
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct ChatCompletionRequestToolMessage {
    ///The contents of the tool message.
    pub content: ChatCompletionRequestToolMessageContent,
    ///The role of the messages author, in this case `tool`.
    pub role: String,
    ///Tool call that this message is responding to.
    pub tool_call_id: String,
}
///The contents of the tool message.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
#[serde(untagged)]
pub enum ChatCompletionRequestToolMessageContent {
    TextContent(String),
    ArrayOfContentParts(Vec<ChatCompletionRequestToolMessageContentPart>),
}
pub type ChatCompletionRequestToolMessageContentPart = ChatCompletionRequestMessageContentPartText;
///Messages sent by an end user, containing prompts or additional context information.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct ChatCompletionRequestUserMessage {
    ///The contents of the user message.
    pub content: ChatCompletionRequestUserMessageContent,
    ///An optional name for the participant. Provides the model information to differentiate between participants of the same role.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: ::std::option::Option<String>,
    ///The role of the messages author, in this case `user`.
    pub role: String,
}
///The contents of the user message.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
#[serde(untagged)]
pub enum ChatCompletionRequestUserMessageContent {
    TextContent(String),
    ArrayOfContentParts(Vec<ChatCompletionRequestUserMessageContentPart>),
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
#[serde(untagged)]
pub enum ChatCompletionRequestUserMessageContentPart {
    ChatCompletionRequestMessageContentPartText(
        ChatCompletionRequestMessageContentPartText,
    ),
    ChatCompletionRequestMessageContentPartImage(
        ChatCompletionRequestMessageContentPartImage,
    ),
    ChatCompletionRequestMessageContentPartAudio(
        ChatCompletionRequestMessageContentPartAudio,
    ),
    ChatCompletionRequestMessageContentPartFile(
        ChatCompletionRequestMessageContentPartFile,
    ),
}
///A chat completion message generated by the model.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct ChatCompletionResponseMessage {
    ///Annotations for the message, when applicable, as when using the [web search tool](/docs/guides/tools-web-search?api-mode=chat).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub annotations: ::std::option::Option<Vec<ChatCompletionResponseMessageAnnotation>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub audio: ::std::option::Option<ChatCompletionResponseMessageAudio>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub content: ::std::option::Option<String>,
    ///Deprecated and replaced by `tool_calls`. The name and arguments of a function that should be called, as generated by the model.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub function_call: ::std::option::Option<ChatCompletionResponseMessageFunctionCall>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub refusal: ::std::option::Option<String>,
    ///The role of the author of this message.
    pub role: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tool_calls: ::std::option::Option<ChatCompletionMessageToolCalls>,
}
///A URL citation when using web search.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct ChatCompletionResponseMessageAnnotation {
    ///The type of the URL citation. Always `url_citation`.
    #[serde(rename = "type")]
    pub type_value: String,
    ///A URL citation when using web search.
    pub url_citation: ChatCompletionResponseMessageAnnotationUrlCitation,
}
///A URL citation when using web search.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct ChatCompletionResponseMessageAnnotationUrlCitation {
    ///The index of the last character of the URL citation in the message.
    pub end_index: i32,
    ///The index of the first character of the URL citation in the message.
    pub start_index: i32,
    ///The title of the web resource.
    pub title: String,
    ///The URL of the web resource.
    pub url: String,
}
///If the audio output modality is requested, this object contains data about the audio response from the model. [Learn more](/docs/guides/audio).
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct ChatCompletionResponseMessageAudio {
    ///Base64 encoded audio bytes generated by the model, in the format specified in the request.
    pub data: String,
    ///The Unix timestamp (in seconds) for when this audio response will no longer be accessible on the server for use in multi-turn conversations.
    pub expires_at: i64,
    ///Unique identifier for this audio response.
    pub id: String,
    ///Transcript of the audio generated by the model.
    pub transcript: String,
}
///Deprecated and replaced by `tool_calls`. The name and arguments of a function that should be called, as generated by the model.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct ChatCompletionResponseMessageFunctionCall {
    ///The arguments to call the function with, as generated by the model in JSON format. Note that the model does not always generate valid JSON, and may hallucinate parameters not defined by your function schema. Validate the arguments in your code before calling your function.
    pub arguments: String,
    ///The name of the function to call.
    pub name: String,
}
///The role of the author of a message
pub type ChatCompletionRole = String;
pub type ChatCompletionStreamOptions = ChatCompletionStreamOptions2;
///Options for streaming response. Only set this when you set `stream: true`.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct ChatCompletionStreamOptions2 {
    ///When true, stream obfuscation will be enabled. Stream obfuscation adds random characters to an `obfuscation` field on streaming delta events to normalize payload sizes as a mitigation to certain side-channel attacks. These obfuscation fields are included by default, but add a small amount of overhead to the data stream. You can set `include_obfuscation` to false to optimize for bandwidth if you trust the network links between your application and the OpenAI API.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub include_obfuscation: ::std::option::Option<bool>,
    ///If set, an additional chunk will be streamed before the `data: [DONE]` message. The `usage` field on this chunk shows the token usage statistics for the entire request, and the `choices` field will always be an empty array. All other chunks will also include a `usage` field, but with a null value. **NOTE:** If the stream is interrupted, you may not receive the final usage chunk which contains the total token usage for the request.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub include_usage: ::std::option::Option<bool>,
}
///A chat completion delta generated by streamed model responses.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct ChatCompletionStreamResponseDelta {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub content: ::std::option::Option<String>,
    ///Deprecated and replaced by `tool_calls`. The name and arguments of a function that should be called, as generated by the model.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub function_call: ::std::option::Option<
        ChatCompletionStreamResponseDeltaFunctionCall,
    >,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub refusal: ::std::option::Option<String>,
    ///The role of the author of this message.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub role: ::std::option::Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tool_calls: ::std::option::Option<Vec<ChatCompletionMessageToolCallChunk>>,
}
///Deprecated and replaced by `tool_calls`. The name and arguments of a function that should be called, as generated by the model.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct ChatCompletionStreamResponseDeltaFunctionCall {
    ///The arguments to call the function with, as generated by the model in JSON format. Note that the model does not always generate valid JSON, and may hallucinate parameters not defined by your function schema. Validate the arguments in your code before calling your function.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub arguments: ::std::option::Option<String>,
    ///The name of the function to call.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: ::std::option::Option<String>,
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct ChatCompletionTokenLogprob {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bytes: ::std::option::Option<Vec<i32>>,
    ///The log probability of this token, if it is within the top 20 most likely tokens. Otherwise, the value `-9999.0` is used to signify that the token is very unlikely.
    pub logprob: f64,
    ///The token.
    pub token: String,
    ///List of the most likely tokens and their log probability, at this token position. The number of entries may be fewer than the requested `top_logprobs`.
    pub top_logprobs: Vec<ChatCompletionTokenLogprobTopLogprob>,
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct ChatCompletionTokenLogprobTopLogprob {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bytes: ::std::option::Option<Vec<i32>>,
    ///The log probability of this token, if it is within the top 20 most likely tokens. Otherwise, the value `-9999.0` is used to signify that the token is very unlikely.
    pub logprob: f64,
    ///The token.
    pub token: String,
}
///A function tool that can be used to generate a response.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct ChatCompletionTool {
    pub function: FunctionObject,
    ///The type of the tool. Currently, only `function` is supported.
    #[serde(rename = "type")]
    pub type_value: String,
}
///Controls which (if any) tool is called by the model. `none` means the model will not call any tool and instead generates a message. `auto` means the model can pick between generating a message or calling one or more tools. `required` means the model must call one or more tools. Specifying a particular tool via `{"type": "function", "function": {"name": "my_function"}}` forces the model to call that tool. `none` is the default when no tools are present. `auto` is the default if tools are present.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
#[serde(untagged)]
pub enum ChatCompletionToolChoiceOption {
    ToolChoiceMode(String),
    ChatCompletionAllowedToolsChoice(ChatCompletionAllowedToolsChoice),
    ChatCompletionNamedToolChoice(ChatCompletionNamedToolChoice),
    ChatCompletionNamedToolChoiceCustom(ChatCompletionNamedToolChoiceCustom),
}
///Automatic thread title preferences for the session.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct ChatSessionAutomaticThreadTitling {
    ///Whether automatic thread titling is enabled.
    pub enabled: bool,
}
///ChatKit configuration for the session.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct ChatSessionChatkitConfiguration {
    ///Automatic thread titling preferences.
    pub automatic_thread_titling: ChatSessionAutomaticThreadTitling,
    ///Upload settings for the session.
    pub file_upload: ChatSessionFileUpload,
    ///History retention configuration.
    pub history: ChatSessionHistory,
}
///Upload permissions and limits applied to the session.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct ChatSessionFileUpload {
    ///Indicates if uploads are enabled for the session.
    pub enabled: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_file_size: ::std::option::Option<i32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_files: ::std::option::Option<i32>,
}
///History retention preferences returned for the session.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct ChatSessionHistory {
    ///Indicates if chat history is persisted for the session.
    pub enabled: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub recent_threads: ::std::option::Option<i32>,
}
///Active per-minute request limit for the session.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct ChatSessionRateLimits {
    ///Maximum allowed requests per one-minute window.
    pub max_requests_per_1_minute: i32,
}
///Represents a ChatKit session and its resolved configuration.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct ChatSessionResource {
    ///Resolved ChatKit feature configuration for the session.
    pub chatkit_configuration: ChatSessionChatkitConfiguration,
    ///Ephemeral client secret that authenticates session requests.
    pub client_secret: String,
    ///Unix timestamp (in seconds) for when the session expires.
    pub expires_at: i64,
    ///Identifier for the ChatKit session.
    pub id: String,
    ///Convenience copy of the per-minute request limit.
    pub max_requests_per_1_minute: i32,
    ///Type discriminator that is always `chatkit.session`.
    pub object: String,
    ///Resolved rate limit values.
    pub rate_limits: ChatSessionRateLimits,
    ///Current lifecycle state of the session.
    pub status: ChatSessionStatus,
    ///User identifier associated with the session.
    pub user: String,
    ///Workflow metadata for the session.
    pub workflow: ChatkitWorkflow,
}
pub type ChatSessionStatus = String;
///Optional per-session configuration settings for ChatKit behavior.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct ChatkitConfigurationParam {
    ///Configuration for automatic thread titling. When omitted, automatic thread titling is enabled by default.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub automatic_thread_titling: ::std::option::Option<AutomaticThreadTitlingParam>,
    ///Configuration for upload enablement and limits. When omitted, uploads are disabled by default (max_files 10, max_file_size 512 MB).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub file_upload: ::std::option::Option<FileUploadParam>,
    ///Configuration for chat history retention. When omitted, history is enabled by default with no limit on recent_threads (null).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub history: ::std::option::Option<HistoryParam>,
}
///Workflow metadata and state returned for the session.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct ChatkitWorkflow {
    ///Identifier of the workflow backing the session.
    pub id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub state_variables: ::std::option::Option<OpenAiJsonValue>,
    ///Tracing settings applied to the workflow.
    pub tracing: ChatkitWorkflowTracing,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub version: ::std::option::Option<String>,
}
///Controls diagnostic tracing during the session.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct ChatkitWorkflowTracing {
    ///Indicates whether tracing is enabled.
    pub enabled: bool,
}
///The chunking strategy used to chunk the file(s). If not set, will use the `auto` strategy.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct ChunkingStrategyRequestParam {
    #[serde(flatten)]
    pub value: OpenAiJsonObject,
}
pub type ClickButtonType = String;
///A click action.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct ClickParam {
    ///Indicates which mouse button was pressed during the click. One of `left`, `right`, `wheel`, `back`, or `forward`.
    pub button: ClickButtonType,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub keys: ::std::option::Option<Vec<String>>,
    ///Specifies the event type. For a click action, this property is always `click`.
    #[serde(rename = "type")]
    pub type_value: String,
    ///The x-coordinate where the click occurred.
    pub x: i32,
    ///The y-coordinate where the click occurred.
    pub y: i32,
}
///Record of a client side tool invocation initiated by the assistant.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct ClientToolCallItem {
    ///JSON-encoded arguments that were sent to the tool.
    pub arguments: String,
    ///Identifier for the client tool call.
    pub call_id: String,
    ///Unix timestamp (in seconds) for when the item was created.
    pub created_at: i64,
    ///Identifier of the thread item.
    pub id: String,
    ///Tool name that was invoked.
    pub name: String,
    ///Type discriminator that is always `chatkit.thread_item`.
    pub object: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub output: ::std::option::Option<String>,
    ///Execution status for the tool call.
    pub status: ClientToolCallStatus,
    ///Identifier of the parent thread.
    pub thread_id: String,
    ///Type discriminator that is always `chatkit.client_tool_call`.
    #[serde(rename = "type")]
    pub type_value: String,
}
pub type ClientToolCallStatus = String;
///Indicates that a thread has been closed.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct ClosedStatus {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reason: ::std::option::Option<String>,
    ///Status discriminator that is always `closed`.
    #[serde(rename = "type")]
    pub type_value: String,
}
///The output of a code interpreter tool call that is a file.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct CodeInterpreterFileOutput {
    pub files: Vec<CodeInterpreterFileOutputFile>,
    ///The type of the code interpreter file output. Always `files`.
    #[serde(rename = "type")]
    pub type_value: String,
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct CodeInterpreterFileOutputFile {
    ///The ID of the file.
    pub file_id: String,
    ///The MIME type of the file.
    pub mime_type: String,
}
///The image output from the code interpreter.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct CodeInterpreterOutputImage {
    ///The type of the output. Always `image`.
    #[serde(rename = "type")]
    pub type_value: String,
    ///The URL of the image output from the code interpreter.
    pub url: String,
}
///The logs output from the code interpreter.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct CodeInterpreterOutputLogs {
    ///The logs output from the code interpreter.
    pub logs: String,
    ///The type of the output. Always `logs`.
    #[serde(rename = "type")]
    pub type_value: String,
}
///The output of a code interpreter tool call that is text.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct CodeInterpreterTextOutput {
    ///The logs of the code interpreter tool call.
    pub logs: String,
    ///The type of the code interpreter text output. Always `logs`.
    #[serde(rename = "type")]
    pub type_value: String,
}
///A tool that runs Python code to help generate a response to a prompt.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct CodeInterpreterTool {
    ///The code interpreter container. Can be a container ID or an object that specifies uploaded file IDs to make available to your code, along with an optional `memory_limit` setting.
    pub container: CodeInterpreterToolContainer,
    ///The type of the code interpreter tool. Always `code_interpreter`.
    #[serde(rename = "type")]
    pub type_value: String,
}
///A tool call to run code.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct CodeInterpreterToolCall {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub code: ::std::option::Option<String>,
    ///The ID of the container used to run the code.
    pub container_id: String,
    ///The unique ID of the code interpreter tool call.
    pub id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub outputs: ::std::option::Option<Vec<CodeInterpreterToolCallOutput>>,
    ///The status of the code interpreter tool call. Valid values are `in_progress`, `completed`, `incomplete`, `interpreting`, and `failed`.
    pub status: String,
    ///The type of the code interpreter tool call. Always `code_interpreter_call`.
    #[serde(rename = "type")]
    pub type_value: String,
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
#[serde(untagged)]
pub enum CodeInterpreterToolCallOutput {
    CodeInterpreterOutputLogs(CodeInterpreterOutputLogs),
    CodeInterpreterOutputImage(CodeInterpreterOutputImage),
}
///The code interpreter container. Can be a container ID or an object that specifies uploaded file IDs to make available to your code, along with an optional `memory_limit` setting.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
#[serde(untagged)]
pub enum CodeInterpreterToolContainer {
    String(String),
    AutoCodeInterpreterToolParam(AutoCodeInterpreterToolParam),
}
///The compacted response object
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct CompactResource {
    ///Unix timestamp (in seconds) when the compacted conversation was created.
    pub created_at: i64,
    ///The unique identifier for the compacted response.
    pub id: String,
    ///The object type. Always `response.compaction`.
    pub object: String,
    ///The compacted list of output items.
    pub output: Vec<ItemField>,
    ///Token accounting for the compaction pass, including cached, reasoning, and total tokens.
    pub usage: ResponseUsage,
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct CompactResponseMethodPublicBody {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub input: ::std::option::Option<CompactResponseMethodPublicBodyInput>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub instructions: ::std::option::Option<String>,
    pub model: ModelIdsCompaction,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub previous_response_id: ::std::option::Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub prompt_cache_key: ::std::option::Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub prompt_cache_retention: ::std::option::Option<PromptCacheRetentionEnum>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub service_tier: ::std::option::Option<ServiceTierEnum>,
}
///Text, image, or file inputs to the model, used to generate a response
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
#[serde(untagged)]
pub enum CompactResponseMethodPublicBodyInput {
    String(String),
    Array(Vec<InputItem>),
}
///A compaction item generated by the [`v1/responses/compact` API](/docs/api-reference/responses/compact).
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct CompactionBody {
    ///The identifier of the actor that created the item.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub created_by: ::std::option::Option<String>,
    ///The encrypted content that was produced by compaction.
    pub encrypted_content: String,
    ///The unique ID of the compaction item.
    pub id: String,
    ///The type of the item. Always `compaction`.
    #[serde(rename = "type")]
    pub type_value: String,
}
///A compaction item generated by the [`v1/responses/compact` API](/docs/api-reference/responses/compact).
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct CompactionSummaryItemParam {
    ///The encrypted content of the compaction summary.
    pub encrypted_content: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: ::std::option::Option<String>,
    ///The type of the item. Always `compaction`.
    #[serde(rename = "type")]
    pub type_value: String,
}
///A filter used to compare a specified attribute key to a given value using a defined comparison operation.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct ComparisonFilter {
    ///The key to compare against the value.
    pub key: String,
    ///Specifies the comparison operator: `eq`, `ne`, `gt`, `gte`, `lt`, `lte`, `in`, `nin`. - `eq`: equals - `ne`: not equal - `gt`: greater than - `gte`: greater than or equal - `lt`: less than - `lte`: less than or equal - `in`: in - `nin`: not in
    #[serde(rename = "type")]
    pub type_value: String,
    ///The value to compare against the attribute key; supports string, number, or boolean types.
    pub value: ComparisonFilterValue,
}
///The value to compare against the attribute key; supports string, number, or boolean types.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
#[serde(untagged)]
pub enum ComparisonFilterValue {
    String(String),
    Number(f64),
    Boolean(bool),
    Array(Vec<ComparisonFilterValueArrayItem>),
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
#[serde(untagged)]
pub enum ComparisonFilterValueArrayItem {
    String(String),
    Number(f64),
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
#[serde(untagged)]
pub enum ComparisonFilterValueItem {
    String(String),
    Number(f64),
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct CompleteUploadRequest {
    ///The optional md5 checksum for the file contents to verify if the bytes uploaded matches what you expect.
    #[serde(rename = "md5", default, skip_serializing_if = "Option::is_none")]
    pub md_5: ::std::option::Option<String>,
    ///The ordered list of Part IDs.
    pub part_ids: Vec<String>,
}
///Usage statistics for the completion request.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct CompletionUsage {
    ///Number of tokens in the generated completion.
    pub completion_tokens: i32,
    ///Breakdown of tokens used in a completion.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub completion_tokens_details: ::std::option::Option<
        CompletionUsageCompletionTokensDetails,
    >,
    ///Number of tokens in the prompt.
    pub prompt_tokens: i32,
    ///Breakdown of tokens used in the prompt.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub prompt_tokens_details: ::std::option::Option<CompletionUsagePromptTokensDetails>,
    ///Total number of tokens used in the request (prompt + completion).
    pub total_tokens: i32,
}
///Breakdown of tokens used in a completion.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct CompletionUsageCompletionTokensDetails {
    ///When using Predicted Outputs, the number of tokens in the prediction that appeared in the completion.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub accepted_prediction_tokens: ::std::option::Option<i32>,
    ///Audio input tokens generated by the model.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub audio_tokens: ::std::option::Option<i32>,
    ///Tokens generated by the model for reasoning.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reasoning_tokens: ::std::option::Option<i32>,
    ///When using Predicted Outputs, the number of tokens in the prediction that did not appear in the completion. However, like reasoning tokens, these tokens are still counted in the total completion tokens for purposes of billing, output, and context window limits.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rejected_prediction_tokens: ::std::option::Option<i32>,
}
///Breakdown of tokens used in the prompt.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct CompletionUsagePromptTokensDetails {
    ///Audio input tokens present in the prompt.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub audio_tokens: ::std::option::Option<i32>,
    ///Cached tokens present in the prompt.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cached_tokens: ::std::option::Option<i32>,
}
///Combine multiple filters using `and` or `or`.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct CompoundFilter {
    ///Array of filters to combine. Items can be `ComparisonFilter` or `CompoundFilter`.
    pub filters: Vec<CompoundFilterFilter>,
    ///Type of operation: `and` or `or`.
    #[serde(rename = "type")]
    pub type_value: String,
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
#[serde(untagged)]
pub enum CompoundFilterFilter {
    ComparisonFilter(ComparisonFilter),
    Variant2(OpenAiJsonValue),
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
#[serde(untagged)]
pub enum ComputerAction {
    ClickParam(ClickParam),
    DoubleClickAction(DoubleClickAction),
    DragParam(DragParam),
    KeyPressAction(KeyPressAction),
    MoveParam(MoveParam),
    ScreenshotParam(ScreenshotParam),
    ScrollParam(ScrollParam),
    TypeParam(TypeParam),
    WaitParam(WaitParam),
}
///Flattened batched actions for `computer_use`. Each action includes an `type` discriminator and action-specific fields.
pub type ComputerActionList = Vec<ComputerAction>;
///The output of a computer tool call.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct ComputerCallOutputItemParam {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub acknowledged_safety_checks: ::std::option::Option<
        Vec<ComputerCallSafetyCheckParam>,
    >,
    ///The ID of the computer tool call that produced the output.
    pub call_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: ::std::option::Option<String>,
    pub output: ComputerScreenshotImage,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub status: ::std::option::Option<FunctionCallItemStatus>,
    ///The type of the computer tool call output. Always `computer_call_output`.
    #[serde(rename = "type")]
    pub type_value: String,
}
pub type ComputerCallOutputStatus = String;
///A pending safety check for the computer call.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct ComputerCallSafetyCheckParam {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub code: ::std::option::Option<String>,
    ///The ID of the pending safety check.
    pub id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub message: ::std::option::Option<String>,
}
pub type ComputerEnvironment = String;
///A screenshot of a computer.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct ComputerScreenshotContent {
    ///The detail level of the screenshot image to be sent to the model. One of `high`, `low`, `auto`, or `original`. Defaults to `auto`.
    pub detail: ImageDetail,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub file_id: ::std::option::Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub image_url: ::std::option::Option<String>,
    ///Specifies the event type. For a computer screenshot, this property is always set to `computer_screenshot`.
    #[serde(rename = "type")]
    pub type_value: String,
}
///A computer screenshot image used with the computer use tool.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct ComputerScreenshotImage {
    ///The identifier of an uploaded file that contains the screenshot.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub file_id: ::std::option::Option<String>,
    ///The URL of the screenshot image.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub image_url: ::std::option::Option<String>,
    ///Specifies the event type. For a computer screenshot, this property is always set to `computer_screenshot`.
    #[serde(rename = "type")]
    pub type_value: String,
}
///A tool that controls a virtual computer. Learn more about the [computer tool](https://platform.openai.com/docs/guides/tools-computer-use).
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct ComputerTool {
    ///The type of the computer tool. Always `computer`.
    #[serde(rename = "type")]
    pub type_value: String,
}
///A tool call to a computer use tool. See the [computer use guide](/docs/guides/tools-computer-use) for more information.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct ComputerToolCall {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub action: ::std::option::Option<ComputerAction>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub actions: ::std::option::Option<ComputerActionList>,
    ///An identifier used when responding to the tool call with output.
    pub call_id: String,
    ///The unique ID of the computer call.
    pub id: String,
    ///The pending safety checks for the computer call.
    pub pending_safety_checks: Vec<ComputerCallSafetyCheckParam>,
    ///The status of the item. One of `in_progress`, `completed`, or `incomplete`. Populated when items are returned via API.
    pub status: String,
    ///The type of the computer call. Always `computer_call`.
    #[serde(rename = "type")]
    pub type_value: String,
}
///The output of a computer tool call.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct ComputerToolCallOutput {
    ///The safety checks reported by the API that have been acknowledged by the developer.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub acknowledged_safety_checks: ::std::option::Option<
        Vec<ComputerCallSafetyCheckParam>,
    >,
    ///The ID of the computer tool call that produced the output.
    pub call_id: String,
    ///The ID of the computer tool call output.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: ::std::option::Option<String>,
    pub output: ComputerScreenshotImage,
    ///The status of the message input. One of `in_progress`, `completed`, or `incomplete`. Populated when input items are returned via API.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub status: ::std::option::Option<String>,
    ///The type of the computer tool call output. Always `computer_call_output`.
    #[serde(rename = "type")]
    pub type_value: String,
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct ComputerToolCallOutputResource {
    ///The safety checks reported by the API that have been acknowledged by the developer.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub acknowledged_safety_checks: ::std::option::Option<
        Vec<ComputerCallSafetyCheckParam>,
    >,
    ///The ID of the computer tool call that produced the output.
    pub call_id: String,
    ///The identifier of the actor that created the item.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub created_by: ::std::option::Option<String>,
    ///The ID of the computer tool call output.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: ::std::option::Option<String>,
    pub output: ComputerScreenshotImage,
    ///The status of the message input. One of `in_progress`, `completed`, or `incomplete`. Populated when input items are returned via API.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub status: ::std::option::Option<String>,
    ///The type of the computer tool call output. Always `computer_call_output`.
    #[serde(rename = "type")]
    pub type_value: String,
}
///A tool that controls a virtual computer. Learn more about the [computer tool](https://platform.openai.com/docs/guides/tools-computer-use).
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct ComputerUsePreviewTool {
    ///The height of the computer display.
    pub display_height: i32,
    ///The width of the computer display.
    pub display_width: i32,
    ///The type of computer environment to control.
    pub environment: ComputerEnvironment,
    ///The type of the computer use tool. Always `computer_use_preview`.
    #[serde(rename = "type")]
    pub type_value: String,
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct ContainerAutoParam {
    ///An optional list of uploaded files to make available to your code.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub file_ids: ::std::option::Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub memory_limit: ::std::option::Option<ContainerMemoryLimit>,
    ///Network access policy for the container.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub network_policy: ::std::option::Option<ContainerAutoParamNetworkPolicy>,
    ///An optional list of skills referenced by id or inline data.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub skills: ::std::option::Option<Vec<ContainerAutoParamSkill>>,
    ///Automatically creates a container for this request
    #[serde(rename = "type")]
    pub type_value: String,
}
///Network access policy for the container.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
#[serde(untagged)]
pub enum ContainerAutoParamNetworkPolicy {
    ContainerNetworkPolicyDisabledParam(ContainerNetworkPolicyDisabledParam),
    ContainerNetworkPolicyAllowlistParam(ContainerNetworkPolicyAllowlistParam),
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
#[serde(untagged)]
pub enum ContainerAutoParamSkill {
    SkillReferenceParam(SkillReferenceParam),
    InlineSkillParam(InlineSkillParam),
}
///A citation for a container file used to generate a model response.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct ContainerFileCitationBody {
    ///The ID of the container file.
    pub container_id: String,
    ///The index of the last character of the container file citation in the message.
    pub end_index: i32,
    ///The ID of the file.
    pub file_id: String,
    ///The filename of the container file cited.
    pub filename: String,
    ///The index of the first character of the container file citation in the message.
    pub start_index: i32,
    ///The type of the container file citation. Always `container_file_citation`.
    #[serde(rename = "type")]
    pub type_value: String,
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct ContainerFileListResource {
    ///A list of container files.
    pub data: Vec<ContainerFileResource>,
    ///The ID of the first file in the list.
    pub first_id: String,
    ///Whether there are more files available.
    pub has_more: bool,
    ///The ID of the last file in the list.
    pub last_id: String,
    ///The type of object returned, must be 'list'.
    pub object: String,
}
///The container file object
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct ContainerFileResource {
    ///Size of the file in bytes.
    pub bytes: i32,
    ///The container this file belongs to.
    pub container_id: String,
    ///Unix timestamp (in seconds) when the file was created.
    pub created_at: i64,
    ///Unique identifier for the file.
    pub id: String,
    ///The type of this object (`container.file`).
    pub object: String,
    ///Path of the file in the container.
    pub path: String,
    ///Source of the file (e.g., `user`, `assistant`).
    pub source: String,
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct ContainerListResource {
    ///A list of containers.
    pub data: Vec<ContainerResource>,
    ///The ID of the first container in the list.
    pub first_id: String,
    ///Whether there are more containers available.
    pub has_more: bool,
    ///The ID of the last container in the list.
    pub last_id: String,
    ///The type of object returned, must be 'list'.
    pub object: String,
}
pub type ContainerMemoryLimit = String;
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct ContainerNetworkPolicyAllowlistParam {
    ///A list of allowed domains when type is `allowlist`.
    pub allowed_domains: Vec<String>,
    ///Optional domain-scoped secrets for allowlisted domains.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub domain_secrets: ::std::option::Option<
        Vec<ContainerNetworkPolicyDomainSecretParam>,
    >,
    ///Allow outbound network access only to specified domains. Always `allowlist`.
    #[serde(rename = "type")]
    pub type_value: String,
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct ContainerNetworkPolicyDisabledParam {
    ///Disable outbound network access. Always `disabled`.
    #[serde(rename = "type")]
    pub type_value: String,
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct ContainerNetworkPolicyDomainSecretParam {
    ///The domain associated with the secret.
    pub domain: String,
    ///The name of the secret to inject for the domain.
    pub name: String,
    ///The secret value to inject for the domain.
    pub value: String,
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct ContainerReferenceParam {
    ///The ID of the referenced container.
    pub container_id: String,
    ///References a container created with the /v1/containers endpoint
    #[serde(rename = "type")]
    pub type_value: String,
}
///Represents a container created with /v1/containers.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct ContainerReferenceResource {
    pub container_id: String,
    ///The environment type. Always `container_reference`.
    #[serde(rename = "type")]
    pub type_value: String,
}
///The container object
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct ContainerResource {
    ///Unix timestamp (in seconds) when the container was created.
    pub created_at: i64,
    ///The container will expire after this time period. The anchor is the reference point for the expiration. The minutes is the number of minutes after the anchor before the container expires.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expires_after: ::std::option::Option<ContainerResourceExpiresAfter>,
    ///Unique identifier for the container.
    pub id: String,
    ///Unix timestamp (in seconds) when the container was last active.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_active_at: ::std::option::Option<i64>,
    ///The memory limit configured for the container.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub memory_limit: ::std::option::Option<String>,
    ///Name of the container.
    pub name: String,
    ///Network access policy for the container.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub network_policy: ::std::option::Option<ContainerResourceNetworkPolicy>,
    ///The type of this object.
    pub object: String,
    ///Status of the container (e.g., active, deleted).
    pub status: String,
}
///The container will expire after this time period. The anchor is the reference point for the expiration. The minutes is the number of minutes after the anchor before the container expires.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct ContainerResourceExpiresAfter {
    ///The reference point for the expiration.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub anchor: ::std::option::Option<String>,
    ///The number of minutes after the anchor before the container expires.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub minutes: ::std::option::Option<i32>,
}
///Network access policy for the container.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct ContainerResourceNetworkPolicy {
    ///Allowed outbound domains when `type` is `allowlist`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub allowed_domains: ::std::option::Option<Vec<String>>,
    ///The network policy mode.
    #[serde(rename = "type")]
    pub type_value: String,
}
///Multi-modal input and output contents.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
#[serde(untagged)]
pub enum Content {
    InputContent(InputContent),
    OutputContent(OutputContent),
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct ContextManagementParam {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub compact_threshold: ::std::option::Option<i32>,
    ///The context management entry type. Currently only 'compaction' is supported.
    #[serde(rename = "type")]
    pub type_value: String,
}
///The conversation that this response belonged to. Input items and output items from this response were automatically added to this conversation.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct Conversation2 {
    ///The unique ID of the conversation that this response was associated with.
    pub id: String,
}
///A single item within a conversation. The set of possible types are the same as the `output` type of a [Response object](/docs/api-reference/responses/object#responses/object-output).
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
#[serde(untagged)]
pub enum ConversationItem {
    Message(Message),
    FunctionToolCallResource(FunctionToolCallResource),
    FunctionToolCallOutputResource(FunctionToolCallOutputResource),
    FileSearchToolCall(FileSearchToolCall),
    WebSearchToolCall(WebSearchToolCall),
    ImageGenToolCall(ImageGenToolCall),
    ComputerToolCall(ComputerToolCall),
    ComputerToolCallOutputResource(ComputerToolCallOutputResource),
    ToolSearchCall(ToolSearchCall),
    ToolSearchOutput(ToolSearchOutput),
    ReasoningItem(ReasoningItem),
    CompactionBody(CompactionBody),
    CodeInterpreterToolCall(CodeInterpreterToolCall),
    LocalShellToolCall(LocalShellToolCall),
    LocalShellToolCallOutput(LocalShellToolCallOutput),
    FunctionShellCall(FunctionShellCall),
    FunctionShellCallOutput(FunctionShellCallOutput),
    ApplyPatchToolCall(ApplyPatchToolCall),
    ApplyPatchToolCallOutput(ApplyPatchToolCallOutput),
    McpListTools(McpListTools),
    McpApprovalRequest(McpApprovalRequest),
    McpApprovalResponseResource(McpApprovalResponseResource),
    McpToolCall(McpToolCall),
    CustomToolCall(CustomToolCall),
    CustomToolCallOutput(CustomToolCallOutput),
}
///A list of Conversation items.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct ConversationItemList {
    ///A list of conversation items.
    pub data: Vec<ConversationItem>,
    ///The ID of the first item in the list.
    pub first_id: String,
    ///Whether there are more items available.
    pub has_more: bool,
    ///The ID of the last item in the list.
    pub last_id: String,
    ///The type of object returned, must be `list`.
    pub object: String,
}
///The conversation that this response belongs to. Items from this conversation are prepended to `input_items` for this response request. Input items and output items from this response are automatically added to this conversation after this response completes.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
#[serde(untagged)]
pub enum ConversationParam {
    ConversationId(String),
    ConversationParam2(ConversationParam2),
}
///The conversation that this response belongs to.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct ConversationParam2 {
    ///The unique ID of the conversation.
    pub id: String,
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct ConversationResource {
    ///The time at which the conversation was created, measured in seconds since the Unix epoch.
    pub created_at: i64,
    ///The unique ID of the conversation.
    pub id: String,
    ///Set of 16 key-value pairs that can be attached to an object. This can be useful for storing additional information about the object in a structured format, and querying for objects via API or the dashboard. Keys are strings with a maximum length of 64 characters. Values are strings with a maximum length of 512 characters.
    pub metadata: OpenAiJsonValue,
    ///The object type, which is always `conversation`.
    pub object: String,
}
///An x/y coordinate pair, e.g. `{ x: 100, y: 200 }`.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct CoordParam {
    ///The x-coordinate.
    pub x: i32,
    ///The y-coordinate.
    pub y: i32,
}
///The aggregated costs details of the specific time bucket.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct CostsResult {
    ///The monetary value in its associated currency.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub amount: ::std::option::Option<CostsResultAmount>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub api_key_id: ::std::option::Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub line_item: ::std::option::Option<String>,
    pub object: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub project_id: ::std::option::Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub quantity: ::std::option::Option<f64>,
}
///The monetary value in its associated currency.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct CostsResultAmount {
    ///Lowercase ISO-4217 currency e.g. "usd"
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub currency: ::std::option::Option<String>,
    ///The numeric value of the cost.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub value: ::std::option::Option<f64>,
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct CreateAssistantRequest {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: ::std::option::Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub instructions: ::std::option::Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub metadata: ::std::option::Option<Metadata>,
    ///ID of the model to use. You can use the [List models](/docs/api-reference/models/list) API to see all of your available models, or see our [Model overview](/docs/models) for descriptions of them.
    pub model: CreateAssistantRequestModel,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: ::std::option::Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reasoning_effort: ::std::option::Option<ReasoningEffort>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub response_format: ::std::option::Option<AssistantsApiResponseFormatOption>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub temperature: ::std::option::Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tool_resources: ::std::option::Option<CreateAssistantRequestToolResources>,
    ///A list of tool enabled on the assistant. There can be a maximum of 128 tools per assistant. Tools can be of types `code_interpreter`, `file_search`, or `function`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tools: ::std::option::Option<Vec<CreateAssistantRequestTool>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub top_p: ::std::option::Option<f64>,
}
///ID of the model to use. You can use the [List models](/docs/api-reference/models/list) API to see all of your available models, or see our [Model overview](/docs/models) for descriptions of them.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
#[serde(untagged)]
pub enum CreateAssistantRequestModel {
    String(String),
    AssistantSupportedModels(AssistantSupportedModels),
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
#[serde(untagged)]
pub enum CreateAssistantRequestTool {
    AssistantToolsCode(AssistantToolsCode),
    AssistantToolsFileSearch(AssistantToolsFileSearch),
    AssistantToolsFunction(AssistantToolsFunction),
}
///A set of resources that are used by the assistant's tools. The resources are specific to the type of tool. For example, the `code_interpreter` tool requires a list of file IDs, while the `file_search` tool requires a list of vector store IDs.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct CreateAssistantRequestToolResources {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub code_interpreter: ::std::option::Option<
        CreateAssistantRequestToolResourcesCodeInterpreter,
    >,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub file_search: ::std::option::Option<
        CreateAssistantRequestToolResourcesFileSearch,
    >,
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct CreateAssistantRequestToolResourcesCodeInterpreter {
    ///A list of [file](/docs/api-reference/files) IDs made available to the `code_interpreter` tool. There can be a maximum of 20 files associated with the tool.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub file_ids: ::std::option::Option<Vec<String>>,
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
#[serde(untagged)]
pub enum CreateAssistantRequestToolResourcesFileSearch {
    Variant1(OpenAiJsonValue),
    Variant2(OpenAiJsonValue),
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct CreateBatchRequest {
    ///The time frame within which the batch should be processed. Currently only `24h` is supported.
    pub completion_window: String,
    ///The endpoint to be used for all requests in the batch. Currently `/v1/responses`, `/v1/chat/completions`, `/v1/embeddings`, `/v1/completions`, `/v1/moderations`, `/v1/images/generations`, `/v1/images/edits`, and `/v1/videos` are supported. Note that `/v1/embeddings` batches are also restricted to a maximum of 50,000 embedding inputs across all requests in the batch.
    pub endpoint: String,
    ///The ID of an uploaded file that contains requests for the new batch. See [upload file](/docs/api-reference/files/create) for how to upload a file. Your input file must be formatted as a [JSONL file](/docs/api-reference/batch/request-input), and must be uploaded with the purpose `batch`. The file can contain up to 50,000 requests, and can be up to 200 MB in size.
    pub input_file_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub metadata: ::std::option::Option<Metadata>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub output_expires_after: ::std::option::Option<BatchFileExpirationAfter>,
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct CreateChatCompletionRequest {
    ///Parameters for audio output. Required when audio output is requested with `modalities: ["audio"]`. [Learn more](/docs/guides/audio).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub audio: ::std::option::Option<CreateChatCompletionRequestAudio>,
    ///Number between -2.0 and 2.0. Positive values penalize new tokens based on their existing frequency in the text so far, decreasing the model's likelihood to repeat the same line verbatim.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub frequency_penalty: ::std::option::Option<f64>,
    ///Deprecated in favor of `tool_choice`. Controls which (if any) function is called by the model. `none` means the model will not call a function and instead generates a message. `auto` means the model can pick between generating a message or calling a function. Specifying a particular function via `{"name": "my_function"}` forces the model to call that function. `none` is the default when no functions are present. `auto` is the default if functions are present.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub function_call: ::std::option::Option<CreateChatCompletionRequestFunctionCall>,
    ///Deprecated in favor of `tools`. A list of functions the model may generate JSON inputs for.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub functions: ::std::option::Option<Vec<ChatCompletionFunctions>>,
    ///Modify the likelihood of specified tokens appearing in the completion. Accepts a JSON object that maps tokens (specified by their token ID in the tokenizer) to an associated bias value from -100 to 100. Mathematically, the bias is added to the logits generated by the model prior to sampling. The exact effect will vary per model, but values between -1 and 1 should decrease or increase likelihood of selection; values like -100 or 100 should result in a ban or exclusive selection of the relevant token.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub logit_bias: ::std::option::Option<OpenAiJsonValue>,
    ///Whether to return log probabilities of the output tokens or not. If true, returns the log probabilities of each output token returned in the `content` of `message`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub logprobs: ::std::option::Option<bool>,
    ///An upper bound for the number of tokens that can be generated for a completion, including visible output tokens and [reasoning tokens](/docs/guides/reasoning).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_completion_tokens: ::std::option::Option<i32>,
    ///The maximum number of [tokens](/tokenizer) that can be generated in the chat completion. This value can be used to control [costs](https://openai.com/api/pricing/) for text generated via API. This value is now deprecated in favor of `max_completion_tokens`, and is not compatible with [o-series models](/docs/guides/reasoning).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_tokens: ::std::option::Option<i32>,
    ///A list of messages comprising the conversation so far. Depending on the [model](/docs/models) you use, different message types (modalities) are supported, like [text](/docs/guides/text-generation), [images](/docs/guides/vision), and [audio](/docs/guides/audio).
    pub messages: Vec<ChatCompletionRequestMessage>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub metadata: ::std::option::Option<Metadata>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub modalities: ::std::option::Option<ResponseModalities>,
    ///Model ID used to generate the response, like `gpt-4o` or `o3`. OpenAI offers a wide range of models with different capabilities, performance characteristics, and price points. Refer to the [model guide](/docs/models) to browse and compare available models.
    pub model: ModelIdsShared,
    ///How many chat completion choices to generate for each input message. Note that you will be charged based on the number of generated tokens across all of the choices. Keep `n` as `1` to minimize costs.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub n: ::std::option::Option<i32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parallel_tool_calls: ::std::option::Option<ParallelToolCalls>,
    ///Configuration for a [Predicted Output](/docs/guides/predicted-outputs), which can greatly improve response times when large parts of the model response are known ahead of time. This is most common when you are regenerating a file with only minor changes to most of the content.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub prediction: ::std::option::Option<PredictionContent>,
    ///Number between -2.0 and 2.0. Positive values penalize new tokens based on whether they appear in the text so far, increasing the model's likelihood to talk about new topics.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub presence_penalty: ::std::option::Option<f64>,
    ///Used by OpenAI to cache responses for similar requests to optimize your cache hit rates. Replaces the `user` field. [Learn more](/docs/guides/prompt-caching).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub prompt_cache_key: ::std::option::Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub prompt_cache_retention: ::std::option::Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reasoning_effort: ::std::option::Option<ReasoningEffort>,
    ///An object specifying the format that the model must output. Setting to `{ "type": "json_schema", "json_schema": {...} }` enables Structured Outputs which ensures the model will match your supplied JSON schema. Learn more in the [Structured Outputs guide](/docs/guides/structured-outputs). Setting to `{ "type": "json_object" }` enables the older JSON mode, which ensures the message the model generates is valid JSON. Using `json_schema` is preferred for models that support it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub response_format: ::std::option::Option<
        CreateChatCompletionRequestResponseFormat,
    >,
    ///A stable identifier used to help detect users of your application that may be violating OpenAI's usage policies. The IDs should be a string that uniquely identifies each user, with a maximum length of 64 characters. We recommend hashing their username or email address, in order to avoid sending us any identifying information. [Learn more](/docs/guides/safety-best-practices#safety-identifiers).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub safety_identifier: ::std::option::Option<String>,
    ///This feature is in Beta. If specified, our system will make a best effort to sample deterministically, such that repeated requests with the same `seed` and parameters should return the same result. Determinism is not guaranteed, and you should refer to the `system_fingerprint` response parameter to monitor changes in the backend.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub seed: ::std::option::Option<i32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub service_tier: ::std::option::Option<ServiceTier>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stop: ::std::option::Option<StopConfiguration>,
    ///Whether or not to store the output of this chat completion request for use in our [model distillation](/docs/guides/distillation) or [evals](/docs/guides/evals) products. Supports text and image inputs. Note: image inputs over 8MB will be dropped.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub store: ::std::option::Option<bool>,
    ///If set to true, the model response data will be streamed to the client as it is generated using [server-sent events](https://developer.mozilla.org/en-US/docs/Web/API/Server-sent_events/Using_server-sent_events#Event_stream_format). See the [Streaming section below](/docs/api-reference/chat/streaming) for more information, along with the [streaming responses](/docs/guides/streaming-responses) guide for more information on how to handle the streaming events.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stream: ::std::option::Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stream_options: ::std::option::Option<ChatCompletionStreamOptions>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub temperature: ::std::option::Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tool_choice: ::std::option::Option<ChatCompletionToolChoiceOption>,
    ///A list of tools the model may call. You can provide either [custom tools](/docs/guides/function-calling#custom-tools) or [function tools](/docs/guides/function-calling).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tools: ::std::option::Option<Vec<CreateChatCompletionRequestTool>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub top_logprobs: ::std::option::Option<i32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub top_p: ::std::option::Option<f64>,
    ///This field is being replaced by `safety_identifier` and `prompt_cache_key`. Use `prompt_cache_key` instead to maintain caching optimizations. A stable identifier for your end-users. Used to boost cache hit rates by better bucketing similar requests and to help OpenAI detect and prevent abuse. [Learn more](/docs/guides/safety-best-practices#safety-identifiers).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub user: ::std::option::Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub verbosity: ::std::option::Option<Verbosity>,
    ///This tool searches the web for relevant results to use in a response. Learn more about the [web search tool](/docs/guides/tools-web-search?api-mode=chat).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub web_search_options: ::std::option::Option<
        CreateChatCompletionRequestWebSearchOptions,
    >,
}
///Parameters for audio output. Required when audio output is requested with `modalities: ["audio"]`. [Learn more](/docs/guides/audio).
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct CreateChatCompletionRequestAudio {
    ///Specifies the output audio format. Must be one of `wav`, `mp3`, `flac`, `opus`, or `pcm16`.
    pub format: String,
    ///The voice the model uses to respond. Supported built-in voices are `alloy`, `ash`, `ballad`, `coral`, `echo`, `fable`, `nova`, `onyx`, `sage`, `shimmer`, `marin`, and `cedar`. You may also provide a custom voice object with an `id`, for example `{ "id": "voice_1234" }`.
    pub voice: VoiceIdsOrCustomVoice,
}
///Deprecated in favor of `tool_choice`. Controls which (if any) function is called by the model. `none` means the model will not call a function and instead generates a message. `auto` means the model can pick between generating a message or calling a function. Specifying a particular function via `{"name": "my_function"}` forces the model to call that function. `none` is the default when no functions are present. `auto` is the default if functions are present.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
#[serde(untagged)]
pub enum CreateChatCompletionRequestFunctionCall {
    String(String),
    ChatCompletionFunctionCallOption(ChatCompletionFunctionCallOption),
}
///An object specifying the format that the model must output. Setting to `{ "type": "json_schema", "json_schema": {...} }` enables Structured Outputs which ensures the model will match your supplied JSON schema. Learn more in the [Structured Outputs guide](/docs/guides/structured-outputs). Setting to `{ "type": "json_object" }` enables the older JSON mode, which ensures the message the model generates is valid JSON. Using `json_schema` is preferred for models that support it.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
#[serde(untagged)]
pub enum CreateChatCompletionRequestResponseFormat {
    ResponseFormatText(ResponseFormatText),
    ResponseFormatJsonSchema(ResponseFormatJsonSchema),
    ResponseFormatJsonObject(ResponseFormatJsonObject),
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
#[serde(untagged)]
pub enum CreateChatCompletionRequestTool {
    ChatCompletionTool(ChatCompletionTool),
    CustomToolChatCompletions(CustomToolChatCompletions),
}
///This tool searches the web for relevant results to use in a response. Learn more about the [web search tool](/docs/guides/tools-web-search?api-mode=chat).
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct CreateChatCompletionRequestWebSearchOptions {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub search_context_size: ::std::option::Option<WebSearchContextSize>,
    ///Approximate location parameters for the search.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub user_location: ::std::option::Option<
        CreateChatCompletionRequestWebSearchOptionsUserLocation,
    >,
}
///Approximate location parameters for the search.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct CreateChatCompletionRequestWebSearchOptionsUserLocation {
    pub approximate: WebSearchLocation,
    ///The type of location approximation. Always `approximate`.
    #[serde(rename = "type")]
    pub type_value: String,
}
///Represents a chat completion response returned by model, based on the provided input.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct CreateChatCompletionResponse {
    ///A list of chat completion choices. Can be more than one if `n` is greater than 1.
    pub choices: Vec<CreateChatCompletionResponseChoice>,
    ///The Unix timestamp (in seconds) of when the chat completion was created.
    pub created: i64,
    ///A unique identifier for the chat completion.
    pub id: String,
    ///The model used for the chat completion.
    pub model: String,
    ///The object type, which is always `chat.completion`.
    pub object: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub service_tier: ::std::option::Option<ServiceTier>,
    ///This fingerprint represents the backend configuration that the model runs with. Can be used in conjunction with the `seed` request parameter to understand when backend changes have been made that might impact determinism.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub system_fingerprint: ::std::option::Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub usage: ::std::option::Option<CompletionUsage>,
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct CreateChatCompletionResponseChoice {
    ///The reason the model stopped generating tokens. This will be `stop` if the model hit a natural stop point or a provided stop sequence, `length` if the maximum number of tokens specified in the request was reached, `content_filter` if content was omitted due to a flag from our content filters, `tool_calls` if the model called a tool, or `function_call` (deprecated) if the model called a function.
    pub finish_reason: String,
    ///The index of the choice in the list of choices.
    pub index: i32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub logprobs: ::std::option::Option<CreateChatCompletionResponseChoiceLogprobs>,
    pub message: ChatCompletionResponseMessage,
}
///Log probability information for the choice.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct CreateChatCompletionResponseChoiceLogprobs {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub content: ::std::option::Option<Vec<ChatCompletionTokenLogprob>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub refusal: ::std::option::Option<Vec<ChatCompletionTokenLogprob>>,
}
///Represents a streamed chunk of a chat completion response returned by the model, based on the provided input. [Learn more](/docs/guides/streaming-responses).
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct CreateChatCompletionStreamResponse {
    ///A list of chat completion choices. Can contain more than one elements if `n` is greater than 1. Can also be empty for the last chunk if you set `stream_options: {"include_usage": true}`.
    pub choices: Vec<CreateChatCompletionStreamResponseChoice>,
    ///The Unix timestamp (in seconds) of when the chat completion was created. Each chunk has the same timestamp.
    pub created: i64,
    ///A unique identifier for the chat completion. Each chunk has the same ID.
    pub id: String,
    ///The model to generate the completion.
    pub model: String,
    ///The object type, which is always `chat.completion.chunk`.
    pub object: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub service_tier: ::std::option::Option<ServiceTier>,
    ///This fingerprint represents the backend configuration that the model runs with. Can be used in conjunction with the `seed` request parameter to understand when backend changes have been made that might impact determinism.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub system_fingerprint: ::std::option::Option<String>,
    ///An optional field that will only be present when you set `stream_options: {"include_usage": true}` in your request. When present, it contains a null value **except for the last chunk** which contains the token usage statistics for the entire request. **NOTE:** If the stream is interrupted or cancelled, you may not receive the final usage chunk which contains the total token usage for the request.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub usage: ::std::option::Option<CompletionUsage>,
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct CreateChatCompletionStreamResponseChoice {
    pub delta: ChatCompletionStreamResponseDelta,
    ///The reason the model stopped generating tokens. This will be `stop` if the model hit a natural stop point or a provided stop sequence, `length` if the maximum number of tokens specified in the request was reached, `content_filter` if content was omitted due to a flag from our content filters, `tool_calls` if the model called a tool, or `function_call` (deprecated) if the model called a function.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub finish_reason: ::std::option::Option<String>,
    ///The index of the choice in the list of choices.
    pub index: i32,
    ///Log probability information for the choice.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub logprobs: ::std::option::Option<
        CreateChatCompletionStreamResponseChoiceLogprobs,
    >,
}
///Log probability information for the choice.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct CreateChatCompletionStreamResponseChoiceLogprobs {
    ///A list of message content tokens with log probability information.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub content: ::std::option::Option<Vec<ChatCompletionTokenLogprob>>,
    ///A list of message refusal tokens with log probability information.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub refusal: ::std::option::Option<Vec<ChatCompletionTokenLogprob>>,
}
///Parameters for provisioning a new ChatKit session.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct CreateChatSessionBody {
    ///Optional overrides for ChatKit runtime configuration features
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub chatkit_configuration: ::std::option::Option<ChatkitConfigurationParam>,
    ///Optional override for session expiration timing in seconds from creation. Defaults to 10 minutes.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expires_after: ::std::option::Option<ExpiresAfterParam>,
    ///Optional override for per-minute request limits. When omitted, defaults to 10.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rate_limits: ::std::option::Option<RateLimitsParam>,
    ///A free-form string that identifies your end user; ensures this Session can access other objects that have the same `user` scope.
    pub user: String,
    ///Workflow that powers the session.
    pub workflow: WorkflowParam,
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct CreateCompletionRequest {
    ///Generates `best_of` completions server-side and returns the "best" (the one with the highest log probability per token). Results cannot be streamed. When used with `n`, `best_of` controls the number of candidate completions and `n` specifies how many to return – `best_of` must be greater than `n`. **Note:** Because this parameter generates many completions, it can quickly consume your token quota. Use carefully and ensure that you have reasonable settings for `max_tokens` and `stop`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub best_of: ::std::option::Option<i32>,
    ///Echo back the prompt in addition to the completion
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub echo: ::std::option::Option<bool>,
    ///Number between -2.0 and 2.0. Positive values penalize new tokens based on their existing frequency in the text so far, decreasing the model's likelihood to repeat the same line verbatim. [See more information about frequency and presence penalties.](/docs/guides/text-generation)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub frequency_penalty: ::std::option::Option<f64>,
    ///Modify the likelihood of specified tokens appearing in the completion. Accepts a JSON object that maps tokens (specified by their token ID in the GPT tokenizer) to an associated bias value from -100 to 100. You can use this [tokenizer tool](/tokenizer?view=bpe) to convert text to token IDs. Mathematically, the bias is added to the logits generated by the model prior to sampling. The exact effect will vary per model, but values between -1 and 1 should decrease or increase likelihood of selection; values like -100 or 100 should result in a ban or exclusive selection of the relevant token. As an example, you can pass `{"50256": -100}` to prevent the <|endoftext|> token from being generated.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub logit_bias: ::std::option::Option<OpenAiJsonValue>,
    ///Include the log probabilities on the `logprobs` most likely output tokens, as well the chosen tokens. For example, if `logprobs` is 5, the API will return a list of the 5 most likely tokens. The API will always return the `logprob` of the sampled token, so there may be up to `logprobs+1` elements in the response. The maximum value for `logprobs` is 5.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub logprobs: ::std::option::Option<i32>,
    ///The maximum number of [tokens](/tokenizer) that can be generated in the completion. The token count of your prompt plus `max_tokens` cannot exceed the model's context length. [Example Python code](https://cookbook.openai.com/examples/how_to_count_tokens_with_tiktoken) for counting tokens.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_tokens: ::std::option::Option<i32>,
    ///ID of the model to use. You can use the [List models](/docs/api-reference/models/list) API to see all of your available models, or see our [Model overview](/docs/models) for descriptions of them.
    pub model: String,
    ///How many completions to generate for each prompt. **Note:** Because this parameter generates many completions, it can quickly consume your token quota. Use carefully and ensure that you have reasonable settings for `max_tokens` and `stop`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub n: ::std::option::Option<i32>,
    ///Number between -2.0 and 2.0. Positive values penalize new tokens based on whether they appear in the text so far, increasing the model's likelihood to talk about new topics. [See more information about frequency and presence penalties.](/docs/guides/text-generation)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub presence_penalty: ::std::option::Option<f64>,
    ///The prompt(s) to generate completions for, encoded as a string, array of strings, array of tokens, or array of token arrays. Note that <|endoftext|> is the document separator that the model sees during training, so if a prompt is not specified the model will generate as if from the beginning of a new document.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub prompt: ::std::option::Option<CreateCompletionRequestPrompt>,
    ///If specified, our system will make a best effort to sample deterministically, such that repeated requests with the same `seed` and parameters should return the same result. Determinism is not guaranteed, and you should refer to the `system_fingerprint` response parameter to monitor changes in the backend.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub seed: ::std::option::Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stop: ::std::option::Option<StopConfiguration>,
    ///Whether to stream back partial progress. If set, tokens will be sent as data-only [server-sent events](https://developer.mozilla.org/en-US/docs/Web/API/Server-sent_events/Using_server-sent_events#Event_stream_format) as they become available, with the stream terminated by a `data: [DONE]` message. [Example Python code](https://cookbook.openai.com/examples/how_to_stream_completions).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stream: ::std::option::Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stream_options: ::std::option::Option<ChatCompletionStreamOptions>,
    ///The suffix that comes after a completion of inserted text. This parameter is only supported for `gpt-3.5-turbo-instruct`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub suffix: ::std::option::Option<String>,
    ///What sampling temperature to use, between 0 and 2. Higher values like 0.8 will make the output more random, while lower values like 0.2 will make it more focused and deterministic. We generally recommend altering this or `top_p` but not both.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub temperature: ::std::option::Option<f64>,
    ///An alternative to sampling with temperature, called nucleus sampling, where the model considers the results of the tokens with top_p probability mass. So 0.1 means only the tokens comprising the top 10% probability mass are considered. We generally recommend altering this or `temperature` but not both.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub top_p: ::std::option::Option<f64>,
    ///A unique identifier representing your end-user, which can help OpenAI to monitor and detect abuse. [Learn more](/docs/guides/safety-best-practices#end-user-ids).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub user: ::std::option::Option<String>,
}
///The prompt(s) to generate completions for, encoded as a string, array of strings, array of tokens, or array of token arrays. Note that <|endoftext|> is the document separator that the model sees during training, so if a prompt is not specified the model will generate as if from the beginning of a new document.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
#[serde(untagged)]
pub enum CreateCompletionRequestPrompt {
    String(String),
    Array(Vec<String>),
    Array3(Vec<i32>),
    Array4(Vec<Vec<i32>>),
}
///Represents a completion response from the API. Note: both the streamed and non-streamed response objects share the same shape (unlike the chat endpoint).
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct CreateCompletionResponse {
    ///The list of completion choices the model generated for the input prompt.
    pub choices: Vec<CreateCompletionResponseChoice>,
    ///The Unix timestamp (in seconds) of when the completion was created.
    pub created: i64,
    ///A unique identifier for the completion.
    pub id: String,
    ///The model used for completion.
    pub model: String,
    ///The object type, which is always "text_completion"
    pub object: String,
    ///This fingerprint represents the backend configuration that the model runs with. Can be used in conjunction with the `seed` request parameter to understand when backend changes have been made that might impact determinism.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub system_fingerprint: ::std::option::Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub usage: ::std::option::Option<CompletionUsage>,
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct CreateCompletionResponseChoice {
    ///The reason the model stopped generating tokens. This will be `stop` if the model hit a natural stop point or a provided stop sequence, `length` if the maximum number of tokens specified in the request was reached, or `content_filter` if content was omitted due to a flag from our content filters.
    pub finish_reason: String,
    pub index: i32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub logprobs: ::std::option::Option<CreateCompletionResponseChoiceLogprobs>,
    pub text: String,
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct CreateCompletionResponseChoiceLogprobs {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub text_offset: ::std::option::Option<Vec<i32>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub token_logprobs: ::std::option::Option<Vec<f64>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tokens: ::std::option::Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub top_logprobs: ::std::option::Option<Vec<OpenAiJsonValue>>,
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct CreateContainerBody {
    ///Container expiration time in seconds relative to the 'anchor' time.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expires_after: ::std::option::Option<CreateContainerBodyExpiresAfter>,
    ///IDs of files to copy to the container.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub file_ids: ::std::option::Option<Vec<String>>,
    ///Optional memory limit for the container. Defaults to "1g".
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub memory_limit: ::std::option::Option<String>,
    ///Name of the container to create.
    pub name: String,
    ///Network access policy for the container.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub network_policy: ::std::option::Option<CreateContainerBodyNetworkPolicy>,
    ///An optional list of skills referenced by id or inline data.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub skills: ::std::option::Option<Vec<CreateContainerBodySkill>>,
}
///Container expiration time in seconds relative to the 'anchor' time.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct CreateContainerBodyExpiresAfter {
    ///Time anchor for the expiration time. Currently only 'last_active_at' is supported.
    pub anchor: String,
    pub minutes: i32,
}
///Network access policy for the container.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
#[serde(untagged)]
pub enum CreateContainerBodyNetworkPolicy {
    ContainerNetworkPolicyDisabledParam(ContainerNetworkPolicyDisabledParam),
    ContainerNetworkPolicyAllowlistParam(ContainerNetworkPolicyAllowlistParam),
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
#[serde(untagged)]
pub enum CreateContainerBodySkill {
    SkillReferenceParam(SkillReferenceParam),
    InlineSkillParam(InlineSkillParam),
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct CreateContainerFileBody {
    ///The File object (not file name) to be uploaded.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub file: ::std::option::Option<OpenAiBinaryBody>,
    ///Name of the file to create.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub file_id: ::std::option::Option<String>,
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct CreateConversationBody {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub items: ::std::option::Option<Vec<InputItem>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub metadata: ::std::option::Option<Metadata>,
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct CreateConversationItemsRequest {
    ///The items to add to the conversation. You may add up to 20 items at a time.
    pub items: Vec<InputItem>,
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct CreateEmbeddingRequest {
    ///The number of dimensions the resulting output embeddings should have. Only supported in `text-embedding-3` and later models.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dimensions: ::std::option::Option<i32>,
    ///The format to return the embeddings in. Can be either `float` or [`base64`](https://pypi.org/project/pybase64/).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub encoding_format: ::std::option::Option<String>,
    ///Input text to embed, encoded as a string or array of tokens. To embed multiple inputs in a single request, pass an array of strings or array of token arrays. The input must not exceed the max input tokens for the model (8192 tokens for all embedding models), cannot be an empty string, and any array must be 2048 dimensions or less. [Example Python code](https://cookbook.openai.com/examples/how_to_count_tokens_with_tiktoken) for counting tokens. In addition to the per-input token limit, all embedding models enforce a maximum of 300,000 tokens summed across all inputs in a single request.
    pub input: CreateEmbeddingRequestInput,
    ///ID of the model to use. You can use the [List models](/docs/api-reference/models/list) API to see all of your available models, or see our [Model overview](/docs/models) for descriptions of them.
    pub model: String,
    ///A unique identifier representing your end-user, which can help OpenAI to monitor and detect abuse. [Learn more](/docs/guides/safety-best-practices#end-user-ids).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub user: ::std::option::Option<String>,
}
///Input text to embed, encoded as a string or array of tokens. To embed multiple inputs in a single request, pass an array of strings or array of token arrays. The input must not exceed the max input tokens for the model (8192 tokens for all embedding models), cannot be an empty string, and any array must be 2048 dimensions or less. [Example Python code](https://cookbook.openai.com/examples/how_to_count_tokens_with_tiktoken) for counting tokens. In addition to the per-input token limit, all embedding models enforce a maximum of 300,000 tokens summed across all inputs in a single request.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
#[serde(untagged)]
pub enum CreateEmbeddingRequestInput {
    String(String),
    Array(Vec<String>),
    Array3(Vec<i32>),
    Array4(Vec<Vec<i32>>),
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct CreateEmbeddingResponse {
    ///The list of embeddings generated by the model.
    pub data: Vec<Embedding>,
    ///The name of the model used to generate the embedding.
    pub model: String,
    ///The object type, which is always "list".
    pub object: String,
    ///The usage information for the request.
    pub usage: CreateEmbeddingResponseUsage,
}
///The usage information for the request.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct CreateEmbeddingResponseUsage {
    ///The number of tokens used by the prompt.
    pub prompt_tokens: i32,
    ///The total number of tokens used by the request.
    pub total_tokens: i32,
}
///A CompletionsRunDataSource object describing a model sampling configuration.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct CreateEvalCompletionsRunDataSource {
    ///Used when sampling from a model. Dictates the structure of the messages passed into the model. Can either be a reference to a prebuilt trajectory (ie, `item.input_trajectory`), or a template with variable references to the `item` namespace.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub input_messages: ::std::option::Option<
        CreateEvalCompletionsRunDataSourceInputMessages3,
    >,
    ///The name of the model to use for generating completions (e.g. "o3-mini").
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model: ::std::option::Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sampling_params: ::std::option::Option<
        CreateEvalCompletionsRunDataSourceSamplingParams,
    >,
    ///Determines what populates the `item` namespace in this run's data source.
    pub source: CreateEvalCompletionsRunDataSourceSource,
    ///The type of run data source. Always `completions`.
    #[serde(rename = "type")]
    pub type_value: String,
}
///TemplateInputMessages
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct CreateEvalCompletionsRunDataSourceInputMessages {
    ///A list of chat messages forming the prompt or context. May include variable references to the `item` namespace, ie {{item.name}}.
    pub template: Vec<CreateEvalCompletionsRunDataSourceInputMessagesTemplateItem>,
    ///The type of input messages. Always `template`.
    #[serde(rename = "type")]
    pub type_value: String,
}
///ItemReferenceInputMessages
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct CreateEvalCompletionsRunDataSourceInputMessages2 {
    ///A reference to a variable in the `item` namespace. Ie, "item.input_trajectory"
    pub item_reference: String,
    ///The type of input messages. Always `item_reference`.
    #[serde(rename = "type")]
    pub type_value: String,
}
///Used when sampling from a model. Dictates the structure of the messages passed into the model. Can either be a reference to a prebuilt trajectory (ie, `item.input_trajectory`), or a template with variable references to the `item` namespace.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
#[serde(untagged)]
pub enum CreateEvalCompletionsRunDataSourceInputMessages3 {
    TemplateInputMessages(
        CreateEvalCompletionsRunDataSourceInputMessages3TemplateInputMessages,
    ),
    ItemReferenceInputMessages(
        CreateEvalCompletionsRunDataSourceInputMessages3ItemReferenceInputMessages,
    ),
}
///ItemReferenceInputMessages
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct CreateEvalCompletionsRunDataSourceInputMessages3ItemReferenceInputMessages {
    ///A reference to a variable in the `item` namespace. Ie, "item.input_trajectory"
    pub item_reference: String,
    ///The type of input messages. Always `item_reference`.
    #[serde(rename = "type")]
    pub type_value: String,
}
///TemplateInputMessages
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct CreateEvalCompletionsRunDataSourceInputMessages3TemplateInputMessages {
    ///A list of chat messages forming the prompt or context. May include variable references to the `item` namespace, ie {{item.name}}.
    pub template: Vec<
        CreateEvalCompletionsRunDataSourceInputMessages3TemplateInputMessagesTemplateItem,
    >,
    ///The type of input messages. Always `template`.
    #[serde(rename = "type")]
    pub type_value: String,
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
#[serde(untagged)]
pub enum CreateEvalCompletionsRunDataSourceInputMessages3TemplateInputMessagesTemplateItem {
    EasyInputMessage(EasyInputMessage),
    EvalItem(EvalItem),
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
#[serde(untagged)]
pub enum CreateEvalCompletionsRunDataSourceInputMessagesTemplateItem {
    EasyInputMessage(EasyInputMessage),
    EvalItem(EvalItem),
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct CreateEvalCompletionsRunDataSourceSamplingParams {
    ///The maximum number of tokens in the generated output.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_completion_tokens: ::std::option::Option<i32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reasoning_effort: ::std::option::Option<ReasoningEffort>,
    ///An object specifying the format that the model must output. Setting to `{ "type": "json_schema", "json_schema": {...} }` enables Structured Outputs which ensures the model will match your supplied JSON schema. Learn more in the [Structured Outputs guide](/docs/guides/structured-outputs). Setting to `{ "type": "json_object" }` enables the older JSON mode, which ensures the message the model generates is valid JSON. Using `json_schema` is preferred for models that support it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub response_format: ::std::option::Option<
        CreateEvalCompletionsRunDataSourceSamplingParamsResponseFormat,
    >,
    ///A seed value to initialize the randomness, during sampling.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub seed: ::std::option::Option<i32>,
    ///A higher temperature increases randomness in the outputs.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub temperature: ::std::option::Option<f64>,
    ///A list of tools the model may call. Currently, only functions are supported as a tool. Use this to provide a list of functions the model may generate JSON inputs for. A max of 128 functions are supported.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tools: ::std::option::Option<Vec<ChatCompletionTool>>,
    ///An alternative to temperature for nucleus sampling; 1.0 includes all tokens.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub top_p: ::std::option::Option<f64>,
}
///An object specifying the format that the model must output. Setting to `{ "type": "json_schema", "json_schema": {...} }` enables Structured Outputs which ensures the model will match your supplied JSON schema. Learn more in the [Structured Outputs guide](/docs/guides/structured-outputs). Setting to `{ "type": "json_object" }` enables the older JSON mode, which ensures the message the model generates is valid JSON. Using `json_schema` is preferred for models that support it.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
#[serde(untagged)]
pub enum CreateEvalCompletionsRunDataSourceSamplingParamsResponseFormat {
    ResponseFormatText(ResponseFormatText),
    ResponseFormatJsonSchema(ResponseFormatJsonSchema),
    ResponseFormatJsonObject(ResponseFormatJsonObject),
}
///Determines what populates the `item` namespace in this run's data source.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
#[serde(untagged)]
pub enum CreateEvalCompletionsRunDataSourceSource {
    EvalJsonlFileContentSource(EvalJsonlFileContentSource),
    EvalJsonlFileIdSource(EvalJsonlFileIdSource),
    EvalStoredCompletionsSource(EvalStoredCompletionsSource),
}
///A CustomDataSourceConfig object that defines the schema for the data source used for the evaluation runs. This schema is used to define the shape of the data that will be: - Used to define your testing criteria and - What data is required when creating a run
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct CreateEvalCustomDataSourceConfig {
    ///Whether the eval should expect you to populate the sample namespace (ie, by generating responses off of your data source)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub include_sample_schema: ::std::option::Option<bool>,
    ///The json schema for each row in the data source.
    pub item_schema: OpenAiJsonValue,
    ///The type of data source. Always `custom`.
    #[serde(rename = "type")]
    pub type_value: String,
}
///A chat message that makes up the prompt or context. May include variable references to the `item` namespace, ie {{item.name}}.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct CreateEvalItem {
    #[serde(flatten)]
    pub value: OpenAiJsonObject,
}
///A JsonlRunDataSource object with that specifies a JSONL file that matches the eval
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct CreateEvalJsonlRunDataSource {
    ///Determines what populates the `item` namespace in the data source.
    pub source: CreateEvalJsonlRunDataSourceSource,
    ///The type of data source. Always `jsonl`.
    #[serde(rename = "type")]
    pub type_value: String,
}
///Determines what populates the `item` namespace in the data source.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
#[serde(untagged)]
pub enum CreateEvalJsonlRunDataSourceSource {
    EvalJsonlFileContentSource(EvalJsonlFileContentSource),
    EvalJsonlFileIdSource(EvalJsonlFileIdSource),
}
///A LabelModelGrader object which uses a model to assign labels to each item in the evaluation.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct CreateEvalLabelModelGrader {
    ///A list of chat messages forming the prompt or context. May include variable references to the `item` namespace, ie {{item.name}}.
    pub input: Vec<CreateEvalItem>,
    ///The labels to classify to each item in the evaluation.
    pub labels: Vec<String>,
    ///The model to use for the evaluation. Must support structured outputs.
    pub model: String,
    ///The name of the grader.
    pub name: String,
    ///The labels that indicate a passing result. Must be a subset of labels.
    pub passing_labels: Vec<String>,
    ///The object type, which is always `label_model`.
    #[serde(rename = "type")]
    pub type_value: String,
}
///A data source config which specifies the metadata property of your logs query. This is usually metadata like `usecase=chatbot` or `prompt-version=v2`, etc.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct CreateEvalLogsDataSourceConfig {
    ///Metadata filters for the logs data source.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub metadata: ::std::option::Option<OpenAiJsonValue>,
    ///The type of data source. Always `logs`.
    #[serde(rename = "type")]
    pub type_value: String,
}
///CreateEvalRequest
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct CreateEvalRequest {
    ///The configuration for the data source used for the evaluation runs. Dictates the schema of the data used in the evaluation.
    pub data_source_config: CreateEvalRequestDataSourceConfig,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub metadata: ::std::option::Option<Metadata>,
    ///The name of the evaluation.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: ::std::option::Option<String>,
    ///A list of graders for all eval runs in this group. Graders can reference variables in the data source using double curly braces notation, like `{{item.variable_name}}`. To reference the model's output, use the `sample` namespace (ie, `{{sample.output_text}}`).
    pub testing_criteria: Vec<CreateEvalRequestTestingCriteriaItem>,
}
///The configuration for the data source used for the evaluation runs. Dictates the schema of the data used in the evaluation.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
#[serde(untagged)]
pub enum CreateEvalRequestDataSourceConfig {
    CreateEvalCustomDataSourceConfig(CreateEvalCustomDataSourceConfig),
    CreateEvalLogsDataSourceConfig(CreateEvalLogsDataSourceConfig),
    CreateEvalStoredCompletionsDataSourceConfig(
        CreateEvalStoredCompletionsDataSourceConfig,
    ),
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
#[serde(untagged)]
pub enum CreateEvalRequestTestingCriteriaItem {
    CreateEvalLabelModelGrader(CreateEvalLabelModelGrader),
    EvalGraderStringCheck(EvalGraderStringCheck),
    EvalGraderTextSimilarity(EvalGraderTextSimilarity),
    EvalGraderPython(EvalGraderPython),
    EvalGraderScoreModel(EvalGraderScoreModel),
}
///A ResponsesRunDataSource object describing a model sampling configuration.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct CreateEvalResponsesRunDataSource {
    ///Used when sampling from a model. Dictates the structure of the messages passed into the model. Can either be a reference to a prebuilt trajectory (ie, `item.input_trajectory`), or a template with variable references to the `item` namespace.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub input_messages: ::std::option::Option<
        CreateEvalResponsesRunDataSourceInputMessages3,
    >,
    ///The name of the model to use for generating completions (e.g. "o3-mini").
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model: ::std::option::Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sampling_params: ::std::option::Option<
        CreateEvalResponsesRunDataSourceSamplingParams,
    >,
    ///Determines what populates the `item` namespace in this run's data source.
    pub source: CreateEvalResponsesRunDataSourceSource,
    ///The type of run data source. Always `responses`.
    #[serde(rename = "type")]
    pub type_value: String,
}
///InputMessagesTemplate
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct CreateEvalResponsesRunDataSourceInputMessages {
    ///A list of chat messages forming the prompt or context. May include variable references to the `item` namespace, ie {{item.name}}.
    pub template: Vec<CreateEvalResponsesRunDataSourceInputMessagesTemplateItem2>,
    ///The type of input messages. Always `template`.
    #[serde(rename = "type")]
    pub type_value: String,
}
///InputMessagesItemReference
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct CreateEvalResponsesRunDataSourceInputMessages2 {
    ///A reference to a variable in the `item` namespace. Ie, "item.name"
    pub item_reference: String,
    ///The type of input messages. Always `item_reference`.
    #[serde(rename = "type")]
    pub type_value: String,
}
///Used when sampling from a model. Dictates the structure of the messages passed into the model. Can either be a reference to a prebuilt trajectory (ie, `item.input_trajectory`), or a template with variable references to the `item` namespace.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
#[serde(untagged)]
pub enum CreateEvalResponsesRunDataSourceInputMessages3 {
    InputMessagesTemplate(
        CreateEvalResponsesRunDataSourceInputMessages3InputMessagesTemplate,
    ),
    InputMessagesItemReference(
        CreateEvalResponsesRunDataSourceInputMessages3InputMessagesItemReference,
    ),
}
///InputMessagesItemReference
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct CreateEvalResponsesRunDataSourceInputMessages3InputMessagesItemReference {
    ///A reference to a variable in the `item` namespace. Ie, "item.name"
    pub item_reference: String,
    ///The type of input messages. Always `item_reference`.
    #[serde(rename = "type")]
    pub type_value: String,
}
///InputMessagesTemplate
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct CreateEvalResponsesRunDataSourceInputMessages3InputMessagesTemplate {
    ///A list of chat messages forming the prompt or context. May include variable references to the `item` namespace, ie {{item.name}}.
    pub template: Vec<
        CreateEvalResponsesRunDataSourceInputMessages3InputMessagesTemplateTemplateItem2,
    >,
    ///The type of input messages. Always `template`.
    #[serde(rename = "type")]
    pub type_value: String,
}
///ChatMessage
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct CreateEvalResponsesRunDataSourceInputMessages3InputMessagesTemplateTemplateItem {
    ///The content of the message.
    pub content: String,
    ///The role of the message (e.g. "system", "assistant", "user").
    pub role: String,
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
#[serde(untagged)]
pub enum CreateEvalResponsesRunDataSourceInputMessages3InputMessagesTemplateTemplateItem2 {
    ChatMessage(
        CreateEvalResponsesRunDataSourceInputMessages3InputMessagesTemplateTemplateItem2ChatMessage,
    ),
    EvalItem(EvalItem),
}
///ChatMessage
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct CreateEvalResponsesRunDataSourceInputMessages3InputMessagesTemplateTemplateItem2ChatMessage {
    ///The content of the message.
    pub content: String,
    ///The role of the message (e.g. "system", "assistant", "user").
    pub role: String,
}
///ChatMessage
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct CreateEvalResponsesRunDataSourceInputMessagesTemplateItem {
    ///The content of the message.
    pub content: String,
    ///The role of the message (e.g. "system", "assistant", "user").
    pub role: String,
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
#[serde(untagged)]
pub enum CreateEvalResponsesRunDataSourceInputMessagesTemplateItem2 {
    ChatMessage(CreateEvalResponsesRunDataSourceInputMessagesTemplateItem2ChatMessage),
    EvalItem(EvalItem),
}
///ChatMessage
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct CreateEvalResponsesRunDataSourceInputMessagesTemplateItem2ChatMessage {
    ///The content of the message.
    pub content: String,
    ///The role of the message (e.g. "system", "assistant", "user").
    pub role: String,
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct CreateEvalResponsesRunDataSourceSamplingParams {
    ///The maximum number of tokens in the generated output.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_completion_tokens: ::std::option::Option<i32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reasoning_effort: ::std::option::Option<ReasoningEffort>,
    ///A seed value to initialize the randomness, during sampling.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub seed: ::std::option::Option<i32>,
    ///A higher temperature increases randomness in the outputs.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub temperature: ::std::option::Option<f64>,
    ///Configuration options for a text response from the model. Can be plain text or structured JSON data. Learn more: - [Text inputs and outputs](/docs/guides/text) - [Structured Outputs](/docs/guides/structured-outputs)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub text: ::std::option::Option<CreateEvalResponsesRunDataSourceSamplingParamsText>,
    ///An array of tools the model may call while generating a response. You can specify which tool to use by setting the `tool_choice` parameter. The two categories of tools you can provide the model are: - **Built-in tools**: Tools that are provided by OpenAI that extend the model's capabilities, like [web search](/docs/guides/tools-web-search) or [file search](/docs/guides/tools-file-search). Learn more about [built-in tools](/docs/guides/tools). - **Function calls (custom tools)**: Functions that are defined by you, enabling the model to call your own code. Learn more about [function calling](/docs/guides/function-calling).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tools: ::std::option::Option<Vec<Tool>>,
    ///An alternative to temperature for nucleus sampling; 1.0 includes all tokens.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub top_p: ::std::option::Option<f64>,
}
///Configuration options for a text response from the model. Can be plain text or structured JSON data. Learn more: - [Text inputs and outputs](/docs/guides/text) - [Structured Outputs](/docs/guides/structured-outputs)
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct CreateEvalResponsesRunDataSourceSamplingParamsText {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub format: ::std::option::Option<TextResponseFormatConfiguration>,
}
///Determines what populates the `item` namespace in this run's data source.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
#[serde(untagged)]
pub enum CreateEvalResponsesRunDataSourceSource {
    EvalJsonlFileContentSource(EvalJsonlFileContentSource),
    EvalJsonlFileIdSource(EvalJsonlFileIdSource),
    EvalResponsesSource(EvalResponsesSource),
}
///CreateEvalRunRequest
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct CreateEvalRunRequest {
    ///Details about the run's data source.
    pub data_source: CreateEvalRunRequestDataSource,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub metadata: ::std::option::Option<Metadata>,
    ///The name of the run.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: ::std::option::Option<String>,
}
///Details about the run's data source.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
#[serde(untagged)]
pub enum CreateEvalRunRequestDataSource {
    CreateEvalJsonlRunDataSource(CreateEvalJsonlRunDataSource),
    CreateEvalCompletionsRunDataSource(CreateEvalCompletionsRunDataSource),
    CreateEvalResponsesRunDataSource(CreateEvalResponsesRunDataSource),
}
///Deprecated in favor of LogsDataSourceConfig.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct CreateEvalStoredCompletionsDataSourceConfig {
    ///Metadata filters for the stored completions data source.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub metadata: ::std::option::Option<OpenAiJsonValue>,
    ///The type of data source. Always `stored_completions`.
    #[serde(rename = "type")]
    pub type_value: String,
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct CreateFileRequest {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expires_after: ::std::option::Option<FileExpirationAfter>,
    ///The File object (not file name) to be uploaded.
    pub file: OpenAiBinaryBody,
    ///The intended purpose of the uploaded file. One of: - `assistants`: Used in the Assistants API - `batch`: Used in the Batch API - `fine-tune`: Used for fine-tuning - `vision`: Images used for vision fine-tuning - `user_data`: Flexible file type for any purpose - `evals`: Used for eval data sets
    pub purpose: String,
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct CreateFineTuningCheckpointPermissionRequest {
    ///The project identifiers to grant access to.
    pub project_ids: Vec<String>,
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct CreateFineTuningJobRequest {
    ///The hyperparameters used for the fine-tuning job. This value is now deprecated in favor of `method`, and should be passed in under the `method` parameter.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hyperparameters: ::std::option::Option<
        CreateFineTuningJobRequestHyperparameters,
    >,
    ///A list of integrations to enable for your fine-tuning job.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub integrations: ::std::option::Option<Vec<CreateFineTuningJobRequestIntegration>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub metadata: ::std::option::Option<Metadata>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub method: ::std::option::Option<FineTuneMethod>,
    ///The name of the model to fine-tune. You can select one of the [supported models](/docs/guides/fine-tuning#which-models-can-be-fine-tuned).
    pub model: String,
    ///The seed controls the reproducibility of the job. Passing in the same seed and job parameters should produce the same results, but may differ in rare cases. If a seed is not specified, one will be generated for you.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub seed: ::std::option::Option<i32>,
    ///A string of up to 64 characters that will be added to your fine-tuned model name. For example, a `suffix` of "custom-model-name" would produce a model name like `ft:gpt-4o-mini:openai:custom-model-name:7p4lURel`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub suffix: ::std::option::Option<String>,
    ///The ID of an uploaded file that contains training data. See [upload file](/docs/api-reference/files/create) for how to upload a file. Your dataset must be formatted as a JSONL file. Additionally, you must upload your file with the purpose `fine-tune`. The contents of the file should differ depending on if the model uses the [chat](/docs/api-reference/fine-tuning/chat-input), [completions](/docs/api-reference/fine-tuning/completions-input) format, or if the fine-tuning method uses the [preference](/docs/api-reference/fine-tuning/preference-input) format. See the [fine-tuning guide](/docs/guides/model-optimization) for more details.
    pub training_file: String,
    ///The ID of an uploaded file that contains validation data. If you provide this file, the data is used to generate validation metrics periodically during fine-tuning. These metrics can be viewed in the fine-tuning results file. The same data should not be present in both train and validation files. Your dataset must be formatted as a JSONL file. You must upload your file with the purpose `fine-tune`. See the [fine-tuning guide](/docs/guides/model-optimization) for more details.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub validation_file: ::std::option::Option<String>,
}
///The hyperparameters used for the fine-tuning job. This value is now deprecated in favor of `method`, and should be passed in under the `method` parameter.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct CreateFineTuningJobRequestHyperparameters {
    ///Number of examples in each batch. A larger batch size means that model parameters are updated less frequently, but with lower variance.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub batch_size: ::std::option::Option<
        CreateFineTuningJobRequestHyperparametersBatchSize,
    >,
    ///Scaling factor for the learning rate. A smaller learning rate may be useful to avoid overfitting.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub learning_rate_multiplier: ::std::option::Option<
        CreateFineTuningJobRequestHyperparametersLearningRateMultiplier,
    >,
    ///The number of epochs to train the model for. An epoch refers to one full cycle through the training dataset.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub n_epochs: ::std::option::Option<
        CreateFineTuningJobRequestHyperparametersNEpochs,
    >,
}
///Number of examples in each batch. A larger batch size means that model parameters are updated less frequently, but with lower variance.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
#[serde(untagged)]
pub enum CreateFineTuningJobRequestHyperparametersBatchSize {
    Auto(String),
    Integer(i32),
}
///Scaling factor for the learning rate. A smaller learning rate may be useful to avoid overfitting.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
#[serde(untagged)]
pub enum CreateFineTuningJobRequestHyperparametersLearningRateMultiplier {
    Auto(String),
    Number(f64),
}
///The number of epochs to train the model for. An epoch refers to one full cycle through the training dataset.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
#[serde(untagged)]
pub enum CreateFineTuningJobRequestHyperparametersNEpochs {
    Auto(String),
    Integer(i32),
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct CreateFineTuningJobRequestIntegration {
    ///The type of integration to enable. Currently, only "wandb" (Weights and Biases) is supported.
    #[serde(rename = "type")]
    pub type_value: String,
    ///The settings for your integration with Weights and Biases. This payload specifies the project that metrics will be sent to. Optionally, you can set an explicit display name for your run, add tags to your run, and set a default entity (team, username, etc) to be associated with your run.
    pub wandb: CreateFineTuningJobRequestIntegrationWandb,
}
///The settings for your integration with Weights and Biases. This payload specifies the project that metrics will be sent to. Optionally, you can set an explicit display name for your run, add tags to your run, and set a default entity (team, username, etc) to be associated with your run.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct CreateFineTuningJobRequestIntegrationWandb {
    ///The entity to use for the run. This allows you to set the team or username of the WandB user that you would like associated with the run. If not set, the default entity for the registered WandB API key is used.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub entity: ::std::option::Option<String>,
    ///A display name to set for the run. If not set, we will use the Job ID as the name.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: ::std::option::Option<String>,
    ///The name of the project that the new run will be created under.
    pub project: String,
    ///A list of tags to be attached to the newly created run. These tags are passed through directly to WandB. Some default tags are generated by OpenAI: "openai/finetune", "openai/{base-model}", "openai/{ftjob-abcdef}".
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tags: ::std::option::Option<Vec<String>>,
}
///Request payload for creating a new group in the organization.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct CreateGroupBody {
    ///Human readable name for the group.
    pub name: String,
}
///Request payload for adding a user to a group.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct CreateGroupUserBody {
    ///Identifier of the user to add to the group.
    pub user_id: String,
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct CreateImageEditRequest {
    ///Allows to set transparency for the background of the generated image(s). This parameter is only supported for the GPT image models. Must be one of `transparent`, `opaque` or `auto` (default value). When `auto` is used, the model will automatically determine the best background for the image. If `transparent`, the output format needs to support transparency, so it should be set to either `png` (default value) or `webp`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub background: ::std::option::Option<String>,
    ///The image(s) to edit. Must be a supported image file or an array of images. For the GPT image models (`gpt-image-1`, `gpt-image-1-mini`, and `gpt-image-1.5`), each image should be a `png`, `webp`, or `jpg` file less than 50MB. You can provide up to 16 images. `chatgpt-image-latest` follows the same input constraints as GPT image models. For `dall-e-2`, you can only provide one image, and it should be a square `png` file less than 4MB.
    pub image: CreateImageEditRequestImage,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub input_fidelity: ::std::option::Option<InputFidelity>,
    ///An additional image whose fully transparent areas (e.g. where alpha is zero) indicate where `image` should be edited. If there are multiple images provided, the mask will be applied on the first image. Must be a valid PNG file, less than 4MB, and have the same dimensions as `image`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mask: ::std::option::Option<OpenAiBinaryBody>,
    ///The model to use for image generation. Defaults to `gpt-image-1.5`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model: ::std::option::Option<String>,
    ///The number of images to generate. Must be between 1 and 10.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub n: ::std::option::Option<i32>,
    ///The compression level (0-100%) for the generated images. This parameter is only supported for the GPT image models with the `webp` or `jpeg` output formats, and defaults to 100.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub output_compression: ::std::option::Option<i32>,
    ///The format in which the generated images are returned. This parameter is only supported for the GPT image models. Must be one of `png`, `jpeg`, or `webp`. The default value is `png`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub output_format: ::std::option::Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub partial_images: ::std::option::Option<PartialImages>,
    ///A text description of the desired image(s). The maximum length is 1000 characters for `dall-e-2`, and 32000 characters for the GPT image models.
    pub prompt: String,
    ///The quality of the image that will be generated for GPT image models. Defaults to `auto`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub quality: ::std::option::Option<String>,
    ///The format in which the generated images are returned. Must be one of `url` or `b64_json`. URLs are only valid for 60 minutes after the image has been generated. This parameter is only supported for `dall-e-2` (default is `url` for `dall-e-2`), as GPT image models always return base64-encoded images.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub response_format: ::std::option::Option<String>,
    ///The size of the generated images. For `gpt-image-2` and `gpt-image-2-2026-04-21`, arbitrary resolutions are supported as `WIDTHxHEIGHT` strings, for example `1536x864`. Width and height must both be divisible by 16 and the requested aspect ratio must be between 1:3 and 3:1. Resolutions above `2560x1440` are experimental, and the maximum supported resolution is `3840x2160`. The requested size must also satisfy the model's current pixel and edge limits. The standard sizes `1024x1024`, `1536x1024`, and `1024x1536` are supported by the GPT image models; `auto` is supported for models that allow automatic sizing. For `dall-e-2`, use one of `256x256`, `512x512`, or `1024x1024`. For `dall-e-3`, use one of `1024x1024`, `1792x1024`, or `1024x1792`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub size: ::std::option::Option<String>,
    ///Edit the image in streaming mode. Defaults to `false`. See the [Image generation guide](/docs/guides/image-generation) for more information.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stream: ::std::option::Option<bool>,
    ///A unique identifier representing your end-user, which can help OpenAI to monitor and detect abuse. [Learn more](/docs/guides/safety-best-practices#end-user-ids).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub user: ::std::option::Option<String>,
}
///The image(s) to edit. Must be a supported image file or an array of images. For the GPT image models (`gpt-image-1`, `gpt-image-1-mini`, and `gpt-image-1.5`), each image should be a `png`, `webp`, or `jpg` file less than 50MB. You can provide up to 16 images. `chatgpt-image-latest` follows the same input constraints as GPT image models. For `dall-e-2`, you can only provide one image, and it should be a square `png` file less than 4MB.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
#[serde(untagged)]
pub enum CreateImageEditRequestImage {
    String(OpenAiBinaryBody),
    Array(Vec<OpenAiBinaryBody>),
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct CreateImageRequest {
    ///Allows to set transparency for the background of the generated image(s). This parameter is only supported for the GPT image models. Must be one of `transparent`, `opaque` or `auto` (default value). When `auto` is used, the model will automatically determine the best background for the image. If `transparent`, the output format needs to support transparency, so it should be set to either `png` (default value) or `webp`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub background: ::std::option::Option<String>,
    ///The model to use for image generation. One of `dall-e-2`, `dall-e-3`, or a GPT image model (`gpt-image-1`, `gpt-image-1-mini`, `gpt-image-1.5`). Defaults to `dall-e-2` unless a parameter specific to the GPT image models is used.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model: ::std::option::Option<String>,
    ///Control the content-moderation level for images generated by the GPT image models. Must be either `low` for less restrictive filtering or `auto` (default value).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub moderation: ::std::option::Option<String>,
    ///The number of images to generate. Must be between 1 and 10. For `dall-e-3`, only `n=1` is supported.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub n: ::std::option::Option<i32>,
    ///The compression level (0-100%) for the generated images. This parameter is only supported for the GPT image models with the `webp` or `jpeg` output formats, and defaults to 100.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub output_compression: ::std::option::Option<i32>,
    ///The format in which the generated images are returned. This parameter is only supported for the GPT image models. Must be one of `png`, `jpeg`, or `webp`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub output_format: ::std::option::Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub partial_images: ::std::option::Option<PartialImages>,
    ///A text description of the desired image(s). The maximum length is 32000 characters for the GPT image models, 1000 characters for `dall-e-2` and 4000 characters for `dall-e-3`.
    pub prompt: String,
    ///The quality of the image that will be generated. - `auto` (default value) will automatically select the best quality for the given model. - `high`, `medium` and `low` are supported for the GPT image models. - `hd` and `standard` are supported for `dall-e-3`. - `standard` is the only option for `dall-e-2`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub quality: ::std::option::Option<String>,
    ///The format in which generated images with `dall-e-2` and `dall-e-3` are returned. Must be one of `url` or `b64_json`. URLs are only valid for 60 minutes after the image has been generated. This parameter isn't supported for the GPT image models, which always return base64-encoded images.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub response_format: ::std::option::Option<String>,
    ///The size of the generated images. For `gpt-image-2` and `gpt-image-2-2026-04-21`, arbitrary resolutions are supported as `WIDTHxHEIGHT` strings, for example `1536x864`. Width and height must both be divisible by 16 and the requested aspect ratio must be between 1:3 and 3:1. Resolutions above `2560x1440` are experimental, and the maximum supported resolution is `3840x2160`. The requested size must also satisfy the model's current pixel and edge limits. The standard sizes `1024x1024`, `1536x1024`, and `1024x1536` are supported by the GPT image models; `auto` is supported for models that allow automatic sizing. For `dall-e-2`, use one of `256x256`, `512x512`, or `1024x1024`. For `dall-e-3`, use one of `1024x1024`, `1792x1024`, or `1024x1792`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub size: ::std::option::Option<String>,
    ///Generate the image in streaming mode. Defaults to `false`. See the [Image generation guide](/docs/guides/image-generation) for more information. This parameter is only supported for the GPT image models.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stream: ::std::option::Option<bool>,
    ///The style of the generated images. This parameter is only supported for `dall-e-3`. Must be one of `vivid` or `natural`. Vivid causes the model to lean towards generating hyper-real and dramatic images. Natural causes the model to produce more natural, less hyper-real looking images.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub style: ::std::option::Option<String>,
    ///A unique identifier representing your end-user, which can help OpenAI to monitor and detect abuse. [Learn more](/docs/guides/safety-best-practices#end-user-ids).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub user: ::std::option::Option<String>,
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct CreateImageVariationRequest {
    ///The image to use as the basis for the variation(s). Must be a valid PNG file, less than 4MB, and square.
    pub image: OpenAiBinaryBody,
    ///The model to use for image generation. Only `dall-e-2` is supported at this time.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model: ::std::option::Option<String>,
    ///The number of images to generate. Must be between 1 and 10.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub n: ::std::option::Option<i32>,
    ///The format in which the generated images are returned. Must be one of `url` or `b64_json`. URLs are only valid for 60 minutes after the image has been generated.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub response_format: ::std::option::Option<String>,
    ///The size of the generated images. Must be one of `256x256`, `512x512`, or `1024x1024`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub size: ::std::option::Option<String>,
    ///A unique identifier representing your end-user, which can help OpenAI to monitor and detect abuse. [Learn more](/docs/guides/safety-best-practices#end-user-ids).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub user: ::std::option::Option<String>,
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct CreateMessageRequest {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub attachments: ::std::option::Option<Vec<CreateMessageRequestAttachment>>,
    pub content: CreateMessageRequestContent,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub metadata: ::std::option::Option<Metadata>,
    ///The role of the entity that is creating the message. Allowed values include: - `user`: Indicates the message is sent by an actual user and should be used in most cases to represent user-generated messages. - `assistant`: Indicates the message is generated by the assistant. Use this value to insert messages from the assistant into the conversation.
    pub role: String,
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct CreateMessageRequestAttachment {
    ///The ID of the file to attach to the message.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub file_id: ::std::option::Option<String>,
    ///The tools to add this file to.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tools: ::std::option::Option<Vec<CreateMessageRequestAttachmentTool>>,
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
#[serde(untagged)]
pub enum CreateMessageRequestAttachmentTool {
    AssistantToolsCode(AssistantToolsCode),
    AssistantToolsFileSearchTypeOnly(AssistantToolsFileSearchTypeOnly),
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
#[serde(untagged)]
pub enum CreateMessageRequestContent {
    TextContent(String),
    ArrayOfContentParts(Vec<CreateMessageRequestContentArrayOfContentPart>),
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
#[serde(untagged)]
pub enum CreateMessageRequestContentArrayOfContentPart {
    MessageContentImageFileObject(MessageContentImageFileObject),
    MessageContentImageUrlObject(MessageContentImageUrlObject),
    MessageRequestContentTextObject(MessageRequestContentTextObject),
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
#[serde(untagged)]
pub enum CreateMessageRequestContentItem {
    MessageContentImageFileObject(MessageContentImageFileObject),
    MessageContentImageUrlObject(MessageContentImageUrlObject),
    MessageRequestContentTextObject(MessageRequestContentTextObject),
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct CreateModelResponseProperties {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub metadata: ::std::option::Option<Metadata>,
    ///Used by OpenAI to cache responses for similar requests to optimize your cache hit rates. Replaces the `user` field. [Learn more](/docs/guides/prompt-caching).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub prompt_cache_key: ::std::option::Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub prompt_cache_retention: ::std::option::Option<String>,
    ///A stable identifier used to help detect users of your application that may be violating OpenAI's usage policies. The IDs should be a string that uniquely identifies each user, with a maximum length of 64 characters. We recommend hashing their username or email address, in order to avoid sending us any identifying information. [Learn more](/docs/guides/safety-best-practices#safety-identifiers).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub safety_identifier: ::std::option::Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub service_tier: ::std::option::Option<ServiceTier>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub temperature: ::std::option::Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub top_logprobs: ::std::option::Option<i32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub top_p: ::std::option::Option<f64>,
    ///This field is being replaced by `safety_identifier` and `prompt_cache_key`. Use `prompt_cache_key` instead to maintain caching optimizations. A stable identifier for your end-users. Used to boost cache hit rates by better bucketing similar requests and to help OpenAI detect and prevent abuse. [Learn more](/docs/guides/safety-best-practices#safety-identifiers).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub user: ::std::option::Option<String>,
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct CreateModerationRequest {
    ///Input (or inputs) to classify. Can be a single string, an array of strings, or an array of multi-modal input objects similar to other models.
    pub input: CreateModerationRequestInput,
    ///The content moderation model you would like to use. Learn more in [the moderation guide](/docs/guides/moderation), and learn about available models [here](/docs/models#moderation).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model: ::std::option::Option<String>,
}
///Input (or inputs) to classify. Can be a single string, an array of strings, or an array of multi-modal input objects similar to other models.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
#[serde(untagged)]
pub enum CreateModerationRequestInput {
    String(String),
    Array(Vec<String>),
    Array3(Vec<CreateModerationRequestInputArray3Item3>),
}
///An object describing an image to classify.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct CreateModerationRequestInputArray3Item {
    ///Contains either an image URL or a data URL for a base64 encoded image.
    pub image_url: CreateModerationRequestInputArray3ItemImageUrl,
    ///Always `image_url`.
    #[serde(rename = "type")]
    pub type_value: String,
}
///An object describing text to classify.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct CreateModerationRequestInputArray3Item2 {
    ///A string of text to classify.
    pub text: String,
    ///Always `text`.
    #[serde(rename = "type")]
    pub type_value: String,
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
#[serde(untagged)]
pub enum CreateModerationRequestInputArray3Item3 {
    Object(CreateModerationRequestInputArray3Item3Object),
    Object2(CreateModerationRequestInputArray3Item3Object2),
}
///An object describing an image to classify.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct CreateModerationRequestInputArray3Item3Object {
    ///Contains either an image URL or a data URL for a base64 encoded image.
    pub image_url: CreateModerationRequestInputArray3Item3ObjectImageUrl,
    ///Always `image_url`.
    #[serde(rename = "type")]
    pub type_value: String,
}
///An object describing text to classify.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct CreateModerationRequestInputArray3Item3Object2 {
    ///A string of text to classify.
    pub text: String,
    ///Always `text`.
    #[serde(rename = "type")]
    pub type_value: String,
}
///Contains either an image URL or a data URL for a base64 encoded image.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct CreateModerationRequestInputArray3Item3ObjectImageUrl {
    ///Either a URL of the image or the base64 encoded image data.
    pub url: String,
}
///Contains either an image URL or a data URL for a base64 encoded image.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct CreateModerationRequestInputArray3ItemImageUrl {
    ///Either a URL of the image or the base64 encoded image data.
    pub url: String,
}
///An object describing an image to classify.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct CreateModerationRequestInputItem {
    ///Contains either an image URL or a data URL for a base64 encoded image.
    pub image_url: CreateModerationRequestInputItemImageUrl,
    ///Always `image_url`.
    #[serde(rename = "type")]
    pub type_value: String,
}
///An object describing text to classify.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct CreateModerationRequestInputItem2 {
    ///A string of text to classify.
    pub text: String,
    ///Always `text`.
    #[serde(rename = "type")]
    pub type_value: String,
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
#[serde(untagged)]
pub enum CreateModerationRequestInputItem3 {
    Object(CreateModerationRequestInputItem3Object),
    Object2(CreateModerationRequestInputItem3Object2),
}
///An object describing an image to classify.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct CreateModerationRequestInputItem3Object {
    ///Contains either an image URL or a data URL for a base64 encoded image.
    pub image_url: CreateModerationRequestInputItem3ObjectImageUrl,
    ///Always `image_url`.
    #[serde(rename = "type")]
    pub type_value: String,
}
///An object describing text to classify.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct CreateModerationRequestInputItem3Object2 {
    ///A string of text to classify.
    pub text: String,
    ///Always `text`.
    #[serde(rename = "type")]
    pub type_value: String,
}
///Contains either an image URL or a data URL for a base64 encoded image.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct CreateModerationRequestInputItem3ObjectImageUrl {
    ///Either a URL of the image or the base64 encoded image data.
    pub url: String,
}
///Contains either an image URL or a data URL for a base64 encoded image.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct CreateModerationRequestInputItemImageUrl {
    ///Either a URL of the image or the base64 encoded image data.
    pub url: String,
}
///Represents if a given text input is potentially harmful.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct CreateModerationResponse {
    ///The unique identifier for the moderation request.
    pub id: String,
    ///The model used to generate the moderation results.
    pub model: String,
    ///A list of moderation objects.
    pub results: Vec<CreateModerationResponseResult>,
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct CreateModerationResponseResult {
    ///A list of the categories, and whether they are flagged or not.
    pub categories: CreateModerationResponseResultCategories,
    ///A list of the categories along with the input type(s) that the score applies to.
    pub category_applied_input_types: CreateModerationResponseResultCategoryAppliedInputTypes,
    ///A list of the categories along with their scores as predicted by model.
    pub category_scores: CreateModerationResponseResultCategoryScores,
    ///Whether any of the below categories are flagged.
    pub flagged: bool,
}
///A list of the categories, and whether they are flagged or not.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct CreateModerationResponseResultCategories {
    ///Content that expresses, incites, or promotes harassing language towards any target.
    pub harassment: bool,
    ///Harassment content that also includes violence or serious harm towards any target.
    #[serde(rename = "harassment/threatening")]
    pub harassment_threatening: bool,
    ///Content that expresses, incites, or promotes hate based on race, gender, ethnicity, religion, nationality, sexual orientation, disability status, or caste. Hateful content aimed at non-protected groups (e.g., chess players) is harassment.
    pub hate: bool,
    ///Hateful content that also includes violence or serious harm towards the targeted group based on race, gender, ethnicity, religion, nationality, sexual orientation, disability status, or caste.
    #[serde(rename = "hate/threatening")]
    pub hate_threatening: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub illicit: ::std::option::Option<bool>,
    #[serde(
        rename = "illicit/violent",
        default,
        skip_serializing_if = "Option::is_none",
    )]
    pub illicit_violent: ::std::option::Option<bool>,
    ///Content that promotes, encourages, or depicts acts of self-harm, such as suicide, cutting, and eating disorders.
    #[serde(rename = "self-harm")]
    pub self_harm: bool,
    ///Content that encourages performing acts of self-harm, such as suicide, cutting, and eating disorders, or that gives instructions or advice on how to commit such acts.
    #[serde(rename = "self-harm/instructions")]
    pub self_harm_instructions: bool,
    ///Content where the speaker expresses that they are engaging or intend to engage in acts of self-harm, such as suicide, cutting, and eating disorders.
    #[serde(rename = "self-harm/intent")]
    pub self_harm_intent: bool,
    ///Content meant to arouse sexual excitement, such as the description of sexual activity, or that promotes sexual services (excluding sex education and wellness).
    pub sexual: bool,
    ///Sexual content that includes an individual who is under 18 years old.
    #[serde(rename = "sexual/minors")]
    pub sexual_minors: bool,
    ///Content that depicts death, violence, or physical injury.
    pub violence: bool,
    ///Content that depicts death, violence, or physical injury in graphic detail.
    #[serde(rename = "violence/graphic")]
    pub violence_graphic: bool,
}
///A list of the categories along with the input type(s) that the score applies to.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct CreateModerationResponseResultCategoryAppliedInputTypes {
    ///The applied input type(s) for the category 'harassment'.
    pub harassment: Vec<String>,
    ///The applied input type(s) for the category 'harassment/threatening'.
    #[serde(rename = "harassment/threatening")]
    pub harassment_threatening: Vec<String>,
    ///The applied input type(s) for the category 'hate'.
    pub hate: Vec<String>,
    ///The applied input type(s) for the category 'hate/threatening'.
    #[serde(rename = "hate/threatening")]
    pub hate_threatening: Vec<String>,
    ///The applied input type(s) for the category 'illicit'.
    pub illicit: Vec<String>,
    ///The applied input type(s) for the category 'illicit/violent'.
    #[serde(rename = "illicit/violent")]
    pub illicit_violent: Vec<String>,
    ///The applied input type(s) for the category 'self-harm'.
    #[serde(rename = "self-harm")]
    pub self_harm: Vec<String>,
    ///The applied input type(s) for the category 'self-harm/instructions'.
    #[serde(rename = "self-harm/instructions")]
    pub self_harm_instructions: Vec<String>,
    ///The applied input type(s) for the category 'self-harm/intent'.
    #[serde(rename = "self-harm/intent")]
    pub self_harm_intent: Vec<String>,
    ///The applied input type(s) for the category 'sexual'.
    pub sexual: Vec<String>,
    ///The applied input type(s) for the category 'sexual/minors'.
    #[serde(rename = "sexual/minors")]
    pub sexual_minors: Vec<String>,
    ///The applied input type(s) for the category 'violence'.
    pub violence: Vec<String>,
    ///The applied input type(s) for the category 'violence/graphic'.
    #[serde(rename = "violence/graphic")]
    pub violence_graphic: Vec<String>,
}
///A list of the categories along with their scores as predicted by model.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct CreateModerationResponseResultCategoryScores {
    ///The score for the category 'harassment'.
    pub harassment: f64,
    ///The score for the category 'harassment/threatening'.
    #[serde(rename = "harassment/threatening")]
    pub harassment_threatening: f64,
    ///The score for the category 'hate'.
    pub hate: f64,
    ///The score for the category 'hate/threatening'.
    #[serde(rename = "hate/threatening")]
    pub hate_threatening: f64,
    ///The score for the category 'illicit'.
    pub illicit: f64,
    ///The score for the category 'illicit/violent'.
    #[serde(rename = "illicit/violent")]
    pub illicit_violent: f64,
    ///The score for the category 'self-harm'.
    #[serde(rename = "self-harm")]
    pub self_harm: f64,
    ///The score for the category 'self-harm/instructions'.
    #[serde(rename = "self-harm/instructions")]
    pub self_harm_instructions: f64,
    ///The score for the category 'self-harm/intent'.
    #[serde(rename = "self-harm/intent")]
    pub self_harm_intent: f64,
    ///The score for the category 'sexual'.
    pub sexual: f64,
    ///The score for the category 'sexual/minors'.
    #[serde(rename = "sexual/minors")]
    pub sexual_minors: f64,
    ///The score for the category 'violence'.
    pub violence: f64,
    ///The score for the category 'violence/graphic'.
    #[serde(rename = "violence/graphic")]
    pub violence_graphic: f64,
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct CreateResponse {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub background: ::std::option::Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub context_management: ::std::option::Option<Vec<ContextManagementParam>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub conversation: ::std::option::Option<ConversationParam>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub include: ::std::option::Option<Vec<IncludeEnum>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub input: ::std::option::Option<InputParam>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub instructions: ::std::option::Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_output_tokens: ::std::option::Option<i32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_tool_calls: ::std::option::Option<i32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub metadata: ::std::option::Option<Metadata>,
    ///Model ID used to generate the response, like `gpt-4o` or `o3`. OpenAI offers a wide range of models with different capabilities, performance characteristics, and price points. Refer to the [model guide](/docs/models) to browse and compare available models.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model: ::std::option::Option<ModelIdsResponses>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parallel_tool_calls: ::std::option::Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub previous_response_id: ::std::option::Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub prompt: ::std::option::Option<Prompt>,
    ///Used by OpenAI to cache responses for similar requests to optimize your cache hit rates. Replaces the `user` field. [Learn more](/docs/guides/prompt-caching).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub prompt_cache_key: ::std::option::Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub prompt_cache_retention: ::std::option::Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reasoning: ::std::option::Option<Reasoning>,
    ///A stable identifier used to help detect users of your application that may be violating OpenAI's usage policies. The IDs should be a string that uniquely identifies each user, with a maximum length of 64 characters. We recommend hashing their username or email address, in order to avoid sending us any identifying information. [Learn more](/docs/guides/safety-best-practices#safety-identifiers).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub safety_identifier: ::std::option::Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub service_tier: ::std::option::Option<ServiceTier>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub store: ::std::option::Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stream: ::std::option::Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stream_options: ::std::option::Option<ResponseStreamOptions>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub temperature: ::std::option::Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub text: ::std::option::Option<ResponseTextParam>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tool_choice: ::std::option::Option<ToolChoiceParam>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tools: ::std::option::Option<ToolsArray>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub top_logprobs: ::std::option::Option<i32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub top_p: ::std::option::Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub truncation: ::std::option::Option<String>,
    ///This field is being replaced by `safety_identifier` and `prompt_cache_key`. Use `prompt_cache_key` instead to maintain caching optimizations. A stable identifier for your end-users. Used to boost cache hit rates by better bucketing similar requests and to help OpenAI detect and prevent abuse. [Learn more](/docs/guides/safety-best-practices#safety-identifiers).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub user: ::std::option::Option<String>,
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct CreateRunRequest {
    ///Appends additional instructions at the end of the instructions for the run. This is useful for modifying the behavior on a per-run basis without overriding other instructions.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub additional_instructions: ::std::option::Option<String>,
    ///Adds additional messages to the thread before creating the run.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub additional_messages: ::std::option::Option<Vec<CreateMessageRequest>>,
    ///The ID of the [assistant](/docs/api-reference/assistants) to use to execute this run.
    pub assistant_id: String,
    ///Overrides the [instructions](/docs/api-reference/assistants/createAssistant) of the assistant. This is useful for modifying the behavior on a per-run basis.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub instructions: ::std::option::Option<String>,
    ///The maximum number of completion tokens that may be used over the course of the run. The run will make a best effort to use only the number of completion tokens specified, across multiple turns of the run. If the run exceeds the number of completion tokens specified, the run will end with status `incomplete`. See `incomplete_details` for more info.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_completion_tokens: ::std::option::Option<i32>,
    ///The maximum number of prompt tokens that may be used over the course of the run. The run will make a best effort to use only the number of prompt tokens specified, across multiple turns of the run. If the run exceeds the number of prompt tokens specified, the run will end with status `incomplete`. See `incomplete_details` for more info.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_prompt_tokens: ::std::option::Option<i32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub metadata: ::std::option::Option<Metadata>,
    ///The ID of the [Model](/docs/api-reference/models) to be used to execute this run. If a value is provided here, it will override the model associated with the assistant. If not, the model associated with the assistant will be used.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model: ::std::option::Option<CreateRunRequestModel>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parallel_tool_calls: ::std::option::Option<ParallelToolCalls>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reasoning_effort: ::std::option::Option<ReasoningEffort>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub response_format: ::std::option::Option<AssistantsApiResponseFormatOption>,
    ///If `true`, returns a stream of events that happen during the Run as server-sent events, terminating when the Run enters a terminal state with a `data: [DONE]` message.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stream: ::std::option::Option<bool>,
    ///What sampling temperature to use, between 0 and 2. Higher values like 0.8 will make the output more random, while lower values like 0.2 will make it more focused and deterministic.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub temperature: ::std::option::Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tool_choice: ::std::option::Option<CreateRunRequestToolChoice>,
    ///Override the tools the assistant can use for this run. This is useful for modifying the behavior on a per-run basis.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tools: ::std::option::Option<Vec<CreateRunRequestTool>>,
    ///An alternative to sampling with temperature, called nucleus sampling, where the model considers the results of the tokens with top_p probability mass. So 0.1 means only the tokens comprising the top 10% probability mass are considered. We generally recommend altering this or temperature but not both.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub top_p: ::std::option::Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub truncation_strategy: ::std::option::Option<CreateRunRequestTruncationStrategy>,
}
///The ID of the [Model](/docs/api-reference/models) to be used to execute this run. If a value is provided here, it will override the model associated with the assistant. If not, the model associated with the assistant will be used.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
#[serde(untagged)]
pub enum CreateRunRequestModel {
    String(String),
    AssistantSupportedModels(AssistantSupportedModels),
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
#[serde(untagged)]
pub enum CreateRunRequestTool {
    AssistantToolsCode(AssistantToolsCode),
    AssistantToolsFileSearch(AssistantToolsFileSearch),
    AssistantToolsFunction(AssistantToolsFunction),
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct CreateRunRequestToolChoice {
    #[serde(flatten)]
    pub value: OpenAiJsonObject,
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct CreateRunRequestTruncationStrategy {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_messages: ::std::option::Option<i32>,
    ///The truncation strategy to use for the thread. The default is `auto`. If set to `last_messages`, the thread will be truncated to the n most recent messages in the thread. When set to `auto`, messages in the middle of the thread will be dropped to fit the context length of the model, `max_prompt_tokens`.
    #[serde(rename = "type")]
    pub type_value: String,
}
///Uploads a skill either as a directory (multipart `files[]`) or as a single zip file.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct CreateSkillBody {
    pub files: CreateSkillBodyFiles,
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
#[serde(untagged)]
pub enum CreateSkillBodyFiles {
    Array(Vec<OpenAiBinaryBody>),
    String(OpenAiBinaryBody),
}
///Uploads a new immutable version of a skill.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct CreateSkillVersionBody {
    ///Whether to set this version as the default.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default: ::std::option::Option<bool>,
    pub files: CreateSkillVersionBodyFiles,
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
#[serde(untagged)]
pub enum CreateSkillVersionBodyFiles {
    Array(Vec<OpenAiBinaryBody>),
    String(OpenAiBinaryBody),
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct CreateSpeechRequest {
    ///The text to generate audio for. The maximum length is 4096 characters.
    pub input: String,
    ///Control the voice of your generated audio with additional instructions. Does not work with `tts-1` or `tts-1-hd`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub instructions: ::std::option::Option<String>,
    ///One of the available [TTS models](/docs/models#tts): `tts-1`, `tts-1-hd`, `gpt-4o-mini-tts`, or `gpt-4o-mini-tts-2025-12-15`.
    pub model: String,
    ///The format to audio in. Supported formats are `mp3`, `opus`, `aac`, `flac`, `wav`, and `pcm`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub response_format: ::std::option::Option<String>,
    ///The speed of the generated audio. Select a value from `0.25` to `4.0`. `1.0` is the default.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub speed: ::std::option::Option<f64>,
    ///The format to stream the audio in. Supported formats are `sse` and `audio`. `sse` is not supported for `tts-1` or `tts-1-hd`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stream_format: ::std::option::Option<String>,
    ///The voice to use when generating the audio. Supported built-in voices are `alloy`, `ash`, `ballad`, `coral`, `echo`, `fable`, `onyx`, `nova`, `sage`, `shimmer`, `verse`, `marin`, and `cedar`. You may also provide a custom voice object with an `id`, for example `{ "id": "voice_1234" }`. Previews of the voices are available in the [Text to speech guide](/docs/guides/text-to-speech#voice-options).
    pub voice: VoiceIdsOrCustomVoice,
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
#[serde(untagged)]
pub enum CreateSpeechResponseStreamEvent {
    SpeechAudioDeltaEvent(SpeechAudioDeltaEvent),
    SpeechAudioDoneEvent(SpeechAudioDoneEvent),
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct CreateThreadAndRunRequest {
    ///The ID of the [assistant](/docs/api-reference/assistants) to use to execute this run.
    pub assistant_id: String,
    ///Override the default system message of the assistant. This is useful for modifying the behavior on a per-run basis.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub instructions: ::std::option::Option<String>,
    ///The maximum number of completion tokens that may be used over the course of the run. The run will make a best effort to use only the number of completion tokens specified, across multiple turns of the run. If the run exceeds the number of completion tokens specified, the run will end with status `incomplete`. See `incomplete_details` for more info.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_completion_tokens: ::std::option::Option<i32>,
    ///The maximum number of prompt tokens that may be used over the course of the run. The run will make a best effort to use only the number of prompt tokens specified, across multiple turns of the run. If the run exceeds the number of prompt tokens specified, the run will end with status `incomplete`. See `incomplete_details` for more info.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_prompt_tokens: ::std::option::Option<i32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub metadata: ::std::option::Option<Metadata>,
    ///The ID of the [Model](/docs/api-reference/models) to be used to execute this run. If a value is provided here, it will override the model associated with the assistant. If not, the model associated with the assistant will be used.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model: ::std::option::Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parallel_tool_calls: ::std::option::Option<ParallelToolCalls>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub response_format: ::std::option::Option<AssistantsApiResponseFormatOption>,
    ///If `true`, returns a stream of events that happen during the Run as server-sent events, terminating when the Run enters a terminal state with a `data: [DONE]` message.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stream: ::std::option::Option<bool>,
    ///What sampling temperature to use, between 0 and 2. Higher values like 0.8 will make the output more random, while lower values like 0.2 will make it more focused and deterministic.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub temperature: ::std::option::Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub thread: ::std::option::Option<CreateThreadRequest>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tool_choice: ::std::option::Option<CreateThreadAndRunRequestToolChoice>,
    ///A set of resources that are used by the assistant's tools. The resources are specific to the type of tool. For example, the `code_interpreter` tool requires a list of file IDs, while the `file_search` tool requires a list of vector store IDs.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tool_resources: ::std::option::Option<CreateThreadAndRunRequestToolResources>,
    ///Override the tools the assistant can use for this run. This is useful for modifying the behavior on a per-run basis.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tools: ::std::option::Option<Vec<CreateThreadAndRunRequestTool>>,
    ///An alternative to sampling with temperature, called nucleus sampling, where the model considers the results of the tokens with top_p probability mass. So 0.1 means only the tokens comprising the top 10% probability mass are considered. We generally recommend altering this or temperature but not both.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub top_p: ::std::option::Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub truncation_strategy: ::std::option::Option<
        CreateThreadAndRunRequestTruncationStrategy,
    >,
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
#[serde(untagged)]
pub enum CreateThreadAndRunRequestTool {
    AssistantToolsCode(AssistantToolsCode),
    AssistantToolsFileSearch(AssistantToolsFileSearch),
    AssistantToolsFunction(AssistantToolsFunction),
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct CreateThreadAndRunRequestToolChoice {
    #[serde(flatten)]
    pub value: OpenAiJsonObject,
}
///A set of resources that are used by the assistant's tools. The resources are specific to the type of tool. For example, the `code_interpreter` tool requires a list of file IDs, while the `file_search` tool requires a list of vector store IDs.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct CreateThreadAndRunRequestToolResources {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub code_interpreter: ::std::option::Option<
        CreateThreadAndRunRequestToolResourcesCodeInterpreter,
    >,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub file_search: ::std::option::Option<
        CreateThreadAndRunRequestToolResourcesFileSearch,
    >,
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct CreateThreadAndRunRequestToolResourcesCodeInterpreter {
    ///A list of [file](/docs/api-reference/files) IDs made available to the `code_interpreter` tool. There can be a maximum of 20 files associated with the tool.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub file_ids: ::std::option::Option<Vec<String>>,
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct CreateThreadAndRunRequestToolResourcesFileSearch {
    ///The ID of the [vector store](/docs/api-reference/vector-stores/object) attached to this assistant. There can be a maximum of 1 vector store attached to the assistant.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub vector_store_ids: ::std::option::Option<Vec<String>>,
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct CreateThreadAndRunRequestTruncationStrategy {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_messages: ::std::option::Option<i32>,
    ///The truncation strategy to use for the thread. The default is `auto`. If set to `last_messages`, the thread will be truncated to the n most recent messages in the thread. When set to `auto`, messages in the middle of the thread will be dropped to fit the context length of the model, `max_prompt_tokens`.
    #[serde(rename = "type")]
    pub type_value: String,
}
///Options to create a new thread. If no thread is provided when running a request, an empty thread will be created.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct CreateThreadRequest {
    ///A list of [messages](/docs/api-reference/messages) to start the thread with.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub messages: ::std::option::Option<Vec<CreateMessageRequest>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub metadata: ::std::option::Option<Metadata>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tool_resources: ::std::option::Option<CreateThreadRequestToolResources>,
}
///A set of resources that are made available to the assistant's tools in this thread. The resources are specific to the type of tool. For example, the `code_interpreter` tool requires a list of file IDs, while the `file_search` tool requires a list of vector store IDs.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct CreateThreadRequestToolResources {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub code_interpreter: ::std::option::Option<
        CreateThreadRequestToolResourcesCodeInterpreter,
    >,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub file_search: ::std::option::Option<CreateThreadRequestToolResourcesFileSearch>,
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct CreateThreadRequestToolResourcesCodeInterpreter {
    ///A list of [file](/docs/api-reference/files) IDs made available to the `code_interpreter` tool. There can be a maximum of 20 files associated with the tool.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub file_ids: ::std::option::Option<Vec<String>>,
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
#[serde(untagged)]
pub enum CreateThreadRequestToolResourcesFileSearch {
    Variant1(OpenAiJsonValue),
    Variant2(OpenAiJsonValue),
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct CreateTranscriptionRequest {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub chunking_strategy: ::std::option::Option<
        CreateTranscriptionRequestChunkingStrategy,
    >,
    ///The audio file object (not file name) to transcribe, in one of these formats: flac, mp3, mp4, mpeg, mpga, m4a, ogg, wav, or webm.
    pub file: OpenAiBinaryBody,
    ///Additional information to include in the transcription response. `logprobs` will return the log probabilities of the tokens in the response to understand the model's confidence in the transcription. `logprobs` only works with response_format set to `json` and only with the models `gpt-4o-transcribe`, `gpt-4o-mini-transcribe`, and `gpt-4o-mini-transcribe-2025-12-15`. This field is not supported when using `gpt-4o-transcribe-diarize`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub include: ::std::option::Option<Vec<TranscriptionInclude>>,
    ///Optional list of speaker names that correspond to the audio samples provided in `known_speaker_references[]`. Each entry should be a short identifier (for example `customer` or `agent`). Up to 4 speakers are supported.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub known_speaker_names: ::std::option::Option<Vec<String>>,
    ///Optional list of audio samples (as [data URLs](https://developer.mozilla.org/en-US/docs/Web/HTTP/Basics_of_HTTP/Data_URLs)) that contain known speaker references matching `known_speaker_names[]`. Each sample must be between 2 and 10 seconds, and can use any of the same input audio formats supported by `file`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub known_speaker_references: ::std::option::Option<Vec<String>>,
    ///The language of the input audio. Supplying the input language in [ISO-639-1](https://en.wikipedia.org/wiki/List_of_ISO_639-1_codes) (e.g. `en`) format will improve accuracy and latency.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub language: ::std::option::Option<String>,
    ///ID of the model to use. The options are `gpt-4o-transcribe`, `gpt-4o-mini-transcribe`, `gpt-4o-mini-transcribe-2025-12-15`, `whisper-1` (which is powered by our open source Whisper V2 model), and `gpt-4o-transcribe-diarize`.
    pub model: String,
    ///An optional text to guide the model's style or continue a previous audio segment. The [prompt](/docs/guides/speech-to-text#prompting) should match the audio language. This field is not supported when using `gpt-4o-transcribe-diarize`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub prompt: ::std::option::Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub response_format: ::std::option::Option<AudioResponseFormat>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stream: ::std::option::Option<bool>,
    ///The sampling temperature, between 0 and 1. Higher values like 0.8 will make the output more random, while lower values like 0.2 will make it more focused and deterministic. If set to 0, the model will use [log probability](https://en.wikipedia.org/wiki/Log_probability) to automatically increase the temperature until certain thresholds are hit.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub temperature: ::std::option::Option<f64>,
    ///The timestamp granularities to populate for this transcription. `response_format` must be set `verbose_json` to use timestamp granularities. Either or both of these options are supported: `word`, or `segment`. Note: There is no additional latency for segment timestamps, but generating word timestamps incurs additional latency. This option is not available for `gpt-4o-transcribe-diarize`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub timestamp_granularities: ::std::option::Option<Vec<String>>,
}
///Controls how the audio is cut into chunks. When set to `"auto"`, the server first normalizes loudness and then uses voice activity detection (VAD) to choose boundaries. `server_vad` object can be provided to tweak VAD detection parameters manually. If unset, the audio is transcribed as a single block. Required when using `gpt-4o-transcribe-diarize` for inputs longer than 30 seconds.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
#[serde(untagged)]
pub enum CreateTranscriptionRequestChunkingStrategy {
    Auto(String),
    VadConfig(VadConfig),
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
#[serde(untagged)]
pub enum CreateTranscriptionResponse {
    CreateTranscriptionResponseJson(CreateTranscriptionResponseJson),
    CreateTranscriptionResponseDiarizedJson(CreateTranscriptionResponseDiarizedJson),
    CreateTranscriptionResponseVerboseJson(CreateTranscriptionResponseVerboseJson),
}
///Represents a diarized transcription response returned by the model, including the combined transcript and speaker-segment annotations.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct CreateTranscriptionResponseDiarizedJson {
    ///Duration of the input audio in seconds.
    pub duration: f64,
    ///Segments of the transcript annotated with timestamps and speaker labels.
    pub segments: Vec<TranscriptionDiarizedSegment>,
    ///The type of task that was run. Always `transcribe`.
    pub task: String,
    ///The concatenated transcript text for the entire audio input.
    pub text: String,
    ///Token or duration usage statistics for the request.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub usage: ::std::option::Option<CreateTranscriptionResponseDiarizedJsonUsage>,
}
///Token or duration usage statistics for the request.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
#[serde(untagged)]
pub enum CreateTranscriptionResponseDiarizedJsonUsage {
    TranscriptTextUsageTokens(TranscriptTextUsageTokens),
    TranscriptTextUsageDuration(TranscriptTextUsageDuration),
}
///Represents a transcription response returned by model, based on the provided input.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct CreateTranscriptionResponseJson {
    ///The log probabilities of the tokens in the transcription. Only returned with the models `gpt-4o-transcribe` and `gpt-4o-mini-transcribe` if `logprobs` is added to the `include` array.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub logprobs: ::std::option::Option<Vec<CreateTranscriptionResponseJsonLogprob>>,
    ///The transcribed text.
    pub text: String,
    ///Token usage statistics for the request.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub usage: ::std::option::Option<CreateTranscriptionResponseJsonUsage>,
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct CreateTranscriptionResponseJsonLogprob {
    ///The bytes of the token.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bytes: ::std::option::Option<Vec<f64>>,
    ///The log probability of the token.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub logprob: ::std::option::Option<f64>,
    ///The token in the transcription.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub token: ::std::option::Option<String>,
}
///Token usage statistics for the request.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
#[serde(untagged)]
pub enum CreateTranscriptionResponseJsonUsage {
    TranscriptTextUsageTokens(TranscriptTextUsageTokens),
    TranscriptTextUsageDuration(TranscriptTextUsageDuration),
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
#[serde(untagged)]
pub enum CreateTranscriptionResponseStreamEvent {
    TranscriptTextSegmentEvent(TranscriptTextSegmentEvent),
    TranscriptTextDeltaEvent(TranscriptTextDeltaEvent),
    TranscriptTextDoneEvent(TranscriptTextDoneEvent),
}
///Represents a verbose json transcription response returned by model, based on the provided input.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct CreateTranscriptionResponseVerboseJson {
    ///The duration of the input audio.
    pub duration: f64,
    ///The language of the input audio.
    pub language: String,
    ///Segments of the transcribed text and their corresponding details.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub segments: ::std::option::Option<Vec<TranscriptionSegment>>,
    ///The transcribed text.
    pub text: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub usage: ::std::option::Option<TranscriptTextUsageDuration>,
    ///Extracted words and their corresponding timestamps.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub words: ::std::option::Option<Vec<TranscriptionWord>>,
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct CreateTranslationRequest {
    ///The audio file object (not file name) translate, in one of these formats: flac, mp3, mp4, mpeg, mpga, m4a, ogg, wav, or webm.
    pub file: OpenAiBinaryBody,
    ///ID of the model to use. Only `whisper-1` (which is powered by our open source Whisper V2 model) is currently available.
    pub model: String,
    ///An optional text to guide the model's style or continue a previous audio segment. The [prompt](/docs/guides/speech-to-text#prompting) should be in English.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub prompt: ::std::option::Option<String>,
    ///The format of the output, in one of these options: `json`, `text`, `srt`, `verbose_json`, or `vtt`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub response_format: ::std::option::Option<String>,
    ///The sampling temperature, between 0 and 1. Higher values like 0.8 will make the output more random, while lower values like 0.2 will make it more focused and deterministic. If set to 0, the model will use [log probability](https://en.wikipedia.org/wiki/Log_probability) to automatically increase the temperature until certain thresholds are hit.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub temperature: ::std::option::Option<f64>,
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
#[serde(untagged)]
pub enum CreateTranslationResponse {
    CreateTranslationResponseJson(CreateTranslationResponseJson),
    CreateTranslationResponseVerboseJson(CreateTranslationResponseVerboseJson),
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct CreateTranslationResponseJson {
    pub text: String,
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct CreateTranslationResponseVerboseJson {
    ///The duration of the input audio.
    pub duration: f64,
    ///The language of the output translation (always `english`).
    pub language: String,
    ///Segments of the translated text and their corresponding details.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub segments: ::std::option::Option<Vec<TranscriptionSegment>>,
    ///The translated text.
    pub text: String,
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct CreateUploadRequest {
    ///The number of bytes in the file you are uploading.
    pub bytes: i32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expires_after: ::std::option::Option<FileExpirationAfter>,
    ///The name of the file to upload.
    pub filename: String,
    ///The MIME type of the file. This must fall within the supported MIME types for your file purpose. See the supported MIME types for assistants and vision.
    pub mime_type: String,
    ///The intended purpose of the uploaded file. See the [documentation on File purposes](/docs/api-reference/files/create#files-create-purpose).
    pub purpose: String,
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct CreateVectorStoreFileBatchRequest {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub attributes: ::std::option::Option<VectorStoreFileAttributes>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub chunking_strategy: ::std::option::Option<ChunkingStrategyRequestParam>,
    ///A list of [File](/docs/api-reference/files) IDs that the vector store should use. Useful for tools like `file_search` that can access files. If `attributes` or `chunking_strategy` are provided, they will be applied to all files in the batch. The maximum batch size is 2000 files. This endpoint is recommended for multi-file ingestion and helps reduce per-vector-store write request pressure. Mutually exclusive with `files`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub file_ids: ::std::option::Option<Vec<String>>,
    ///A list of objects that each include a `file_id` plus optional `attributes` or `chunking_strategy`. Use this when you need to override metadata for specific files. The global `attributes` or `chunking_strategy` will be ignored and must be specified for each file. The maximum batch size is 2000 files. This endpoint is recommended for multi-file ingestion and helps reduce per-vector-store write request pressure. Mutually exclusive with `file_ids`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub files: ::std::option::Option<Vec<CreateVectorStoreFileRequest>>,
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct CreateVectorStoreFileRequest {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub attributes: ::std::option::Option<VectorStoreFileAttributes>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub chunking_strategy: ::std::option::Option<ChunkingStrategyRequestParam>,
    ///A [File](/docs/api-reference/files) ID that the vector store should use. Useful for tools like `file_search` that can access files. For multi-file ingestion, we recommend [`file_batches`](/docs/api-reference/vector-stores-file-batches/createBatch) to minimize per-vector-store write requests.
    pub file_id: String,
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct CreateVectorStoreRequest {
    ///The chunking strategy used to chunk the file(s). If not set, will use the `auto` strategy. Only applicable if `file_ids` is non-empty.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub chunking_strategy: ::std::option::Option<
        CreateVectorStoreRequestChunkingStrategy,
    >,
    ///A description for the vector store. Can be used to describe the vector store's purpose.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: ::std::option::Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expires_after: ::std::option::Option<VectorStoreExpirationAfter>,
    ///A list of [File](/docs/api-reference/files) IDs that the vector store should use. Useful for tools like `file_search` that can access files.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub file_ids: ::std::option::Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub metadata: ::std::option::Option<Metadata>,
    ///The name of the vector store.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: ::std::option::Option<String>,
}
///The chunking strategy used to chunk the file(s). If not set, will use the `auto` strategy. Only applicable if `file_ids` is non-empty.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
#[serde(untagged)]
pub enum CreateVectorStoreRequestChunkingStrategy {
    AutoChunkingStrategyRequestParam(AutoChunkingStrategyRequestParam),
    StaticChunkingStrategyRequestParam(StaticChunkingStrategyRequestParam),
}
///Parameters for creating a character from an uploaded video.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct CreateVideoCharacterBody {
    ///Display name for this API character.
    pub name: String,
    ///Video file used to create a character.
    pub video: OpenAiBinaryBody,
}
///JSON parameters for editing an existing generated video.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct CreateVideoEditJsonBody {
    ///Text prompt that describes how to edit the source video.
    pub prompt: String,
    ///Reference to the completed video to edit.
    pub video: VideoReferenceInputParam,
}
///Parameters for editing an existing generated video.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct CreateVideoEditMultipartBody {
    ///Text prompt that describes how to edit the source video.
    pub prompt: String,
    pub video: CreateVideoEditMultipartBodyVideo,
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
#[serde(untagged)]
pub enum CreateVideoEditMultipartBodyVideo {
    String(OpenAiBinaryBody),
    VideoReferenceInputParam(VideoReferenceInputParam),
}
///JSON parameters for extending an existing generated video.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct CreateVideoExtendJsonBody {
    ///Updated text prompt that directs the extension generation.
    pub prompt: String,
    ///Length of the newly generated extension segment in seconds (allowed values: 4, 8, 12, 16, 20).
    pub seconds: VideoSeconds,
    ///Reference to the completed video to extend.
    pub video: VideoReferenceInputParam,
}
///Multipart parameters for extending an existing generated video.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct CreateVideoExtendMultipartBody {
    ///Updated text prompt that directs the extension generation.
    pub prompt: String,
    ///Length of the newly generated extension segment in seconds (allowed values: 4, 8, 12, 16, 20).
    pub seconds: VideoSeconds,
    pub video: CreateVideoExtendMultipartBodyVideo,
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
#[serde(untagged)]
pub enum CreateVideoExtendMultipartBodyVideo {
    VideoReferenceInputParam(VideoReferenceInputParam),
    String(OpenAiBinaryBody),
}
///JSON parameters for creating a new video generation job.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct CreateVideoJsonBody {
    ///Optional reference object that guides generation. Provide exactly one of `image_url` or `file_id`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub input_reference: ::std::option::Option<ImageRefParam2>,
    ///The video generation model to use (allowed values: sora-2, sora-2-pro). Defaults to `sora-2`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model: ::std::option::Option<VideoModel>,
    ///Text prompt that describes the video to generate.
    pub prompt: String,
    ///Clip duration in seconds (allowed values: 4, 8, 12). Defaults to 4 seconds.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub seconds: ::std::option::Option<VideoSeconds>,
    ///Output resolution formatted as width x height (allowed values: 720x1280, 1280x720, 1024x1792, 1792x1024). Defaults to 720x1280.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub size: ::std::option::Option<VideoSize>,
}
///Multipart parameters for creating a new video generation job.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct CreateVideoMultipartBody {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub input_reference: ::std::option::Option<CreateVideoMultipartBodyInputReference>,
    ///The video generation model to use (allowed values: sora-2, sora-2-pro). Defaults to `sora-2`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model: ::std::option::Option<VideoModel>,
    ///Text prompt that describes the video to generate.
    pub prompt: String,
    ///Clip duration in seconds (allowed values: 4, 8, 12). Defaults to 4 seconds.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub seconds: ::std::option::Option<VideoSeconds>,
    ///Output resolution formatted as width x height (allowed values: 720x1280, 1280x720, 1024x1792, 1792x1024). Defaults to 720x1280.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub size: ::std::option::Option<VideoSize>,
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
#[serde(untagged)]
pub enum CreateVideoMultipartBodyInputReference {
    String(OpenAiBinaryBody),
    ImageRefParam2(ImageRefParam2),
}
///Parameters for remixing an existing generated video.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct CreateVideoRemixBody {
    ///Updated text prompt that directs the remix generation.
    pub prompt: String,
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct CreateVoiceConsentRequest {
    ///The BCP 47 language tag for the consent phrase (for example, `en-US`).
    pub language: String,
    ///The label to use for this consent recording.
    pub name: String,
    ///The consent audio recording file. Maximum size is 10 MiB. Supported MIME types: `audio/mpeg`, `audio/wav`, `audio/x-wav`, `audio/ogg`, `audio/aac`, `audio/flac`, `audio/webm`, `audio/mp4`.
    pub recording: OpenAiBinaryBody,
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct CreateVoiceRequest {
    ///The sample audio recording file. Maximum size is 10 MiB. Supported MIME types: `audio/mpeg`, `audio/wav`, `audio/x-wav`, `audio/ogg`, `audio/aac`, `audio/flac`, `audio/webm`, `audio/mp4`.
    pub audio_sample: OpenAiBinaryBody,
    ///The consent recording ID (for example, `cons_1234`).
    pub consent: String,
    ///The name of the new voice.
    pub name: String,
}
///A grammar defined by the user.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct CustomGrammarFormatParam {
    ///The grammar definition.
    pub definition: String,
    ///The syntax of the grammar definition. One of `lark` or `regex`.
    pub syntax: GrammarSyntax1,
    ///Grammar format. Always `grammar`.
    #[serde(rename = "type")]
    pub type_value: String,
}
///Unconstrained free-form text.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct CustomTextFormatParam {
    ///Unconstrained text format. Always `text`.
    #[serde(rename = "type")]
    pub type_value: String,
}
///A call to a custom tool created by the model.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct CustomToolCall {
    ///An identifier used to map this custom tool call to a tool call output.
    pub call_id: String,
    ///The unique ID of the custom tool call in the OpenAI platform.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: ::std::option::Option<String>,
    ///The input for the custom tool call generated by the model.
    pub input: String,
    ///The name of the custom tool being called.
    pub name: String,
    ///The namespace of the custom tool being called.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub namespace: ::std::option::Option<String>,
    ///The type of the custom tool call. Always `custom_tool_call`.
    #[serde(rename = "type")]
    pub type_value: String,
}
///The output of a custom tool call from your code, being sent back to the model.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct CustomToolCallOutput {
    ///The call ID, used to map this custom tool call output to a custom tool call.
    pub call_id: String,
    ///The unique ID of the custom tool call output in the OpenAI platform.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: ::std::option::Option<String>,
    ///The output from the custom tool call generated by your code. Can be a string or an list of output content.
    pub output: CustomToolCallOutputOutput,
    ///The type of the custom tool call output. Always `custom_tool_call_output`.
    #[serde(rename = "type")]
    pub type_value: String,
}
///The output from the custom tool call generated by your code. Can be a string or an list of output content.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
#[serde(untagged)]
pub enum CustomToolCallOutputOutput {
    StringOutput(String),
    OutputContentList(Vec<FunctionAndCustomToolCallOutput>),
}
///ResponseCustomToolCallOutputItem
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct CustomToolCallOutputResource {
    ///The call ID, used to map this custom tool call output to a custom tool call.
    pub call_id: String,
    ///The identifier of the actor that created the item.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub created_by: ::std::option::Option<String>,
    ///The unique ID of the custom tool call output in the OpenAI platform.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: ::std::option::Option<String>,
    ///The output from the custom tool call generated by your code. Can be a string or an list of output content.
    pub output: CustomToolCallOutputResourceOutput,
    ///The status of the item. One of `in_progress`, `completed`, or `incomplete`. Populated when items are returned via API.
    pub status: FunctionCallOutputStatusEnum,
    ///The type of the custom tool call output. Always `custom_tool_call_output`.
    #[serde(rename = "type")]
    pub type_value: String,
}
///The output from the custom tool call generated by your code. Can be a string or an list of output content.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
#[serde(untagged)]
pub enum CustomToolCallOutputResourceOutput {
    StringOutput(String),
    OutputContentList(Vec<FunctionAndCustomToolCallOutput>),
}
///ResponseCustomToolCallItem
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct CustomToolCallResource {
    ///An identifier used to map this custom tool call to a tool call output.
    pub call_id: String,
    ///The identifier of the actor that created the item.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub created_by: ::std::option::Option<String>,
    ///The unique ID of the custom tool call in the OpenAI platform.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: ::std::option::Option<String>,
    ///The input for the custom tool call generated by the model.
    pub input: String,
    ///The name of the custom tool being called.
    pub name: String,
    ///The namespace of the custom tool being called.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub namespace: ::std::option::Option<String>,
    ///The status of the item. One of `in_progress`, `completed`, or `incomplete`. Populated when items are returned via API.
    pub status: FunctionCallStatus,
    ///The type of the custom tool call. Always `custom_tool_call`.
    #[serde(rename = "type")]
    pub type_value: String,
}
///A custom tool that processes input using a specified format.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct CustomToolChatCompletions {
    ///Properties of the custom tool.
    pub custom: CustomToolChatCompletionsCustom,
    ///The type of the custom tool. Always `custom`.
    #[serde(rename = "type")]
    pub type_value: String,
}
///Properties of the custom tool.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct CustomToolChatCompletionsCustom {
    ///Optional description of the custom tool, used to provide more context.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: ::std::option::Option<String>,
    ///The input format for the custom tool. Default is unconstrained text.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub format: ::std::option::Option<CustomToolChatCompletionsCustomFormat3>,
    ///The name of the custom tool, used to identify it in tool calls.
    pub name: String,
}
///Unconstrained free-form text.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct CustomToolChatCompletionsCustomFormat {
    ///Unconstrained text format. Always `text`.
    #[serde(rename = "type")]
    pub type_value: String,
}
///A grammar defined by the user.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct CustomToolChatCompletionsCustomFormat2 {
    ///Your chosen grammar.
    pub grammar: CustomToolChatCompletionsCustomFormat2Grammar,
    ///Grammar format. Always `grammar`.
    #[serde(rename = "type")]
    pub type_value: String,
}
///Your chosen grammar.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct CustomToolChatCompletionsCustomFormat2Grammar {
    ///The grammar definition.
    pub definition: String,
    ///The syntax of the grammar definition. One of `lark` or `regex`.
    pub syntax: String,
}
///The input format for the custom tool. Default is unconstrained text.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
#[serde(untagged)]
pub enum CustomToolChatCompletionsCustomFormat3 {
    TextFormat(CustomToolChatCompletionsCustomFormat3TextFormat),
    GrammarFormat(CustomToolChatCompletionsCustomFormat3GrammarFormat),
}
///A grammar defined by the user.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct CustomToolChatCompletionsCustomFormat3GrammarFormat {
    ///Your chosen grammar.
    pub grammar: CustomToolChatCompletionsCustomFormat3GrammarFormatGrammar,
    ///Grammar format. Always `grammar`.
    #[serde(rename = "type")]
    pub type_value: String,
}
///Your chosen grammar.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct CustomToolChatCompletionsCustomFormat3GrammarFormatGrammar {
    ///The grammar definition.
    pub definition: String,
    ///The syntax of the grammar definition. One of `lark` or `regex`.
    pub syntax: String,
}
///Unconstrained free-form text.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct CustomToolChatCompletionsCustomFormat3TextFormat {
    ///Unconstrained text format. Always `text`.
    #[serde(rename = "type")]
    pub type_value: String,
}
///A custom tool that processes input using a specified format. Learn more about [custom tools](/docs/guides/function-calling#custom-tools)
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct CustomToolParam {
    ///Whether this tool should be deferred and discovered via tool search.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub defer_loading: ::std::option::Option<bool>,
    ///Optional description of the custom tool, used to provide more context.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: ::std::option::Option<String>,
    ///The input format for the custom tool. Default is unconstrained text.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub format: ::std::option::Option<CustomToolParamFormat>,
    ///The name of the custom tool, used to identify it in tool calls.
    pub name: String,
    ///The type of the custom tool. Always `custom`.
    #[serde(rename = "type")]
    pub type_value: String,
}
///The input format for the custom tool. Default is unconstrained text.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
#[serde(untagged)]
pub enum CustomToolParamFormat {
    CustomTextFormatParam(CustomTextFormatParam),
    CustomGrammarFormatParam(CustomGrammarFormatParam),
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct DeleteAssistantResponse {
    pub deleted: bool,
    pub id: String,
    pub object: String,
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct DeleteCertificateResponse {
    ///The ID of the certificate that was deleted.
    pub id: String,
    ///The object type, must be `certificate.deleted`.
    pub object: String,
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct DeleteEvalResponse {
    pub deleted: bool,
    pub eval_id: String,
    pub object: String,
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct DeleteEvalRunResponse {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub deleted: ::std::option::Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub object: ::std::option::Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub run_id: ::std::option::Option<String>,
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct DeleteFileResponse {
    pub deleted: bool,
    pub id: String,
    pub object: String,
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct DeleteFineTuningCheckpointPermissionResponse {
    ///Whether the fine-tuned model checkpoint permission was successfully deleted.
    pub deleted: bool,
    ///The ID of the fine-tuned model checkpoint permission that was deleted.
    pub id: String,
    ///The object type, which is always "checkpoint.permission".
    pub object: String,
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct DeleteMessageResponse {
    pub deleted: bool,
    pub id: String,
    pub object: String,
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct DeleteModelResponse {
    pub deleted: bool,
    pub id: String,
    pub object: String,
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct DeleteThreadResponse {
    pub deleted: bool,
    pub id: String,
    pub object: String,
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct DeleteVectorStoreFileResponse {
    pub deleted: bool,
    pub id: String,
    pub object: String,
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct DeleteVectorStoreResponse {
    pub deleted: bool,
    pub id: String,
    pub object: String,
}
///The deleted conversation object
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct DeletedConversation {
    pub deleted: bool,
    pub id: String,
    pub object: String,
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct DeletedConversationResource {
    pub deleted: bool,
    pub id: String,
    pub object: String,
}
///Confirmation payload returned after unassigning a role.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct DeletedRoleAssignmentResource {
    ///Whether the assignment was removed.
    pub deleted: bool,
    ///Identifier for the deleted assignment, such as `group.role.deleted` or `user.role.deleted`.
    pub object: String,
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct DeletedSkillResource {
    pub deleted: bool,
    pub id: String,
    pub object: String,
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct DeletedSkillVersionResource {
    pub deleted: bool,
    pub id: String,
    pub object: String,
    ///The deleted skill version.
    pub version: String,
}
///Confirmation payload returned after deleting a thread.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct DeletedThreadResource {
    ///Indicates that the thread has been deleted.
    pub deleted: bool,
    ///Identifier of the deleted thread.
    pub id: String,
    ///Type discriminator that is always `chatkit.thread.deleted`.
    pub object: String,
}
///Confirmation payload returned after deleting a video.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct DeletedVideoResource {
    ///Indicates that the video resource was deleted.
    pub deleted: bool,
    ///Identifier of the deleted video.
    pub id: String,
    ///The object type that signals the deletion response.
    pub object: String,
}
pub type DetailEnum = String;
///Occurs when a stream ends.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct DoneEvent {
    pub data: String,
    pub event: String,
}
///A double click action.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct DoubleClickAction {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub keys: ::std::option::Option<Vec<String>>,
    ///Specifies the event type. For a double click action, this property is always set to `double_click`.
    #[serde(rename = "type")]
    pub type_value: String,
    ///The x-coordinate where the double click occurred.
    pub x: i32,
    ///The y-coordinate where the double click occurred.
    pub y: i32,
}
///A drag action.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct DragParam {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub keys: ::std::option::Option<Vec<String>>,
    ///An array of coordinates representing the path of the drag action. Coordinates will appear as an array of objects, eg ``` [ { x: 100, y: 200 }, { x: 200, y: 300 } ] ```
    pub path: Vec<CoordParam>,
    ///Specifies the event type. For a drag action, this property is always set to `drag`.
    #[serde(rename = "type")]
    pub type_value: String,
}
///An x/y coordinate pair, e.g. `{ x: 100, y: 200 }`.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct DragPoint {
    ///The x-coordinate.
    pub x: i32,
    ///The y-coordinate.
    pub y: i32,
}
///A message input to the model with a role indicating instruction following hierarchy. Instructions given with the `developer` or `system` role take precedence over instructions given with the `user` role. Messages with the `assistant` role are presumed to have been generated by the model in previous interactions.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct EasyInputMessage {
    ///Text, image, or audio input to the model, used to generate a response. Can also contain previous assistant responses.
    pub content: EasyInputMessageContent,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub phase: ::std::option::Option<MessagePhase>,
    ///The role of the message input. One of `user`, `assistant`, `system`, or `developer`.
    pub role: String,
    ///The type of the message input. Always `message`.
    #[serde(rename = "type", default, skip_serializing_if = "Option::is_none")]
    pub type_value: ::std::option::Option<String>,
}
///Text, image, or audio input to the model, used to generate a response. Can also contain previous assistant responses.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
#[serde(untagged)]
pub enum EasyInputMessageContent {
    TextInput(String),
    InputMessageContentList(InputMessageContentList),
}
///JSON request body for image edits. Use `images` (array of `ImageRefParam`) instead of multipart `image` uploads. You can reference images via external URLs, data URLs, or uploaded file IDs. JSON edits support GPT image models only; DALL-E edits require multipart (`dall-e-2` only).
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct EditImageBodyJsonParam {
    ///Background behavior for generated image output.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub background: ::std::option::Option<String>,
    ///Input image references to edit. For GPT image models, you can provide up to 16 images.
    pub images: Vec<ImageRefParam>,
    ///Controls fidelity to the original input image(s).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub input_fidelity: ::std::option::Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mask: ::std::option::Option<ImageRefParam>,
    ///The model to use for image editing.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model: ::std::option::Option<String>,
    ///Moderation level for GPT image models.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub moderation: ::std::option::Option<String>,
    ///The number of edited images to generate.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub n: ::std::option::Option<i32>,
    ///Compression level for `jpeg` or `webp` output.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub output_compression: ::std::option::Option<i32>,
    ///Output image format. Supported for GPT image models.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub output_format: ::std::option::Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub partial_images: ::std::option::Option<PartialImages>,
    ///A text description of the desired image edit.
    pub prompt: String,
    ///Output quality for GPT image models.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub quality: ::std::option::Option<String>,
    ///Requested output image size.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub size: ::std::option::Option<String>,
    ///Stream partial image results as events.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stream: ::std::option::Option<bool>,
    ///A unique identifier representing your end-user, which can help OpenAI monitor and detect abuse.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub user: ::std::option::Option<String>,
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct EffectiveAtParameter {
    ///Return only events whose `effective_at` (Unix seconds) is greater than this value.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub gt: ::std::option::Option<i32>,
    ///Return only events whose `effective_at` (Unix seconds) is greater than or equal to this value.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub gte: ::std::option::Option<i32>,
    ///Return only events whose `effective_at` (Unix seconds) is less than this value.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub lt: ::std::option::Option<i32>,
    ///Return only events whose `effective_at` (Unix seconds) is less than or equal to this value.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub lte: ::std::option::Option<i32>,
}
///Represents an embedding vector returned by embedding endpoint.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct Embedding {
    ///The embedding vector, which is a list of floats. The length of vector depends on the model as listed in the [embedding guide](/docs/guides/embeddings).
    pub embedding: Vec<f32>,
    ///The index of the embedding in the list of embeddings.
    pub index: i32,
    ///The object type, which is always "embedding".
    pub object: String,
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct EmptyModelParam {
    #[serde(flatten)]
    pub value: OpenAiJsonObject,
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct Error {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub code: ::std::option::Option<String>,
    pub message: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub param: ::std::option::Option<String>,
    #[serde(rename = "type")]
    pub type_value: String,
}
///An error that occurred while generating the response.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct Error2 {
    ///A machine-readable error code that was returned.
    pub code: String,
    ///A human-readable description of the error that was returned.
    pub message: String,
}
///Occurs when an [error](/docs/guides/error-codes#api-errors) occurs. This can happen due to an internal server error or a timeout.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct ErrorEvent {
    pub data: Error,
    pub event: String,
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct ErrorResponse {
    pub error: Error,
}
///An Eval object with a data source config and testing criteria. An Eval represents a task to be done for your LLM integration. Like: - Improve the quality of my chatbot - See how well my chatbot handles customer support - Check if o4-mini is better at my usecase than gpt-4o
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct Eval {
    ///The Unix timestamp (in seconds) for when the eval was created.
    pub created_at: i64,
    ///Configuration of data sources used in runs of the evaluation.
    pub data_source_config: EvalDataSourceConfig,
    ///Unique identifier for the evaluation.
    pub id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub metadata: ::std::option::Option<Metadata>,
    ///The name of the evaluation.
    pub name: String,
    ///The object type.
    pub object: String,
    ///A list of testing criteria.
    pub testing_criteria: Vec<EvalTestingCriteriaItem>,
}
///An object representing an error response from the Eval API.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct EvalApiError {
    ///The error code.
    pub code: String,
    ///The error message.
    pub message: String,
}
///A CustomDataSourceConfig which specifies the schema of your `item` and optionally `sample` namespaces. The response schema defines the shape of the data that will be: - Used to define your testing criteria and - What data is required when creating a run
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct EvalCustomDataSourceConfig {
    ///The json schema for the run data source items. Learn how to build JSON schemas [here](https://json-schema.org/).
    pub schema: OpenAiJsonValue,
    ///The type of data source. Always `custom`.
    #[serde(rename = "type")]
    pub type_value: String,
}
///Configuration of data sources used in runs of the evaluation.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
#[serde(untagged)]
pub enum EvalDataSourceConfig {
    EvalCustomDataSourceConfig(EvalCustomDataSourceConfig),
    EvalLogsDataSourceConfig(EvalLogsDataSourceConfig),
    EvalStoredCompletionsDataSourceConfig(EvalStoredCompletionsDataSourceConfig),
}
///LabelModelGrader
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct EvalGraderLabelModel {
    pub input: Vec<EvalItem>,
    ///The labels to assign to each item in the evaluation.
    pub labels: Vec<String>,
    ///The model to use for the evaluation. Must support structured outputs.
    pub model: String,
    ///The name of the grader.
    pub name: String,
    ///The labels that indicate a passing result. Must be a subset of labels.
    pub passing_labels: Vec<String>,
    ///The object type, which is always `label_model`.
    #[serde(rename = "type")]
    pub type_value: String,
}
///PythonGrader
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct EvalGraderPython {
    ///The image tag to use for the python script.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub image_tag: ::std::option::Option<String>,
    ///The name of the grader.
    pub name: String,
    ///The threshold for the score.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pass_threshold: ::std::option::Option<f64>,
    ///The source code of the python script.
    pub source: String,
    ///The object type, which is always `python`.
    #[serde(rename = "type")]
    pub type_value: String,
}
///ScoreModelGrader
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct EvalGraderScoreModel {
    ///The input messages evaluated by the grader. Supports text, output text, input image, and input audio content blocks, and may include template strings.
    pub input: Vec<EvalItem>,
    ///The model to use for the evaluation.
    pub model: String,
    ///The name of the grader.
    pub name: String,
    ///The threshold for the score.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pass_threshold: ::std::option::Option<f64>,
    ///The range of the score. Defaults to `[0, 1]`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub range: ::std::option::Option<Vec<f64>>,
    ///The sampling parameters for the model.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sampling_params: ::std::option::Option<EvalGraderScoreModelSamplingParams>,
    ///The object type, which is always `score_model`.
    #[serde(rename = "type")]
    pub type_value: String,
}
///The sampling parameters for the model.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct EvalGraderScoreModelSamplingParams {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_completions_tokens: ::std::option::Option<i32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reasoning_effort: ::std::option::Option<ReasoningEffort>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub seed: ::std::option::Option<i32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub temperature: ::std::option::Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub top_p: ::std::option::Option<f64>,
}
///StringCheckGrader
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct EvalGraderStringCheck {
    ///The input text. This may include template strings.
    pub input: String,
    ///The name of the grader.
    pub name: String,
    ///The string check operation to perform. One of `eq`, `ne`, `like`, or `ilike`.
    pub operation: String,
    ///The reference text. This may include template strings.
    pub reference: String,
    ///The object type, which is always `string_check`.
    #[serde(rename = "type")]
    pub type_value: String,
}
///TextSimilarityGrader
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct EvalGraderTextSimilarity {
    ///The evaluation metric to use. One of `cosine`, `fuzzy_match`, `bleu`, `gleu`, `meteor`, `rouge_1`, `rouge_2`, `rouge_3`, `rouge_4`, `rouge_5`, or `rouge_l`.
    pub evaluation_metric: String,
    ///The text being graded.
    pub input: String,
    ///The name of the grader.
    pub name: String,
    ///The threshold for the score.
    pub pass_threshold: f64,
    ///The text being graded against.
    pub reference: String,
    ///The type of grader.
    #[serde(rename = "type")]
    pub type_value: String,
}
///A message input to the model with a role indicating instruction following hierarchy. Instructions given with the `developer` or `system` role take precedence over instructions given with the `user` role. Messages with the `assistant` role are presumed to have been generated by the model in previous interactions.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct EvalItem {
    pub content: EvalItemContent,
    ///The role of the message input. One of `user`, `assistant`, `system`, or `developer`.
    pub role: String,
    ///The type of the message input. Always `message`.
    #[serde(rename = "type", default, skip_serializing_if = "Option::is_none")]
    pub type_value: ::std::option::Option<String>,
}
///Inputs to the model - can contain template strings. Supports text, output text, input images, and input audio, either as a single item or an array of items.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
#[serde(untagged)]
pub enum EvalItemContent {
    EvalItemContentItem(EvalItemContentItem),
    EvalItemContentArray(EvalItemContentArray),
}
///A list of inputs, each of which may be either an input text, output text, input image, or input audio object.
pub type EvalItemContentArray = Vec<EvalItemContentItem>;
///A single content item: input text, output text, input image, or input audio.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
#[serde(untagged)]
pub enum EvalItemContentItem {
    EvalItemContentText(EvalItemContentText),
    InputTextContent(InputTextContent),
    EvalItemContentOutputText(EvalItemContentOutputText),
    EvalItemInputImage(EvalItemInputImage),
    InputAudio(InputAudio),
}
///A text output from the model.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct EvalItemContentOutputText {
    ///The text output from the model.
    pub text: String,
    ///The type of the output text. Always `output_text`.
    #[serde(rename = "type")]
    pub type_value: String,
}
///A text input to the model.
pub type EvalItemContentText = String;
///An image input block used within EvalItem content arrays.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct EvalItemInputImage {
    ///The detail level of the image to be sent to the model. One of `high`, `low`, or `auto`. Defaults to `auto`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub detail: ::std::option::Option<String>,
    ///The URL of the image input.
    pub image_url: String,
    ///The type of the image input. Always `input_image`.
    #[serde(rename = "type")]
    pub type_value: String,
}
///EvalJsonlFileContentSource
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct EvalJsonlFileContentSource {
    ///The content of the jsonl file.
    pub content: Vec<EvalJsonlFileContentSourceContentItem>,
    ///The type of jsonl source. Always `file_content`.
    #[serde(rename = "type")]
    pub type_value: String,
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct EvalJsonlFileContentSourceContentItem {
    pub item: OpenAiJsonValue,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sample: ::std::option::Option<OpenAiJsonValue>,
}
///EvalJsonlFileIdSource
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct EvalJsonlFileIdSource {
    ///The identifier of the file.
    pub id: String,
    ///The type of jsonl source. Always `file_id`.
    #[serde(rename = "type")]
    pub type_value: String,
}
///An object representing a list of evals.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct EvalList {
    ///An array of eval objects.
    pub data: Vec<Eval>,
    ///The identifier of the first eval in the data array.
    pub first_id: String,
    ///Indicates whether there are more evals available.
    pub has_more: bool,
    ///The identifier of the last eval in the data array.
    pub last_id: String,
    ///The type of this object. It is always set to "list".
    pub object: String,
}
///A LogsDataSourceConfig which specifies the metadata property of your logs query. This is usually metadata like `usecase=chatbot` or `prompt-version=v2`, etc. The schema returned by this data source config is used to defined what variables are available in your evals. `item` and `sample` are both defined when using this data source config.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct EvalLogsDataSourceConfig {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub metadata: ::std::option::Option<Metadata>,
    ///The json schema for the run data source items. Learn how to build JSON schemas [here](https://json-schema.org/).
    pub schema: OpenAiJsonValue,
    ///The type of data source. Always `logs`.
    #[serde(rename = "type")]
    pub type_value: String,
}
///A EvalResponsesSource object describing a run data source configuration.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct EvalResponsesSource {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub created_after: ::std::option::Option<i32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub created_before: ::std::option::Option<i32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub instructions_search: ::std::option::Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub metadata: ::std::option::Option<OpenAiJsonValue>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model: ::std::option::Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reasoning_effort: ::std::option::Option<ReasoningEffort>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub temperature: ::std::option::Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tools: ::std::option::Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub top_p: ::std::option::Option<f64>,
    ///The type of run data source. Always `responses`.
    #[serde(rename = "type")]
    pub type_value: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub users: ::std::option::Option<Vec<String>>,
}
///A schema representing an evaluation run.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct EvalRun {
    ///Unix timestamp (in seconds) when the evaluation run was created.
    pub created_at: i64,
    ///Information about the run's data source.
    pub data_source: EvalRunDataSource,
    pub error: EvalApiError,
    ///The identifier of the associated evaluation.
    pub eval_id: String,
    ///Unique identifier for the evaluation run.
    pub id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub metadata: ::std::option::Option<Metadata>,
    ///The model that is evaluated, if applicable.
    pub model: String,
    ///The name of the evaluation run.
    pub name: String,
    ///The type of the object. Always "eval.run".
    pub object: String,
    ///Usage statistics for each model during the evaluation run.
    pub per_model_usage: Vec<EvalRunPerModelUsageItem>,
    ///Results per testing criteria applied during the evaluation run.
    pub per_testing_criteria_results: Vec<EvalRunPerTestingCriteriaResult>,
    ///The URL to the rendered evaluation run report on the UI dashboard.
    pub report_url: String,
    ///Counters summarizing the outcomes of the evaluation run.
    pub result_counts: EvalRunResultCounts,
    ///The status of the evaluation run.
    pub status: String,
}
///Information about the run's data source.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
#[serde(untagged)]
pub enum EvalRunDataSource {
    CreateEvalJsonlRunDataSource(CreateEvalJsonlRunDataSource),
    CreateEvalCompletionsRunDataSource(CreateEvalCompletionsRunDataSource),
    CreateEvalResponsesRunDataSource(CreateEvalResponsesRunDataSource),
}
///An object representing a list of runs for an evaluation.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct EvalRunList {
    ///An array of eval run objects.
    pub data: Vec<EvalRun>,
    ///The identifier of the first eval run in the data array.
    pub first_id: String,
    ///Indicates whether there are more evals available.
    pub has_more: bool,
    ///The identifier of the last eval run in the data array.
    pub last_id: String,
    ///The type of this object. It is always set to "list".
    pub object: String,
}
///A schema representing an evaluation run output item.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct EvalRunOutputItem {
    ///Unix timestamp (in seconds) when the evaluation run was created.
    pub created_at: i64,
    ///Details of the input data source item.
    pub datasource_item: OpenAiJsonValue,
    ///The identifier for the data source item.
    pub datasource_item_id: i32,
    ///The identifier of the evaluation group.
    pub eval_id: String,
    ///Unique identifier for the evaluation run output item.
    pub id: String,
    ///The type of the object. Always "eval.run.output_item".
    pub object: String,
    ///A list of grader results for this output item.
    pub results: Vec<EvalRunOutputItemResult>,
    ///The identifier of the evaluation run associated with this output item.
    pub run_id: String,
    ///A sample containing the input and output of the evaluation run.
    pub sample: EvalRunOutputItemSample,
    ///The status of the evaluation run.
    pub status: String,
}
///An object representing a list of output items for an evaluation run.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct EvalRunOutputItemList {
    ///An array of eval run output item objects.
    pub data: Vec<EvalRunOutputItem>,
    ///The identifier of the first eval run output item in the data array.
    pub first_id: String,
    ///Indicates whether there are more eval run output items available.
    pub has_more: bool,
    ///The identifier of the last eval run output item in the data array.
    pub last_id: String,
    ///The type of this object. It is always set to "list".
    pub object: String,
}
///A single grader result for an evaluation run output item.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct EvalRunOutputItemResult {
    ///The name of the grader.
    pub name: String,
    ///Whether the grader considered the output a pass.
    pub passed: bool,
    ///Optional sample or intermediate data produced by the grader.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sample: ::std::option::Option<OpenAiJsonValue>,
    ///The numeric score produced by the grader.
    pub score: f64,
    ///The grader type (for example, "string-check-grader").
    #[serde(rename = "type", default, skip_serializing_if = "Option::is_none")]
    pub type_value: ::std::option::Option<String>,
}
///A sample containing the input and output of the evaluation run.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct EvalRunOutputItemSample {
    pub error: EvalApiError,
    ///The reason why the sample generation was finished.
    pub finish_reason: String,
    ///An array of input messages.
    pub input: Vec<EvalRunOutputItemSampleInputItem>,
    ///The maximum number of tokens allowed for completion.
    pub max_completion_tokens: i32,
    ///The model used for generating the sample.
    pub model: String,
    ///An array of output messages.
    pub output: Vec<EvalRunOutputItemSampleOutputItem>,
    ///The seed used for generating the sample.
    pub seed: i32,
    ///The sampling temperature used.
    pub temperature: f64,
    ///The top_p value used for sampling.
    pub top_p: f64,
    ///Token usage details for the sample.
    pub usage: EvalRunOutputItemSampleUsage,
}
///An input message.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct EvalRunOutputItemSampleInputItem {
    ///The content of the message.
    pub content: String,
    ///The role of the message sender (e.g., system, user, developer).
    pub role: String,
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct EvalRunOutputItemSampleOutputItem {
    ///The content of the message.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub content: ::std::option::Option<String>,
    ///The role of the message (e.g. "system", "assistant", "user").
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub role: ::std::option::Option<String>,
}
///Token usage details for the sample.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct EvalRunOutputItemSampleUsage {
    ///The number of tokens retrieved from cache.
    pub cached_tokens: i32,
    ///The number of completion tokens generated.
    pub completion_tokens: i32,
    ///The number of prompt tokens used.
    pub prompt_tokens: i32,
    ///The total number of tokens used.
    pub total_tokens: i32,
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct EvalRunPerModelUsageItem {
    ///The number of tokens retrieved from cache.
    pub cached_tokens: i32,
    ///The number of completion tokens generated.
    pub completion_tokens: i32,
    ///The number of invocations.
    pub invocation_count: i32,
    ///The name of the model.
    pub model_name: String,
    ///The number of prompt tokens used.
    pub prompt_tokens: i32,
    ///The total number of tokens used.
    pub total_tokens: i32,
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct EvalRunPerTestingCriteriaResult {
    ///Number of tests failed for this criteria.
    pub failed: i32,
    ///Number of tests passed for this criteria.
    pub passed: i32,
    ///A description of the testing criteria.
    pub testing_criteria: String,
}
///Counters summarizing the outcomes of the evaluation run.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct EvalRunResultCounts {
    ///Number of output items that resulted in an error.
    pub errored: i32,
    ///Number of output items that failed to pass the evaluation.
    pub failed: i32,
    ///Number of output items that passed the evaluation.
    pub passed: i32,
    ///Total number of executed output items.
    pub total: i32,
}
///Deprecated in favor of LogsDataSourceConfig.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct EvalStoredCompletionsDataSourceConfig {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub metadata: ::std::option::Option<Metadata>,
    ///The json schema for the run data source items. Learn how to build JSON schemas [here](https://json-schema.org/).
    pub schema: OpenAiJsonValue,
    ///The type of data source. Always `stored_completions`.
    #[serde(rename = "type")]
    pub type_value: String,
}
///A StoredCompletionsRunDataSource configuration describing a set of filters
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct EvalStoredCompletionsSource {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub created_after: ::std::option::Option<i32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub created_before: ::std::option::Option<i32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub limit: ::std::option::Option<i32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub metadata: ::std::option::Option<Metadata>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model: ::std::option::Option<String>,
    ///The type of source. Always `stored_completions`.
    #[serde(rename = "type")]
    pub type_value: String,
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
#[serde(untagged)]
pub enum EvalTestingCriteriaItem {
    EvalGraderLabelModel(EvalGraderLabelModel),
    EvalGraderStringCheck(EvalGraderStringCheck),
    EvalGraderTextSimilarity(EvalGraderTextSimilarity),
    EvalGraderPython(EvalGraderPython),
    EvalGraderScoreModel(EvalGraderScoreModel),
}
///Controls when the session expires relative to an anchor timestamp.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct ExpiresAfterParam {
    ///Base timestamp used to calculate expiration. Currently fixed to `created_at`.
    pub anchor: String,
    ///Number of seconds after the anchor when the session expires.
    pub seconds: i64,
}
///Annotation that references an uploaded file.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct FileAnnotation {
    ///File attachment referenced by the annotation.
    pub source: FileAnnotationSource,
    ///Type discriminator that is always `file` for this annotation.
    #[serde(rename = "type")]
    pub type_value: String,
}
///Attachment source referenced by an annotation.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct FileAnnotationSource {
    ///Filename referenced by the annotation.
    pub filename: String,
    ///Type discriminator that is always `file`.
    #[serde(rename = "type")]
    pub type_value: String,
}
///A citation to a file.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct FileCitationBody {
    ///The ID of the file.
    pub file_id: String,
    ///The filename of the file cited.
    pub filename: String,
    ///The index of the file in the list of files.
    pub index: i32,
    ///The type of the file citation. Always `file_citation`.
    #[serde(rename = "type")]
    pub type_value: String,
}
pub type FileDetailEnum = String;
///The expiration policy for a file. By default, files with `purpose=batch` expire after 30 days and all other files are persisted until they are manually deleted.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct FileExpirationAfter {
    ///Anchor timestamp after which the expiration policy applies. Supported anchors: `created_at`.
    pub anchor: String,
    ///The number of seconds after the anchor time that the file will expire. Must be between 3600 (1 hour) and 2592000 (30 days).
    pub seconds: i64,
}
pub type FileInputDetail = String;
///A path to a file.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct FilePath {
    ///The ID of the file.
    pub file_id: String,
    ///The index of the file in the list of files.
    pub index: i32,
    ///The type of the file path. Always `file_path`.
    #[serde(rename = "type")]
    pub type_value: String,
}
///The ranker to use for the file search. If not specified will use the `auto` ranker.
pub type FileSearchRanker = String;
///The ranking options for the file search. If not specified, the file search tool will use the `auto` ranker and a score_threshold of 0. See the [file search tool documentation](/docs/assistants/tools/file-search#customizing-file-search-settings) for more information.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct FileSearchRankingOptions {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ranker: ::std::option::Option<FileSearchRanker>,
    ///The score threshold for the file search. All values must be a floating point number between 0 and 1.
    pub score_threshold: f64,
}
///A tool that searches for relevant content from uploaded files. Learn more about the [file search tool](https://platform.openai.com/docs/guides/tools-file-search).
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct FileSearchTool {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub filters: ::std::option::Option<Filters>,
    ///The maximum number of results to return. This number should be between 1 and 50 inclusive.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_num_results: ::std::option::Option<i32>,
    ///Ranking options for search.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ranking_options: ::std::option::Option<RankingOptions>,
    ///The type of the file search tool. Always `file_search`.
    #[serde(rename = "type")]
    pub type_value: String,
    ///The IDs of the vector stores to search.
    pub vector_store_ids: Vec<String>,
}
///The results of a file search tool call. See the [file search guide](/docs/guides/tools-file-search) for more information.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct FileSearchToolCall {
    ///The unique ID of the file search tool call.
    pub id: String,
    ///The queries used to search for files.
    pub queries: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub results: ::std::option::Option<Vec<FileSearchToolCallResult>>,
    ///The status of the file search tool call. One of `in_progress`, `searching`, `incomplete` or `failed`,
    pub status: String,
    ///The type of the file search tool call. Always `file_search_call`.
    #[serde(rename = "type")]
    pub type_value: String,
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct FileSearchToolCallResult {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub attributes: ::std::option::Option<VectorStoreFileAttributes>,
    ///The unique ID of the file.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub file_id: ::std::option::Option<String>,
    ///The name of the file.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub filename: ::std::option::Option<String>,
    ///The relevance score of the file - a value between 0 and 1.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub score: ::std::option::Option<f32>,
    ///The text that was retrieved from the file.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub text: ::std::option::Option<String>,
}
///Controls whether users can upload files.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct FileUploadParam {
    ///Enable uploads for this session. Defaults to false.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub enabled: ::std::option::Option<bool>,
    ///Maximum size in megabytes for each uploaded file. Defaults to 512 MB, which is the maximum allowable size.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_file_size: ::std::option::Option<i32>,
    ///Maximum number of files that can be uploaded to the session. Defaults to 10.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_files: ::std::option::Option<i32>,
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
#[serde(untagged)]
pub enum Filters {
    ComparisonFilter(ComparisonFilter),
    CompoundFilter(CompoundFilter),
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct FineTuneChatCompletionRequestAssistantMessage {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub audio: ::std::option::Option<FineTuneChatCompletionRequestAssistantMessageAudio>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub content: ::std::option::Option<
        FineTuneChatCompletionRequestAssistantMessageContent,
    >,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub function_call: ::std::option::Option<
        FineTuneChatCompletionRequestAssistantMessageFunctionCall,
    >,
    ///An optional name for the participant. Provides the model information to differentiate between participants of the same role.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: ::std::option::Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub refusal: ::std::option::Option<String>,
    ///The role of the messages author, in this case `assistant`.
    pub role: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tool_calls: ::std::option::Option<ChatCompletionMessageToolCalls>,
    ///Controls whether the assistant message is trained against (0 or 1)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub weight: ::std::option::Option<i32>,
}
///Data about a previous audio response from the model. [Learn more](/docs/guides/audio).
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct FineTuneChatCompletionRequestAssistantMessageAudio {
    ///Unique identifier for a previous audio response from the model.
    pub id: String,
}
///The contents of the assistant message. Required unless `tool_calls` or `function_call` is specified.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
#[serde(untagged)]
pub enum FineTuneChatCompletionRequestAssistantMessageContent {
    TextContent(String),
    ArrayOfContentParts(Vec<ChatCompletionRequestAssistantMessageContentPart>),
}
///Deprecated and replaced by `tool_calls`. The name and arguments of a function that should be called, as generated by the model.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct FineTuneChatCompletionRequestAssistantMessageFunctionCall {
    ///The arguments to call the function with, as generated by the model in JSON format. Note that the model does not always generate valid JSON, and may hallucinate parameters not defined by your function schema. Validate the arguments in your code before calling your function.
    pub arguments: String,
    ///The name of the function to call.
    pub name: String,
}
///The hyperparameters used for the DPO fine-tuning job.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct FineTuneDpoHyperparameters {
    ///Number of examples in each batch. A larger batch size means that model parameters are updated less frequently, but with lower variance.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub batch_size: ::std::option::Option<FineTuneDpoHyperparametersBatchSize>,
    ///The beta value for the DPO method. A higher beta value will increase the weight of the penalty between the policy and reference model.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub beta: ::std::option::Option<FineTuneDpoHyperparametersBeta>,
    ///Scaling factor for the learning rate. A smaller learning rate may be useful to avoid overfitting.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub learning_rate_multiplier: ::std::option::Option<
        FineTuneDpoHyperparametersLearningRateMultiplier,
    >,
    ///The number of epochs to train the model for. An epoch refers to one full cycle through the training dataset.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub n_epochs: ::std::option::Option<FineTuneDpoHyperparametersNEpochs>,
}
///Number of examples in each batch. A larger batch size means that model parameters are updated less frequently, but with lower variance.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
#[serde(untagged)]
pub enum FineTuneDpoHyperparametersBatchSize {
    Auto(String),
    Integer(i32),
}
///The beta value for the DPO method. A higher beta value will increase the weight of the penalty between the policy and reference model.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
#[serde(untagged)]
pub enum FineTuneDpoHyperparametersBeta {
    Auto(String),
    Number(f64),
}
///Scaling factor for the learning rate. A smaller learning rate may be useful to avoid overfitting.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
#[serde(untagged)]
pub enum FineTuneDpoHyperparametersLearningRateMultiplier {
    Auto(String),
    Number(f64),
}
///The number of epochs to train the model for. An epoch refers to one full cycle through the training dataset.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
#[serde(untagged)]
pub enum FineTuneDpoHyperparametersNEpochs {
    Auto(String),
    Integer(i32),
}
///Configuration for the DPO fine-tuning method.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct FineTuneDpoMethod {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hyperparameters: ::std::option::Option<FineTuneDpoHyperparameters>,
}
///The method used for fine-tuning.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct FineTuneMethod {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dpo: ::std::option::Option<FineTuneDpoMethod>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reinforcement: ::std::option::Option<FineTuneReinforcementMethod>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub supervised: ::std::option::Option<FineTuneSupervisedMethod>,
    ///The type of method. Is either `supervised`, `dpo`, or `reinforcement`.
    #[serde(rename = "type")]
    pub type_value: String,
}
///The hyperparameters used for the reinforcement fine-tuning job.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct FineTuneReinforcementHyperparameters {
    ///Number of examples in each batch. A larger batch size means that model parameters are updated less frequently, but with lower variance.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub batch_size: ::std::option::Option<FineTuneReinforcementHyperparametersBatchSize>,
    ///Multiplier on amount of compute used for exploring search space during training.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub compute_multiplier: ::std::option::Option<
        FineTuneReinforcementHyperparametersComputeMultiplier,
    >,
    ///The number of training steps between evaluation runs.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub eval_interval: ::std::option::Option<
        FineTuneReinforcementHyperparametersEvalInterval,
    >,
    ///Number of evaluation samples to generate per training step.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub eval_samples: ::std::option::Option<
        FineTuneReinforcementHyperparametersEvalSamples,
    >,
    ///Scaling factor for the learning rate. A smaller learning rate may be useful to avoid overfitting.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub learning_rate_multiplier: ::std::option::Option<
        FineTuneReinforcementHyperparametersLearningRateMultiplier,
    >,
    ///The number of epochs to train the model for. An epoch refers to one full cycle through the training dataset.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub n_epochs: ::std::option::Option<FineTuneReinforcementHyperparametersNEpochs>,
    ///Level of reasoning effort.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reasoning_effort: ::std::option::Option<String>,
}
///Number of examples in each batch. A larger batch size means that model parameters are updated less frequently, but with lower variance.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
#[serde(untagged)]
pub enum FineTuneReinforcementHyperparametersBatchSize {
    Auto(String),
    Integer(i32),
}
///Multiplier on amount of compute used for exploring search space during training.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
#[serde(untagged)]
pub enum FineTuneReinforcementHyperparametersComputeMultiplier {
    Auto(String),
    Number(f64),
}
///The number of training steps between evaluation runs.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
#[serde(untagged)]
pub enum FineTuneReinforcementHyperparametersEvalInterval {
    Auto(String),
    Integer(i32),
}
///Number of evaluation samples to generate per training step.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
#[serde(untagged)]
pub enum FineTuneReinforcementHyperparametersEvalSamples {
    Auto(String),
    Integer(i32),
}
///Scaling factor for the learning rate. A smaller learning rate may be useful to avoid overfitting.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
#[serde(untagged)]
pub enum FineTuneReinforcementHyperparametersLearningRateMultiplier {
    Auto(String),
    Number(f64),
}
///The number of epochs to train the model for. An epoch refers to one full cycle through the training dataset.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
#[serde(untagged)]
pub enum FineTuneReinforcementHyperparametersNEpochs {
    Auto(String),
    Integer(i32),
}
///Configuration for the reinforcement fine-tuning method.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct FineTuneReinforcementMethod {
    ///The grader used for the fine-tuning job.
    pub grader: FineTuneReinforcementMethodGrader,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hyperparameters: ::std::option::Option<FineTuneReinforcementHyperparameters>,
}
///The grader used for the fine-tuning job.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
#[serde(untagged)]
pub enum FineTuneReinforcementMethodGrader {
    GraderStringCheck(GraderStringCheck),
    GraderTextSimilarity(GraderTextSimilarity),
    GraderPython(GraderPython),
    GraderScoreModel(GraderScoreModel),
    GraderMulti(GraderMulti),
}
///The hyperparameters used for the fine-tuning job.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct FineTuneSupervisedHyperparameters {
    ///Number of examples in each batch. A larger batch size means that model parameters are updated less frequently, but with lower variance.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub batch_size: ::std::option::Option<FineTuneSupervisedHyperparametersBatchSize>,
    ///Scaling factor for the learning rate. A smaller learning rate may be useful to avoid overfitting.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub learning_rate_multiplier: ::std::option::Option<
        FineTuneSupervisedHyperparametersLearningRateMultiplier,
    >,
    ///The number of epochs to train the model for. An epoch refers to one full cycle through the training dataset.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub n_epochs: ::std::option::Option<FineTuneSupervisedHyperparametersNEpochs>,
}
///Number of examples in each batch. A larger batch size means that model parameters are updated less frequently, but with lower variance.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
#[serde(untagged)]
pub enum FineTuneSupervisedHyperparametersBatchSize {
    Auto(String),
    Integer(i32),
}
///Scaling factor for the learning rate. A smaller learning rate may be useful to avoid overfitting.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
#[serde(untagged)]
pub enum FineTuneSupervisedHyperparametersLearningRateMultiplier {
    Auto(String),
    Number(f64),
}
///The number of epochs to train the model for. An epoch refers to one full cycle through the training dataset.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
#[serde(untagged)]
pub enum FineTuneSupervisedHyperparametersNEpochs {
    Auto(String),
    Integer(i32),
}
///Configuration for the supervised fine-tuning method.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct FineTuneSupervisedMethod {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hyperparameters: ::std::option::Option<FineTuneSupervisedHyperparameters>,
}
///The `checkpoint.permission` object represents a permission for a fine-tuned model checkpoint.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct FineTuningCheckpointPermission {
    ///The Unix timestamp (in seconds) for when the permission was created.
    pub created_at: i64,
    ///The permission identifier, which can be referenced in the API endpoints.
    pub id: String,
    ///The object type, which is always "checkpoint.permission".
    pub object: String,
    ///The project identifier that the permission is for.
    pub project_id: String,
}
///Fine-Tuning Job Integration
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct FineTuningIntegration {
    ///The type of the integration being enabled for the fine-tuning job
    #[serde(rename = "type")]
    pub type_value: String,
    ///The settings for your integration with Weights and Biases. This payload specifies the project that metrics will be sent to. Optionally, you can set an explicit display name for your run, add tags to your run, and set a default entity (team, username, etc) to be associated with your run.
    pub wandb: FineTuningIntegrationWandb,
}
///The settings for your integration with Weights and Biases. This payload specifies the project that metrics will be sent to. Optionally, you can set an explicit display name for your run, add tags to your run, and set a default entity (team, username, etc) to be associated with your run.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct FineTuningIntegrationWandb {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub entity: ::std::option::Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: ::std::option::Option<String>,
    ///The name of the project that the new run will be created under.
    pub project: String,
    ///A list of tags to be attached to the newly created run. These tags are passed through directly to WandB. Some default tags are generated by OpenAI: "openai/finetune", "openai/{base-model}", "openai/{ftjob-abcdef}".
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tags: ::std::option::Option<Vec<String>>,
}
///The `fine_tuning.job` object represents a fine-tuning job that has been created through the API.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct FineTuningJob {
    ///The Unix timestamp (in seconds) for when the fine-tuning job was created.
    pub created_at: i64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: ::std::option::Option<FineTuningJobError>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub estimated_finish: ::std::option::Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fine_tuned_model: ::std::option::Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub finished_at: ::std::option::Option<i64>,
    ///The hyperparameters used for the fine-tuning job. This value will only be returned when running `supervised` jobs.
    pub hyperparameters: FineTuningJobHyperparameters,
    ///The object identifier, which can be referenced in the API endpoints.
    pub id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub integrations: ::std::option::Option<Vec<FineTuningIntegration>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub metadata: ::std::option::Option<Metadata>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub method: ::std::option::Option<FineTuneMethod>,
    ///The base model that is being fine-tuned.
    pub model: String,
    ///The object type, which is always "fine_tuning.job".
    pub object: String,
    ///The organization that owns the fine-tuning job.
    pub organization_id: String,
    ///The compiled results file ID(s) for the fine-tuning job. You can retrieve the results with the [Files API](/docs/api-reference/files/retrieve-contents).
    pub result_files: Vec<String>,
    ///The seed used for the fine-tuning job.
    pub seed: i32,
    ///The current status of the fine-tuning job, which can be either `validating_files`, `queued`, `running`, `succeeded`, `failed`, or `cancelled`.
    pub status: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub trained_tokens: ::std::option::Option<i32>,
    ///The file ID used for training. You can retrieve the training data with the [Files API](/docs/api-reference/files/retrieve-contents).
    pub training_file: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub validation_file: ::std::option::Option<String>,
}
///The `fine_tuning.job.checkpoint` object represents a model checkpoint for a fine-tuning job that is ready to use.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct FineTuningJobCheckpoint {
    ///The Unix timestamp (in seconds) for when the checkpoint was created.
    pub created_at: i64,
    ///The name of the fine-tuned checkpoint model that is created.
    pub fine_tuned_model_checkpoint: String,
    ///The name of the fine-tuning job that this checkpoint was created from.
    pub fine_tuning_job_id: String,
    ///The checkpoint identifier, which can be referenced in the API endpoints.
    pub id: String,
    ///Metrics at the step number during the fine-tuning job.
    pub metrics: FineTuningJobCheckpointMetrics,
    ///The object type, which is always "fine_tuning.job.checkpoint".
    pub object: String,
    ///The step number that the checkpoint was created at.
    pub step_number: i32,
}
///Metrics at the step number during the fine-tuning job.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct FineTuningJobCheckpointMetrics {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub full_valid_loss: ::std::option::Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub full_valid_mean_token_accuracy: ::std::option::Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub step: ::std::option::Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub train_loss: ::std::option::Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub train_mean_token_accuracy: ::std::option::Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub valid_loss: ::std::option::Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub valid_mean_token_accuracy: ::std::option::Option<f64>,
}
///For fine-tuning jobs that have `failed`, this will contain more information on the cause of the failure.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct FineTuningJobError {
    ///A machine-readable error code.
    pub code: String,
    ///A human-readable error message.
    pub message: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub param: ::std::option::Option<String>,
}
///Fine-tuning job event object
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct FineTuningJobEvent {
    ///The Unix timestamp (in seconds) for when the fine-tuning job was created.
    pub created_at: i64,
    ///The data associated with the event.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub data: ::std::option::Option<OpenAiJsonValue>,
    ///The object identifier.
    pub id: String,
    ///The log level of the event.
    pub level: String,
    ///The message of the event.
    pub message: String,
    ///The object type, which is always "fine_tuning.job.event".
    pub object: String,
    ///The type of event.
    #[serde(rename = "type", default, skip_serializing_if = "Option::is_none")]
    pub type_value: ::std::option::Option<String>,
}
///The hyperparameters used for the fine-tuning job. This value will only be returned when running `supervised` jobs.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct FineTuningJobHyperparameters {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub batch_size: ::std::option::Option<FineTuningJobHyperparametersBatchSize>,
    ///Scaling factor for the learning rate. A smaller learning rate may be useful to avoid overfitting.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub learning_rate_multiplier: ::std::option::Option<
        FineTuningJobHyperparametersLearningRateMultiplier,
    >,
    ///The number of epochs to train the model for. An epoch refers to one full cycle through the training dataset.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub n_epochs: ::std::option::Option<FineTuningJobHyperparametersNEpochs>,
}
///Number of examples in each batch. A larger batch size means that model parameters are updated less frequently, but with lower variance.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
#[serde(untagged)]
pub enum FineTuningJobHyperparametersBatchSize {
    Auto(String),
    Integer(i32),
}
///Scaling factor for the learning rate. A smaller learning rate may be useful to avoid overfitting.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
#[serde(untagged)]
pub enum FineTuningJobHyperparametersLearningRateMultiplier {
    Auto(String),
    Number(f64),
}
///The number of epochs to train the model for. An epoch refers to one full cycle through the training dataset.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
#[serde(untagged)]
pub enum FineTuningJobHyperparametersNEpochs {
    Auto(String),
    Integer(i32),
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
#[serde(untagged)]
pub enum FunctionAndCustomToolCallOutput {
    InputTextContent(InputTextContent),
    InputImageContent(InputImageContent),
    InputFileContent(InputFileContent),
}
pub type FunctionCallItemStatus = String;
///The output of a function tool call.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct FunctionCallOutputItemParam {
    ///The unique ID of the function tool call generated by the model.
    pub call_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: ::std::option::Option<String>,
    ///Text, image, or file output of the function tool call.
    pub output: FunctionCallOutputItemParamOutput,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub status: ::std::option::Option<FunctionCallItemStatus>,
    ///The type of the function tool call output. Always `function_call_output`.
    #[serde(rename = "type")]
    pub type_value: String,
}
///Text, image, or file output of the function tool call.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
#[serde(untagged)]
pub enum FunctionCallOutputItemParamOutput {
    String(String),
    Array(Vec<FunctionCallOutputItemParamOutputArrayItem>),
}
///A piece of message content, such as text, an image, or a file.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
#[serde(untagged)]
pub enum FunctionCallOutputItemParamOutputArrayItem {
    InputTextContentParam(InputTextContentParam),
    InputImageContentParamAutoParam(InputImageContentParamAutoParam),
    InputFileContentParam(InputFileContentParam),
}
///A piece of message content, such as text, an image, or a file.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
#[serde(untagged)]
pub enum FunctionCallOutputItemParamOutputItem {
    InputTextContentParam(InputTextContentParam),
    InputImageContentParamAutoParam(InputImageContentParamAutoParam),
    InputFileContentParam(InputFileContentParam),
}
pub type FunctionCallOutputStatusEnum = String;
pub type FunctionCallStatus = String;
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct FunctionObject {
    ///A description of what the function does, used by the model to choose when and how to call the function.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: ::std::option::Option<String>,
    ///The name of the function to be called. Must be a-z, A-Z, 0-9, or contain underscores and dashes, with a maximum length of 64.
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parameters: ::std::option::Option<FunctionParameters>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub strict: ::std::option::Option<bool>,
}
///The parameters the functions accepts, described as a JSON Schema object. See the [guide](/docs/guides/function-calling) for examples, and the [JSON Schema reference](https://json-schema.org/understanding-json-schema/) for documentation about the format. Omitting `parameters` defines a function with an empty parameter list.
pub type FunctionParameters = OpenAiJsonValue;
///Execute a shell command.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct FunctionShellAction {
    pub commands: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_output_length: ::std::option::Option<i32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub timeout_ms: ::std::option::Option<i32>,
}
///Commands and limits describing how to run the shell tool call.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct FunctionShellActionParam {
    ///Ordered shell commands for the execution environment to run.
    pub commands: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_output_length: ::std::option::Option<i32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub timeout_ms: ::std::option::Option<i32>,
}
///A tool call that executes one or more shell commands in a managed environment.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct FunctionShellCall {
    ///The shell commands and limits that describe how to run the tool call.
    pub action: FunctionShellAction,
    ///The unique ID of the shell tool call generated by the model.
    pub call_id: String,
    ///The ID of the entity that created this tool call.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub created_by: ::std::option::Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub environment: ::std::option::Option<FunctionShellCallEnvironment>,
    ///The unique ID of the shell tool call. Populated when this item is returned via API.
    pub id: String,
    ///The status of the shell call. One of `in_progress`, `completed`, or `incomplete`.
    pub status: FunctionShellCallStatus,
    ///The type of the item. Always `shell_call`.
    #[serde(rename = "type")]
    pub type_value: String,
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
#[serde(untagged)]
pub enum FunctionShellCallEnvironment {
    LocalEnvironmentResource(LocalEnvironmentResource),
    ContainerReferenceResource(ContainerReferenceResource),
}
///A tool representing a request to execute one or more shell commands.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct FunctionShellCallItemParam {
    ///The shell commands and limits that describe how to run the tool call.
    pub action: FunctionShellActionParam,
    ///The unique ID of the shell tool call generated by the model.
    pub call_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub environment: ::std::option::Option<FunctionShellCallItemParamEnvironment>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: ::std::option::Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub status: ::std::option::Option<FunctionShellCallItemStatus>,
    ///The type of the item. Always `shell_call`.
    #[serde(rename = "type")]
    pub type_value: String,
}
///The environment to execute the shell commands in.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
#[serde(untagged)]
pub enum FunctionShellCallItemParamEnvironment {
    LocalEnvironmentParam(LocalEnvironmentParam),
    ContainerReferenceParam(ContainerReferenceParam),
}
///Status values reported for shell tool calls.
pub type FunctionShellCallItemStatus = String;
///The output of a shell tool call that was emitted.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct FunctionShellCallOutput {
    ///The unique ID of the shell tool call generated by the model.
    pub call_id: String,
    ///The identifier of the actor that created the item.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub created_by: ::std::option::Option<String>,
    ///The unique ID of the shell call output. Populated when this item is returned via API.
    pub id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_output_length: ::std::option::Option<i32>,
    ///An array of shell call output contents
    pub output: Vec<FunctionShellCallOutputContent>,
    ///The status of the shell call output. One of `in_progress`, `completed`, or `incomplete`.
    pub status: FunctionShellCallOutputStatusEnum,
    ///The type of the shell call output. Always `shell_call_output`.
    #[serde(rename = "type")]
    pub type_value: String,
}
///The content of a shell tool call output that was emitted.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct FunctionShellCallOutputContent {
    ///The identifier of the actor that created the item.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub created_by: ::std::option::Option<String>,
    ///Represents either an exit outcome (with an exit code) or a timeout outcome for a shell call output chunk.
    pub outcome: FunctionShellCallOutputContentOutcome,
    ///The standard error output that was captured.
    pub stderr: String,
    ///The standard output that was captured.
    pub stdout: String,
}
///Represents either an exit outcome (with an exit code) or a timeout outcome for a shell call output chunk.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
#[serde(untagged)]
pub enum FunctionShellCallOutputContentOutcome {
    FunctionShellCallOutputTimeoutOutcome(FunctionShellCallOutputTimeoutOutcome),
    FunctionShellCallOutputExitOutcome(FunctionShellCallOutputExitOutcome),
}
///Captured stdout and stderr for a portion of a shell tool call output.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct FunctionShellCallOutputContentParam {
    ///The exit or timeout outcome associated with this shell call.
    pub outcome: FunctionShellCallOutputOutcomeParam,
    ///Captured stderr output for the shell call.
    pub stderr: String,
    ///Captured stdout output for the shell call.
    pub stdout: String,
}
///Indicates that the shell commands finished and returned an exit code.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct FunctionShellCallOutputExitOutcome {
    ///Exit code from the shell process.
    pub exit_code: i32,
    ///The outcome type. Always `exit`.
    #[serde(rename = "type")]
    pub type_value: String,
}
///Indicates that the shell commands finished and returned an exit code.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct FunctionShellCallOutputExitOutcomeParam {
    ///The exit code returned by the shell process.
    pub exit_code: i32,
    ///The outcome type. Always `exit`.
    #[serde(rename = "type")]
    pub type_value: String,
}
///The streamed output items emitted by a shell tool call.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct FunctionShellCallOutputItemParam {
    ///The unique ID of the shell tool call generated by the model.
    pub call_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: ::std::option::Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_output_length: ::std::option::Option<i32>,
    ///Captured chunks of stdout and stderr output, along with their associated outcomes.
    pub output: Vec<FunctionShellCallOutputContentParam>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub status: ::std::option::Option<FunctionShellCallItemStatus>,
    ///The type of the item. Always `shell_call_output`.
    #[serde(rename = "type")]
    pub type_value: String,
}
///The exit or timeout outcome associated with this shell call.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
#[serde(untagged)]
pub enum FunctionShellCallOutputOutcomeParam {
    FunctionShellCallOutputTimeoutOutcomeParam(
        FunctionShellCallOutputTimeoutOutcomeParam,
    ),
    FunctionShellCallOutputExitOutcomeParam(FunctionShellCallOutputExitOutcomeParam),
}
pub type FunctionShellCallOutputStatusEnum = String;
///Indicates that the shell call exceeded its configured time limit.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct FunctionShellCallOutputTimeoutOutcome {
    ///The outcome type. Always `timeout`.
    #[serde(rename = "type")]
    pub type_value: String,
}
///Indicates that the shell call exceeded its configured time limit.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct FunctionShellCallOutputTimeoutOutcomeParam {
    ///The outcome type. Always `timeout`.
    #[serde(rename = "type")]
    pub type_value: String,
}
pub type FunctionShellCallStatus = String;
///A tool that allows the model to execute shell commands.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct FunctionShellToolParam {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub environment: ::std::option::Option<FunctionShellToolParamEnvironment>,
    ///The type of the shell tool. Always `shell`.
    #[serde(rename = "type")]
    pub type_value: String,
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
#[serde(untagged)]
pub enum FunctionShellToolParamEnvironment {
    ContainerAutoParam(ContainerAutoParam),
    LocalEnvironmentParam(LocalEnvironmentParam),
    ContainerReferenceParam(ContainerReferenceParam),
}
///Defines a function in your own code the model can choose to call. Learn more about [function calling](https://platform.openai.com/docs/guides/function-calling).
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct FunctionTool {
    ///Whether this function is deferred and loaded via tool search.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub defer_loading: ::std::option::Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: ::std::option::Option<String>,
    ///The name of the function to call.
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parameters: ::std::option::Option<OpenAiJsonValue>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub strict: ::std::option::Option<bool>,
    ///The type of the function tool. Always `function`.
    #[serde(rename = "type")]
    pub type_value: String,
}
///A tool call to run a function. See the [function calling guide](/docs/guides/function-calling) for more information.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct FunctionToolCall {
    ///A JSON string of the arguments to pass to the function.
    pub arguments: String,
    ///The unique ID of the function tool call generated by the model.
    pub call_id: String,
    ///The unique ID of the function tool call.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: ::std::option::Option<String>,
    ///The name of the function to run.
    pub name: String,
    ///The namespace of the function to run.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub namespace: ::std::option::Option<String>,
    ///The status of the item. One of `in_progress`, `completed`, or `incomplete`. Populated when items are returned via API.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub status: ::std::option::Option<String>,
    ///The type of the function tool call. Always `function_call`.
    #[serde(rename = "type")]
    pub type_value: String,
}
///The output of a function tool call.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct FunctionToolCallOutput {
    ///The unique ID of the function tool call generated by the model.
    pub call_id: String,
    ///The unique ID of the function tool call output. Populated when this item is returned via API.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: ::std::option::Option<String>,
    ///The output from the function call generated by your code. Can be a string or an list of output content.
    pub output: FunctionToolCallOutputOutput,
    ///The status of the item. One of `in_progress`, `completed`, or `incomplete`. Populated when items are returned via API.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub status: ::std::option::Option<String>,
    ///The type of the function tool call output. Always `function_call_output`.
    #[serde(rename = "type")]
    pub type_value: String,
}
///The output from the function call generated by your code. Can be a string or an list of output content.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
#[serde(untagged)]
pub enum FunctionToolCallOutputOutput {
    StringOutput(String),
    OutputContentList(Vec<FunctionAndCustomToolCallOutput>),
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct FunctionToolCallOutputResource {
    ///The unique ID of the function tool call generated by the model.
    pub call_id: String,
    ///The identifier of the actor that created the item.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub created_by: ::std::option::Option<String>,
    ///The unique ID of the function tool call output. Populated when this item is returned via API.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: ::std::option::Option<String>,
    ///The output from the function call generated by your code. Can be a string or an list of output content.
    pub output: FunctionToolCallOutputResourceOutput,
    ///The status of the item. One of `in_progress`, `completed`, or `incomplete`. Populated when items are returned via API.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub status: ::std::option::Option<String>,
    ///The type of the function tool call output. Always `function_call_output`.
    #[serde(rename = "type")]
    pub type_value: String,
}
///The output from the function call generated by your code. Can be a string or an list of output content.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
#[serde(untagged)]
pub enum FunctionToolCallOutputResourceOutput {
    StringOutput(String),
    OutputContentList(Vec<FunctionAndCustomToolCallOutput>),
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct FunctionToolCallResource {
    ///A JSON string of the arguments to pass to the function.
    pub arguments: String,
    ///The unique ID of the function tool call generated by the model.
    pub call_id: String,
    ///The identifier of the actor that created the item.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub created_by: ::std::option::Option<String>,
    ///The unique ID of the function tool call.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: ::std::option::Option<String>,
    ///The name of the function to run.
    pub name: String,
    ///The namespace of the function to run.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub namespace: ::std::option::Option<String>,
    ///The status of the item. One of `in_progress`, `completed`, or `incomplete`. Populated when items are returned via API.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub status: ::std::option::Option<String>,
    ///The type of the function tool call. Always `function_call`.
    #[serde(rename = "type")]
    pub type_value: String,
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct FunctionToolParam {
    ///Whether this function should be deferred and discovered via tool search.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub defer_loading: ::std::option::Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: ::std::option::Option<String>,
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parameters: ::std::option::Option<EmptyModelParam>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub strict: ::std::option::Option<bool>,
    #[serde(rename = "type")]
    pub type_value: String,
}
///A LabelModelGrader object which uses a model to assign labels to each item in the evaluation.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct GraderLabelModel {
    pub input: Vec<EvalItem>,
    ///The labels to assign to each item in the evaluation.
    pub labels: Vec<String>,
    ///The model to use for the evaluation. Must support structured outputs.
    pub model: String,
    ///The name of the grader.
    pub name: String,
    ///The labels that indicate a passing result. Must be a subset of labels.
    pub passing_labels: Vec<String>,
    ///The object type, which is always `label_model`.
    #[serde(rename = "type")]
    pub type_value: String,
}
///A MultiGrader object combines the output of multiple graders to produce a single score.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct GraderMulti {
    ///A formula to calculate the output based on grader results.
    pub calculate_output: String,
    pub graders: GraderMultiGraders,
    ///The name of the grader.
    pub name: String,
    ///The object type, which is always `multi`.
    #[serde(rename = "type")]
    pub type_value: String,
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
#[serde(untagged)]
pub enum GraderMultiGraders {
    GraderStringCheck(GraderStringCheck),
    GraderTextSimilarity(GraderTextSimilarity),
    GraderPython(GraderPython),
    GraderScoreModel(GraderScoreModel),
    GraderLabelModel(GraderLabelModel),
}
///A PythonGrader object that runs a python script on the input.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct GraderPython {
    ///The image tag to use for the python script.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub image_tag: ::std::option::Option<String>,
    ///The name of the grader.
    pub name: String,
    ///The source code of the python script.
    pub source: String,
    ///The object type, which is always `python`.
    #[serde(rename = "type")]
    pub type_value: String,
}
///A ScoreModelGrader object that uses a model to assign a score to the input.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct GraderScoreModel {
    ///The input messages evaluated by the grader. Supports text, output text, input image, and input audio content blocks, and may include template strings.
    pub input: Vec<EvalItem>,
    ///The model to use for the evaluation.
    pub model: String,
    ///The name of the grader.
    pub name: String,
    ///The range of the score. Defaults to `[0, 1]`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub range: ::std::option::Option<Vec<f64>>,
    ///The sampling parameters for the model.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sampling_params: ::std::option::Option<GraderScoreModelSamplingParams>,
    ///The object type, which is always `score_model`.
    #[serde(rename = "type")]
    pub type_value: String,
}
///The sampling parameters for the model.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct GraderScoreModelSamplingParams {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_completions_tokens: ::std::option::Option<i32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reasoning_effort: ::std::option::Option<ReasoningEffort>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub seed: ::std::option::Option<i32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub temperature: ::std::option::Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub top_p: ::std::option::Option<f64>,
}
///A StringCheckGrader object that performs a string comparison between input and reference using a specified operation.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct GraderStringCheck {
    ///The input text. This may include template strings.
    pub input: String,
    ///The name of the grader.
    pub name: String,
    ///The string check operation to perform. One of `eq`, `ne`, `like`, or `ilike`.
    pub operation: String,
    ///The reference text. This may include template strings.
    pub reference: String,
    ///The object type, which is always `string_check`.
    #[serde(rename = "type")]
    pub type_value: String,
}
///A TextSimilarityGrader object which grades text based on similarity metrics.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct GraderTextSimilarity {
    ///The evaluation metric to use. One of `cosine`, `fuzzy_match`, `bleu`, `gleu`, `meteor`, `rouge_1`, `rouge_2`, `rouge_3`, `rouge_4`, `rouge_5`, or `rouge_l`.
    pub evaluation_metric: String,
    ///The text being graded.
    pub input: String,
    ///The name of the grader.
    pub name: String,
    ///The text being graded against.
    pub reference: String,
    ///The type of grader.
    #[serde(rename = "type")]
    pub type_value: String,
}
pub type GrammarSyntax1 = String;
///Summary information about a group returned in role assignment responses.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct Group {
    ///Unix timestamp (in seconds) when the group was created.
    pub created_at: i64,
    ///Identifier for the group.
    pub id: String,
    ///Display name of the group.
    pub name: String,
    ///Always `group`.
    pub object: String,
    ///Whether the group is managed through SCIM.
    pub scim_managed: bool,
}
///Confirmation payload returned after deleting a group.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct GroupDeletedResource {
    ///Whether the group was deleted.
    pub deleted: bool,
    ///Identifier of the deleted group.
    pub id: String,
    ///Always `group.deleted`.
    pub object: String,
}
///Paginated list of organization groups.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct GroupListResource {
    ///Groups returned in the current page.
    pub data: Vec<GroupResponse>,
    ///Whether additional groups are available when paginating.
    pub has_more: bool,
    ///Cursor to fetch the next page of results, or `null` if there are no more results.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub next: ::std::option::Option<String>,
    ///Always `list`.
    pub object: String,
}
///Response returned after updating a group.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct GroupResourceWithSuccess {
    ///Unix timestamp (in seconds) when the group was created.
    pub created_at: i64,
    ///Identifier for the group.
    pub id: String,
    ///Whether the group is managed through SCIM and controlled by your identity provider.
    pub is_scim_managed: bool,
    ///Updated display name for the group.
    pub name: String,
}
///Details about an organization group.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct GroupResponse {
    ///Unix timestamp (in seconds) when the group was created.
    pub created_at: i64,
    ///The type of the group.
    pub group_type: String,
    ///Identifier for the group.
    pub id: String,
    ///Whether the group is managed through SCIM and controlled by your identity provider.
    pub is_scim_managed: bool,
    ///Display name of the group.
    pub name: String,
}
///Role assignment linking a group to a role.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct GroupRoleAssignment {
    pub group: Group,
    ///Always `group.role`.
    pub object: String,
    pub role: Role,
}
///Represents an individual user returned when inspecting group membership.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct GroupUser {
    ///The email address of the user.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub email: ::std::option::Option<String>,
    ///The identifier, which can be referenced in API endpoints
    pub id: String,
    ///The name of the user.
    pub name: String,
}
///Confirmation payload returned after adding a user to a group.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct GroupUserAssignment {
    ///Identifier of the group the user was added to.
    pub group_id: String,
    ///Always `group.user`.
    pub object: String,
    ///Identifier of the user that was added.
    pub user_id: String,
}
///Confirmation payload returned after removing a user from a group.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct GroupUserDeletedResource {
    ///Whether the group membership was removed.
    pub deleted: bool,
    ///Always `group.user.deleted`.
    pub object: String,
}
///Controls how much historical context is retained for the session.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct HistoryParam {
    ///Enables chat users to access previous ChatKit threads. Defaults to true.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub enabled: ::std::option::Option<bool>,
    ///Number of recent ChatKit threads users have access to. Defaults to unlimited when unset.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub recent_threads: ::std::option::Option<i32>,
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct HybridSearchOptions {
    ///The weight of the embedding in the reciprocal ranking fusion.
    pub embedding_weight: f64,
    ///The weight of the text in the reciprocal ranking fusion.
    pub text_weight: f64,
}
///Represents the content or the URL of an image generated by the OpenAI API.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct Image {
    ///The base64-encoded JSON of the generated image. Returned by default for the GPT image models, and only present if `response_format` is set to `b64_json` for `dall-e-2` and `dall-e-3`.
    #[serde(rename = "b64_json", default, skip_serializing_if = "Option::is_none")]
    pub b_64_json: ::std::option::Option<String>,
    ///For `dall-e-3` only, the revised prompt that was used to generate the image.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub revised_prompt: ::std::option::Option<String>,
    ///When using `dall-e-2` or `dall-e-3`, the URL of the generated image if `response_format` is set to `url` (default value). Unsupported for the GPT image models.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub url: ::std::option::Option<String>,
}
pub type ImageDetail = String;
///Emitted when image editing has completed and the final image is available.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct ImageEditCompletedEvent {
    ///Base64-encoded final edited image data, suitable for rendering as an image.
    #[serde(rename = "b64_json")]
    pub b_64_json: String,
    ///The background setting for the edited image.
    pub background: String,
    ///The Unix timestamp when the event was created.
    pub created_at: i64,
    ///The output format for the edited image.
    pub output_format: String,
    ///The quality setting for the edited image.
    pub quality: String,
    ///The size of the edited image.
    pub size: String,
    ///The type of the event. Always `image_edit.completed`.
    #[serde(rename = "type")]
    pub type_value: String,
    pub usage: ImagesUsage,
}
///Emitted when a partial image is available during image editing streaming.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct ImageEditPartialImageEvent {
    ///Base64-encoded partial image data, suitable for rendering as an image.
    #[serde(rename = "b64_json")]
    pub b_64_json: String,
    ///The background setting for the requested edited image.
    pub background: String,
    ///The Unix timestamp when the event was created.
    pub created_at: i64,
    ///The output format for the requested edited image.
    pub output_format: String,
    ///0-based index for the partial image (streaming).
    pub partial_image_index: i32,
    ///The quality setting for the requested edited image.
    pub quality: String,
    ///The size of the requested edited image.
    pub size: String,
    ///The type of the event. Always `image_edit.partial_image`.
    #[serde(rename = "type")]
    pub type_value: String,
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
#[serde(untagged)]
pub enum ImageEditStreamEvent {
    ImageEditPartialImageEvent(ImageEditPartialImageEvent),
    ImageEditCompletedEvent(ImageEditCompletedEvent),
}
pub type ImageGenActionEnum = String;
///Emitted when image generation has completed and the final image is available.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct ImageGenCompletedEvent {
    ///Base64-encoded image data, suitable for rendering as an image.
    #[serde(rename = "b64_json")]
    pub b_64_json: String,
    ///The background setting for the generated image.
    pub background: String,
    ///The Unix timestamp when the event was created.
    pub created_at: i64,
    ///The output format for the generated image.
    pub output_format: String,
    ///The quality setting for the generated image.
    pub quality: String,
    ///The size of the generated image.
    pub size: String,
    ///The type of the event. Always `image_generation.completed`.
    #[serde(rename = "type")]
    pub type_value: String,
    pub usage: ImagesUsage,
}
///The input tokens detailed information for the image generation.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct ImageGenInputUsageDetails {
    ///The number of image tokens in the input prompt.
    pub image_tokens: i32,
    ///The number of text tokens in the input prompt.
    pub text_tokens: i32,
}
///The output token details for the image generation.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct ImageGenOutputTokensDetails {
    ///The number of image output tokens generated by the model.
    pub image_tokens: i32,
    ///The number of text output tokens generated by the model.
    pub text_tokens: i32,
}
///Emitted when a partial image is available during image generation streaming.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct ImageGenPartialImageEvent {
    ///Base64-encoded partial image data, suitable for rendering as an image.
    #[serde(rename = "b64_json")]
    pub b_64_json: String,
    ///The background setting for the requested image.
    pub background: String,
    ///The Unix timestamp when the event was created.
    pub created_at: i64,
    ///The output format for the requested image.
    pub output_format: String,
    ///0-based index for the partial image (streaming).
    pub partial_image_index: i32,
    ///The quality setting for the requested image.
    pub quality: String,
    ///The size of the requested image.
    pub size: String,
    ///The type of the event. Always `image_generation.partial_image`.
    #[serde(rename = "type")]
    pub type_value: String,
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
#[serde(untagged)]
pub enum ImageGenStreamEvent {
    ImageGenPartialImageEvent(ImageGenPartialImageEvent),
    ImageGenCompletedEvent(ImageGenCompletedEvent),
}
///A tool that generates images using the GPT image models.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct ImageGenTool {
    ///Whether to generate a new image or edit an existing image. Default: `auto`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub action: ::std::option::Option<ImageGenActionEnum>,
    ///Background type for the generated image. One of `transparent`, `opaque`, or `auto`. Default: `auto`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub background: ::std::option::Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub input_fidelity: ::std::option::Option<InputFidelity>,
    ///Optional mask for inpainting. Contains `image_url` (string, optional) and `file_id` (string, optional).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub input_image_mask: ::std::option::Option<ImageGenToolInputImageMask>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model: ::std::option::Option<String>,
    ///Moderation level for the generated image. Default: `auto`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub moderation: ::std::option::Option<String>,
    ///Compression level for the output image. Default: 100.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub output_compression: ::std::option::Option<i32>,
    ///The output format of the generated image. One of `png`, `webp`, or `jpeg`. Default: `png`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub output_format: ::std::option::Option<String>,
    ///Number of partial images to generate in streaming mode, from 0 (default value) to 3.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub partial_images: ::std::option::Option<i32>,
    ///The quality of the generated image. One of `low`, `medium`, `high`, or `auto`. Default: `auto`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub quality: ::std::option::Option<String>,
    ///The size of the generated images. For `gpt-image-2` and `gpt-image-2-2026-04-21`, arbitrary resolutions are supported as `WIDTHxHEIGHT` strings, for example `1536x864`. Width and height must both be divisible by 16 and the requested aspect ratio must be between 1:3 and 3:1. Resolutions above `2560x1440` are experimental, and the maximum supported resolution is `3840x2160`. The requested size must also satisfy the model's current pixel and edge limits. The standard sizes `1024x1024`, `1536x1024`, and `1024x1536` are supported by the GPT image models; `auto` is supported for models that allow automatic sizing. For `dall-e-2`, use one of `256x256`, `512x512`, or `1024x1024`. For `dall-e-3`, use one of `1024x1024`, `1792x1024`, or `1024x1792`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub size: ::std::option::Option<String>,
    ///The type of the image generation tool. Always `image_generation`.
    #[serde(rename = "type")]
    pub type_value: String,
}
///An image generation request made by the model.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct ImageGenToolCall {
    ///The unique ID of the image generation call.
    pub id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub result: ::std::option::Option<String>,
    ///The status of the image generation call.
    pub status: String,
    ///The type of the image generation call. Always `image_generation_call`.
    #[serde(rename = "type")]
    pub type_value: String,
}
///Optional mask for inpainting. Contains `image_url` (string, optional) and `file_id` (string, optional).
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct ImageGenToolInputImageMask {
    ///File ID for the mask image.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub file_id: ::std::option::Option<String>,
    ///Base64-encoded mask image.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub image_url: ::std::option::Option<String>,
}
///For `gpt-image-1` only, the token usage information for the image generation.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct ImageGenUsage {
    ///The number of tokens (images and text) in the input prompt.
    pub input_tokens: i32,
    pub input_tokens_details: ImageGenInputUsageDetails,
    ///The number of output tokens generated by the model.
    pub output_tokens: i32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub output_tokens_details: ::std::option::Option<ImageGenOutputTokensDetails>,
    ///The total number of tokens (images and text) used for the image generation.
    pub total_tokens: i32,
}
///Reference an input image by either URL or uploaded file ID. Provide exactly one of `image_url` or `file_id`.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct ImageRefParam {
    ///The File API ID of an uploaded image to use as input.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub file_id: ::std::option::Option<String>,
    ///A fully qualified URL or base64-encoded data URL.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub image_url: ::std::option::Option<String>,
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct ImageRefParam2 {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub file_id: ::std::option::Option<String>,
    ///A fully qualified URL or base64-encoded data URL.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub image_url: ::std::option::Option<String>,
}
///The response from the image generation endpoint.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct ImagesResponse {
    ///The background parameter used for the image generation. Either `transparent` or `opaque`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub background: ::std::option::Option<String>,
    ///The Unix timestamp (in seconds) of when the image was created.
    pub created: i64,
    ///The list of generated images.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub data: ::std::option::Option<Vec<Image>>,
    ///The output format of the image generation. Either `png`, `webp`, or `jpeg`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub output_format: ::std::option::Option<String>,
    ///The quality of the image generated. Either `low`, `medium`, or `high`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub quality: ::std::option::Option<String>,
    ///The size of the image generated. Either `1024x1024`, `1024x1536`, or `1536x1024`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub size: ::std::option::Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub usage: ::std::option::Option<ImageGenUsage>,
}
///For the GPT image models only, the token usage information for the image generation.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct ImagesUsage {
    ///The number of tokens (images and text) in the input prompt.
    pub input_tokens: i32,
    ///The input tokens detailed information for the image generation.
    pub input_tokens_details: ImagesUsageInputTokensDetails,
    ///The number of image tokens in the output image.
    pub output_tokens: i32,
    ///The total number of tokens (images and text) used for the image generation.
    pub total_tokens: i32,
}
///The input tokens detailed information for the image generation.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct ImagesUsageInputTokensDetails {
    ///The number of image tokens in the input prompt.
    pub image_tokens: i32,
    ///The number of text tokens in the input prompt.
    pub text_tokens: i32,
}
///Specify additional output data to include in the model response. Currently supported values are: - `web_search_call.results`: Include the search results of the web search tool call. - `web_search_call.action.sources`: Include the sources of the web search tool call. - `code_interpreter_call.outputs`: Includes the outputs of python code execution in code interpreter tool call items. - `computer_call_output.output.image_url`: Include image urls from the computer call output. - `file_search_call.results`: Include the search results of the file search tool call. - `message.input_image.image_url`: Include image urls from the input message. - `message.output_text.logprobs`: Include logprobs with assistant messages. - `reasoning.encrypted_content`: Includes an encrypted version of reasoning tokens in reasoning item outputs. This enables reasoning items to be used in multi-turn conversations when using the Responses API statelessly (like when the `store` parameter is set to `false`, or when an organization is enrolled in the zero data retention program).
pub type IncludeEnum = String;
///Model and tool overrides applied when generating the assistant response.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct InferenceOptions {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model: ::std::option::Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tool_choice: ::std::option::Option<ToolChoice>,
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct InlineSkillParam {
    ///The description of the skill.
    pub description: String,
    ///The name of the skill.
    pub name: String,
    ///Inline skill payload
    pub source: InlineSkillSourceParam,
    ///Defines an inline skill for this request.
    #[serde(rename = "type")]
    pub type_value: String,
}
///Inline skill payload
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct InlineSkillSourceParam {
    ///Base64-encoded skill zip bundle.
    pub data: String,
    ///The media type of the inline skill payload. Must be `application/zip`.
    pub media_type: String,
    ///The type of the inline skill source. Must be `base64`.
    #[serde(rename = "type")]
    pub type_value: String,
}
///An audio input to the model.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct InputAudio {
    pub input_audio: InputAudioInputAudio,
    ///The type of the input item. Always `input_audio`.
    #[serde(rename = "type")]
    pub type_value: String,
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct InputAudioInputAudio {
    ///Base64-encoded audio data.
    pub data: String,
    ///The format of the audio data. Currently supported formats are `mp3` and `wav`.
    pub format: String,
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
#[serde(untagged)]
pub enum InputContent {
    InputTextContent(InputTextContent),
    InputImageContent(InputImageContent),
    InputFileContent(InputFileContent),
}
///Control how much effort the model will exert to match the style and features, especially facial features, of input images. This parameter is only supported for `gpt-image-1` and `gpt-image-1.5` and later models, unsupported for `gpt-image-1-mini`. Supports `high` and `low`. Defaults to `low`.
pub type InputFidelity = String;
///A file input to the model.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct InputFileContent {
    ///The detail level of the file to be sent to the model. Use `low` for the default rendering behavior, or `high` to render the file at higher quality. Defaults to `low`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub detail: ::std::option::Option<FileInputDetail>,
    ///The content of the file to be sent to the model.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub file_data: ::std::option::Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub file_id: ::std::option::Option<String>,
    ///The URL of the file to be sent to the model.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub file_url: ::std::option::Option<String>,
    ///The name of the file to be sent to the model.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub filename: ::std::option::Option<String>,
    ///The type of the input item. Always `input_file`.
    #[serde(rename = "type")]
    pub type_value: String,
}
///A file input to the model.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct InputFileContentParam {
    ///The detail level of the file to be sent to the model. Use `low` for the default rendering behavior, or `high` to render the file at higher quality. Defaults to `low`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub detail: ::std::option::Option<FileDetailEnum>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub file_data: ::std::option::Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub file_id: ::std::option::Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub file_url: ::std::option::Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub filename: ::std::option::Option<String>,
    ///The type of the input item. Always `input_file`.
    #[serde(rename = "type")]
    pub type_value: String,
}
///An image input to the model. Learn about [image inputs](/docs/guides/vision).
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct InputImageContent {
    ///The detail level of the image to be sent to the model. One of `high`, `low`, `auto`, or `original`. Defaults to `auto`.
    pub detail: ImageDetail,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub file_id: ::std::option::Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub image_url: ::std::option::Option<String>,
    ///The type of the input item. Always `input_image`.
    #[serde(rename = "type")]
    pub type_value: String,
}
///An image input to the model. Learn about [image inputs](/docs/guides/vision)
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct InputImageContentParamAutoParam {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub detail: ::std::option::Option<DetailEnum>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub file_id: ::std::option::Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub image_url: ::std::option::Option<String>,
    ///The type of the input item. Always `input_image`.
    #[serde(rename = "type")]
    pub type_value: String,
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
#[serde(untagged)]
pub enum InputItem {
    EasyInputMessage(EasyInputMessage),
    Item(Item),
    ItemReferenceParam(ItemReferenceParam),
}
///A message input to the model with a role indicating instruction following hierarchy. Instructions given with the `developer` or `system` role take precedence over instructions given with the `user` role.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct InputMessage {
    pub content: InputMessageContentList,
    ///The role of the message input. One of `user`, `system`, or `developer`.
    pub role: String,
    ///The status of item. One of `in_progress`, `completed`, or `incomplete`. Populated when items are returned via API.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub status: ::std::option::Option<String>,
    ///The type of the message input. Always set to `message`.
    #[serde(rename = "type", default, skip_serializing_if = "Option::is_none")]
    pub type_value: ::std::option::Option<String>,
}
///A list of one or many input items to the model, containing different content types.
pub type InputMessageContentList = Vec<InputContent>;
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct InputMessageResource {
    pub content: InputMessageContentList,
    ///The unique ID of the message input.
    pub id: String,
    ///The role of the message input. One of `user`, `system`, or `developer`.
    pub role: String,
    ///The status of item. One of `in_progress`, `completed`, or `incomplete`. Populated when items are returned via API.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub status: ::std::option::Option<String>,
    ///The type of the message input. Always set to `message`.
    #[serde(rename = "type", default, skip_serializing_if = "Option::is_none")]
    pub type_value: ::std::option::Option<String>,
}
///Text, image, or file inputs to the model, used to generate a response. Learn more: - [Text inputs and outputs](/docs/guides/text) - [Image inputs](/docs/guides/images) - [File inputs](/docs/guides/pdf-files) - [Conversation state](/docs/guides/conversation-state) - [Function calling](/docs/guides/function-calling)
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
#[serde(untagged)]
pub enum InputParam {
    TextInput(String),
    InputItemList(Vec<InputItem>),
}
///A text input to the model.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct InputTextContent {
    ///The text input to the model.
    pub text: String,
    ///The type of the input item. Always `input_text`.
    #[serde(rename = "type")]
    pub type_value: String,
}
///A text input to the model.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct InputTextContentParam {
    ///The text input to the model.
    pub text: String,
    ///The type of the input item. Always `input_text`.
    #[serde(rename = "type")]
    pub type_value: String,
}
///Represents an individual `invite` to the organization.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct Invite {
    ///The Unix timestamp (in seconds) of when the invite was accepted.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub accepted_at: ::std::option::Option<i64>,
    ///The Unix timestamp (in seconds) of when the invite was sent.
    pub created_at: i64,
    ///The email address of the individual to whom the invite was sent
    pub email: String,
    ///The Unix timestamp (in seconds) of when the invite expires.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expires_at: ::std::option::Option<i64>,
    ///The identifier, which can be referenced in API endpoints
    pub id: String,
    ///The object type, which is always `organization.invite`
    pub object: String,
    ///The projects that were granted membership upon acceptance of the invite.
    pub projects: Vec<InviteProject>,
    ///`owner` or `reader`
    pub role: String,
    ///`accepted`,`expired`, or `pending`
    pub status: String,
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct InviteDeleteResponse {
    pub deleted: bool,
    pub id: String,
    ///The object type, which is always `organization.invite.deleted`
    pub object: String,
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct InviteListResponse {
    pub data: Vec<Invite>,
    ///The first `invite_id` in the retrieved `list`
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub first_id: ::std::option::Option<String>,
    ///The `has_more` property is used for pagination to indicate there are additional results.
    pub has_more: bool,
    ///The last `invite_id` in the retrieved `list`
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_id: ::std::option::Option<String>,
    ///The object type, which is always `list`
    pub object: String,
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct InviteProject {
    ///Project's public ID
    pub id: String,
    ///Project membership role
    pub role: String,
}
///Request payload for granting a group access to a project.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct InviteProjectGroupBody {
    ///Identifier of the group to add to the project.
    pub group_id: String,
    ///Identifier of the project role to grant to the group.
    pub role: String,
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct InviteRequest {
    ///Send an email to this address
    pub email: String,
    ///An array of projects to which membership is granted at the same time the org invite is accepted. If omitted, the user will be invited to the default project for compatibility with legacy behavior.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub projects: ::std::option::Option<Vec<InviteRequestProject>>,
    ///`owner` or `reader`
    pub role: String,
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct InviteRequestProject {
    ///Project's public ID
    pub id: String,
    ///Project membership role
    pub role: String,
}
///Content item used to generate a response.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct Item {
    #[serde(flatten)]
    pub value: OpenAiJsonObject,
}
///An item representing a message, tool call, tool output, reasoning, or other response element.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
#[serde(untagged)]
pub enum ItemField {
    Message(Message),
    FunctionToolCall(FunctionToolCall),
    ToolSearchCall(ToolSearchCall),
    ToolSearchOutput(ToolSearchOutput),
    FunctionToolCallOutput(FunctionToolCallOutput),
    FileSearchToolCall(FileSearchToolCall),
    WebSearchToolCall(WebSearchToolCall),
    ImageGenToolCall(ImageGenToolCall),
    ComputerToolCall(ComputerToolCall),
    ComputerToolCallOutputResource(ComputerToolCallOutputResource),
    ReasoningItem(ReasoningItem),
    CompactionBody(CompactionBody),
    CodeInterpreterToolCall(CodeInterpreterToolCall),
    LocalShellToolCall(LocalShellToolCall),
    LocalShellToolCallOutput(LocalShellToolCallOutput),
    FunctionShellCall(FunctionShellCall),
    FunctionShellCallOutput(FunctionShellCallOutput),
    ApplyPatchToolCall(ApplyPatchToolCall),
    ApplyPatchToolCallOutput(ApplyPatchToolCallOutput),
    McpListTools(McpListTools),
    McpApprovalRequest(McpApprovalRequest),
    McpApprovalResponseResource(McpApprovalResponseResource),
    McpToolCall(McpToolCall),
    CustomToolCall(CustomToolCall),
    CustomToolCallOutput(CustomToolCallOutput),
}
///An internal identifier for an item to reference.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct ItemReferenceParam {
    ///The ID of the item to reference.
    pub id: String,
    #[serde(rename = "type", default, skip_serializing_if = "Option::is_none")]
    pub type_value: ::std::option::Option<String>,
}
///Content item used to generate a response.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
#[serde(untagged)]
pub enum ItemResource {
    InputMessageResource(InputMessageResource),
    OutputMessage(OutputMessage),
    FileSearchToolCall(FileSearchToolCall),
    ComputerToolCall(ComputerToolCall),
    ComputerToolCallOutputResource(ComputerToolCallOutputResource),
    WebSearchToolCall(WebSearchToolCall),
    FunctionToolCallResource(FunctionToolCallResource),
    FunctionToolCallOutputResource(FunctionToolCallOutputResource),
    ToolSearchCall(ToolSearchCall),
    ToolSearchOutput(ToolSearchOutput),
    ReasoningItem(ReasoningItem),
    CompactionBody(CompactionBody),
    ImageGenToolCall(ImageGenToolCall),
    CodeInterpreterToolCall(CodeInterpreterToolCall),
    LocalShellToolCall(LocalShellToolCall),
    LocalShellToolCallOutput(LocalShellToolCallOutput),
    FunctionShellCall(FunctionShellCall),
    FunctionShellCallOutput(FunctionShellCallOutput),
    ApplyPatchToolCall(ApplyPatchToolCall),
    ApplyPatchToolCallOutput(ApplyPatchToolCallOutput),
    McpListTools(McpListTools),
    McpApprovalRequest(McpApprovalRequest),
    McpApprovalResponseResource(McpApprovalResponseResource),
    McpToolCall(McpToolCall),
    CustomToolCallResource(CustomToolCallResource),
    CustomToolCallOutputResource(CustomToolCallOutputResource),
}
///A collection of keypresses the model would like to perform.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct KeyPressAction {
    ///The combination of keys the model is requesting to be pressed. This is an array of strings, each representing a key.
    pub keys: Vec<String>,
    ///Specifies the event type. For a keypress action, this property is always set to `keypress`.
    #[serde(rename = "type")]
    pub type_value: String,
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct ListAssistantsResponse {
    pub data: Vec<AssistantObject>,
    pub first_id: String,
    pub has_more: bool,
    pub last_id: String,
    pub object: String,
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct ListAuditLogsResponse {
    pub data: Vec<AuditLog>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub first_id: ::std::option::Option<String>,
    pub has_more: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_id: ::std::option::Option<String>,
    pub object: String,
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct ListBatchesResponse {
    pub data: Vec<Batch>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub first_id: ::std::option::Option<String>,
    pub has_more: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_id: ::std::option::Option<String>,
    pub object: String,
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct ListCertificatesResponse {
    pub data: Vec<OrganizationCertificate>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub first_id: ::std::option::Option<String>,
    pub has_more: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_id: ::std::option::Option<String>,
    pub object: String,
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct ListFilesResponse {
    pub data: Vec<OpenAiFile>,
    pub first_id: String,
    pub has_more: bool,
    pub last_id: String,
    pub object: String,
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct ListFineTuningCheckpointPermissionResponse {
    pub data: Vec<FineTuningCheckpointPermission>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub first_id: ::std::option::Option<String>,
    pub has_more: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_id: ::std::option::Option<String>,
    pub object: String,
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct ListFineTuningJobCheckpointsResponse {
    pub data: Vec<FineTuningJobCheckpoint>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub first_id: ::std::option::Option<String>,
    pub has_more: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_id: ::std::option::Option<String>,
    pub object: String,
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct ListFineTuningJobEventsResponse {
    pub data: Vec<FineTuningJobEvent>,
    pub has_more: bool,
    pub object: String,
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct ListMessagesResponse {
    pub data: Vec<MessageObject>,
    pub first_id: String,
    pub has_more: bool,
    pub last_id: String,
    pub object: String,
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct ListModelsResponse {
    pub data: Vec<Model>,
    pub object: String,
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct ListPaginatedFineTuningJobsResponse {
    pub data: Vec<FineTuningJob>,
    pub has_more: bool,
    pub object: String,
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct ListProjectCertificatesResponse {
    pub data: Vec<OrganizationProjectCertificate>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub first_id: ::std::option::Option<String>,
    pub has_more: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_id: ::std::option::Option<String>,
    pub object: String,
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct ListRunStepsResponse {
    pub data: Vec<RunStepObject>,
    pub first_id: String,
    pub has_more: bool,
    pub last_id: String,
    pub object: String,
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct ListRunsResponse {
    pub data: Vec<RunObject>,
    pub first_id: String,
    pub has_more: bool,
    pub last_id: String,
    pub object: String,
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct ListVectorStoreFilesResponse {
    pub data: Vec<VectorStoreFileObject>,
    pub first_id: String,
    pub has_more: bool,
    pub last_id: String,
    pub object: String,
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct ListVectorStoresResponse {
    pub data: Vec<VectorStoreObject>,
    pub first_id: String,
    pub has_more: bool,
    pub last_id: String,
    pub object: String,
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct LocalEnvironmentParam {
    ///An optional list of skills.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub skills: ::std::option::Option<Vec<LocalSkillParam>>,
    ///Use a local computer environment.
    #[serde(rename = "type")]
    pub type_value: String,
}
///Represents the use of a local environment to perform shell actions.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct LocalEnvironmentResource {
    ///The environment type. Always `local`.
    #[serde(rename = "type")]
    pub type_value: String,
}
///Execute a shell command on the server.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct LocalShellExecAction {
    ///The command to run.
    pub command: Vec<String>,
    ///Environment variables to set for the command.
    pub env: OpenAiJsonValue,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub timeout_ms: ::std::option::Option<i32>,
    ///The type of the local shell action. Always `exec`.
    #[serde(rename = "type")]
    pub type_value: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub user: ::std::option::Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub working_directory: ::std::option::Option<String>,
}
///A tool call to run a command on the local shell.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct LocalShellToolCall {
    pub action: LocalShellExecAction,
    ///The unique ID of the local shell tool call generated by the model.
    pub call_id: String,
    ///The unique ID of the local shell call.
    pub id: String,
    ///The status of the local shell call.
    pub status: String,
    ///The type of the local shell call. Always `local_shell_call`.
    #[serde(rename = "type")]
    pub type_value: String,
}
///The output of a local shell tool call.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct LocalShellToolCallOutput {
    ///The unique ID of the local shell tool call generated by the model.
    pub id: String,
    ///A JSON string of the output of the local shell tool call.
    pub output: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub status: ::std::option::Option<String>,
    ///The type of the local shell tool call output. Always `local_shell_call_output`.
    #[serde(rename = "type")]
    pub type_value: String,
}
///A tool that allows the model to execute shell commands in a local environment.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct LocalShellToolParam {
    ///The type of the local shell tool. Always `local_shell`.
    #[serde(rename = "type")]
    pub type_value: String,
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct LocalSkillParam {
    ///The description of the skill.
    pub description: String,
    ///The name of the skill.
    pub name: String,
    ///The path to the directory containing the skill.
    pub path: String,
}
///Indicates that a thread is locked and cannot accept new input.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct LockedStatus {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reason: ::std::option::Option<String>,
    ///Status discriminator that is always `locked`.
    #[serde(rename = "type")]
    pub type_value: String,
}
///The log probability of a token.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct LogProb {
    pub bytes: Vec<i32>,
    pub logprob: f64,
    pub token: String,
    pub top_logprobs: Vec<TopLogProb>,
}
///A log probability object.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct LogProbProperties {
    ///The bytes that were used to generate the log probability.
    pub bytes: Vec<i32>,
    ///The log probability of the token.
    pub logprob: f64,
    ///The token that was used to generate the log probability.
    pub token: String,
}
///A request for human approval of a tool invocation.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct McpApprovalRequest {
    ///A JSON string of arguments for the tool.
    pub arguments: String,
    ///The unique ID of the approval request.
    pub id: String,
    ///The name of the tool to run.
    pub name: String,
    ///The label of the MCP server making the request.
    pub server_label: String,
    ///The type of the item. Always `mcp_approval_request`.
    #[serde(rename = "type")]
    pub type_value: String,
}
///A response to an MCP approval request.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct McpApprovalResponse {
    ///The ID of the approval request being answered.
    pub approval_request_id: String,
    ///Whether the request was approved.
    pub approve: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: ::std::option::Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reason: ::std::option::Option<String>,
    ///The type of the item. Always `mcp_approval_response`.
    #[serde(rename = "type")]
    pub type_value: String,
}
///A response to an MCP approval request.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct McpApprovalResponseResource {
    ///The ID of the approval request being answered.
    pub approval_request_id: String,
    ///Whether the request was approved.
    pub approve: bool,
    ///The unique ID of the approval response
    pub id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reason: ::std::option::Option<String>,
    ///The type of the item. Always `mcp_approval_response`.
    #[serde(rename = "type")]
    pub type_value: String,
}
///A list of tools available on an MCP server.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct McpListTools {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: ::std::option::Option<String>,
    ///The unique ID of the list.
    pub id: String,
    ///The label of the MCP server.
    pub server_label: String,
    ///The tools available on the server.
    pub tools: Vec<McpListToolsTool>,
    ///The type of the item. Always `mcp_list_tools`.
    #[serde(rename = "type")]
    pub type_value: String,
}
///A tool available on an MCP server.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct McpListToolsTool {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub annotations: ::std::option::Option<OpenAiJsonValue>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: ::std::option::Option<String>,
    ///The JSON schema describing the tool's input.
    pub input_schema: OpenAiJsonValue,
    ///The name of the tool.
    pub name: String,
}
///Give the model access to additional tools via remote Model Context Protocol (MCP) servers. [Learn more about MCP](/docs/guides/tools-remote-mcp).
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct McpTool {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub allowed_tools: ::std::option::Option<McpToolAllowedTools>,
    ///An OAuth access token that can be used with a remote MCP server, either with a custom MCP server URL or a service connector. Your application must handle the OAuth authorization flow and provide the token here.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub authorization: ::std::option::Option<String>,
    ///Identifier for service connectors, like those available in ChatGPT. One of `server_url` or `connector_id` must be provided. Learn more about service connectors [here](/docs/guides/tools-remote-mcp#connectors). Currently supported `connector_id` values are: - Dropbox: `connector_dropbox` - Gmail: `connector_gmail` - Google Calendar: `connector_googlecalendar` - Google Drive: `connector_googledrive` - Microsoft Teams: `connector_microsoftteams` - Outlook Calendar: `connector_outlookcalendar` - Outlook Email: `connector_outlookemail` - SharePoint: `connector_sharepoint`
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub connector_id: ::std::option::Option<String>,
    ///Whether this MCP tool is deferred and discovered via tool search.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub defer_loading: ::std::option::Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub headers: ::std::option::Option<OpenAiJsonValue>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub require_approval: ::std::option::Option<McpToolRequireApproval2>,
    ///Optional description of the MCP server, used to provide more context.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub server_description: ::std::option::Option<String>,
    ///A label for this MCP server, used to identify it in tool calls.
    pub server_label: String,
    ///The URL for the MCP server. One of `server_url` or `connector_id` must be provided.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub server_url: ::std::option::Option<String>,
    ///The type of the MCP tool. Always `mcp`.
    #[serde(rename = "type")]
    pub type_value: String,
}
///List of allowed tool names or a filter object.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
#[serde(untagged)]
pub enum McpToolAllowedTools {
    McpAllowedTools(Vec<String>),
    McpToolFilter(McpToolFilter),
}
///An invocation of a tool on an MCP server.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct McpToolCall {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub approval_request_id: ::std::option::Option<String>,
    ///A JSON string of the arguments passed to the tool.
    pub arguments: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: ::std::option::Option<String>,
    ///The unique ID of the tool call.
    pub id: String,
    ///The name of the tool that was run.
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub output: ::std::option::Option<String>,
    ///The label of the MCP server running the tool.
    pub server_label: String,
    ///The status of the tool call. One of `in_progress`, `completed`, `incomplete`, `calling`, or `failed`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub status: ::std::option::Option<McpToolCallStatus>,
    ///The type of the item. Always `mcp_call`.
    #[serde(rename = "type")]
    pub type_value: String,
}
pub type McpToolCallStatus = String;
///A filter object to specify which tools are allowed.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct McpToolFilter {
    ///Indicates whether or not a tool modifies data or is read-only. If an MCP server is [annotated with `readOnlyHint`](https://modelcontextprotocol.io/specification/2025-06-18/schema#toolannotations-readonlyhint), it will match this filter.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub read_only: ::std::option::Option<bool>,
    ///List of allowed tool names.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tool_names: ::std::option::Option<Vec<String>>,
}
///Specify which of the MCP server's tools require approval. Can be `always`, `never`, or a filter object associated with tools that require approval.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct McpToolRequireApproval {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub always: ::std::option::Option<McpToolFilter>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub never: ::std::option::Option<McpToolFilter>,
}
///Specify which of the MCP server's tools require approval.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
#[serde(untagged)]
pub enum McpToolRequireApproval2 {
    McpToolApprovalFilter(McpToolRequireApproval2McpToolApprovalFilter),
    McpToolApprovalSetting(String),
}
///Specify which of the MCP server's tools require approval. Can be `always`, `never`, or a filter object associated with tools that require approval.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct McpToolRequireApproval2McpToolApprovalFilter {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub always: ::std::option::Option<McpToolFilter>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub never: ::std::option::Option<McpToolFilter>,
}
///A message to or from the model.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct Message {
    ///The content of the message
    pub content: Vec<MessageContentItem>,
    ///The unique ID of the message.
    pub id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub phase: ::std::option::Option<MessagePhase2>,
    ///The role of the message. One of `unknown`, `user`, `assistant`, `system`, `critic`, `discriminator`, `developer`, or `tool`.
    pub role: MessageRole,
    ///The status of item. One of `in_progress`, `completed`, or `incomplete`. Populated when items are returned via API.
    pub status: MessageStatus,
    ///The type of the message. Always set to `message`.
    #[serde(rename = "type")]
    pub type_value: String,
}
///References an image [File](/docs/api-reference/files) in the content of a message.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct MessageContentImageFileObject {
    pub image_file: MessageContentImageFileObjectImageFile,
    ///Always `image_file`.
    #[serde(rename = "type")]
    pub type_value: String,
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct MessageContentImageFileObjectImageFile {
    ///Specifies the detail level of the image if specified by the user. `low` uses fewer tokens, you can opt in to high resolution using `high`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub detail: ::std::option::Option<String>,
    ///The [File](/docs/api-reference/files) ID of the image in the message content. Set `purpose="vision"` when uploading the File if you need to later display the file content.
    pub file_id: String,
}
///References an image URL in the content of a message.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct MessageContentImageUrlObject {
    pub image_url: MessageContentImageUrlObjectImageUrl,
    ///The type of the content part.
    #[serde(rename = "type")]
    pub type_value: String,
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct MessageContentImageUrlObjectImageUrl {
    ///Specifies the detail level of the image. `low` uses fewer tokens, you can opt in to high resolution using `high`. Default value is `auto`
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub detail: ::std::option::Option<String>,
    ///The external URL of the image, must be a supported image types: jpeg, jpg, png, gif, webp.
    pub url: String,
}
///A content part that makes up an input or output item.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
#[serde(untagged)]
pub enum MessageContentItem {
    InputTextContent(InputTextContent),
    OutputTextContent(OutputTextContent),
    TextContent(TextContent),
    SummaryTextContent(SummaryTextContent),
    ReasoningTextContent(ReasoningTextContent),
    RefusalContent(RefusalContent),
    InputImageContent(InputImageContent),
    ComputerScreenshotContent(ComputerScreenshotContent),
    InputFileContent(InputFileContent),
}
///The refusal content generated by the assistant.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct MessageContentRefusalObject {
    pub refusal: String,
    ///Always `refusal`.
    #[serde(rename = "type")]
    pub type_value: String,
}
///A citation within the message that points to a specific quote from a specific File associated with the assistant or the message. Generated when the assistant uses the "file_search" tool to search files.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct MessageContentTextAnnotationsFileCitationObject {
    pub end_index: i32,
    pub file_citation: MessageContentTextAnnotationsFileCitationObjectFileCitation,
    pub start_index: i32,
    ///The text in the message content that needs to be replaced.
    pub text: String,
    ///Always `file_citation`.
    #[serde(rename = "type")]
    pub type_value: String,
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct MessageContentTextAnnotationsFileCitationObjectFileCitation {
    ///The ID of the specific File the citation is from.
    pub file_id: String,
}
///A URL for the file that's generated when the assistant used the `code_interpreter` tool to generate a file.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct MessageContentTextAnnotationsFilePathObject {
    pub end_index: i32,
    pub file_path: MessageContentTextAnnotationsFilePathObjectFilePath,
    pub start_index: i32,
    ///The text in the message content that needs to be replaced.
    pub text: String,
    ///Always `file_path`.
    #[serde(rename = "type")]
    pub type_value: String,
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct MessageContentTextAnnotationsFilePathObjectFilePath {
    ///The ID of the file that was generated.
    pub file_id: String,
}
///The text content that is part of a message.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct MessageContentTextObject {
    pub text: MessageContentTextObjectText,
    ///Always `text`.
    #[serde(rename = "type")]
    pub type_value: String,
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct MessageContentTextObjectText {
    pub annotations: Vec<MessageContentTextObjectTextAnnotation>,
    ///The data that makes up the text.
    pub value: String,
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
#[serde(untagged)]
pub enum MessageContentTextObjectTextAnnotation {
    MessageContentTextAnnotationsFileCitationObject(
        MessageContentTextAnnotationsFileCitationObject,
    ),
    MessageContentTextAnnotationsFilePathObject(
        MessageContentTextAnnotationsFilePathObject,
    ),
}
///References an image [File](/docs/api-reference/files) in the content of a message.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct MessageDeltaContentImageFileObject {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub image_file: ::std::option::Option<MessageDeltaContentImageFileObjectImageFile>,
    ///The index of the content part in the message.
    pub index: i32,
    ///Always `image_file`.
    #[serde(rename = "type")]
    pub type_value: String,
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct MessageDeltaContentImageFileObjectImageFile {
    ///Specifies the detail level of the image if specified by the user. `low` uses fewer tokens, you can opt in to high resolution using `high`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub detail: ::std::option::Option<String>,
    ///The [File](/docs/api-reference/files) ID of the image in the message content. Set `purpose="vision"` when uploading the File if you need to later display the file content.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub file_id: ::std::option::Option<String>,
}
///References an image URL in the content of a message.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct MessageDeltaContentImageUrlObject {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub image_url: ::std::option::Option<MessageDeltaContentImageUrlObjectImageUrl>,
    ///The index of the content part in the message.
    pub index: i32,
    ///Always `image_url`.
    #[serde(rename = "type")]
    pub type_value: String,
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct MessageDeltaContentImageUrlObjectImageUrl {
    ///Specifies the detail level of the image. `low` uses fewer tokens, you can opt in to high resolution using `high`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub detail: ::std::option::Option<String>,
    ///The URL of the image, must be a supported image types: jpeg, jpg, png, gif, webp.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub url: ::std::option::Option<String>,
}
///The refusal content that is part of a message.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct MessageDeltaContentRefusalObject {
    ///The index of the refusal part in the message.
    pub index: i32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub refusal: ::std::option::Option<String>,
    ///Always `refusal`.
    #[serde(rename = "type")]
    pub type_value: String,
}
///A citation within the message that points to a specific quote from a specific File associated with the assistant or the message. Generated when the assistant uses the "file_search" tool to search files.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct MessageDeltaContentTextAnnotationsFileCitationObject {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub end_index: ::std::option::Option<i32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub file_citation: ::std::option::Option<
        MessageDeltaContentTextAnnotationsFileCitationObjectFileCitation,
    >,
    ///The index of the annotation in the text content part.
    pub index: i32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub start_index: ::std::option::Option<i32>,
    ///The text in the message content that needs to be replaced.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub text: ::std::option::Option<String>,
    ///Always `file_citation`.
    #[serde(rename = "type")]
    pub type_value: String,
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct MessageDeltaContentTextAnnotationsFileCitationObjectFileCitation {
    ///The ID of the specific File the citation is from.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub file_id: ::std::option::Option<String>,
    ///The specific quote in the file.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub quote: ::std::option::Option<String>,
}
///A URL for the file that's generated when the assistant used the `code_interpreter` tool to generate a file.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct MessageDeltaContentTextAnnotationsFilePathObject {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub end_index: ::std::option::Option<i32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub file_path: ::std::option::Option<
        MessageDeltaContentTextAnnotationsFilePathObjectFilePath,
    >,
    ///The index of the annotation in the text content part.
    pub index: i32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub start_index: ::std::option::Option<i32>,
    ///The text in the message content that needs to be replaced.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub text: ::std::option::Option<String>,
    ///Always `file_path`.
    #[serde(rename = "type")]
    pub type_value: String,
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct MessageDeltaContentTextAnnotationsFilePathObjectFilePath {
    ///The ID of the file that was generated.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub file_id: ::std::option::Option<String>,
}
///The text content that is part of a message.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct MessageDeltaContentTextObject {
    ///The index of the content part in the message.
    pub index: i32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub text: ::std::option::Option<MessageDeltaContentTextObjectText>,
    ///Always `text`.
    #[serde(rename = "type")]
    pub type_value: String,
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct MessageDeltaContentTextObjectText {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub annotations: ::std::option::Option<
        Vec<MessageDeltaContentTextObjectTextAnnotation>,
    >,
    ///The data that makes up the text.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub value: ::std::option::Option<String>,
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
#[serde(untagged)]
pub enum MessageDeltaContentTextObjectTextAnnotation {
    MessageDeltaContentTextAnnotationsFileCitationObject(
        MessageDeltaContentTextAnnotationsFileCitationObject,
    ),
    MessageDeltaContentTextAnnotationsFilePathObject(
        MessageDeltaContentTextAnnotationsFilePathObject,
    ),
}
///Represents a message delta i.e. any changed fields on a message during streaming.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct MessageDeltaObject {
    ///The delta containing the fields that have changed on the Message.
    pub delta: MessageDeltaObjectDelta,
    ///The identifier of the message, which can be referenced in API endpoints.
    pub id: String,
    ///The object type, which is always `thread.message.delta`.
    pub object: String,
}
///The delta containing the fields that have changed on the Message.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct MessageDeltaObjectDelta {
    ///The content of the message in array of text and/or images.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub content: ::std::option::Option<Vec<MessageDeltaObjectDeltaContentItem>>,
    ///The entity that produced the message. One of `user` or `assistant`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub role: ::std::option::Option<String>,
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
#[serde(untagged)]
pub enum MessageDeltaObjectDeltaContentItem {
    MessageDeltaContentImageFileObject(MessageDeltaContentImageFileObject),
    MessageDeltaContentTextObject(MessageDeltaContentTextObject),
    MessageDeltaContentRefusalObject(MessageDeltaContentRefusalObject),
    MessageDeltaContentImageUrlObject(MessageDeltaContentImageUrlObject),
}
///Represents a message within a [thread](/docs/api-reference/threads).
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct MessageObject {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub assistant_id: ::std::option::Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub attachments: ::std::option::Option<Vec<MessageObjectAttachment>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub completed_at: ::std::option::Option<i64>,
    ///The content of the message in array of text and/or images.
    pub content: Vec<MessageObjectContentItem>,
    ///The Unix timestamp (in seconds) for when the message was created.
    pub created_at: i64,
    ///The identifier, which can be referenced in API endpoints.
    pub id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub incomplete_at: ::std::option::Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub incomplete_details: ::std::option::Option<MessageObjectIncompleteDetails>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub metadata: ::std::option::Option<Metadata>,
    ///The object type, which is always `thread.message`.
    pub object: String,
    ///The entity that produced the message. One of `user` or `assistant`.
    pub role: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub run_id: ::std::option::Option<String>,
    ///The status of the message, which can be either `in_progress`, `incomplete`, or `completed`.
    pub status: String,
    ///The [thread](/docs/api-reference/threads) ID that this message belongs to.
    pub thread_id: String,
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct MessageObjectAttachment {
    ///The ID of the file to attach to the message.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub file_id: ::std::option::Option<String>,
    ///The tools to add this file to.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tools: ::std::option::Option<Vec<MessageObjectAttachmentTool>>,
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
#[serde(untagged)]
pub enum MessageObjectAttachmentTool {
    AssistantToolsCode(AssistantToolsCode),
    AssistantToolsFileSearchTypeOnly(AssistantToolsFileSearchTypeOnly),
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
#[serde(untagged)]
pub enum MessageObjectContentItem {
    MessageContentImageFileObject(MessageContentImageFileObject),
    MessageContentImageUrlObject(MessageContentImageUrlObject),
    MessageContentTextObject(MessageContentTextObject),
    MessageContentRefusalObject(MessageContentRefusalObject),
}
///On an incomplete message, details about why the message is incomplete.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct MessageObjectIncompleteDetails {
    ///The reason the message is incomplete.
    pub reason: String,
}
///Labels an `assistant` message as intermediate commentary (`commentary`) or the final answer (`final_answer`). For models like `gpt-5.3-codex` and beyond, when sending follow-up requests, preserve and resend phase on all assistant messages — dropping it can degrade performance. Not used for user messages.
pub type MessagePhase = String;
pub type MessagePhase2 = String;
///The text content that is part of a message.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct MessageRequestContentTextObject {
    ///Text content to be sent to the model
    pub text: String,
    ///Always `text`.
    #[serde(rename = "type")]
    pub type_value: String,
}
pub type MessageRole = String;
pub type MessageStatus = String;
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
#[serde(untagged)]
pub enum MessageStreamEvent {
    Object(MessageStreamEventObject),
    Object2(MessageStreamEventObject2),
    Object3(MessageStreamEventObject3),
    Object4(MessageStreamEventObject4),
    Object5(MessageStreamEventObject5),
}
///Occurs when a [message](/docs/api-reference/messages/object) is created.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct MessageStreamEventObject {
    pub data: MessageObject,
    pub event: String,
}
///Occurs when a [message](/docs/api-reference/messages/object) moves to an `in_progress` state.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct MessageStreamEventObject2 {
    pub data: MessageObject,
    pub event: String,
}
///Occurs when parts of a [Message](/docs/api-reference/messages/object) are being streamed.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct MessageStreamEventObject3 {
    pub data: MessageDeltaObject,
    pub event: String,
}
///Occurs when a [message](/docs/api-reference/messages/object) is completed.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct MessageStreamEventObject4 {
    pub data: MessageObject,
    pub event: String,
}
///Occurs when a [message](/docs/api-reference/messages/object) ends before it is completed.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct MessageStreamEventObject5 {
    pub data: MessageObject,
    pub event: String,
}
pub type Metadata = OpenAiJsonValue;
///Describes an OpenAI model offering that can be used with the API.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct Model {
    ///The Unix timestamp (in seconds) when the model was created.
    pub created: i64,
    ///The model identifier, which can be referenced in the API endpoints.
    pub id: String,
    ///The object type, which is always "model".
    pub object: String,
    ///The organization that owns the model.
    pub owned_by: String,
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
#[serde(untagged)]
pub enum ModelIds {
    ModelIdsShared(ModelIdsShared),
    ModelIdsResponses(ModelIdsResponses),
}
///Model ID used to generate the response, like `gpt-5` or `o3`. OpenAI offers a wide range of models with different capabilities, performance characteristics, and price points. Refer to the [model guide](/docs/models) to browse and compare available models.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
#[serde(untagged)]
pub enum ModelIdsCompaction {
    ModelIdsResponses(ModelIdsResponses),
    String(String),
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
#[serde(untagged)]
pub enum ModelIdsResponses {
    ModelIdsShared(ModelIdsShared),
    ResponsesOnlyModel(String),
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
#[serde(untagged)]
pub enum ModelIdsShared {
    String(String),
    String2(String),
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct ModelResponseProperties {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub metadata: ::std::option::Option<Metadata>,
    ///Used by OpenAI to cache responses for similar requests to optimize your cache hit rates. Replaces the `user` field. [Learn more](/docs/guides/prompt-caching).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub prompt_cache_key: ::std::option::Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub prompt_cache_retention: ::std::option::Option<String>,
    ///A stable identifier used to help detect users of your application that may be violating OpenAI's usage policies. The IDs should be a string that uniquely identifies each user, with a maximum length of 64 characters. We recommend hashing their username or email address, in order to avoid sending us any identifying information. [Learn more](/docs/guides/safety-best-practices#safety-identifiers).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub safety_identifier: ::std::option::Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub service_tier: ::std::option::Option<ServiceTier>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub temperature: ::std::option::Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub top_logprobs: ::std::option::Option<i32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub top_p: ::std::option::Option<f64>,
    ///This field is being replaced by `safety_identifier` and `prompt_cache_key`. Use `prompt_cache_key` instead to maintain caching optimizations. A stable identifier for your end-users. Used to boost cache hit rates by better bucketing similar requests and to help OpenAI detect and prevent abuse. [Learn more](/docs/guides/safety-best-practices#safety-identifiers).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub user: ::std::option::Option<String>,
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct ModifyAssistantRequest {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: ::std::option::Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub instructions: ::std::option::Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub metadata: ::std::option::Option<Metadata>,
    ///ID of the model to use. You can use the [List models](/docs/api-reference/models/list) API to see all of your available models, or see our [Model overview](/docs/models) for descriptions of them.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model: ::std::option::Option<ModifyAssistantRequestModel>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: ::std::option::Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reasoning_effort: ::std::option::Option<ReasoningEffort>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub response_format: ::std::option::Option<AssistantsApiResponseFormatOption>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub temperature: ::std::option::Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tool_resources: ::std::option::Option<ModifyAssistantRequestToolResources>,
    ///A list of tool enabled on the assistant. There can be a maximum of 128 tools per assistant. Tools can be of types `code_interpreter`, `file_search`, or `function`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tools: ::std::option::Option<Vec<ModifyAssistantRequestTool>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub top_p: ::std::option::Option<f64>,
}
///ID of the model to use. You can use the [List models](/docs/api-reference/models/list) API to see all of your available models, or see our [Model overview](/docs/models) for descriptions of them.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
#[serde(untagged)]
pub enum ModifyAssistantRequestModel {
    String(String),
    AssistantSupportedModels(AssistantSupportedModels),
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
#[serde(untagged)]
pub enum ModifyAssistantRequestTool {
    AssistantToolsCode(AssistantToolsCode),
    AssistantToolsFileSearch(AssistantToolsFileSearch),
    AssistantToolsFunction(AssistantToolsFunction),
}
///A set of resources that are used by the assistant's tools. The resources are specific to the type of tool. For example, the `code_interpreter` tool requires a list of file IDs, while the `file_search` tool requires a list of vector store IDs.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct ModifyAssistantRequestToolResources {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub code_interpreter: ::std::option::Option<
        ModifyAssistantRequestToolResourcesCodeInterpreter,
    >,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub file_search: ::std::option::Option<
        ModifyAssistantRequestToolResourcesFileSearch,
    >,
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct ModifyAssistantRequestToolResourcesCodeInterpreter {
    ///Overrides the list of [file](/docs/api-reference/files) IDs made available to the `code_interpreter` tool. There can be a maximum of 20 files associated with the tool.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub file_ids: ::std::option::Option<Vec<String>>,
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct ModifyAssistantRequestToolResourcesFileSearch {
    ///Overrides the [vector store](/docs/api-reference/vector-stores/object) attached to this assistant. There can be a maximum of 1 vector store attached to the assistant.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub vector_store_ids: ::std::option::Option<Vec<String>>,
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct ModifyCertificateRequest {
    ///The updated name for the certificate
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: ::std::option::Option<String>,
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct ModifyMessageRequest {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub metadata: ::std::option::Option<Metadata>,
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct ModifyRunRequest {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub metadata: ::std::option::Option<Metadata>,
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct ModifyThreadRequest {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub metadata: ::std::option::Option<Metadata>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tool_resources: ::std::option::Option<ModifyThreadRequestToolResources>,
}
///A set of resources that are made available to the assistant's tools in this thread. The resources are specific to the type of tool. For example, the `code_interpreter` tool requires a list of file IDs, while the `file_search` tool requires a list of vector store IDs.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct ModifyThreadRequestToolResources {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub code_interpreter: ::std::option::Option<
        ModifyThreadRequestToolResourcesCodeInterpreter,
    >,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub file_search: ::std::option::Option<ModifyThreadRequestToolResourcesFileSearch>,
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct ModifyThreadRequestToolResourcesCodeInterpreter {
    ///A list of [file](/docs/api-reference/files) IDs made available to the `code_interpreter` tool. There can be a maximum of 20 files associated with the tool.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub file_ids: ::std::option::Option<Vec<String>>,
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct ModifyThreadRequestToolResourcesFileSearch {
    ///The [vector store](/docs/api-reference/vector-stores/object) attached to this thread. There can be a maximum of 1 vector store attached to the thread.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub vector_store_ids: ::std::option::Option<Vec<String>>,
}
///A mouse move action.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct MoveParam {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub keys: ::std::option::Option<Vec<String>>,
    ///Specifies the event type. For a move action, this property is always set to `move`.
    #[serde(rename = "type")]
    pub type_value: String,
    ///The x-coordinate to move to.
    pub x: i32,
    ///The y-coordinate to move to.
    pub y: i32,
}
///Groups function/custom tools under a shared namespace.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct NamespaceToolParam {
    ///A description of the namespace shown to the model.
    pub description: String,
    ///The namespace name used in tool calls (for example, `crm`).
    pub name: String,
    ///The function/custom tools available inside this namespace.
    pub tools: Vec<NamespaceToolParamTool>,
    ///The type of the tool. Always `namespace`.
    #[serde(rename = "type")]
    pub type_value: String,
}
///A function or custom tool that belongs to a namespace.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
#[serde(untagged)]
pub enum NamespaceToolParamTool {
    FunctionToolParam(FunctionToolParam),
    CustomToolParam(CustomToolParam),
}
///Type of noise reduction. `near_field` is for close-talking microphones such as headphones, `far_field` is for far-field microphones such as laptop or conference room microphones.
pub type NoiseReductionType = String;
///The `File` object represents a document that has been uploaded to OpenAI.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct OpenAiFile {
    ///The size of the file, in bytes.
    pub bytes: i32,
    ///The Unix timestamp (in seconds) for when the file was created.
    pub created_at: i64,
    ///The Unix timestamp (in seconds) for when the file will expire.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expires_at: ::std::option::Option<i64>,
    ///The name of the file.
    pub filename: String,
    ///The file identifier, which can be referenced in the API endpoints.
    pub id: String,
    ///The object type, which is always `file`.
    pub object: String,
    ///The intended purpose of the file. Supported values are `assistants`, `assistants_output`, `batch`, `batch_output`, `fine-tune`, `fine-tune-results`, `vision`, and `user_data`.
    pub purpose: String,
    ///Deprecated. The current status of the file, which can be either `uploaded`, `processed`, or `error`.
    pub status: String,
    ///Deprecated. For details on why a fine-tuning training file failed validation, see the `error` field on `fine_tuning.job`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub status_details: ::std::option::Option<String>,
}
pub type OrderEnum = String;
///Represents an individual certificate configured at the organization level.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct OrganizationCertificate {
    ///Whether the certificate is currently active at the organization level.
    pub active: bool,
    pub certificate_details: OrganizationCertificateCertificateDetails,
    ///The Unix timestamp (in seconds) of when the certificate was uploaded.
    pub created_at: i64,
    ///The identifier, which can be referenced in API endpoints
    pub id: String,
    ///The name of the certificate.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: ::std::option::Option<String>,
    ///The object type, which is always `organization.certificate`.
    pub object: String,
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct OrganizationCertificateActivationResponse {
    pub data: Vec<OrganizationCertificate>,
    ///The organization certificate activation result type.
    pub object: String,
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct OrganizationCertificateCertificateDetails {
    ///The Unix timestamp (in seconds) of when the certificate expires.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expires_at: ::std::option::Option<i64>,
    ///The Unix timestamp (in seconds) of when the certificate becomes valid.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub valid_at: ::std::option::Option<i64>,
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct OrganizationCertificateDeactivationResponse {
    pub data: Vec<OrganizationCertificate>,
    ///The organization certificate deactivation result type.
    pub object: String,
}
///Represents an individual certificate configured at the project level.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct OrganizationProjectCertificate {
    ///Whether the certificate is currently active at the project level.
    pub active: bool,
    pub certificate_details: OrganizationProjectCertificateCertificateDetails,
    ///The Unix timestamp (in seconds) of when the certificate was uploaded.
    pub created_at: i64,
    ///The identifier, which can be referenced in API endpoints
    pub id: String,
    ///The name of the certificate.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: ::std::option::Option<String>,
    ///The object type, which is always `organization.project.certificate`.
    pub object: String,
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct OrganizationProjectCertificateActivationResponse {
    pub data: Vec<OrganizationProjectCertificate>,
    ///The project certificate activation result type.
    pub object: String,
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct OrganizationProjectCertificateCertificateDetails {
    ///The Unix timestamp (in seconds) of when the certificate expires.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expires_at: ::std::option::Option<i64>,
    ///The Unix timestamp (in seconds) of when the certificate becomes valid.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub valid_at: ::std::option::Option<i64>,
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct OrganizationProjectCertificateDeactivationResponse {
    pub data: Vec<OrganizationProjectCertificate>,
    ///The project certificate deactivation result type.
    pub object: String,
}
///This is returned when the chunking strategy is unknown. Typically, this is because the file was indexed before the `chunking_strategy` concept was introduced in the API.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct OtherChunkingStrategyResponseParam {
    ///Always `other`.
    #[serde(rename = "type")]
    pub type_value: String,
}
///An audio output from the model.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct OutputAudio {
    ///Base64-encoded audio data from the model.
    pub data: String,
    ///The transcript of the audio data from the model.
    pub transcript: String,
    ///The type of the output audio. Always `output_audio`.
    #[serde(rename = "type")]
    pub type_value: String,
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
#[serde(untagged)]
pub enum OutputContent {
    OutputTextContent(OutputTextContent),
    RefusalContent(RefusalContent),
    ReasoningTextContent(ReasoningTextContent),
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
#[serde(untagged)]
pub enum OutputItem {
    OutputMessage(OutputMessage),
    FileSearchToolCall(FileSearchToolCall),
    FunctionToolCall(FunctionToolCall),
    FunctionToolCallOutputResource(FunctionToolCallOutputResource),
    WebSearchToolCall(WebSearchToolCall),
    ComputerToolCall(ComputerToolCall),
    ComputerToolCallOutputResource(ComputerToolCallOutputResource),
    ReasoningItem(ReasoningItem),
    ToolSearchCall(ToolSearchCall),
    ToolSearchOutput(ToolSearchOutput),
    CompactionBody(CompactionBody),
    ImageGenToolCall(ImageGenToolCall),
    CodeInterpreterToolCall(CodeInterpreterToolCall),
    LocalShellToolCall(LocalShellToolCall),
    LocalShellToolCallOutput(LocalShellToolCallOutput),
    FunctionShellCall(FunctionShellCall),
    FunctionShellCallOutput(FunctionShellCallOutput),
    ApplyPatchToolCall(ApplyPatchToolCall),
    ApplyPatchToolCallOutput(ApplyPatchToolCallOutput),
    McpToolCall(McpToolCall),
    McpListTools(McpListTools),
    McpApprovalRequest(McpApprovalRequest),
    McpApprovalResponseResource(McpApprovalResponseResource),
    CustomToolCall(CustomToolCall),
    CustomToolCallOutputResource(CustomToolCallOutputResource),
}
///An output message from the model.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct OutputMessage {
    ///The content of the output message.
    pub content: Vec<OutputMessageContent>,
    ///The unique ID of the output message.
    pub id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub phase: ::std::option::Option<MessagePhase>,
    ///The role of the output message. Always `assistant`.
    pub role: String,
    ///The status of the message input. One of `in_progress`, `completed`, or `incomplete`. Populated when input items are returned via API.
    pub status: String,
    ///The type of the output message. Always `message`.
    #[serde(rename = "type")]
    pub type_value: String,
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
#[serde(untagged)]
pub enum OutputMessageContent {
    OutputTextContent(OutputTextContent),
    RefusalContent(RefusalContent),
}
///A text output from the model.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct OutputTextContent {
    ///The annotations of the text output.
    pub annotations: Vec<Annotation>,
    pub logprobs: Vec<LogProb>,
    ///The text output from the model.
    pub text: String,
    ///The type of the output text. Always `output_text`.
    #[serde(rename = "type")]
    pub type_value: String,
}
///Whether to enable [parallel function calling](/docs/guides/function-calling#configuring-parallel-function-calling) during tool use.
pub type ParallelToolCalls = bool;
pub type PartialImages = i32;
///Static predicted output content, such as the content of a text file that is being regenerated.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct PredictionContent {
    ///The content that should be matched when generating a model response. If generated tokens would match this content, the entire model response can be returned much more quickly.
    pub content: PredictionContentContent,
    ///The type of the predicted content you want to provide. This type is currently always `content`.
    #[serde(rename = "type")]
    pub type_value: String,
}
///The content that should be matched when generating a model response. If generated tokens would match this content, the entire model response can be returned much more quickly.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
#[serde(untagged)]
pub enum PredictionContentContent {
    TextContent(String),
    ArrayOfContentParts(Vec<ChatCompletionRequestMessageContentPartText>),
}
///Represents an individual project.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct Project {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub archived_at: ::std::option::Option<i64>,
    ///The Unix timestamp (in seconds) of when the project was created.
    pub created_at: i64,
    ///The external key associated with the project.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub external_key_id: ::std::option::Option<String>,
    ///The identifier, which can be referenced in API endpoints
    pub id: String,
    ///The name of the project. This appears in reporting.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: ::std::option::Option<String>,
    ///The object type, which is always `organization.project`
    pub object: String,
    ///`active` or `archived`
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub status: ::std::option::Option<String>,
}
///Represents an individual API key in a project.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct ProjectApiKey {
    ///The Unix timestamp (in seconds) of when the API key was created
    pub created_at: i64,
    ///The identifier, which can be referenced in API endpoints
    pub id: String,
    ///The Unix timestamp (in seconds) of when the API key was last used.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_used_at: ::std::option::Option<i64>,
    ///The name of the API key
    pub name: String,
    ///The object type, which is always `organization.project.api_key`
    pub object: String,
    pub owner: ProjectApiKeyOwner,
    ///The redacted value of the API key
    pub redacted_value: String,
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct ProjectApiKeyDeleteResponse {
    pub deleted: bool,
    pub id: String,
    pub object: String,
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct ProjectApiKeyListResponse {
    pub data: Vec<ProjectApiKey>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub first_id: ::std::option::Option<String>,
    pub has_more: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_id: ::std::option::Option<String>,
    pub object: String,
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct ProjectApiKeyOwner {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub service_account: ::std::option::Option<ProjectApiKeyOwnerServiceAccount>,
    ///`user` or `service_account`
    #[serde(rename = "type", default, skip_serializing_if = "Option::is_none")]
    pub type_value: ::std::option::Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub user: ::std::option::Option<ProjectApiKeyOwnerUser>,
}
///The service account that owns a project API key.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct ProjectApiKeyOwnerServiceAccount {
    ///The Unix timestamp (in seconds) of when the service account was created.
    pub created_at: i64,
    ///The identifier, which can be referenced in API endpoints
    pub id: String,
    ///The name of the service account.
    pub name: String,
    ///The service account's project role.
    pub role: String,
}
///The user that owns a project API key.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct ProjectApiKeyOwnerUser {
    ///The Unix timestamp (in seconds) of when the user was created.
    pub created_at: i64,
    ///The email address of the user.
    pub email: String,
    ///The identifier, which can be referenced in API endpoints
    pub id: String,
    ///The name of the user.
    pub name: String,
    ///The user's project role.
    pub role: String,
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct ProjectCreateRequest {
    ///External key ID to associate with the project.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub external_key_id: ::std::option::Option<String>,
    ///Create the project with the specified data residency region. Your organization must have access to Data residency functionality in order to use. See [data residency controls](/docs/guides/your-data#data-residency-controls) to review the functionality and limitations of setting this field.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub geography: ::std::option::Option<String>,
    ///The friendly name of the project, this name appears in reports.
    pub name: String,
}
///Details about a group's membership in a project.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct ProjectGroup {
    ///Unix timestamp (in seconds) when the group was granted project access.
    pub created_at: i64,
    ///Identifier of the group that has access to the project.
    pub group_id: String,
    ///Display name of the group.
    pub group_name: String,
    ///The type of the group.
    pub group_type: String,
    ///Always `project.group`.
    pub object: String,
    ///Identifier of the project.
    pub project_id: String,
}
///Confirmation payload returned after removing a group from a project.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct ProjectGroupDeletedResource {
    ///Whether the group membership in the project was removed.
    pub deleted: bool,
    ///Always `project.group.deleted`.
    pub object: String,
}
///Paginated list of groups that have access to a project.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct ProjectGroupListResource {
    ///Project group memberships returned in the current page.
    pub data: Vec<ProjectGroup>,
    ///Whether additional project group memberships are available.
    pub has_more: bool,
    ///Cursor to fetch the next page of results, or `null` when there are no more results.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub next: ::std::option::Option<String>,
    ///Always `list`.
    pub object: String,
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct ProjectListResponse {
    pub data: Vec<Project>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub first_id: ::std::option::Option<String>,
    pub has_more: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_id: ::std::option::Option<String>,
    pub object: String,
}
///Represents a project rate limit config.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct ProjectRateLimit {
    ///The maximum batch input tokens per day. Only present for relevant models.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub batch_1_day_max_input_tokens: ::std::option::Option<i32>,
    ///The identifier, which can be referenced in API endpoints.
    pub id: String,
    ///The maximum audio megabytes per minute. Only present for relevant models.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_audio_megabytes_per_1_minute: ::std::option::Option<i32>,
    ///The maximum images per minute. Only present for relevant models.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_images_per_1_minute: ::std::option::Option<i32>,
    ///The maximum requests per day. Only present for relevant models.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_requests_per_1_day: ::std::option::Option<i32>,
    ///The maximum requests per minute.
    pub max_requests_per_1_minute: i32,
    ///The maximum tokens per minute.
    pub max_tokens_per_1_minute: i32,
    ///The model this rate limit applies to.
    pub model: String,
    ///The object type, which is always `project.rate_limit`
    pub object: String,
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct ProjectRateLimitListResponse {
    pub data: Vec<ProjectRateLimit>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub first_id: ::std::option::Option<String>,
    pub has_more: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_id: ::std::option::Option<String>,
    pub object: String,
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct ProjectRateLimitUpdateRequest {
    ///The maximum batch input tokens per day. Only relevant for certain models.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub batch_1_day_max_input_tokens: ::std::option::Option<i32>,
    ///The maximum audio megabytes per minute. Only relevant for certain models.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_audio_megabytes_per_1_minute: ::std::option::Option<i32>,
    ///The maximum images per minute. Only relevant for certain models.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_images_per_1_minute: ::std::option::Option<i32>,
    ///The maximum requests per day. Only relevant for certain models.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_requests_per_1_day: ::std::option::Option<i32>,
    ///The maximum requests per minute.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_requests_per_1_minute: ::std::option::Option<i32>,
    ///The maximum tokens per minute.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_tokens_per_1_minute: ::std::option::Option<i32>,
}
///Represents an individual service account in a project.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct ProjectServiceAccount {
    ///The Unix timestamp (in seconds) of when the service account was created
    pub created_at: i64,
    ///The identifier, which can be referenced in API endpoints
    pub id: String,
    ///The name of the service account
    pub name: String,
    ///The object type, which is always `organization.project.service_account`
    pub object: String,
    ///`owner` or `member`
    pub role: String,
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct ProjectServiceAccountApiKey {
    pub created_at: i64,
    pub id: String,
    pub name: String,
    ///The object type, which is always `organization.project.service_account.api_key`
    pub object: String,
    pub value: String,
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct ProjectServiceAccountCreateRequest {
    ///The name of the service account being created.
    pub name: String,
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct ProjectServiceAccountCreateResponse {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub api_key: ::std::option::Option<ProjectServiceAccountApiKey>,
    pub created_at: i64,
    pub id: String,
    pub name: String,
    pub object: String,
    ///Service accounts can only have one role of type `member`
    pub role: String,
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct ProjectServiceAccountDeleteResponse {
    pub deleted: bool,
    pub id: String,
    pub object: String,
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct ProjectServiceAccountListResponse {
    pub data: Vec<ProjectServiceAccount>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub first_id: ::std::option::Option<String>,
    pub has_more: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_id: ::std::option::Option<String>,
    pub object: String,
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct ProjectUpdateRequest {
    ///External key ID to associate with the project.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub external_key_id: ::std::option::Option<String>,
    ///Geography for the project.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub geography: ::std::option::Option<String>,
    ///The updated name of the project, this name appears in reports.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: ::std::option::Option<String>,
}
///Represents an individual user in a project.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct ProjectUser {
    ///The Unix timestamp (in seconds) of when the project was added.
    pub added_at: i64,
    ///The email address of the user
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub email: ::std::option::Option<String>,
    ///The identifier, which can be referenced in API endpoints
    pub id: String,
    ///The name of the user
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: ::std::option::Option<String>,
    ///The object type, which is always `organization.project.user`
    pub object: String,
    ///`owner` or `member`
    pub role: String,
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct ProjectUserCreateRequest {
    ///Email of the user to add.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub email: ::std::option::Option<String>,
    ///`owner` or `member`
    pub role: String,
    ///The ID of the user.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub user_id: ::std::option::Option<String>,
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct ProjectUserDeleteResponse {
    pub deleted: bool,
    pub id: String,
    pub object: String,
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct ProjectUserListResponse {
    pub data: Vec<ProjectUser>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub first_id: ::std::option::Option<String>,
    pub has_more: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_id: ::std::option::Option<String>,
    pub object: String,
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct ProjectUserUpdateRequest {
    ///`owner` or `member`
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub role: ::std::option::Option<String>,
}
pub type Prompt = Prompt2;
///Reference to a prompt template and its variables. [Learn more](/docs/guides/text?api-mode=responses#reusable-prompts).
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct Prompt2 {
    ///The unique identifier of the prompt template to use.
    pub id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub variables: ::std::option::Option<ResponsePromptVariables>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub version: ::std::option::Option<String>,
}
pub type PromptCacheRetentionEnum = String;
///Request payload for assigning a role to a group or user.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct PublicAssignOrganizationGroupRoleBody {
    ///Identifier of the role to assign.
    pub role_id: String,
}
///Request payload for creating a custom role.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct PublicCreateOrganizationRoleBody {
    ///Optional description of the role.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: ::std::option::Option<String>,
    ///Permissions to grant to the role.
    pub permissions: Vec<String>,
    ///Unique name for the role.
    pub role_name: String,
}
///Paginated list of roles available on an organization or project.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct PublicRoleListResource {
    ///Roles returned in the current page.
    pub data: Vec<Role>,
    ///Whether more roles are available when paginating.
    pub has_more: bool,
    ///Cursor to fetch the next page of results, or `null` when there are no additional roles.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub next: ::std::option::Option<String>,
    ///Always `list`.
    pub object: String,
}
///Request payload for updating an existing role.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct PublicUpdateOrganizationRoleBody {
    ///New description for the role.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: ::std::option::Option<String>,
    ///Updated set of permissions for the role.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub permissions: ::std::option::Option<Vec<String>>,
    ///New name for the role.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub role_name: ::std::option::Option<String>,
}
pub type RankerVersionType = String;
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct RankingOptions {
    ///Weights that control how reciprocal rank fusion balances semantic embedding matches versus sparse keyword matches when hybrid search is enabled.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hybrid_search: ::std::option::Option<HybridSearchOptions>,
    ///The ranker to use for the file search.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ranker: ::std::option::Option<RankerVersionType>,
    ///The score threshold for the file search, a number between 0 and 1. Numbers closer to 1 will attempt to return only the most relevant results, but may return fewer results.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub score_threshold: ::std::option::Option<f64>,
}
///Controls request rate limits for the session.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct RateLimitsParam {
    ///Maximum number of requests allowed per minute for the session. Defaults to 10.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_requests_per_1_minute: ::std::option::Option<i32>,
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
#[serde(untagged)]
pub enum RealtimeAudioFormats {
    PcmAudioFormat(RealtimeAudioFormatsPcmAudioFormat),
    PcmuAudioFormat(RealtimeAudioFormatsPcmuAudioFormat),
    PcmaAudioFormat(RealtimeAudioFormatsPcmaAudioFormat),
}
///The PCM audio format. Only a 24kHz sample rate is supported.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct RealtimeAudioFormatsPcmAudioFormat {
    ///The sample rate of the audio. Always `24000`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rate: ::std::option::Option<i32>,
    ///The audio format. Always `audio/pcm`.
    #[serde(rename = "type", default, skip_serializing_if = "Option::is_none")]
    pub type_value: ::std::option::Option<String>,
}
///The G.711 A-law format.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct RealtimeAudioFormatsPcmaAudioFormat {
    ///The audio format. Always `audio/pcma`.
    #[serde(rename = "type", default, skip_serializing_if = "Option::is_none")]
    pub type_value: ::std::option::Option<String>,
}
///The G.711 μ-law format.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct RealtimeAudioFormatsPcmuAudioFormat {
    ///The audio format. Always `audio/pcmu`.
    #[serde(rename = "type", default, skip_serializing_if = "Option::is_none")]
    pub type_value: ::std::option::Option<String>,
}
///Add a new Item to the Conversation's context, including messages, function calls, and function call responses. This event can be used both to populate a "history" of the conversation and to add new items mid-stream, but has the current limitation that it cannot populate assistant audio messages. If successful, the server will respond with a `conversation.item.created` event, otherwise an `error` event will be sent.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct RealtimeBetaClientEventConversationItemCreate {
    ///Optional client-generated ID used to identify this event.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub event_id: ::std::option::Option<String>,
    pub item: RealtimeConversationItem,
    ///The ID of the preceding item after which the new item will be inserted. If not set, the new item will be appended to the end of the conversation. If set to `root`, the new item will be added to the beginning of the conversation. If set to an existing ID, it allows an item to be inserted mid-conversation. If the ID cannot be found, an error will be returned and the item will not be added.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub previous_item_id: ::std::option::Option<String>,
    ///The event type, must be `conversation.item.create`.
    #[serde(rename = "type")]
    pub type_value: String,
}
///Send this event when you want to remove any item from the conversation history. The server will respond with a `conversation.item.deleted` event, unless the item does not exist in the conversation history, in which case the server will respond with an error.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct RealtimeBetaClientEventConversationItemDelete {
    ///Optional client-generated ID used to identify this event.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub event_id: ::std::option::Option<String>,
    ///The ID of the item to delete.
    pub item_id: String,
    ///The event type, must be `conversation.item.delete`.
    #[serde(rename = "type")]
    pub type_value: String,
}
///Send this event when you want to retrieve the server's representation of a specific item in the conversation history. This is useful, for example, to inspect user audio after noise cancellation and VAD. The server will respond with a `conversation.item.retrieved` event, unless the item does not exist in the conversation history, in which case the server will respond with an error.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct RealtimeBetaClientEventConversationItemRetrieve {
    ///Optional client-generated ID used to identify this event.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub event_id: ::std::option::Option<String>,
    ///The ID of the item to retrieve.
    pub item_id: String,
    ///The event type, must be `conversation.item.retrieve`.
    #[serde(rename = "type")]
    pub type_value: String,
}
///Send this event to truncate a previous assistant message’s audio. The server will produce audio faster than realtime, so this event is useful when the user interrupts to truncate audio that has already been sent to the client but not yet played. This will synchronize the server's understanding of the audio with the client's playback. Truncating audio will delete the server-side text transcript to ensure there is not text in the context that hasn't been heard by the user. If successful, the server will respond with a `conversation.item.truncated` event.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct RealtimeBetaClientEventConversationItemTruncate {
    ///Inclusive duration up to which audio is truncated, in milliseconds. If the audio_end_ms is greater than the actual audio duration, the server will respond with an error.
    pub audio_end_ms: i32,
    ///The index of the content part to truncate. Set this to 0.
    pub content_index: i32,
    ///Optional client-generated ID used to identify this event.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub event_id: ::std::option::Option<String>,
    ///The ID of the assistant message item to truncate. Only assistant message items can be truncated.
    pub item_id: String,
    ///The event type, must be `conversation.item.truncate`.
    #[serde(rename = "type")]
    pub type_value: String,
}
///Send this event to append audio bytes to the input audio buffer. The audio buffer is temporary storage you can write to and later commit. In Server VAD mode, the audio buffer is used to detect speech and the server will decide when to commit. When Server VAD is disabled, you must commit the audio buffer manually. The client may choose how much audio to place in each event up to a maximum of 15 MiB, for example streaming smaller chunks from the client may allow the VAD to be more responsive. Unlike made other client events, the server will not send a confirmation response to this event.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct RealtimeBetaClientEventInputAudioBufferAppend {
    ///Base64-encoded audio bytes. This must be in the format specified by the `input_audio_format` field in the session configuration.
    pub audio: String,
    ///Optional client-generated ID used to identify this event.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub event_id: ::std::option::Option<String>,
    ///The event type, must be `input_audio_buffer.append`.
    #[serde(rename = "type")]
    pub type_value: String,
}
///Send this event to clear the audio bytes in the buffer. The server will respond with an `input_audio_buffer.cleared` event.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct RealtimeBetaClientEventInputAudioBufferClear {
    ///Optional client-generated ID used to identify this event.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub event_id: ::std::option::Option<String>,
    ///The event type, must be `input_audio_buffer.clear`.
    #[serde(rename = "type")]
    pub type_value: String,
}
///Send this event to commit the user input audio buffer, which will create a new user message item in the conversation. This event will produce an error if the input audio buffer is empty. When in Server VAD mode, the client does not need to send this event, the server will commit the audio buffer automatically. Committing the input audio buffer will trigger input audio transcription (if enabled in session configuration), but it will not create a response from the model. The server will respond with an `input_audio_buffer.committed` event.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct RealtimeBetaClientEventInputAudioBufferCommit {
    ///Optional client-generated ID used to identify this event.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub event_id: ::std::option::Option<String>,
    ///The event type, must be `input_audio_buffer.commit`.
    #[serde(rename = "type")]
    pub type_value: String,
}
///**WebRTC/SIP Only:** Emit to cut off the current audio response. This will trigger the server to stop generating audio and emit a `output_audio_buffer.cleared` event. This event should be preceded by a `response.cancel` client event to stop the generation of the current response. [Learn more](/docs/guides/realtime-conversations#client-and-server-events-for-audio-in-webrtc).
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct RealtimeBetaClientEventOutputAudioBufferClear {
    ///The unique ID of the client event used for error handling.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub event_id: ::std::option::Option<String>,
    ///The event type, must be `output_audio_buffer.clear`.
    #[serde(rename = "type")]
    pub type_value: String,
}
///Send this event to cancel an in-progress response. The server will respond with a `response.done` event with a status of `response.status=cancelled`. If there is no response to cancel, the server will respond with an error.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct RealtimeBetaClientEventResponseCancel {
    ///Optional client-generated ID used to identify this event.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub event_id: ::std::option::Option<String>,
    ///A specific response ID to cancel - if not provided, will cancel an in-progress response in the default conversation.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub response_id: ::std::option::Option<String>,
    ///The event type, must be `response.cancel`.
    #[serde(rename = "type")]
    pub type_value: String,
}
///This event instructs the server to create a Response, which means triggering model inference. When in Server VAD mode, the server will create Responses automatically. A Response will include at least one Item, and may have two, in which case the second will be a function call. These Items will be appended to the conversation history. The server will respond with a `response.created` event, events for Items and content created, and finally a `response.done` event to indicate the Response is complete. The `response.create` event can optionally include inference configuration like `instructions`, and `temperature`. These fields will override the Session's configuration for this Response only. Responses can be created out-of-band of the default Conversation, meaning that they can have arbitrary input, and it's possible to disable writing the output to the Conversation. Only one Response can write to the default Conversation at a time, but otherwise multiple Responses can be created in parallel. Clients can set `conversation` to `none` to create a Response that does not write to the default Conversation. Arbitrary input can be provided with the `input` field, which is an array accepting raw Items and references to existing Items.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct RealtimeBetaClientEventResponseCreate {
    ///Optional client-generated ID used to identify this event.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub event_id: ::std::option::Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub response: ::std::option::Option<RealtimeBetaResponseCreateParams>,
    ///The event type, must be `response.create`.
    #[serde(rename = "type")]
    pub type_value: String,
}
///Send this event to update the session’s default configuration. The client may send this event at any time to update any field, except for `voice`. However, note that once a session has been initialized with a particular `model`, it can’t be changed to another model using `session.update`. When the server receives a `session.update`, it will respond with a `session.updated` event showing the full, effective configuration. Only the fields that are present are updated. To clear a field like `instructions`, pass an empty string.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct RealtimeBetaClientEventSessionUpdate {
    ///Optional client-generated ID used to identify this event.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub event_id: ::std::option::Option<String>,
    pub session: RealtimeSessionCreateRequest,
    ///The event type, must be `session.update`.
    #[serde(rename = "type")]
    pub type_value: String,
}
///Send this event to update a transcription session.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct RealtimeBetaClientEventTranscriptionSessionUpdate {
    ///Optional client-generated ID used to identify this event.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub event_id: ::std::option::Option<String>,
    pub session: RealtimeTranscriptionSessionCreateRequest,
    ///The event type, must be `transcription_session.update`.
    #[serde(rename = "type")]
    pub type_value: String,
}
///The response resource.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct RealtimeBetaResponse {
    ///Which conversation the response is added to, determined by the `conversation` field in the `response.create` event. If `auto`, the response will be added to the default conversation and the value of `conversation_id` will be an id like `conv_1234`. If `none`, the response will not be added to any conversation and the value of `conversation_id` will be `null`. If responses are being triggered by server VAD, the response will be added to the default conversation, thus the `conversation_id` will be an id like `conv_1234`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub conversation_id: ::std::option::Option<String>,
    ///The unique ID of the response.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: ::std::option::Option<String>,
    ///Maximum number of output tokens for a single assistant response, inclusive of tool calls, that was used in this response.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_output_tokens: ::std::option::Option<RealtimeBetaResponseMaxOutputTokens>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub metadata: ::std::option::Option<Metadata>,
    ///The set of modalities the model used to respond. If there are multiple modalities, the model will pick one, for example if `modalities` is `["text", "audio"]`, the model could be responding in either text or audio.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub modalities: ::std::option::Option<Vec<String>>,
    ///The object type, must be `realtime.response`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub object: ::std::option::Option<String>,
    ///The list of output items generated by the response.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub output: ::std::option::Option<Vec<RealtimeConversationItem>>,
    ///The format of output audio. Options are `pcm16`, `g711_ulaw`, or `g711_alaw`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub output_audio_format: ::std::option::Option<String>,
    ///The final status of the response (`completed`, `cancelled`, `failed`, or `incomplete`, `in_progress`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub status: ::std::option::Option<String>,
    ///Additional details about the status.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub status_details: ::std::option::Option<RealtimeBetaResponseStatusDetails>,
    ///Sampling temperature for the model, limited to [0.6, 1.2]. Defaults to 0.8.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub temperature: ::std::option::Option<f64>,
    ///Usage statistics for the Response, this will correspond to billing. A Realtime API session will maintain a conversation context and append new Items to the Conversation, thus output from previous turns (text and audio tokens) will become the input for later turns.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub usage: ::std::option::Option<RealtimeBetaResponseUsage>,
    ///The voice the model used to respond. Current voice options are `alloy`, `ash`, `ballad`, `coral`, `echo`, `sage`, `shimmer`, and `verse`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub voice: ::std::option::Option<VoiceIdsShared>,
}
///Create a new Realtime response with these parameters
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct RealtimeBetaResponseCreateParams {
    ///Controls which conversation the response is added to. Currently supports `auto` and `none`, with `auto` as the default value. The `auto` value means that the contents of the response will be added to the default conversation. Set this to `none` to create an out-of-band response which will not add items to default conversation.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub conversation: ::std::option::Option<String>,
    ///Input items to include in the prompt for the model. Using this field creates a new context for this Response instead of using the default conversation. An empty array `[]` will clear the context for this Response. Note that this can include references to items from the default conversation.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub input: ::std::option::Option<Vec<RealtimeConversationItem>>,
    ///The default system instructions (i.e. system message) prepended to model calls. This field allows the client to guide the model on desired responses. The model can be instructed on response content and format, (e.g. "be extremely succinct", "act friendly", "here are examples of good responses") and on audio behavior (e.g. "talk quickly", "inject emotion into your voice", "laugh frequently"). The instructions are not guaranteed to be followed by the model, but they provide guidance to the model on the desired behavior. Note that the server sets default instructions which will be used if this field is not set and are visible in the `session.created` event at the start of the session.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub instructions: ::std::option::Option<String>,
    ///Maximum number of output tokens for a single assistant response, inclusive of tool calls. Provide an integer between 1 and 4096 to limit output tokens, or `inf` for the maximum available tokens for a given model. Defaults to `inf`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_output_tokens: ::std::option::Option<
        RealtimeBetaResponseCreateParamsMaxOutputTokens,
    >,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub metadata: ::std::option::Option<Metadata>,
    ///The set of modalities the model can respond with. To disable audio, set this to ["text"].
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub modalities: ::std::option::Option<Vec<String>>,
    ///The format of output audio. Options are `pcm16`, `g711_ulaw`, or `g711_alaw`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub output_audio_format: ::std::option::Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub prompt: ::std::option::Option<Prompt>,
    ///Sampling temperature for the model, limited to [0.6, 1.2]. Defaults to 0.8.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub temperature: ::std::option::Option<f64>,
    ///How the model chooses tools. Provide one of the string modes or force a specific function/MCP tool.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tool_choice: ::std::option::Option<RealtimeBetaResponseCreateParamsToolChoice>,
    ///Tools (functions) available to the model.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tools: ::std::option::Option<Vec<RealtimeBetaResponseCreateParamsTool>>,
    ///The voice the model uses to respond. Supported built-in voices are `alloy`, `ash`, `ballad`, `coral`, `echo`, `sage`, `shimmer`, `verse`, `marin`, and `cedar`. You may also provide a custom voice object with an `id`, for example `{ "id": "voice_1234" }`. Voice cannot be changed during the session once the model has responded with audio at least once.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub voice: ::std::option::Option<VoiceIdsOrCustomVoice>,
}
///Maximum number of output tokens for a single assistant response, inclusive of tool calls. Provide an integer between 1 and 4096 to limit output tokens, or `inf` for the maximum available tokens for a given model. Defaults to `inf`.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
#[serde(untagged)]
pub enum RealtimeBetaResponseCreateParamsMaxOutputTokens {
    Integer(i32),
    Inf(String),
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct RealtimeBetaResponseCreateParamsTool {
    ///The description of the function, including guidance on when and how to call it, and guidance about what to tell the user when calling (if anything).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: ::std::option::Option<String>,
    ///The name of the function.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: ::std::option::Option<String>,
    ///Parameters of the function in JSON Schema.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parameters: ::std::option::Option<OpenAiJsonValue>,
    ///The type of the tool, i.e. `function`.
    #[serde(rename = "type", default, skip_serializing_if = "Option::is_none")]
    pub type_value: ::std::option::Option<String>,
}
///How the model chooses tools. Provide one of the string modes or force a specific function/MCP tool.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
#[serde(untagged)]
pub enum RealtimeBetaResponseCreateParamsToolChoice {
    ToolChoiceOptions(ToolChoiceOptions),
    ToolChoiceFunction(ToolChoiceFunction),
    ToolChoiceMcp(ToolChoiceMcp),
}
///Maximum number of output tokens for a single assistant response, inclusive of tool calls, that was used in this response.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
#[serde(untagged)]
pub enum RealtimeBetaResponseMaxOutputTokens {
    Integer(i32),
    Inf(String),
}
///Additional details about the status.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct RealtimeBetaResponseStatusDetails {
    ///A description of the error that caused the response to fail, populated when the `status` is `failed`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: ::std::option::Option<RealtimeBetaResponseStatusDetailsError>,
    ///The reason the Response did not complete. For a `cancelled` Response, one of `turn_detected` (the server VAD detected a new start of speech) or `client_cancelled` (the client sent a cancel event). For an `incomplete` Response, one of `max_output_tokens` or `content_filter` (the server-side safety filter activated and cut off the response).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reason: ::std::option::Option<String>,
    ///The type of error that caused the response to fail, corresponding with the `status` field (`completed`, `cancelled`, `incomplete`, `failed`).
    #[serde(rename = "type", default, skip_serializing_if = "Option::is_none")]
    pub type_value: ::std::option::Option<String>,
}
///A description of the error that caused the response to fail, populated when the `status` is `failed`.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct RealtimeBetaResponseStatusDetailsError {
    ///Error code, if any.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub code: ::std::option::Option<String>,
    ///The type of error.
    #[serde(rename = "type", default, skip_serializing_if = "Option::is_none")]
    pub type_value: ::std::option::Option<String>,
}
///Usage statistics for the Response, this will correspond to billing. A Realtime API session will maintain a conversation context and append new Items to the Conversation, thus output from previous turns (text and audio tokens) will become the input for later turns.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct RealtimeBetaResponseUsage {
    ///Details about the input tokens used in the Response.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub input_token_details: ::std::option::Option<
        RealtimeBetaResponseUsageInputTokenDetails,
    >,
    ///The number of input tokens used in the Response, including text and audio tokens.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub input_tokens: ::std::option::Option<i32>,
    ///Details about the output tokens used in the Response.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub output_token_details: ::std::option::Option<
        RealtimeBetaResponseUsageOutputTokenDetails,
    >,
    ///The number of output tokens sent in the Response, including text and audio tokens.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub output_tokens: ::std::option::Option<i32>,
    ///The total number of tokens in the Response including input and output text and audio tokens.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub total_tokens: ::std::option::Option<i32>,
}
///Details about the input tokens used in the Response.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct RealtimeBetaResponseUsageInputTokenDetails {
    ///The number of audio tokens used as input for the Response.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub audio_tokens: ::std::option::Option<i32>,
    ///The number of cached tokens used as input for the Response.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cached_tokens: ::std::option::Option<i32>,
    ///Details about the cached tokens used as input for the Response.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cached_tokens_details: ::std::option::Option<
        RealtimeBetaResponseUsageInputTokenDetailsCachedTokensDetails,
    >,
    ///The number of image tokens used as input for the Response.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub image_tokens: ::std::option::Option<i32>,
    ///The number of text tokens used as input for the Response.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub text_tokens: ::std::option::Option<i32>,
}
///Details about the cached tokens used as input for the Response.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct RealtimeBetaResponseUsageInputTokenDetailsCachedTokensDetails {
    ///The number of cached audio tokens used as input for the Response.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub audio_tokens: ::std::option::Option<i32>,
    ///The number of cached image tokens used as input for the Response.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub image_tokens: ::std::option::Option<i32>,
    ///The number of cached text tokens used as input for the Response.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub text_tokens: ::std::option::Option<i32>,
}
///Details about the output tokens used in the Response.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct RealtimeBetaResponseUsageOutputTokenDetails {
    ///The number of audio tokens used in the Response.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub audio_tokens: ::std::option::Option<i32>,
    ///The number of text tokens used in the Response.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub text_tokens: ::std::option::Option<i32>,
}
///Returned when a conversation item is created. There are several scenarios that produce this event: - The server is generating a Response, which if successful will produce either one or two Items, which will be of type `message` (role `assistant`) or type `function_call`. - The input audio buffer has been committed, either by the client or the server (in `server_vad` mode). The server will take the content of the input audio buffer and add it to a new user message Item. - The client has sent a `conversation.item.create` event to add a new Item to the Conversation.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct RealtimeBetaServerEventConversationItemCreated {
    ///The unique ID of the server event.
    pub event_id: String,
    pub item: RealtimeConversationItem,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub previous_item_id: ::std::option::Option<String>,
    ///The event type, must be `conversation.item.created`.
    #[serde(rename = "type")]
    pub type_value: String,
}
///Returned when an item in the conversation is deleted by the client with a `conversation.item.delete` event. This event is used to synchronize the server's understanding of the conversation history with the client's view.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct RealtimeBetaServerEventConversationItemDeleted {
    ///The unique ID of the server event.
    pub event_id: String,
    ///The ID of the item that was deleted.
    pub item_id: String,
    ///The event type, must be `conversation.item.deleted`.
    #[serde(rename = "type")]
    pub type_value: String,
}
///This event is the output of audio transcription for user audio written to the user audio buffer. Transcription begins when the input audio buffer is committed by the client or server (in `server_vad` mode). Transcription runs asynchronously with Response creation, so this event may come before or after the Response events. Realtime API models accept audio natively, and thus input transcription is a separate process run on a separate ASR (Automatic Speech Recognition) model. The transcript may diverge somewhat from the model's interpretation, and should be treated as a rough guide.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct RealtimeBetaServerEventConversationItemInputAudioTranscriptionCompleted {
    ///The index of the content part containing the audio.
    pub content_index: i32,
    ///The unique ID of the server event.
    pub event_id: String,
    ///The ID of the user message item containing the audio.
    pub item_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub logprobs: ::std::option::Option<Vec<LogProbProperties>>,
    ///The transcribed text.
    pub transcript: String,
    ///The event type, must be `conversation.item.input_audio_transcription.completed`.
    #[serde(rename = "type")]
    pub type_value: String,
    ///Usage statistics for the transcription.
    pub usage: RealtimeBetaServerEventConversationItemInputAudioTranscriptionCompletedUsage,
}
///Usage statistics for the transcription.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
#[serde(untagged)]
pub enum RealtimeBetaServerEventConversationItemInputAudioTranscriptionCompletedUsage {
    TranscriptTextUsageTokens(TranscriptTextUsageTokens),
    TranscriptTextUsageDuration(TranscriptTextUsageDuration),
}
///Returned when the text value of an input audio transcription content part is updated.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct RealtimeBetaServerEventConversationItemInputAudioTranscriptionDelta {
    ///The index of the content part in the item's content array.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub content_index: ::std::option::Option<i32>,
    ///The text delta.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub delta: ::std::option::Option<String>,
    ///The unique ID of the server event.
    pub event_id: String,
    ///The ID of the item.
    pub item_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub logprobs: ::std::option::Option<Vec<LogProbProperties>>,
    ///The event type, must be `conversation.item.input_audio_transcription.delta`.
    #[serde(rename = "type")]
    pub type_value: String,
}
///Returned when input audio transcription is configured, and a transcription request for a user message failed. These events are separate from other `error` events so that the client can identify the related Item.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct RealtimeBetaServerEventConversationItemInputAudioTranscriptionFailed {
    ///The index of the content part containing the audio.
    pub content_index: i32,
    ///Details of the transcription error.
    pub error: RealtimeBetaServerEventConversationItemInputAudioTranscriptionFailedError,
    ///The unique ID of the server event.
    pub event_id: String,
    ///The ID of the user message item.
    pub item_id: String,
    ///The event type, must be `conversation.item.input_audio_transcription.failed`.
    #[serde(rename = "type")]
    pub type_value: String,
}
///Details of the transcription error.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct RealtimeBetaServerEventConversationItemInputAudioTranscriptionFailedError {
    ///Error code, if any.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub code: ::std::option::Option<String>,
    ///A human-readable error message.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub message: ::std::option::Option<String>,
    ///Parameter related to the error, if any.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub param: ::std::option::Option<String>,
    ///The type of error.
    #[serde(rename = "type", default, skip_serializing_if = "Option::is_none")]
    pub type_value: ::std::option::Option<String>,
}
///Returned when an input audio transcription segment is identified for an item.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct RealtimeBetaServerEventConversationItemInputAudioTranscriptionSegment {
    ///The index of the input audio content part within the item.
    pub content_index: i32,
    ///End time of the segment in seconds.
    pub end: f64,
    ///The unique ID of the server event.
    pub event_id: String,
    ///The segment identifier.
    pub id: String,
    ///The ID of the item containing the input audio content.
    pub item_id: String,
    ///The detected speaker label for this segment.
    pub speaker: String,
    ///Start time of the segment in seconds.
    pub start: f64,
    ///The text for this segment.
    pub text: String,
    ///The event type, must be `conversation.item.input_audio_transcription.segment`.
    #[serde(rename = "type")]
    pub type_value: String,
}
///Returned when a conversation item is retrieved with `conversation.item.retrieve`.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct RealtimeBetaServerEventConversationItemRetrieved {
    ///The unique ID of the server event.
    pub event_id: String,
    pub item: RealtimeConversationItem,
    ///The event type, must be `conversation.item.retrieved`.
    #[serde(rename = "type")]
    pub type_value: String,
}
///Returned when an earlier assistant audio message item is truncated by the client with a `conversation.item.truncate` event. This event is used to synchronize the server's understanding of the audio with the client's playback. This action will truncate the audio and remove the server-side text transcript to ensure there is no text in the context that hasn't been heard by the user.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct RealtimeBetaServerEventConversationItemTruncated {
    ///The duration up to which the audio was truncated, in milliseconds.
    pub audio_end_ms: i32,
    ///The index of the content part that was truncated.
    pub content_index: i32,
    ///The unique ID of the server event.
    pub event_id: String,
    ///The ID of the assistant message item that was truncated.
    pub item_id: String,
    ///The event type, must be `conversation.item.truncated`.
    #[serde(rename = "type")]
    pub type_value: String,
}
///Returned when an error occurs, which could be a client problem or a server problem. Most errors are recoverable and the session will stay open, we recommend to implementors to monitor and log error messages by default.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct RealtimeBetaServerEventError {
    ///Details of the error.
    pub error: RealtimeBetaServerEventErrorError,
    ///The unique ID of the server event.
    pub event_id: String,
    ///The event type, must be `error`.
    #[serde(rename = "type")]
    pub type_value: String,
}
///Details of the error.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct RealtimeBetaServerEventErrorError {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub code: ::std::option::Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub event_id: ::std::option::Option<String>,
    ///A human-readable error message.
    pub message: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub param: ::std::option::Option<String>,
    ///The type of error (e.g., "invalid_request_error", "server_error").
    #[serde(rename = "type")]
    pub type_value: String,
}
///Returned when the input audio buffer is cleared by the client with a `input_audio_buffer.clear` event.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct RealtimeBetaServerEventInputAudioBufferCleared {
    ///The unique ID of the server event.
    pub event_id: String,
    ///The event type, must be `input_audio_buffer.cleared`.
    #[serde(rename = "type")]
    pub type_value: String,
}
///Returned when an input audio buffer is committed, either by the client or automatically in server VAD mode. The `item_id` property is the ID of the user message item that will be created, thus a `conversation.item.created` event will also be sent to the client.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct RealtimeBetaServerEventInputAudioBufferCommitted {
    ///The unique ID of the server event.
    pub event_id: String,
    ///The ID of the user message item that will be created.
    pub item_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub previous_item_id: ::std::option::Option<String>,
    ///The event type, must be `input_audio_buffer.committed`.
    #[serde(rename = "type")]
    pub type_value: String,
}
///Sent by the server when in `server_vad` mode to indicate that speech has been detected in the audio buffer. This can happen any time audio is added to the buffer (unless speech is already detected). The client may want to use this event to interrupt audio playback or provide visual feedback to the user. The client should expect to receive a `input_audio_buffer.speech_stopped` event when speech stops. The `item_id` property is the ID of the user message item that will be created when speech stops and will also be included in the `input_audio_buffer.speech_stopped` event (unless the client manually commits the audio buffer during VAD activation).
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct RealtimeBetaServerEventInputAudioBufferSpeechStarted {
    ///Milliseconds from the start of all audio written to the buffer during the session when speech was first detected. This will correspond to the beginning of audio sent to the model, and thus includes the `prefix_padding_ms` configured in the Session.
    pub audio_start_ms: i32,
    ///The unique ID of the server event.
    pub event_id: String,
    ///The ID of the user message item that will be created when speech stops.
    pub item_id: String,
    ///The event type, must be `input_audio_buffer.speech_started`.
    #[serde(rename = "type")]
    pub type_value: String,
}
///Returned in `server_vad` mode when the server detects the end of speech in the audio buffer. The server will also send an `conversation.item.created` event with the user message item that is created from the audio buffer.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct RealtimeBetaServerEventInputAudioBufferSpeechStopped {
    ///Milliseconds since the session started when speech stopped. This will correspond to the end of audio sent to the model, and thus includes the `min_silence_duration_ms` configured in the Session.
    pub audio_end_ms: i32,
    ///The unique ID of the server event.
    pub event_id: String,
    ///The ID of the user message item that will be created.
    pub item_id: String,
    ///The event type, must be `input_audio_buffer.speech_stopped`.
    #[serde(rename = "type")]
    pub type_value: String,
}
///Returned when listing MCP tools has completed for an item.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct RealtimeBetaServerEventMcpListToolsCompleted {
    ///The unique ID of the server event.
    pub event_id: String,
    ///The ID of the MCP list tools item.
    pub item_id: String,
    ///The event type, must be `mcp_list_tools.completed`.
    #[serde(rename = "type")]
    pub type_value: String,
}
///Returned when listing MCP tools has failed for an item.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct RealtimeBetaServerEventMcpListToolsFailed {
    ///The unique ID of the server event.
    pub event_id: String,
    ///The ID of the MCP list tools item.
    pub item_id: String,
    ///The event type, must be `mcp_list_tools.failed`.
    #[serde(rename = "type")]
    pub type_value: String,
}
///Returned when listing MCP tools is in progress for an item.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct RealtimeBetaServerEventMcpListToolsInProgress {
    ///The unique ID of the server event.
    pub event_id: String,
    ///The ID of the MCP list tools item.
    pub item_id: String,
    ///The event type, must be `mcp_list_tools.in_progress`.
    #[serde(rename = "type")]
    pub type_value: String,
}
///Emitted at the beginning of a Response to indicate the updated rate limits. When a Response is created some tokens will be "reserved" for the output tokens, the rate limits shown here reflect that reservation, which is then adjusted accordingly once the Response is completed.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct RealtimeBetaServerEventRateLimitsUpdated {
    ///The unique ID of the server event.
    pub event_id: String,
    ///List of rate limit information.
    pub rate_limits: Vec<RealtimeBetaServerEventRateLimitsUpdatedRateLimit>,
    ///The event type, must be `rate_limits.updated`.
    #[serde(rename = "type")]
    pub type_value: String,
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct RealtimeBetaServerEventRateLimitsUpdatedRateLimit {
    ///The maximum allowed value for the rate limit.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub limit: ::std::option::Option<i32>,
    ///The name of the rate limit (`requests`, `tokens`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: ::std::option::Option<String>,
    ///The remaining value before the limit is reached.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub remaining: ::std::option::Option<i32>,
    ///Seconds until the rate limit resets.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reset_seconds: ::std::option::Option<f64>,
}
///Returned when the model-generated audio is updated.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct RealtimeBetaServerEventResponseAudioDelta {
    ///The index of the content part in the item's content array.
    pub content_index: i32,
    ///Base64-encoded audio data delta.
    pub delta: String,
    ///The unique ID of the server event.
    pub event_id: String,
    ///The ID of the item.
    pub item_id: String,
    ///The index of the output item in the response.
    pub output_index: i32,
    ///The ID of the response.
    pub response_id: String,
    ///The event type, must be `response.output_audio.delta`.
    #[serde(rename = "type")]
    pub type_value: String,
}
///Returned when the model-generated audio is done. Also emitted when a Response is interrupted, incomplete, or cancelled.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct RealtimeBetaServerEventResponseAudioDone {
    ///The index of the content part in the item's content array.
    pub content_index: i32,
    ///The unique ID of the server event.
    pub event_id: String,
    ///The ID of the item.
    pub item_id: String,
    ///The index of the output item in the response.
    pub output_index: i32,
    ///The ID of the response.
    pub response_id: String,
    ///The event type, must be `response.output_audio.done`.
    #[serde(rename = "type")]
    pub type_value: String,
}
///Returned when the model-generated transcription of audio output is updated.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct RealtimeBetaServerEventResponseAudioTranscriptDelta {
    ///The index of the content part in the item's content array.
    pub content_index: i32,
    ///The transcript delta.
    pub delta: String,
    ///The unique ID of the server event.
    pub event_id: String,
    ///The ID of the item.
    pub item_id: String,
    ///The index of the output item in the response.
    pub output_index: i32,
    ///The ID of the response.
    pub response_id: String,
    ///The event type, must be `response.output_audio_transcript.delta`.
    #[serde(rename = "type")]
    pub type_value: String,
}
///Returned when the model-generated transcription of audio output is done streaming. Also emitted when a Response is interrupted, incomplete, or cancelled.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct RealtimeBetaServerEventResponseAudioTranscriptDone {
    ///The index of the content part in the item's content array.
    pub content_index: i32,
    ///The unique ID of the server event.
    pub event_id: String,
    ///The ID of the item.
    pub item_id: String,
    ///The index of the output item in the response.
    pub output_index: i32,
    ///The ID of the response.
    pub response_id: String,
    ///The final transcript of the audio.
    pub transcript: String,
    ///The event type, must be `response.output_audio_transcript.done`.
    #[serde(rename = "type")]
    pub type_value: String,
}
///Returned when a new content part is added to an assistant message item during response generation.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct RealtimeBetaServerEventResponseContentPartAdded {
    ///The index of the content part in the item's content array.
    pub content_index: i32,
    ///The unique ID of the server event.
    pub event_id: String,
    ///The ID of the item to which the content part was added.
    pub item_id: String,
    ///The index of the output item in the response.
    pub output_index: i32,
    ///The content part that was added.
    pub part: RealtimeBetaServerEventResponseContentPartAddedPart,
    ///The ID of the response.
    pub response_id: String,
    ///The event type, must be `response.content_part.added`.
    #[serde(rename = "type")]
    pub type_value: String,
}
///The content part that was added.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct RealtimeBetaServerEventResponseContentPartAddedPart {
    ///Base64-encoded audio data (if type is "audio").
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub audio: ::std::option::Option<String>,
    ///The text content (if type is "text").
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub text: ::std::option::Option<String>,
    ///The transcript of the audio (if type is "audio").
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub transcript: ::std::option::Option<String>,
    ///The content type ("text", "audio").
    #[serde(rename = "type", default, skip_serializing_if = "Option::is_none")]
    pub type_value: ::std::option::Option<String>,
}
///Returned when a content part is done streaming in an assistant message item. Also emitted when a Response is interrupted, incomplete, or cancelled.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct RealtimeBetaServerEventResponseContentPartDone {
    ///The index of the content part in the item's content array.
    pub content_index: i32,
    ///The unique ID of the server event.
    pub event_id: String,
    ///The ID of the item.
    pub item_id: String,
    ///The index of the output item in the response.
    pub output_index: i32,
    ///The content part that is done.
    pub part: RealtimeBetaServerEventResponseContentPartDonePart,
    ///The ID of the response.
    pub response_id: String,
    ///The event type, must be `response.content_part.done`.
    #[serde(rename = "type")]
    pub type_value: String,
}
///The content part that is done.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct RealtimeBetaServerEventResponseContentPartDonePart {
    ///Base64-encoded audio data (if type is "audio").
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub audio: ::std::option::Option<String>,
    ///The text content (if type is "text").
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub text: ::std::option::Option<String>,
    ///The transcript of the audio (if type is "audio").
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub transcript: ::std::option::Option<String>,
    ///The content type ("text", "audio").
    #[serde(rename = "type", default, skip_serializing_if = "Option::is_none")]
    pub type_value: ::std::option::Option<String>,
}
///Returned when a new Response is created. The first event of response creation, where the response is in an initial state of `in_progress`.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct RealtimeBetaServerEventResponseCreated {
    ///The unique ID of the server event.
    pub event_id: String,
    pub response: RealtimeBetaResponse,
    ///The event type, must be `response.created`.
    #[serde(rename = "type")]
    pub type_value: String,
}
///Returned when a Response is done streaming. Always emitted, no matter the final state. The Response object included in the `response.done` event will include all output Items in the Response but will omit the raw audio data.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct RealtimeBetaServerEventResponseDone {
    ///The unique ID of the server event.
    pub event_id: String,
    pub response: RealtimeBetaResponse,
    ///The event type, must be `response.done`.
    #[serde(rename = "type")]
    pub type_value: String,
}
///Returned when the model-generated function call arguments are updated.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct RealtimeBetaServerEventResponseFunctionCallArgumentsDelta {
    ///The ID of the function call.
    pub call_id: String,
    ///The arguments delta as a JSON string.
    pub delta: String,
    ///The unique ID of the server event.
    pub event_id: String,
    ///The ID of the function call item.
    pub item_id: String,
    ///The index of the output item in the response.
    pub output_index: i32,
    ///The ID of the response.
    pub response_id: String,
    ///The event type, must be `response.function_call_arguments.delta`.
    #[serde(rename = "type")]
    pub type_value: String,
}
///Returned when the model-generated function call arguments are done streaming. Also emitted when a Response is interrupted, incomplete, or cancelled.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct RealtimeBetaServerEventResponseFunctionCallArgumentsDone {
    ///The final arguments as a JSON string.
    pub arguments: String,
    ///The ID of the function call.
    pub call_id: String,
    ///The unique ID of the server event.
    pub event_id: String,
    ///The ID of the function call item.
    pub item_id: String,
    ///The name of the function that was called.
    pub name: String,
    ///The index of the output item in the response.
    pub output_index: i32,
    ///The ID of the response.
    pub response_id: String,
    ///The event type, must be `response.function_call_arguments.done`.
    #[serde(rename = "type")]
    pub type_value: String,
}
///Returned when MCP tool call arguments are updated during response generation.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct RealtimeBetaServerEventResponseMcpCallArgumentsDelta {
    ///The JSON-encoded arguments delta.
    pub delta: String,
    ///The unique ID of the server event.
    pub event_id: String,
    ///The ID of the MCP tool call item.
    pub item_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub obfuscation: ::std::option::Option<String>,
    ///The index of the output item in the response.
    pub output_index: i32,
    ///The ID of the response.
    pub response_id: String,
    ///The event type, must be `response.mcp_call_arguments.delta`.
    #[serde(rename = "type")]
    pub type_value: String,
}
///Returned when MCP tool call arguments are finalized during response generation.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct RealtimeBetaServerEventResponseMcpCallArgumentsDone {
    ///The final JSON-encoded arguments string.
    pub arguments: String,
    ///The unique ID of the server event.
    pub event_id: String,
    ///The ID of the MCP tool call item.
    pub item_id: String,
    ///The index of the output item in the response.
    pub output_index: i32,
    ///The ID of the response.
    pub response_id: String,
    ///The event type, must be `response.mcp_call_arguments.done`.
    #[serde(rename = "type")]
    pub type_value: String,
}
///Returned when an MCP tool call has completed successfully.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct RealtimeBetaServerEventResponseMcpCallCompleted {
    ///The unique ID of the server event.
    pub event_id: String,
    ///The ID of the MCP tool call item.
    pub item_id: String,
    ///The index of the output item in the response.
    pub output_index: i32,
    ///The event type, must be `response.mcp_call.completed`.
    #[serde(rename = "type")]
    pub type_value: String,
}
///Returned when an MCP tool call has failed.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct RealtimeBetaServerEventResponseMcpCallFailed {
    ///The unique ID of the server event.
    pub event_id: String,
    ///The ID of the MCP tool call item.
    pub item_id: String,
    ///The index of the output item in the response.
    pub output_index: i32,
    ///The event type, must be `response.mcp_call.failed`.
    #[serde(rename = "type")]
    pub type_value: String,
}
///Returned when an MCP tool call has started and is in progress.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct RealtimeBetaServerEventResponseMcpCallInProgress {
    ///The unique ID of the server event.
    pub event_id: String,
    ///The ID of the MCP tool call item.
    pub item_id: String,
    ///The index of the output item in the response.
    pub output_index: i32,
    ///The event type, must be `response.mcp_call.in_progress`.
    #[serde(rename = "type")]
    pub type_value: String,
}
///Returned when a new Item is created during Response generation.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct RealtimeBetaServerEventResponseOutputItemAdded {
    ///The unique ID of the server event.
    pub event_id: String,
    pub item: RealtimeConversationItem,
    ///The index of the output item in the Response.
    pub output_index: i32,
    ///The ID of the Response to which the item belongs.
    pub response_id: String,
    ///The event type, must be `response.output_item.added`.
    #[serde(rename = "type")]
    pub type_value: String,
}
///Returned when an Item is done streaming. Also emitted when a Response is interrupted, incomplete, or cancelled.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct RealtimeBetaServerEventResponseOutputItemDone {
    ///The unique ID of the server event.
    pub event_id: String,
    pub item: RealtimeConversationItem,
    ///The index of the output item in the Response.
    pub output_index: i32,
    ///The ID of the Response to which the item belongs.
    pub response_id: String,
    ///The event type, must be `response.output_item.done`.
    #[serde(rename = "type")]
    pub type_value: String,
}
///Returned when the text value of an "output_text" content part is updated.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct RealtimeBetaServerEventResponseTextDelta {
    ///The index of the content part in the item's content array.
    pub content_index: i32,
    ///The text delta.
    pub delta: String,
    ///The unique ID of the server event.
    pub event_id: String,
    ///The ID of the item.
    pub item_id: String,
    ///The index of the output item in the response.
    pub output_index: i32,
    ///The ID of the response.
    pub response_id: String,
    ///The event type, must be `response.output_text.delta`.
    #[serde(rename = "type")]
    pub type_value: String,
}
///Returned when the text value of an "output_text" content part is done streaming. Also emitted when a Response is interrupted, incomplete, or cancelled.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct RealtimeBetaServerEventResponseTextDone {
    ///The index of the content part in the item's content array.
    pub content_index: i32,
    ///The unique ID of the server event.
    pub event_id: String,
    ///The ID of the item.
    pub item_id: String,
    ///The index of the output item in the response.
    pub output_index: i32,
    ///The ID of the response.
    pub response_id: String,
    ///The final text content.
    pub text: String,
    ///The event type, must be `response.output_text.done`.
    #[serde(rename = "type")]
    pub type_value: String,
}
///Returned when a Session is created. Emitted automatically when a new connection is established as the first server event. This event will contain the default Session configuration.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct RealtimeBetaServerEventSessionCreated {
    ///The unique ID of the server event.
    pub event_id: String,
    pub session: RealtimeSession,
    ///The event type, must be `session.created`.
    #[serde(rename = "type")]
    pub type_value: String,
}
///Returned when a session is updated with a `session.update` event, unless there is an error.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct RealtimeBetaServerEventSessionUpdated {
    ///The unique ID of the server event.
    pub event_id: String,
    pub session: RealtimeSession,
    ///The event type, must be `session.updated`.
    #[serde(rename = "type")]
    pub type_value: String,
}
///Returned when a transcription session is created.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct RealtimeBetaServerEventTranscriptionSessionCreated {
    ///The unique ID of the server event.
    pub event_id: String,
    pub session: RealtimeTranscriptionSessionCreateResponse,
    ///The event type, must be `transcription_session.created`.
    #[serde(rename = "type")]
    pub type_value: String,
}
///Returned when a transcription session is updated with a `transcription_session.update` event, unless there is an error.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct RealtimeBetaServerEventTranscriptionSessionUpdated {
    ///The unique ID of the server event.
    pub event_id: String,
    pub session: RealtimeTranscriptionSessionCreateResponse,
    ///The event type, must be `transcription_session.updated`.
    #[serde(rename = "type")]
    pub type_value: String,
}
///Parameters required to initiate a realtime call and receive the SDP answer needed to complete a WebRTC peer connection. Provide an SDP offer generated by your client and optionally configure the session that will answer the call.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct RealtimeCallCreateRequest {
    ///WebRTC Session Description Protocol (SDP) offer generated by the caller.
    pub sdp: String,
    ///Optional session configuration to apply before the realtime session is created. Use the same parameters you would send in a [`create client secret`](/docs/api-reference/realtime-sessions/create-realtime-client-secret) request.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub session: ::std::option::Option<RealtimeCallCreateRequestSession>,
}
///Optional session configuration to apply before the realtime session is created. Use the same parameters you would send in a [`create client secret`](/docs/api-reference/realtime-sessions/create-realtime-client-secret) request.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct RealtimeCallCreateRequestSession {
    ///Configuration for input and output audio.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub audio: ::std::option::Option<RealtimeCallCreateRequestSessionAudio>,
    ///Additional fields to include in server outputs. `item.input_audio_transcription.logprobs`: Include logprobs for input audio transcription.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub include: ::std::option::Option<Vec<String>>,
    ///The default system instructions (i.e. system message) prepended to model calls. This field allows the client to guide the model on desired responses. The model can be instructed on response content and format, (e.g. "be extremely succinct", "act friendly", "here are examples of good responses") and on audio behavior (e.g. "talk quickly", "inject emotion into your voice", "laugh frequently"). The instructions are not guaranteed to be followed by the model, but they provide guidance to the model on the desired behavior. Note that the server sets default instructions which will be used if this field is not set and are visible in the `session.created` event at the start of the session.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub instructions: ::std::option::Option<String>,
    ///Maximum number of output tokens for a single assistant response, inclusive of tool calls. Provide an integer between 1 and 4096 to limit output tokens, or `inf` for the maximum available tokens for a given model. Defaults to `inf`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_output_tokens: ::std::option::Option<
        RealtimeCallCreateRequestSessionMaxOutputTokens,
    >,
    ///The Realtime model used for this session.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model: ::std::option::Option<String>,
    ///The set of modalities the model can respond with. It defaults to `["audio"]`, indicating that the model will respond with audio plus a transcript. `["text"]` can be used to make the model respond with text only. It is not possible to request both `text` and `audio` at the same time.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub output_modalities: ::std::option::Option<Vec<String>>,
    ///Whether the model may call multiple tools in parallel. Only supported by reasoning Realtime models such as `gpt-realtime-2`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parallel_tool_calls: ::std::option::Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub prompt: ::std::option::Option<Prompt>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reasoning: ::std::option::Option<RealtimeReasoning>,
    ///How the model chooses tools. Provide one of the string modes or force a specific function/MCP tool.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tool_choice: ::std::option::Option<RealtimeCallCreateRequestSessionToolChoice>,
    ///Tools available to the model.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tools: ::std::option::Option<Vec<RealtimeCallCreateRequestSessionTool>>,
    ///Realtime API can write session traces to the [Traces Dashboard](https://platform.openai.com/logs?api=traces). Set to null to disable tracing. Once tracing is enabled for a session, the configuration cannot be modified. `auto` will create a trace for the session with default values for the workflow name, group id, and metadata.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tracing: ::std::option::Option<RealtimeCallCreateRequestSessionTracing2>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub truncation: ::std::option::Option<RealtimeTruncation>,
    ///The type of session to create. Always `realtime` for the Realtime API.
    #[serde(rename = "type")]
    pub type_value: String,
}
///Configuration for input and output audio.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct RealtimeCallCreateRequestSessionAudio {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub input: ::std::option::Option<RealtimeCallCreateRequestSessionAudioInput>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub output: ::std::option::Option<RealtimeCallCreateRequestSessionAudioOutput>,
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct RealtimeCallCreateRequestSessionAudioInput {
    ///The format of the input audio.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub format: ::std::option::Option<RealtimeAudioFormats>,
    ///Configuration for input audio noise reduction. This can be set to `null` to turn off. Noise reduction filters audio added to the input audio buffer before it is sent to VAD and the model. Filtering the audio can improve VAD and turn detection accuracy (reducing false positives) and model performance by improving perception of the input audio.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub noise_reduction: ::std::option::Option<
        RealtimeCallCreateRequestSessionAudioInputNoiseReduction,
    >,
    ///Configuration for input audio transcription, defaults to off and can be set to `null` to turn off once on. Input audio transcription is not native to the model, since the model consumes audio directly. Transcription runs asynchronously through [the /audio/transcriptions endpoint](/docs/api-reference/audio/createTranscription) and should be treated as guidance of input audio content rather than precisely what the model heard. The client can optionally set the language and prompt for transcription, these offer additional guidance to the transcription service.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub transcription: ::std::option::Option<AudioTranscription>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub turn_detection: ::std::option::Option<RealtimeTurnDetection>,
}
///Configuration for input audio noise reduction. This can be set to `null` to turn off. Noise reduction filters audio added to the input audio buffer before it is sent to VAD and the model. Filtering the audio can improve VAD and turn detection accuracy (reducing false positives) and model performance by improving perception of the input audio.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct RealtimeCallCreateRequestSessionAudioInputNoiseReduction {
    #[serde(rename = "type", default, skip_serializing_if = "Option::is_none")]
    pub type_value: ::std::option::Option<NoiseReductionType>,
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct RealtimeCallCreateRequestSessionAudioOutput {
    ///The format of the output audio.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub format: ::std::option::Option<RealtimeAudioFormats>,
    ///The speed of the model's spoken response as a multiple of the original speed. 1.0 is the default speed. 0.25 is the minimum speed. 1.5 is the maximum speed. This value can only be changed in between model turns, not while a response is in progress. This parameter is a post-processing adjustment to the audio after it is generated, it's also possible to prompt the model to speak faster or slower.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub speed: ::std::option::Option<f64>,
    ///The voice the model uses to respond. Supported built-in voices are `alloy`, `ash`, `ballad`, `coral`, `echo`, `sage`, `shimmer`, `verse`, `marin`, and `cedar`. You may also provide a custom voice object with an `id`, for example `{ "id": "voice_1234" }`. Voice cannot be changed during the session once the model has responded with audio at least once. We recommend `marin` and `cedar` for best quality.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub voice: ::std::option::Option<VoiceIdsOrCustomVoice>,
}
///Maximum number of output tokens for a single assistant response, inclusive of tool calls. Provide an integer between 1 and 4096 to limit output tokens, or `inf` for the maximum available tokens for a given model. Defaults to `inf`.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
#[serde(untagged)]
pub enum RealtimeCallCreateRequestSessionMaxOutputTokens {
    Integer(i32),
    Inf(String),
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
#[serde(untagged)]
pub enum RealtimeCallCreateRequestSessionTool {
    RealtimeFunctionTool(RealtimeFunctionTool),
    McpTool(McpTool),
}
///How the model chooses tools. Provide one of the string modes or force a specific function/MCP tool.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
#[serde(untagged)]
pub enum RealtimeCallCreateRequestSessionToolChoice {
    ToolChoiceOptions(ToolChoiceOptions),
    ToolChoiceFunction(ToolChoiceFunction),
    ToolChoiceMcp(ToolChoiceMcp),
}
///Granular configuration for tracing.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct RealtimeCallCreateRequestSessionTracing {
    ///The group id to attach to this trace to enable filtering and grouping in the Traces Dashboard.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub group_id: ::std::option::Option<String>,
    ///The arbitrary metadata to attach to this trace to enable filtering in the Traces Dashboard.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub metadata: ::std::option::Option<OpenAiJsonValue>,
    ///The name of the workflow to attach to this trace. This is used to name the trace in the Traces Dashboard.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub workflow_name: ::std::option::Option<String>,
}
///Realtime API can write session traces to the [Traces Dashboard](https://platform.openai.com/logs?api=traces). Set to null to disable tracing. Once tracing is enabled for a session, the configuration cannot be modified. `auto` will create a trace for the session with default values for the workflow name, group id, and metadata.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
#[serde(untagged)]
pub enum RealtimeCallCreateRequestSessionTracing2 {
    Auto(String),
    TracingConfiguration(RealtimeCallCreateRequestSessionTracing2TracingConfiguration),
}
///Granular configuration for tracing.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct RealtimeCallCreateRequestSessionTracing2TracingConfiguration {
    ///The group id to attach to this trace to enable filtering and grouping in the Traces Dashboard.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub group_id: ::std::option::Option<String>,
    ///The arbitrary metadata to attach to this trace to enable filtering in the Traces Dashboard.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub metadata: ::std::option::Option<OpenAiJsonValue>,
    ///The name of the workflow to attach to this trace. This is used to name the trace in the Traces Dashboard.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub workflow_name: ::std::option::Option<String>,
}
///Parameters required to transfer a SIP call to a new destination using the Realtime API.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct RealtimeCallReferRequest {
    ///URI that should appear in the SIP Refer-To header. Supports values like `tel:+14155550123` or `sip:agent@example.com`.
    pub target_uri: String,
}
///Parameters used to decline an incoming SIP call handled by the Realtime API.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct RealtimeCallRejectRequest {
    ///SIP response code to send back to the caller. Defaults to `603` (Decline) when omitted.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub status_code: ::std::option::Option<i32>,
}
///A realtime client event.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
#[serde(untagged)]
pub enum RealtimeClientEvent {
    RealtimeClientEventConversationItemCreate(RealtimeClientEventConversationItemCreate),
    RealtimeClientEventConversationItemDelete(RealtimeClientEventConversationItemDelete),
    RealtimeClientEventConversationItemRetrieve(
        RealtimeClientEventConversationItemRetrieve,
    ),
    RealtimeClientEventConversationItemTruncate(
        RealtimeClientEventConversationItemTruncate,
    ),
    RealtimeClientEventInputAudioBufferAppend(RealtimeClientEventInputAudioBufferAppend),
    RealtimeClientEventInputAudioBufferClear(RealtimeClientEventInputAudioBufferClear),
    RealtimeClientEventOutputAudioBufferClear(RealtimeClientEventOutputAudioBufferClear),
    RealtimeClientEventInputAudioBufferCommit(RealtimeClientEventInputAudioBufferCommit),
    RealtimeClientEventResponseCancel(RealtimeClientEventResponseCancel),
    RealtimeClientEventResponseCreate(RealtimeClientEventResponseCreate),
    RealtimeClientEventSessionUpdate(RealtimeClientEventSessionUpdate),
}
///Add a new Item to the Conversation's context, including messages, function calls, and function call responses. This event can be used both to populate a "history" of the conversation and to add new items mid-stream, but has the current limitation that it cannot populate assistant audio messages. If successful, the server will respond with a `conversation.item.created` event, otherwise an `error` event will be sent.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct RealtimeClientEventConversationItemCreate {
    ///Optional client-generated ID used to identify this event.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub event_id: ::std::option::Option<String>,
    pub item: RealtimeConversationItem,
    ///The ID of the preceding item after which the new item will be inserted. If not set, the new item will be appended to the end of the conversation. If set to `root`, the new item will be added to the beginning of the conversation. If set to an existing ID, it allows an item to be inserted mid-conversation. If the ID cannot be found, an error will be returned and the item will not be added.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub previous_item_id: ::std::option::Option<String>,
    ///The event type, must be `conversation.item.create`.
    #[serde(rename = "type")]
    pub type_value: String,
}
///Send this event when you want to remove any item from the conversation history. The server will respond with a `conversation.item.deleted` event, unless the item does not exist in the conversation history, in which case the server will respond with an error.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct RealtimeClientEventConversationItemDelete {
    ///Optional client-generated ID used to identify this event.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub event_id: ::std::option::Option<String>,
    ///The ID of the item to delete.
    pub item_id: String,
    ///The event type, must be `conversation.item.delete`.
    #[serde(rename = "type")]
    pub type_value: String,
}
///Send this event when you want to retrieve the server's representation of a specific item in the conversation history. This is useful, for example, to inspect user audio after noise cancellation and VAD. The server will respond with a `conversation.item.retrieved` event, unless the item does not exist in the conversation history, in which case the server will respond with an error.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct RealtimeClientEventConversationItemRetrieve {
    ///Optional client-generated ID used to identify this event.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub event_id: ::std::option::Option<String>,
    ///The ID of the item to retrieve.
    pub item_id: String,
    ///The event type, must be `conversation.item.retrieve`.
    #[serde(rename = "type")]
    pub type_value: String,
}
///Send this event to truncate a previous assistant message’s audio. The server will produce audio faster than realtime, so this event is useful when the user interrupts to truncate audio that has already been sent to the client but not yet played. This will synchronize the server's understanding of the audio with the client's playback. Truncating audio will delete the server-side text transcript to ensure there is not text in the context that hasn't been heard by the user. If successful, the server will respond with a `conversation.item.truncated` event.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct RealtimeClientEventConversationItemTruncate {
    ///Inclusive duration up to which audio is truncated, in milliseconds. If the audio_end_ms is greater than the actual audio duration, the server will respond with an error.
    pub audio_end_ms: i32,
    ///The index of the content part to truncate. Set this to `0`.
    pub content_index: i32,
    ///Optional client-generated ID used to identify this event.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub event_id: ::std::option::Option<String>,
    ///The ID of the assistant message item to truncate. Only assistant message items can be truncated.
    pub item_id: String,
    ///The event type, must be `conversation.item.truncate`.
    #[serde(rename = "type")]
    pub type_value: String,
}
///Send this event to append audio bytes to the input audio buffer. The audio buffer is temporary storage you can write to and later commit. A "commit" will create a new user message item in the conversation history from the buffer content and clear the buffer. Input audio transcription (if enabled) will be generated when the buffer is committed. If VAD is enabled the audio buffer is used to detect speech and the server will decide when to commit. When Server VAD is disabled, you must commit the audio buffer manually. Input audio noise reduction operates on writes to the audio buffer. The client may choose how much audio to place in each event up to a maximum of 15 MiB, for example streaming smaller chunks from the client may allow the VAD to be more responsive. Unlike most other client events, the server will not send a confirmation response to this event.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct RealtimeClientEventInputAudioBufferAppend {
    ///Base64-encoded audio bytes. This must be in the format specified by the `input_audio_format` field in the session configuration.
    pub audio: String,
    ///Optional client-generated ID used to identify this event.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub event_id: ::std::option::Option<String>,
    ///The event type, must be `input_audio_buffer.append`.
    #[serde(rename = "type")]
    pub type_value: String,
}
///Send this event to clear the audio bytes in the buffer. The server will respond with an `input_audio_buffer.cleared` event.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct RealtimeClientEventInputAudioBufferClear {
    ///Optional client-generated ID used to identify this event.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub event_id: ::std::option::Option<String>,
    ///The event type, must be `input_audio_buffer.clear`.
    #[serde(rename = "type")]
    pub type_value: String,
}
///Send this event to commit the user input audio buffer, which will create a new user message item in the conversation. This event will produce an error if the input audio buffer is empty. When in Server VAD mode, the client does not need to send this event, the server will commit the audio buffer automatically. Committing the input audio buffer will trigger input audio transcription (if enabled in session configuration), but it will not create a response from the model. The server will respond with an `input_audio_buffer.committed` event.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct RealtimeClientEventInputAudioBufferCommit {
    ///Optional client-generated ID used to identify this event.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub event_id: ::std::option::Option<String>,
    ///The event type, must be `input_audio_buffer.commit`.
    #[serde(rename = "type")]
    pub type_value: String,
}
///**WebRTC/SIP Only:** Emit to cut off the current audio response. This will trigger the server to stop generating audio and emit a `output_audio_buffer.cleared` event. This event should be preceded by a `response.cancel` client event to stop the generation of the current response. [Learn more](/docs/guides/realtime-conversations#client-and-server-events-for-audio-in-webrtc).
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct RealtimeClientEventOutputAudioBufferClear {
    ///The unique ID of the client event used for error handling.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub event_id: ::std::option::Option<String>,
    ///The event type, must be `output_audio_buffer.clear`.
    #[serde(rename = "type")]
    pub type_value: String,
}
///Send this event to cancel an in-progress response. The server will respond with a `response.done` event with a status of `response.status=cancelled`. If there is no response to cancel, the server will respond with an error. It's safe to call `response.cancel` even if no response is in progress, an error will be returned the session will remain unaffected.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct RealtimeClientEventResponseCancel {
    ///Optional client-generated ID used to identify this event.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub event_id: ::std::option::Option<String>,
    ///A specific response ID to cancel - if not provided, will cancel an in-progress response in the default conversation.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub response_id: ::std::option::Option<String>,
    ///The event type, must be `response.cancel`.
    #[serde(rename = "type")]
    pub type_value: String,
}
///This event instructs the server to create a Response, which means triggering model inference. When in Server VAD mode, the server will create Responses automatically. A Response will include at least one Item, and may have two, in which case the second will be a function call. These Items will be appended to the conversation history by default. The server will respond with a `response.created` event, events for Items and content created, and finally a `response.done` event to indicate the Response is complete. The `response.create` event includes inference configuration like `instructions` and `tools`. If these are set, they will override the Session's configuration for this Response only. Responses can be created out-of-band of the default Conversation, meaning that they can have arbitrary input, and it's possible to disable writing the output to the Conversation. Only one Response can write to the default Conversation at a time, but otherwise multiple Responses can be created in parallel. The `metadata` field is a good way to disambiguate multiple simultaneous Responses. Clients can set `conversation` to `none` to create a Response that does not write to the default Conversation. Arbitrary input can be provided with the `input` field, which is an array accepting raw Items and references to existing Items.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct RealtimeClientEventResponseCreate {
    ///Optional client-generated ID used to identify this event.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub event_id: ::std::option::Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub response: ::std::option::Option<RealtimeResponseCreateParams>,
    ///The event type, must be `response.create`.
    #[serde(rename = "type")]
    pub type_value: String,
}
///Send this event to update the session’s configuration. The client may send this event at any time to update any field except for `voice` and `model`. `voice` can be updated only if there have been no other audio outputs yet. When the server receives a `session.update`, it will respond with a `session.updated` event showing the full, effective configuration. Only the fields that are present in the `session.update` are updated. To clear a field like `instructions`, pass an empty string. To clear a field like `tools`, pass an empty array. To clear a field like `turn_detection`, pass `null`.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct RealtimeClientEventSessionUpdate {
    ///Optional client-generated ID used to identify this event. This is an arbitrary string that a client may assign. It will be passed back if there is an error with the event, but the corresponding `session.updated` event will not include it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub event_id: ::std::option::Option<String>,
    ///Update the Realtime session. Choose either a realtime session or a transcription session.
    pub session: RealtimeClientEventSessionUpdateSession,
    ///The event type, must be `session.update`.
    #[serde(rename = "type")]
    pub type_value: String,
}
///Update the Realtime session. Choose either a realtime session or a transcription session.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
#[serde(untagged)]
pub enum RealtimeClientEventSessionUpdateSession {
    RealtimeSessionCreateRequestGa(RealtimeSessionCreateRequestGa),
    RealtimeTranscriptionSessionCreateRequestGa(
        RealtimeTranscriptionSessionCreateRequestGa,
    ),
}
///Send this event to update a transcription session.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct RealtimeClientEventTranscriptionSessionUpdate {
    ///Optional client-generated ID used to identify this event.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub event_id: ::std::option::Option<String>,
    pub session: RealtimeTranscriptionSessionCreateRequest,
    ///The event type, must be `transcription_session.update`.
    #[serde(rename = "type")]
    pub type_value: String,
}
///A single item within a Realtime conversation.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
#[serde(untagged)]
pub enum RealtimeConversationItem {
    RealtimeConversationItemMessageSystem(RealtimeConversationItemMessageSystem),
    RealtimeConversationItemMessageUser(RealtimeConversationItemMessageUser),
    RealtimeConversationItemMessageAssistant(RealtimeConversationItemMessageAssistant),
    RealtimeConversationItemFunctionCall(RealtimeConversationItemFunctionCall),
    RealtimeConversationItemFunctionCallOutput(
        RealtimeConversationItemFunctionCallOutput,
    ),
    RealtimeMcpApprovalResponse(RealtimeMcpApprovalResponse),
    RealtimeMcpListTools(RealtimeMcpListTools),
    RealtimeMcpToolCall(RealtimeMcpToolCall),
    RealtimeMcpApprovalRequest(RealtimeMcpApprovalRequest),
}
///A function call item in a Realtime conversation.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct RealtimeConversationItemFunctionCall {
    ///The arguments of the function call. This is a JSON-encoded string representing the arguments passed to the function, for example `{"arg1": "value1", "arg2": 42}`.
    pub arguments: String,
    ///The ID of the function call.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub call_id: ::std::option::Option<String>,
    ///The unique ID of the item. This may be provided by the client or generated by the server.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: ::std::option::Option<String>,
    ///The name of the function being called.
    pub name: String,
    ///Identifier for the API object being returned - always `realtime.item`. Optional when creating a new item.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub object: ::std::option::Option<String>,
    ///The status of the item. Has no effect on the conversation.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub status: ::std::option::Option<String>,
    ///The type of the item. Always `function_call`.
    #[serde(rename = "type")]
    pub type_value: String,
}
///A function call output item in a Realtime conversation.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct RealtimeConversationItemFunctionCallOutput {
    ///The ID of the function call this output is for.
    pub call_id: String,
    ///The unique ID of the item. This may be provided by the client or generated by the server.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: ::std::option::Option<String>,
    ///Identifier for the API object being returned - always `realtime.item`. Optional when creating a new item.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub object: ::std::option::Option<String>,
    ///The output of the function call, this is free text and can contain any information or simply be empty.
    pub output: String,
    ///The status of the item. Has no effect on the conversation.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub status: ::std::option::Option<String>,
    ///The type of the item. Always `function_call_output`.
    #[serde(rename = "type")]
    pub type_value: String,
}
///An assistant message item in a Realtime conversation.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct RealtimeConversationItemMessageAssistant {
    ///The content of the message.
    pub content: Vec<RealtimeConversationItemMessageAssistantContentItem>,
    ///The unique ID of the item. This may be provided by the client or generated by the server.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: ::std::option::Option<String>,
    ///Identifier for the API object being returned - always `realtime.item`. Optional when creating a new item.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub object: ::std::option::Option<String>,
    ///The role of the message sender. Always `assistant`.
    pub role: String,
    ///The status of the item. Has no effect on the conversation.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub status: ::std::option::Option<String>,
    ///The type of the item. Always `message`.
    #[serde(rename = "type")]
    pub type_value: String,
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct RealtimeConversationItemMessageAssistantContentItem {
    ///Base64-encoded audio bytes, these will be parsed as the format specified in the session output audio type configuration. This defaults to PCM 16-bit 24kHz mono if not specified.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub audio: ::std::option::Option<String>,
    ///The text content.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub text: ::std::option::Option<String>,
    ///The transcript of the audio content, this will always be present if the output type is `audio`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub transcript: ::std::option::Option<String>,
    ///The content type, `output_text` or `output_audio` depending on the session `output_modalities` configuration.
    #[serde(rename = "type", default, skip_serializing_if = "Option::is_none")]
    pub type_value: ::std::option::Option<String>,
}
///A system message in a Realtime conversation can be used to provide additional context or instructions to the model. This is similar but distinct from the instruction prompt provided at the start of a conversation, as system messages can be added at any point in the conversation. For major changes to the conversation's behavior, use instructions, but for smaller updates (e.g. "the user is now asking about a different topic"), use system messages.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct RealtimeConversationItemMessageSystem {
    ///The content of the message.
    pub content: Vec<RealtimeConversationItemMessageSystemContentItem>,
    ///The unique ID of the item. This may be provided by the client or generated by the server.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: ::std::option::Option<String>,
    ///Identifier for the API object being returned - always `realtime.item`. Optional when creating a new item.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub object: ::std::option::Option<String>,
    ///The role of the message sender. Always `system`.
    pub role: String,
    ///The status of the item. Has no effect on the conversation.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub status: ::std::option::Option<String>,
    ///The type of the item. Always `message`.
    #[serde(rename = "type")]
    pub type_value: String,
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct RealtimeConversationItemMessageSystemContentItem {
    ///The text content.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub text: ::std::option::Option<String>,
    ///The content type. Always `input_text` for system messages.
    #[serde(rename = "type", default, skip_serializing_if = "Option::is_none")]
    pub type_value: ::std::option::Option<String>,
}
///A user message item in a Realtime conversation.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct RealtimeConversationItemMessageUser {
    ///The content of the message.
    pub content: Vec<RealtimeConversationItemMessageUserContentItem>,
    ///The unique ID of the item. This may be provided by the client or generated by the server.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: ::std::option::Option<String>,
    ///Identifier for the API object being returned - always `realtime.item`. Optional when creating a new item.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub object: ::std::option::Option<String>,
    ///The role of the message sender. Always `user`.
    pub role: String,
    ///The status of the item. Has no effect on the conversation.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub status: ::std::option::Option<String>,
    ///The type of the item. Always `message`.
    #[serde(rename = "type")]
    pub type_value: String,
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct RealtimeConversationItemMessageUserContentItem {
    ///Base64-encoded audio bytes (for `input_audio`), these will be parsed as the format specified in the session input audio type configuration. This defaults to PCM 16-bit 24kHz mono if not specified.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub audio: ::std::option::Option<String>,
    ///The detail level of the image (for `input_image`). `auto` will default to `high`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub detail: ::std::option::Option<String>,
    ///Base64-encoded image bytes (for `input_image`) as a data URI. For example `data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAA...`. Supported formats are PNG and JPEG.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub image_url: ::std::option::Option<String>,
    ///The text content (for `input_text`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub text: ::std::option::Option<String>,
    ///Transcript of the audio (for `input_audio`). This is not sent to the model, but will be attached to the message item for reference.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub transcript: ::std::option::Option<String>,
    ///The content type (`input_text`, `input_audio`, or `input_image`).
    #[serde(rename = "type", default, skip_serializing_if = "Option::is_none")]
    pub type_value: ::std::option::Option<String>,
}
///The item to add to the conversation.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct RealtimeConversationItemWithReference {
    ///The arguments of the function call (for `function_call` items).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub arguments: ::std::option::Option<String>,
    ///The ID of the function call (for `function_call` and `function_call_output` items). If passed on a `function_call_output` item, the server will check that a `function_call` item with the same ID exists in the conversation history.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub call_id: ::std::option::Option<String>,
    ///The content of the message, applicable for `message` items. - Message items of role `system` support only `input_text` content - Message items of role `user` support `input_text` and `input_audio` content - Message items of role `assistant` support `text` content.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub content: ::std::option::Option<
        Vec<RealtimeConversationItemWithReferenceContentItem>,
    >,
    ///For an item of type (`message` | `function_call` | `function_call_output`) this field allows the client to assign the unique ID of the item. It is not required because the server will generate one if not provided. For an item of type `item_reference`, this field is required and is a reference to any item that has previously existed in the conversation.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: ::std::option::Option<String>,
    ///The name of the function being called (for `function_call` items).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: ::std::option::Option<String>,
    ///Identifier for the API object being returned - always `realtime.item`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub object: ::std::option::Option<String>,
    ///The output of the function call (for `function_call_output` items).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub output: ::std::option::Option<String>,
    ///The role of the message sender (`user`, `assistant`, `system`), only applicable for `message` items.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub role: ::std::option::Option<String>,
    ///The status of the item (`completed`, `incomplete`, `in_progress`). These have no effect on the conversation, but are accepted for consistency with the `conversation.item.created` event.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub status: ::std::option::Option<String>,
    ///The type of the item (`message`, `function_call`, `function_call_output`, `item_reference`).
    #[serde(rename = "type", default, skip_serializing_if = "Option::is_none")]
    pub type_value: ::std::option::Option<String>,
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct RealtimeConversationItemWithReferenceContentItem {
    ///Base64-encoded audio bytes, used for `input_audio` content type.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub audio: ::std::option::Option<String>,
    ///ID of a previous conversation item to reference (for `item_reference` content types in `response.create` events). These can reference both client and server created items.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: ::std::option::Option<String>,
    ///The text content, used for `input_text` and `text` content types.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub text: ::std::option::Option<String>,
    ///The transcript of the audio, used for `input_audio` content type.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub transcript: ::std::option::Option<String>,
    ///The content type (`input_text`, `input_audio`, `item_reference`, `text`).
    #[serde(rename = "type", default, skip_serializing_if = "Option::is_none")]
    pub type_value: ::std::option::Option<String>,
}
///Create a session and client secret for the Realtime API. The request can specify either a realtime or a transcription session configuration. [Learn more about the Realtime API](/docs/guides/realtime).
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct RealtimeCreateClientSecretRequest {
    ///Configuration for the client secret expiration. Expiration refers to the time after which a client secret will no longer be valid for creating sessions. The session itself may continue after that time once started. A secret can be used to create multiple sessions until it expires.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expires_after: ::std::option::Option<
        RealtimeCreateClientSecretRequestExpiresAfter,
    >,
    ///Session configuration to use for the client secret. Choose either a realtime session or a transcription session.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub session: ::std::option::Option<RealtimeCreateClientSecretRequestSession>,
}
///Configuration for the client secret expiration. Expiration refers to the time after which a client secret will no longer be valid for creating sessions. The session itself may continue after that time once started. A secret can be used to create multiple sessions until it expires.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct RealtimeCreateClientSecretRequestExpiresAfter {
    ///The anchor point for the client secret expiration, meaning that `seconds` will be added to the `created_at` time of the client secret to produce an expiration timestamp. Only `created_at` is currently supported.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub anchor: ::std::option::Option<String>,
    ///The number of seconds from the anchor point to the expiration. Select a value between `10` and `7200` (2 hours). This default to 600 seconds (10 minutes) if not specified.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub seconds: ::std::option::Option<i64>,
}
///Session configuration to use for the client secret. Choose either a realtime session or a transcription session.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
#[serde(untagged)]
pub enum RealtimeCreateClientSecretRequestSession {
    RealtimeSessionCreateRequestGa(RealtimeSessionCreateRequestGa),
    RealtimeTranscriptionSessionCreateRequestGa(
        RealtimeTranscriptionSessionCreateRequestGa,
    ),
}
///Response from creating a session and client secret for the Realtime API.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct RealtimeCreateClientSecretResponse {
    ///Expiration timestamp for the client secret, in seconds since epoch.
    pub expires_at: i64,
    ///The session configuration for either a realtime or transcription session.
    pub session: RealtimeCreateClientSecretResponseSession,
    ///The generated client secret value.
    pub value: String,
}
///The session configuration for either a realtime or transcription session.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
#[serde(untagged)]
pub enum RealtimeCreateClientSecretResponseSession {
    RealtimeSessionCreateResponseGa(RealtimeSessionCreateResponseGa),
    RealtimeTranscriptionSessionCreateResponseGa(
        RealtimeTranscriptionSessionCreateResponseGa,
    ),
}
///Function tool
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct RealtimeFunctionTool {
    ///The description of the function, including guidance on when and how to call it, and guidance about what to tell the user when calling (if anything).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: ::std::option::Option<String>,
    ///The name of the function.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: ::std::option::Option<String>,
    ///Parameters of the function in JSON Schema.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parameters: ::std::option::Option<OpenAiJsonValue>,
    ///The type of the tool, i.e. `function`.
    #[serde(rename = "type", default, skip_serializing_if = "Option::is_none")]
    pub type_value: ::std::option::Option<String>,
}
///A Realtime item requesting human approval of a tool invocation.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct RealtimeMcpApprovalRequest {
    ///A JSON string of arguments for the tool.
    pub arguments: String,
    ///The unique ID of the approval request.
    pub id: String,
    ///The name of the tool to run.
    pub name: String,
    ///The label of the MCP server making the request.
    pub server_label: String,
    ///The type of the item. Always `mcp_approval_request`.
    #[serde(rename = "type")]
    pub type_value: String,
}
///A Realtime item responding to an MCP approval request.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct RealtimeMcpApprovalResponse {
    ///The ID of the approval request being answered.
    pub approval_request_id: String,
    ///Whether the request was approved.
    pub approve: bool,
    ///The unique ID of the approval response.
    pub id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reason: ::std::option::Option<String>,
    ///The type of the item. Always `mcp_approval_response`.
    #[serde(rename = "type")]
    pub type_value: String,
}
///A Realtime item listing tools available on an MCP server.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct RealtimeMcpListTools {
    ///The unique ID of the list.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: ::std::option::Option<String>,
    ///The label of the MCP server.
    pub server_label: String,
    ///The tools available on the server.
    pub tools: Vec<McpListToolsTool>,
    ///The type of the item. Always `mcp_list_tools`.
    #[serde(rename = "type")]
    pub type_value: String,
}
///Realtime MCP protocol error
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct RealtimeMcpProtocolError {
    pub code: i32,
    pub message: String,
    #[serde(rename = "type")]
    pub type_value: String,
}
///A Realtime item representing an invocation of a tool on an MCP server.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct RealtimeMcpToolCall {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub approval_request_id: ::std::option::Option<String>,
    ///A JSON string of the arguments passed to the tool.
    pub arguments: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: ::std::option::Option<RealtimeMcpToolCallError>,
    ///The unique ID of the tool call.
    pub id: String,
    ///The name of the tool that was run.
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub output: ::std::option::Option<String>,
    ///The label of the MCP server running the tool.
    pub server_label: String,
    ///The type of the item. Always `mcp_call`.
    #[serde(rename = "type")]
    pub type_value: String,
}
///The error from the tool call, if any.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
#[serde(untagged)]
pub enum RealtimeMcpToolCallError {
    RealtimeMcpProtocolError(RealtimeMcpProtocolError),
    RealtimeMcpToolExecutionError(RealtimeMcpToolExecutionError),
    RealtimeMcphttpError(RealtimeMcphttpError),
}
///Realtime MCP tool execution error
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct RealtimeMcpToolExecutionError {
    pub message: String,
    #[serde(rename = "type")]
    pub type_value: String,
}
///Realtime MCP HTTP error
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct RealtimeMcphttpError {
    pub code: i32,
    pub message: String,
    #[serde(rename = "type")]
    pub type_value: String,
}
///Configuration for reasoning-capable Realtime models such as `gpt-realtime-2`.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct RealtimeReasoning {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub effort: ::std::option::Option<RealtimeReasoningEffort>,
}
///Constrains effort on reasoning for reasoning-capable Realtime models such as `gpt-realtime-2`.
pub type RealtimeReasoningEffort = String;
///The response resource.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct RealtimeResponse {
    ///Configuration for audio output.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub audio: ::std::option::Option<RealtimeResponseAudio>,
    ///Which conversation the response is added to, determined by the `conversation` field in the `response.create` event. If `auto`, the response will be added to the default conversation and the value of `conversation_id` will be an id like `conv_1234`. If `none`, the response will not be added to any conversation and the value of `conversation_id` will be `null`. If responses are being triggered automatically by VAD the response will be added to the default conversation
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub conversation_id: ::std::option::Option<String>,
    ///The unique ID of the response, will look like `resp_1234`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: ::std::option::Option<String>,
    ///Maximum number of output tokens for a single assistant response, inclusive of tool calls, that was used in this response.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_output_tokens: ::std::option::Option<RealtimeResponseMaxOutputTokens>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub metadata: ::std::option::Option<Metadata>,
    ///The object type, must be `realtime.response`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub object: ::std::option::Option<String>,
    ///The list of output items generated by the response.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub output: ::std::option::Option<Vec<RealtimeConversationItem>>,
    ///The set of modalities the model used to respond, currently the only possible values are `[\"audio\"]`, `[\"text\"]`. Audio output always include a text transcript. Setting the output to mode `text` will disable audio output from the model.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub output_modalities: ::std::option::Option<Vec<String>>,
    ///The final status of the response (`completed`, `cancelled`, `failed`, or `incomplete`, `in_progress`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub status: ::std::option::Option<String>,
    ///Additional details about the status.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub status_details: ::std::option::Option<RealtimeResponseStatusDetails>,
    ///Usage statistics for the Response, this will correspond to billing. A Realtime API session will maintain a conversation context and append new Items to the Conversation, thus output from previous turns (text and audio tokens) will become the input for later turns.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub usage: ::std::option::Option<RealtimeResponseUsage>,
}
///Configuration for audio output.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct RealtimeResponseAudio {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub output: ::std::option::Option<RealtimeResponseAudioOutput>,
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct RealtimeResponseAudioOutput {
    ///The format of the output audio.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub format: ::std::option::Option<RealtimeAudioFormats>,
    ///The voice the model uses to respond. Voice cannot be changed during the session once the model has responded with audio at least once. Current voice options are `alloy`, `ash`, `ballad`, `coral`, `echo`, `sage`, `shimmer`, `verse`, `marin`, and `cedar`. We recommend `marin` and `cedar` for best quality.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub voice: ::std::option::Option<VoiceIdsShared>,
}
///Create a new Realtime response with these parameters
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct RealtimeResponseCreateParams {
    ///Configuration for audio input and output.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub audio: ::std::option::Option<RealtimeResponseCreateParamsAudio>,
    ///Controls which conversation the response is added to. Currently supports `auto` and `none`, with `auto` as the default value. The `auto` value means that the contents of the response will be added to the default conversation. Set this to `none` to create an out-of-band response which will not add items to default conversation.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub conversation: ::std::option::Option<String>,
    ///Input items to include in the prompt for the model. Using this field creates a new context for this Response instead of using the default conversation. An empty array `[]` will clear the context for this Response. Note that this can include references to items that previously appeared in the session using their id.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub input: ::std::option::Option<Vec<RealtimeConversationItem>>,
    ///The default system instructions (i.e. system message) prepended to model calls. This field allows the client to guide the model on desired responses. The model can be instructed on response content and format, (e.g. "be extremely succinct", "act friendly", "here are examples of good responses") and on audio behavior (e.g. "talk quickly", "inject emotion into your voice", "laugh frequently"). The instructions are not guaranteed to be followed by the model, but they provide guidance to the model on the desired behavior. Note that the server sets default instructions which will be used if this field is not set and are visible in the `session.created` event at the start of the session.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub instructions: ::std::option::Option<String>,
    ///Maximum number of output tokens for a single assistant response, inclusive of tool calls. Provide an integer between 1 and 4096 to limit output tokens, or `inf` for the maximum available tokens for a given model. Defaults to `inf`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_output_tokens: ::std::option::Option<
        RealtimeResponseCreateParamsMaxOutputTokens,
    >,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub metadata: ::std::option::Option<Metadata>,
    ///The set of modalities the model used to respond, currently the only possible values are `[\"audio\"]`, `[\"text\"]`. Audio output always include a text transcript. Setting the output to mode `text` will disable audio output from the model.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub output_modalities: ::std::option::Option<Vec<String>>,
    ///Whether the model may call multiple tools in parallel. Only supported by reasoning Realtime models such as `gpt-realtime-2`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parallel_tool_calls: ::std::option::Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub prompt: ::std::option::Option<Prompt>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reasoning: ::std::option::Option<RealtimeReasoning>,
    ///How the model chooses tools. Provide one of the string modes or force a specific function/MCP tool.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tool_choice: ::std::option::Option<RealtimeResponseCreateParamsToolChoice>,
    ///Tools available to the model.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tools: ::std::option::Option<Vec<RealtimeResponseCreateParamsTool>>,
}
///Configuration for audio input and output.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct RealtimeResponseCreateParamsAudio {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub output: ::std::option::Option<RealtimeResponseCreateParamsAudioOutput>,
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct RealtimeResponseCreateParamsAudioOutput {
    ///The format of the output audio.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub format: ::std::option::Option<RealtimeAudioFormats>,
    ///The voice the model uses to respond. Supported built-in voices are `alloy`, `ash`, `ballad`, `coral`, `echo`, `sage`, `shimmer`, `verse`, `marin`, and `cedar`. You may also provide a custom voice object with an `id`, for example `{ "id": "voice_1234" }`. Voice cannot be changed during the session once the model has responded with audio at least once. We recommend `marin` and `cedar` for best quality.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub voice: ::std::option::Option<VoiceIdsOrCustomVoice>,
}
///Maximum number of output tokens for a single assistant response, inclusive of tool calls. Provide an integer between 1 and 4096 to limit output tokens, or `inf` for the maximum available tokens for a given model. Defaults to `inf`.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
#[serde(untagged)]
pub enum RealtimeResponseCreateParamsMaxOutputTokens {
    Integer(i32),
    Inf(String),
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
#[serde(untagged)]
pub enum RealtimeResponseCreateParamsTool {
    RealtimeFunctionTool(RealtimeFunctionTool),
    McpTool(McpTool),
}
///How the model chooses tools. Provide one of the string modes or force a specific function/MCP tool.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
#[serde(untagged)]
pub enum RealtimeResponseCreateParamsToolChoice {
    ToolChoiceOptions(ToolChoiceOptions),
    ToolChoiceFunction(ToolChoiceFunction),
    ToolChoiceMcp(ToolChoiceMcp),
}
///Maximum number of output tokens for a single assistant response, inclusive of tool calls, that was used in this response.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
#[serde(untagged)]
pub enum RealtimeResponseMaxOutputTokens {
    Integer(i32),
    Inf(String),
}
///Additional details about the status.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct RealtimeResponseStatusDetails {
    ///A description of the error that caused the response to fail, populated when the `status` is `failed`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: ::std::option::Option<RealtimeResponseStatusDetailsError>,
    ///The reason the Response did not complete. For a `cancelled` Response, one of `turn_detected` (the server VAD detected a new start of speech) or `client_cancelled` (the client sent a cancel event). For an `incomplete` Response, one of `max_output_tokens` or `content_filter` (the server-side safety filter activated and cut off the response).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reason: ::std::option::Option<String>,
    ///The type of error that caused the response to fail, corresponding with the `status` field (`completed`, `cancelled`, `incomplete`, `failed`).
    #[serde(rename = "type", default, skip_serializing_if = "Option::is_none")]
    pub type_value: ::std::option::Option<String>,
}
///A description of the error that caused the response to fail, populated when the `status` is `failed`.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct RealtimeResponseStatusDetailsError {
    ///Error code, if any.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub code: ::std::option::Option<String>,
    ///The type of error.
    #[serde(rename = "type", default, skip_serializing_if = "Option::is_none")]
    pub type_value: ::std::option::Option<String>,
}
///Usage statistics for the Response, this will correspond to billing. A Realtime API session will maintain a conversation context and append new Items to the Conversation, thus output from previous turns (text and audio tokens) will become the input for later turns.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct RealtimeResponseUsage {
    ///Details about the input tokens used in the Response. Cached tokens are tokens from previous turns in the conversation that are included as context for the current response. Cached tokens here are counted as a subset of input tokens, meaning input tokens will include cached and uncached tokens.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub input_token_details: ::std::option::Option<
        RealtimeResponseUsageInputTokenDetails,
    >,
    ///The number of input tokens used in the Response, including text and audio tokens.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub input_tokens: ::std::option::Option<i32>,
    ///Details about the output tokens used in the Response.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub output_token_details: ::std::option::Option<
        RealtimeResponseUsageOutputTokenDetails,
    >,
    ///The number of output tokens sent in the Response, including text and audio tokens.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub output_tokens: ::std::option::Option<i32>,
    ///The total number of tokens in the Response including input and output text and audio tokens.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub total_tokens: ::std::option::Option<i32>,
}
///Details about the input tokens used in the Response. Cached tokens are tokens from previous turns in the conversation that are included as context for the current response. Cached tokens here are counted as a subset of input tokens, meaning input tokens will include cached and uncached tokens.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct RealtimeResponseUsageInputTokenDetails {
    ///The number of audio tokens used as input for the Response.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub audio_tokens: ::std::option::Option<i32>,
    ///The number of cached tokens used as input for the Response.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cached_tokens: ::std::option::Option<i32>,
    ///Details about the cached tokens used as input for the Response.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cached_tokens_details: ::std::option::Option<
        RealtimeResponseUsageInputTokenDetailsCachedTokensDetails,
    >,
    ///The number of image tokens used as input for the Response.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub image_tokens: ::std::option::Option<i32>,
    ///The number of text tokens used as input for the Response.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub text_tokens: ::std::option::Option<i32>,
}
///Details about the cached tokens used as input for the Response.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct RealtimeResponseUsageInputTokenDetailsCachedTokensDetails {
    ///The number of cached audio tokens used as input for the Response.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub audio_tokens: ::std::option::Option<i32>,
    ///The number of cached image tokens used as input for the Response.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub image_tokens: ::std::option::Option<i32>,
    ///The number of cached text tokens used as input for the Response.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub text_tokens: ::std::option::Option<i32>,
}
///Details about the output tokens used in the Response.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct RealtimeResponseUsageOutputTokenDetails {
    ///The number of audio tokens used in the Response.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub audio_tokens: ::std::option::Option<i32>,
    ///The number of text tokens used in the Response.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub text_tokens: ::std::option::Option<i32>,
}
///A realtime server event.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
#[serde(untagged)]
pub enum RealtimeServerEvent {
    RealtimeServerEventConversationCreated(RealtimeServerEventConversationCreated),
    RealtimeServerEventConversationItemCreated(
        RealtimeServerEventConversationItemCreated,
    ),
    RealtimeServerEventConversationItemDeleted(
        RealtimeServerEventConversationItemDeleted,
    ),
    RealtimeServerEventConversationItemInputAudioTranscriptionCompleted(
        RealtimeServerEventConversationItemInputAudioTranscriptionCompleted,
    ),
    RealtimeServerEventConversationItemInputAudioTranscriptionDelta(
        RealtimeServerEventConversationItemInputAudioTranscriptionDelta,
    ),
    RealtimeServerEventConversationItemInputAudioTranscriptionFailed(
        RealtimeServerEventConversationItemInputAudioTranscriptionFailed,
    ),
    RealtimeServerEventConversationItemRetrieved(
        RealtimeServerEventConversationItemRetrieved,
    ),
    RealtimeServerEventConversationItemTruncated(
        RealtimeServerEventConversationItemTruncated,
    ),
    RealtimeServerEventError(RealtimeServerEventError),
    RealtimeServerEventInputAudioBufferCleared(
        RealtimeServerEventInputAudioBufferCleared,
    ),
    RealtimeServerEventInputAudioBufferCommitted(
        RealtimeServerEventInputAudioBufferCommitted,
    ),
    RealtimeServerEventInputAudioBufferDtmfEventReceived(
        RealtimeServerEventInputAudioBufferDtmfEventReceived,
    ),
    RealtimeServerEventInputAudioBufferSpeechStarted(
        RealtimeServerEventInputAudioBufferSpeechStarted,
    ),
    RealtimeServerEventInputAudioBufferSpeechStopped(
        RealtimeServerEventInputAudioBufferSpeechStopped,
    ),
    RealtimeServerEventRateLimitsUpdated(RealtimeServerEventRateLimitsUpdated),
    RealtimeServerEventResponseAudioDelta(RealtimeServerEventResponseAudioDelta),
    RealtimeServerEventResponseAudioDone(RealtimeServerEventResponseAudioDone),
    RealtimeServerEventResponseAudioTranscriptDelta(
        RealtimeServerEventResponseAudioTranscriptDelta,
    ),
    RealtimeServerEventResponseAudioTranscriptDone(
        RealtimeServerEventResponseAudioTranscriptDone,
    ),
    RealtimeServerEventResponseContentPartAdded(
        RealtimeServerEventResponseContentPartAdded,
    ),
    RealtimeServerEventResponseContentPartDone(
        RealtimeServerEventResponseContentPartDone,
    ),
    RealtimeServerEventResponseCreated(RealtimeServerEventResponseCreated),
    RealtimeServerEventResponseDone(RealtimeServerEventResponseDone),
    RealtimeServerEventResponseFunctionCallArgumentsDelta(
        RealtimeServerEventResponseFunctionCallArgumentsDelta,
    ),
    RealtimeServerEventResponseFunctionCallArgumentsDone(
        RealtimeServerEventResponseFunctionCallArgumentsDone,
    ),
    RealtimeServerEventResponseOutputItemAdded(
        RealtimeServerEventResponseOutputItemAdded,
    ),
    RealtimeServerEventResponseOutputItemDone(RealtimeServerEventResponseOutputItemDone),
    RealtimeServerEventResponseTextDelta(RealtimeServerEventResponseTextDelta),
    RealtimeServerEventResponseTextDone(RealtimeServerEventResponseTextDone),
    RealtimeServerEventSessionCreated(RealtimeServerEventSessionCreated),
    RealtimeServerEventSessionUpdated(RealtimeServerEventSessionUpdated),
    RealtimeServerEventOutputAudioBufferStarted(
        RealtimeServerEventOutputAudioBufferStarted,
    ),
    RealtimeServerEventOutputAudioBufferStopped(
        RealtimeServerEventOutputAudioBufferStopped,
    ),
    RealtimeServerEventOutputAudioBufferCleared(
        RealtimeServerEventOutputAudioBufferCleared,
    ),
    RealtimeServerEventConversationItemAdded(RealtimeServerEventConversationItemAdded),
    RealtimeServerEventConversationItemDone(RealtimeServerEventConversationItemDone),
    RealtimeServerEventInputAudioBufferTimeoutTriggered(
        RealtimeServerEventInputAudioBufferTimeoutTriggered,
    ),
    RealtimeServerEventConversationItemInputAudioTranscriptionSegment(
        RealtimeServerEventConversationItemInputAudioTranscriptionSegment,
    ),
    RealtimeServerEventMcpListToolsInProgress(RealtimeServerEventMcpListToolsInProgress),
    RealtimeServerEventMcpListToolsCompleted(RealtimeServerEventMcpListToolsCompleted),
    RealtimeServerEventMcpListToolsFailed(RealtimeServerEventMcpListToolsFailed),
    RealtimeServerEventResponseMcpCallArgumentsDelta(
        RealtimeServerEventResponseMcpCallArgumentsDelta,
    ),
    RealtimeServerEventResponseMcpCallArgumentsDone(
        RealtimeServerEventResponseMcpCallArgumentsDone,
    ),
    RealtimeServerEventResponseMcpCallInProgress(
        RealtimeServerEventResponseMcpCallInProgress,
    ),
    RealtimeServerEventResponseMcpCallCompleted(
        RealtimeServerEventResponseMcpCallCompleted,
    ),
    RealtimeServerEventResponseMcpCallFailed(RealtimeServerEventResponseMcpCallFailed),
}
///Returned when a conversation is created. Emitted right after session creation.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct RealtimeServerEventConversationCreated {
    ///The conversation resource.
    pub conversation: RealtimeServerEventConversationCreatedConversation,
    ///The unique ID of the server event.
    pub event_id: String,
    ///The event type, must be `conversation.created`.
    #[serde(rename = "type")]
    pub type_value: String,
}
///The conversation resource.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct RealtimeServerEventConversationCreatedConversation {
    ///The unique ID of the conversation.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: ::std::option::Option<String>,
    ///The object type, must be `realtime.conversation`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub object: ::std::option::Option<String>,
}
///Sent by the server when an Item is added to the default Conversation. This can happen in several cases: - When the client sends a `conversation.item.create` event. - When the input audio buffer is committed. In this case the item will be a user message containing the audio from the buffer. - When the model is generating a Response. In this case the `conversation.item.added` event will be sent when the model starts generating a specific Item, and thus it will not yet have any content (and `status` will be `in_progress`). The event will include the full content of the Item (except when model is generating a Response) except for audio data, which can be retrieved separately with a `conversation.item.retrieve` event if necessary.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct RealtimeServerEventConversationItemAdded {
    ///The unique ID of the server event.
    pub event_id: String,
    pub item: RealtimeConversationItem,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub previous_item_id: ::std::option::Option<String>,
    ///The event type, must be `conversation.item.added`.
    #[serde(rename = "type")]
    pub type_value: String,
}
///Returned when a conversation item is created. There are several scenarios that produce this event: - The server is generating a Response, which if successful will produce either one or two Items, which will be of type `message` (role `assistant`) or type `function_call`. - The input audio buffer has been committed, either by the client or the server (in `server_vad` mode). The server will take the content of the input audio buffer and add it to a new user message Item. - The client has sent a `conversation.item.create` event to add a new Item to the Conversation.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct RealtimeServerEventConversationItemCreated {
    ///The unique ID of the server event.
    pub event_id: String,
    pub item: RealtimeConversationItem,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub previous_item_id: ::std::option::Option<String>,
    ///The event type, must be `conversation.item.created`.
    #[serde(rename = "type")]
    pub type_value: String,
}
///Returned when an item in the conversation is deleted by the client with a `conversation.item.delete` event. This event is used to synchronize the server's understanding of the conversation history with the client's view.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct RealtimeServerEventConversationItemDeleted {
    ///The unique ID of the server event.
    pub event_id: String,
    ///The ID of the item that was deleted.
    pub item_id: String,
    ///The event type, must be `conversation.item.deleted`.
    #[serde(rename = "type")]
    pub type_value: String,
}
///Returned when a conversation item is finalized. The event will include the full content of the Item except for audio data, which can be retrieved separately with a `conversation.item.retrieve` event if needed.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct RealtimeServerEventConversationItemDone {
    ///The unique ID of the server event.
    pub event_id: String,
    pub item: RealtimeConversationItem,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub previous_item_id: ::std::option::Option<String>,
    ///The event type, must be `conversation.item.done`.
    #[serde(rename = "type")]
    pub type_value: String,
}
///This event is the output of audio transcription for user audio written to the user audio buffer. Transcription begins when the input audio buffer is committed by the client or server (when VAD is enabled). Transcription runs asynchronously with Response creation, so this event may come before or after the Response events. Realtime API models accept audio natively, and thus input transcription is a separate process run on a separate ASR (Automatic Speech Recognition) model. The transcript may diverge somewhat from the model's interpretation, and should be treated as a rough guide.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct RealtimeServerEventConversationItemInputAudioTranscriptionCompleted {
    ///The index of the content part containing the audio.
    pub content_index: i32,
    ///The unique ID of the server event.
    pub event_id: String,
    ///The ID of the item containing the audio that is being transcribed.
    pub item_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub logprobs: ::std::option::Option<Vec<LogProbProperties>>,
    ///The transcribed text.
    pub transcript: String,
    ///The event type, must be `conversation.item.input_audio_transcription.completed`.
    #[serde(rename = "type")]
    pub type_value: String,
    ///Usage statistics for the transcription, this is billed according to the ASR model's pricing rather than the realtime model's pricing.
    pub usage: RealtimeServerEventConversationItemInputAudioTranscriptionCompletedUsage,
}
///Usage statistics for the transcription, this is billed according to the ASR model's pricing rather than the realtime model's pricing.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
#[serde(untagged)]
pub enum RealtimeServerEventConversationItemInputAudioTranscriptionCompletedUsage {
    TranscriptTextUsageTokens(TranscriptTextUsageTokens),
    TranscriptTextUsageDuration(TranscriptTextUsageDuration),
}
///Returned when the text value of an input audio transcription content part is updated with incremental transcription results.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct RealtimeServerEventConversationItemInputAudioTranscriptionDelta {
    ///The index of the content part in the item's content array.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub content_index: ::std::option::Option<i32>,
    ///The text delta.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub delta: ::std::option::Option<String>,
    ///The unique ID of the server event.
    pub event_id: String,
    ///The ID of the item containing the audio that is being transcribed.
    pub item_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub logprobs: ::std::option::Option<Vec<LogProbProperties>>,
    ///The event type, must be `conversation.item.input_audio_transcription.delta`.
    #[serde(rename = "type")]
    pub type_value: String,
}
///Returned when input audio transcription is configured, and a transcription request for a user message failed. These events are separate from other `error` events so that the client can identify the related Item.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct RealtimeServerEventConversationItemInputAudioTranscriptionFailed {
    ///The index of the content part containing the audio.
    pub content_index: i32,
    ///Details of the transcription error.
    pub error: RealtimeServerEventConversationItemInputAudioTranscriptionFailedError,
    ///The unique ID of the server event.
    pub event_id: String,
    ///The ID of the user message item.
    pub item_id: String,
    ///The event type, must be `conversation.item.input_audio_transcription.failed`.
    #[serde(rename = "type")]
    pub type_value: String,
}
///Details of the transcription error.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct RealtimeServerEventConversationItemInputAudioTranscriptionFailedError {
    ///Error code, if any.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub code: ::std::option::Option<String>,
    ///A human-readable error message.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub message: ::std::option::Option<String>,
    ///Parameter related to the error, if any.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub param: ::std::option::Option<String>,
    ///The type of error.
    #[serde(rename = "type", default, skip_serializing_if = "Option::is_none")]
    pub type_value: ::std::option::Option<String>,
}
///Returned when an input audio transcription segment is identified for an item.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct RealtimeServerEventConversationItemInputAudioTranscriptionSegment {
    ///The index of the input audio content part within the item.
    pub content_index: i32,
    ///End time of the segment in seconds.
    pub end: f64,
    ///The unique ID of the server event.
    pub event_id: String,
    ///The segment identifier.
    pub id: String,
    ///The ID of the item containing the input audio content.
    pub item_id: String,
    ///The detected speaker label for this segment.
    pub speaker: String,
    ///Start time of the segment in seconds.
    pub start: f64,
    ///The text for this segment.
    pub text: String,
    ///The event type, must be `conversation.item.input_audio_transcription.segment`.
    #[serde(rename = "type")]
    pub type_value: String,
}
///Returned when a conversation item is retrieved with `conversation.item.retrieve`. This is provided as a way to fetch the server's representation of an item, for example to get access to the post-processed audio data after noise cancellation and VAD. It includes the full content of the Item, including audio data.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct RealtimeServerEventConversationItemRetrieved {
    ///The unique ID of the server event.
    pub event_id: String,
    pub item: RealtimeConversationItem,
    ///The event type, must be `conversation.item.retrieved`.
    #[serde(rename = "type")]
    pub type_value: String,
}
///Returned when an earlier assistant audio message item is truncated by the client with a `conversation.item.truncate` event. This event is used to synchronize the server's understanding of the audio with the client's playback. This action will truncate the audio and remove the server-side text transcript to ensure there is no text in the context that hasn't been heard by the user.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct RealtimeServerEventConversationItemTruncated {
    ///The duration up to which the audio was truncated, in milliseconds.
    pub audio_end_ms: i32,
    ///The index of the content part that was truncated.
    pub content_index: i32,
    ///The unique ID of the server event.
    pub event_id: String,
    ///The ID of the assistant message item that was truncated.
    pub item_id: String,
    ///The event type, must be `conversation.item.truncated`.
    #[serde(rename = "type")]
    pub type_value: String,
}
///Returned when an error occurs, which could be a client problem or a server problem. Most errors are recoverable and the session will stay open, we recommend to implementors to monitor and log error messages by default.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct RealtimeServerEventError {
    ///Details of the error.
    pub error: RealtimeServerEventErrorError,
    ///The unique ID of the server event.
    pub event_id: String,
    ///The event type, must be `error`.
    #[serde(rename = "type")]
    pub type_value: String,
}
///Details of the error.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct RealtimeServerEventErrorError {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub code: ::std::option::Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub event_id: ::std::option::Option<String>,
    ///A human-readable error message.
    pub message: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub param: ::std::option::Option<String>,
    ///The type of error (e.g., "invalid_request_error", "server_error").
    #[serde(rename = "type")]
    pub type_value: String,
}
///Returned when the input audio buffer is cleared by the client with a `input_audio_buffer.clear` event.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct RealtimeServerEventInputAudioBufferCleared {
    ///The unique ID of the server event.
    pub event_id: String,
    ///The event type, must be `input_audio_buffer.cleared`.
    #[serde(rename = "type")]
    pub type_value: String,
}
///Returned when an input audio buffer is committed, either by the client or automatically in server VAD mode. The `item_id` property is the ID of the user message item that will be created, thus a `conversation.item.created` event will also be sent to the client.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct RealtimeServerEventInputAudioBufferCommitted {
    ///The unique ID of the server event.
    pub event_id: String,
    ///The ID of the user message item that will be created.
    pub item_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub previous_item_id: ::std::option::Option<String>,
    ///The event type, must be `input_audio_buffer.committed`.
    #[serde(rename = "type")]
    pub type_value: String,
}
///**SIP Only:** Returned when an DTMF event is received. A DTMF event is a message that represents a telephone keypad press (0–9, *, #, A–D). The `event` property is the keypad that the user press. The `received_at` is the UTC Unix Timestamp that the server received the event.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct RealtimeServerEventInputAudioBufferDtmfEventReceived {
    ///The telephone keypad that was pressed by the user.
    pub event: String,
    ///UTC Unix Timestamp when DTMF Event was received by server.
    pub received_at: i32,
    ///The event type, must be `input_audio_buffer.dtmf_event_received`.
    #[serde(rename = "type")]
    pub type_value: String,
}
///Sent by the server when in `server_vad` mode to indicate that speech has been detected in the audio buffer. This can happen any time audio is added to the buffer (unless speech is already detected). The client may want to use this event to interrupt audio playback or provide visual feedback to the user. The client should expect to receive a `input_audio_buffer.speech_stopped` event when speech stops. The `item_id` property is the ID of the user message item that will be created when speech stops and will also be included in the `input_audio_buffer.speech_stopped` event (unless the client manually commits the audio buffer during VAD activation).
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct RealtimeServerEventInputAudioBufferSpeechStarted {
    ///Milliseconds from the start of all audio written to the buffer during the session when speech was first detected. This will correspond to the beginning of audio sent to the model, and thus includes the `prefix_padding_ms` configured in the Session.
    pub audio_start_ms: i32,
    ///The unique ID of the server event.
    pub event_id: String,
    ///The ID of the user message item that will be created when speech stops.
    pub item_id: String,
    ///The event type, must be `input_audio_buffer.speech_started`.
    #[serde(rename = "type")]
    pub type_value: String,
}
///Returned in `server_vad` mode when the server detects the end of speech in the audio buffer. The server will also send an `conversation.item.created` event with the user message item that is created from the audio buffer.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct RealtimeServerEventInputAudioBufferSpeechStopped {
    ///Milliseconds since the session started when speech stopped. This will correspond to the end of audio sent to the model, and thus includes the `min_silence_duration_ms` configured in the Session.
    pub audio_end_ms: i32,
    ///The unique ID of the server event.
    pub event_id: String,
    ///The ID of the user message item that will be created.
    pub item_id: String,
    ///The event type, must be `input_audio_buffer.speech_stopped`.
    #[serde(rename = "type")]
    pub type_value: String,
}
///Returned when the Server VAD timeout is triggered for the input audio buffer. This is configured with `idle_timeout_ms` in the `turn_detection` settings of the session, and it indicates that there hasn't been any speech detected for the configured duration. The `audio_start_ms` and `audio_end_ms` fields indicate the segment of audio after the last model response up to the triggering time, as an offset from the beginning of audio written to the input audio buffer. This means it demarcates the segment of audio that was silent and the difference between the start and end values will roughly match the configured timeout. The empty audio will be committed to the conversation as an `input_audio` item (there will be a `input_audio_buffer.committed` event) and a model response will be generated. There may be speech that didn't trigger VAD but is still detected by the model, so the model may respond with something relevant to the conversation or a prompt to continue speaking.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct RealtimeServerEventInputAudioBufferTimeoutTriggered {
    ///Millisecond offset of audio written to the input audio buffer at the time the timeout was triggered.
    pub audio_end_ms: i32,
    ///Millisecond offset of audio written to the input audio buffer that was after the playback time of the last model response.
    pub audio_start_ms: i32,
    ///The unique ID of the server event.
    pub event_id: String,
    ///The ID of the item associated with this segment.
    pub item_id: String,
    ///The event type, must be `input_audio_buffer.timeout_triggered`.
    #[serde(rename = "type")]
    pub type_value: String,
}
///Returned when listing MCP tools has completed for an item.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct RealtimeServerEventMcpListToolsCompleted {
    ///The unique ID of the server event.
    pub event_id: String,
    ///The ID of the MCP list tools item.
    pub item_id: String,
    ///The event type, must be `mcp_list_tools.completed`.
    #[serde(rename = "type")]
    pub type_value: String,
}
///Returned when listing MCP tools has failed for an item.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct RealtimeServerEventMcpListToolsFailed {
    ///The unique ID of the server event.
    pub event_id: String,
    ///The ID of the MCP list tools item.
    pub item_id: String,
    ///The event type, must be `mcp_list_tools.failed`.
    #[serde(rename = "type")]
    pub type_value: String,
}
///Returned when listing MCP tools is in progress for an item.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct RealtimeServerEventMcpListToolsInProgress {
    ///The unique ID of the server event.
    pub event_id: String,
    ///The ID of the MCP list tools item.
    pub item_id: String,
    ///The event type, must be `mcp_list_tools.in_progress`.
    #[serde(rename = "type")]
    pub type_value: String,
}
///**WebRTC/SIP Only:** Emitted when the output audio buffer is cleared. This happens either in VAD mode when the user has interrupted (`input_audio_buffer.speech_started`), or when the client has emitted the `output_audio_buffer.clear` event to manually cut off the current audio response. [Learn more](/docs/guides/realtime-conversations#client-and-server-events-for-audio-in-webrtc).
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct RealtimeServerEventOutputAudioBufferCleared {
    ///The unique ID of the server event.
    pub event_id: String,
    ///The unique ID of the response that produced the audio.
    pub response_id: String,
    ///The event type, must be `output_audio_buffer.cleared`.
    #[serde(rename = "type")]
    pub type_value: String,
}
///**WebRTC/SIP Only:** Emitted when the server begins streaming audio to the client. This event is emitted after an audio content part has been added (`response.content_part.added`) to the response. [Learn more](/docs/guides/realtime-conversations#client-and-server-events-for-audio-in-webrtc).
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct RealtimeServerEventOutputAudioBufferStarted {
    ///The unique ID of the server event.
    pub event_id: String,
    ///The unique ID of the response that produced the audio.
    pub response_id: String,
    ///The event type, must be `output_audio_buffer.started`.
    #[serde(rename = "type")]
    pub type_value: String,
}
///**WebRTC/SIP Only:** Emitted when the output audio buffer has been completely drained on the server, and no more audio is forthcoming. This event is emitted after the full response data has been sent to the client (`response.done`). [Learn more](/docs/guides/realtime-conversations#client-and-server-events-for-audio-in-webrtc).
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct RealtimeServerEventOutputAudioBufferStopped {
    ///The unique ID of the server event.
    pub event_id: String,
    ///The unique ID of the response that produced the audio.
    pub response_id: String,
    ///The event type, must be `output_audio_buffer.stopped`.
    #[serde(rename = "type")]
    pub type_value: String,
}
///Emitted at the beginning of a Response to indicate the updated rate limits. When a Response is created some tokens will be "reserved" for the output tokens, the rate limits shown here reflect that reservation, which is then adjusted accordingly once the Response is completed.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct RealtimeServerEventRateLimitsUpdated {
    ///The unique ID of the server event.
    pub event_id: String,
    ///List of rate limit information.
    pub rate_limits: Vec<RealtimeServerEventRateLimitsUpdatedRateLimit>,
    ///The event type, must be `rate_limits.updated`.
    #[serde(rename = "type")]
    pub type_value: String,
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct RealtimeServerEventRateLimitsUpdatedRateLimit {
    ///The maximum allowed value for the rate limit.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub limit: ::std::option::Option<i32>,
    ///The name of the rate limit (`requests`, `tokens`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: ::std::option::Option<String>,
    ///The remaining value before the limit is reached.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub remaining: ::std::option::Option<i32>,
    ///Seconds until the rate limit resets.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reset_seconds: ::std::option::Option<f64>,
}
///Returned when the model-generated audio is updated.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct RealtimeServerEventResponseAudioDelta {
    ///The index of the content part in the item's content array.
    pub content_index: i32,
    ///Base64-encoded audio data delta.
    pub delta: String,
    ///The unique ID of the server event.
    pub event_id: String,
    ///The ID of the item.
    pub item_id: String,
    ///The index of the output item in the response.
    pub output_index: i32,
    ///The ID of the response.
    pub response_id: String,
    ///The event type, must be `response.output_audio.delta`.
    #[serde(rename = "type")]
    pub type_value: String,
}
///Returned when the model-generated audio is done. Also emitted when a Response is interrupted, incomplete, or cancelled.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct RealtimeServerEventResponseAudioDone {
    ///The index of the content part in the item's content array.
    pub content_index: i32,
    ///The unique ID of the server event.
    pub event_id: String,
    ///The ID of the item.
    pub item_id: String,
    ///The index of the output item in the response.
    pub output_index: i32,
    ///The ID of the response.
    pub response_id: String,
    ///The event type, must be `response.output_audio.done`.
    #[serde(rename = "type")]
    pub type_value: String,
}
///Returned when the model-generated transcription of audio output is updated.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct RealtimeServerEventResponseAudioTranscriptDelta {
    ///The index of the content part in the item's content array.
    pub content_index: i32,
    ///The transcript delta.
    pub delta: String,
    ///The unique ID of the server event.
    pub event_id: String,
    ///The ID of the item.
    pub item_id: String,
    ///The index of the output item in the response.
    pub output_index: i32,
    ///The ID of the response.
    pub response_id: String,
    ///The event type, must be `response.output_audio_transcript.delta`.
    #[serde(rename = "type")]
    pub type_value: String,
}
///Returned when the model-generated transcription of audio output is done streaming. Also emitted when a Response is interrupted, incomplete, or cancelled.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct RealtimeServerEventResponseAudioTranscriptDone {
    ///The index of the content part in the item's content array.
    pub content_index: i32,
    ///The unique ID of the server event.
    pub event_id: String,
    ///The ID of the item.
    pub item_id: String,
    ///The index of the output item in the response.
    pub output_index: i32,
    ///The ID of the response.
    pub response_id: String,
    ///The final transcript of the audio.
    pub transcript: String,
    ///The event type, must be `response.output_audio_transcript.done`.
    #[serde(rename = "type")]
    pub type_value: String,
}
///Returned when a new content part is added to an assistant message item during response generation.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct RealtimeServerEventResponseContentPartAdded {
    ///The index of the content part in the item's content array.
    pub content_index: i32,
    ///The unique ID of the server event.
    pub event_id: String,
    ///The ID of the item to which the content part was added.
    pub item_id: String,
    ///The index of the output item in the response.
    pub output_index: i32,
    ///The content part that was added.
    pub part: RealtimeServerEventResponseContentPartAddedPart,
    ///The ID of the response.
    pub response_id: String,
    ///The event type, must be `response.content_part.added`.
    #[serde(rename = "type")]
    pub type_value: String,
}
///The content part that was added.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct RealtimeServerEventResponseContentPartAddedPart {
    ///Base64-encoded audio data (if type is "audio").
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub audio: ::std::option::Option<String>,
    ///The text content (if type is "text").
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub text: ::std::option::Option<String>,
    ///The transcript of the audio (if type is "audio").
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub transcript: ::std::option::Option<String>,
    ///The content type ("text", "audio").
    #[serde(rename = "type", default, skip_serializing_if = "Option::is_none")]
    pub type_value: ::std::option::Option<String>,
}
///Returned when a content part is done streaming in an assistant message item. Also emitted when a Response is interrupted, incomplete, or cancelled.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct RealtimeServerEventResponseContentPartDone {
    ///The index of the content part in the item's content array.
    pub content_index: i32,
    ///The unique ID of the server event.
    pub event_id: String,
    ///The ID of the item.
    pub item_id: String,
    ///The index of the output item in the response.
    pub output_index: i32,
    ///The content part that is done.
    pub part: RealtimeServerEventResponseContentPartDonePart,
    ///The ID of the response.
    pub response_id: String,
    ///The event type, must be `response.content_part.done`.
    #[serde(rename = "type")]
    pub type_value: String,
}
///The content part that is done.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct RealtimeServerEventResponseContentPartDonePart {
    ///Base64-encoded audio data (if type is "audio").
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub audio: ::std::option::Option<String>,
    ///The text content (if type is "text").
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub text: ::std::option::Option<String>,
    ///The transcript of the audio (if type is "audio").
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub transcript: ::std::option::Option<String>,
    ///The content type ("text", "audio").
    #[serde(rename = "type", default, skip_serializing_if = "Option::is_none")]
    pub type_value: ::std::option::Option<String>,
}
///Returned when a new Response is created. The first event of response creation, where the response is in an initial state of `in_progress`.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct RealtimeServerEventResponseCreated {
    ///The unique ID of the server event.
    pub event_id: String,
    pub response: RealtimeResponse,
    ///The event type, must be `response.created`.
    #[serde(rename = "type")]
    pub type_value: String,
}
///Returned when a Response is done streaming. Always emitted, no matter the final state. The Response object included in the `response.done` event will include all output Items in the Response but will omit the raw audio data. Clients should check the `status` field of the Response to determine if it was successful (`completed`) or if there was another outcome: `cancelled`, `failed`, or `incomplete`. A response will contain all output items that were generated during the response, excluding any audio content.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct RealtimeServerEventResponseDone {
    ///The unique ID of the server event.
    pub event_id: String,
    pub response: RealtimeResponse,
    ///The event type, must be `response.done`.
    #[serde(rename = "type")]
    pub type_value: String,
}
///Returned when the model-generated function call arguments are updated.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct RealtimeServerEventResponseFunctionCallArgumentsDelta {
    ///The ID of the function call.
    pub call_id: String,
    ///The arguments delta as a JSON string.
    pub delta: String,
    ///The unique ID of the server event.
    pub event_id: String,
    ///The ID of the function call item.
    pub item_id: String,
    ///The index of the output item in the response.
    pub output_index: i32,
    ///The ID of the response.
    pub response_id: String,
    ///The event type, must be `response.function_call_arguments.delta`.
    #[serde(rename = "type")]
    pub type_value: String,
}
///Returned when the model-generated function call arguments are done streaming. Also emitted when a Response is interrupted, incomplete, or cancelled.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct RealtimeServerEventResponseFunctionCallArgumentsDone {
    ///The final arguments as a JSON string.
    pub arguments: String,
    ///The ID of the function call.
    pub call_id: String,
    ///The unique ID of the server event.
    pub event_id: String,
    ///The ID of the function call item.
    pub item_id: String,
    ///The name of the function that was called.
    pub name: String,
    ///The index of the output item in the response.
    pub output_index: i32,
    ///The ID of the response.
    pub response_id: String,
    ///The event type, must be `response.function_call_arguments.done`.
    #[serde(rename = "type")]
    pub type_value: String,
}
///Returned when MCP tool call arguments are updated during response generation.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct RealtimeServerEventResponseMcpCallArgumentsDelta {
    ///The JSON-encoded arguments delta.
    pub delta: String,
    ///The unique ID of the server event.
    pub event_id: String,
    ///The ID of the MCP tool call item.
    pub item_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub obfuscation: ::std::option::Option<String>,
    ///The index of the output item in the response.
    pub output_index: i32,
    ///The ID of the response.
    pub response_id: String,
    ///The event type, must be `response.mcp_call_arguments.delta`.
    #[serde(rename = "type")]
    pub type_value: String,
}
///Returned when MCP tool call arguments are finalized during response generation.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct RealtimeServerEventResponseMcpCallArgumentsDone {
    ///The final JSON-encoded arguments string.
    pub arguments: String,
    ///The unique ID of the server event.
    pub event_id: String,
    ///The ID of the MCP tool call item.
    pub item_id: String,
    ///The index of the output item in the response.
    pub output_index: i32,
    ///The ID of the response.
    pub response_id: String,
    ///The event type, must be `response.mcp_call_arguments.done`.
    #[serde(rename = "type")]
    pub type_value: String,
}
///Returned when an MCP tool call has completed successfully.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct RealtimeServerEventResponseMcpCallCompleted {
    ///The unique ID of the server event.
    pub event_id: String,
    ///The ID of the MCP tool call item.
    pub item_id: String,
    ///The index of the output item in the response.
    pub output_index: i32,
    ///The event type, must be `response.mcp_call.completed`.
    #[serde(rename = "type")]
    pub type_value: String,
}
///Returned when an MCP tool call has failed.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct RealtimeServerEventResponseMcpCallFailed {
    ///The unique ID of the server event.
    pub event_id: String,
    ///The ID of the MCP tool call item.
    pub item_id: String,
    ///The index of the output item in the response.
    pub output_index: i32,
    ///The event type, must be `response.mcp_call.failed`.
    #[serde(rename = "type")]
    pub type_value: String,
}
///Returned when an MCP tool call has started and is in progress.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct RealtimeServerEventResponseMcpCallInProgress {
    ///The unique ID of the server event.
    pub event_id: String,
    ///The ID of the MCP tool call item.
    pub item_id: String,
    ///The index of the output item in the response.
    pub output_index: i32,
    ///The event type, must be `response.mcp_call.in_progress`.
    #[serde(rename = "type")]
    pub type_value: String,
}
///Returned when a new Item is created during Response generation.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct RealtimeServerEventResponseOutputItemAdded {
    ///The unique ID of the server event.
    pub event_id: String,
    pub item: RealtimeConversationItem,
    ///The index of the output item in the Response.
    pub output_index: i32,
    ///The ID of the Response to which the item belongs.
    pub response_id: String,
    ///The event type, must be `response.output_item.added`.
    #[serde(rename = "type")]
    pub type_value: String,
}
///Returned when an Item is done streaming. Also emitted when a Response is interrupted, incomplete, or cancelled.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct RealtimeServerEventResponseOutputItemDone {
    ///The unique ID of the server event.
    pub event_id: String,
    pub item: RealtimeConversationItem,
    ///The index of the output item in the Response.
    pub output_index: i32,
    ///The ID of the Response to which the item belongs.
    pub response_id: String,
    ///The event type, must be `response.output_item.done`.
    #[serde(rename = "type")]
    pub type_value: String,
}
///Returned when the text value of an "output_text" content part is updated.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct RealtimeServerEventResponseTextDelta {
    ///The index of the content part in the item's content array.
    pub content_index: i32,
    ///The text delta.
    pub delta: String,
    ///The unique ID of the server event.
    pub event_id: String,
    ///The ID of the item.
    pub item_id: String,
    ///The index of the output item in the response.
    pub output_index: i32,
    ///The ID of the response.
    pub response_id: String,
    ///The event type, must be `response.output_text.delta`.
    #[serde(rename = "type")]
    pub type_value: String,
}
///Returned when the text value of an "output_text" content part is done streaming. Also emitted when a Response is interrupted, incomplete, or cancelled.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct RealtimeServerEventResponseTextDone {
    ///The index of the content part in the item's content array.
    pub content_index: i32,
    ///The unique ID of the server event.
    pub event_id: String,
    ///The ID of the item.
    pub item_id: String,
    ///The index of the output item in the response.
    pub output_index: i32,
    ///The ID of the response.
    pub response_id: String,
    ///The final text content.
    pub text: String,
    ///The event type, must be `response.output_text.done`.
    #[serde(rename = "type")]
    pub type_value: String,
}
///Returned when a Session is created. Emitted automatically when a new connection is established as the first server event. This event will contain the default Session configuration.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct RealtimeServerEventSessionCreated {
    ///The unique ID of the server event.
    pub event_id: String,
    ///The session configuration.
    pub session: RealtimeServerEventSessionCreatedSession,
    ///The event type, must be `session.created`.
    #[serde(rename = "type")]
    pub type_value: String,
}
///The session configuration.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
#[serde(untagged)]
pub enum RealtimeServerEventSessionCreatedSession {
    RealtimeSessionCreateResponseGa(RealtimeSessionCreateResponseGa),
    RealtimeTranscriptionSessionCreateResponseGa(
        RealtimeTranscriptionSessionCreateResponseGa,
    ),
}
///Returned when a session is updated with a `session.update` event, unless there is an error.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct RealtimeServerEventSessionUpdated {
    ///The unique ID of the server event.
    pub event_id: String,
    ///The session configuration.
    pub session: RealtimeServerEventSessionUpdatedSession,
    ///The event type, must be `session.updated`.
    #[serde(rename = "type")]
    pub type_value: String,
}
///The session configuration.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
#[serde(untagged)]
pub enum RealtimeServerEventSessionUpdatedSession {
    RealtimeSessionCreateResponseGa(RealtimeSessionCreateResponseGa),
    RealtimeTranscriptionSessionCreateResponseGa(
        RealtimeTranscriptionSessionCreateResponseGa,
    ),
}
///Returned when a transcription session is updated with a `transcription_session.update` event, unless there is an error.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct RealtimeServerEventTranscriptionSessionUpdated {
    ///The unique ID of the server event.
    pub event_id: String,
    pub session: RealtimeTranscriptionSessionCreateResponse,
    ///The event type, must be `transcription_session.updated`.
    #[serde(rename = "type")]
    pub type_value: String,
}
///Realtime session object for the beta interface.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct RealtimeSession {
    ///Expiration timestamp for the session, in seconds since epoch.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expires_at: ::std::option::Option<i64>,
    ///Unique identifier for the session that looks like `sess_1234567890abcdef`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: ::std::option::Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub include: ::std::option::Option<Vec<String>>,
    ///The format of input audio. Options are `pcm16`, `g711_ulaw`, or `g711_alaw`. For `pcm16`, input audio must be 16-bit PCM at a 24kHz sample rate, single channel (mono), and little-endian byte order.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub input_audio_format: ::std::option::Option<String>,
    ///Configuration for input audio noise reduction. This can be set to `null` to turn off. Noise reduction filters audio added to the input audio buffer before it is sent to VAD and the model. Filtering the audio can improve VAD and turn detection accuracy (reducing false positives) and model performance by improving perception of the input audio.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub input_audio_noise_reduction: ::std::option::Option<
        RealtimeSessionInputAudioNoiseReduction,
    >,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub input_audio_transcription: ::std::option::Option<
        RealtimeSessionInputAudioTranscription,
    >,
    ///The default system instructions (i.e. system message) prepended to model calls. This field allows the client to guide the model on desired responses. The model can be instructed on response content and format, (e.g. "be extremely succinct", "act friendly", "here are examples of good responses") and on audio behavior (e.g. "talk quickly", "inject emotion into your voice", "laugh frequently"). The instructions are not guaranteed to be followed by the model, but they provide guidance to the model on the desired behavior. Note that the server sets default instructions which will be used if this field is not set and are visible in the `session.created` event at the start of the session.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub instructions: ::std::option::Option<String>,
    ///Maximum number of output tokens for a single assistant response, inclusive of tool calls. Provide an integer between 1 and 4096 to limit output tokens, or `inf` for the maximum available tokens for a given model. Defaults to `inf`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_response_output_tokens: ::std::option::Option<
        RealtimeSessionMaxResponseOutputTokens,
    >,
    ///The set of modalities the model can respond with. To disable audio, set this to ["text"].
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub modalities: ::std::option::Option<OpenAiJsonValue>,
    ///The Realtime model used for this session.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model: ::std::option::Option<String>,
    ///The object type. Always `realtime.session`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub object: ::std::option::Option<String>,
    ///The format of output audio. Options are `pcm16`, `g711_ulaw`, or `g711_alaw`. For `pcm16`, output audio is sampled at a rate of 24kHz.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub output_audio_format: ::std::option::Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub prompt: ::std::option::Option<Prompt>,
    ///The speed of the model's spoken response. 1.0 is the default speed. 0.25 is the minimum speed. 1.5 is the maximum speed. This value can only be changed in between model turns, not while a response is in progress.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub speed: ::std::option::Option<f64>,
    ///Sampling temperature for the model, limited to [0.6, 1.2]. For audio models a temperature of 0.8 is highly recommended for best performance.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub temperature: ::std::option::Option<f64>,
    ///How the model chooses tools. Options are `auto`, `none`, `required`, or specify a function.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tool_choice: ::std::option::Option<String>,
    ///Tools (functions) available to the model.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tools: ::std::option::Option<Vec<RealtimeFunctionTool>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tracing: ::std::option::Option<RealtimeSessionTracing2>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub turn_detection: ::std::option::Option<RealtimeTurnDetection>,
    ///The voice the model uses to respond. Voice cannot be changed during the session once the model has responded with audio at least once. Current voice options are `alloy`, `ash`, `ballad`, `coral`, `echo`, `sage`, `shimmer`, and `verse`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub voice: ::std::option::Option<VoiceIdsShared>,
}
///A new Realtime session configuration, with an ephemeral key. Default TTL for keys is one minute.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct RealtimeSessionCreateRequest {
    ///Ephemeral key returned by the API.
    pub client_secret: RealtimeSessionCreateRequestClientSecret,
    ///The format of input audio. Options are `pcm16`, `g711_ulaw`, or `g711_alaw`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub input_audio_format: ::std::option::Option<String>,
    ///Configuration for input audio transcription, defaults to off and can be set to `null` to turn off once on. Input audio transcription is not native to the model, since the model consumes audio directly. Transcription runs asynchronously and should be treated as rough guidance rather than the representation understood by the model.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub input_audio_transcription: ::std::option::Option<
        RealtimeSessionCreateRequestInputAudioTranscription,
    >,
    ///The default system instructions (i.e. system message) prepended to model calls. This field allows the client to guide the model on desired responses. The model can be instructed on response content and format, (e.g. "be extremely succinct", "act friendly", "here are examples of good responses") and on audio behavior (e.g. "talk quickly", "inject emotion into your voice", "laugh frequently"). The instructions are not guaranteed to be followed by the model, but they provide guidance to the model on the desired behavior. Note that the server sets default instructions which will be used if this field is not set and are visible in the `session.created` event at the start of the session.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub instructions: ::std::option::Option<String>,
    ///Maximum number of output tokens for a single assistant response, inclusive of tool calls. Provide an integer between 1 and 4096 to limit output tokens, or `inf` for the maximum available tokens for a given model. Defaults to `inf`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_response_output_tokens: ::std::option::Option<
        RealtimeSessionCreateRequestMaxResponseOutputTokens,
    >,
    ///The set of modalities the model can respond with. To disable audio, set this to ["text"].
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub modalities: ::std::option::Option<OpenAiJsonValue>,
    ///The format of output audio. Options are `pcm16`, `g711_ulaw`, or `g711_alaw`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub output_audio_format: ::std::option::Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub prompt: ::std::option::Option<Prompt>,
    ///The speed of the model's spoken response. 1.0 is the default speed. 0.25 is the minimum speed. 1.5 is the maximum speed. This value can only be changed in between model turns, not while a response is in progress.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub speed: ::std::option::Option<f64>,
    ///Sampling temperature for the model, limited to [0.6, 1.2]. Defaults to 0.8.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub temperature: ::std::option::Option<f64>,
    ///How the model chooses tools. Options are `auto`, `none`, `required`, or specify a function.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tool_choice: ::std::option::Option<String>,
    ///Tools (functions) available to the model.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tools: ::std::option::Option<Vec<RealtimeSessionCreateRequestTool>>,
    ///Configuration options for tracing. Set to null to disable tracing. Once tracing is enabled for a session, the configuration cannot be modified. `auto` will create a trace for the session with default values for the workflow name, group id, and metadata.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tracing: ::std::option::Option<RealtimeSessionCreateRequestTracing2>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub truncation: ::std::option::Option<RealtimeTruncation>,
    ///Configuration for turn detection. Can be set to `null` to turn off. Server VAD means that the model will detect the start and end of speech based on audio volume and respond at the end of user speech.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub turn_detection: ::std::option::Option<RealtimeSessionCreateRequestTurnDetection>,
    ///The voice the model uses to respond. Supported built-in voices are `alloy`, `ash`, `ballad`, `coral`, `echo`, `sage`, `shimmer`, `verse`, `marin`, and `cedar`. You may also provide a custom voice object with an `id`, for example `{ "id": "voice_1234" }`. Voice cannot be changed during the session once the model has responded with audio at least once.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub voice: ::std::option::Option<VoiceIdsOrCustomVoice>,
}
///Ephemeral key returned by the API.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct RealtimeSessionCreateRequestClientSecret {
    ///Timestamp for when the token expires. Currently, all tokens expire after one minute.
    pub expires_at: i64,
    ///Ephemeral key usable in client environments to authenticate connections to the Realtime API. Use this in client-side environments rather than a standard API token, which should only be used server-side.
    pub value: String,
}
///Realtime session object configuration.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct RealtimeSessionCreateRequestGa {
    ///Configuration for input and output audio.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub audio: ::std::option::Option<RealtimeSessionCreateRequestGaAudio>,
    ///Additional fields to include in server outputs. `item.input_audio_transcription.logprobs`: Include logprobs for input audio transcription.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub include: ::std::option::Option<Vec<String>>,
    ///The default system instructions (i.e. system message) prepended to model calls. This field allows the client to guide the model on desired responses. The model can be instructed on response content and format, (e.g. "be extremely succinct", "act friendly", "here are examples of good responses") and on audio behavior (e.g. "talk quickly", "inject emotion into your voice", "laugh frequently"). The instructions are not guaranteed to be followed by the model, but they provide guidance to the model on the desired behavior. Note that the server sets default instructions which will be used if this field is not set and are visible in the `session.created` event at the start of the session.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub instructions: ::std::option::Option<String>,
    ///Maximum number of output tokens for a single assistant response, inclusive of tool calls. Provide an integer between 1 and 4096 to limit output tokens, or `inf` for the maximum available tokens for a given model. Defaults to `inf`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_output_tokens: ::std::option::Option<
        RealtimeSessionCreateRequestGaMaxOutputTokens,
    >,
    ///The Realtime model used for this session.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model: ::std::option::Option<String>,
    ///The set of modalities the model can respond with. It defaults to `["audio"]`, indicating that the model will respond with audio plus a transcript. `["text"]` can be used to make the model respond with text only. It is not possible to request both `text` and `audio` at the same time.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub output_modalities: ::std::option::Option<Vec<String>>,
    ///Whether the model may call multiple tools in parallel. Only supported by reasoning Realtime models such as `gpt-realtime-2`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parallel_tool_calls: ::std::option::Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub prompt: ::std::option::Option<Prompt>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reasoning: ::std::option::Option<RealtimeReasoning>,
    ///How the model chooses tools. Provide one of the string modes or force a specific function/MCP tool.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tool_choice: ::std::option::Option<RealtimeSessionCreateRequestGaToolChoice>,
    ///Tools available to the model.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tools: ::std::option::Option<Vec<RealtimeSessionCreateRequestGaTool>>,
    ///Realtime API can write session traces to the [Traces Dashboard](https://platform.openai.com/logs?api=traces). Set to null to disable tracing. Once tracing is enabled for a session, the configuration cannot be modified. `auto` will create a trace for the session with default values for the workflow name, group id, and metadata.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tracing: ::std::option::Option<RealtimeSessionCreateRequestGaTracing2>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub truncation: ::std::option::Option<RealtimeTruncation>,
    ///The type of session to create. Always `realtime` for the Realtime API.
    #[serde(rename = "type")]
    pub type_value: String,
}
///Configuration for input and output audio.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct RealtimeSessionCreateRequestGaAudio {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub input: ::std::option::Option<RealtimeSessionCreateRequestGaAudioInput>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub output: ::std::option::Option<RealtimeSessionCreateRequestGaAudioOutput>,
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct RealtimeSessionCreateRequestGaAudioInput {
    ///The format of the input audio.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub format: ::std::option::Option<RealtimeAudioFormats>,
    ///Configuration for input audio noise reduction. This can be set to `null` to turn off. Noise reduction filters audio added to the input audio buffer before it is sent to VAD and the model. Filtering the audio can improve VAD and turn detection accuracy (reducing false positives) and model performance by improving perception of the input audio.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub noise_reduction: ::std::option::Option<
        RealtimeSessionCreateRequestGaAudioInputNoiseReduction,
    >,
    ///Configuration for input audio transcription, defaults to off and can be set to `null` to turn off once on. Input audio transcription is not native to the model, since the model consumes audio directly. Transcription runs asynchronously through [the /audio/transcriptions endpoint](/docs/api-reference/audio/createTranscription) and should be treated as guidance of input audio content rather than precisely what the model heard. The client can optionally set the language and prompt for transcription, these offer additional guidance to the transcription service.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub transcription: ::std::option::Option<AudioTranscription>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub turn_detection: ::std::option::Option<RealtimeTurnDetection>,
}
///Configuration for input audio noise reduction. This can be set to `null` to turn off. Noise reduction filters audio added to the input audio buffer before it is sent to VAD and the model. Filtering the audio can improve VAD and turn detection accuracy (reducing false positives) and model performance by improving perception of the input audio.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct RealtimeSessionCreateRequestGaAudioInputNoiseReduction {
    #[serde(rename = "type", default, skip_serializing_if = "Option::is_none")]
    pub type_value: ::std::option::Option<NoiseReductionType>,
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct RealtimeSessionCreateRequestGaAudioOutput {
    ///The format of the output audio.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub format: ::std::option::Option<RealtimeAudioFormats>,
    ///The speed of the model's spoken response as a multiple of the original speed. 1.0 is the default speed. 0.25 is the minimum speed. 1.5 is the maximum speed. This value can only be changed in between model turns, not while a response is in progress. This parameter is a post-processing adjustment to the audio after it is generated, it's also possible to prompt the model to speak faster or slower.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub speed: ::std::option::Option<f64>,
    ///The voice the model uses to respond. Supported built-in voices are `alloy`, `ash`, `ballad`, `coral`, `echo`, `sage`, `shimmer`, `verse`, `marin`, and `cedar`. You may also provide a custom voice object with an `id`, for example `{ "id": "voice_1234" }`. Voice cannot be changed during the session once the model has responded with audio at least once. We recommend `marin` and `cedar` for best quality.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub voice: ::std::option::Option<VoiceIdsOrCustomVoice>,
}
///Maximum number of output tokens for a single assistant response, inclusive of tool calls. Provide an integer between 1 and 4096 to limit output tokens, or `inf` for the maximum available tokens for a given model. Defaults to `inf`.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
#[serde(untagged)]
pub enum RealtimeSessionCreateRequestGaMaxOutputTokens {
    Integer(i32),
    Inf(String),
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
#[serde(untagged)]
pub enum RealtimeSessionCreateRequestGaTool {
    RealtimeFunctionTool(RealtimeFunctionTool),
    McpTool(McpTool),
}
///How the model chooses tools. Provide one of the string modes or force a specific function/MCP tool.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
#[serde(untagged)]
pub enum RealtimeSessionCreateRequestGaToolChoice {
    ToolChoiceOptions(ToolChoiceOptions),
    ToolChoiceFunction(ToolChoiceFunction),
    ToolChoiceMcp(ToolChoiceMcp),
}
///Granular configuration for tracing.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct RealtimeSessionCreateRequestGaTracing {
    ///The group id to attach to this trace to enable filtering and grouping in the Traces Dashboard.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub group_id: ::std::option::Option<String>,
    ///The arbitrary metadata to attach to this trace to enable filtering in the Traces Dashboard.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub metadata: ::std::option::Option<OpenAiJsonValue>,
    ///The name of the workflow to attach to this trace. This is used to name the trace in the Traces Dashboard.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub workflow_name: ::std::option::Option<String>,
}
///Realtime API can write session traces to the [Traces Dashboard](https://platform.openai.com/logs?api=traces). Set to null to disable tracing. Once tracing is enabled for a session, the configuration cannot be modified. `auto` will create a trace for the session with default values for the workflow name, group id, and metadata.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
#[serde(untagged)]
pub enum RealtimeSessionCreateRequestGaTracing2 {
    Auto(String),
    TracingConfiguration(RealtimeSessionCreateRequestGaTracing2TracingConfiguration),
}
///Granular configuration for tracing.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct RealtimeSessionCreateRequestGaTracing2TracingConfiguration {
    ///The group id to attach to this trace to enable filtering and grouping in the Traces Dashboard.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub group_id: ::std::option::Option<String>,
    ///The arbitrary metadata to attach to this trace to enable filtering in the Traces Dashboard.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub metadata: ::std::option::Option<OpenAiJsonValue>,
    ///The name of the workflow to attach to this trace. This is used to name the trace in the Traces Dashboard.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub workflow_name: ::std::option::Option<String>,
}
///Configuration for input audio transcription, defaults to off and can be set to `null` to turn off once on. Input audio transcription is not native to the model, since the model consumes audio directly. Transcription runs asynchronously and should be treated as rough guidance rather than the representation understood by the model.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct RealtimeSessionCreateRequestInputAudioTranscription {
    ///The model to use for transcription.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model: ::std::option::Option<String>,
}
///Maximum number of output tokens for a single assistant response, inclusive of tool calls. Provide an integer between 1 and 4096 to limit output tokens, or `inf` for the maximum available tokens for a given model. Defaults to `inf`.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
#[serde(untagged)]
pub enum RealtimeSessionCreateRequestMaxResponseOutputTokens {
    Integer(i32),
    Inf(String),
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct RealtimeSessionCreateRequestTool {
    ///The description of the function, including guidance on when and how to call it, and guidance about what to tell the user when calling (if anything).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: ::std::option::Option<String>,
    ///The name of the function.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: ::std::option::Option<String>,
    ///Parameters of the function in JSON Schema.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parameters: ::std::option::Option<OpenAiJsonValue>,
    ///The type of the tool, i.e. `function`.
    #[serde(rename = "type", default, skip_serializing_if = "Option::is_none")]
    pub type_value: ::std::option::Option<String>,
}
///Granular configuration for tracing.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct RealtimeSessionCreateRequestTracing {
    ///The group id to attach to this trace to enable filtering and grouping in the traces dashboard.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub group_id: ::std::option::Option<String>,
    ///The arbitrary metadata to attach to this trace to enable filtering in the traces dashboard.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub metadata: ::std::option::Option<OpenAiJsonValue>,
    ///The name of the workflow to attach to this trace. This is used to name the trace in the traces dashboard.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub workflow_name: ::std::option::Option<String>,
}
///Configuration options for tracing. Set to null to disable tracing. Once tracing is enabled for a session, the configuration cannot be modified. `auto` will create a trace for the session with default values for the workflow name, group id, and metadata.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
#[serde(untagged)]
pub enum RealtimeSessionCreateRequestTracing2 {
    Auto(String),
    TracingConfiguration(RealtimeSessionCreateRequestTracing2TracingConfiguration),
}
///Granular configuration for tracing.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct RealtimeSessionCreateRequestTracing2TracingConfiguration {
    ///The group id to attach to this trace to enable filtering and grouping in the traces dashboard.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub group_id: ::std::option::Option<String>,
    ///The arbitrary metadata to attach to this trace to enable filtering in the traces dashboard.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub metadata: ::std::option::Option<OpenAiJsonValue>,
    ///The name of the workflow to attach to this trace. This is used to name the trace in the traces dashboard.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub workflow_name: ::std::option::Option<String>,
}
///Configuration for turn detection. Can be set to `null` to turn off. Server VAD means that the model will detect the start and end of speech based on audio volume and respond at the end of user speech.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct RealtimeSessionCreateRequestTurnDetection {
    ///Amount of audio to include before the VAD detected speech (in milliseconds). Defaults to 300ms.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub prefix_padding_ms: ::std::option::Option<i32>,
    ///Duration of silence to detect speech stop (in milliseconds). Defaults to 500ms. With shorter values the model will respond more quickly, but may jump in on short pauses from the user.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub silence_duration_ms: ::std::option::Option<i32>,
    ///Activation threshold for VAD (0.0 to 1.0), this defaults to 0.5. A higher threshold will require louder audio to activate the model, and thus might perform better in noisy environments.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub threshold: ::std::option::Option<f64>,
    ///Type of turn detection, only `server_vad` is currently supported.
    #[serde(rename = "type", default, skip_serializing_if = "Option::is_none")]
    pub type_value: ::std::option::Option<String>,
}
///A Realtime session configuration object.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct RealtimeSessionCreateResponse {
    ///Configuration for input and output audio for the session.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub audio: ::std::option::Option<RealtimeSessionCreateResponseAudio>,
    ///Expiration timestamp for the session, in seconds since epoch.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expires_at: ::std::option::Option<i64>,
    ///Unique identifier for the session that looks like `sess_1234567890abcdef`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: ::std::option::Option<String>,
    ///Additional fields to include in server outputs. - `item.input_audio_transcription.logprobs`: Include logprobs for input audio transcription.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub include: ::std::option::Option<Vec<String>>,
    ///The default system instructions (i.e. system message) prepended to model calls. This field allows the client to guide the model on desired responses. The model can be instructed on response content and format, (e.g. "be extremely succinct", "act friendly", "here are examples of good responses") and on audio behavior (e.g. "talk quickly", "inject emotion into your voice", "laugh frequently"). The instructions are not guaranteed to be followed by the model, but they provide guidance to the model on the desired behavior. Note that the server sets default instructions which will be used if this field is not set and are visible in the `session.created` event at the start of the session.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub instructions: ::std::option::Option<String>,
    ///Maximum number of output tokens for a single assistant response, inclusive of tool calls. Provide an integer between 1 and 4096 to limit output tokens, or `inf` for the maximum available tokens for a given model. Defaults to `inf`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_output_tokens: ::std::option::Option<
        RealtimeSessionCreateResponseMaxOutputTokens,
    >,
    ///The Realtime model used for this session.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model: ::std::option::Option<String>,
    ///The object type. Always `realtime.session`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub object: ::std::option::Option<String>,
    ///The set of modalities the model can respond with. To disable audio, set this to ["text"].
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub output_modalities: ::std::option::Option<OpenAiJsonValue>,
    ///How the model chooses tools. Options are `auto`, `none`, `required`, or specify a function.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tool_choice: ::std::option::Option<String>,
    ///Tools (functions) available to the model.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tools: ::std::option::Option<Vec<RealtimeFunctionTool>>,
    ///Configuration options for tracing. Set to null to disable tracing. Once tracing is enabled for a session, the configuration cannot be modified. `auto` will create a trace for the session with default values for the workflow name, group id, and metadata.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tracing: ::std::option::Option<RealtimeSessionCreateResponseTracing2>,
    ///Configuration for turn detection. Can be set to `null` to turn off. Server VAD means that the model will detect the start and end of speech based on audio volume and respond at the end of user speech.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub turn_detection: ::std::option::Option<
        RealtimeSessionCreateResponseTurnDetection,
    >,
}
///Configuration for input and output audio for the session.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct RealtimeSessionCreateResponseAudio {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub input: ::std::option::Option<RealtimeSessionCreateResponseAudioInput>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub output: ::std::option::Option<RealtimeSessionCreateResponseAudioOutput>,
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct RealtimeSessionCreateResponseAudioInput {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub format: ::std::option::Option<RealtimeAudioFormats>,
    ///Configuration for input audio noise reduction.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub noise_reduction: ::std::option::Option<
        RealtimeSessionCreateResponseAudioInputNoiseReduction,
    >,
    ///Configuration for input audio transcription.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub transcription: ::std::option::Option<AudioTranscriptionResponse>,
    ///Configuration for turn detection.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub turn_detection: ::std::option::Option<
        RealtimeSessionCreateResponseAudioInputTurnDetection,
    >,
}
///Configuration for input audio noise reduction.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct RealtimeSessionCreateResponseAudioInputNoiseReduction {
    #[serde(rename = "type", default, skip_serializing_if = "Option::is_none")]
    pub type_value: ::std::option::Option<NoiseReductionType>,
}
///Configuration for turn detection.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct RealtimeSessionCreateResponseAudioInputTurnDetection {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub prefix_padding_ms: ::std::option::Option<i32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub silence_duration_ms: ::std::option::Option<i32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub threshold: ::std::option::Option<f64>,
    ///Type of turn detection, only `server_vad` is currently supported.
    #[serde(rename = "type", default, skip_serializing_if = "Option::is_none")]
    pub type_value: ::std::option::Option<String>,
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct RealtimeSessionCreateResponseAudioOutput {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub format: ::std::option::Option<RealtimeAudioFormats>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub speed: ::std::option::Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub voice: ::std::option::Option<VoiceIdsShared>,
}
///A Realtime session configuration object.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct RealtimeSessionCreateResponseGa {
    ///Configuration for input and output audio.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub audio: ::std::option::Option<RealtimeSessionCreateResponseGaAudio>,
    ///Expiration timestamp for the session, in seconds since epoch.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expires_at: ::std::option::Option<i64>,
    ///Unique identifier for the session that looks like `sess_1234567890abcdef`.
    pub id: String,
    ///Additional fields to include in server outputs. `item.input_audio_transcription.logprobs`: Include logprobs for input audio transcription.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub include: ::std::option::Option<Vec<String>>,
    ///The default system instructions (i.e. system message) prepended to model calls. This field allows the client to guide the model on desired responses. The model can be instructed on response content and format, (e.g. "be extremely succinct", "act friendly", "here are examples of good responses") and on audio behavior (e.g. "talk quickly", "inject emotion into your voice", "laugh frequently"). The instructions are not guaranteed to be followed by the model, but they provide guidance to the model on the desired behavior. Note that the server sets default instructions which will be used if this field is not set and are visible in the `session.created` event at the start of the session.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub instructions: ::std::option::Option<String>,
    ///Maximum number of output tokens for a single assistant response, inclusive of tool calls. Provide an integer between 1 and 4096 to limit output tokens, or `inf` for the maximum available tokens for a given model. Defaults to `inf`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_output_tokens: ::std::option::Option<
        RealtimeSessionCreateResponseGaMaxOutputTokens,
    >,
    ///The Realtime model used for this session.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model: ::std::option::Option<String>,
    ///The object type. Always `realtime.session`.
    pub object: String,
    ///The set of modalities the model can respond with. It defaults to `["audio"]`, indicating that the model will respond with audio plus a transcript. `["text"]` can be used to make the model respond with text only. It is not possible to request both `text` and `audio` at the same time.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub output_modalities: ::std::option::Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub prompt: ::std::option::Option<Prompt>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reasoning: ::std::option::Option<RealtimeReasoning>,
    ///How the model chooses tools. Provide one of the string modes or force a specific function/MCP tool.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tool_choice: ::std::option::Option<RealtimeSessionCreateResponseGaToolChoice>,
    ///Tools available to the model.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tools: ::std::option::Option<Vec<RealtimeSessionCreateResponseGaTool>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tracing: ::std::option::Option<RealtimeSessionCreateResponseGaTracing2>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub truncation: ::std::option::Option<RealtimeTruncation>,
    ///The type of session to create. Always `realtime` for the Realtime API.
    #[serde(rename = "type")]
    pub type_value: String,
}
///Configuration for input and output audio.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct RealtimeSessionCreateResponseGaAudio {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub input: ::std::option::Option<RealtimeSessionCreateResponseGaAudioInput>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub output: ::std::option::Option<RealtimeSessionCreateResponseGaAudioOutput>,
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct RealtimeSessionCreateResponseGaAudioInput {
    ///The format of the input audio.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub format: ::std::option::Option<RealtimeAudioFormats>,
    ///Configuration for input audio noise reduction. This can be set to `null` to turn off. Noise reduction filters audio added to the input audio buffer before it is sent to VAD and the model. Filtering the audio can improve VAD and turn detection accuracy (reducing false positives) and model performance by improving perception of the input audio.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub noise_reduction: ::std::option::Option<
        RealtimeSessionCreateResponseGaAudioInputNoiseReduction,
    >,
    ///Configuration for input audio transcription, defaults to off and can be set to `null` to turn off once on. Input audio transcription is not native to the model, since the model consumes audio directly. Transcription runs asynchronously through [the /audio/transcriptions endpoint](/docs/api-reference/audio/createTranscription) and should be treated as guidance of input audio content rather than precisely what the model heard. The client can optionally set the language and prompt for transcription, these offer additional guidance to the transcription service.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub transcription: ::std::option::Option<AudioTranscriptionResponse>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub turn_detection: ::std::option::Option<RealtimeTurnDetection>,
}
///Configuration for input audio noise reduction. This can be set to `null` to turn off. Noise reduction filters audio added to the input audio buffer before it is sent to VAD and the model. Filtering the audio can improve VAD and turn detection accuracy (reducing false positives) and model performance by improving perception of the input audio.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct RealtimeSessionCreateResponseGaAudioInputNoiseReduction {
    #[serde(rename = "type", default, skip_serializing_if = "Option::is_none")]
    pub type_value: ::std::option::Option<NoiseReductionType>,
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct RealtimeSessionCreateResponseGaAudioOutput {
    ///The format of the output audio.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub format: ::std::option::Option<RealtimeAudioFormats>,
    ///The speed of the model's spoken response as a multiple of the original speed. 1.0 is the default speed. 0.25 is the minimum speed. 1.5 is the maximum speed. This value can only be changed in between model turns, not while a response is in progress. This parameter is a post-processing adjustment to the audio after it is generated, it's also possible to prompt the model to speak faster or slower.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub speed: ::std::option::Option<f64>,
    ///The voice the model uses to respond. Voice cannot be changed during the session once the model has responded with audio at least once. Current voice options are `alloy`, `ash`, `ballad`, `coral`, `echo`, `sage`, `shimmer`, `verse`, `marin`, and `cedar`. We recommend `marin` and `cedar` for best quality.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub voice: ::std::option::Option<VoiceIdsShared>,
}
///Maximum number of output tokens for a single assistant response, inclusive of tool calls. Provide an integer between 1 and 4096 to limit output tokens, or `inf` for the maximum available tokens for a given model. Defaults to `inf`.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
#[serde(untagged)]
pub enum RealtimeSessionCreateResponseGaMaxOutputTokens {
    Integer(i32),
    Inf(String),
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
#[serde(untagged)]
pub enum RealtimeSessionCreateResponseGaTool {
    RealtimeFunctionTool(RealtimeFunctionTool),
    McpTool(McpTool),
}
///How the model chooses tools. Provide one of the string modes or force a specific function/MCP tool.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
#[serde(untagged)]
pub enum RealtimeSessionCreateResponseGaToolChoice {
    ToolChoiceOptions(ToolChoiceOptions),
    ToolChoiceFunction(ToolChoiceFunction),
    ToolChoiceMcp(ToolChoiceMcp),
}
///Granular configuration for tracing.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct RealtimeSessionCreateResponseGaTracing {
    ///The group id to attach to this trace to enable filtering and grouping in the Traces Dashboard.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub group_id: ::std::option::Option<String>,
    ///The arbitrary metadata to attach to this trace to enable filtering in the Traces Dashboard.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub metadata: ::std::option::Option<OpenAiJsonValue>,
    ///The name of the workflow to attach to this trace. This is used to name the trace in the Traces Dashboard.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub workflow_name: ::std::option::Option<String>,
}
///Realtime API can write session traces to the [Traces Dashboard](https://platform.openai.com/logs?api=traces). Set to null to disable tracing. Once tracing is enabled for a session, the configuration cannot be modified. `auto` will create a trace for the session with default values for the workflow name, group id, and metadata.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
#[serde(untagged)]
pub enum RealtimeSessionCreateResponseGaTracing2 {
    Auto(String),
    TracingConfiguration(RealtimeSessionCreateResponseGaTracing2TracingConfiguration),
}
///Granular configuration for tracing.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct RealtimeSessionCreateResponseGaTracing2TracingConfiguration {
    ///The group id to attach to this trace to enable filtering and grouping in the Traces Dashboard.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub group_id: ::std::option::Option<String>,
    ///The arbitrary metadata to attach to this trace to enable filtering in the Traces Dashboard.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub metadata: ::std::option::Option<OpenAiJsonValue>,
    ///The name of the workflow to attach to this trace. This is used to name the trace in the Traces Dashboard.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub workflow_name: ::std::option::Option<String>,
}
///Maximum number of output tokens for a single assistant response, inclusive of tool calls. Provide an integer between 1 and 4096 to limit output tokens, or `inf` for the maximum available tokens for a given model. Defaults to `inf`.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
#[serde(untagged)]
pub enum RealtimeSessionCreateResponseMaxOutputTokens {
    Integer(i32),
    Inf(String),
}
///Granular configuration for tracing.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct RealtimeSessionCreateResponseTracing {
    ///The group id to attach to this trace to enable filtering and grouping in the traces dashboard.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub group_id: ::std::option::Option<String>,
    ///The arbitrary metadata to attach to this trace to enable filtering in the traces dashboard.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub metadata: ::std::option::Option<OpenAiJsonValue>,
    ///The name of the workflow to attach to this trace. This is used to name the trace in the traces dashboard.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub workflow_name: ::std::option::Option<String>,
}
///Configuration options for tracing. Set to null to disable tracing. Once tracing is enabled for a session, the configuration cannot be modified. `auto` will create a trace for the session with default values for the workflow name, group id, and metadata.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
#[serde(untagged)]
pub enum RealtimeSessionCreateResponseTracing2 {
    Auto(String),
    TracingConfiguration(RealtimeSessionCreateResponseTracing2TracingConfiguration),
}
///Granular configuration for tracing.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct RealtimeSessionCreateResponseTracing2TracingConfiguration {
    ///The group id to attach to this trace to enable filtering and grouping in the traces dashboard.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub group_id: ::std::option::Option<String>,
    ///The arbitrary metadata to attach to this trace to enable filtering in the traces dashboard.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub metadata: ::std::option::Option<OpenAiJsonValue>,
    ///The name of the workflow to attach to this trace. This is used to name the trace in the traces dashboard.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub workflow_name: ::std::option::Option<String>,
}
///Configuration for turn detection. Can be set to `null` to turn off. Server VAD means that the model will detect the start and end of speech based on audio volume and respond at the end of user speech.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct RealtimeSessionCreateResponseTurnDetection {
    ///Amount of audio to include before the VAD detected speech (in milliseconds). Defaults to 300ms.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub prefix_padding_ms: ::std::option::Option<i32>,
    ///Duration of silence to detect speech stop (in milliseconds). Defaults to 500ms. With shorter values the model will respond more quickly, but may jump in on short pauses from the user.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub silence_duration_ms: ::std::option::Option<i32>,
    ///Activation threshold for VAD (0.0 to 1.0), this defaults to 0.5. A higher threshold will require louder audio to activate the model, and thus might perform better in noisy environments.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub threshold: ::std::option::Option<f64>,
    ///Type of turn detection, only `server_vad` is currently supported.
    #[serde(rename = "type", default, skip_serializing_if = "Option::is_none")]
    pub type_value: ::std::option::Option<String>,
}
///Configuration for input audio noise reduction. This can be set to `null` to turn off. Noise reduction filters audio added to the input audio buffer before it is sent to VAD and the model. Filtering the audio can improve VAD and turn detection accuracy (reducing false positives) and model performance by improving perception of the input audio.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct RealtimeSessionInputAudioNoiseReduction {
    #[serde(rename = "type", default, skip_serializing_if = "Option::is_none")]
    pub type_value: ::std::option::Option<NoiseReductionType>,
}
///Configuration for input audio transcription, defaults to off and can be set to `null` to turn off once on. Input audio transcription is not native to the model, since the model consumes audio directly. Transcription runs asynchronously through [the /audio/transcriptions endpoint](https://platform.openai.com/docs/api-reference/audio/createTranscription) and should be treated as guidance of input audio content rather than precisely what the model heard. The client can optionally set the language and prompt for transcription, these offer additional guidance to the transcription service.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct RealtimeSessionInputAudioTranscription {
    ///The language of the input audio.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub language: ::std::option::Option<String>,
    ///The model used for transcription. Current options are `whisper-1`, `gpt-4o-mini-transcribe`, `gpt-4o-mini-transcribe-2025-12-15`, `gpt-4o-transcribe`, `gpt-4o-transcribe-diarize`, and `gpt-realtime-whisper`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model: ::std::option::Option<String>,
    ///The prompt configured for input audio transcription, when present.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub prompt: ::std::option::Option<String>,
}
///Maximum number of output tokens for a single assistant response, inclusive of tool calls. Provide an integer between 1 and 4096 to limit output tokens, or `inf` for the maximum available tokens for a given model. Defaults to `inf`.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
#[serde(untagged)]
pub enum RealtimeSessionMaxResponseOutputTokens {
    Integer(i32),
    Inf(String),
}
///Granular configuration for tracing.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct RealtimeSessionTracing {
    ///The group id to attach to this trace to enable filtering and grouping in the traces dashboard.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub group_id: ::std::option::Option<String>,
    ///The arbitrary metadata to attach to this trace to enable filtering in the traces dashboard.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub metadata: ::std::option::Option<OpenAiJsonValue>,
    ///The name of the workflow to attach to this trace. This is used to name the trace in the traces dashboard.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub workflow_name: ::std::option::Option<String>,
}
///Configuration options for tracing. Set to null to disable tracing. Once tracing is enabled for a session, the configuration cannot be modified. `auto` will create a trace for the session with default values for the workflow name, group id, and metadata.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
#[serde(untagged)]
pub enum RealtimeSessionTracing2 {
    Auto(String),
    TracingConfiguration(RealtimeSessionTracing2TracingConfiguration),
}
///Granular configuration for tracing.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct RealtimeSessionTracing2TracingConfiguration {
    ///The group id to attach to this trace to enable filtering and grouping in the traces dashboard.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub group_id: ::std::option::Option<String>,
    ///The arbitrary metadata to attach to this trace to enable filtering in the traces dashboard.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub metadata: ::std::option::Option<OpenAiJsonValue>,
    ///The name of the workflow to attach to this trace. This is used to name the trace in the traces dashboard.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub workflow_name: ::std::option::Option<String>,
}
///Realtime transcription session object configuration.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct RealtimeTranscriptionSessionCreateRequest {
    ///The set of items to include in the transcription. Current available items are: `item.input_audio_transcription.logprobs`
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub include: ::std::option::Option<Vec<String>>,
    ///The format of input audio. Options are `pcm16`, `g711_ulaw`, or `g711_alaw`. For `pcm16`, input audio must be 16-bit PCM at a 24kHz sample rate, single channel (mono), and little-endian byte order.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub input_audio_format: ::std::option::Option<String>,
    ///Configuration for input audio noise reduction. This can be set to `null` to turn off. Noise reduction filters audio added to the input audio buffer before it is sent to VAD and the model. Filtering the audio can improve VAD and turn detection accuracy (reducing false positives) and model performance by improving perception of the input audio.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub input_audio_noise_reduction: ::std::option::Option<
        RealtimeTranscriptionSessionCreateRequestInputAudioNoiseReduction,
    >,
    ///Configuration for input audio transcription. The client can optionally set the language and prompt for transcription, these offer additional guidance to the transcription service.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub input_audio_transcription: ::std::option::Option<AudioTranscription>,
    ///Configuration for turn detection. Can be set to `null` to turn off. Server VAD means that the model will detect the start and end of speech based on audio volume and respond at the end of user speech.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub turn_detection: ::std::option::Option<
        RealtimeTranscriptionSessionCreateRequestTurnDetection,
    >,
}
///Realtime transcription session object configuration.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct RealtimeTranscriptionSessionCreateRequestGa {
    ///Configuration for input and output audio.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub audio: ::std::option::Option<RealtimeTranscriptionSessionCreateRequestGaAudio>,
    ///Additional fields to include in server outputs. `item.input_audio_transcription.logprobs`: Include logprobs for input audio transcription.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub include: ::std::option::Option<Vec<String>>,
    ///The type of session to create. Always `transcription` for transcription sessions.
    #[serde(rename = "type")]
    pub type_value: String,
}
///Configuration for input and output audio.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct RealtimeTranscriptionSessionCreateRequestGaAudio {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub input: ::std::option::Option<
        RealtimeTranscriptionSessionCreateRequestGaAudioInput,
    >,
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct RealtimeTranscriptionSessionCreateRequestGaAudioInput {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub format: ::std::option::Option<RealtimeAudioFormats>,
    ///Configuration for input audio noise reduction. This can be set to `null` to turn off. Noise reduction filters audio added to the input audio buffer before it is sent to VAD and the model. Filtering the audio can improve VAD and turn detection accuracy (reducing false positives) and model performance by improving perception of the input audio.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub noise_reduction: ::std::option::Option<
        RealtimeTranscriptionSessionCreateRequestGaAudioInputNoiseReduction,
    >,
    ///Configuration for input audio transcription, defaults to off and can be set to `null` to turn off once on. Input audio transcription is not native to the model, since the model consumes audio directly. Transcription runs asynchronously through [the /audio/transcriptions endpoint](/docs/api-reference/audio/createTranscription) and should be treated as guidance of input audio content rather than precisely what the model heard. The client can optionally set the language and prompt for transcription, these offer additional guidance to the transcription service.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub transcription: ::std::option::Option<AudioTranscription>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub turn_detection: ::std::option::Option<RealtimeTurnDetection>,
}
///Configuration for input audio noise reduction. This can be set to `null` to turn off. Noise reduction filters audio added to the input audio buffer before it is sent to VAD and the model. Filtering the audio can improve VAD and turn detection accuracy (reducing false positives) and model performance by improving perception of the input audio.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct RealtimeTranscriptionSessionCreateRequestGaAudioInputNoiseReduction {
    #[serde(rename = "type", default, skip_serializing_if = "Option::is_none")]
    pub type_value: ::std::option::Option<NoiseReductionType>,
}
///Configuration for input audio noise reduction. This can be set to `null` to turn off. Noise reduction filters audio added to the input audio buffer before it is sent to VAD and the model. Filtering the audio can improve VAD and turn detection accuracy (reducing false positives) and model performance by improving perception of the input audio.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct RealtimeTranscriptionSessionCreateRequestInputAudioNoiseReduction {
    #[serde(rename = "type", default, skip_serializing_if = "Option::is_none")]
    pub type_value: ::std::option::Option<NoiseReductionType>,
}
///Configuration for turn detection. Can be set to `null` to turn off. Server VAD means that the model will detect the start and end of speech based on audio volume and respond at the end of user speech.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct RealtimeTranscriptionSessionCreateRequestTurnDetection {
    ///Amount of audio to include before the VAD detected speech (in milliseconds). Defaults to 300ms.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub prefix_padding_ms: ::std::option::Option<i32>,
    ///Duration of silence to detect speech stop (in milliseconds). Defaults to 500ms. With shorter values the model will respond more quickly, but may jump in on short pauses from the user.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub silence_duration_ms: ::std::option::Option<i32>,
    ///Activation threshold for VAD (0.0 to 1.0), this defaults to 0.5. A higher threshold will require louder audio to activate the model, and thus might perform better in noisy environments.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub threshold: ::std::option::Option<f64>,
    ///Type of turn detection. Only `server_vad` is currently supported for transcription sessions.
    #[serde(rename = "type", default, skip_serializing_if = "Option::is_none")]
    pub type_value: ::std::option::Option<String>,
}
///A new Realtime transcription session configuration. When a session is created on the server via REST API, the session object also contains an ephemeral key. Default TTL for keys is 10 minutes. This property is not present when a session is updated via the WebSocket API.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct RealtimeTranscriptionSessionCreateResponse {
    ///Ephemeral key returned by the API. Only present when the session is created on the server via REST API.
    pub client_secret: RealtimeTranscriptionSessionCreateResponseClientSecret,
    ///The format of input audio. Options are `pcm16`, `g711_ulaw`, or `g711_alaw`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub input_audio_format: ::std::option::Option<String>,
    ///Configuration of the transcription model.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub input_audio_transcription: ::std::option::Option<AudioTranscriptionResponse>,
    ///The set of modalities the model can respond with. To disable audio, set this to ["text"].
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub modalities: ::std::option::Option<OpenAiJsonValue>,
    ///Configuration for turn detection. Can be set to `null` to turn off. Server VAD means that the model will detect the start and end of speech based on audio volume and respond at the end of user speech.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub turn_detection: ::std::option::Option<
        RealtimeTranscriptionSessionCreateResponseTurnDetection,
    >,
}
///Ephemeral key returned by the API. Only present when the session is created on the server via REST API.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct RealtimeTranscriptionSessionCreateResponseClientSecret {
    ///Timestamp for when the token expires. Currently, all tokens expire after one minute.
    pub expires_at: i64,
    ///Ephemeral key usable in client environments to authenticate connections to the Realtime API. Use this in client-side environments rather than a standard API token, which should only be used server-side.
    pub value: String,
}
///A Realtime transcription session configuration object.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct RealtimeTranscriptionSessionCreateResponseGa {
    ///Configuration for input audio for the session.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub audio: ::std::option::Option<RealtimeTranscriptionSessionCreateResponseGaAudio>,
    ///Expiration timestamp for the session, in seconds since epoch.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expires_at: ::std::option::Option<i64>,
    ///Unique identifier for the session that looks like `sess_1234567890abcdef`.
    pub id: String,
    ///Additional fields to include in server outputs. - `item.input_audio_transcription.logprobs`: Include logprobs for input audio transcription.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub include: ::std::option::Option<Vec<String>>,
    ///The object type. Always `realtime.transcription_session`.
    pub object: String,
    ///The type of session. Always `transcription` for transcription sessions.
    #[serde(rename = "type")]
    pub type_value: String,
}
///Configuration for input audio for the session.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct RealtimeTranscriptionSessionCreateResponseGaAudio {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub input: ::std::option::Option<
        RealtimeTranscriptionSessionCreateResponseGaAudioInput,
    >,
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct RealtimeTranscriptionSessionCreateResponseGaAudioInput {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub format: ::std::option::Option<RealtimeAudioFormats>,
    ///Configuration for input audio noise reduction.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub noise_reduction: ::std::option::Option<
        RealtimeTranscriptionSessionCreateResponseGaAudioInputNoiseReduction,
    >,
    ///Configuration of the transcription model.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub transcription: ::std::option::Option<AudioTranscriptionResponse>,
    ///Configuration for turn detection. For `gpt-realtime-whisper`, this must be `null`; VAD is not supported.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub turn_detection: ::std::option::Option<
        RealtimeTranscriptionSessionCreateResponseGaAudioInputTurnDetection,
    >,
}
///Configuration for input audio noise reduction.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct RealtimeTranscriptionSessionCreateResponseGaAudioInputNoiseReduction {
    #[serde(rename = "type", default, skip_serializing_if = "Option::is_none")]
    pub type_value: ::std::option::Option<NoiseReductionType>,
}
///Configuration for turn detection. Can be set to `null` to turn off. Server VAD means that the model will detect the start and end of speech based on audio volume and respond at the end of user speech. For `gpt-realtime-whisper`, this must be `null`; VAD is not supported.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct RealtimeTranscriptionSessionCreateResponseGaAudioInputTurnDetection {
    ///Amount of audio to include before the VAD detected speech (in milliseconds). Defaults to 300ms.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub prefix_padding_ms: ::std::option::Option<i32>,
    ///Duration of silence to detect speech stop (in milliseconds). Defaults to 500ms. With shorter values the model will respond more quickly, but may jump in on short pauses from the user.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub silence_duration_ms: ::std::option::Option<i32>,
    ///Activation threshold for VAD (0.0 to 1.0), this defaults to 0.5. A higher threshold will require louder audio to activate the model, and thus might perform better in noisy environments.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub threshold: ::std::option::Option<f64>,
    ///Type of turn detection, only `server_vad` is currently supported.
    #[serde(rename = "type", default, skip_serializing_if = "Option::is_none")]
    pub type_value: ::std::option::Option<String>,
}
///Configuration for turn detection. Can be set to `null` to turn off. Server VAD means that the model will detect the start and end of speech based on audio volume and respond at the end of user speech.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct RealtimeTranscriptionSessionCreateResponseTurnDetection {
    ///Amount of audio to include before the VAD detected speech (in milliseconds). Defaults to 300ms.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub prefix_padding_ms: ::std::option::Option<i32>,
    ///Duration of silence to detect speech stop (in milliseconds). Defaults to 500ms. With shorter values the model will respond more quickly, but may jump in on short pauses from the user.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub silence_duration_ms: ::std::option::Option<i32>,
    ///Activation threshold for VAD (0.0 to 1.0), this defaults to 0.5. A higher threshold will require louder audio to activate the model, and thus might perform better in noisy environments.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub threshold: ::std::option::Option<f64>,
    ///Type of turn detection, only `server_vad` is currently supported.
    #[serde(rename = "type", default, skip_serializing_if = "Option::is_none")]
    pub type_value: ::std::option::Option<String>,
}
///A Realtime translation client event.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
#[serde(untagged)]
pub enum RealtimeTranslationClientEvent {
    RealtimeTranslationClientEventSessionUpdate(
        RealtimeTranslationClientEventSessionUpdate,
    ),
    RealtimeTranslationClientEventInputAudioBufferAppend(
        RealtimeTranslationClientEventInputAudioBufferAppend,
    ),
    RealtimeTranslationClientEventSessionClose(
        RealtimeTranslationClientEventSessionClose,
    ),
}
///Send this event to append audio bytes to the translation session input audio buffer. WebSocket translation sessions accept base64-encoded 24 kHz PCM16 mono little-endian raw audio bytes. Unsupported websocket audio formats return a validation error because lower-quality audio materially degrades translation quality. Translation consumes 200 ms engine frames. For best realtime behavior, append audio in 200 ms chunks. If a chunk is shorter, the server buffers it until it has enough audio for one frame. If a chunk is longer, the server splits it into 200 ms frames and enqueues them back-to-back. Keep appending silence while the session is active. If a client stops sending audio and later resumes, model time treats the resumed audio as contiguous with the previous audio rather than as a real-world pause.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct RealtimeTranslationClientEventInputAudioBufferAppend {
    ///Base64-encoded 24 kHz PCM16 mono audio bytes.
    pub audio: String,
    ///Optional client-generated ID used to identify this event.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub event_id: ::std::option::Option<String>,
    ///The event type, must be `session.input_audio_buffer.append`.
    #[serde(rename = "type")]
    pub type_value: String,
}
///Gracefully close the realtime translation session. The server flushes pending input audio and emits any remaining translated output before closing the session.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct RealtimeTranslationClientEventSessionClose {
    ///Optional client-generated ID used to identify this event.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub event_id: ::std::option::Option<String>,
    ///The event type, must be `session.close`.
    #[serde(rename = "type")]
    pub type_value: String,
}
///Send this event to update the translation session configuration. Translation sessions support updates to `audio.output.language`, `audio.input.transcription`, and `audio.input.noise_reduction`.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct RealtimeTranslationClientEventSessionUpdate {
    ///Optional client-generated ID used to identify this event.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub event_id: ::std::option::Option<String>,
    ///Translation session fields to update. The session `type` and `model` are set at creation and cannot be changed with `session.update`.
    pub session: RealtimeTranslationSessionUpdateRequest,
    ///The event type, must be `session.update`.
    #[serde(rename = "type")]
    pub type_value: String,
}
///Create a translation session and client secret for the Realtime API.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct RealtimeTranslationClientSecretCreateRequest {
    ///Configuration for the client secret expiration. Expiration refers to the time after which a client secret will no longer be valid for creating sessions. The session itself may continue after that time once started. A secret can be used to create multiple sessions until it expires.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expires_after: ::std::option::Option<
        RealtimeTranslationClientSecretCreateRequestExpiresAfter,
    >,
    pub session: RealtimeTranslationSessionCreateRequest,
}
///Configuration for the client secret expiration. Expiration refers to the time after which a client secret will no longer be valid for creating sessions. The session itself may continue after that time once started. A secret can be used to create multiple sessions until it expires.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct RealtimeTranslationClientSecretCreateRequestExpiresAfter {
    ///The anchor point for the client secret expiration, meaning that `seconds` will be added to the `created_at` time of the client secret to produce an expiration timestamp. Only `created_at` is currently supported.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub anchor: ::std::option::Option<String>,
    ///The number of seconds from the anchor point to the expiration. Select a value between `10` and `7200` (2 hours). This default to 600 seconds (10 minutes) if not specified.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub seconds: ::std::option::Option<i64>,
}
///Response from creating a translation session and client secret for the Realtime API.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct RealtimeTranslationClientSecretCreateResponse {
    ///Expiration timestamp for the client secret, in seconds since epoch.
    pub expires_at: i64,
    pub session: RealtimeTranslationSession,
    ///The generated client secret value.
    pub value: String,
}
///A Realtime translation server event.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
#[serde(untagged)]
pub enum RealtimeTranslationServerEvent {
    RealtimeServerEventError(RealtimeServerEventError),
    RealtimeTranslationServerEventSessionCreated(
        RealtimeTranslationServerEventSessionCreated,
    ),
    RealtimeTranslationServerEventSessionUpdated(
        RealtimeTranslationServerEventSessionUpdated,
    ),
    RealtimeTranslationServerEventSessionClosed(
        RealtimeTranslationServerEventSessionClosed,
    ),
    RealtimeTranslationServerEventSessionInputTranscriptDelta(
        RealtimeTranslationServerEventSessionInputTranscriptDelta,
    ),
    RealtimeTranslationServerEventSessionOutputTranscriptDelta(
        RealtimeTranslationServerEventSessionOutputTranscriptDelta,
    ),
    RealtimeTranslationServerEventSessionOutputAudioDelta(
        RealtimeTranslationServerEventSessionOutputAudioDelta,
    ),
}
///Returned when a realtime translation session is closed.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct RealtimeTranslationServerEventSessionClosed {
    ///The unique ID of the server event.
    pub event_id: String,
    ///The event type, must be `session.closed`.
    #[serde(rename = "type")]
    pub type_value: String,
}
///Returned when a translation session is created. Emitted automatically when a new connection is established as the first server event. This event contains the default translation session configuration.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct RealtimeTranslationServerEventSessionCreated {
    ///The unique ID of the server event.
    pub event_id: String,
    ///The translation session configuration.
    pub session: RealtimeTranslationSession,
    ///The event type, must be `session.created`.
    #[serde(rename = "type")]
    pub type_value: String,
}
///Returned when optional source-language transcript text is available. This event is emitted only when `audio.input.transcription` is configured. Transcript deltas are append-only text fragments. Clients should not insert unconditional spaces between deltas.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct RealtimeTranslationServerEventSessionInputTranscriptDelta {
    ///Append-only source-language transcript text.
    pub delta: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub elapsed_ms: ::std::option::Option<i32>,
    ///The unique ID of the server event.
    pub event_id: String,
    ///The event type, must be `session.input_transcript.delta`.
    #[serde(rename = "type")]
    pub type_value: String,
}
///Returned when translated output audio is available. Output audio deltas are 200 ms frames of PCM16 audio.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct RealtimeTranslationServerEventSessionOutputAudioDelta {
    ///Number of audio channels.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub channels: ::std::option::Option<i32>,
    ///Base64-encoded translated audio data.
    pub delta: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub elapsed_ms: ::std::option::Option<i32>,
    ///The unique ID of the server event.
    pub event_id: String,
    ///Audio encoding for `delta`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub format: ::std::option::Option<String>,
    ///Sample rate of the audio delta.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sample_rate: ::std::option::Option<i32>,
    ///The event type, must be `session.output_audio.delta`.
    #[serde(rename = "type")]
    pub type_value: String,
}
///Returned when translated transcript text is available. Transcript deltas are append-only text fragments. Clients should not insert unconditional spaces between deltas.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct RealtimeTranslationServerEventSessionOutputTranscriptDelta {
    ///Append-only transcript text for the translated output audio.
    pub delta: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub elapsed_ms: ::std::option::Option<i32>,
    ///The unique ID of the server event.
    pub event_id: String,
    ///The event type, must be `session.output_transcript.delta`.
    #[serde(rename = "type")]
    pub type_value: String,
}
///Returned when a translation session is updated with a `session.update` event, unless there is an error.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct RealtimeTranslationServerEventSessionUpdated {
    ///The unique ID of the server event.
    pub event_id: String,
    ///The translation session configuration.
    pub session: RealtimeTranslationSession,
    ///The event type, must be `session.updated`.
    #[serde(rename = "type")]
    pub type_value: String,
}
///A Realtime translation session. Translation sessions continuously translate input audio into the configured output language.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct RealtimeTranslationSession {
    ///Configuration for translation input and output audio.
    pub audio: RealtimeTranslationSessionAudio,
    ///Expiration timestamp for the session, in seconds since epoch.
    pub expires_at: i64,
    ///Unique identifier for the session that looks like `sess_1234567890abcdef`.
    pub id: String,
    ///The Realtime translation model used for this session. This field is set at session creation and cannot be changed with `session.update`.
    pub model: String,
    ///The session type. Always `translation` for Realtime translation sessions.
    #[serde(rename = "type")]
    pub type_value: String,
}
///Configuration for translation input and output audio.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct RealtimeTranslationSessionAudio {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub input: ::std::option::Option<RealtimeTranslationSessionAudioInput>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub output: ::std::option::Option<RealtimeTranslationSessionAudioOutput>,
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct RealtimeTranslationSessionAudioInput {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub noise_reduction: ::std::option::Option<
        RealtimeTranslationSessionAudioInputNoiseReduction,
    >,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub transcription: ::std::option::Option<
        RealtimeTranslationSessionAudioInputTranscription,
    >,
}
///Optional input noise reduction.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct RealtimeTranslationSessionAudioInputNoiseReduction {
    #[serde(rename = "type")]
    pub type_value: NoiseReductionType,
}
///Optional source-language transcription. When configured, the server emits `session.input_transcript.delta` events. Translation itself still runs from the input audio stream.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct RealtimeTranslationSessionAudioInputTranscription {
    ///The transcription model used for source transcript deltas.
    pub model: String,
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct RealtimeTranslationSessionAudioOutput {
    ///Target language for translated output audio and transcript deltas.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub language: ::std::option::Option<String>,
}
///Realtime translation session configuration. Translation sessions stream source audio in and translated audio plus transcript deltas out continuously.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct RealtimeTranslationSessionCreateRequest {
    ///Configuration for translation input and output audio.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub audio: ::std::option::Option<RealtimeTranslationSessionCreateRequestAudio>,
    ///The Realtime translation model used for this session.
    pub model: String,
}
///Configuration for translation input and output audio.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct RealtimeTranslationSessionCreateRequestAudio {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub input: ::std::option::Option<RealtimeTranslationSessionCreateRequestAudioInput>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub output: ::std::option::Option<
        RealtimeTranslationSessionCreateRequestAudioOutput,
    >,
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct RealtimeTranslationSessionCreateRequestAudioInput {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub noise_reduction: ::std::option::Option<
        RealtimeTranslationSessionCreateRequestAudioInputNoiseReduction,
    >,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub transcription: ::std::option::Option<
        RealtimeTranslationSessionCreateRequestAudioInputTranscription,
    >,
}
///Optional input noise reduction. Set to `null` to disable it.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct RealtimeTranslationSessionCreateRequestAudioInputNoiseReduction {
    #[serde(rename = "type")]
    pub type_value: NoiseReductionType,
}
///Optional source-language transcription. When configured, the server emits `session.input_transcript.delta` events. Translation itself still runs from the input audio stream.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct RealtimeTranslationSessionCreateRequestAudioInputTranscription {
    ///The transcription model to use for source transcript deltas.
    pub model: String,
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct RealtimeTranslationSessionCreateRequestAudioOutput {
    ///Target language for translated output audio and transcript deltas.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub language: ::std::option::Option<String>,
}
///Realtime translation session fields that can be updated with `session.update`.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct RealtimeTranslationSessionUpdateRequest {
    ///Configuration for translation input and output audio.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub audio: ::std::option::Option<RealtimeTranslationSessionUpdateRequestAudio>,
}
///Configuration for translation input and output audio.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct RealtimeTranslationSessionUpdateRequestAudio {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub input: ::std::option::Option<RealtimeTranslationSessionUpdateRequestAudioInput>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub output: ::std::option::Option<
        RealtimeTranslationSessionUpdateRequestAudioOutput,
    >,
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct RealtimeTranslationSessionUpdateRequestAudioInput {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub noise_reduction: ::std::option::Option<
        RealtimeTranslationSessionUpdateRequestAudioInputNoiseReduction,
    >,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub transcription: ::std::option::Option<
        RealtimeTranslationSessionUpdateRequestAudioInputTranscription,
    >,
}
///Optional input noise reduction. Set to `null` to disable it.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct RealtimeTranslationSessionUpdateRequestAudioInputNoiseReduction {
    #[serde(rename = "type")]
    pub type_value: NoiseReductionType,
}
///Optional source-language transcription. When configured, the server emits `session.input_transcript.delta` events. Translation itself still runs from the input audio stream.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct RealtimeTranslationSessionUpdateRequestAudioInputTranscription {
    ///The transcription model to use for source transcript deltas.
    pub model: String,
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct RealtimeTranslationSessionUpdateRequestAudioOutput {
    ///Target language for translated output audio and transcript deltas.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub language: ::std::option::Option<String>,
}
///When the number of tokens in a conversation exceeds the model's input token limit, the conversation be truncated, meaning messages (starting from the oldest) will not be included in the model's context. A 32k context model with 4,096 max output tokens can only include 28,224 tokens in the context before truncation occurs. Clients can configure truncation behavior to truncate with a lower max token limit, which is an effective way to control token usage and cost. Truncation will reduce the number of cached tokens on the next turn (busting the cache), since messages are dropped from the beginning of the context. However, clients can also configure truncation to retain messages up to a fraction of the maximum context size, which will reduce the need for future truncations and thus improve the cache rate. Truncation can be disabled entirely, which means the server will never truncate but would instead return an error if the conversation exceeds the model's input token limit.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
#[serde(untagged)]
pub enum RealtimeTruncation {
    String(String),
    RetentionRatioTruncation(RealtimeTruncationRetentionRatioTruncation),
}
///Retain a fraction of the conversation tokens when the conversation exceeds the input token limit. This allows you to amortize truncations across multiple turns, which can help improve cached token usage.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct RealtimeTruncationRetentionRatioTruncation {
    ///Fraction of post-instruction conversation tokens to retain (`0.0` - `1.0`) when the conversation exceeds the input token limit. Setting this to `0.8` means that messages will be dropped until 80% of the maximum allowed tokens are used. This helps reduce the frequency of truncations and improve cache rates.
    pub retention_ratio: f64,
    ///Optional custom token limits for this truncation strategy. If not provided, the model's default token limits will be used.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub token_limits: ::std::option::Option<
        RealtimeTruncationRetentionRatioTruncationTokenLimits,
    >,
    ///Use retention ratio truncation.
    #[serde(rename = "type")]
    pub type_value: String,
}
///Optional custom token limits for this truncation strategy. If not provided, the model's default token limits will be used.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct RealtimeTruncationRetentionRatioTruncationTokenLimits {
    ///Maximum tokens allowed in the conversation after instructions (which including tool definitions). For example, setting this to 5,000 would mean that truncation would occur when the conversation exceeds 5,000 tokens after instructions. This cannot be higher than the model's context window size minus the maximum output tokens.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub post_instructions: ::std::option::Option<i32>,
}
pub type RealtimeTurnDetection = RealtimeTurnDetection4;
///Server-side voice activity detection (VAD) which flips on when user speech is detected and off after a period of silence.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct RealtimeTurnDetection2 {
    ///Whether or not to automatically generate a response when a VAD stop event occurs. If `interrupt_response` is set to `false` this may fail to create a response if the model is already responding. If both `create_response` and `interrupt_response` are set to `false`, the model will never respond automatically but VAD events will still be emitted.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub create_response: ::std::option::Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub idle_timeout_ms: ::std::option::Option<i32>,
    ///Whether or not to automatically interrupt (cancel) any ongoing response with output to the default conversation (i.e. `conversation` of `auto`) when a VAD start event occurs. If `true` then the response will be cancelled, otherwise it will continue until complete. If both `create_response` and `interrupt_response` are set to `false`, the model will never respond automatically but VAD events will still be emitted.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub interrupt_response: ::std::option::Option<bool>,
    ///Used only for `server_vad` mode. Amount of audio to include before the VAD detected speech (in milliseconds). Defaults to 300ms.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub prefix_padding_ms: ::std::option::Option<i32>,
    ///Used only for `server_vad` mode. Duration of silence to detect speech stop (in milliseconds). Defaults to 500ms. With shorter values the model will respond more quickly, but may jump in on short pauses from the user.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub silence_duration_ms: ::std::option::Option<i32>,
    ///Used only for `server_vad` mode. Activation threshold for VAD (0.0 to 1.0), this defaults to 0.5. A higher threshold will require louder audio to activate the model, and thus might perform better in noisy environments.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub threshold: ::std::option::Option<f64>,
    ///Type of turn detection, `server_vad` to turn on simple Server VAD.
    #[serde(rename = "type")]
    pub type_value: String,
}
///Server-side semantic turn detection which uses a model to determine when the user has finished speaking.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct RealtimeTurnDetection3 {
    ///Whether or not to automatically generate a response when a VAD stop event occurs.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub create_response: ::std::option::Option<bool>,
    ///Used only for `semantic_vad` mode. The eagerness of the model to respond. `low` will wait longer for the user to continue speaking, `high` will respond more quickly. `auto` is the default and is equivalent to `medium`. `low`, `medium`, and `high` have max timeouts of 8s, 4s, and 2s respectively.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub eagerness: ::std::option::Option<String>,
    ///Whether or not to automatically interrupt any ongoing response with output to the default conversation (i.e. `conversation` of `auto`) when a VAD start event occurs.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub interrupt_response: ::std::option::Option<bool>,
    ///Type of turn detection, `semantic_vad` to turn on Semantic VAD.
    #[serde(rename = "type")]
    pub type_value: String,
}
///Configuration for turn detection, ether Server VAD or Semantic VAD. This can be set to `null` to turn off, in which case the client must manually trigger model response. Server VAD means that the model will detect the start and end of speech based on audio volume and respond at the end of user speech. Semantic VAD is more advanced and uses a turn detection model (in conjunction with VAD) to semantically estimate whether the user has finished speaking, then dynamically sets a timeout based on this probability. For example, if user audio trails off with "uhhm", the model will score a low probability of turn end and wait longer for the user to continue speaking. This can be useful for more natural conversations, but may have a higher latency. For `gpt-realtime-whisper` transcription sessions, turn detection must be set to `null`; VAD is not supported.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
#[serde(untagged)]
pub enum RealtimeTurnDetection4 {
    ServerVad(RealtimeTurnDetection4ServerVad),
    SemanticVad(RealtimeTurnDetection4SemanticVad),
}
///Server-side semantic turn detection which uses a model to determine when the user has finished speaking.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct RealtimeTurnDetection4SemanticVad {
    ///Whether or not to automatically generate a response when a VAD stop event occurs.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub create_response: ::std::option::Option<bool>,
    ///Used only for `semantic_vad` mode. The eagerness of the model to respond. `low` will wait longer for the user to continue speaking, `high` will respond more quickly. `auto` is the default and is equivalent to `medium`. `low`, `medium`, and `high` have max timeouts of 8s, 4s, and 2s respectively.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub eagerness: ::std::option::Option<String>,
    ///Whether or not to automatically interrupt any ongoing response with output to the default conversation (i.e. `conversation` of `auto`) when a VAD start event occurs.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub interrupt_response: ::std::option::Option<bool>,
    ///Type of turn detection, `semantic_vad` to turn on Semantic VAD.
    #[serde(rename = "type")]
    pub type_value: String,
}
///Server-side voice activity detection (VAD) which flips on when user speech is detected and off after a period of silence.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct RealtimeTurnDetection4ServerVad {
    ///Whether or not to automatically generate a response when a VAD stop event occurs. If `interrupt_response` is set to `false` this may fail to create a response if the model is already responding. If both `create_response` and `interrupt_response` are set to `false`, the model will never respond automatically but VAD events will still be emitted.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub create_response: ::std::option::Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub idle_timeout_ms: ::std::option::Option<i32>,
    ///Whether or not to automatically interrupt (cancel) any ongoing response with output to the default conversation (i.e. `conversation` of `auto`) when a VAD start event occurs. If `true` then the response will be cancelled, otherwise it will continue until complete. If both `create_response` and `interrupt_response` are set to `false`, the model will never respond automatically but VAD events will still be emitted.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub interrupt_response: ::std::option::Option<bool>,
    ///Used only for `server_vad` mode. Amount of audio to include before the VAD detected speech (in milliseconds). Defaults to 300ms.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub prefix_padding_ms: ::std::option::Option<i32>,
    ///Used only for `server_vad` mode. Duration of silence to detect speech stop (in milliseconds). Defaults to 500ms. With shorter values the model will respond more quickly, but may jump in on short pauses from the user.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub silence_duration_ms: ::std::option::Option<i32>,
    ///Used only for `server_vad` mode. Activation threshold for VAD (0.0 to 1.0), this defaults to 0.5. A higher threshold will require louder audio to activate the model, and thus might perform better in noisy environments.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub threshold: ::std::option::Option<f64>,
    ///Type of turn detection, `server_vad` to turn on simple Server VAD.
    #[serde(rename = "type")]
    pub type_value: String,
}
///**gpt-5 and o-series models only** Configuration options for [reasoning models](https://platform.openai.com/docs/guides/reasoning).
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct Reasoning {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub effort: ::std::option::Option<ReasoningEffort>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub generate_summary: ::std::option::Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub summary: ::std::option::Option<String>,
}
pub type ReasoningEffort = String;
///A description of the chain of thought used by a reasoning model while generating a response. Be sure to include these items in your `input` to the Responses API for subsequent turns of a conversation if you are manually [managing context](/docs/guides/conversation-state).
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct ReasoningItem {
    ///Reasoning text content.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub content: ::std::option::Option<Vec<ReasoningTextContent>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub encrypted_content: ::std::option::Option<String>,
    ///The unique identifier of the reasoning content.
    pub id: String,
    ///The status of the item. One of `in_progress`, `completed`, or `incomplete`. Populated when items are returned via API.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub status: ::std::option::Option<String>,
    ///Reasoning summary content.
    pub summary: Vec<SummaryTextContent>,
    ///The type of the object. Always `reasoning`.
    #[serde(rename = "type")]
    pub type_value: String,
}
///Reasoning text from the model.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct ReasoningTextContent {
    ///The reasoning text from the model.
    pub text: String,
    ///The type of the reasoning text. Always `reasoning_text`.
    #[serde(rename = "type")]
    pub type_value: String,
}
///A refusal from the model.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct RefusalContent {
    ///The refusal explanation from the model.
    pub refusal: String,
    ///The type of the refusal. Always `refusal`.
    #[serde(rename = "type")]
    pub type_value: String,
}
///The response object
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct Response {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub background: ::std::option::Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub completed_at: ::std::option::Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub conversation: ::std::option::Option<Conversation2>,
    ///Unix timestamp (in seconds) of when this Response was created.
    pub created_at: f64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: ::std::option::Option<ResponseError>,
    ///Unique identifier for this Response.
    pub id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub incomplete_details: ::std::option::Option<ResponseIncompleteDetails>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub instructions: ::std::option::Option<ResponseInstructions>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_output_tokens: ::std::option::Option<i32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_tool_calls: ::std::option::Option<i32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub metadata: ::std::option::Option<Metadata>,
    ///Model ID used to generate the response, like `gpt-4o` or `o3`. OpenAI offers a wide range of models with different capabilities, performance characteristics, and price points. Refer to the [model guide](/docs/models) to browse and compare available models.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model: ::std::option::Option<ModelIdsResponses>,
    ///The object type of this resource - always set to `response`.
    pub object: String,
    ///An array of content items generated by the model. - The length and order of items in the `output` array is dependent on the model's response. - Rather than accessing the first item in the `output` array and assuming it's an `assistant` message with the content generated by the model, you might consider using the `output_text` property where supported in SDKs.
    pub output: Vec<OutputItem>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub output_text: ::std::option::Option<String>,
    ///Whether to allow the model to run tool calls in parallel.
    pub parallel_tool_calls: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub previous_response_id: ::std::option::Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub prompt: ::std::option::Option<Prompt>,
    ///Used by OpenAI to cache responses for similar requests to optimize your cache hit rates. Replaces the `user` field. [Learn more](/docs/guides/prompt-caching).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub prompt_cache_key: ::std::option::Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub prompt_cache_retention: ::std::option::Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reasoning: ::std::option::Option<Reasoning>,
    ///A stable identifier used to help detect users of your application that may be violating OpenAI's usage policies. The IDs should be a string that uniquely identifies each user, with a maximum length of 64 characters. We recommend hashing their username or email address, in order to avoid sending us any identifying information. [Learn more](/docs/guides/safety-best-practices#safety-identifiers).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub safety_identifier: ::std::option::Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub service_tier: ::std::option::Option<ServiceTier>,
    ///The status of the response generation. One of `completed`, `failed`, `in_progress`, `cancelled`, `queued`, or `incomplete`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub status: ::std::option::Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub temperature: ::std::option::Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub text: ::std::option::Option<ResponseTextParam>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tool_choice: ::std::option::Option<ToolChoiceParam>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tools: ::std::option::Option<ToolsArray>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub top_logprobs: ::std::option::Option<i32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub top_p: ::std::option::Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub truncation: ::std::option::Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub usage: ::std::option::Option<ResponseUsage>,
    ///This field is being replaced by `safety_identifier` and `prompt_cache_key`. Use `prompt_cache_key` instead to maintain caching optimizations. A stable identifier for your end-users. Used to boost cache hit rates by better bucketing similar requests and to help OpenAI detect and prevent abuse. [Learn more](/docs/guides/safety-best-practices#safety-identifiers).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub user: ::std::option::Option<String>,
}
///Emitted when there is a partial audio response.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct ResponseAudioDeltaEvent {
    ///A chunk of Base64 encoded response audio bytes.
    pub delta: String,
    ///A sequence number for this chunk of the stream response.
    pub sequence_number: i32,
    ///The type of the event. Always `response.audio.delta`.
    #[serde(rename = "type")]
    pub type_value: String,
}
///Emitted when the audio response is complete.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct ResponseAudioDoneEvent {
    ///The sequence number of the delta.
    pub sequence_number: i32,
    ///The type of the event. Always `response.audio.done`.
    #[serde(rename = "type")]
    pub type_value: String,
}
///Emitted when there is a partial transcript of audio.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct ResponseAudioTranscriptDeltaEvent {
    ///The partial transcript of the audio response.
    pub delta: String,
    ///The sequence number of this event.
    pub sequence_number: i32,
    ///The type of the event. Always `response.audio.transcript.delta`.
    #[serde(rename = "type")]
    pub type_value: String,
}
///Emitted when the full audio transcript is completed.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct ResponseAudioTranscriptDoneEvent {
    ///The sequence number of this event.
    pub sequence_number: i32,
    ///The type of the event. Always `response.audio.transcript.done`.
    #[serde(rename = "type")]
    pub type_value: String,
}
///Emitted when a partial code snippet is streamed by the code interpreter.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct ResponseCodeInterpreterCallCodeDeltaEvent {
    ///The partial code snippet being streamed by the code interpreter.
    pub delta: String,
    ///The unique identifier of the code interpreter tool call item.
    pub item_id: String,
    ///The index of the output item in the response for which the code is being streamed.
    pub output_index: i32,
    ///The sequence number of this event, used to order streaming events.
    pub sequence_number: i32,
    ///The type of the event. Always `response.code_interpreter_call_code.delta`.
    #[serde(rename = "type")]
    pub type_value: String,
}
///Emitted when the code snippet is finalized by the code interpreter.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct ResponseCodeInterpreterCallCodeDoneEvent {
    ///The final code snippet output by the code interpreter.
    pub code: String,
    ///The unique identifier of the code interpreter tool call item.
    pub item_id: String,
    ///The index of the output item in the response for which the code is finalized.
    pub output_index: i32,
    ///The sequence number of this event, used to order streaming events.
    pub sequence_number: i32,
    ///The type of the event. Always `response.code_interpreter_call_code.done`.
    #[serde(rename = "type")]
    pub type_value: String,
}
///Emitted when the code interpreter call is completed.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct ResponseCodeInterpreterCallCompletedEvent {
    ///The unique identifier of the code interpreter tool call item.
    pub item_id: String,
    ///The index of the output item in the response for which the code interpreter call is completed.
    pub output_index: i32,
    ///The sequence number of this event, used to order streaming events.
    pub sequence_number: i32,
    ///The type of the event. Always `response.code_interpreter_call.completed`.
    #[serde(rename = "type")]
    pub type_value: String,
}
///Emitted when a code interpreter call is in progress.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct ResponseCodeInterpreterCallInProgressEvent {
    ///The unique identifier of the code interpreter tool call item.
    pub item_id: String,
    ///The index of the output item in the response for which the code interpreter call is in progress.
    pub output_index: i32,
    ///The sequence number of this event, used to order streaming events.
    pub sequence_number: i32,
    ///The type of the event. Always `response.code_interpreter_call.in_progress`.
    #[serde(rename = "type")]
    pub type_value: String,
}
///Emitted when the code interpreter is actively interpreting the code snippet.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct ResponseCodeInterpreterCallInterpretingEvent {
    ///The unique identifier of the code interpreter tool call item.
    pub item_id: String,
    ///The index of the output item in the response for which the code interpreter is interpreting code.
    pub output_index: i32,
    ///The sequence number of this event, used to order streaming events.
    pub sequence_number: i32,
    ///The type of the event. Always `response.code_interpreter_call.interpreting`.
    #[serde(rename = "type")]
    pub type_value: String,
}
///Emitted when the model response is complete.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct ResponseCompletedEvent {
    ///Properties of the completed response.
    pub response: Response,
    ///The sequence number for this event.
    pub sequence_number: i32,
    ///The type of the event. Always `response.completed`.
    #[serde(rename = "type")]
    pub type_value: String,
}
///Emitted when a new content part is added.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct ResponseContentPartAddedEvent {
    ///The index of the content part that was added.
    pub content_index: i32,
    ///The ID of the output item that the content part was added to.
    pub item_id: String,
    ///The index of the output item that the content part was added to.
    pub output_index: i32,
    ///The content part that was added.
    pub part: OutputContent,
    ///The sequence number of this event.
    pub sequence_number: i32,
    ///The type of the event. Always `response.content_part.added`.
    #[serde(rename = "type")]
    pub type_value: String,
}
///Emitted when a content part is done.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct ResponseContentPartDoneEvent {
    ///The index of the content part that is done.
    pub content_index: i32,
    ///The ID of the output item that the content part was added to.
    pub item_id: String,
    ///The index of the output item that the content part was added to.
    pub output_index: i32,
    ///The content part that is done.
    pub part: OutputContent,
    ///The sequence number of this event.
    pub sequence_number: i32,
    ///The type of the event. Always `response.content_part.done`.
    #[serde(rename = "type")]
    pub type_value: String,
}
///An event that is emitted when a response is created.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct ResponseCreatedEvent {
    ///The response that was created.
    pub response: Response,
    ///The sequence number for this event.
    pub sequence_number: i32,
    ///The type of the event. Always `response.created`.
    #[serde(rename = "type")]
    pub type_value: String,
}
///Event representing a delta (partial update) to the input of a custom tool call.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct ResponseCustomToolCallInputDeltaEvent {
    ///The incremental input data (delta) for the custom tool call.
    pub delta: String,
    ///Unique identifier for the API item associated with this event.
    pub item_id: String,
    ///The index of the output this delta applies to.
    pub output_index: i32,
    ///The sequence number of this event.
    pub sequence_number: i32,
    ///The event type identifier.
    #[serde(rename = "type")]
    pub type_value: String,
}
///Event indicating that input for a custom tool call is complete.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct ResponseCustomToolCallInputDoneEvent {
    ///The complete input data for the custom tool call.
    pub input: String,
    ///Unique identifier for the API item associated with this event.
    pub item_id: String,
    ///The index of the output this event applies to.
    pub output_index: i32,
    ///The sequence number of this event.
    pub sequence_number: i32,
    ///The event type identifier.
    #[serde(rename = "type")]
    pub type_value: String,
}
pub type ResponseError = ResponseError2;
///An error object returned when the model fails to generate a Response.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct ResponseError2 {
    pub code: ResponseErrorCode,
    ///A human-readable description of the error.
    pub message: String,
}
///The error code for the response.
pub type ResponseErrorCode = String;
///Emitted when an error occurs.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct ResponseErrorEvent {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub code: ::std::option::Option<String>,
    ///The error message.
    pub message: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub param: ::std::option::Option<String>,
    ///The sequence number of this event.
    pub sequence_number: i32,
    ///The type of the event. Always `error`.
    #[serde(rename = "type")]
    pub type_value: String,
}
///An event that is emitted when a response fails.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct ResponseFailedEvent {
    ///The response that failed.
    pub response: Response,
    ///The sequence number of this event.
    pub sequence_number: i32,
    ///The type of the event. Always `response.failed`.
    #[serde(rename = "type")]
    pub type_value: String,
}
///Emitted when a file search call is completed (results found).
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct ResponseFileSearchCallCompletedEvent {
    ///The ID of the output item that the file search call is initiated.
    pub item_id: String,
    ///The index of the output item that the file search call is initiated.
    pub output_index: i32,
    ///The sequence number of this event.
    pub sequence_number: i32,
    ///The type of the event. Always `response.file_search_call.completed`.
    #[serde(rename = "type")]
    pub type_value: String,
}
///Emitted when a file search call is initiated.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct ResponseFileSearchCallInProgressEvent {
    ///The ID of the output item that the file search call is initiated.
    pub item_id: String,
    ///The index of the output item that the file search call is initiated.
    pub output_index: i32,
    ///The sequence number of this event.
    pub sequence_number: i32,
    ///The type of the event. Always `response.file_search_call.in_progress`.
    #[serde(rename = "type")]
    pub type_value: String,
}
///Emitted when a file search is currently searching.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct ResponseFileSearchCallSearchingEvent {
    ///The ID of the output item that the file search call is initiated.
    pub item_id: String,
    ///The index of the output item that the file search call is searching.
    pub output_index: i32,
    ///The sequence number of this event.
    pub sequence_number: i32,
    ///The type of the event. Always `response.file_search_call.searching`.
    #[serde(rename = "type")]
    pub type_value: String,
}
///JSON object response format. An older method of generating JSON responses. Using `json_schema` is recommended for models that support it. Note that the model will not generate JSON without a system or user message instructing it to do so.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct ResponseFormatJsonObject {
    ///The type of response format being defined. Always `json_object`.
    #[serde(rename = "type")]
    pub type_value: String,
}
///JSON Schema response format. Used to generate structured JSON responses. Learn more about [Structured Outputs](/docs/guides/structured-outputs).
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct ResponseFormatJsonSchema {
    ///Structured Outputs configuration options, including a JSON Schema.
    pub json_schema: ResponseFormatJsonSchemaJsonSchema,
    ///The type of response format being defined. Always `json_schema`.
    #[serde(rename = "type")]
    pub type_value: String,
}
///Structured Outputs configuration options, including a JSON Schema.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct ResponseFormatJsonSchemaJsonSchema {
    ///A description of what the response format is for, used by the model to determine how to respond in the format.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: ::std::option::Option<String>,
    ///The name of the response format. Must be a-z, A-Z, 0-9, or contain underscores and dashes, with a maximum length of 64.
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub schema: ::std::option::Option<ResponseFormatJsonSchemaSchema>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub strict: ::std::option::Option<bool>,
}
///The schema for the response format, described as a JSON Schema object. Learn how to build JSON schemas [here](https://json-schema.org/).
pub type ResponseFormatJsonSchemaSchema = OpenAiJsonValue;
///Default response format. Used to generate text responses.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct ResponseFormatText {
    ///The type of response format being defined. Always `text`.
    #[serde(rename = "type")]
    pub type_value: String,
}
///A custom grammar for the model to follow when generating text. Learn more in the [custom grammars guide](/docs/guides/custom-grammars).
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct ResponseFormatTextGrammar {
    ///The custom grammar for the model to follow.
    pub grammar: String,
    ///The type of response format being defined. Always `grammar`.
    #[serde(rename = "type")]
    pub type_value: String,
}
///Configure the model to generate valid Python code. See the [custom grammars guide](/docs/guides/custom-grammars) for more details.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct ResponseFormatTextPython {
    ///The type of response format being defined. Always `python`.
    #[serde(rename = "type")]
    pub type_value: String,
}
///Emitted when there is a partial function-call arguments delta.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct ResponseFunctionCallArgumentsDeltaEvent {
    ///The function-call arguments delta that is added.
    pub delta: String,
    ///The ID of the output item that the function-call arguments delta is added to.
    pub item_id: String,
    ///The index of the output item that the function-call arguments delta is added to.
    pub output_index: i32,
    ///The sequence number of this event.
    pub sequence_number: i32,
    ///The type of the event. Always `response.function_call_arguments.delta`.
    #[serde(rename = "type")]
    pub type_value: String,
}
///Emitted when function-call arguments are finalized.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct ResponseFunctionCallArgumentsDoneEvent {
    ///The function-call arguments.
    pub arguments: String,
    ///The ID of the item.
    pub item_id: String,
    ///The name of the function that was called.
    pub name: String,
    ///The index of the output item.
    pub output_index: i32,
    ///The sequence number of this event.
    pub sequence_number: i32,
    #[serde(rename = "type")]
    pub type_value: String,
}
///Emitted when an image generation tool call has completed and the final image is available.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct ResponseImageGenCallCompletedEvent {
    ///The unique identifier of the image generation item being processed.
    pub item_id: String,
    ///The index of the output item in the response's output array.
    pub output_index: i32,
    ///The sequence number of this event.
    pub sequence_number: i32,
    ///The type of the event. Always 'response.image_generation_call.completed'.
    #[serde(rename = "type")]
    pub type_value: String,
}
///Emitted when an image generation tool call is actively generating an image (intermediate state).
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct ResponseImageGenCallGeneratingEvent {
    ///The unique identifier of the image generation item being processed.
    pub item_id: String,
    ///The index of the output item in the response's output array.
    pub output_index: i32,
    ///The sequence number of the image generation item being processed.
    pub sequence_number: i32,
    ///The type of the event. Always 'response.image_generation_call.generating'.
    #[serde(rename = "type")]
    pub type_value: String,
}
///Emitted when an image generation tool call is in progress.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct ResponseImageGenCallInProgressEvent {
    ///The unique identifier of the image generation item being processed.
    pub item_id: String,
    ///The index of the output item in the response's output array.
    pub output_index: i32,
    ///The sequence number of the image generation item being processed.
    pub sequence_number: i32,
    ///The type of the event. Always 'response.image_generation_call.in_progress'.
    #[serde(rename = "type")]
    pub type_value: String,
}
///Emitted when a partial image is available during image generation streaming.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct ResponseImageGenCallPartialImageEvent {
    ///The unique identifier of the image generation item being processed.
    pub item_id: String,
    ///The index of the output item in the response's output array.
    pub output_index: i32,
    ///Base64-encoded partial image data, suitable for rendering as an image.
    #[serde(rename = "partial_image_b64")]
    pub partial_image_b_64: String,
    ///0-based index for the partial image (backend is 1-based, but this is 0-based for the user).
    pub partial_image_index: i32,
    ///The sequence number of the image generation item being processed.
    pub sequence_number: i32,
    ///The type of the event. Always 'response.image_generation_call.partial_image'.
    #[serde(rename = "type")]
    pub type_value: String,
}
///Emitted when the response is in progress.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct ResponseInProgressEvent {
    ///The response that is in progress.
    pub response: Response,
    ///The sequence number of this event.
    pub sequence_number: i32,
    ///The type of the event. Always `response.in_progress`.
    #[serde(rename = "type")]
    pub type_value: String,
}
///Details about why the response is incomplete.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct ResponseIncompleteDetails {
    ///The reason why the response is incomplete.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reason: ::std::option::Option<String>,
}
///An event that is emitted when a response finishes as incomplete.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct ResponseIncompleteEvent {
    ///The response that was incomplete.
    pub response: Response,
    ///The sequence number of this event.
    pub sequence_number: i32,
    ///The type of the event. Always `response.incomplete`.
    #[serde(rename = "type")]
    pub type_value: String,
}
///A system (or developer) message inserted into the model's context. When using along with `previous_response_id`, the instructions from a previous response will not be carried over to the next response. This makes it simple to swap out system (or developer) messages in new responses.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
#[serde(untagged)]
pub enum ResponseInstructions {
    String(String),
    InputItemList(Vec<InputItem>),
}
///A list of Response items.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct ResponseItemList {
    ///A list of items used to generate this response.
    pub data: Vec<ItemResource>,
    ///The ID of the first item in the list.
    pub first_id: String,
    ///Whether there are more items available.
    pub has_more: bool,
    ///The ID of the last item in the list.
    pub last_id: String,
    ///The type of object returned, must be `list`.
    pub object: String,
}
///A logprob is the logarithmic probability that the model assigns to producing a particular token at a given position in the sequence. Less-negative (higher) logprob values indicate greater model confidence in that token choice.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct ResponseLogProb {
    ///The log probability of this token.
    pub logprob: f64,
    ///A possible text token.
    pub token: String,
    ///The log probabilities of up to 20 of the most likely tokens.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub top_logprobs: ::std::option::Option<Vec<ResponseLogProbTopLogprob>>,
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct ResponseLogProbTopLogprob {
    ///The log probability of this token.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub logprob: ::std::option::Option<f64>,
    ///A possible text token.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub token: ::std::option::Option<String>,
}
///Emitted when there is a delta (partial update) to the arguments of an MCP tool call.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct ResponseMcpCallArgumentsDeltaEvent {
    ///A JSON string containing the partial update to the arguments for the MCP tool call.
    pub delta: String,
    ///The unique identifier of the MCP tool call item being processed.
    pub item_id: String,
    ///The index of the output item in the response's output array.
    pub output_index: i32,
    ///The sequence number of this event.
    pub sequence_number: i32,
    ///The type of the event. Always 'response.mcp_call_arguments.delta'.
    #[serde(rename = "type")]
    pub type_value: String,
}
///Emitted when the arguments for an MCP tool call are finalized.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct ResponseMcpCallArgumentsDoneEvent {
    ///A JSON string containing the finalized arguments for the MCP tool call.
    pub arguments: String,
    ///The unique identifier of the MCP tool call item being processed.
    pub item_id: String,
    ///The index of the output item in the response's output array.
    pub output_index: i32,
    ///The sequence number of this event.
    pub sequence_number: i32,
    ///The type of the event. Always 'response.mcp_call_arguments.done'.
    #[serde(rename = "type")]
    pub type_value: String,
}
///Emitted when an MCP tool call has completed successfully.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct ResponseMcpCallCompletedEvent {
    ///The ID of the MCP tool call item that completed.
    pub item_id: String,
    ///The index of the output item that completed.
    pub output_index: i32,
    ///The sequence number of this event.
    pub sequence_number: i32,
    ///The type of the event. Always 'response.mcp_call.completed'.
    #[serde(rename = "type")]
    pub type_value: String,
}
///Emitted when an MCP tool call has failed.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct ResponseMcpCallFailedEvent {
    ///The ID of the MCP tool call item that failed.
    pub item_id: String,
    ///The index of the output item that failed.
    pub output_index: i32,
    ///The sequence number of this event.
    pub sequence_number: i32,
    ///The type of the event. Always 'response.mcp_call.failed'.
    #[serde(rename = "type")]
    pub type_value: String,
}
///Emitted when an MCP tool call is in progress.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct ResponseMcpCallInProgressEvent {
    ///The unique identifier of the MCP tool call item being processed.
    pub item_id: String,
    ///The index of the output item in the response's output array.
    pub output_index: i32,
    ///The sequence number of this event.
    pub sequence_number: i32,
    ///The type of the event. Always 'response.mcp_call.in_progress'.
    #[serde(rename = "type")]
    pub type_value: String,
}
///Emitted when the list of available MCP tools has been successfully retrieved.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct ResponseMcpListToolsCompletedEvent {
    ///The ID of the MCP tool call item that produced this output.
    pub item_id: String,
    ///The index of the output item that was processed.
    pub output_index: i32,
    ///The sequence number of this event.
    pub sequence_number: i32,
    ///The type of the event. Always 'response.mcp_list_tools.completed'.
    #[serde(rename = "type")]
    pub type_value: String,
}
///Emitted when the attempt to list available MCP tools has failed.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct ResponseMcpListToolsFailedEvent {
    ///The ID of the MCP tool call item that failed.
    pub item_id: String,
    ///The index of the output item that failed.
    pub output_index: i32,
    ///The sequence number of this event.
    pub sequence_number: i32,
    ///The type of the event. Always 'response.mcp_list_tools.failed'.
    #[serde(rename = "type")]
    pub type_value: String,
}
///Emitted when the system is in the process of retrieving the list of available MCP tools.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct ResponseMcpListToolsInProgressEvent {
    ///The ID of the MCP tool call item that is being processed.
    pub item_id: String,
    ///The index of the output item that is being processed.
    pub output_index: i32,
    ///The sequence number of this event.
    pub sequence_number: i32,
    ///The type of the event. Always 'response.mcp_list_tools.in_progress'.
    #[serde(rename = "type")]
    pub type_value: String,
}
pub type ResponseModalities = Vec<String>;
///Emitted when a new output item is added.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct ResponseOutputItemAddedEvent {
    ///The output item that was added.
    pub item: OutputItem,
    ///The index of the output item that was added.
    pub output_index: i32,
    ///The sequence number of this event.
    pub sequence_number: i32,
    ///The type of the event. Always `response.output_item.added`.
    #[serde(rename = "type")]
    pub type_value: String,
}
///Emitted when an output item is marked done.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct ResponseOutputItemDoneEvent {
    ///The output item that was marked done.
    pub item: OutputItem,
    ///The index of the output item that was marked done.
    pub output_index: i32,
    ///The sequence number of this event.
    pub sequence_number: i32,
    ///The type of the event. Always `response.output_item.done`.
    #[serde(rename = "type")]
    pub type_value: String,
}
///Assistant response text accompanied by optional annotations.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct ResponseOutputText {
    ///Ordered list of annotations attached to the response text.
    pub annotations: Vec<ResponseOutputTextAnnotation>,
    ///Assistant generated text.
    pub text: String,
    ///Type discriminator that is always `output_text`.
    #[serde(rename = "type")]
    pub type_value: String,
}
///Annotation object describing a cited source.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
#[serde(untagged)]
pub enum ResponseOutputTextAnnotation {
    FileAnnotation(FileAnnotation),
    UrlAnnotation(UrlAnnotation),
}
///Emitted when an annotation is added to output text content.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct ResponseOutputTextAnnotationAddedEvent {
    ///The annotation object being added. (See annotation schema for details.)
    pub annotation: OpenAiJsonValue,
    ///The index of the annotation within the content part.
    pub annotation_index: i32,
    ///The index of the content part within the output item.
    pub content_index: i32,
    ///The unique identifier of the item to which the annotation is being added.
    pub item_id: String,
    ///The index of the output item in the response's output array.
    pub output_index: i32,
    ///The sequence number of this event.
    pub sequence_number: i32,
    ///The type of the event. Always 'response.output_text.annotation.added'.
    #[serde(rename = "type")]
    pub type_value: String,
}
pub type ResponsePromptVariables = OpenAiJsonValue;
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct ResponseProperties {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub background: ::std::option::Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_tool_calls: ::std::option::Option<i32>,
    ///Model ID used to generate the response, like `gpt-4o` or `o3`. OpenAI offers a wide range of models with different capabilities, performance characteristics, and price points. Refer to the [model guide](/docs/models) to browse and compare available models.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model: ::std::option::Option<ModelIdsResponses>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub previous_response_id: ::std::option::Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub prompt: ::std::option::Option<Prompt>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reasoning: ::std::option::Option<Reasoning>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub text: ::std::option::Option<ResponseTextParam>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tool_choice: ::std::option::Option<ToolChoiceParam>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tools: ::std::option::Option<ToolsArray>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub truncation: ::std::option::Option<String>,
}
///Emitted when a response is queued and waiting to be processed.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct ResponseQueuedEvent {
    ///The full response object that is queued.
    pub response: Response,
    ///The sequence number for this event.
    pub sequence_number: i32,
    ///The type of the event. Always 'response.queued'.
    #[serde(rename = "type")]
    pub type_value: String,
}
///Emitted when a new reasoning summary part is added.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct ResponseReasoningSummaryPartAddedEvent {
    ///The ID of the item this summary part is associated with.
    pub item_id: String,
    ///The index of the output item this summary part is associated with.
    pub output_index: i32,
    ///The summary part that was added.
    pub part: ResponseReasoningSummaryPartAddedEventPart,
    ///The sequence number of this event.
    pub sequence_number: i32,
    ///The index of the summary part within the reasoning summary.
    pub summary_index: i32,
    ///The type of the event. Always `response.reasoning_summary_part.added`.
    #[serde(rename = "type")]
    pub type_value: String,
}
///The summary part that was added.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct ResponseReasoningSummaryPartAddedEventPart {
    ///The text of the summary part.
    pub text: String,
    ///The type of the summary part. Always `summary_text`.
    #[serde(rename = "type")]
    pub type_value: String,
}
///Emitted when a reasoning summary part is completed.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct ResponseReasoningSummaryPartDoneEvent {
    ///The ID of the item this summary part is associated with.
    pub item_id: String,
    ///The index of the output item this summary part is associated with.
    pub output_index: i32,
    ///The completed summary part.
    pub part: ResponseReasoningSummaryPartDoneEventPart,
    ///The sequence number of this event.
    pub sequence_number: i32,
    ///The index of the summary part within the reasoning summary.
    pub summary_index: i32,
    ///The type of the event. Always `response.reasoning_summary_part.done`.
    #[serde(rename = "type")]
    pub type_value: String,
}
///The completed summary part.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct ResponseReasoningSummaryPartDoneEventPart {
    ///The text of the summary part.
    pub text: String,
    ///The type of the summary part. Always `summary_text`.
    #[serde(rename = "type")]
    pub type_value: String,
}
///Emitted when a delta is added to a reasoning summary text.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct ResponseReasoningSummaryTextDeltaEvent {
    ///The text delta that was added to the summary.
    pub delta: String,
    ///The ID of the item this summary text delta is associated with.
    pub item_id: String,
    ///The index of the output item this summary text delta is associated with.
    pub output_index: i32,
    ///The sequence number of this event.
    pub sequence_number: i32,
    ///The index of the summary part within the reasoning summary.
    pub summary_index: i32,
    ///The type of the event. Always `response.reasoning_summary_text.delta`.
    #[serde(rename = "type")]
    pub type_value: String,
}
///Emitted when a reasoning summary text is completed.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct ResponseReasoningSummaryTextDoneEvent {
    ///The ID of the item this summary text is associated with.
    pub item_id: String,
    ///The index of the output item this summary text is associated with.
    pub output_index: i32,
    ///The sequence number of this event.
    pub sequence_number: i32,
    ///The index of the summary part within the reasoning summary.
    pub summary_index: i32,
    ///The full text of the completed reasoning summary.
    pub text: String,
    ///The type of the event. Always `response.reasoning_summary_text.done`.
    #[serde(rename = "type")]
    pub type_value: String,
}
///Emitted when a delta is added to a reasoning text.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct ResponseReasoningTextDeltaEvent {
    ///The index of the reasoning content part this delta is associated with.
    pub content_index: i32,
    ///The text delta that was added to the reasoning content.
    pub delta: String,
    ///The ID of the item this reasoning text delta is associated with.
    pub item_id: String,
    ///The index of the output item this reasoning text delta is associated with.
    pub output_index: i32,
    ///The sequence number of this event.
    pub sequence_number: i32,
    ///The type of the event. Always `response.reasoning_text.delta`.
    #[serde(rename = "type")]
    pub type_value: String,
}
///Emitted when a reasoning text is completed.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct ResponseReasoningTextDoneEvent {
    ///The index of the reasoning content part.
    pub content_index: i32,
    ///The ID of the item this reasoning text is associated with.
    pub item_id: String,
    ///The index of the output item this reasoning text is associated with.
    pub output_index: i32,
    ///The sequence number of this event.
    pub sequence_number: i32,
    ///The full text of the completed reasoning content.
    pub text: String,
    ///The type of the event. Always `response.reasoning_text.done`.
    #[serde(rename = "type")]
    pub type_value: String,
}
///Emitted when there is a partial refusal text.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct ResponseRefusalDeltaEvent {
    ///The index of the content part that the refusal text is added to.
    pub content_index: i32,
    ///The refusal text that is added.
    pub delta: String,
    ///The ID of the output item that the refusal text is added to.
    pub item_id: String,
    ///The index of the output item that the refusal text is added to.
    pub output_index: i32,
    ///The sequence number of this event.
    pub sequence_number: i32,
    ///The type of the event. Always `response.refusal.delta`.
    #[serde(rename = "type")]
    pub type_value: String,
}
///Emitted when refusal text is finalized.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct ResponseRefusalDoneEvent {
    ///The index of the content part that the refusal text is finalized.
    pub content_index: i32,
    ///The ID of the output item that the refusal text is finalized.
    pub item_id: String,
    ///The index of the output item that the refusal text is finalized.
    pub output_index: i32,
    ///The refusal text that is finalized.
    pub refusal: String,
    ///The sequence number of this event.
    pub sequence_number: i32,
    ///The type of the event. Always `response.refusal.done`.
    #[serde(rename = "type")]
    pub type_value: String,
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
#[serde(untagged)]
pub enum ResponseStreamEvent {
    ResponseAudioDeltaEvent(ResponseAudioDeltaEvent),
    ResponseAudioDoneEvent(ResponseAudioDoneEvent),
    ResponseAudioTranscriptDeltaEvent(ResponseAudioTranscriptDeltaEvent),
    ResponseAudioTranscriptDoneEvent(ResponseAudioTranscriptDoneEvent),
    ResponseCodeInterpreterCallCodeDeltaEvent(ResponseCodeInterpreterCallCodeDeltaEvent),
    ResponseCodeInterpreterCallCodeDoneEvent(ResponseCodeInterpreterCallCodeDoneEvent),
    ResponseCodeInterpreterCallCompletedEvent(ResponseCodeInterpreterCallCompletedEvent),
    ResponseCodeInterpreterCallInProgressEvent(
        ResponseCodeInterpreterCallInProgressEvent,
    ),
    ResponseCodeInterpreterCallInterpretingEvent(
        ResponseCodeInterpreterCallInterpretingEvent,
    ),
    ResponseCompletedEvent(ResponseCompletedEvent),
    ResponseContentPartAddedEvent(ResponseContentPartAddedEvent),
    ResponseContentPartDoneEvent(ResponseContentPartDoneEvent),
    ResponseCreatedEvent(ResponseCreatedEvent),
    ResponseErrorEvent(ResponseErrorEvent),
    ResponseFileSearchCallCompletedEvent(ResponseFileSearchCallCompletedEvent),
    ResponseFileSearchCallInProgressEvent(ResponseFileSearchCallInProgressEvent),
    ResponseFileSearchCallSearchingEvent(ResponseFileSearchCallSearchingEvent),
    ResponseFunctionCallArgumentsDeltaEvent(ResponseFunctionCallArgumentsDeltaEvent),
    ResponseFunctionCallArgumentsDoneEvent(ResponseFunctionCallArgumentsDoneEvent),
    ResponseInProgressEvent(ResponseInProgressEvent),
    ResponseFailedEvent(ResponseFailedEvent),
    ResponseIncompleteEvent(ResponseIncompleteEvent),
    ResponseOutputItemAddedEvent(ResponseOutputItemAddedEvent),
    ResponseOutputItemDoneEvent(ResponseOutputItemDoneEvent),
    ResponseReasoningSummaryPartAddedEvent(ResponseReasoningSummaryPartAddedEvent),
    ResponseReasoningSummaryPartDoneEvent(ResponseReasoningSummaryPartDoneEvent),
    ResponseReasoningSummaryTextDeltaEvent(ResponseReasoningSummaryTextDeltaEvent),
    ResponseReasoningSummaryTextDoneEvent(ResponseReasoningSummaryTextDoneEvent),
    ResponseReasoningTextDeltaEvent(ResponseReasoningTextDeltaEvent),
    ResponseReasoningTextDoneEvent(ResponseReasoningTextDoneEvent),
    ResponseRefusalDeltaEvent(ResponseRefusalDeltaEvent),
    ResponseRefusalDoneEvent(ResponseRefusalDoneEvent),
    ResponseTextDeltaEvent(ResponseTextDeltaEvent),
    ResponseTextDoneEvent(ResponseTextDoneEvent),
    ResponseWebSearchCallCompletedEvent(ResponseWebSearchCallCompletedEvent),
    ResponseWebSearchCallInProgressEvent(ResponseWebSearchCallInProgressEvent),
    ResponseWebSearchCallSearchingEvent(ResponseWebSearchCallSearchingEvent),
    ResponseImageGenCallCompletedEvent(ResponseImageGenCallCompletedEvent),
    ResponseImageGenCallGeneratingEvent(ResponseImageGenCallGeneratingEvent),
    ResponseImageGenCallInProgressEvent(ResponseImageGenCallInProgressEvent),
    ResponseImageGenCallPartialImageEvent(ResponseImageGenCallPartialImageEvent),
    ResponseMcpCallArgumentsDeltaEvent(ResponseMcpCallArgumentsDeltaEvent),
    ResponseMcpCallArgumentsDoneEvent(ResponseMcpCallArgumentsDoneEvent),
    ResponseMcpCallCompletedEvent(ResponseMcpCallCompletedEvent),
    ResponseMcpCallFailedEvent(ResponseMcpCallFailedEvent),
    ResponseMcpCallInProgressEvent(ResponseMcpCallInProgressEvent),
    ResponseMcpListToolsCompletedEvent(ResponseMcpListToolsCompletedEvent),
    ResponseMcpListToolsFailedEvent(ResponseMcpListToolsFailedEvent),
    ResponseMcpListToolsInProgressEvent(ResponseMcpListToolsInProgressEvent),
    ResponseOutputTextAnnotationAddedEvent(ResponseOutputTextAnnotationAddedEvent),
    ResponseQueuedEvent(ResponseQueuedEvent),
    ResponseCustomToolCallInputDeltaEvent(ResponseCustomToolCallInputDeltaEvent),
    ResponseCustomToolCallInputDoneEvent(ResponseCustomToolCallInputDoneEvent),
}
pub type ResponseStreamOptions = ResponseStreamOptions2;
///Options for streaming responses. Only set this when you set `stream: true`.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct ResponseStreamOptions2 {
    ///When true, stream obfuscation will be enabled. Stream obfuscation adds random characters to an `obfuscation` field on streaming delta events to normalize payload sizes as a mitigation to certain side-channel attacks. These obfuscation fields are included by default, but add a small amount of overhead to the data stream. You can set `include_obfuscation` to false to optimize for bandwidth if you trust the network links between your application and the OpenAI API.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub include_obfuscation: ::std::option::Option<bool>,
}
///Emitted when there is an additional text delta.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct ResponseTextDeltaEvent {
    ///The index of the content part that the text delta was added to.
    pub content_index: i32,
    ///The text delta that was added.
    pub delta: String,
    ///The ID of the output item that the text delta was added to.
    pub item_id: String,
    ///The log probabilities of the tokens in the delta.
    pub logprobs: Vec<ResponseLogProb>,
    ///The index of the output item that the text delta was added to.
    pub output_index: i32,
    ///The sequence number for this event.
    pub sequence_number: i32,
    ///The type of the event. Always `response.output_text.delta`.
    #[serde(rename = "type")]
    pub type_value: String,
}
///Emitted when text content is finalized.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct ResponseTextDoneEvent {
    ///The index of the content part that the text content is finalized.
    pub content_index: i32,
    ///The ID of the output item that the text content is finalized.
    pub item_id: String,
    ///The log probabilities of the tokens in the delta.
    pub logprobs: Vec<ResponseLogProb>,
    ///The index of the output item that the text content is finalized.
    pub output_index: i32,
    ///The sequence number for this event.
    pub sequence_number: i32,
    ///The text content that is finalized.
    pub text: String,
    ///The type of the event. Always `response.output_text.done`.
    #[serde(rename = "type")]
    pub type_value: String,
}
///Configuration options for a text response from the model. Can be plain text or structured JSON data. Learn more: - [Text inputs and outputs](/docs/guides/text) - [Structured Outputs](/docs/guides/structured-outputs)
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct ResponseTextParam {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub format: ::std::option::Option<TextResponseFormatConfiguration>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub verbosity: ::std::option::Option<Verbosity>,
}
///Represents token usage details including input tokens, output tokens, a breakdown of output tokens, and the total tokens used.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct ResponseUsage {
    ///The number of input tokens.
    pub input_tokens: i32,
    ///A detailed breakdown of the input tokens.
    pub input_tokens_details: ResponseUsageInputTokensDetails,
    ///The number of output tokens.
    pub output_tokens: i32,
    ///A detailed breakdown of the output tokens.
    pub output_tokens_details: ResponseUsageOutputTokensDetails,
    ///The total number of tokens used.
    pub total_tokens: i32,
}
///A detailed breakdown of the input tokens.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct ResponseUsageInputTokensDetails {
    ///The number of tokens that were retrieved from the cache. [More on prompt caching](/docs/guides/prompt-caching).
    pub cached_tokens: i32,
}
///A detailed breakdown of the output tokens.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct ResponseUsageOutputTokensDetails {
    ///The number of reasoning tokens.
    pub reasoning_tokens: i32,
}
///Emitted when a web search call is completed.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct ResponseWebSearchCallCompletedEvent {
    ///Unique ID for the output item associated with the web search call.
    pub item_id: String,
    ///The index of the output item that the web search call is associated with.
    pub output_index: i32,
    ///The sequence number of the web search call being processed.
    pub sequence_number: i32,
    ///The type of the event. Always `response.web_search_call.completed`.
    #[serde(rename = "type")]
    pub type_value: String,
}
///Emitted when a web search call is initiated.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct ResponseWebSearchCallInProgressEvent {
    ///Unique ID for the output item associated with the web search call.
    pub item_id: String,
    ///The index of the output item that the web search call is associated with.
    pub output_index: i32,
    ///The sequence number of the web search call being processed.
    pub sequence_number: i32,
    ///The type of the event. Always `response.web_search_call.in_progress`.
    #[serde(rename = "type")]
    pub type_value: String,
}
///Emitted when a web search call is executing.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct ResponseWebSearchCallSearchingEvent {
    ///Unique ID for the output item associated with the web search call.
    pub item_id: String,
    ///The index of the output item that the web search call is associated with.
    pub output_index: i32,
    ///The sequence number of the web search call being processed.
    pub sequence_number: i32,
    ///The type of the event. Always `response.web_search_call.searching`.
    #[serde(rename = "type")]
    pub type_value: String,
}
///Client events accepted by the Responses WebSocket server.
pub type ResponsesClientEvent = ResponsesClientEventResponseCreate;
///Client event for creating a response over a persistent WebSocket connection. This payload uses the same top-level fields as `POST /v1/responses`. Notes: - `stream` is implicit over WebSocket and should not be sent. - `background` is not supported over WebSocket.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct ResponsesClientEventResponseCreate {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub background: ::std::option::Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub context_management: ::std::option::Option<Vec<ContextManagementParam>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub conversation: ::std::option::Option<ConversationParam>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub include: ::std::option::Option<Vec<IncludeEnum>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub input: ::std::option::Option<InputParam>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub instructions: ::std::option::Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_output_tokens: ::std::option::Option<i32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_tool_calls: ::std::option::Option<i32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub metadata: ::std::option::Option<Metadata>,
    ///Model ID used to generate the response, like `gpt-4o` or `o3`. OpenAI offers a wide range of models with different capabilities, performance characteristics, and price points. Refer to the [model guide](/docs/models) to browse and compare available models.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model: ::std::option::Option<ModelIdsResponses>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parallel_tool_calls: ::std::option::Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub previous_response_id: ::std::option::Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub prompt: ::std::option::Option<Prompt>,
    ///Used by OpenAI to cache responses for similar requests to optimize your cache hit rates. Replaces the `user` field. [Learn more](/docs/guides/prompt-caching).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub prompt_cache_key: ::std::option::Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub prompt_cache_retention: ::std::option::Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reasoning: ::std::option::Option<Reasoning>,
    ///A stable identifier used to help detect users of your application that may be violating OpenAI's usage policies. The IDs should be a string that uniquely identifies each user, with a maximum length of 64 characters. We recommend hashing their username or email address, in order to avoid sending us any identifying information. [Learn more](/docs/guides/safety-best-practices#safety-identifiers).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub safety_identifier: ::std::option::Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub service_tier: ::std::option::Option<ServiceTier>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub store: ::std::option::Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stream: ::std::option::Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stream_options: ::std::option::Option<ResponseStreamOptions>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub temperature: ::std::option::Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub text: ::std::option::Option<ResponseTextParam>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tool_choice: ::std::option::Option<ToolChoiceParam>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tools: ::std::option::Option<ToolsArray>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub top_logprobs: ::std::option::Option<i32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub top_p: ::std::option::Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub truncation: ::std::option::Option<String>,
    ///The type of the client event. Always `response.create`.
    #[serde(rename = "type")]
    pub type_value: String,
    ///This field is being replaced by `safety_identifier` and `prompt_cache_key`. Use `prompt_cache_key` instead to maintain caching optimizations. A stable identifier for your end-users. Used to boost cache hit rates by better bucketing similar requests and to help OpenAI detect and prevent abuse. [Learn more](/docs/guides/safety-best-practices#safety-identifiers).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub user: ::std::option::Option<String>,
}
///Server events emitted by the Responses WebSocket server.
pub type ResponsesServerEvent = ResponseStreamEvent;
///Details about a role that can be assigned through the public Roles API.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct Role {
    ///Optional description of the role.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: ::std::option::Option<String>,
    ///Identifier for the role.
    pub id: String,
    ///Unique name for the role.
    pub name: String,
    ///Always `role`.
    pub object: String,
    ///Permissions granted by the role.
    pub permissions: Vec<String>,
    ///Whether the role is predefined and managed by OpenAI.
    pub predefined_role: bool,
    ///Resource type the role is bound to (for example `api.organization` or `api.project`).
    pub resource_type: String,
}
///Confirmation payload returned after deleting a role.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct RoleDeletedResource {
    ///Whether the role was deleted.
    pub deleted: bool,
    ///Identifier of the deleted role.
    pub id: String,
    ///Always `role.deleted`.
    pub object: String,
}
///Paginated list of roles assigned to a principal.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct RoleListResource {
    ///Role assignments returned in the current page.
    pub data: Vec<AssignedRoleDetails>,
    ///Whether additional assignments are available when paginating.
    pub has_more: bool,
    ///Cursor to fetch the next page of results, or `null` when there are no more assignments.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub next: ::std::option::Option<String>,
    ///Always `list`.
    pub object: String,
}
pub type RunCompletionUsage = RunCompletionUsage2;
///Usage statistics related to the run. This value will be `null` if the run is not in a terminal state (i.e. `in_progress`, `queued`, etc.).
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct RunCompletionUsage2 {
    ///Number of completion tokens used over the course of the run.
    pub completion_tokens: i32,
    ///Number of prompt tokens used over the course of the run.
    pub prompt_tokens: i32,
    ///Total number of tokens used (prompt + completion).
    pub total_tokens: i32,
}
///RunGraderRequest
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct RunGraderRequest {
    ///The grader used for the fine-tuning job.
    pub grader: RunGraderRequestGrader,
    ///The dataset item provided to the grader. This will be used to populate the `item` namespace. See [the guide](/docs/guides/graders) for more details.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub item: ::std::option::Option<OpenAiJsonValue>,
    ///The model sample to be evaluated. This value will be used to populate the `sample` namespace. See [the guide](/docs/guides/graders) for more details. The `output_json` variable will be populated if the model sample is a valid JSON string.
    pub model_sample: String,
}
///The grader used for the fine-tuning job.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
#[serde(untagged)]
pub enum RunGraderRequestGrader {
    GraderStringCheck(GraderStringCheck),
    GraderTextSimilarity(GraderTextSimilarity),
    GraderPython(GraderPython),
    GraderScoreModel(GraderScoreModel),
    GraderMulti(GraderMulti),
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct RunGraderResponse {
    pub metadata: RunGraderResponseMetadata,
    pub model_grader_token_usage_per_model: OpenAiJsonValue,
    pub reward: f64,
    pub sub_rewards: OpenAiJsonValue,
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct RunGraderResponseMetadata {
    pub errors: RunGraderResponseMetadataErrors,
    pub execution_time: f64,
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sampled_model_name: ::std::option::Option<String>,
    pub scores: OpenAiJsonValue,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub token_usage: ::std::option::Option<i32>,
    #[serde(rename = "type")]
    pub type_value: String,
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct RunGraderResponseMetadataErrors {
    pub formula_parse_error: bool,
    pub invalid_variable_error: bool,
    pub model_grader_parse_error: bool,
    pub model_grader_refusal_error: bool,
    pub model_grader_server_error: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model_grader_server_error_details: ::std::option::Option<String>,
    pub other_error: bool,
    pub python_grader_runtime_error: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub python_grader_runtime_error_details: ::std::option::Option<String>,
    pub python_grader_server_error: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub python_grader_server_error_type: ::std::option::Option<String>,
    pub sample_parse_error: bool,
    pub truncated_observation_error: bool,
    pub unresponsive_reward_error: bool,
}
///Represents an execution run on a [thread](/docs/api-reference/threads).
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct RunObject {
    ///The ID of the [assistant](/docs/api-reference/assistants) used for execution of this run.
    pub assistant_id: String,
    ///The Unix timestamp (in seconds) for when the run was cancelled.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cancelled_at: ::std::option::Option<i64>,
    ///The Unix timestamp (in seconds) for when the run was completed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub completed_at: ::std::option::Option<i64>,
    ///The Unix timestamp (in seconds) for when the run was created.
    pub created_at: i64,
    ///The Unix timestamp (in seconds) for when the run will expire.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expires_at: ::std::option::Option<i64>,
    ///The Unix timestamp (in seconds) for when the run failed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub failed_at: ::std::option::Option<i64>,
    ///The identifier, which can be referenced in API endpoints.
    pub id: String,
    ///Details on why the run is incomplete. Will be `null` if the run is not incomplete.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub incomplete_details: ::std::option::Option<RunObjectIncompleteDetails>,
    ///The instructions that the [assistant](/docs/api-reference/assistants) used for this run.
    pub instructions: String,
    ///The last error associated with this run. Will be `null` if there are no errors.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_error: ::std::option::Option<RunObjectLastError>,
    ///The maximum number of completion tokens specified to have been used over the course of the run.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_completion_tokens: ::std::option::Option<i32>,
    ///The maximum number of prompt tokens specified to have been used over the course of the run.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_prompt_tokens: ::std::option::Option<i32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub metadata: ::std::option::Option<Metadata>,
    ///The model that the [assistant](/docs/api-reference/assistants) used for this run.
    pub model: String,
    ///The object type, which is always `thread.run`.
    pub object: String,
    pub parallel_tool_calls: ParallelToolCalls,
    ///Details on the action required to continue the run. Will be `null` if no action is required.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub required_action: ::std::option::Option<RunObjectRequiredAction>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub response_format: ::std::option::Option<AssistantsApiResponseFormatOption>,
    ///The Unix timestamp (in seconds) for when the run was started.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub started_at: ::std::option::Option<i64>,
    ///The status of the run, which can be either `queued`, `in_progress`, `requires_action`, `cancelling`, `cancelled`, `failed`, `completed`, `incomplete`, or `expired`.
    pub status: String,
    ///The sampling temperature used for this run. If not set, defaults to 1.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub temperature: ::std::option::Option<f64>,
    ///The ID of the [thread](/docs/api-reference/threads) that was executed on as a part of this run.
    pub thread_id: String,
    pub tool_choice: RunObjectToolChoice,
    ///The list of tools that the [assistant](/docs/api-reference/assistants) used for this run.
    pub tools: Vec<RunObjectTool>,
    ///The nucleus sampling value used for this run. If not set, defaults to 1.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub top_p: ::std::option::Option<f64>,
    pub truncation_strategy: RunObjectTruncationStrategy,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub usage: ::std::option::Option<RunCompletionUsage>,
}
///Details on why the run is incomplete. Will be `null` if the run is not incomplete.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct RunObjectIncompleteDetails {
    ///The reason why the run is incomplete. This will point to which specific token limit was reached over the course of the run.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reason: ::std::option::Option<String>,
}
///The last error associated with this run. Will be `null` if there are no errors.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct RunObjectLastError {
    ///One of `server_error`, `rate_limit_exceeded`, or `invalid_prompt`.
    pub code: String,
    ///A human-readable description of the error.
    pub message: String,
}
///Details on the action required to continue the run. Will be `null` if no action is required.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct RunObjectRequiredAction {
    ///Details on the tool outputs needed for this run to continue.
    pub submit_tool_outputs: RunObjectRequiredActionSubmitToolOutputs,
    ///For now, this is always `submit_tool_outputs`.
    #[serde(rename = "type")]
    pub type_value: String,
}
///Details on the tool outputs needed for this run to continue.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct RunObjectRequiredActionSubmitToolOutputs {
    ///A list of the relevant tool calls.
    pub tool_calls: Vec<RunToolCallObject>,
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
#[serde(untagged)]
pub enum RunObjectTool {
    AssistantToolsCode(AssistantToolsCode),
    AssistantToolsFileSearch(AssistantToolsFileSearch),
    AssistantToolsFunction(AssistantToolsFunction),
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct RunObjectToolChoice {
    #[serde(flatten)]
    pub value: OpenAiJsonObject,
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct RunObjectTruncationStrategy {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_messages: ::std::option::Option<i32>,
    ///The truncation strategy to use for the thread. The default is `auto`. If set to `last_messages`, the thread will be truncated to the n most recent messages in the thread. When set to `auto`, messages in the middle of the thread will be dropped to fit the context length of the model, `max_prompt_tokens`.
    #[serde(rename = "type")]
    pub type_value: String,
}
pub type RunStepCompletionUsage = RunStepCompletionUsage2;
///Usage statistics related to the run step. This value will be `null` while the run step's status is `in_progress`.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct RunStepCompletionUsage2 {
    ///Number of completion tokens used over the course of the run step.
    pub completion_tokens: i32,
    ///Number of prompt tokens used over the course of the run step.
    pub prompt_tokens: i32,
    ///Total number of tokens used (prompt + completion).
    pub total_tokens: i32,
}
///Represents a run step delta i.e. any changed fields on a run step during streaming.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct RunStepDeltaObject {
    ///The delta containing the fields that have changed on the run step.
    pub delta: RunStepDeltaObjectDelta,
    ///The identifier of the run step, which can be referenced in API endpoints.
    pub id: String,
    ///The object type, which is always `thread.run.step.delta`.
    pub object: String,
}
///The delta containing the fields that have changed on the run step.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct RunStepDeltaObjectDelta {
    ///The details of the run step.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub step_details: ::std::option::Option<RunStepDeltaObjectDeltaStepDetails>,
}
///The details of the run step.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
#[serde(untagged)]
pub enum RunStepDeltaObjectDeltaStepDetails {
    RunStepDeltaStepDetailsMessageCreationObject(
        RunStepDeltaStepDetailsMessageCreationObject,
    ),
    RunStepDeltaStepDetailsToolCallsObject(RunStepDeltaStepDetailsToolCallsObject),
}
///Details of the message creation by the run step.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct RunStepDeltaStepDetailsMessageCreationObject {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub message_creation: ::std::option::Option<
        RunStepDeltaStepDetailsMessageCreationObjectMessageCreation,
    >,
    ///Always `message_creation`.
    #[serde(rename = "type")]
    pub type_value: String,
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct RunStepDeltaStepDetailsMessageCreationObjectMessageCreation {
    ///The ID of the message that was created by this run step.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub message_id: ::std::option::Option<String>,
}
///Details of the Code Interpreter tool call the run step was involved in.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct RunStepDeltaStepDetailsToolCallsCodeObject {
    ///The Code Interpreter tool call definition.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub code_interpreter: ::std::option::Option<
        RunStepDeltaStepDetailsToolCallsCodeObjectCodeInterpreter,
    >,
    ///The ID of the tool call.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: ::std::option::Option<String>,
    ///The index of the tool call in the tool calls array.
    pub index: i32,
    ///The type of tool call. This is always going to be `code_interpreter` for this type of tool call.
    #[serde(rename = "type")]
    pub type_value: String,
}
///The Code Interpreter tool call definition.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct RunStepDeltaStepDetailsToolCallsCodeObjectCodeInterpreter {
    ///The input to the Code Interpreter tool call.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub input: ::std::option::Option<String>,
    ///The outputs from the Code Interpreter tool call. Code Interpreter can output one or more items, including text (`logs`) or images (`image`). Each of these are represented by a different object type.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub outputs: ::std::option::Option<
        Vec<RunStepDeltaStepDetailsToolCallsCodeObjectCodeInterpreterOutput>,
    >,
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
#[serde(untagged)]
pub enum RunStepDeltaStepDetailsToolCallsCodeObjectCodeInterpreterOutput {
    RunStepDeltaStepDetailsToolCallsCodeOutputLogsObject(
        RunStepDeltaStepDetailsToolCallsCodeOutputLogsObject,
    ),
    RunStepDeltaStepDetailsToolCallsCodeOutputImageObject(
        RunStepDeltaStepDetailsToolCallsCodeOutputImageObject,
    ),
}
///Code interpreter image output
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct RunStepDeltaStepDetailsToolCallsCodeOutputImageObject {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub image: ::std::option::Option<
        RunStepDeltaStepDetailsToolCallsCodeOutputImageObjectImage,
    >,
    ///The index of the output in the outputs array.
    pub index: i32,
    ///Always `image`.
    #[serde(rename = "type")]
    pub type_value: String,
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct RunStepDeltaStepDetailsToolCallsCodeOutputImageObjectImage {
    ///The [file](/docs/api-reference/files) ID of the image.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub file_id: ::std::option::Option<String>,
}
///Text output from the Code Interpreter tool call as part of a run step.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct RunStepDeltaStepDetailsToolCallsCodeOutputLogsObject {
    ///The index of the output in the outputs array.
    pub index: i32,
    ///The text output from the Code Interpreter tool call.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub logs: ::std::option::Option<String>,
    ///Always `logs`.
    #[serde(rename = "type")]
    pub type_value: String,
}
///File search tool call
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct RunStepDeltaStepDetailsToolCallsFileSearchObject {
    ///For now, this is always going to be an empty object.
    pub file_search: OpenAiJsonValue,
    ///The ID of the tool call object.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: ::std::option::Option<String>,
    ///The index of the tool call in the tool calls array.
    pub index: i32,
    ///The type of tool call. This is always going to be `file_search` for this type of tool call.
    #[serde(rename = "type")]
    pub type_value: String,
}
///Function tool call
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct RunStepDeltaStepDetailsToolCallsFunctionObject {
    ///The definition of the function that was called.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub function: ::std::option::Option<
        RunStepDeltaStepDetailsToolCallsFunctionObjectFunction,
    >,
    ///The ID of the tool call object.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: ::std::option::Option<String>,
    ///The index of the tool call in the tool calls array.
    pub index: i32,
    ///The type of tool call. This is always going to be `function` for this type of tool call.
    #[serde(rename = "type")]
    pub type_value: String,
}
///The definition of the function that was called.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct RunStepDeltaStepDetailsToolCallsFunctionObjectFunction {
    ///The arguments passed to the function.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub arguments: ::std::option::Option<String>,
    ///The name of the function.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: ::std::option::Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub output: ::std::option::Option<String>,
}
///Details of the tool call.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct RunStepDeltaStepDetailsToolCallsObject {
    ///An array of tool calls the run step was involved in. These can be associated with one of three types of tools: `code_interpreter`, `file_search`, or `function`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tool_calls: ::std::option::Option<
        Vec<RunStepDeltaStepDetailsToolCallsObjectToolCall>,
    >,
    ///Always `tool_calls`.
    #[serde(rename = "type")]
    pub type_value: String,
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
#[serde(untagged)]
pub enum RunStepDeltaStepDetailsToolCallsObjectToolCall {
    RunStepDeltaStepDetailsToolCallsCodeObject(
        RunStepDeltaStepDetailsToolCallsCodeObject,
    ),
    RunStepDeltaStepDetailsToolCallsFileSearchObject(
        RunStepDeltaStepDetailsToolCallsFileSearchObject,
    ),
    RunStepDeltaStepDetailsToolCallsFunctionObject(
        RunStepDeltaStepDetailsToolCallsFunctionObject,
    ),
}
///Details of the message creation by the run step.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct RunStepDetailsMessageCreationObject {
    pub message_creation: RunStepDetailsMessageCreationObjectMessageCreation,
    ///Always `message_creation`.
    #[serde(rename = "type")]
    pub type_value: String,
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct RunStepDetailsMessageCreationObjectMessageCreation {
    ///The ID of the message that was created by this run step.
    pub message_id: String,
}
///Details of the Code Interpreter tool call the run step was involved in.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct RunStepDetailsToolCallsCodeObject {
    ///The Code Interpreter tool call definition.
    pub code_interpreter: RunStepDetailsToolCallsCodeObjectCodeInterpreter,
    ///The ID of the tool call.
    pub id: String,
    ///The type of tool call. This is always going to be `code_interpreter` for this type of tool call.
    #[serde(rename = "type")]
    pub type_value: String,
}
///The Code Interpreter tool call definition.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct RunStepDetailsToolCallsCodeObjectCodeInterpreter {
    ///The input to the Code Interpreter tool call.
    pub input: String,
    ///The outputs from the Code Interpreter tool call. Code Interpreter can output one or more items, including text (`logs`) or images (`image`). Each of these are represented by a different object type.
    pub outputs: Vec<RunStepDetailsToolCallsCodeObjectCodeInterpreterOutput>,
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
#[serde(untagged)]
pub enum RunStepDetailsToolCallsCodeObjectCodeInterpreterOutput {
    RunStepDetailsToolCallsCodeOutputLogsObject(
        RunStepDetailsToolCallsCodeOutputLogsObject,
    ),
    RunStepDetailsToolCallsCodeOutputImageObject(
        RunStepDetailsToolCallsCodeOutputImageObject,
    ),
}
///Code Interpreter image output
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct RunStepDetailsToolCallsCodeOutputImageObject {
    pub image: RunStepDetailsToolCallsCodeOutputImageObjectImage,
    ///Always `image`.
    #[serde(rename = "type")]
    pub type_value: String,
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct RunStepDetailsToolCallsCodeOutputImageObjectImage {
    ///The [file](/docs/api-reference/files) ID of the image.
    pub file_id: String,
}
///Text output from the Code Interpreter tool call as part of a run step.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct RunStepDetailsToolCallsCodeOutputLogsObject {
    ///The text output from the Code Interpreter tool call.
    pub logs: String,
    ///Always `logs`.
    #[serde(rename = "type")]
    pub type_value: String,
}
///File search tool call
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct RunStepDetailsToolCallsFileSearchObject {
    ///For now, this is always going to be an empty object.
    pub file_search: RunStepDetailsToolCallsFileSearchObjectFileSearch,
    ///The ID of the tool call object.
    pub id: String,
    ///The type of tool call. This is always going to be `file_search` for this type of tool call.
    #[serde(rename = "type")]
    pub type_value: String,
}
///For now, this is always going to be an empty object.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct RunStepDetailsToolCallsFileSearchObjectFileSearch {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ranking_options: ::std::option::Option<
        RunStepDetailsToolCallsFileSearchRankingOptionsObject,
    >,
    ///The results of the file search.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub results: ::std::option::Option<
        Vec<RunStepDetailsToolCallsFileSearchResultObject>,
    >,
}
///The ranking options for the file search.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct RunStepDetailsToolCallsFileSearchRankingOptionsObject {
    pub ranker: FileSearchRanker,
    ///The score threshold for the file search. All values must be a floating point number between 0 and 1.
    pub score_threshold: f64,
}
///A result instance of the file search.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct RunStepDetailsToolCallsFileSearchResultObject {
    ///The content of the result that was found. The content is only included if requested via the include query parameter.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub content: ::std::option::Option<
        Vec<RunStepDetailsToolCallsFileSearchResultObjectContentItem>,
    >,
    ///The ID of the file that result was found in.
    pub file_id: String,
    ///The name of the file that result was found in.
    pub file_name: String,
    ///The score of the result. All values must be a floating point number between 0 and 1.
    pub score: f64,
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct RunStepDetailsToolCallsFileSearchResultObjectContentItem {
    ///The text content of the file.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub text: ::std::option::Option<String>,
    ///The type of the content.
    #[serde(rename = "type", default, skip_serializing_if = "Option::is_none")]
    pub type_value: ::std::option::Option<String>,
}
///Function tool call
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct RunStepDetailsToolCallsFunctionObject {
    ///The definition of the function that was called.
    pub function: RunStepDetailsToolCallsFunctionObjectFunction,
    ///The ID of the tool call object.
    pub id: String,
    ///The type of tool call. This is always going to be `function` for this type of tool call.
    #[serde(rename = "type")]
    pub type_value: String,
}
///The definition of the function that was called.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct RunStepDetailsToolCallsFunctionObjectFunction {
    ///The arguments passed to the function.
    pub arguments: String,
    ///The name of the function.
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub output: ::std::option::Option<String>,
}
///Details of the tool call.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct RunStepDetailsToolCallsObject {
    ///An array of tool calls the run step was involved in. These can be associated with one of three types of tools: `code_interpreter`, `file_search`, or `function`.
    pub tool_calls: Vec<RunStepDetailsToolCallsObjectToolCall>,
    ///Always `tool_calls`.
    #[serde(rename = "type")]
    pub type_value: String,
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
#[serde(untagged)]
pub enum RunStepDetailsToolCallsObjectToolCall {
    RunStepDetailsToolCallsCodeObject(RunStepDetailsToolCallsCodeObject),
    RunStepDetailsToolCallsFileSearchObject(RunStepDetailsToolCallsFileSearchObject),
    RunStepDetailsToolCallsFunctionObject(RunStepDetailsToolCallsFunctionObject),
}
///Represents a step in execution of a run.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct RunStepObject {
    ///The ID of the [assistant](/docs/api-reference/assistants) associated with the run step.
    pub assistant_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cancelled_at: ::std::option::Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub completed_at: ::std::option::Option<i64>,
    ///The Unix timestamp (in seconds) for when the run step was created.
    pub created_at: i64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expired_at: ::std::option::Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub failed_at: ::std::option::Option<i64>,
    ///The identifier of the run step, which can be referenced in API endpoints.
    pub id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_error: ::std::option::Option<RunStepObjectLastError>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub metadata: ::std::option::Option<Metadata>,
    ///The object type, which is always `thread.run.step`.
    pub object: String,
    ///The ID of the [run](/docs/api-reference/runs) that this run step is a part of.
    pub run_id: String,
    ///The status of the run step, which can be either `in_progress`, `cancelled`, `failed`, `completed`, or `expired`.
    pub status: String,
    ///The details of the run step.
    pub step_details: RunStepObjectStepDetails,
    ///The ID of the [thread](/docs/api-reference/threads) that was run.
    pub thread_id: String,
    ///The type of run step, which can be either `message_creation` or `tool_calls`.
    #[serde(rename = "type")]
    pub type_value: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub usage: ::std::option::Option<RunStepCompletionUsage>,
}
///The last error associated with this run step. Will be `null` if there are no errors.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct RunStepObjectLastError {
    ///One of `server_error` or `rate_limit_exceeded`.
    pub code: String,
    ///A human-readable description of the error.
    pub message: String,
}
///The details of the run step.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
#[serde(untagged)]
pub enum RunStepObjectStepDetails {
    RunStepDetailsMessageCreationObject(RunStepDetailsMessageCreationObject),
    RunStepDetailsToolCallsObject(RunStepDetailsToolCallsObject),
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
#[serde(untagged)]
pub enum RunStepStreamEvent {
    Object(RunStepStreamEventObject),
    Object2(RunStepStreamEventObject2),
    Object3(RunStepStreamEventObject3),
    Object4(RunStepStreamEventObject4),
    Object5(RunStepStreamEventObject5),
    Object6(RunStepStreamEventObject6),
    Object7(RunStepStreamEventObject7),
}
///Occurs when a [run step](/docs/api-reference/run-steps/step-object) is created.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct RunStepStreamEventObject {
    pub data: RunStepObject,
    pub event: String,
}
///Occurs when a [run step](/docs/api-reference/run-steps/step-object) moves to an `in_progress` state.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct RunStepStreamEventObject2 {
    pub data: RunStepObject,
    pub event: String,
}
///Occurs when parts of a [run step](/docs/api-reference/run-steps/step-object) are being streamed.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct RunStepStreamEventObject3 {
    pub data: RunStepDeltaObject,
    pub event: String,
}
///Occurs when a [run step](/docs/api-reference/run-steps/step-object) is completed.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct RunStepStreamEventObject4 {
    pub data: RunStepObject,
    pub event: String,
}
///Occurs when a [run step](/docs/api-reference/run-steps/step-object) fails.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct RunStepStreamEventObject5 {
    pub data: RunStepObject,
    pub event: String,
}
///Occurs when a [run step](/docs/api-reference/run-steps/step-object) is cancelled.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct RunStepStreamEventObject6 {
    pub data: RunStepObject,
    pub event: String,
}
///Occurs when a [run step](/docs/api-reference/run-steps/step-object) expires.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct RunStepStreamEventObject7 {
    pub data: RunStepObject,
    pub event: String,
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
#[serde(untagged)]
pub enum RunStreamEvent {
    Object(RunStreamEventObject),
    Object2(RunStreamEventObject2),
    Object3(RunStreamEventObject3),
    Object4(RunStreamEventObject4),
    Object5(RunStreamEventObject5),
    Object6(RunStreamEventObject6),
    Object7(RunStreamEventObject7),
    Object8(RunStreamEventObject8),
    Object9(RunStreamEventObject9),
    Object10(RunStreamEventObject10),
}
///Occurs when a new [run](/docs/api-reference/runs/object) is created.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct RunStreamEventObject {
    pub data: RunObject,
    pub event: String,
}
///Occurs when a [run](/docs/api-reference/runs/object) expires.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct RunStreamEventObject10 {
    pub data: RunObject,
    pub event: String,
}
///Occurs when a [run](/docs/api-reference/runs/object) moves to a `queued` status.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct RunStreamEventObject2 {
    pub data: RunObject,
    pub event: String,
}
///Occurs when a [run](/docs/api-reference/runs/object) moves to an `in_progress` status.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct RunStreamEventObject3 {
    pub data: RunObject,
    pub event: String,
}
///Occurs when a [run](/docs/api-reference/runs/object) moves to a `requires_action` status.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct RunStreamEventObject4 {
    pub data: RunObject,
    pub event: String,
}
///Occurs when a [run](/docs/api-reference/runs/object) is completed.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct RunStreamEventObject5 {
    pub data: RunObject,
    pub event: String,
}
///Occurs when a [run](/docs/api-reference/runs/object) ends with status `incomplete`.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct RunStreamEventObject6 {
    pub data: RunObject,
    pub event: String,
}
///Occurs when a [run](/docs/api-reference/runs/object) fails.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct RunStreamEventObject7 {
    pub data: RunObject,
    pub event: String,
}
///Occurs when a [run](/docs/api-reference/runs/object) moves to a `cancelling` status.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct RunStreamEventObject8 {
    pub data: RunObject,
    pub event: String,
}
///Occurs when a [run](/docs/api-reference/runs/object) is cancelled.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct RunStreamEventObject9 {
    pub data: RunObject,
    pub event: String,
}
///Tool call objects
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct RunToolCallObject {
    ///The function definition.
    pub function: RunToolCallObjectFunction,
    ///The ID of the tool call. This ID must be referenced when you submit the tool outputs in using the [Submit tool outputs to run](/docs/api-reference/runs/submitToolOutputs) endpoint.
    pub id: String,
    ///The type of tool call the output is required for. For now, this is always `function`.
    #[serde(rename = "type")]
    pub type_value: String,
}
///The function definition.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct RunToolCallObjectFunction {
    ///The arguments that the model expects you to pass to the function.
    pub arguments: String,
    ///The name of the function.
    pub name: String,
}
///A screenshot action.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct ScreenshotParam {
    ///Specifies the event type. For a screenshot action, this property is always set to `screenshot`.
    #[serde(rename = "type")]
    pub type_value: String,
}
///A scroll action.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct ScrollParam {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub keys: ::std::option::Option<Vec<String>>,
    ///The horizontal scroll distance.
    pub scroll_x: i32,
    ///The vertical scroll distance.
    pub scroll_y: i32,
    ///Specifies the event type. For a scroll action, this property is always set to `scroll`.
    #[serde(rename = "type")]
    pub type_value: String,
    ///The x-coordinate where the scroll occurred.
    pub x: i32,
    ///The y-coordinate where the scroll occurred.
    pub y: i32,
}
pub type SearchContentType = String;
pub type SearchContextSize = String;
pub type ServiceTier = String;
pub type ServiceTierEnum = String;
///Updates the default version pointer for a skill.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct SetDefaultSkillVersionBody {
    ///The skill version number to set as default.
    pub default_version: String,
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct SkillListResource {
    ///A list of items
    pub data: Vec<SkillResource>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub first_id: ::std::option::Option<String>,
    ///Whether there are more items available.
    pub has_more: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_id: ::std::option::Option<String>,
    ///The type of object returned, must be `list`.
    pub object: String,
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct SkillReferenceParam {
    ///The ID of the referenced skill.
    pub skill_id: String,
    ///References a skill created with the /v1/skills endpoint.
    #[serde(rename = "type")]
    pub type_value: String,
    ///Optional skill version. Use a positive integer or 'latest'. Omit for default.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub version: ::std::option::Option<String>,
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct SkillResource {
    ///Unix timestamp (seconds) for when the skill was created.
    pub created_at: i64,
    ///Default version for the skill.
    pub default_version: String,
    ///Description of the skill.
    pub description: String,
    ///Unique identifier for the skill.
    pub id: String,
    ///Latest version for the skill.
    pub latest_version: String,
    ///Name of the skill.
    pub name: String,
    ///The object type, which is `skill`.
    pub object: String,
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct SkillVersionListResource {
    ///A list of items
    pub data: Vec<SkillVersionResource>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub first_id: ::std::option::Option<String>,
    ///Whether there are more items available.
    pub has_more: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_id: ::std::option::Option<String>,
    ///The type of object returned, must be `list`.
    pub object: String,
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct SkillVersionResource {
    ///Unix timestamp (seconds) for when the version was created.
    pub created_at: i64,
    ///Description of the skill version.
    pub description: String,
    ///Unique identifier for the skill version.
    pub id: String,
    ///Name of the skill version.
    pub name: String,
    ///The object type, which is `skill.version`.
    pub object: String,
    ///Identifier of the skill for this version.
    pub skill_id: String,
    ///Version number for this skill.
    pub version: String,
}
///Forces the model to call the apply_patch tool when executing a tool call.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct SpecificApplyPatchParam {
    ///The tool to call. Always `apply_patch`.
    #[serde(rename = "type")]
    pub type_value: String,
}
///Forces the model to call the shell tool when a tool call is required.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct SpecificFunctionShellParam {
    ///The tool to call. Always `shell`.
    #[serde(rename = "type")]
    pub type_value: String,
}
///Emitted for each chunk of audio data generated during speech synthesis.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct SpeechAudioDeltaEvent {
    ///A chunk of Base64-encoded audio data.
    pub audio: String,
    ///The type of the event. Always `speech.audio.delta`.
    #[serde(rename = "type")]
    pub type_value: String,
}
///Emitted when the speech synthesis is complete and all audio has been streamed.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct SpeechAudioDoneEvent {
    ///The type of the event. Always `speech.audio.done`.
    #[serde(rename = "type")]
    pub type_value: String,
    ///Token usage statistics for the request.
    pub usage: SpeechAudioDoneEventUsage,
}
///Token usage statistics for the request.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct SpeechAudioDoneEventUsage {
    ///Number of input tokens in the prompt.
    pub input_tokens: i32,
    ///Number of output tokens generated.
    pub output_tokens: i32,
    ///Total number of tokens used (input + output).
    pub total_tokens: i32,
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct StaticChunkingStrategy {
    ///The number of tokens that overlap between chunks. The default value is `400`. Note that the overlap must not exceed half of `max_chunk_size_tokens`.
    pub chunk_overlap_tokens: i32,
    ///The maximum number of tokens in each chunk. The default value is `800`. The minimum value is `100` and the maximum value is `4096`.
    pub max_chunk_size_tokens: i32,
}
///Customize your own chunking strategy by setting chunk size and chunk overlap.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct StaticChunkingStrategyRequestParam {
    #[serde(rename = "static")]
    pub static_value: StaticChunkingStrategy,
    ///Always `static`.
    #[serde(rename = "type")]
    pub type_value: String,
}
///Static Chunking Strategy
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct StaticChunkingStrategyResponseParam {
    #[serde(rename = "static")]
    pub static_value: StaticChunkingStrategy,
    ///Always `static`.
    #[serde(rename = "type")]
    pub type_value: String,
}
///Not supported with latest reasoning models `o3` and `o4-mini`. Up to 4 sequences where the API will stop generating further tokens. The returned text will not contain the stop sequence.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
#[serde(untagged)]
pub enum StopConfiguration {
    String(String),
    Array(Vec<String>),
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct SubmitToolOutputsRunRequest {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stream: ::std::option::Option<bool>,
    ///A list of tools for which the outputs are being submitted.
    pub tool_outputs: Vec<SubmitToolOutputsRunRequestToolOutput>,
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct SubmitToolOutputsRunRequestToolOutput {
    ///The output of the tool call to be submitted to continue the run.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub output: ::std::option::Option<String>,
    ///The ID of the tool call in the `required_action` object within the run object the output is being submitted for.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tool_call_id: ::std::option::Option<String>,
}
///A summary text from the model.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct SummaryTextContent {
    ///A summary of the reasoning output from the model so far.
    pub text: String,
    ///The type of the object. Always `summary_text`.
    #[serde(rename = "type")]
    pub type_value: String,
}
///Collection of workflow tasks grouped together in the thread.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct TaskGroupItem {
    ///Unix timestamp (in seconds) for when the item was created.
    pub created_at: i64,
    ///Identifier of the thread item.
    pub id: String,
    ///Type discriminator that is always `chatkit.thread_item`.
    pub object: String,
    ///Tasks included in the group.
    pub tasks: Vec<TaskGroupTask>,
    ///Identifier of the parent thread.
    pub thread_id: String,
    ///Type discriminator that is always `chatkit.task_group`.
    #[serde(rename = "type")]
    pub type_value: String,
}
///Task entry that appears within a TaskGroup.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct TaskGroupTask {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub heading: ::std::option::Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub summary: ::std::option::Option<String>,
    ///Subtype for the grouped task.
    #[serde(rename = "type")]
    pub type_value: TaskType,
}
///Task emitted by the workflow to show progress and status updates.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct TaskItem {
    ///Unix timestamp (in seconds) for when the item was created.
    pub created_at: i64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub heading: ::std::option::Option<String>,
    ///Identifier of the thread item.
    pub id: String,
    ///Type discriminator that is always `chatkit.thread_item`.
    pub object: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub summary: ::std::option::Option<String>,
    ///Subtype for the task.
    pub task_type: TaskType,
    ///Identifier of the parent thread.
    pub thread_id: String,
    ///Type discriminator that is always `chatkit.task`.
    #[serde(rename = "type")]
    pub type_value: String,
}
pub type TaskType = String;
///A text content.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct TextContent {
    pub text: String,
    #[serde(rename = "type")]
    pub type_value: String,
}
///An object specifying the format that the model must output. Configuring `{ "type": "json_schema" }` enables Structured Outputs, which ensures the model will match your supplied JSON schema. Learn more in the [Structured Outputs guide](/docs/guides/structured-outputs). The default format is `{ "type": "text" }` with no additional options. **Not recommended for gpt-4o and newer models:** Setting to `{ "type": "json_object" }` enables the older JSON mode, which ensures the message the model generates is valid JSON. Using `json_schema` is preferred for models that support it.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
#[serde(untagged)]
pub enum TextResponseFormatConfiguration {
    ResponseFormatText(ResponseFormatText),
    TextResponseFormatJsonSchema(TextResponseFormatJsonSchema),
    ResponseFormatJsonObject(ResponseFormatJsonObject),
}
///JSON Schema response format. Used to generate structured JSON responses. Learn more about [Structured Outputs](/docs/guides/structured-outputs).
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct TextResponseFormatJsonSchema {
    ///A description of what the response format is for, used by the model to determine how to respond in the format.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: ::std::option::Option<String>,
    ///The name of the response format. Must be a-z, A-Z, 0-9, or contain underscores and dashes, with a maximum length of 64.
    pub name: String,
    pub schema: ResponseFormatJsonSchemaSchema,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub strict: ::std::option::Option<bool>,
    ///The type of response format being defined. Always `json_schema`.
    #[serde(rename = "type")]
    pub type_value: String,
}
///The thread item
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
#[serde(untagged)]
pub enum ThreadItem {
    UserMessageItem(UserMessageItem),
    AssistantMessageItem(AssistantMessageItem),
    WidgetMessageItem(WidgetMessageItem),
    ClientToolCallItem(ClientToolCallItem),
    TaskItem(TaskItem),
    TaskGroupItem(TaskGroupItem),
}
///A paginated list of thread items rendered for the ChatKit API.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct ThreadItemListResource {
    ///A list of items
    pub data: Vec<ThreadItem>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub first_id: ::std::option::Option<String>,
    ///Whether there are more items available.
    pub has_more: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_id: ::std::option::Option<String>,
    ///The type of object returned, must be `list`.
    pub object: String,
}
///A paginated list of ChatKit threads.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct ThreadListResource {
    ///A list of items
    pub data: Vec<ThreadResource>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub first_id: ::std::option::Option<String>,
    ///Whether there are more items available.
    pub has_more: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_id: ::std::option::Option<String>,
    ///The type of object returned, must be `list`.
    pub object: String,
}
///Represents a thread that contains [messages](/docs/api-reference/messages).
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct ThreadObject {
    ///The Unix timestamp (in seconds) for when the thread was created.
    pub created_at: i64,
    ///The identifier, which can be referenced in API endpoints.
    pub id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub metadata: ::std::option::Option<Metadata>,
    ///The object type, which is always `thread`.
    pub object: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tool_resources: ::std::option::Option<ThreadObjectToolResources>,
}
///A set of resources that are made available to the assistant's tools in this thread. The resources are specific to the type of tool. For example, the `code_interpreter` tool requires a list of file IDs, while the `file_search` tool requires a list of vector store IDs.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct ThreadObjectToolResources {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub code_interpreter: ::std::option::Option<
        ThreadObjectToolResourcesCodeInterpreter,
    >,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub file_search: ::std::option::Option<ThreadObjectToolResourcesFileSearch>,
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct ThreadObjectToolResourcesCodeInterpreter {
    ///A list of [file](/docs/api-reference/files) IDs made available to the `code_interpreter` tool. There can be a maximum of 20 files associated with the tool.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub file_ids: ::std::option::Option<Vec<String>>,
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct ThreadObjectToolResourcesFileSearch {
    ///The [vector store](/docs/api-reference/vector-stores/object) attached to this thread. There can be a maximum of 1 vector store attached to the thread.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub vector_store_ids: ::std::option::Option<Vec<String>>,
}
///Represents a ChatKit thread and its current status.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct ThreadResource {
    ///Unix timestamp (in seconds) for when the thread was created.
    pub created_at: i64,
    ///Identifier of the thread.
    pub id: String,
    ///Type discriminator that is always `chatkit.thread`.
    pub object: String,
    ///Current status for the thread. Defaults to `active` for newly created threads.
    pub status: ThreadResourceStatus,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: ::std::option::Option<String>,
    ///Free-form string that identifies your end user who owns the thread.
    pub user: String,
}
///Current status for the thread. Defaults to `active` for newly created threads.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
#[serde(untagged)]
pub enum ThreadResourceStatus {
    ActiveStatus(ActiveStatus),
    LockedStatus(LockedStatus),
    ClosedStatus(ClosedStatus),
}
pub type ThreadStreamEvent = ThreadStreamEvent2;
///Occurs when a new [thread](/docs/api-reference/threads/object) is created.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct ThreadStreamEvent2 {
    pub data: ThreadObject,
    ///Whether to enable input audio transcription.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub enabled: ::std::option::Option<bool>,
    pub event: String,
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct ToggleCertificatesRequest {
    pub certificate_ids: Vec<String>,
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct TokenCountsBody {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub conversation: ::std::option::Option<ConversationParam>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub input: ::std::option::Option<TokenCountsBodyInput>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub instructions: ::std::option::Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model: ::std::option::Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parallel_tool_calls: ::std::option::Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub previous_response_id: ::std::option::Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reasoning: ::std::option::Option<Reasoning>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub text: ::std::option::Option<ResponseTextParam>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tool_choice: ::std::option::Option<ToolChoiceParam>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tools: ::std::option::Option<Vec<Tool>>,
    ///The truncation strategy to use for the model response. - `auto`: If the input to this Response exceeds the model's context window size, the model will truncate the response to fit the context window by dropping items from the beginning of the conversation. - `disabled` (default): If the input size will exceed the context window size for a model, the request will fail with a 400 error.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub truncation: ::std::option::Option<TruncationEnum>,
}
///Text, image, or file inputs to the model, used to generate a response
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
#[serde(untagged)]
pub enum TokenCountsBodyInput {
    String(String),
    Array(Vec<InputItem>),
}
///Token counts
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct TokenCountsResource {
    pub input_tokens: i32,
    pub object: String,
}
///A tool that can be used to generate a response.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
#[serde(untagged)]
pub enum Tool {
    FunctionTool(FunctionTool),
    FileSearchTool(FileSearchTool),
    ComputerTool(ComputerTool),
    ComputerUsePreviewTool(ComputerUsePreviewTool),
    WebSearchTool(WebSearchTool),
    McpTool(McpTool),
    CodeInterpreterTool(CodeInterpreterTool),
    ImageGenTool(ImageGenTool),
    LocalShellToolParam(LocalShellToolParam),
    FunctionShellToolParam(FunctionShellToolParam),
    CustomToolParam(CustomToolParam),
    NamespaceToolParam(NamespaceToolParam),
    ToolSearchToolParam(ToolSearchToolParam),
    WebSearchPreviewTool(WebSearchPreviewTool),
    ApplyPatchToolParam(ApplyPatchToolParam),
}
///Tool selection that the assistant should honor when executing the item.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct ToolChoice {
    ///Identifier of the requested tool.
    pub id: String,
}
///Constrains the tools available to the model to a pre-defined set.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct ToolChoiceAllowed {
    ///Constrains the tools available to the model to a pre-defined set. `auto` allows the model to pick from among the allowed tools and generate a message. `required` requires the model to call one or more of the allowed tools.
    pub mode: String,
    ///A list of tool definitions that the model should be allowed to call. For the Responses API, the list of tool definitions might look like: ```json [ { "type": "function", "name": "get_weather" }, { "type": "mcp", "server_label": "deepwiki" }, { "type": "image_generation" } ] ```
    pub tools: Vec<OpenAiJsonValue>,
    ///Allowed tool configuration type. Always `allowed_tools`.
    #[serde(rename = "type")]
    pub type_value: String,
}
///Use this option to force the model to call a specific custom tool.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct ToolChoiceCustom {
    ///The name of the custom tool to call.
    pub name: String,
    ///For custom tool calling, the type is always `custom`.
    #[serde(rename = "type")]
    pub type_value: String,
}
///Use this option to force the model to call a specific function.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct ToolChoiceFunction {
    ///The name of the function to call.
    pub name: String,
    ///For function calling, the type is always `function`.
    #[serde(rename = "type")]
    pub type_value: String,
}
///Use this option to force the model to call a specific tool on a remote MCP server.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct ToolChoiceMcp {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: ::std::option::Option<String>,
    ///The label of the MCP server to use.
    pub server_label: String,
    ///For MCP tools, the type is always `mcp`.
    #[serde(rename = "type")]
    pub type_value: String,
}
///Controls which (if any) tool is called by the model. `none` means the model will not call any tool and instead generates a message. `auto` means the model can pick between generating a message or calling one or more tools. `required` means the model must call one or more tools.
pub type ToolChoiceOptions = String;
///How the model should select which tool (or tools) to use when generating a response. See the `tools` parameter to see how to specify which tools the model can call.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
#[serde(untagged)]
pub enum ToolChoiceParam {
    ToolChoiceOptions(ToolChoiceOptions),
    ToolChoiceAllowed(ToolChoiceAllowed),
    ToolChoiceTypes(ToolChoiceTypes),
    ToolChoiceFunction(ToolChoiceFunction),
    ToolChoiceMcp(ToolChoiceMcp),
    ToolChoiceCustom(ToolChoiceCustom),
    SpecificApplyPatchParam(SpecificApplyPatchParam),
    SpecificFunctionShellParam(SpecificFunctionShellParam),
}
///Indicates that the model should use a built-in tool to generate a response. [Learn more about built-in tools](/docs/guides/tools).
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct ToolChoiceTypes {
    ///The type of hosted tool the model should to use. Learn more about [built-in tools](/docs/guides/tools). Allowed values are: - `file_search` - `web_search_preview` - `computer` - `computer_use_preview` - `computer_use` - `code_interpreter` - `image_generation`
    #[serde(rename = "type")]
    pub type_value: String,
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct ToolSearchCall {
    ///Arguments used for the tool search call.
    pub arguments: OpenAiJsonValue,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub call_id: ::std::option::Option<String>,
    ///The identifier of the actor that created the item.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub created_by: ::std::option::Option<String>,
    ///Whether tool search was executed by the server or by the client.
    pub execution: ToolSearchExecutionType,
    ///The unique ID of the tool search call item.
    pub id: String,
    ///The status of the tool search call item that was recorded.
    pub status: FunctionCallStatus,
    ///The type of the item. Always `tool_search_call`.
    #[serde(rename = "type")]
    pub type_value: String,
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct ToolSearchCallItemParam {
    ///The arguments supplied to the tool search call.
    pub arguments: EmptyModelParam,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub call_id: ::std::option::Option<String>,
    ///Whether tool search was executed by the server or by the client.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub execution: ::std::option::Option<ToolSearchExecutionType>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: ::std::option::Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub status: ::std::option::Option<FunctionCallItemStatus>,
    ///The item type. Always `tool_search_call`.
    #[serde(rename = "type")]
    pub type_value: String,
}
pub type ToolSearchExecutionType = String;
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct ToolSearchOutput {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub call_id: ::std::option::Option<String>,
    ///The identifier of the actor that created the item.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub created_by: ::std::option::Option<String>,
    ///Whether tool search was executed by the server or by the client.
    pub execution: ToolSearchExecutionType,
    ///The unique ID of the tool search output item.
    pub id: String,
    ///The status of the tool search output item that was recorded.
    pub status: FunctionCallOutputStatusEnum,
    ///The loaded tool definitions returned by tool search.
    pub tools: Vec<Tool>,
    ///The type of the item. Always `tool_search_output`.
    #[serde(rename = "type")]
    pub type_value: String,
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct ToolSearchOutputItemParam {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub call_id: ::std::option::Option<String>,
    ///Whether tool search was executed by the server or by the client.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub execution: ::std::option::Option<ToolSearchExecutionType>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: ::std::option::Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub status: ::std::option::Option<FunctionCallItemStatus>,
    ///The loaded tool definitions returned by the tool search output.
    pub tools: Vec<Tool>,
    ///The item type. Always `tool_search_output`.
    #[serde(rename = "type")]
    pub type_value: String,
}
///Hosted or BYOT tool search configuration for deferred tools.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct ToolSearchToolParam {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: ::std::option::Option<String>,
    ///Whether tool search is executed by the server or by the client.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub execution: ::std::option::Option<ToolSearchExecutionType>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parameters: ::std::option::Option<EmptyModelParam>,
    ///The type of the tool. Always `tool_search`.
    #[serde(rename = "type")]
    pub type_value: String,
}
///An array of tools the model may call while generating a response. You can specify which tool to use by setting the `tool_choice` parameter. We support the following categories of tools: - **Built-in tools**: Tools that are provided by OpenAI that extend the model's capabilities, like [web search](/docs/guides/tools-web-search) or [file search](/docs/guides/tools-file-search). Learn more about [built-in tools](/docs/guides/tools). - **MCP Tools**: Integrations with third-party systems via custom MCP servers or predefined connectors such as Google Drive and SharePoint. Learn more about [MCP Tools](/docs/guides/tools-connectors-mcp). - **Function calls (custom tools)**: Functions that are defined by you, enabling the model to call your own code with strongly typed arguments and outputs. Learn more about [function calling](/docs/guides/function-calling). You can also use custom tools to call your own code.
pub type ToolsArray = Vec<Tool>;
///The top log probability of a token.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct TopLogProb {
    pub bytes: Vec<i32>,
    pub logprob: f64,
    pub token: String,
}
///Emitted when there is an additional text delta. This is also the first event emitted when the transcription starts. Only emitted when you [create a transcription](/docs/api-reference/audio/create-transcription) with the `Stream` parameter set to `true`.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct TranscriptTextDeltaEvent {
    ///The text delta that was additionally transcribed.
    pub delta: String,
    ///The log probabilities of the delta. Only included if you [create a transcription](/docs/api-reference/audio/create-transcription) with the `include[]` parameter set to `logprobs`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub logprobs: ::std::option::Option<Vec<TranscriptTextDeltaEventLogprob>>,
    ///Identifier of the diarized segment that this delta belongs to. Only present when using `gpt-4o-transcribe-diarize`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub segment_id: ::std::option::Option<String>,
    ///The type of the event. Always `transcript.text.delta`.
    #[serde(rename = "type")]
    pub type_value: String,
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct TranscriptTextDeltaEventLogprob {
    ///The bytes that were used to generate the log probability.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bytes: ::std::option::Option<Vec<i32>>,
    ///The log probability of the token.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub logprob: ::std::option::Option<f64>,
    ///The token that was used to generate the log probability.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub token: ::std::option::Option<String>,
}
///Emitted when the transcription is complete. Contains the complete transcription text. Only emitted when you [create a transcription](/docs/api-reference/audio/create-transcription) with the `Stream` parameter set to `true`.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct TranscriptTextDoneEvent {
    ///The log probabilities of the individual tokens in the transcription. Only included if you [create a transcription](/docs/api-reference/audio/create-transcription) with the `include[]` parameter set to `logprobs`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub logprobs: ::std::option::Option<Vec<TranscriptTextDoneEventLogprob>>,
    ///The text that was transcribed.
    pub text: String,
    ///The type of the event. Always `transcript.text.done`.
    #[serde(rename = "type")]
    pub type_value: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub usage: ::std::option::Option<TranscriptTextUsageTokens>,
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct TranscriptTextDoneEventLogprob {
    ///The bytes that were used to generate the log probability.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bytes: ::std::option::Option<Vec<i32>>,
    ///The log probability of the token.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub logprob: ::std::option::Option<f64>,
    ///The token that was used to generate the log probability.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub token: ::std::option::Option<String>,
}
///Emitted when a diarized transcription returns a completed segment with speaker information. Only emitted when you [create a transcription](/docs/api-reference/audio/create-transcription) with `stream` set to `true` and `response_format` set to `diarized_json`.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct TranscriptTextSegmentEvent {
    ///End timestamp of the segment in seconds.
    pub end: f64,
    ///Unique identifier for the segment.
    pub id: String,
    ///Speaker label for this segment.
    pub speaker: String,
    ///Start timestamp of the segment in seconds.
    pub start: f64,
    ///Transcript text for this segment.
    pub text: String,
    ///The type of the event. Always `transcript.text.segment`.
    #[serde(rename = "type")]
    pub type_value: String,
}
///Usage statistics for models billed by audio input duration.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct TranscriptTextUsageDuration {
    ///Duration of the input audio in seconds.
    pub seconds: f64,
    ///The type of the usage object. Always `duration` for this variant.
    #[serde(rename = "type")]
    pub type_value: String,
}
///Usage statistics for models billed by token usage.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct TranscriptTextUsageTokens {
    ///Details about the input tokens billed for this request.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub input_token_details: ::std::option::Option<
        TranscriptTextUsageTokensInputTokenDetails,
    >,
    ///Number of input tokens billed for this request.
    pub input_tokens: i32,
    ///Number of output tokens generated.
    pub output_tokens: i32,
    ///Total number of tokens used (input + output).
    pub total_tokens: i32,
    ///The type of the usage object. Always `tokens` for this variant.
    #[serde(rename = "type")]
    pub type_value: String,
}
///Details about the input tokens billed for this request.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct TranscriptTextUsageTokensInputTokenDetails {
    ///Number of audio tokens billed for this request.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub audio_tokens: ::std::option::Option<i32>,
    ///Number of text tokens billed for this request.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub text_tokens: ::std::option::Option<i32>,
}
///Controls how the audio is cut into chunks. When set to `"auto"`, the server first normalizes loudness and then uses voice activity detection (VAD) to choose boundaries. `server_vad` object can be provided to tweak VAD detection parameters manually. If unset, the audio is transcribed as a single block.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct TranscriptionChunkingStrategy {
    #[serde(flatten)]
    pub value: OpenAiJsonObject,
}
///A segment of diarized transcript text with speaker metadata.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct TranscriptionDiarizedSegment {
    ///End timestamp of the segment in seconds.
    pub end: f64,
    ///Unique identifier for the segment.
    pub id: String,
    ///Speaker label for this segment. When known speakers are provided, the label matches `known_speaker_names[]`. Otherwise speakers are labeled sequentially using capital letters (`A`, `B`, ...).
    pub speaker: String,
    ///Start timestamp of the segment in seconds.
    pub start: f64,
    ///Transcript text for this segment.
    pub text: String,
    ///The type of the segment. Always `transcript.text.segment`.
    #[serde(rename = "type")]
    pub type_value: String,
}
pub type TranscriptionInclude = String;
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct TranscriptionSegment {
    ///Average logprob of the segment. If the value is lower than -1, consider the logprobs failed.
    pub avg_logprob: f32,
    ///Compression ratio of the segment. If the value is greater than 2.4, consider the compression failed.
    pub compression_ratio: f32,
    ///End time of the segment in seconds.
    pub end: f64,
    ///Unique identifier of the segment.
    pub id: i32,
    ///Probability of no speech in the segment. If the value is higher than 1.0 and the `avg_logprob` is below -1, consider this segment silent.
    pub no_speech_prob: f32,
    ///Seek offset of the segment.
    pub seek: i32,
    ///Start time of the segment in seconds.
    pub start: f64,
    ///Temperature parameter used for generating the segment.
    pub temperature: f32,
    ///Text content of the segment.
    pub text: String,
    ///Array of token IDs for the text content.
    pub tokens: Vec<i32>,
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct TranscriptionWord {
    ///End time of the word in seconds.
    pub end: f64,
    ///Start time of the word in seconds.
    pub start: f64,
    ///The text content of the word.
    pub word: String,
}
pub type TruncationEnum = String;
///Controls for how a thread will be truncated prior to the run. Use this to control the initial context window of the run.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct TruncationObject {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_messages: ::std::option::Option<i32>,
    ///The truncation strategy to use for the thread. The default is `auto`. If set to `last_messages`, the thread will be truncated to the n most recent messages in the thread. When set to `auto`, messages in the middle of the thread will be dropped to fit the context length of the model, `max_prompt_tokens`.
    #[serde(rename = "type")]
    pub type_value: String,
}
///An action to type in text.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct TypeParam {
    ///The text to type.
    pub text: String,
    ///Specifies the event type. For a type action, this property is always set to `type`.
    #[serde(rename = "type")]
    pub type_value: String,
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct UpdateChatCompletionRequest {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub metadata: ::std::option::Option<Metadata>,
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct UpdateConversationBody {
    ///Set of 16 key-value pairs that can be attached to an object. This can be useful for storing additional information about the object in a structured format, and querying for objects via API or the dashboard. Keys are strings with a maximum length of 64 characters. Values are strings with a maximum length of 512 characters.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub metadata: ::std::option::Option<Metadata>,
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct UpdateEvalRequest {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub metadata: ::std::option::Option<Metadata>,
    ///Rename the evaluation.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: ::std::option::Option<String>,
}
///Request payload for updating the details of an existing group.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct UpdateGroupBody {
    ///New display name for the group.
    pub name: String,
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct UpdateVectorStoreFileAttributesRequest {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub attributes: ::std::option::Option<VectorStoreFileAttributes>,
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct UpdateVectorStoreRequest {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expires_after: ::std::option::Option<UpdateVectorStoreRequestExpiresAfter>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub metadata: ::std::option::Option<Metadata>,
    ///The name of the vector store.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: ::std::option::Option<String>,
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct UpdateVectorStoreRequestExpiresAfter {
    ///Anchor timestamp after which the expiration policy applies. Supported anchors: `last_active_at`.
    pub anchor: String,
    ///The number of days after the anchor time that the vector store will expire.
    pub days: i32,
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct UpdateVoiceConsentRequest {
    ///The updated label for this consent recording.
    pub name: String,
}
///The Upload object can accept byte chunks in the form of Parts.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct Upload {
    ///The intended number of bytes to be uploaded.
    pub bytes: i32,
    ///The Unix timestamp (in seconds) for when the Upload was created.
    pub created_at: i64,
    ///The Unix timestamp (in seconds) for when the Upload will expire.
    pub expires_at: i64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub file: ::std::option::Option<UploadFile>,
    ///The name of the file to be uploaded.
    pub filename: String,
    ///The Upload unique identifier, which can be referenced in API endpoints.
    pub id: String,
    ///The object type, which is always "upload".
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub object: ::std::option::Option<String>,
    ///The intended purpose of the file. [Please refer here](/docs/api-reference/files/object#files/object-purpose) for acceptable values.
    pub purpose: String,
    ///The status of the Upload.
    pub status: String,
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct UploadCertificateRequest {
    ///The certificate content in PEM format
    pub certificate: String,
    ///An optional name for the certificate
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: ::std::option::Option<String>,
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct UploadFile {
    ///The size of the file, in bytes.
    pub bytes: i32,
    ///The Unix timestamp (in seconds) for when the file was created.
    pub created_at: i64,
    ///The Unix timestamp (in seconds) for when the file will expire.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expires_at: ::std::option::Option<i64>,
    ///The name of the file.
    pub filename: String,
    ///The file identifier, which can be referenced in the API endpoints.
    pub id: String,
    ///The object type, which is always `file`.
    pub object: String,
    ///The intended purpose of the file. Supported values are `assistants`, `assistants_output`, `batch`, `batch_output`, `fine-tune`, `fine-tune-results`, `vision`, and `user_data`.
    pub purpose: String,
    ///Deprecated. The current status of the file, which can be either `uploaded`, `processed`, or `error`.
    pub status: String,
    ///Deprecated. For details on why a fine-tuning training file failed validation, see the `error` field on `fine_tuning.job`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub status_details: ::std::option::Option<String>,
}
///The upload Part represents a chunk of bytes we can add to an Upload object.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct UploadPart {
    ///The Unix timestamp (in seconds) for when the Part was created.
    pub created_at: i64,
    ///The upload Part unique identifier, which can be referenced in API endpoints.
    pub id: String,
    ///The object type, which is always `upload.part`.
    pub object: String,
    ///The ID of the Upload object that this Part was added to.
    pub upload_id: String,
}
///Annotation that references a URL.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct UrlAnnotation {
    ///URL referenced by the annotation.
    pub source: UrlAnnotationSource,
    ///Type discriminator that is always `url` for this annotation.
    #[serde(rename = "type")]
    pub type_value: String,
}
///URL backing an annotation entry.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct UrlAnnotationSource {
    ///Type discriminator that is always `url`.
    #[serde(rename = "type")]
    pub type_value: String,
    ///URL referenced by the annotation.
    pub url: String,
}
///A citation for a web resource used to generate a model response.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct UrlCitationBody {
    ///The index of the last character of the URL citation in the message.
    pub end_index: i32,
    ///The index of the first character of the URL citation in the message.
    pub start_index: i32,
    ///The title of the web resource.
    pub title: String,
    ///The type of the URL citation. Always `url_citation`.
    #[serde(rename = "type")]
    pub type_value: String,
    ///The URL of the web resource.
    pub url: String,
}
///The aggregated audio speeches usage details of the specific time bucket.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct UsageAudioSpeechesResult {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub api_key_id: ::std::option::Option<String>,
    ///The number of characters processed.
    pub characters: i32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model: ::std::option::Option<String>,
    ///The count of requests made to the model.
    pub num_model_requests: i32,
    pub object: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub project_id: ::std::option::Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub user_id: ::std::option::Option<String>,
}
///The aggregated audio transcriptions usage details of the specific time bucket.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct UsageAudioTranscriptionsResult {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub api_key_id: ::std::option::Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model: ::std::option::Option<String>,
    ///The count of requests made to the model.
    pub num_model_requests: i32,
    pub object: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub project_id: ::std::option::Option<String>,
    ///The number of seconds processed.
    pub seconds: i64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub user_id: ::std::option::Option<String>,
}
///The aggregated code interpreter sessions usage details of the specific time bucket.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct UsageCodeInterpreterSessionsResult {
    ///The number of code interpreter sessions.
    pub num_sessions: i32,
    pub object: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub project_id: ::std::option::Option<String>,
}
///The aggregated completions usage details of the specific time bucket.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct UsageCompletionsResult {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub api_key_id: ::std::option::Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub batch: ::std::option::Option<bool>,
    ///The aggregated number of audio input tokens used, including cached tokens.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub input_audio_tokens: ::std::option::Option<i32>,
    ///The aggregated number of text input tokens that has been cached from previous requests. For customers subscribe to scale tier, this includes scale tier tokens.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub input_cached_tokens: ::std::option::Option<i32>,
    ///The aggregated number of text input tokens used, including cached tokens. For customers subscribe to scale tier, this includes scale tier tokens.
    pub input_tokens: i32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model: ::std::option::Option<String>,
    ///The count of requests made to the model.
    pub num_model_requests: i32,
    pub object: String,
    ///The aggregated number of audio output tokens used.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub output_audio_tokens: ::std::option::Option<i32>,
    ///The aggregated number of text output tokens used. For customers subscribe to scale tier, this includes scale tier tokens.
    pub output_tokens: i32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub project_id: ::std::option::Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub service_tier: ::std::option::Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub user_id: ::std::option::Option<String>,
}
///The aggregated embeddings usage details of the specific time bucket.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct UsageEmbeddingsResult {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub api_key_id: ::std::option::Option<String>,
    ///The aggregated number of input tokens used.
    pub input_tokens: i32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model: ::std::option::Option<String>,
    ///The count of requests made to the model.
    pub num_model_requests: i32,
    pub object: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub project_id: ::std::option::Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub user_id: ::std::option::Option<String>,
}
///The aggregated images usage details of the specific time bucket.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct UsageImagesResult {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub api_key_id: ::std::option::Option<String>,
    ///The number of images processed.
    pub images: i32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model: ::std::option::Option<String>,
    ///The count of requests made to the model.
    pub num_model_requests: i32,
    pub object: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub project_id: ::std::option::Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub size: ::std::option::Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source: ::std::option::Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub user_id: ::std::option::Option<String>,
}
///The aggregated moderations usage details of the specific time bucket.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct UsageModerationsResult {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub api_key_id: ::std::option::Option<String>,
    ///The aggregated number of input tokens used.
    pub input_tokens: i32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model: ::std::option::Option<String>,
    ///The count of requests made to the model.
    pub num_model_requests: i32,
    pub object: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub project_id: ::std::option::Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub user_id: ::std::option::Option<String>,
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct UsageResponse {
    pub data: Vec<UsageTimeBucket>,
    pub has_more: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub next_page: ::std::option::Option<String>,
    pub object: String,
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct UsageTimeBucket {
    pub end_time: i32,
    pub object: String,
    pub results: Vec<UsageTimeBucketResult>,
    pub start_time: i32,
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
#[serde(untagged)]
pub enum UsageTimeBucketResult {
    UsageCompletionsResult(UsageCompletionsResult),
    UsageEmbeddingsResult(UsageEmbeddingsResult),
    UsageModerationsResult(UsageModerationsResult),
    UsageImagesResult(UsageImagesResult),
    UsageAudioSpeechesResult(UsageAudioSpeechesResult),
    UsageAudioTranscriptionsResult(UsageAudioTranscriptionsResult),
    UsageVectorStoresResult(UsageVectorStoresResult),
    UsageCodeInterpreterSessionsResult(UsageCodeInterpreterSessionsResult),
    CostsResult(CostsResult),
}
///The aggregated vector stores usage details of the specific time bucket.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct UsageVectorStoresResult {
    pub object: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub project_id: ::std::option::Option<String>,
    ///The vector stores usage in bytes.
    pub usage_bytes: i32,
}
///Represents an individual `user` within an organization.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct User {
    ///The Unix timestamp (in seconds) of when the user was added.
    pub added_at: i64,
    ///The Unix timestamp (in seconds) of the user's last API key usage.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub api_key_last_used_at: ::std::option::Option<i64>,
    ///The Unix timestamp (in seconds) of when the user was created.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub created: ::std::option::Option<i64>,
    ///The developer persona metadata for the user.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub developer_persona: ::std::option::Option<String>,
    ///The email address of the user
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub email: ::std::option::Option<String>,
    ///The identifier, which can be referenced in API endpoints
    pub id: String,
    ///Whether this is the organization's default user.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub is_default: ::std::option::Option<bool>,
    ///Whether the user is an authorized purchaser for Scale Tier.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub is_scale_tier_authorized_purchaser: ::std::option::Option<bool>,
    ///Whether the user is managed through SCIM.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub is_scim_managed: ::std::option::Option<bool>,
    ///Whether the user is a service account.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub is_service_account: ::std::option::Option<bool>,
    ///The name of the user
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: ::std::option::Option<String>,
    ///The object type, which is always `organization.user`
    pub object: String,
    ///Projects associated with the user, if included.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub projects: ::std::option::Option<UserProjects>,
    ///`owner` or `reader`
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub role: ::std::option::Option<String>,
    ///The technical level metadata for the user.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub technical_level: ::std::option::Option<String>,
    ///Nested user details.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub user: ::std::option::Option<UserUser>,
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct UserDeleteResponse {
    pub deleted: bool,
    pub id: String,
    pub object: String,
}
///Paginated list of user objects returned when inspecting group membership.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct UserListResource {
    ///Users in the current page.
    pub data: Vec<GroupUser>,
    ///Whether more users are available when paginating.
    pub has_more: bool,
    ///Cursor to fetch the next page of results, or `null` when no further users are available.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub next: ::std::option::Option<String>,
    ///Always `list`.
    pub object: String,
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct UserListResponse {
    pub data: Vec<User>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub first_id: ::std::option::Option<String>,
    pub has_more: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_id: ::std::option::Option<String>,
    pub object: String,
}
///Text block that a user contributed to the thread.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct UserMessageInputText {
    ///Plain-text content supplied by the user.
    pub text: String,
    ///Type discriminator that is always `input_text`.
    #[serde(rename = "type")]
    pub type_value: String,
}
///User-authored messages within a thread.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct UserMessageItem {
    ///Attachments associated with the user message. Defaults to an empty list.
    pub attachments: Vec<Attachment>,
    ///Ordered content elements supplied by the user.
    pub content: Vec<UserMessageItemContentItem>,
    ///Unix timestamp (in seconds) for when the item was created.
    pub created_at: i64,
    ///Identifier of the thread item.
    pub id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub inference_options: ::std::option::Option<InferenceOptions>,
    ///Type discriminator that is always `chatkit.thread_item`.
    pub object: String,
    ///Identifier of the parent thread.
    pub thread_id: String,
    #[serde(rename = "type")]
    pub type_value: String,
}
///Content blocks that comprise a user message.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
#[serde(untagged)]
pub enum UserMessageItemContentItem {
    UserMessageInputText(UserMessageInputText),
    UserMessageQuotedText(UserMessageQuotedText),
}
///Quoted snippet that the user referenced in their message.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct UserMessageQuotedText {
    ///Quoted text content.
    pub text: String,
    ///Type discriminator that is always `quoted_text`.
    #[serde(rename = "type")]
    pub type_value: String,
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct UserProjects {
    pub data: Vec<UserProjectsDataItem>,
    pub object: String,
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct UserProjectsDataItem {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: ::std::option::Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: ::std::option::Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub role: ::std::option::Option<String>,
}
///Role assignment linking a user to a role.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct UserRoleAssignment {
    ///Always `user.role`.
    pub object: String,
    pub role: Role,
    pub user: User,
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct UserRoleUpdateRequest {
    ///Developer persona metadata.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub developer_persona: ::std::option::Option<String>,
    ///`owner` or `reader`
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub role: ::std::option::Option<String>,
    ///Role ID to assign to the user.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub role_id: ::std::option::Option<String>,
    ///Technical level metadata.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub technical_level: ::std::option::Option<String>,
}
///Nested user details.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct UserUser {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub banned: ::std::option::Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub banned_at: ::std::option::Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub email: ::std::option::Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub enabled: ::std::option::Option<bool>,
    pub id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: ::std::option::Option<String>,
    pub object: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub picture: ::std::option::Option<String>,
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct VadConfig {
    ///Amount of audio to include before the VAD detected speech (in milliseconds).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub prefix_padding_ms: ::std::option::Option<i32>,
    ///Duration of silence to detect speech stop (in milliseconds). With shorter values the model will respond more quickly, but may jump in on short pauses from the user.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub silence_duration_ms: ::std::option::Option<i32>,
    ///Sensitivity threshold (0.0 to 1.0) for voice activity detection. A higher threshold will require louder audio to activate the model, and thus might perform better in noisy environments.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub threshold: ::std::option::Option<f64>,
    ///Must be set to `server_vad` to enable manual chunking using server side VAD.
    #[serde(rename = "type")]
    pub type_value: String,
}
///ValidateGraderRequest
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct ValidateGraderRequest {
    ///The grader used for the fine-tuning job.
    pub grader: ValidateGraderRequestGrader,
}
///The grader used for the fine-tuning job.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
#[serde(untagged)]
pub enum ValidateGraderRequestGrader {
    GraderStringCheck(GraderStringCheck),
    GraderTextSimilarity(GraderTextSimilarity),
    GraderPython(GraderPython),
    GraderScoreModel(GraderScoreModel),
    GraderMulti(GraderMulti),
}
///ValidateGraderResponse
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct ValidateGraderResponse {
    ///The grader used for the fine-tuning job.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub grader: ::std::option::Option<ValidateGraderResponseGrader>,
}
///The grader used for the fine-tuning job.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
#[serde(untagged)]
pub enum ValidateGraderResponseGrader {
    GraderStringCheck(GraderStringCheck),
    GraderTextSimilarity(GraderTextSimilarity),
    GraderPython(GraderPython),
    GraderScoreModel(GraderScoreModel),
    GraderMulti(GraderMulti),
}
///The expiration policy for a vector store.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct VectorStoreExpirationAfter {
    ///Anchor timestamp after which the expiration policy applies. Supported anchors: `last_active_at`.
    pub anchor: String,
    ///The number of days after the anchor time that the vector store will expire.
    pub days: i32,
}
pub type VectorStoreFileAttributes = OpenAiJsonValue;
///A batch of files attached to a vector store.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct VectorStoreFileBatchObject {
    ///The Unix timestamp (in seconds) for when the vector store files batch was created.
    pub created_at: i64,
    pub file_counts: VectorStoreFileBatchObjectFileCounts,
    ///The identifier, which can be referenced in API endpoints.
    pub id: String,
    ///The object type, which is always `vector_store.file_batch`.
    pub object: String,
    ///The status of the vector store files batch, which can be either `in_progress`, `completed`, `cancelled` or `failed`.
    pub status: String,
    ///The ID of the [vector store](/docs/api-reference/vector-stores/object) that the [File](/docs/api-reference/files) is attached to.
    pub vector_store_id: String,
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct VectorStoreFileBatchObjectFileCounts {
    ///The number of files that where cancelled.
    pub cancelled: i32,
    ///The number of files that have been processed.
    pub completed: i32,
    ///The number of files that have failed to process.
    pub failed: i32,
    ///The number of files that are currently being processed.
    pub in_progress: i32,
    ///The total number of files.
    pub total: i32,
}
///Represents the parsed content of a vector store file.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct VectorStoreFileContentResponse {
    ///Parsed content of the file.
    pub data: Vec<VectorStoreFileContentResponseDataItem>,
    ///Indicates if there are more content pages to fetch.
    pub has_more: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub next_page: ::std::option::Option<String>,
    ///The object type, which is always `vector_store.file_content.page`
    pub object: String,
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct VectorStoreFileContentResponseDataItem {
    ///The text content
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub text: ::std::option::Option<String>,
    ///The content type (currently only `"text"`)
    #[serde(rename = "type", default, skip_serializing_if = "Option::is_none")]
    pub type_value: ::std::option::Option<String>,
}
///A list of files attached to a vector store.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct VectorStoreFileObject {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub attributes: ::std::option::Option<VectorStoreFileAttributes>,
    ///The strategy used to chunk the file.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub chunking_strategy: ::std::option::Option<VectorStoreFileObjectChunkingStrategy>,
    ///The Unix timestamp (in seconds) for when the vector store file was created.
    pub created_at: i64,
    ///The identifier, which can be referenced in API endpoints.
    pub id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_error: ::std::option::Option<VectorStoreFileObjectLastError>,
    ///The object type, which is always `vector_store.file`.
    pub object: String,
    ///The status of the vector store file, which can be either `in_progress`, `completed`, `cancelled`, or `failed`. The status `completed` indicates that the vector store file is ready for use.
    pub status: String,
    ///The total vector store usage in bytes. Note that this may be different from the original file size.
    pub usage_bytes: i32,
    ///The ID of the [vector store](/docs/api-reference/vector-stores/object) that the [File](/docs/api-reference/files) is attached to.
    pub vector_store_id: String,
}
///The strategy used to chunk the file.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
#[serde(untagged)]
pub enum VectorStoreFileObjectChunkingStrategy {
    StaticChunkingStrategyResponseParam(StaticChunkingStrategyResponseParam),
    OtherChunkingStrategyResponseParam(OtherChunkingStrategyResponseParam),
}
///The last error associated with this vector store file. Will be `null` if there are no errors.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct VectorStoreFileObjectLastError {
    ///One of `server_error`, `unsupported_file`, or `invalid_file`.
    pub code: String,
    ///A human-readable description of the error.
    pub message: String,
}
///A vector store is a collection of processed files can be used by the `file_search` tool.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct VectorStoreObject {
    ///The Unix timestamp (in seconds) for when the vector store was created.
    pub created_at: i64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expires_after: ::std::option::Option<VectorStoreExpirationAfter>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expires_at: ::std::option::Option<i64>,
    pub file_counts: VectorStoreObjectFileCounts,
    ///The identifier, which can be referenced in API endpoints.
    pub id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_active_at: ::std::option::Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub metadata: ::std::option::Option<Metadata>,
    ///The name of the vector store.
    pub name: String,
    ///The object type, which is always `vector_store`.
    pub object: String,
    ///The status of the vector store, which can be either `expired`, `in_progress`, or `completed`. A status of `completed` indicates that the vector store is ready for use.
    pub status: String,
    ///The total number of bytes used by the files in the vector store.
    pub usage_bytes: i32,
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct VectorStoreObjectFileCounts {
    ///The number of files that were cancelled.
    pub cancelled: i32,
    ///The number of files that have been successfully processed.
    pub completed: i32,
    ///The number of files that have failed to process.
    pub failed: i32,
    ///The number of files that are currently being processed.
    pub in_progress: i32,
    ///The total number of files.
    pub total: i32,
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct VectorStoreSearchRequest {
    ///A filter to apply based on file attributes.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub filters: ::std::option::Option<VectorStoreSearchRequestFilters>,
    ///The maximum number of results to return. This number should be between 1 and 50 inclusive.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_num_results: ::std::option::Option<i32>,
    ///A query string for a search
    pub query: VectorStoreSearchRequestQuery,
    ///Ranking options for search.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ranking_options: ::std::option::Option<VectorStoreSearchRequestRankingOptions>,
    ///Whether to rewrite the natural language query for vector search.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rewrite_query: ::std::option::Option<bool>,
}
///A filter to apply based on file attributes.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
#[serde(untagged)]
pub enum VectorStoreSearchRequestFilters {
    ComparisonFilter(ComparisonFilter),
    CompoundFilter(CompoundFilter),
}
///A query string for a search
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
#[serde(untagged)]
pub enum VectorStoreSearchRequestQuery {
    String(String),
    Array(Vec<String>),
}
///Ranking options for search.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct VectorStoreSearchRequestRankingOptions {
    ///Enable re-ranking; set to `none` to disable, which can help reduce latency.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ranker: ::std::option::Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub score_threshold: ::std::option::Option<f64>,
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct VectorStoreSearchResultContentObject {
    ///The text content returned from search.
    pub text: String,
    ///The type of content.
    #[serde(rename = "type")]
    pub type_value: String,
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct VectorStoreSearchResultItem {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub attributes: ::std::option::Option<VectorStoreFileAttributes>,
    ///Content chunks from the file.
    pub content: Vec<VectorStoreSearchResultContentObject>,
    ///The ID of the vector store file.
    pub file_id: String,
    ///The name of the vector store file.
    pub filename: String,
    ///The similarity score for the result.
    pub score: f64,
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct VectorStoreSearchResultsPage {
    ///The list of search result items.
    pub data: Vec<VectorStoreSearchResultItem>,
    ///Indicates if there are more results to fetch.
    pub has_more: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub next_page: ::std::option::Option<String>,
    ///The object type, which is always `vector_store.search_results.page`
    pub object: String,
    pub search_query: Vec<String>,
}
pub type Verbosity = String;
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct VideoCharacterResource {
    ///Unix timestamp (in seconds) when the character was created.
    pub created_at: i64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: ::std::option::Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: ::std::option::Option<String>,
}
pub type VideoContentVariant = String;
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct VideoListResource {
    ///A list of items
    pub data: Vec<VideoResource>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub first_id: ::std::option::Option<String>,
    ///Whether there are more items available.
    pub has_more: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_id: ::std::option::Option<String>,
    ///The type of object returned, must be `list`.
    pub object: String,
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
#[serde(untagged)]
pub enum VideoModel {
    String(String),
    String2(String),
}
///Reference to the completed video.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct VideoReferenceInputParam {
    ///The identifier of the completed video.
    pub id: String,
}
///Structured information describing a generated video job.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct VideoResource {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub completed_at: ::std::option::Option<i64>,
    ///Unix timestamp (seconds) for when the job was created.
    pub created_at: i64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: ::std::option::Option<Error2>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expires_at: ::std::option::Option<i64>,
    ///Unique identifier for the video job.
    pub id: String,
    ///The video generation model that produced the job.
    pub model: VideoModel,
    ///The object type, which is always `video`.
    pub object: String,
    ///Approximate completion percentage for the generation task.
    pub progress: i32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub prompt: ::std::option::Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub remixed_from_video_id: ::std::option::Option<String>,
    ///Duration of the generated clip in seconds. For extensions, this is the stitched total duration.
    pub seconds: String,
    ///The resolution of the generated video.
    pub size: VideoSize,
    ///Current lifecycle status of the video job.
    pub status: VideoStatus,
}
pub type VideoSeconds = String;
pub type VideoSize = String;
pub type VideoStatus = String;
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct VoiceConsentDeletedResource {
    pub deleted: bool,
    ///The consent recording identifier.
    pub id: String,
    pub object: String,
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct VoiceConsentListResource {
    pub data: Vec<VoiceConsentResource>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub first_id: ::std::option::Option<String>,
    pub has_more: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_id: ::std::option::Option<String>,
    pub object: String,
}
///A consent recording used to authorize creation of a custom voice.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct VoiceConsentResource {
    ///The Unix timestamp (in seconds) for when the consent recording was created.
    pub created_at: i64,
    ///The consent recording identifier.
    pub id: String,
    ///The BCP 47 language tag for the consent phrase (for example, `en-US`).
    pub language: String,
    ///The label provided when the consent recording was uploaded.
    pub name: String,
    ///The object type, which is always `audio.voice_consent`.
    pub object: String,
}
///A built-in voice name or a custom voice reference.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
#[serde(untagged)]
pub enum VoiceIdsOrCustomVoice {
    VoiceIdsShared(VoiceIdsShared),
    Object(VoiceIdsOrCustomVoiceObject),
}
///Custom voice reference.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct VoiceIdsOrCustomVoiceObject {
    ///The custom voice ID, e.g. `voice_1234`.
    pub id: String,
}
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
#[serde(untagged)]
pub enum VoiceIdsShared {
    String(String),
    String2(String),
}
///A custom voice that can be used for audio output.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct VoiceResource {
    ///The Unix timestamp (in seconds) for when the voice was created.
    pub created_at: i64,
    ///The voice identifier, which can be referenced in API endpoints.
    pub id: String,
    ///The name of the voice.
    pub name: String,
    ///The object type, which is always `audio.voice`.
    pub object: String,
}
///A wait action.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct WaitParam {
    ///Specifies the event type. For a wait action, this property is always set to `wait`.
    #[serde(rename = "type")]
    pub type_value: String,
}
///Action type "find_in_page": Searches for a pattern within a loaded page.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct WebSearchActionFind {
    ///The pattern or text to search for within the page.
    pub pattern: String,
    ///The action type.
    #[serde(rename = "type")]
    pub type_value: String,
    ///The URL of the page searched for the pattern.
    pub url: String,
}
///Action type "open_page" - Opens a specific URL from search results.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct WebSearchActionOpenPage {
    ///The action type.
    #[serde(rename = "type")]
    pub type_value: String,
    ///The URL opened by the model.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub url: ::std::option::Option<String>,
}
///Action type "search" - Performs a web search query.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct WebSearchActionSearch {
    ///The search queries.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub queries: ::std::option::Option<Vec<String>>,
    ///[DEPRECATED] The search query.
    pub query: String,
    ///The sources used in the search.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sources: ::std::option::Option<Vec<WebSearchActionSearchSource>>,
    ///The action type.
    #[serde(rename = "type")]
    pub type_value: String,
}
///A source used in the search.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct WebSearchActionSearchSource {
    ///The type of source. Always `url`.
    #[serde(rename = "type")]
    pub type_value: String,
    ///The URL of the source.
    pub url: String,
}
pub type WebSearchApproximateLocation = WebSearchApproximateLocation2;
///The approximate location of the user.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct WebSearchApproximateLocation2 {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub city: ::std::option::Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub country: ::std::option::Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub region: ::std::option::Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub timezone: ::std::option::Option<String>,
    ///The type of location approximation. Always `approximate`.
    #[serde(rename = "type", default, skip_serializing_if = "Option::is_none")]
    pub type_value: ::std::option::Option<String>,
}
///High level guidance for the amount of context window space to use for the search. One of `low`, `medium`, or `high`. `medium` is the default.
pub type WebSearchContextSize = String;
///Approximate location parameters for the search.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct WebSearchLocation {
    ///Free text input for the city of the user, e.g. `San Francisco`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub city: ::std::option::Option<String>,
    ///The two-letter [ISO country code](https://en.wikipedia.org/wiki/ISO_3166-1) of the user, e.g. `US`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub country: ::std::option::Option<String>,
    ///Free text input for the region of the user, e.g. `California`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub region: ::std::option::Option<String>,
    ///The [IANA timezone](https://timeapi.io/documentation/iana-timezones) of the user, e.g. `America/Los_Angeles`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub timezone: ::std::option::Option<String>,
}
///This tool searches the web for relevant results to use in a response. Learn more about the [web search tool](https://platform.openai.com/docs/guides/tools-web-search).
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct WebSearchPreviewTool {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub search_content_types: ::std::option::Option<Vec<SearchContentType>>,
    ///High level guidance for the amount of context window space to use for the search. One of `low`, `medium`, or `high`. `medium` is the default.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub search_context_size: ::std::option::Option<SearchContextSize>,
    ///The type of the web search tool. One of `web_search_preview` or `web_search_preview_2025_03_11`.
    #[serde(rename = "type")]
    pub type_value: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub user_location: ::std::option::Option<ApproximateLocation>,
}
///Search the Internet for sources related to the prompt. Learn more about the [web search tool](/docs/guides/tools-web-search).
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct WebSearchTool {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub filters: ::std::option::Option<WebSearchToolFilters>,
    ///High level guidance for the amount of context window space to use for the search. One of `low`, `medium`, or `high`. `medium` is the default.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub search_context_size: ::std::option::Option<String>,
    ///The type of the web search tool. One of `web_search` or `web_search_2025_08_26`.
    #[serde(rename = "type")]
    pub type_value: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub user_location: ::std::option::Option<WebSearchApproximateLocation>,
}
///The results of a web search tool call. See the [web search guide](/docs/guides/tools-web-search) for more information.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct WebSearchToolCall {
    ///An object describing the specific action taken in this web search call. Includes details on how the model used the web (search, open_page, find_in_page).
    pub action: WebSearchToolCallAction,
    ///The unique ID of the web search tool call.
    pub id: String,
    ///The status of the web search tool call.
    pub status: String,
    ///The type of the web search tool call. Always `web_search_call`.
    #[serde(rename = "type")]
    pub type_value: String,
}
///An object describing the specific action taken in this web search call. Includes details on how the model used the web (search, open_page, find_in_page).
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
#[serde(untagged)]
pub enum WebSearchToolCallAction {
    WebSearchActionSearch(WebSearchActionSearch),
    WebSearchActionOpenPage(WebSearchActionOpenPage),
    WebSearchActionFind(WebSearchActionFind),
}
///Filters for the search.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct WebSearchToolFilters {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub allowed_domains: ::std::option::Option<Vec<String>>,
}
///Sent when a batch API request has been cancelled.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct WebhookBatchCancelled {
    ///The Unix timestamp (in seconds) of when the batch API request was cancelled.
    pub created_at: i64,
    ///Event data payload.
    pub data: WebhookBatchCancelledData,
    ///The unique ID of the event.
    pub id: String,
    ///The object of the event. Always `event`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub object: ::std::option::Option<String>,
    ///The type of the event. Always `batch.cancelled`.
    #[serde(rename = "type")]
    pub type_value: String,
}
///Event data payload.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct WebhookBatchCancelledData {
    ///The unique ID of the batch API request.
    pub id: String,
}
///Sent when a batch API request has been completed.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct WebhookBatchCompleted {
    ///The Unix timestamp (in seconds) of when the batch API request was completed.
    pub created_at: i64,
    ///Event data payload.
    pub data: WebhookBatchCompletedData,
    ///The unique ID of the event.
    pub id: String,
    ///The object of the event. Always `event`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub object: ::std::option::Option<String>,
    ///The type of the event. Always `batch.completed`.
    #[serde(rename = "type")]
    pub type_value: String,
}
///Event data payload.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct WebhookBatchCompletedData {
    ///The unique ID of the batch API request.
    pub id: String,
}
///Sent when a batch API request has expired.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct WebhookBatchExpired {
    ///The Unix timestamp (in seconds) of when the batch API request expired.
    pub created_at: i64,
    ///Event data payload.
    pub data: WebhookBatchExpiredData,
    ///The unique ID of the event.
    pub id: String,
    ///The object of the event. Always `event`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub object: ::std::option::Option<String>,
    ///The type of the event. Always `batch.expired`.
    #[serde(rename = "type")]
    pub type_value: String,
}
///Event data payload.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct WebhookBatchExpiredData {
    ///The unique ID of the batch API request.
    pub id: String,
}
///Sent when a batch API request has failed.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct WebhookBatchFailed {
    ///The Unix timestamp (in seconds) of when the batch API request failed.
    pub created_at: i64,
    ///Event data payload.
    pub data: WebhookBatchFailedData,
    ///The unique ID of the event.
    pub id: String,
    ///The object of the event. Always `event`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub object: ::std::option::Option<String>,
    ///The type of the event. Always `batch.failed`.
    #[serde(rename = "type")]
    pub type_value: String,
}
///Event data payload.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct WebhookBatchFailedData {
    ///The unique ID of the batch API request.
    pub id: String,
}
///Sent when an eval run has been canceled.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct WebhookEvalRunCanceled {
    ///The Unix timestamp (in seconds) of when the eval run was canceled.
    pub created_at: i64,
    ///Event data payload.
    pub data: WebhookEvalRunCanceledData,
    ///The unique ID of the event.
    pub id: String,
    ///The object of the event. Always `event`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub object: ::std::option::Option<String>,
    ///The type of the event. Always `eval.run.canceled`.
    #[serde(rename = "type")]
    pub type_value: String,
}
///Event data payload.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct WebhookEvalRunCanceledData {
    ///The unique ID of the eval run.
    pub id: String,
}
///Sent when an eval run has failed.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct WebhookEvalRunFailed {
    ///The Unix timestamp (in seconds) of when the eval run failed.
    pub created_at: i64,
    ///Event data payload.
    pub data: WebhookEvalRunFailedData,
    ///The unique ID of the event.
    pub id: String,
    ///The object of the event. Always `event`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub object: ::std::option::Option<String>,
    ///The type of the event. Always `eval.run.failed`.
    #[serde(rename = "type")]
    pub type_value: String,
}
///Event data payload.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct WebhookEvalRunFailedData {
    ///The unique ID of the eval run.
    pub id: String,
}
///Sent when an eval run has succeeded.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct WebhookEvalRunSucceeded {
    ///The Unix timestamp (in seconds) of when the eval run succeeded.
    pub created_at: i64,
    ///Event data payload.
    pub data: WebhookEvalRunSucceededData,
    ///The unique ID of the event.
    pub id: String,
    ///The object of the event. Always `event`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub object: ::std::option::Option<String>,
    ///The type of the event. Always `eval.run.succeeded`.
    #[serde(rename = "type")]
    pub type_value: String,
}
///Event data payload.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct WebhookEvalRunSucceededData {
    ///The unique ID of the eval run.
    pub id: String,
}
///Sent when a fine-tuning job has been cancelled.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct WebhookFineTuningJobCancelled {
    ///The Unix timestamp (in seconds) of when the fine-tuning job was cancelled.
    pub created_at: i64,
    ///Event data payload.
    pub data: WebhookFineTuningJobCancelledData,
    ///The unique ID of the event.
    pub id: String,
    ///The object of the event. Always `event`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub object: ::std::option::Option<String>,
    ///The type of the event. Always `fine_tuning.job.cancelled`.
    #[serde(rename = "type")]
    pub type_value: String,
}
///Event data payload.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct WebhookFineTuningJobCancelledData {
    ///The unique ID of the fine-tuning job.
    pub id: String,
}
///Sent when a fine-tuning job has failed.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct WebhookFineTuningJobFailed {
    ///The Unix timestamp (in seconds) of when the fine-tuning job failed.
    pub created_at: i64,
    ///Event data payload.
    pub data: WebhookFineTuningJobFailedData,
    ///The unique ID of the event.
    pub id: String,
    ///The object of the event. Always `event`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub object: ::std::option::Option<String>,
    ///The type of the event. Always `fine_tuning.job.failed`.
    #[serde(rename = "type")]
    pub type_value: String,
}
///Event data payload.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct WebhookFineTuningJobFailedData {
    ///The unique ID of the fine-tuning job.
    pub id: String,
}
///Sent when a fine-tuning job has succeeded.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct WebhookFineTuningJobSucceeded {
    ///The Unix timestamp (in seconds) of when the fine-tuning job succeeded.
    pub created_at: i64,
    ///Event data payload.
    pub data: WebhookFineTuningJobSucceededData,
    ///The unique ID of the event.
    pub id: String,
    ///The object of the event. Always `event`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub object: ::std::option::Option<String>,
    ///The type of the event. Always `fine_tuning.job.succeeded`.
    #[serde(rename = "type")]
    pub type_value: String,
}
///Event data payload.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct WebhookFineTuningJobSucceededData {
    ///The unique ID of the fine-tuning job.
    pub id: String,
}
///Sent when Realtime API Receives a incoming SIP call.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct WebhookRealtimeCallIncoming {
    ///The Unix timestamp (in seconds) of when the model response was completed.
    pub created_at: i64,
    ///Event data payload.
    pub data: WebhookRealtimeCallIncomingData,
    ///The unique ID of the event.
    pub id: String,
    ///The object of the event. Always `event`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub object: ::std::option::Option<String>,
    ///The type of the event. Always `realtime.call.incoming`.
    #[serde(rename = "type")]
    pub type_value: String,
}
///Event data payload.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct WebhookRealtimeCallIncomingData {
    ///The unique ID of this call.
    pub call_id: String,
    ///Headers from the SIP Invite.
    pub sip_headers: Vec<WebhookRealtimeCallIncomingDataSipHeader>,
}
///A header from the SIP Invite.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct WebhookRealtimeCallIncomingDataSipHeader {
    ///Name of the SIP Header.
    pub name: String,
    ///Value of the SIP Header.
    pub value: String,
}
///Sent when a background response has been cancelled.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct WebhookResponseCancelled {
    ///The Unix timestamp (in seconds) of when the model response was cancelled.
    pub created_at: i64,
    ///Event data payload.
    pub data: WebhookResponseCancelledData,
    ///The unique ID of the event.
    pub id: String,
    ///The object of the event. Always `event`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub object: ::std::option::Option<String>,
    ///The type of the event. Always `response.cancelled`.
    #[serde(rename = "type")]
    pub type_value: String,
}
///Event data payload.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct WebhookResponseCancelledData {
    ///The unique ID of the model response.
    pub id: String,
}
///Sent when a background response has been completed.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct WebhookResponseCompleted {
    ///The Unix timestamp (in seconds) of when the model response was completed.
    pub created_at: i64,
    ///Event data payload.
    pub data: WebhookResponseCompletedData,
    ///The unique ID of the event.
    pub id: String,
    ///The object of the event. Always `event`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub object: ::std::option::Option<String>,
    ///The type of the event. Always `response.completed`.
    #[serde(rename = "type")]
    pub type_value: String,
}
///Event data payload.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct WebhookResponseCompletedData {
    ///The unique ID of the model response.
    pub id: String,
}
///Sent when a background response has failed.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct WebhookResponseFailed {
    ///The Unix timestamp (in seconds) of when the model response failed.
    pub created_at: i64,
    ///Event data payload.
    pub data: WebhookResponseFailedData,
    ///The unique ID of the event.
    pub id: String,
    ///The object of the event. Always `event`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub object: ::std::option::Option<String>,
    ///The type of the event. Always `response.failed`.
    #[serde(rename = "type")]
    pub type_value: String,
}
///Event data payload.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct WebhookResponseFailedData {
    ///The unique ID of the model response.
    pub id: String,
}
///Sent when a background response has been interrupted.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct WebhookResponseIncomplete {
    ///The Unix timestamp (in seconds) of when the model response was interrupted.
    pub created_at: i64,
    ///Event data payload.
    pub data: WebhookResponseIncompleteData,
    ///The unique ID of the event.
    pub id: String,
    ///The object of the event. Always `event`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub object: ::std::option::Option<String>,
    ///The type of the event. Always `response.incomplete`.
    #[serde(rename = "type")]
    pub type_value: String,
}
///Event data payload.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct WebhookResponseIncompleteData {
    ///The unique ID of the model response.
    pub id: String,
}
///Thread item that renders a widget payload.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct WidgetMessageItem {
    ///Unix timestamp (in seconds) for when the item was created.
    pub created_at: i64,
    ///Identifier of the thread item.
    pub id: String,
    ///Type discriminator that is always `chatkit.thread_item`.
    pub object: String,
    ///Identifier of the parent thread.
    pub thread_id: String,
    ///Type discriminator that is always `chatkit.widget`.
    #[serde(rename = "type")]
    pub type_value: String,
    ///Serialized widget payload rendered in the UI.
    pub widget: String,
}
///Workflow reference and overrides applied to the chat session.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct WorkflowParam {
    ///Identifier for the workflow invoked by the session.
    pub id: String,
    ///State variables forwarded to the workflow. Keys may be up to 64 characters, values must be primitive types, and the map defaults to an empty object.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub state_variables: ::std::option::Option<OpenAiJsonValue>,
    ///Optional tracing overrides for the workflow invocation. When omitted, tracing is enabled by default.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tracing: ::std::option::Option<WorkflowTracingParam>,
    ///Specific workflow version to run. Defaults to the latest deployed version.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub version: ::std::option::Option<String>,
}
///Controls diagnostic tracing during the session.
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct WorkflowTracingParam {
    ///Whether tracing is enabled during the session. Defaults to true.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub enabled: ::std::option::Option<bool>,
}
