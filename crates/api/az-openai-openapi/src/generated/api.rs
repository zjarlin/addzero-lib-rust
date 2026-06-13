use super::bodies::*;
use super::models::*;
///AdminApiKeys REST endpoints.
#[::async_trait::async_trait]
pub trait OpenAiAdminApiKeysApi: Send + Sync {
    type Error: ::std::error::Error + Send + Sync + 'static;
    ///List organization API keys
    ///
    ///REST: `GET /organization/admin_api_keys`.
    ///Path constant: `OpenAiApiPath::ORGANIZATION_BY_ADMIN_API_KEYS`.
    async fn admin_api_keys_list(
        &self,
        after: ::std::option::Option<String>,
        order: ::std::option::Option<String>,
        limit: ::std::option::Option<i32>,
    ) -> Result<ApiKeyList, Self::Error>;
    ///Create an organization admin API key
    ///
    ///REST: `POST /organization/admin_api_keys`.
    ///Path constant: `OpenAiApiPath::ORGANIZATION_BY_ADMIN_API_KEYS`.
    async fn admin_api_keys_create(
        &self,
        body: AdminApiKeysCreateRequest,
    ) -> Result<AdminApiKeyCreateResponse, Self::Error>;
    ///Retrieve a single organization API key
    ///
    ///REST: `GET /organization/admin_api_keys/{key_id}`.
    ///Path constant: `OpenAiApiPath::ORGANIZATION_BY_ADMIN_API_KEYS_BY_KEY_ID`.
    async fn admin_api_keys_get(
        &self,
        key_id: String,
    ) -> Result<AdminApiKey, Self::Error>;
    ///Delete an organization admin API key
    ///
    ///REST: `DELETE /organization/admin_api_keys/{key_id}`.
    ///Path constant: `OpenAiApiPath::ORGANIZATION_BY_ADMIN_API_KEYS_BY_KEY_ID`.
    async fn admin_api_keys_delete(
        &self,
        key_id: String,
    ) -> Result<AdminApiKeysDeleteResponse, Self::Error>;
}
///Assistants REST endpoints.
#[::async_trait::async_trait]
pub trait OpenAiAssistantsApi: Send + Sync {
    type Error: ::std::error::Error + Send + Sync + 'static;
    ///Returns a list of assistants.
    ///
    ///REST: `GET /assistants`.
    ///Path constant: `OpenAiApiPath::ASSISTANTS`.
    async fn list_assistants(
        &self,
        limit: ::std::option::Option<i32>,
        order: ::std::option::Option<String>,
        after: ::std::option::Option<String>,
        before: ::std::option::Option<String>,
    ) -> Result<ListAssistantsResponse, Self::Error>;
    ///Create an assistant with a model and instructions.
    ///
    ///REST: `POST /assistants`.
    ///Path constant: `OpenAiApiPath::ASSISTANTS`.
    async fn create_assistant(
        &self,
        body: CreateAssistantRequest,
    ) -> Result<AssistantObject, Self::Error>;
    ///Retrieves an assistant.
    ///
    ///REST: `GET /assistants/{assistant_id}`.
    ///Path constant: `OpenAiApiPath::ASSISTANTS_BY_ASSISTANT_ID`.
    async fn get_assistant(
        &self,
        assistant_id: String,
    ) -> Result<AssistantObject, Self::Error>;
    ///Modifies an assistant.
    ///
    ///REST: `POST /assistants/{assistant_id}`.
    ///Path constant: `OpenAiApiPath::ASSISTANTS_BY_ASSISTANT_ID`.
    async fn modify_assistant(
        &self,
        assistant_id: String,
        body: ModifyAssistantRequest,
    ) -> Result<AssistantObject, Self::Error>;
    ///Delete an assistant.
    ///
    ///REST: `DELETE /assistants/{assistant_id}`.
    ///Path constant: `OpenAiApiPath::ASSISTANTS_BY_ASSISTANT_ID`.
    async fn delete_assistant(
        &self,
        assistant_id: String,
    ) -> Result<DeleteAssistantResponse, Self::Error>;
    ///Create a thread.
    ///
    ///REST: `POST /threads`.
    ///Path constant: `OpenAiApiPath::THREADS`.
    async fn create_thread(
        &self,
        body: ::std::option::Option<CreateThreadRequest>,
    ) -> Result<ThreadObject, Self::Error>;
    ///Create a thread and run it in one request.
    ///
    ///REST: `POST /threads/runs`.
    ///Path constant: `OpenAiApiPath::THREADS_BY_RUNS`.
    async fn create_thread_and_run(
        &self,
        body: CreateThreadAndRunRequest,
    ) -> Result<RunObject, Self::Error>;
    ///Retrieves a thread.
    ///
    ///REST: `GET /threads/{thread_id}`.
    ///Path constant: `OpenAiApiPath::THREADS_BY_THREAD_ID`.
    async fn get_thread(&self, thread_id: String) -> Result<ThreadObject, Self::Error>;
    ///Modifies a thread.
    ///
    ///REST: `POST /threads/{thread_id}`.
    ///Path constant: `OpenAiApiPath::THREADS_BY_THREAD_ID`.
    async fn modify_thread(
        &self,
        thread_id: String,
        body: ModifyThreadRequest,
    ) -> Result<ThreadObject, Self::Error>;
    ///Delete a thread.
    ///
    ///REST: `DELETE /threads/{thread_id}`.
    ///Path constant: `OpenAiApiPath::THREADS_BY_THREAD_ID`.
    async fn delete_thread(
        &self,
        thread_id: String,
    ) -> Result<DeleteThreadResponse, Self::Error>;
    ///Returns a list of messages for a given thread.
    ///
    ///REST: `GET /threads/{thread_id}/messages`.
    ///Path constant: `OpenAiApiPath::THREADS_BY_THREAD_ID_BY_MESSAGES`.
    async fn list_messages(
        &self,
        thread_id: String,
        limit: ::std::option::Option<i32>,
        order: ::std::option::Option<String>,
        after: ::std::option::Option<String>,
        before: ::std::option::Option<String>,
        run_id: ::std::option::Option<String>,
    ) -> Result<ListMessagesResponse, Self::Error>;
    ///Create a message.
    ///
    ///REST: `POST /threads/{thread_id}/messages`.
    ///Path constant: `OpenAiApiPath::THREADS_BY_THREAD_ID_BY_MESSAGES`.
    async fn create_message(
        &self,
        thread_id: String,
        body: CreateMessageRequest,
    ) -> Result<MessageObject, Self::Error>;
    ///Retrieve a message.
    ///
    ///REST: `GET /threads/{thread_id}/messages/{message_id}`.
    ///Path constant: `OpenAiApiPath::THREADS_BY_THREAD_ID_BY_MESSAGES_BY_MESSAGE_ID`.
    async fn get_message(
        &self,
        thread_id: String,
        message_id: String,
    ) -> Result<MessageObject, Self::Error>;
    ///Modifies a message.
    ///
    ///REST: `POST /threads/{thread_id}/messages/{message_id}`.
    ///Path constant: `OpenAiApiPath::THREADS_BY_THREAD_ID_BY_MESSAGES_BY_MESSAGE_ID`.
    async fn modify_message(
        &self,
        thread_id: String,
        message_id: String,
        body: ModifyMessageRequest,
    ) -> Result<MessageObject, Self::Error>;
    ///Deletes a message.
    ///
    ///REST: `DELETE /threads/{thread_id}/messages/{message_id}`.
    ///Path constant: `OpenAiApiPath::THREADS_BY_THREAD_ID_BY_MESSAGES_BY_MESSAGE_ID`.
    async fn delete_message(
        &self,
        thread_id: String,
        message_id: String,
    ) -> Result<DeleteMessageResponse, Self::Error>;
    ///Returns a list of runs belonging to a thread.
    ///
    ///REST: `GET /threads/{thread_id}/runs`.
    ///Path constant: `OpenAiApiPath::THREADS_BY_THREAD_ID_BY_RUNS`.
    async fn list_runs(
        &self,
        thread_id: String,
        limit: ::std::option::Option<i32>,
        order: ::std::option::Option<String>,
        after: ::std::option::Option<String>,
        before: ::std::option::Option<String>,
    ) -> Result<ListRunsResponse, Self::Error>;
    ///Create a run.
    ///
    ///REST: `POST /threads/{thread_id}/runs`.
    ///Path constant: `OpenAiApiPath::THREADS_BY_THREAD_ID_BY_RUNS`.
    async fn create_run(
        &self,
        thread_id: String,
        include__: ::std::option::Option<Vec<String>>,
        body: CreateRunRequest,
    ) -> Result<RunObject, Self::Error>;
    ///Retrieves a run.
    ///
    ///REST: `GET /threads/{thread_id}/runs/{run_id}`.
    ///Path constant: `OpenAiApiPath::THREADS_BY_THREAD_ID_BY_RUNS_BY_RUN_ID`.
    async fn get_run(
        &self,
        thread_id: String,
        run_id: String,
    ) -> Result<RunObject, Self::Error>;
    ///Modifies a run.
    ///
    ///REST: `POST /threads/{thread_id}/runs/{run_id}`.
    ///Path constant: `OpenAiApiPath::THREADS_BY_THREAD_ID_BY_RUNS_BY_RUN_ID`.
    async fn modify_run(
        &self,
        thread_id: String,
        run_id: String,
        body: ModifyRunRequest,
    ) -> Result<RunObject, Self::Error>;
    ///Cancels a run that is `in_progress`.
    ///
    ///REST: `POST /threads/{thread_id}/runs/{run_id}/cancel`.
    ///Path constant: `OpenAiApiPath::THREADS_BY_THREAD_ID_BY_RUNS_BY_RUN_ID_BY_CANCEL`.
    async fn cancel_run(
        &self,
        thread_id: String,
        run_id: String,
    ) -> Result<RunObject, Self::Error>;
    ///Returns a list of run steps belonging to a run.
    ///
    ///REST: `GET /threads/{thread_id}/runs/{run_id}/steps`.
    ///Path constant: `OpenAiApiPath::THREADS_BY_THREAD_ID_BY_RUNS_BY_RUN_ID_BY_STEPS`.
    async fn list_run_steps(
        &self,
        thread_id: String,
        run_id: String,
        limit: ::std::option::Option<i32>,
        order: ::std::option::Option<String>,
        after: ::std::option::Option<String>,
        before: ::std::option::Option<String>,
        include__: ::std::option::Option<Vec<String>>,
    ) -> Result<ListRunStepsResponse, Self::Error>;
    ///Retrieves a run step.
    ///
    ///REST: `GET /threads/{thread_id}/runs/{run_id}/steps/{step_id}`.
    ///Path constant: `OpenAiApiPath::THREADS_BY_THREAD_ID_BY_RUNS_BY_RUN_ID_BY_STEPS_BY_STEP_ID`.
    async fn get_run_step(
        &self,
        thread_id: String,
        run_id: String,
        step_id: String,
        include__: ::std::option::Option<Vec<String>>,
    ) -> Result<RunStepObject, Self::Error>;
    ///When a run has the `status: "requires_action"` and `required_action.type` is `submit_tool_outputs`, this endpoint can be used to submit the outputs from the tool calls once they're all completed. All outputs must be submitted in a single request.
    ///
    ///REST: `POST /threads/{thread_id}/runs/{run_id}/submit_tool_outputs`.
    ///Path constant: `OpenAiApiPath::THREADS_BY_THREAD_ID_BY_RUNS_BY_RUN_ID_BY_SUBMIT_TOOL_OUTPUTS`.
    async fn submit_tool_ouputs_to_run(
        &self,
        thread_id: String,
        run_id: String,
        body: SubmitToolOutputsRunRequest,
    ) -> Result<RunObject, Self::Error>;
}
///Audio REST endpoints.
#[::async_trait::async_trait]
pub trait OpenAiAudioApi: Send + Sync {
    type Error: ::std::error::Error + Send + Sync + 'static;
    ///Generates audio from the input text. Returns the audio file content, or a stream of audio events.
    ///
    ///REST: `POST /audio/speech`.
    ///Path constant: `OpenAiApiPath::AUDIO_BY_SPEECH`.
    async fn create_speech(
        &self,
        body: CreateSpeechRequest,
    ) -> Result<OpenAiBinaryBody, Self::Error>;
    ///Transcribes audio into the input language. Returns a transcription object in `json`, `diarized_json`, or `verbose_json` format, or a stream of transcript events.
    ///
    ///REST: `POST /audio/transcriptions`.
    ///Path constant: `OpenAiApiPath::AUDIO_BY_TRANSCRIPTIONS`.
    async fn create_transcription(
        &self,
        body: CreateTranscriptionRequest,
    ) -> Result<CreateTranscriptionResponse, Self::Error>;
    ///Translates audio into English.
    ///
    ///REST: `POST /audio/translations`.
    ///Path constant: `OpenAiApiPath::AUDIO_BY_TRANSLATIONS`.
    async fn create_translation(
        &self,
        body: CreateTranslationRequest,
    ) -> Result<CreateTranslationResponse, Self::Error>;
    ///Returns a list of voice consent recordings.
    ///
    ///REST: `GET /audio/voice_consents`.
    ///Path constant: `OpenAiApiPath::AUDIO_BY_VOICE_CONSENTS`.
    async fn list_voice_consents(
        &self,
        after: ::std::option::Option<String>,
        limit: ::std::option::Option<i32>,
    ) -> Result<VoiceConsentListResource, Self::Error>;
    ///Upload a voice consent recording.
    ///
    ///REST: `POST /audio/voice_consents`.
    ///Path constant: `OpenAiApiPath::AUDIO_BY_VOICE_CONSENTS`.
    async fn create_voice_consent(
        &self,
        body: CreateVoiceConsentRequest,
    ) -> Result<VoiceConsentResource, Self::Error>;
    ///Retrieves a voice consent recording.
    ///
    ///REST: `GET /audio/voice_consents/{consent_id}`.
    ///Path constant: `OpenAiApiPath::AUDIO_BY_VOICE_CONSENTS_BY_CONSENT_ID`.
    async fn get_voice_consent(
        &self,
        consent_id: String,
    ) -> Result<VoiceConsentResource, Self::Error>;
    ///Updates a voice consent recording (metadata only).
    ///
    ///REST: `POST /audio/voice_consents/{consent_id}`.
    ///Path constant: `OpenAiApiPath::AUDIO_BY_VOICE_CONSENTS_BY_CONSENT_ID`.
    async fn update_voice_consent(
        &self,
        consent_id: String,
        body: UpdateVoiceConsentRequest,
    ) -> Result<VoiceConsentResource, Self::Error>;
    ///Deletes a voice consent recording.
    ///
    ///REST: `DELETE /audio/voice_consents/{consent_id}`.
    ///Path constant: `OpenAiApiPath::AUDIO_BY_VOICE_CONSENTS_BY_CONSENT_ID`.
    async fn delete_voice_consent(
        &self,
        consent_id: String,
    ) -> Result<VoiceConsentDeletedResource, Self::Error>;
    ///Creates a custom voice.
    ///
    ///REST: `POST /audio/voices`.
    ///Path constant: `OpenAiApiPath::AUDIO_BY_VOICES`.
    async fn create_voice(
        &self,
        body: CreateVoiceRequest,
    ) -> Result<VoiceResource, Self::Error>;
}
///AuditLogs REST endpoints.
#[::async_trait::async_trait]
pub trait OpenAiAuditLogsApi: Send + Sync {
    type Error: ::std::error::Error + Send + Sync + 'static;
    ///List user actions and configuration changes within this organization.
    ///
    ///REST: `GET /organization/audit_logs`.
    ///Path constant: `OpenAiApiPath::ORGANIZATION_BY_AUDIT_LOGS`.
    async fn list_audit_logs(
        &self,
        effective_at: ::std::option::Option<EffectiveAtParameter>,
        project_ids__: ::std::option::Option<Vec<String>>,
        event_types__: ::std::option::Option<Vec<AuditLogEventType>>,
        actor_ids__: ::std::option::Option<Vec<String>>,
        actor_emails__: ::std::option::Option<Vec<String>>,
        resource_ids__: ::std::option::Option<Vec<String>>,
        limit: ::std::option::Option<i32>,
        after: ::std::option::Option<String>,
        before: ::std::option::Option<String>,
    ) -> Result<ListAuditLogsResponse, Self::Error>;
}
///Batch REST endpoints.
#[::async_trait::async_trait]
pub trait OpenAiBatchApi: Send + Sync {
    type Error: ::std::error::Error + Send + Sync + 'static;
    ///List your organization's batches.
    ///
    ///REST: `GET /batches`.
    ///Path constant: `OpenAiApiPath::BATCHES`.
    async fn list_batches(
        &self,
        after: ::std::option::Option<String>,
        limit: ::std::option::Option<i32>,
    ) -> Result<ListBatchesResponse, Self::Error>;
    ///Creates and executes a batch from an uploaded file of requests
    ///
    ///REST: `POST /batches`.
    ///Path constant: `OpenAiApiPath::BATCHES`.
    async fn create_batch(&self, body: CreateBatchRequest) -> Result<Batch, Self::Error>;
    ///Retrieves a batch.
    ///
    ///REST: `GET /batches/{batch_id}`.
    ///Path constant: `OpenAiApiPath::BATCHES_BY_BATCH_ID`.
    async fn retrieve_batch(&self, batch_id: String) -> Result<Batch, Self::Error>;
    ///Cancels an in-progress batch. The batch will be in status `cancelling` for up to 10 minutes, before changing to `cancelled`, where it will have partial results (if any) available in the output file.
    ///
    ///REST: `POST /batches/{batch_id}/cancel`.
    ///Path constant: `OpenAiApiPath::BATCHES_BY_BATCH_ID_BY_CANCEL`.
    async fn cancel_batch(&self, batch_id: String) -> Result<Batch, Self::Error>;
}
///Certificates REST endpoints.
#[::async_trait::async_trait]
pub trait OpenAiCertificatesApi: Send + Sync {
    type Error: ::std::error::Error + Send + Sync + 'static;
    ///List uploaded certificates for this organization.
    ///
    ///REST: `GET /organization/certificates`.
    ///Path constant: `OpenAiApiPath::ORGANIZATION_BY_CERTIFICATES`.
    async fn list_organization_certificates(
        &self,
        limit: ::std::option::Option<i32>,
        after: ::std::option::Option<String>,
        order: ::std::option::Option<String>,
    ) -> Result<ListCertificatesResponse, Self::Error>;
    ///Upload a certificate to the organization. This does **not** automatically activate the certificate. Organizations can upload up to 50 certificates.
    ///
    ///REST: `POST /organization/certificates`.
    ///Path constant: `OpenAiApiPath::ORGANIZATION_BY_CERTIFICATES`.
    async fn upload_certificate(
        &self,
        body: UploadCertificateRequest,
    ) -> Result<Certificate, Self::Error>;
    ///Activate certificates at the organization level. You can atomically and idempotently activate up to 10 certificates at a time.
    ///
    ///REST: `POST /organization/certificates/activate`.
    ///Path constant: `OpenAiApiPath::ORGANIZATION_BY_CERTIFICATES_BY_ACTIVATE`.
    async fn activate_organization_certificates(
        &self,
        body: ToggleCertificatesRequest,
    ) -> Result<OrganizationCertificateActivationResponse, Self::Error>;
    ///Deactivate certificates at the organization level. You can atomically and idempotently deactivate up to 10 certificates at a time.
    ///
    ///REST: `POST /organization/certificates/deactivate`.
    ///Path constant: `OpenAiApiPath::ORGANIZATION_BY_CERTIFICATES_BY_DEACTIVATE`.
    async fn deactivate_organization_certificates(
        &self,
        body: ToggleCertificatesRequest,
    ) -> Result<OrganizationCertificateDeactivationResponse, Self::Error>;
    ///Get a certificate that has been uploaded to the organization. You can get a certificate regardless of whether it is active or not.
    ///
    ///REST: `GET /organization/certificates/{certificate_id}`.
    ///Path constant: `OpenAiApiPath::ORGANIZATION_BY_CERTIFICATES_BY_CERTIFICATE_ID`.
    async fn get_certificate(
        &self,
        certificate_id: String,
        include: ::std::option::Option<Vec<String>>,
    ) -> Result<Certificate, Self::Error>;
    ///Modify a certificate. Note that only the name can be modified.
    ///
    ///REST: `POST /organization/certificates/{certificate_id}`.
    ///Path constant: `OpenAiApiPath::ORGANIZATION_BY_CERTIFICATES_BY_CERTIFICATE_ID`.
    async fn modify_certificate(
        &self,
        certificate_id: String,
        body: ModifyCertificateRequest,
    ) -> Result<Certificate, Self::Error>;
    ///Delete a certificate from the organization. The certificate must be inactive for the organization and all projects.
    ///
    ///REST: `DELETE /organization/certificates/{certificate_id}`.
    ///Path constant: `OpenAiApiPath::ORGANIZATION_BY_CERTIFICATES_BY_CERTIFICATE_ID`.
    async fn delete_certificate(
        &self,
        certificate_id: String,
    ) -> Result<DeleteCertificateResponse, Self::Error>;
    ///List certificates for this project.
    ///
    ///REST: `GET /organization/projects/{project_id}/certificates`.
    ///Path constant: `OpenAiApiPath::ORGANIZATION_BY_PROJECTS_BY_PROJECT_ID_BY_CERTIFICATES`.
    async fn list_project_certificates(
        &self,
        project_id: String,
        limit: ::std::option::Option<i32>,
        after: ::std::option::Option<String>,
        order: ::std::option::Option<String>,
    ) -> Result<ListProjectCertificatesResponse, Self::Error>;
    ///Activate certificates at the project level. You can atomically and idempotently activate up to 10 certificates at a time.
    ///
    ///REST: `POST /organization/projects/{project_id}/certificates/activate`.
    ///Path constant: `OpenAiApiPath::ORGANIZATION_BY_PROJECTS_BY_PROJECT_ID_BY_CERTIFICATES_BY_ACTIVATE`.
    async fn activate_project_certificates(
        &self,
        project_id: String,
        body: ToggleCertificatesRequest,
    ) -> Result<OrganizationProjectCertificateActivationResponse, Self::Error>;
    ///Deactivate certificates at the project level. You can atomically and idempotently deactivate up to 10 certificates at a time.
    ///
    ///REST: `POST /organization/projects/{project_id}/certificates/deactivate`.
    ///Path constant: `OpenAiApiPath::ORGANIZATION_BY_PROJECTS_BY_PROJECT_ID_BY_CERTIFICATES_BY_DEACTIVATE`.
    async fn deactivate_project_certificates(
        &self,
        project_id: String,
        body: ToggleCertificatesRequest,
    ) -> Result<OrganizationProjectCertificateDeactivationResponse, Self::Error>;
}
///Chat REST endpoints.
#[::async_trait::async_trait]
pub trait OpenAiChatApi: Send + Sync {
    type Error: ::std::error::Error + Send + Sync + 'static;
    ///List stored Chat Completions. Only Chat Completions that have been stored with the `store` parameter set to `true` will be returned.
    ///
    ///REST: `GET /chat/completions`.
    ///Path constant: `OpenAiApiPath::CHAT_BY_COMPLETIONS`.
    async fn list_chat_completions(
        &self,
        model: ::std::option::Option<String>,
        metadata: ::std::option::Option<Metadata>,
        after: ::std::option::Option<String>,
        limit: ::std::option::Option<i32>,
        order: ::std::option::Option<String>,
    ) -> Result<ChatCompletionList, Self::Error>;
    ///**Starting a new project?** We recommend trying [Responses](/docs/api-reference/responses) to take advantage of the latest OpenAI platform features. Compare [Chat Completions with Responses](/docs/guides/responses-vs-chat-completions?api-mode=responses). --- Creates a model response for the given chat conversation. Learn more in the [text generation](/docs/guides/text-generation), [vision](/docs/guides/vision), and [audio](/docs/guides/audio) guides. Parameter support can differ depending on the model used to generate the response, particularly for newer reasoning models. Parameters that are only supported for reasoning models are noted below. For the current state of unsupported parameters in reasoning models, [refer to the reasoning guide](/docs/guides/reasoning). Returns a chat completion object, or a streamed sequence of chat completion chunk objects if the request is streamed.
    ///
    ///REST: `POST /chat/completions`.
    ///Path constant: `OpenAiApiPath::CHAT_BY_COMPLETIONS`.
    async fn create_chat_completion(
        &self,
        body: CreateChatCompletionRequest,
    ) -> Result<CreateChatCompletionResponse, Self::Error>;
    ///Get a stored chat completion. Only Chat Completions that have been created with the `store` parameter set to `true` will be returned.
    ///
    ///REST: `GET /chat/completions/{completion_id}`.
    ///Path constant: `OpenAiApiPath::CHAT_BY_COMPLETIONS_BY_COMPLETION_ID`.
    async fn get_chat_completion(
        &self,
        completion_id: String,
    ) -> Result<CreateChatCompletionResponse, Self::Error>;
    ///Modify a stored chat completion. Only Chat Completions that have been created with the `store` parameter set to `true` can be modified. Currently, the only supported modification is to update the `metadata` field.
    ///
    ///REST: `POST /chat/completions/{completion_id}`.
    ///Path constant: `OpenAiApiPath::CHAT_BY_COMPLETIONS_BY_COMPLETION_ID`.
    async fn update_chat_completion(
        &self,
        completion_id: String,
        body: UpdateChatCompletionRequest,
    ) -> Result<CreateChatCompletionResponse, Self::Error>;
    ///Delete a stored chat completion. Only Chat Completions that have been created with the `store` parameter set to `true` can be deleted.
    ///
    ///REST: `DELETE /chat/completions/{completion_id}`.
    ///Path constant: `OpenAiApiPath::CHAT_BY_COMPLETIONS_BY_COMPLETION_ID`.
    async fn delete_chat_completion(
        &self,
        completion_id: String,
    ) -> Result<ChatCompletionDeleted, Self::Error>;
    ///Get the messages in a stored chat completion. Only Chat Completions that have been created with the `store` parameter set to `true` will be returned.
    ///
    ///REST: `GET /chat/completions/{completion_id}/messages`.
    ///Path constant: `OpenAiApiPath::CHAT_BY_COMPLETIONS_BY_COMPLETION_ID_BY_MESSAGES`.
    async fn get_chat_completion_messages(
        &self,
        completion_id: String,
        after: ::std::option::Option<String>,
        limit: ::std::option::Option<i32>,
        order: ::std::option::Option<String>,
    ) -> Result<ChatCompletionMessageList, Self::Error>;
}
///Chatkit REST endpoints.
#[::async_trait::async_trait]
pub trait OpenAiChatkitApi: Send + Sync {
    type Error: ::std::error::Error + Send + Sync + 'static;
    ///Create a ChatKit session.
    ///
    ///REST: `POST /chatkit/sessions`.
    ///Path constant: `OpenAiApiPath::CHATKIT_BY_SESSIONS`.
    async fn create_chat_session_method(
        &self,
        body: ::std::option::Option<CreateChatSessionBody>,
    ) -> Result<ChatSessionResource, Self::Error>;
    ///Cancel an active ChatKit session and return its most recent metadata. Cancelling prevents new requests from using the issued client secret.
    ///
    ///REST: `POST /chatkit/sessions/{session_id}/cancel`.
    ///Path constant: `OpenAiApiPath::CHATKIT_BY_SESSIONS_BY_SESSION_ID_BY_CANCEL`.
    async fn cancel_chat_session_method(
        &self,
        session_id: String,
    ) -> Result<ChatSessionResource, Self::Error>;
    ///List ChatKit threads with optional pagination and user filters.
    ///
    ///REST: `GET /chatkit/threads`.
    ///Path constant: `OpenAiApiPath::CHATKIT_BY_THREADS`.
    async fn list_threads_method(
        &self,
        limit: ::std::option::Option<i32>,
        order: ::std::option::Option<OrderEnum>,
        after: ::std::option::Option<String>,
        before: ::std::option::Option<String>,
        user: ::std::option::Option<String>,
    ) -> Result<ThreadListResource, Self::Error>;
    ///Retrieve a ChatKit thread by its identifier.
    ///
    ///REST: `GET /chatkit/threads/{thread_id}`.
    ///Path constant: `OpenAiApiPath::CHATKIT_BY_THREADS_BY_THREAD_ID`.
    async fn get_thread_method(
        &self,
        thread_id: String,
    ) -> Result<ThreadResource, Self::Error>;
    ///Delete a ChatKit thread along with its items and stored attachments.
    ///
    ///REST: `DELETE /chatkit/threads/{thread_id}`.
    ///Path constant: `OpenAiApiPath::CHATKIT_BY_THREADS_BY_THREAD_ID`.
    async fn delete_thread_method(
        &self,
        thread_id: String,
    ) -> Result<DeletedThreadResource, Self::Error>;
    ///List items that belong to a ChatKit thread.
    ///
    ///REST: `GET /chatkit/threads/{thread_id}/items`.
    ///Path constant: `OpenAiApiPath::CHATKIT_BY_THREADS_BY_THREAD_ID_BY_ITEMS`.
    async fn list_thread_items_method(
        &self,
        thread_id: String,
        limit: ::std::option::Option<i32>,
        order: ::std::option::Option<OrderEnum>,
        after: ::std::option::Option<String>,
        before: ::std::option::Option<String>,
    ) -> Result<ThreadItemListResource, Self::Error>;
}
///Completions REST endpoints.
#[::async_trait::async_trait]
pub trait OpenAiCompletionsApi: Send + Sync {
    type Error: ::std::error::Error + Send + Sync + 'static;
    ///Creates a completion for the provided prompt and parameters. Returns a completion object, or a sequence of completion objects if the request is streamed.
    ///
    ///REST: `POST /completions`.
    ///Path constant: `OpenAiApiPath::COMPLETIONS`.
    async fn create_completion(
        &self,
        body: CreateCompletionRequest,
    ) -> Result<CreateCompletionResponse, Self::Error>;
}
///Containers REST endpoints.
#[::async_trait::async_trait]
pub trait OpenAiContainersApi: Send + Sync {
    type Error: ::std::error::Error + Send + Sync + 'static;
    ///List Containers
    ///
    ///REST: `GET /containers`.
    ///Path constant: `OpenAiApiPath::CONTAINERS`.
    async fn list_containers(
        &self,
        limit: ::std::option::Option<i32>,
        order: ::std::option::Option<String>,
        after: ::std::option::Option<String>,
        name: ::std::option::Option<String>,
    ) -> Result<ContainerListResource, Self::Error>;
    ///Create Container
    ///
    ///REST: `POST /containers`.
    ///Path constant: `OpenAiApiPath::CONTAINERS`.
    async fn create_container(
        &self,
        body: ::std::option::Option<CreateContainerBody>,
    ) -> Result<ContainerResource, Self::Error>;
    ///Retrieve Container
    ///
    ///REST: `GET /containers/{container_id}`.
    ///Path constant: `OpenAiApiPath::CONTAINERS_BY_CONTAINER_ID`.
    async fn retrieve_container(
        &self,
        container_id: String,
    ) -> Result<ContainerResource, Self::Error>;
    ///Delete Container
    ///
    ///REST: `DELETE /containers/{container_id}`.
    ///Path constant: `OpenAiApiPath::CONTAINERS_BY_CONTAINER_ID`.
    async fn delete_container(&self, container_id: String) -> Result<(), Self::Error>;
    ///List Container files
    ///
    ///REST: `GET /containers/{container_id}/files`.
    ///Path constant: `OpenAiApiPath::CONTAINERS_BY_CONTAINER_ID_BY_FILES`.
    async fn list_container_files(
        &self,
        container_id: String,
        limit: ::std::option::Option<i32>,
        order: ::std::option::Option<String>,
        after: ::std::option::Option<String>,
    ) -> Result<ContainerFileListResource, Self::Error>;
    ///Create a Container File You can send either a multipart/form-data request with the raw file content, or a JSON request with a file ID.
    ///
    ///REST: `POST /containers/{container_id}/files`.
    ///Path constant: `OpenAiApiPath::CONTAINERS_BY_CONTAINER_ID_BY_FILES`.
    async fn create_container_file(
        &self,
        container_id: String,
        body: CreateContainerFileBody,
    ) -> Result<ContainerFileResource, Self::Error>;
    ///Retrieve Container File
    ///
    ///REST: `GET /containers/{container_id}/files/{file_id}`.
    ///Path constant: `OpenAiApiPath::CONTAINERS_BY_CONTAINER_ID_BY_FILES_BY_FILE_ID`.
    async fn retrieve_container_file(
        &self,
        container_id: String,
        file_id: String,
    ) -> Result<ContainerFileResource, Self::Error>;
    ///Delete Container File
    ///
    ///REST: `DELETE /containers/{container_id}/files/{file_id}`.
    ///Path constant: `OpenAiApiPath::CONTAINERS_BY_CONTAINER_ID_BY_FILES_BY_FILE_ID`.
    async fn delete_container_file(
        &self,
        container_id: String,
        file_id: String,
    ) -> Result<(), Self::Error>;
    ///Retrieve Container File Content
    ///
    ///REST: `GET /containers/{container_id}/files/{file_id}/content`.
    ///Path constant: `OpenAiApiPath::CONTAINERS_BY_CONTAINER_ID_BY_FILES_BY_FILE_ID_BY_CONTENT`.
    async fn retrieve_container_file_content(
        &self,
        container_id: String,
        file_id: String,
    ) -> Result<(), Self::Error>;
}
///Conversations REST endpoints.
#[::async_trait::async_trait]
pub trait OpenAiConversationsApi: Send + Sync {
    type Error: ::std::error::Error + Send + Sync + 'static;
    ///Create a conversation.
    ///
    ///REST: `POST /conversations`.
    ///Path constant: `OpenAiApiPath::CONVERSATIONS`.
    async fn create_conversation(
        &self,
        body: ::std::option::Option<CreateConversationBody>,
    ) -> Result<ConversationResource, Self::Error>;
    ///Get a conversation
    ///
    ///REST: `GET /conversations/{conversation_id}`.
    ///Path constant: `OpenAiApiPath::CONVERSATIONS_BY_CONVERSATION_ID`.
    async fn get_conversation(
        &self,
        conversation_id: String,
    ) -> Result<ConversationResource, Self::Error>;
    ///Update a conversation
    ///
    ///REST: `POST /conversations/{conversation_id}`.
    ///Path constant: `OpenAiApiPath::CONVERSATIONS_BY_CONVERSATION_ID`.
    async fn update_conversation(
        &self,
        conversation_id: String,
        body: ::std::option::Option<UpdateConversationBody>,
    ) -> Result<ConversationResource, Self::Error>;
    ///Delete a conversation. Items in the conversation will not be deleted.
    ///
    ///REST: `DELETE /conversations/{conversation_id}`.
    ///Path constant: `OpenAiApiPath::CONVERSATIONS_BY_CONVERSATION_ID`.
    async fn delete_conversation(
        &self,
        conversation_id: String,
    ) -> Result<DeletedConversationResource, Self::Error>;
    ///List all items for a conversation with the given ID.
    ///
    ///REST: `GET /conversations/{conversation_id}/items`.
    ///Path constant: `OpenAiApiPath::CONVERSATIONS_BY_CONVERSATION_ID_BY_ITEMS`.
    async fn list_conversation_items(
        &self,
        conversation_id: String,
        limit: ::std::option::Option<i32>,
        order: ::std::option::Option<String>,
        after: ::std::option::Option<String>,
        include: ::std::option::Option<Vec<IncludeEnum>>,
    ) -> Result<ConversationItemList, Self::Error>;
    ///Create items in a conversation with the given ID.
    ///
    ///REST: `POST /conversations/{conversation_id}/items`.
    ///Path constant: `OpenAiApiPath::CONVERSATIONS_BY_CONVERSATION_ID_BY_ITEMS`.
    async fn create_conversation_items(
        &self,
        conversation_id: String,
        include: ::std::option::Option<Vec<IncludeEnum>>,
        body: CreateConversationItemsRequest,
    ) -> Result<ConversationItemList, Self::Error>;
    ///Get a single item from a conversation with the given IDs.
    ///
    ///REST: `GET /conversations/{conversation_id}/items/{item_id}`.
    ///Path constant: `OpenAiApiPath::CONVERSATIONS_BY_CONVERSATION_ID_BY_ITEMS_BY_ITEM_ID`.
    async fn get_conversation_item(
        &self,
        conversation_id: String,
        item_id: String,
        include: ::std::option::Option<Vec<IncludeEnum>>,
    ) -> Result<ConversationItem, Self::Error>;
    ///Delete an item from a conversation with the given IDs.
    ///
    ///REST: `DELETE /conversations/{conversation_id}/items/{item_id}`.
    ///Path constant: `OpenAiApiPath::CONVERSATIONS_BY_CONVERSATION_ID_BY_ITEMS_BY_ITEM_ID`.
    async fn delete_conversation_item(
        &self,
        conversation_id: String,
        item_id: String,
    ) -> Result<ConversationResource, Self::Error>;
}
///Embeddings REST endpoints.
#[::async_trait::async_trait]
pub trait OpenAiEmbeddingsApi: Send + Sync {
    type Error: ::std::error::Error + Send + Sync + 'static;
    ///Creates an embedding vector representing the input text.
    ///
    ///REST: `POST /embeddings`.
    ///Path constant: `OpenAiApiPath::EMBEDDINGS`.
    async fn create_embedding(
        &self,
        body: CreateEmbeddingRequest,
    ) -> Result<CreateEmbeddingResponse, Self::Error>;
}
///Evals REST endpoints.
#[::async_trait::async_trait]
pub trait OpenAiEvalsApi: Send + Sync {
    type Error: ::std::error::Error + Send + Sync + 'static;
    ///List evaluations for a project.
    ///
    ///REST: `GET /evals`.
    ///Path constant: `OpenAiApiPath::EVALS`.
    async fn list_evals(
        &self,
        after: ::std::option::Option<String>,
        limit: ::std::option::Option<i32>,
        order: ::std::option::Option<String>,
        order_by: ::std::option::Option<String>,
    ) -> Result<EvalList, Self::Error>;
    ///Create the structure of an evaluation that can be used to test a model's performance. An evaluation is a set of testing criteria and the config for a data source, which dictates the schema of the data used in the evaluation. After creating an evaluation, you can run it on different models and model parameters. We support several types of graders and datasources. For more information, see the [Evals guide](/docs/guides/evals).
    ///
    ///REST: `POST /evals`.
    ///Path constant: `OpenAiApiPath::EVALS`.
    async fn create_eval(&self, body: CreateEvalRequest) -> Result<Eval, Self::Error>;
    ///Get an evaluation by ID.
    ///
    ///REST: `GET /evals/{eval_id}`.
    ///Path constant: `OpenAiApiPath::EVALS_BY_EVAL_ID`.
    async fn get_eval(&self, eval_id: String) -> Result<Eval, Self::Error>;
    ///Update certain properties of an evaluation.
    ///
    ///REST: `POST /evals/{eval_id}`.
    ///Path constant: `OpenAiApiPath::EVALS_BY_EVAL_ID`.
    async fn update_eval(
        &self,
        eval_id: String,
        body: UpdateEvalRequest,
    ) -> Result<Eval, Self::Error>;
    ///Delete an evaluation.
    ///
    ///REST: `DELETE /evals/{eval_id}`.
    ///Path constant: `OpenAiApiPath::EVALS_BY_EVAL_ID`.
    async fn delete_eval(
        &self,
        eval_id: String,
    ) -> Result<DeleteEvalResponse, Self::Error>;
    ///Get a list of runs for an evaluation.
    ///
    ///REST: `GET /evals/{eval_id}/runs`.
    ///Path constant: `OpenAiApiPath::EVALS_BY_EVAL_ID_BY_RUNS`.
    async fn get_eval_runs(
        &self,
        eval_id: String,
        after: ::std::option::Option<String>,
        limit: ::std::option::Option<i32>,
        order: ::std::option::Option<String>,
        status: ::std::option::Option<String>,
    ) -> Result<EvalRunList, Self::Error>;
    ///Kicks off a new run for a given evaluation, specifying the data source, and what model configuration to use to test. The datasource will be validated against the schema specified in the config of the evaluation.
    ///
    ///REST: `POST /evals/{eval_id}/runs`.
    ///Path constant: `OpenAiApiPath::EVALS_BY_EVAL_ID_BY_RUNS`.
    async fn create_eval_run(
        &self,
        eval_id: String,
        body: CreateEvalRunRequest,
    ) -> Result<EvalRun, Self::Error>;
    ///Get an evaluation run by ID.
    ///
    ///REST: `GET /evals/{eval_id}/runs/{run_id}`.
    ///Path constant: `OpenAiApiPath::EVALS_BY_EVAL_ID_BY_RUNS_BY_RUN_ID`.
    async fn get_eval_run(
        &self,
        eval_id: String,
        run_id: String,
    ) -> Result<EvalRun, Self::Error>;
    ///Cancel an ongoing evaluation run.
    ///
    ///REST: `POST /evals/{eval_id}/runs/{run_id}`.
    ///Path constant: `OpenAiApiPath::EVALS_BY_EVAL_ID_BY_RUNS_BY_RUN_ID`.
    async fn cancel_eval_run(
        &self,
        eval_id: String,
        run_id: String,
    ) -> Result<EvalRun, Self::Error>;
    ///Delete an eval run.
    ///
    ///REST: `DELETE /evals/{eval_id}/runs/{run_id}`.
    ///Path constant: `OpenAiApiPath::EVALS_BY_EVAL_ID_BY_RUNS_BY_RUN_ID`.
    async fn delete_eval_run(
        &self,
        eval_id: String,
        run_id: String,
    ) -> Result<DeleteEvalRunResponse, Self::Error>;
    ///Get a list of output items for an evaluation run.
    ///
    ///REST: `GET /evals/{eval_id}/runs/{run_id}/output_items`.
    ///Path constant: `OpenAiApiPath::EVALS_BY_EVAL_ID_BY_RUNS_BY_RUN_ID_BY_OUTPUT_ITEMS`.
    async fn get_eval_run_output_items(
        &self,
        eval_id: String,
        run_id: String,
        after: ::std::option::Option<String>,
        limit: ::std::option::Option<i32>,
        status: ::std::option::Option<String>,
        order: ::std::option::Option<String>,
    ) -> Result<EvalRunOutputItemList, Self::Error>;
    ///Get an evaluation run output item by ID.
    ///
    ///REST: `GET /evals/{eval_id}/runs/{run_id}/output_items/{output_item_id}`.
    ///Path constant: `OpenAiApiPath::EVALS_BY_EVAL_ID_BY_RUNS_BY_RUN_ID_BY_OUTPUT_ITEMS_BY_OUTPUT_ITEM_ID`.
    async fn get_eval_run_output_item(
        &self,
        eval_id: String,
        run_id: String,
        output_item_id: String,
    ) -> Result<EvalRunOutputItem, Self::Error>;
}
///Files REST endpoints.
#[::async_trait::async_trait]
pub trait OpenAiFilesApi: Send + Sync {
    type Error: ::std::error::Error + Send + Sync + 'static;
    ///Returns a list of files.
    ///
    ///REST: `GET /files`.
    ///Path constant: `OpenAiApiPath::FILES`.
    async fn list_files(
        &self,
        purpose: ::std::option::Option<String>,
        limit: ::std::option::Option<i32>,
        order: ::std::option::Option<String>,
        after: ::std::option::Option<String>,
    ) -> Result<ListFilesResponse, Self::Error>;
    ///Upload a file that can be used across various endpoints. Individual files can be up to 512 MB, and each project can store up to 2.5 TB of files in total. There is no organization-wide storage limit. Uploads to this endpoint are rate-limited to 1,000 requests per minute per authenticated user. - The Assistants API supports files up to 2 million tokens and of specific file types. See the [Assistants Tools guide](/docs/assistants/tools) for details. - The Fine-tuning API only supports `.jsonl` files. The input also has certain required formats for fine-tuning [chat](/docs/api-reference/fine-tuning/chat-input) or [completions](/docs/api-reference/fine-tuning/completions-input) models. - The Batch API only supports `.jsonl` files up to 200 MB in size. The input also has a specific required [format](/docs/api-reference/batch/request-input). - For Retrieval or `file_search` ingestion, upload files here first. If you need to attach multiple uploaded files to the same vector store, use [`/vector_stores/{vector_store_id}/file_batches`](/docs/api-reference/vector-stores-file-batches/createBatch) instead of attaching them one by one. Vector store attachment has separate limits from file upload, including 2,000 attached files per minute per organization. Please [contact us](https://help.openai.com/) if you need to increase these storage limits.
    ///
    ///REST: `POST /files`.
    ///Path constant: `OpenAiApiPath::FILES`.
    async fn create_file(
        &self,
        body: CreateFileRequest,
    ) -> Result<OpenAiFile, Self::Error>;
    ///Returns information about a specific file.
    ///
    ///REST: `GET /files/{file_id}`.
    ///Path constant: `OpenAiApiPath::FILES_BY_FILE_ID`.
    async fn retrieve_file(&self, file_id: String) -> Result<OpenAiFile, Self::Error>;
    ///Delete a file and remove it from all vector stores.
    ///
    ///REST: `DELETE /files/{file_id}`.
    ///Path constant: `OpenAiApiPath::FILES_BY_FILE_ID`.
    async fn delete_file(
        &self,
        file_id: String,
    ) -> Result<DeleteFileResponse, Self::Error>;
    ///Returns the contents of the specified file.
    ///
    ///REST: `GET /files/{file_id}/content`.
    ///Path constant: `OpenAiApiPath::FILES_BY_FILE_ID_BY_CONTENT`.
    async fn download_file(&self, file_id: String) -> Result<String, Self::Error>;
}
///FineTuning REST endpoints.
#[::async_trait::async_trait]
pub trait OpenAiFineTuningApi: Send + Sync {
    type Error: ::std::error::Error + Send + Sync + 'static;
    ///Run a grader.
    ///
    ///REST: `POST /fine_tuning/alpha/graders/run`.
    ///Path constant: `OpenAiApiPath::FINE_TUNING_BY_ALPHA_BY_GRADERS_BY_RUN`.
    async fn run_grader(
        &self,
        body: RunGraderRequest,
    ) -> Result<RunGraderResponse, Self::Error>;
    ///Validate a grader.
    ///
    ///REST: `POST /fine_tuning/alpha/graders/validate`.
    ///Path constant: `OpenAiApiPath::FINE_TUNING_BY_ALPHA_BY_GRADERS_BY_VALIDATE`.
    async fn validate_grader(
        &self,
        body: ValidateGraderRequest,
    ) -> Result<ValidateGraderResponse, Self::Error>;
    ///**NOTE:** This endpoint requires an [admin API key](../admin-api-keys). Organization owners can use this endpoint to view all permissions for a fine-tuned model checkpoint.
    ///
    ///REST: `GET /fine_tuning/checkpoints/{fine_tuned_model_checkpoint}/permissions`.
    ///Path constant: `OpenAiApiPath::FINE_TUNING_BY_CHECKPOINTS_BY_FINE_TUNED_MODEL_CHECKPOINT_BY_PERMISSIONS`.
    async fn list_fine_tuning_checkpoint_permissions(
        &self,
        fine_tuned_model_checkpoint: String,
        project_id: ::std::option::Option<String>,
        after: ::std::option::Option<String>,
        limit: ::std::option::Option<i32>,
        order: ::std::option::Option<String>,
    ) -> Result<ListFineTuningCheckpointPermissionResponse, Self::Error>;
    ///**NOTE:** Calling this endpoint requires an [admin API key](../admin-api-keys). This enables organization owners to share fine-tuned models with other projects in their organization.
    ///
    ///REST: `POST /fine_tuning/checkpoints/{fine_tuned_model_checkpoint}/permissions`.
    ///Path constant: `OpenAiApiPath::FINE_TUNING_BY_CHECKPOINTS_BY_FINE_TUNED_MODEL_CHECKPOINT_BY_PERMISSIONS`.
    async fn create_fine_tuning_checkpoint_permission(
        &self,
        fine_tuned_model_checkpoint: String,
        body: CreateFineTuningCheckpointPermissionRequest,
    ) -> Result<ListFineTuningCheckpointPermissionResponse, Self::Error>;
    ///**NOTE:** This endpoint requires an [admin API key](../admin-api-keys). Organization owners can use this endpoint to delete a permission for a fine-tuned model checkpoint.
    ///
    ///REST: `DELETE /fine_tuning/checkpoints/{fine_tuned_model_checkpoint}/permissions/{permission_id}`.
    ///Path constant: `OpenAiApiPath::FINE_TUNING_BY_CHECKPOINTS_BY_FINE_TUNED_MODEL_CHECKPOINT_BY_PERMISSIONS_BY_PERMISSION_ID`.
    async fn delete_fine_tuning_checkpoint_permission(
        &self,
        fine_tuned_model_checkpoint: String,
        permission_id: String,
    ) -> Result<DeleteFineTuningCheckpointPermissionResponse, Self::Error>;
    ///List your organization's fine-tuning jobs
    ///
    ///REST: `GET /fine_tuning/jobs`.
    ///Path constant: `OpenAiApiPath::FINE_TUNING_BY_JOBS`.
    async fn list_paginated_fine_tuning_jobs(
        &self,
        after: ::std::option::Option<String>,
        limit: ::std::option::Option<i32>,
        metadata: ::std::option::Option<OpenAiJsonValue>,
    ) -> Result<ListPaginatedFineTuningJobsResponse, Self::Error>;
    ///Creates a fine-tuning job which begins the process of creating a new model from a given dataset. Response includes details of the enqueued job including job status and the name of the fine-tuned models once complete. [Learn more about fine-tuning](/docs/guides/model-optimization)
    ///
    ///REST: `POST /fine_tuning/jobs`.
    ///Path constant: `OpenAiApiPath::FINE_TUNING_BY_JOBS`.
    async fn create_fine_tuning_job(
        &self,
        body: CreateFineTuningJobRequest,
    ) -> Result<FineTuningJob, Self::Error>;
    ///Get info about a fine-tuning job. [Learn more about fine-tuning](/docs/guides/model-optimization)
    ///
    ///REST: `GET /fine_tuning/jobs/{fine_tuning_job_id}`.
    ///Path constant: `OpenAiApiPath::FINE_TUNING_BY_JOBS_BY_FINE_TUNING_JOB_ID`.
    async fn retrieve_fine_tuning_job(
        &self,
        fine_tuning_job_id: String,
    ) -> Result<FineTuningJob, Self::Error>;
    ///Immediately cancel a fine-tune job.
    ///
    ///REST: `POST /fine_tuning/jobs/{fine_tuning_job_id}/cancel`.
    ///Path constant: `OpenAiApiPath::FINE_TUNING_BY_JOBS_BY_FINE_TUNING_JOB_ID_BY_CANCEL`.
    async fn cancel_fine_tuning_job(
        &self,
        fine_tuning_job_id: String,
    ) -> Result<FineTuningJob, Self::Error>;
    ///List checkpoints for a fine-tuning job.
    ///
    ///REST: `GET /fine_tuning/jobs/{fine_tuning_job_id}/checkpoints`.
    ///Path constant: `OpenAiApiPath::FINE_TUNING_BY_JOBS_BY_FINE_TUNING_JOB_ID_BY_CHECKPOINTS`.
    async fn list_fine_tuning_job_checkpoints(
        &self,
        fine_tuning_job_id: String,
        after: ::std::option::Option<String>,
        limit: ::std::option::Option<i32>,
    ) -> Result<ListFineTuningJobCheckpointsResponse, Self::Error>;
    ///Get status updates for a fine-tuning job.
    ///
    ///REST: `GET /fine_tuning/jobs/{fine_tuning_job_id}/events`.
    ///Path constant: `OpenAiApiPath::FINE_TUNING_BY_JOBS_BY_FINE_TUNING_JOB_ID_BY_EVENTS`.
    async fn list_fine_tuning_events(
        &self,
        fine_tuning_job_id: String,
        after: ::std::option::Option<String>,
        limit: ::std::option::Option<i32>,
    ) -> Result<ListFineTuningJobEventsResponse, Self::Error>;
    ///Pause a fine-tune job.
    ///
    ///REST: `POST /fine_tuning/jobs/{fine_tuning_job_id}/pause`.
    ///Path constant: `OpenAiApiPath::FINE_TUNING_BY_JOBS_BY_FINE_TUNING_JOB_ID_BY_PAUSE`.
    async fn pause_fine_tuning_job(
        &self,
        fine_tuning_job_id: String,
    ) -> Result<FineTuningJob, Self::Error>;
    ///Resume a fine-tune job.
    ///
    ///REST: `POST /fine_tuning/jobs/{fine_tuning_job_id}/resume`.
    ///Path constant: `OpenAiApiPath::FINE_TUNING_BY_JOBS_BY_FINE_TUNING_JOB_ID_BY_RESUME`.
    async fn resume_fine_tuning_job(
        &self,
        fine_tuning_job_id: String,
    ) -> Result<FineTuningJob, Self::Error>;
}
///GroupOrganizationRoleAssignments REST endpoints.
#[::async_trait::async_trait]
pub trait OpenAiGroupOrganizationRoleAssignmentsApi: Send + Sync {
    type Error: ::std::error::Error + Send + Sync + 'static;
    ///Lists the organization roles assigned to a group within the organization.
    ///
    ///REST: `GET /organization/groups/{group_id}/roles`.
    ///Path constant: `OpenAiApiPath::ORGANIZATION_BY_GROUPS_BY_GROUP_ID_BY_ROLES`.
    async fn list_group_role_assignments(
        &self,
        group_id: String,
        limit: ::std::option::Option<i32>,
        after: ::std::option::Option<String>,
        order: ::std::option::Option<String>,
    ) -> Result<RoleListResource, Self::Error>;
    ///Assigns an organization role to a group within the organization.
    ///
    ///REST: `POST /organization/groups/{group_id}/roles`.
    ///Path constant: `OpenAiApiPath::ORGANIZATION_BY_GROUPS_BY_GROUP_ID_BY_ROLES`.
    async fn assign_group_role(
        &self,
        group_id: String,
        body: PublicAssignOrganizationGroupRoleBody,
    ) -> Result<GroupRoleAssignment, Self::Error>;
    ///Unassigns an organization role from a group within the organization.
    ///
    ///REST: `DELETE /organization/groups/{group_id}/roles/{role_id}`.
    ///Path constant: `OpenAiApiPath::ORGANIZATION_BY_GROUPS_BY_GROUP_ID_BY_ROLES_BY_ROLE_ID`.
    async fn unassign_group_role(
        &self,
        group_id: String,
        role_id: String,
    ) -> Result<DeletedRoleAssignmentResource, Self::Error>;
}
///GroupUsers REST endpoints.
#[::async_trait::async_trait]
pub trait OpenAiGroupUsersApi: Send + Sync {
    type Error: ::std::error::Error + Send + Sync + 'static;
    ///Lists the users assigned to a group.
    ///
    ///REST: `GET /organization/groups/{group_id}/users`.
    ///Path constant: `OpenAiApiPath::ORGANIZATION_BY_GROUPS_BY_GROUP_ID_BY_USERS`.
    async fn list_group_users(
        &self,
        group_id: String,
        limit: ::std::option::Option<i32>,
        after: ::std::option::Option<String>,
        order: ::std::option::Option<String>,
    ) -> Result<UserListResource, Self::Error>;
    ///Adds a user to a group.
    ///
    ///REST: `POST /organization/groups/{group_id}/users`.
    ///Path constant: `OpenAiApiPath::ORGANIZATION_BY_GROUPS_BY_GROUP_ID_BY_USERS`.
    async fn add_group_user(
        &self,
        group_id: String,
        body: CreateGroupUserBody,
    ) -> Result<GroupUserAssignment, Self::Error>;
    ///Removes a user from a group.
    ///
    ///REST: `DELETE /organization/groups/{group_id}/users/{user_id}`.
    ///Path constant: `OpenAiApiPath::ORGANIZATION_BY_GROUPS_BY_GROUP_ID_BY_USERS_BY_USER_ID`.
    async fn remove_group_user(
        &self,
        group_id: String,
        user_id: String,
    ) -> Result<GroupUserDeletedResource, Self::Error>;
}
///Groups REST endpoints.
#[::async_trait::async_trait]
pub trait OpenAiGroupsApi: Send + Sync {
    type Error: ::std::error::Error + Send + Sync + 'static;
    ///Lists all groups in the organization.
    ///
    ///REST: `GET /organization/groups`.
    ///Path constant: `OpenAiApiPath::ORGANIZATION_BY_GROUPS`.
    async fn list_groups(
        &self,
        limit: ::std::option::Option<i32>,
        after: ::std::option::Option<String>,
        order: ::std::option::Option<String>,
    ) -> Result<GroupListResource, Self::Error>;
    ///Creates a new group in the organization.
    ///
    ///REST: `POST /organization/groups`.
    ///Path constant: `OpenAiApiPath::ORGANIZATION_BY_GROUPS`.
    async fn create_group(
        &self,
        body: CreateGroupBody,
    ) -> Result<GroupResponse, Self::Error>;
    ///Updates a group's information.
    ///
    ///REST: `POST /organization/groups/{group_id}`.
    ///Path constant: `OpenAiApiPath::ORGANIZATION_BY_GROUPS_BY_GROUP_ID`.
    async fn update_group(
        &self,
        group_id: String,
        body: UpdateGroupBody,
    ) -> Result<GroupResourceWithSuccess, Self::Error>;
    ///Deletes a group from the organization.
    ///
    ///REST: `DELETE /organization/groups/{group_id}`.
    ///Path constant: `OpenAiApiPath::ORGANIZATION_BY_GROUPS_BY_GROUP_ID`.
    async fn delete_group(
        &self,
        group_id: String,
    ) -> Result<GroupDeletedResource, Self::Error>;
}
///Images REST endpoints.
#[::async_trait::async_trait]
pub trait OpenAiImagesApi: Send + Sync {
    type Error: ::std::error::Error + Send + Sync + 'static;
    ///Creates an edited or extended image given one or more source images and a prompt. This endpoint supports GPT Image models (`gpt-image-1.5`, `gpt-image-1`, `gpt-image-1-mini`, and `chatgpt-image-latest`) and `dall-e-2`.
    ///
    ///REST: `POST /images/edits`.
    ///Path constant: `OpenAiApiPath::IMAGES_BY_EDITS`.
    async fn create_image_edit(
        &self,
        body: EditImageBodyJsonParam,
    ) -> Result<ImagesResponse, Self::Error>;
    ///Creates an image given a prompt. [Learn more](/docs/guides/images).
    ///
    ///REST: `POST /images/generations`.
    ///Path constant: `OpenAiApiPath::IMAGES_BY_GENERATIONS`.
    async fn create_image(
        &self,
        body: CreateImageRequest,
    ) -> Result<ImagesResponse, Self::Error>;
    ///Creates a variation of a given image. This endpoint only supports `dall-e-2`.
    ///
    ///REST: `POST /images/variations`.
    ///Path constant: `OpenAiApiPath::IMAGES_BY_VARIATIONS`.
    async fn create_image_variation(
        &self,
        body: CreateImageVariationRequest,
    ) -> Result<ImagesResponse, Self::Error>;
}
///Invites REST endpoints.
#[::async_trait::async_trait]
pub trait OpenAiInvitesApi: Send + Sync {
    type Error: ::std::error::Error + Send + Sync + 'static;
    ///Returns a list of invites in the organization.
    ///
    ///REST: `GET /organization/invites`.
    ///Path constant: `OpenAiApiPath::ORGANIZATION_BY_INVITES`.
    async fn list_invites(
        &self,
        limit: ::std::option::Option<i32>,
        after: ::std::option::Option<String>,
    ) -> Result<InviteListResponse, Self::Error>;
    ///Create an invite for a user to the organization. The invite must be accepted by the user before they have access to the organization.
    ///
    ///REST: `POST /organization/invites`.
    ///Path constant: `OpenAiApiPath::ORGANIZATION_BY_INVITES`.
    async fn invite_user(&self, body: InviteRequest) -> Result<Invite, Self::Error>;
    ///Retrieves an invite.
    ///
    ///REST: `GET /organization/invites/{invite_id}`.
    ///Path constant: `OpenAiApiPath::ORGANIZATION_BY_INVITES_BY_INVITE_ID`.
    async fn retrieve_invite(&self, invite_id: String) -> Result<Invite, Self::Error>;
    ///Delete an invite. If the invite has already been accepted, it cannot be deleted.
    ///
    ///REST: `DELETE /organization/invites/{invite_id}`.
    ///Path constant: `OpenAiApiPath::ORGANIZATION_BY_INVITES_BY_INVITE_ID`.
    async fn delete_invite(
        &self,
        invite_id: String,
    ) -> Result<InviteDeleteResponse, Self::Error>;
}
///Models REST endpoints.
#[::async_trait::async_trait]
pub trait OpenAiModelsApi: Send + Sync {
    type Error: ::std::error::Error + Send + Sync + 'static;
    ///Lists the currently available models, and provides basic information about each one such as the owner and availability.
    ///
    ///REST: `GET /models`.
    ///Path constant: `OpenAiApiPath::MODELS`.
    async fn list_models(&self) -> Result<ListModelsResponse, Self::Error>;
    ///Retrieves a model instance, providing basic information about the model such as the owner and permissioning.
    ///
    ///REST: `GET /models/{model}`.
    ///Path constant: `OpenAiApiPath::MODELS_BY_MODEL`.
    async fn retrieve_model(&self, model: String) -> Result<Model, Self::Error>;
    ///Delete a fine-tuned model. You must have the Owner role in your organization to delete a model.
    ///
    ///REST: `DELETE /models/{model}`.
    ///Path constant: `OpenAiApiPath::MODELS_BY_MODEL`.
    async fn delete_model(
        &self,
        model: String,
    ) -> Result<DeleteModelResponse, Self::Error>;
}
///Moderations REST endpoints.
#[::async_trait::async_trait]
pub trait OpenAiModerationsApi: Send + Sync {
    type Error: ::std::error::Error + Send + Sync + 'static;
    ///Classifies if text and/or image inputs are potentially harmful. Learn more in the [moderation guide](/docs/guides/moderation).
    ///
    ///REST: `POST /moderations`.
    ///Path constant: `OpenAiApiPath::MODERATIONS`.
    async fn create_moderation(
        &self,
        body: CreateModerationRequest,
    ) -> Result<CreateModerationResponse, Self::Error>;
}
///ProjectGroupRoleAssignments REST endpoints.
#[::async_trait::async_trait]
pub trait OpenAiProjectGroupRoleAssignmentsApi: Send + Sync {
    type Error: ::std::error::Error + Send + Sync + 'static;
    ///Lists the project roles assigned to a group within a project.
    ///
    ///REST: `GET /projects/{project_id}/groups/{group_id}/roles`.
    ///Path constant: `OpenAiApiPath::PROJECTS_BY_PROJECT_ID_BY_GROUPS_BY_GROUP_ID_BY_ROLES`.
    async fn list_project_group_role_assignments(
        &self,
        project_id: String,
        group_id: String,
        limit: ::std::option::Option<i32>,
        after: ::std::option::Option<String>,
        order: ::std::option::Option<String>,
    ) -> Result<RoleListResource, Self::Error>;
    ///Assigns a project role to a group within a project.
    ///
    ///REST: `POST /projects/{project_id}/groups/{group_id}/roles`.
    ///Path constant: `OpenAiApiPath::PROJECTS_BY_PROJECT_ID_BY_GROUPS_BY_GROUP_ID_BY_ROLES`.
    async fn assign_project_group_role(
        &self,
        project_id: String,
        group_id: String,
        body: PublicAssignOrganizationGroupRoleBody,
    ) -> Result<GroupRoleAssignment, Self::Error>;
    ///Unassigns a project role from a group within a project.
    ///
    ///REST: `DELETE /projects/{project_id}/groups/{group_id}/roles/{role_id}`.
    ///Path constant: `OpenAiApiPath::PROJECTS_BY_PROJECT_ID_BY_GROUPS_BY_GROUP_ID_BY_ROLES_BY_ROLE_ID`.
    async fn unassign_project_group_role(
        &self,
        project_id: String,
        group_id: String,
        role_id: String,
    ) -> Result<DeletedRoleAssignmentResource, Self::Error>;
}
///ProjectGroups REST endpoints.
#[::async_trait::async_trait]
pub trait OpenAiProjectGroupsApi: Send + Sync {
    type Error: ::std::error::Error + Send + Sync + 'static;
    ///Lists the groups that have access to a project.
    ///
    ///REST: `GET /organization/projects/{project_id}/groups`.
    ///Path constant: `OpenAiApiPath::ORGANIZATION_BY_PROJECTS_BY_PROJECT_ID_BY_GROUPS`.
    async fn list_project_groups(
        &self,
        project_id: String,
        limit: ::std::option::Option<i32>,
        after: ::std::option::Option<String>,
        order: ::std::option::Option<String>,
    ) -> Result<ProjectGroupListResource, Self::Error>;
    ///Grants a group access to a project.
    ///
    ///REST: `POST /organization/projects/{project_id}/groups`.
    ///Path constant: `OpenAiApiPath::ORGANIZATION_BY_PROJECTS_BY_PROJECT_ID_BY_GROUPS`.
    async fn add_project_group(
        &self,
        project_id: String,
        body: InviteProjectGroupBody,
    ) -> Result<ProjectGroup, Self::Error>;
    ///Revokes a group's access to a project.
    ///
    ///REST: `DELETE /organization/projects/{project_id}/groups/{group_id}`.
    ///Path constant: `OpenAiApiPath::ORGANIZATION_BY_PROJECTS_BY_PROJECT_ID_BY_GROUPS_BY_GROUP_ID`.
    async fn remove_project_group(
        &self,
        project_id: String,
        group_id: String,
    ) -> Result<ProjectGroupDeletedResource, Self::Error>;
}
///ProjectUserRoleAssignments REST endpoints.
#[::async_trait::async_trait]
pub trait OpenAiProjectUserRoleAssignmentsApi: Send + Sync {
    type Error: ::std::error::Error + Send + Sync + 'static;
    ///Lists the project roles assigned to a user within a project.
    ///
    ///REST: `GET /projects/{project_id}/users/{user_id}/roles`.
    ///Path constant: `OpenAiApiPath::PROJECTS_BY_PROJECT_ID_BY_USERS_BY_USER_ID_BY_ROLES`.
    async fn list_project_user_role_assignments(
        &self,
        project_id: String,
        user_id: String,
        limit: ::std::option::Option<i32>,
        after: ::std::option::Option<String>,
        order: ::std::option::Option<String>,
    ) -> Result<RoleListResource, Self::Error>;
    ///Assigns a project role to a user within a project.
    ///
    ///REST: `POST /projects/{project_id}/users/{user_id}/roles`.
    ///Path constant: `OpenAiApiPath::PROJECTS_BY_PROJECT_ID_BY_USERS_BY_USER_ID_BY_ROLES`.
    async fn assign_project_user_role(
        &self,
        project_id: String,
        user_id: String,
        body: PublicAssignOrganizationGroupRoleBody,
    ) -> Result<UserRoleAssignment, Self::Error>;
    ///Unassigns a project role from a user within a project.
    ///
    ///REST: `DELETE /projects/{project_id}/users/{user_id}/roles/{role_id}`.
    ///Path constant: `OpenAiApiPath::PROJECTS_BY_PROJECT_ID_BY_USERS_BY_USER_ID_BY_ROLES_BY_ROLE_ID`.
    async fn unassign_project_user_role(
        &self,
        project_id: String,
        user_id: String,
        role_id: String,
    ) -> Result<DeletedRoleAssignmentResource, Self::Error>;
}
///Projects REST endpoints.
#[::async_trait::async_trait]
pub trait OpenAiProjectsApi: Send + Sync {
    type Error: ::std::error::Error + Send + Sync + 'static;
    ///Returns a list of projects.
    ///
    ///REST: `GET /organization/projects`.
    ///Path constant: `OpenAiApiPath::ORGANIZATION_BY_PROJECTS`.
    async fn list_projects(
        &self,
        limit: ::std::option::Option<i32>,
        after: ::std::option::Option<String>,
        include_archived: ::std::option::Option<bool>,
    ) -> Result<ProjectListResponse, Self::Error>;
    ///Create a new project in the organization. Projects can be created and archived, but cannot be deleted.
    ///
    ///REST: `POST /organization/projects`.
    ///Path constant: `OpenAiApiPath::ORGANIZATION_BY_PROJECTS`.
    async fn create_project(
        &self,
        body: ProjectCreateRequest,
    ) -> Result<Project, Self::Error>;
    ///Retrieves a project.
    ///
    ///REST: `GET /organization/projects/{project_id}`.
    ///Path constant: `OpenAiApiPath::ORGANIZATION_BY_PROJECTS_BY_PROJECT_ID`.
    async fn retrieve_project(&self, project_id: String) -> Result<Project, Self::Error>;
    ///Modifies a project in the organization.
    ///
    ///REST: `POST /organization/projects/{project_id}`.
    ///Path constant: `OpenAiApiPath::ORGANIZATION_BY_PROJECTS_BY_PROJECT_ID`.
    async fn modify_project(
        &self,
        project_id: String,
        body: ProjectUpdateRequest,
    ) -> Result<Project, Self::Error>;
    ///Returns a list of API keys in the project.
    ///
    ///REST: `GET /organization/projects/{project_id}/api_keys`.
    ///Path constant: `OpenAiApiPath::ORGANIZATION_BY_PROJECTS_BY_PROJECT_ID_BY_API_KEYS`.
    async fn list_project_api_keys(
        &self,
        project_id: String,
        limit: ::std::option::Option<i32>,
        after: ::std::option::Option<String>,
    ) -> Result<ProjectApiKeyListResponse, Self::Error>;
    ///Retrieves an API key in the project.
    ///
    ///REST: `GET /organization/projects/{project_id}/api_keys/{api_key_id}`.
    ///Path constant: `OpenAiApiPath::ORGANIZATION_BY_PROJECTS_BY_PROJECT_ID_BY_API_KEYS_BY_API_KEY_ID`.
    async fn retrieve_project_api_key(
        &self,
        project_id: String,
        api_key_id: String,
    ) -> Result<ProjectApiKey, Self::Error>;
    ///Deletes an API key from the project. Returns confirmation of the key deletion, or an error if the key belonged to a service account.
    ///
    ///REST: `DELETE /organization/projects/{project_id}/api_keys/{api_key_id}`.
    ///Path constant: `OpenAiApiPath::ORGANIZATION_BY_PROJECTS_BY_PROJECT_ID_BY_API_KEYS_BY_API_KEY_ID`.
    async fn delete_project_api_key(
        &self,
        project_id: String,
        api_key_id: String,
    ) -> Result<ProjectApiKeyDeleteResponse, Self::Error>;
    ///Archives a project in the organization. Archived projects cannot be used or updated.
    ///
    ///REST: `POST /organization/projects/{project_id}/archive`.
    ///Path constant: `OpenAiApiPath::ORGANIZATION_BY_PROJECTS_BY_PROJECT_ID_BY_ARCHIVE`.
    async fn archive_project(&self, project_id: String) -> Result<Project, Self::Error>;
    ///Returns the rate limits per model for a project.
    ///
    ///REST: `GET /organization/projects/{project_id}/rate_limits`.
    ///Path constant: `OpenAiApiPath::ORGANIZATION_BY_PROJECTS_BY_PROJECT_ID_BY_RATE_LIMITS`.
    async fn list_project_rate_limits(
        &self,
        project_id: String,
        limit: ::std::option::Option<i32>,
        after: ::std::option::Option<String>,
        before: ::std::option::Option<String>,
    ) -> Result<ProjectRateLimitListResponse, Self::Error>;
    ///Updates a project rate limit.
    ///
    ///REST: `POST /organization/projects/{project_id}/rate_limits/{rate_limit_id}`.
    ///Path constant: `OpenAiApiPath::ORGANIZATION_BY_PROJECTS_BY_PROJECT_ID_BY_RATE_LIMITS_BY_RATE_LIMIT_ID`.
    async fn update_project_rate_limits(
        &self,
        project_id: String,
        rate_limit_id: String,
        body: ProjectRateLimitUpdateRequest,
    ) -> Result<ProjectRateLimit, Self::Error>;
    ///Returns a list of service accounts in the project.
    ///
    ///REST: `GET /organization/projects/{project_id}/service_accounts`.
    ///Path constant: `OpenAiApiPath::ORGANIZATION_BY_PROJECTS_BY_PROJECT_ID_BY_SERVICE_ACCOUNTS`.
    async fn list_project_service_accounts(
        &self,
        project_id: String,
        limit: ::std::option::Option<i32>,
        after: ::std::option::Option<String>,
    ) -> Result<ProjectServiceAccountListResponse, Self::Error>;
    ///Creates a new service account in the project. This also returns an unredacted API key for the service account.
    ///
    ///REST: `POST /organization/projects/{project_id}/service_accounts`.
    ///Path constant: `OpenAiApiPath::ORGANIZATION_BY_PROJECTS_BY_PROJECT_ID_BY_SERVICE_ACCOUNTS`.
    async fn create_project_service_account(
        &self,
        project_id: String,
        body: ProjectServiceAccountCreateRequest,
    ) -> Result<ProjectServiceAccountCreateResponse, Self::Error>;
    ///Retrieves a service account in the project.
    ///
    ///REST: `GET /organization/projects/{project_id}/service_accounts/{service_account_id}`.
    ///Path constant: `OpenAiApiPath::ORGANIZATION_BY_PROJECTS_BY_PROJECT_ID_BY_SERVICE_ACCOUNTS_BY_SERVICE_ACCOUNT_ID`.
    async fn retrieve_project_service_account(
        &self,
        project_id: String,
        service_account_id: String,
    ) -> Result<ProjectServiceAccount, Self::Error>;
    ///Deletes a service account from the project. Returns confirmation of service account deletion, or an error if the project is archived (archived projects have no service accounts).
    ///
    ///REST: `DELETE /organization/projects/{project_id}/service_accounts/{service_account_id}`.
    ///Path constant: `OpenAiApiPath::ORGANIZATION_BY_PROJECTS_BY_PROJECT_ID_BY_SERVICE_ACCOUNTS_BY_SERVICE_ACCOUNT_ID`.
    async fn delete_project_service_account(
        &self,
        project_id: String,
        service_account_id: String,
    ) -> Result<ProjectServiceAccountDeleteResponse, Self::Error>;
    ///Returns a list of users in the project.
    ///
    ///REST: `GET /organization/projects/{project_id}/users`.
    ///Path constant: `OpenAiApiPath::ORGANIZATION_BY_PROJECTS_BY_PROJECT_ID_BY_USERS`.
    async fn list_project_users(
        &self,
        project_id: String,
        limit: ::std::option::Option<i32>,
        after: ::std::option::Option<String>,
    ) -> Result<ProjectUserListResponse, Self::Error>;
    ///Adds a user to the project. Users must already be members of the organization to be added to a project.
    ///
    ///REST: `POST /organization/projects/{project_id}/users`.
    ///Path constant: `OpenAiApiPath::ORGANIZATION_BY_PROJECTS_BY_PROJECT_ID_BY_USERS`.
    async fn create_project_user(
        &self,
        project_id: String,
        body: ProjectUserCreateRequest,
    ) -> Result<ProjectUser, Self::Error>;
    ///Retrieves a user in the project.
    ///
    ///REST: `GET /organization/projects/{project_id}/users/{user_id}`.
    ///Path constant: `OpenAiApiPath::ORGANIZATION_BY_PROJECTS_BY_PROJECT_ID_BY_USERS_BY_USER_ID`.
    async fn retrieve_project_user(
        &self,
        project_id: String,
        user_id: String,
    ) -> Result<ProjectUser, Self::Error>;
    ///Modifies a user's role in the project.
    ///
    ///REST: `POST /organization/projects/{project_id}/users/{user_id}`.
    ///Path constant: `OpenAiApiPath::ORGANIZATION_BY_PROJECTS_BY_PROJECT_ID_BY_USERS_BY_USER_ID`.
    async fn modify_project_user(
        &self,
        project_id: String,
        user_id: String,
        body: ProjectUserUpdateRequest,
    ) -> Result<ProjectUser, Self::Error>;
    ///Deletes a user from the project. Returns confirmation of project user deletion, or an error if the project is archived (archived projects have no users).
    ///
    ///REST: `DELETE /organization/projects/{project_id}/users/{user_id}`.
    ///Path constant: `OpenAiApiPath::ORGANIZATION_BY_PROJECTS_BY_PROJECT_ID_BY_USERS_BY_USER_ID`.
    async fn delete_project_user(
        &self,
        project_id: String,
        user_id: String,
    ) -> Result<ProjectUserDeleteResponse, Self::Error>;
}
///Realtime REST endpoints.
#[::async_trait::async_trait]
pub trait OpenAiRealtimeApi: Send + Sync {
    type Error: ::std::error::Error + Send + Sync + 'static;
    ///Create a new Realtime API call over WebRTC and receive the SDP answer needed to complete the peer connection.
    ///
    ///REST: `POST /realtime/calls`.
    ///Path constant: `OpenAiApiPath::REALTIME_BY_CALLS`.
    async fn create_realtime_call(
        &self,
        body: RealtimeCallCreateRequest,
    ) -> Result<OpenAiTextBody, Self::Error>;
    ///Accept an incoming SIP call and configure the realtime session that will handle it.
    ///
    ///REST: `POST /realtime/calls/{call_id}/accept`.
    ///Path constant: `OpenAiApiPath::REALTIME_BY_CALLS_BY_CALL_ID_BY_ACCEPT`.
    async fn accept_realtime_call(
        &self,
        call_id: String,
        body: RealtimeSessionCreateRequestGa,
    ) -> Result<(), Self::Error>;
    ///End an active Realtime API call, whether it was initiated over SIP or WebRTC.
    ///
    ///REST: `POST /realtime/calls/{call_id}/hangup`.
    ///Path constant: `OpenAiApiPath::REALTIME_BY_CALLS_BY_CALL_ID_BY_HANGUP`.
    async fn hangup_realtime_call(&self, call_id: String) -> Result<(), Self::Error>;
    ///Transfer an active SIP call to a new destination using the SIP REFER verb.
    ///
    ///REST: `POST /realtime/calls/{call_id}/refer`.
    ///Path constant: `OpenAiApiPath::REALTIME_BY_CALLS_BY_CALL_ID_BY_REFER`.
    async fn refer_realtime_call(
        &self,
        call_id: String,
        body: RealtimeCallReferRequest,
    ) -> Result<(), Self::Error>;
    ///Decline an incoming SIP call by returning a SIP status code to the caller.
    ///
    ///REST: `POST /realtime/calls/{call_id}/reject`.
    ///Path constant: `OpenAiApiPath::REALTIME_BY_CALLS_BY_CALL_ID_BY_REJECT`.
    async fn reject_realtime_call(
        &self,
        call_id: String,
        body: ::std::option::Option<RealtimeCallRejectRequest>,
    ) -> Result<(), Self::Error>;
    ///Create a Realtime client secret with an associated session configuration. Client secrets are short-lived tokens that can be passed to a client app, such as a web frontend or mobile client, which grants access to the Realtime API without leaking your main API key. You can configure a custom TTL for each client secret. You can also attach session configuration options to the client secret, which will be applied to any sessions created using that client secret, but these can also be overridden by the client connection. [Learn more about authentication with client secrets over WebRTC](/docs/guides/realtime-webrtc). Returns the created client secret and the effective session object. The client secret is a string that looks like `ek_1234`.
    ///
    ///REST: `POST /realtime/client_secrets`.
    ///Path constant: `OpenAiApiPath::REALTIME_BY_CLIENT_SECRETS`.
    async fn create_realtime_client_secret(
        &self,
        body: RealtimeCreateClientSecretRequest,
    ) -> Result<RealtimeCreateClientSecretResponse, Self::Error>;
    ///Create an ephemeral API token for use in client-side applications with the Realtime API. Can be configured with the same session parameters as the `session.update` client event. It responds with a session object, plus a `client_secret` key which contains a usable ephemeral API token that can be used to authenticate browser clients for the Realtime API. Returns the created Realtime session object, plus an ephemeral key.
    ///
    ///REST: `POST /realtime/sessions`.
    ///Path constant: `OpenAiApiPath::REALTIME_BY_SESSIONS`.
    async fn create_realtime_session(
        &self,
        body: RealtimeSessionCreateRequest,
    ) -> Result<RealtimeSessionCreateResponse, Self::Error>;
    ///Create an ephemeral API token for use in client-side applications with the Realtime API specifically for realtime transcriptions. Can be configured with the same session parameters as the `transcription_session.update` client event. It responds with a session object, plus a `client_secret` key which contains a usable ephemeral API token that can be used to authenticate browser clients for the Realtime API. Returns the created Realtime transcription session object, plus an ephemeral key.
    ///
    ///REST: `POST /realtime/transcription_sessions`.
    ///Path constant: `OpenAiApiPath::REALTIME_BY_TRANSCRIPTION_SESSIONS`.
    async fn create_realtime_transcription_session(
        &self,
        body: RealtimeTranscriptionSessionCreateRequest,
    ) -> Result<RealtimeTranscriptionSessionCreateResponse, Self::Error>;
    ///Create a Realtime translation client secret with an associated translation session configuration. Client secrets are short-lived tokens that can be passed to a client app, such as a web frontend or mobile client, which grants access to the Realtime Translation API without leaking your main API key. You can configure a custom TTL for each client secret. Returns the created client secret and the effective translation session object. The client secret is a string that looks like `ek_1234`.
    ///
    ///REST: `POST /realtime/translations/client_secrets`.
    ///Path constant: `OpenAiApiPath::REALTIME_BY_TRANSLATIONS_BY_CLIENT_SECRETS`.
    async fn create_realtime_translation_client_secret(
        &self,
        body: RealtimeTranslationClientSecretCreateRequest,
    ) -> Result<RealtimeTranslationClientSecretCreateResponse, Self::Error>;
}
///Responses REST endpoints.
#[::async_trait::async_trait]
pub trait OpenAiResponsesApi: Send + Sync {
    type Error: ::std::error::Error + Send + Sync + 'static;
    ///Creates a model response. Provide [text](/docs/guides/text) or [image](/docs/guides/images) inputs to generate [text](/docs/guides/text) or [JSON](/docs/guides/structured-outputs) outputs. Have the model call your own [custom code](/docs/guides/function-calling) or use built-in [tools](/docs/guides/tools) like [web search](/docs/guides/tools-web-search) or [file search](/docs/guides/tools-file-search) to use your own data as input for the model's response.
    ///
    ///REST: `POST /responses`.
    ///Path constant: `OpenAiApiPath::RESPONSES`.
    async fn create_response(
        &self,
        body: CreateResponse,
    ) -> Result<Response, Self::Error>;
    ///Compact a conversation. Returns a compacted response object. Learn when and how to compact long-running conversations in the [conversation state guide](/docs/guides/conversation-state#managing-the-context-window). For ZDR-compatible compaction details, see [Compaction (advanced)](/docs/guides/conversation-state#compaction-advanced).
    ///
    ///REST: `POST /responses/compact`.
    ///Path constant: `OpenAiApiPath::RESPONSES_BY_COMPACT`.
    async fn compact_conversation(
        &self,
        body: ::std::option::Option<CompactResponseMethodPublicBody>,
    ) -> Result<CompactResource, Self::Error>;
    ///Returns input token counts of the request. Returns an object with `object` set to `response.input_tokens` and an `input_tokens` count.
    ///
    ///REST: `POST /responses/input_tokens`.
    ///Path constant: `OpenAiApiPath::RESPONSES_BY_INPUT_TOKENS`.
    async fn get_input_token_counts(
        &self,
        body: ::std::option::Option<TokenCountsBody>,
    ) -> Result<TokenCountsResource, Self::Error>;
    ///Retrieves a model response with the given ID.
    ///
    ///REST: `GET /responses/{response_id}`.
    ///Path constant: `OpenAiApiPath::RESPONSES_BY_RESPONSE_ID`.
    async fn get_response(
        &self,
        response_id: String,
        include: ::std::option::Option<Vec<IncludeEnum>>,
        stream: ::std::option::Option<bool>,
        starting_after: ::std::option::Option<i32>,
        include_obfuscation: ::std::option::Option<bool>,
    ) -> Result<Response, Self::Error>;
    ///Deletes a model response with the given ID.
    ///
    ///REST: `DELETE /responses/{response_id}`.
    ///Path constant: `OpenAiApiPath::RESPONSES_BY_RESPONSE_ID`.
    async fn delete_response(&self, response_id: String) -> Result<(), Self::Error>;
    ///Cancels a model response with the given ID. Only responses created with the `background` parameter set to `true` can be cancelled. [Learn more](/docs/guides/background).
    ///
    ///REST: `POST /responses/{response_id}/cancel`.
    ///Path constant: `OpenAiApiPath::RESPONSES_BY_RESPONSE_ID_BY_CANCEL`.
    async fn cancel_response(
        &self,
        response_id: String,
    ) -> Result<Response, Self::Error>;
    ///Returns a list of input items for a given response.
    ///
    ///REST: `GET /responses/{response_id}/input_items`.
    ///Path constant: `OpenAiApiPath::RESPONSES_BY_RESPONSE_ID_BY_INPUT_ITEMS`.
    async fn list_input_items(
        &self,
        response_id: String,
        limit: ::std::option::Option<i32>,
        order: ::std::option::Option<String>,
        after: ::std::option::Option<String>,
        include: ::std::option::Option<Vec<IncludeEnum>>,
    ) -> Result<ResponseItemList, Self::Error>;
}
///Roles REST endpoints.
#[::async_trait::async_trait]
pub trait OpenAiRolesApi: Send + Sync {
    type Error: ::std::error::Error + Send + Sync + 'static;
    ///Lists the roles configured for the organization.
    ///
    ///REST: `GET /organization/roles`.
    ///Path constant: `OpenAiApiPath::ORGANIZATION_BY_ROLES`.
    async fn list_roles(
        &self,
        limit: ::std::option::Option<i32>,
        after: ::std::option::Option<String>,
        order: ::std::option::Option<String>,
    ) -> Result<PublicRoleListResource, Self::Error>;
    ///Creates a custom role for the organization.
    ///
    ///REST: `POST /organization/roles`.
    ///Path constant: `OpenAiApiPath::ORGANIZATION_BY_ROLES`.
    async fn create_role(
        &self,
        body: PublicCreateOrganizationRoleBody,
    ) -> Result<Role, Self::Error>;
    ///Updates an existing organization role.
    ///
    ///REST: `POST /organization/roles/{role_id}`.
    ///Path constant: `OpenAiApiPath::ORGANIZATION_BY_ROLES_BY_ROLE_ID`.
    async fn update_role(
        &self,
        role_id: String,
        body: PublicUpdateOrganizationRoleBody,
    ) -> Result<Role, Self::Error>;
    ///Deletes a custom role from the organization.
    ///
    ///REST: `DELETE /organization/roles/{role_id}`.
    ///Path constant: `OpenAiApiPath::ORGANIZATION_BY_ROLES_BY_ROLE_ID`.
    async fn delete_role(
        &self,
        role_id: String,
    ) -> Result<RoleDeletedResource, Self::Error>;
    ///Lists the roles configured for a project.
    ///
    ///REST: `GET /projects/{project_id}/roles`.
    ///Path constant: `OpenAiApiPath::PROJECTS_BY_PROJECT_ID_BY_ROLES`.
    async fn list_project_roles(
        &self,
        project_id: String,
        limit: ::std::option::Option<i32>,
        after: ::std::option::Option<String>,
        order: ::std::option::Option<String>,
    ) -> Result<PublicRoleListResource, Self::Error>;
    ///Creates a custom role for a project.
    ///
    ///REST: `POST /projects/{project_id}/roles`.
    ///Path constant: `OpenAiApiPath::PROJECTS_BY_PROJECT_ID_BY_ROLES`.
    async fn create_project_role(
        &self,
        project_id: String,
        body: PublicCreateOrganizationRoleBody,
    ) -> Result<Role, Self::Error>;
    ///Updates an existing project role.
    ///
    ///REST: `POST /projects/{project_id}/roles/{role_id}`.
    ///Path constant: `OpenAiApiPath::PROJECTS_BY_PROJECT_ID_BY_ROLES_BY_ROLE_ID`.
    async fn update_project_role(
        &self,
        project_id: String,
        role_id: String,
        body: PublicUpdateOrganizationRoleBody,
    ) -> Result<Role, Self::Error>;
    ///Deletes a custom role from a project.
    ///
    ///REST: `DELETE /projects/{project_id}/roles/{role_id}`.
    ///Path constant: `OpenAiApiPath::PROJECTS_BY_PROJECT_ID_BY_ROLES_BY_ROLE_ID`.
    async fn delete_project_role(
        &self,
        project_id: String,
        role_id: String,
    ) -> Result<RoleDeletedResource, Self::Error>;
}
///Skills REST endpoints.
#[::async_trait::async_trait]
pub trait OpenAiSkillsApi: Send + Sync {
    type Error: ::std::error::Error + Send + Sync + 'static;
    ///List all skills for the current project.
    ///
    ///REST: `GET /skills`.
    ///Path constant: `OpenAiApiPath::SKILLS`.
    async fn list_skills(
        &self,
        limit: ::std::option::Option<i32>,
        order: ::std::option::Option<OrderEnum>,
        after: ::std::option::Option<String>,
    ) -> Result<SkillListResource, Self::Error>;
    ///Create a new skill.
    ///
    ///REST: `POST /skills`.
    ///Path constant: `OpenAiApiPath::SKILLS`.
    async fn create_skill(
        &self,
        body: ::std::option::Option<CreateSkillBody>,
    ) -> Result<SkillResource, Self::Error>;
    ///Get a skill by its ID.
    ///
    ///REST: `GET /skills/{skill_id}`.
    ///Path constant: `OpenAiApiPath::SKILLS_BY_SKILL_ID`.
    async fn get_skill(&self, skill_id: String) -> Result<SkillResource, Self::Error>;
    ///Update the default version pointer for a skill.
    ///
    ///REST: `POST /skills/{skill_id}`.
    ///Path constant: `OpenAiApiPath::SKILLS_BY_SKILL_ID`.
    async fn update_skill_default_version(
        &self,
        skill_id: String,
        body: ::std::option::Option<SetDefaultSkillVersionBody>,
    ) -> Result<SkillResource, Self::Error>;
    ///Delete a skill by its ID.
    ///
    ///REST: `DELETE /skills/{skill_id}`.
    ///Path constant: `OpenAiApiPath::SKILLS_BY_SKILL_ID`.
    async fn delete_skill(
        &self,
        skill_id: String,
    ) -> Result<DeletedSkillResource, Self::Error>;
    ///Download a skill zip bundle by its ID.
    ///
    ///REST: `GET /skills/{skill_id}/content`.
    ///Path constant: `OpenAiApiPath::SKILLS_BY_SKILL_ID_BY_CONTENT`.
    async fn get_skill_content(&self, skill_id: String) -> Result<String, Self::Error>;
    ///List skill versions for a skill.
    ///
    ///REST: `GET /skills/{skill_id}/versions`.
    ///Path constant: `OpenAiApiPath::SKILLS_BY_SKILL_ID_BY_VERSIONS`.
    async fn list_skill_versions(
        &self,
        skill_id: String,
        limit: ::std::option::Option<i32>,
        order: ::std::option::Option<OrderEnum>,
        after: ::std::option::Option<String>,
    ) -> Result<SkillVersionListResource, Self::Error>;
    ///Create a new immutable skill version.
    ///
    ///REST: `POST /skills/{skill_id}/versions`.
    ///Path constant: `OpenAiApiPath::SKILLS_BY_SKILL_ID_BY_VERSIONS`.
    async fn create_skill_version(
        &self,
        skill_id: String,
        body: ::std::option::Option<CreateSkillVersionBody>,
    ) -> Result<SkillVersionResource, Self::Error>;
    ///Get a specific skill version.
    ///
    ///REST: `GET /skills/{skill_id}/versions/{version}`.
    ///Path constant: `OpenAiApiPath::SKILLS_BY_SKILL_ID_BY_VERSIONS_BY_VERSION`.
    async fn get_skill_version(
        &self,
        skill_id: String,
        version: String,
    ) -> Result<SkillVersionResource, Self::Error>;
    ///Delete a skill version.
    ///
    ///REST: `DELETE /skills/{skill_id}/versions/{version}`.
    ///Path constant: `OpenAiApiPath::SKILLS_BY_SKILL_ID_BY_VERSIONS_BY_VERSION`.
    async fn delete_skill_version(
        &self,
        skill_id: String,
        version: String,
    ) -> Result<DeletedSkillVersionResource, Self::Error>;
    ///Download a skill version zip bundle.
    ///
    ///REST: `GET /skills/{skill_id}/versions/{version}/content`.
    ///Path constant: `OpenAiApiPath::SKILLS_BY_SKILL_ID_BY_VERSIONS_BY_VERSION_BY_CONTENT`.
    async fn get_skill_version_content(
        &self,
        skill_id: String,
        version: String,
    ) -> Result<String, Self::Error>;
}
///Uploads REST endpoints.
#[::async_trait::async_trait]
pub trait OpenAiUploadsApi: Send + Sync {
    type Error: ::std::error::Error + Send + Sync + 'static;
    ///Creates an intermediate [Upload](/docs/api-reference/uploads/object) object that you can add [Parts](/docs/api-reference/uploads/part-object) to. Currently, an Upload can accept at most 8 GB in total and expires after an hour after you create it. Once you complete the Upload, we will create a [File](/docs/api-reference/files/object) object that contains all the parts you uploaded. This File is usable in the rest of our platform as a regular File object. For certain `purpose` values, the correct `mime_type` must be specified. Please refer to documentation for the [supported MIME types for your use case](/docs/assistants/tools/file-search#supported-files). For guidance on the proper filename extensions for each purpose, please follow the documentation on [creating a File](/docs/api-reference/files/create). Returns the Upload object with status `pending`.
    ///
    ///REST: `POST /uploads`.
    ///Path constant: `OpenAiApiPath::UPLOADS`.
    async fn create_upload(
        &self,
        body: CreateUploadRequest,
    ) -> Result<Upload, Self::Error>;
    ///Cancels the Upload. No Parts may be added after an Upload is cancelled. Returns the Upload object with status `cancelled`.
    ///
    ///REST: `POST /uploads/{upload_id}/cancel`.
    ///Path constant: `OpenAiApiPath::UPLOADS_BY_UPLOAD_ID_BY_CANCEL`.
    async fn cancel_upload(&self, upload_id: String) -> Result<Upload, Self::Error>;
    ///Completes the [Upload](/docs/api-reference/uploads/object). Within the returned Upload object, there is a nested [File](/docs/api-reference/files/object) object that is ready to use in the rest of the platform. You can specify the order of the Parts by passing in an ordered list of the Part IDs. The number of bytes uploaded upon completion must match the number of bytes initially specified when creating the Upload object. No Parts may be added after an Upload is completed. Returns the Upload object with status `completed`, including an additional `file` property containing the created usable File object.
    ///
    ///REST: `POST /uploads/{upload_id}/complete`.
    ///Path constant: `OpenAiApiPath::UPLOADS_BY_UPLOAD_ID_BY_COMPLETE`.
    async fn complete_upload(
        &self,
        upload_id: String,
        body: CompleteUploadRequest,
    ) -> Result<Upload, Self::Error>;
    ///Adds a [Part](/docs/api-reference/uploads/part-object) to an [Upload](/docs/api-reference/uploads/object) object. A Part represents a chunk of bytes from the file you are trying to upload. Each Part can be at most 64 MB, and you can add Parts until you hit the Upload maximum of 8 GB. It is possible to add multiple Parts in parallel. You can decide the intended order of the Parts when you [complete the Upload](/docs/api-reference/uploads/complete).
    ///
    ///REST: `POST /uploads/{upload_id}/parts`.
    ///Path constant: `OpenAiApiPath::UPLOADS_BY_UPLOAD_ID_BY_PARTS`.
    async fn add_upload_part(
        &self,
        upload_id: String,
        body: AddUploadPartRequest,
    ) -> Result<UploadPart, Self::Error>;
}
///Usage REST endpoints.
#[::async_trait::async_trait]
pub trait OpenAiUsageApi: Send + Sync {
    type Error: ::std::error::Error + Send + Sync + 'static;
    ///Get costs details for the organization.
    ///
    ///REST: `GET /organization/costs`.
    ///Path constant: `OpenAiApiPath::ORGANIZATION_BY_COSTS`.
    async fn usage_costs(
        &self,
        start_time: i32,
        end_time: ::std::option::Option<i32>,
        bucket_width: ::std::option::Option<String>,
        project_ids: ::std::option::Option<Vec<String>>,
        api_key_ids: ::std::option::Option<Vec<String>>,
        group_by: ::std::option::Option<Vec<String>>,
        limit: ::std::option::Option<i32>,
        page: ::std::option::Option<String>,
    ) -> Result<UsageResponse, Self::Error>;
    ///Get audio speeches usage details for the organization.
    ///
    ///REST: `GET /organization/usage/audio_speeches`.
    ///Path constant: `OpenAiApiPath::ORGANIZATION_BY_USAGE_BY_AUDIO_SPEECHES`.
    async fn usage_audio_speeches(
        &self,
        start_time: i32,
        end_time: ::std::option::Option<i32>,
        bucket_width: ::std::option::Option<String>,
        project_ids: ::std::option::Option<Vec<String>>,
        user_ids: ::std::option::Option<Vec<String>>,
        api_key_ids: ::std::option::Option<Vec<String>>,
        models: ::std::option::Option<Vec<String>>,
        group_by: ::std::option::Option<Vec<String>>,
        limit: ::std::option::Option<i32>,
        page: ::std::option::Option<String>,
    ) -> Result<UsageResponse, Self::Error>;
    ///Get audio transcriptions usage details for the organization.
    ///
    ///REST: `GET /organization/usage/audio_transcriptions`.
    ///Path constant: `OpenAiApiPath::ORGANIZATION_BY_USAGE_BY_AUDIO_TRANSCRIPTIONS`.
    async fn usage_audio_transcriptions(
        &self,
        start_time: i32,
        end_time: ::std::option::Option<i32>,
        bucket_width: ::std::option::Option<String>,
        project_ids: ::std::option::Option<Vec<String>>,
        user_ids: ::std::option::Option<Vec<String>>,
        api_key_ids: ::std::option::Option<Vec<String>>,
        models: ::std::option::Option<Vec<String>>,
        group_by: ::std::option::Option<Vec<String>>,
        limit: ::std::option::Option<i32>,
        page: ::std::option::Option<String>,
    ) -> Result<UsageResponse, Self::Error>;
    ///Get code interpreter sessions usage details for the organization.
    ///
    ///REST: `GET /organization/usage/code_interpreter_sessions`.
    ///Path constant: `OpenAiApiPath::ORGANIZATION_BY_USAGE_BY_CODE_INTERPRETER_SESSIONS`.
    async fn usage_code_interpreter_sessions(
        &self,
        start_time: i32,
        end_time: ::std::option::Option<i32>,
        bucket_width: ::std::option::Option<String>,
        project_ids: ::std::option::Option<Vec<String>>,
        group_by: ::std::option::Option<Vec<String>>,
        limit: ::std::option::Option<i32>,
        page: ::std::option::Option<String>,
    ) -> Result<UsageResponse, Self::Error>;
    ///Get completions usage details for the organization.
    ///
    ///REST: `GET /organization/usage/completions`.
    ///Path constant: `OpenAiApiPath::ORGANIZATION_BY_USAGE_BY_COMPLETIONS`.
    async fn usage_completions(
        &self,
        start_time: i32,
        end_time: ::std::option::Option<i32>,
        bucket_width: ::std::option::Option<String>,
        project_ids: ::std::option::Option<Vec<String>>,
        user_ids: ::std::option::Option<Vec<String>>,
        api_key_ids: ::std::option::Option<Vec<String>>,
        models: ::std::option::Option<Vec<String>>,
        batch: ::std::option::Option<bool>,
        group_by: ::std::option::Option<Vec<String>>,
        limit: ::std::option::Option<i32>,
        page: ::std::option::Option<String>,
    ) -> Result<UsageResponse, Self::Error>;
    ///Get embeddings usage details for the organization.
    ///
    ///REST: `GET /organization/usage/embeddings`.
    ///Path constant: `OpenAiApiPath::ORGANIZATION_BY_USAGE_BY_EMBEDDINGS`.
    async fn usage_embeddings(
        &self,
        start_time: i32,
        end_time: ::std::option::Option<i32>,
        bucket_width: ::std::option::Option<String>,
        project_ids: ::std::option::Option<Vec<String>>,
        user_ids: ::std::option::Option<Vec<String>>,
        api_key_ids: ::std::option::Option<Vec<String>>,
        models: ::std::option::Option<Vec<String>>,
        group_by: ::std::option::Option<Vec<String>>,
        limit: ::std::option::Option<i32>,
        page: ::std::option::Option<String>,
    ) -> Result<UsageResponse, Self::Error>;
    ///Get images usage details for the organization.
    ///
    ///REST: `GET /organization/usage/images`.
    ///Path constant: `OpenAiApiPath::ORGANIZATION_BY_USAGE_BY_IMAGES`.
    async fn usage_images(
        &self,
        start_time: i32,
        end_time: ::std::option::Option<i32>,
        bucket_width: ::std::option::Option<String>,
        sources: ::std::option::Option<Vec<String>>,
        sizes: ::std::option::Option<Vec<String>>,
        project_ids: ::std::option::Option<Vec<String>>,
        user_ids: ::std::option::Option<Vec<String>>,
        api_key_ids: ::std::option::Option<Vec<String>>,
        models: ::std::option::Option<Vec<String>>,
        group_by: ::std::option::Option<Vec<String>>,
        limit: ::std::option::Option<i32>,
        page: ::std::option::Option<String>,
    ) -> Result<UsageResponse, Self::Error>;
    ///Get moderations usage details for the organization.
    ///
    ///REST: `GET /organization/usage/moderations`.
    ///Path constant: `OpenAiApiPath::ORGANIZATION_BY_USAGE_BY_MODERATIONS`.
    async fn usage_moderations(
        &self,
        start_time: i32,
        end_time: ::std::option::Option<i32>,
        bucket_width: ::std::option::Option<String>,
        project_ids: ::std::option::Option<Vec<String>>,
        user_ids: ::std::option::Option<Vec<String>>,
        api_key_ids: ::std::option::Option<Vec<String>>,
        models: ::std::option::Option<Vec<String>>,
        group_by: ::std::option::Option<Vec<String>>,
        limit: ::std::option::Option<i32>,
        page: ::std::option::Option<String>,
    ) -> Result<UsageResponse, Self::Error>;
    ///Get vector stores usage details for the organization.
    ///
    ///REST: `GET /organization/usage/vector_stores`.
    ///Path constant: `OpenAiApiPath::ORGANIZATION_BY_USAGE_BY_VECTOR_STORES`.
    async fn usage_vector_stores(
        &self,
        start_time: i32,
        end_time: ::std::option::Option<i32>,
        bucket_width: ::std::option::Option<String>,
        project_ids: ::std::option::Option<Vec<String>>,
        group_by: ::std::option::Option<Vec<String>>,
        limit: ::std::option::Option<i32>,
        page: ::std::option::Option<String>,
    ) -> Result<UsageResponse, Self::Error>;
}
///UserOrganizationRoleAssignments REST endpoints.
#[::async_trait::async_trait]
pub trait OpenAiUserOrganizationRoleAssignmentsApi: Send + Sync {
    type Error: ::std::error::Error + Send + Sync + 'static;
    ///Lists the organization roles assigned to a user within the organization.
    ///
    ///REST: `GET /organization/users/{user_id}/roles`.
    ///Path constant: `OpenAiApiPath::ORGANIZATION_BY_USERS_BY_USER_ID_BY_ROLES`.
    async fn list_user_role_assignments(
        &self,
        user_id: String,
        limit: ::std::option::Option<i32>,
        after: ::std::option::Option<String>,
        order: ::std::option::Option<String>,
    ) -> Result<RoleListResource, Self::Error>;
    ///Assigns an organization role to a user within the organization.
    ///
    ///REST: `POST /organization/users/{user_id}/roles`.
    ///Path constant: `OpenAiApiPath::ORGANIZATION_BY_USERS_BY_USER_ID_BY_ROLES`.
    async fn assign_user_role(
        &self,
        user_id: String,
        body: PublicAssignOrganizationGroupRoleBody,
    ) -> Result<UserRoleAssignment, Self::Error>;
    ///Unassigns an organization role from a user within the organization.
    ///
    ///REST: `DELETE /organization/users/{user_id}/roles/{role_id}`.
    ///Path constant: `OpenAiApiPath::ORGANIZATION_BY_USERS_BY_USER_ID_BY_ROLES_BY_ROLE_ID`.
    async fn unassign_user_role(
        &self,
        user_id: String,
        role_id: String,
    ) -> Result<DeletedRoleAssignmentResource, Self::Error>;
}
///Users REST endpoints.
#[::async_trait::async_trait]
pub trait OpenAiUsersApi: Send + Sync {
    type Error: ::std::error::Error + Send + Sync + 'static;
    ///Lists all of the users in the organization.
    ///
    ///REST: `GET /organization/users`.
    ///Path constant: `OpenAiApiPath::ORGANIZATION_BY_USERS`.
    async fn list_users(
        &self,
        limit: ::std::option::Option<i32>,
        after: ::std::option::Option<String>,
        emails: ::std::option::Option<Vec<String>>,
    ) -> Result<UserListResponse, Self::Error>;
    ///Retrieves a user by their identifier.
    ///
    ///REST: `GET /organization/users/{user_id}`.
    ///Path constant: `OpenAiApiPath::ORGANIZATION_BY_USERS_BY_USER_ID`.
    async fn retrieve_user(&self, user_id: String) -> Result<User, Self::Error>;
    ///Modifies a user's role in the organization.
    ///
    ///REST: `POST /organization/users/{user_id}`.
    ///Path constant: `OpenAiApiPath::ORGANIZATION_BY_USERS_BY_USER_ID`.
    async fn modify_user(
        &self,
        user_id: String,
        body: UserRoleUpdateRequest,
    ) -> Result<User, Self::Error>;
    ///Deletes a user from the organization.
    ///
    ///REST: `DELETE /organization/users/{user_id}`.
    ///Path constant: `OpenAiApiPath::ORGANIZATION_BY_USERS_BY_USER_ID`.
    async fn delete_user(
        &self,
        user_id: String,
    ) -> Result<UserDeleteResponse, Self::Error>;
}
///VectorStores REST endpoints.
#[::async_trait::async_trait]
pub trait OpenAiVectorStoresApi: Send + Sync {
    type Error: ::std::error::Error + Send + Sync + 'static;
    ///Returns a list of vector stores.
    ///
    ///REST: `GET /vector_stores`.
    ///Path constant: `OpenAiApiPath::VECTOR_STORES`.
    async fn list_vector_stores(
        &self,
        limit: ::std::option::Option<i32>,
        order: ::std::option::Option<String>,
        after: ::std::option::Option<String>,
        before: ::std::option::Option<String>,
    ) -> Result<ListVectorStoresResponse, Self::Error>;
    ///Create a vector store.
    ///
    ///REST: `POST /vector_stores`.
    ///Path constant: `OpenAiApiPath::VECTOR_STORES`.
    async fn create_vector_store(
        &self,
        body: CreateVectorStoreRequest,
    ) -> Result<VectorStoreObject, Self::Error>;
    ///Retrieves a vector store.
    ///
    ///REST: `GET /vector_stores/{vector_store_id}`.
    ///Path constant: `OpenAiApiPath::VECTOR_STORES_BY_VECTOR_STORE_ID`.
    async fn get_vector_store(
        &self,
        vector_store_id: String,
    ) -> Result<VectorStoreObject, Self::Error>;
    ///Modifies a vector store.
    ///
    ///REST: `POST /vector_stores/{vector_store_id}`.
    ///Path constant: `OpenAiApiPath::VECTOR_STORES_BY_VECTOR_STORE_ID`.
    async fn modify_vector_store(
        &self,
        vector_store_id: String,
        body: UpdateVectorStoreRequest,
    ) -> Result<VectorStoreObject, Self::Error>;
    ///Delete a vector store.
    ///
    ///REST: `DELETE /vector_stores/{vector_store_id}`.
    ///Path constant: `OpenAiApiPath::VECTOR_STORES_BY_VECTOR_STORE_ID`.
    async fn delete_vector_store(
        &self,
        vector_store_id: String,
    ) -> Result<DeleteVectorStoreResponse, Self::Error>;
    ///Create a vector store file batch.
    ///
    ///REST: `POST /vector_stores/{vector_store_id}/file_batches`.
    ///Path constant: `OpenAiApiPath::VECTOR_STORES_BY_VECTOR_STORE_ID_BY_FILE_BATCHES`.
    async fn create_vector_store_file_batch(
        &self,
        vector_store_id: String,
        body: CreateVectorStoreFileBatchRequest,
    ) -> Result<VectorStoreFileBatchObject, Self::Error>;
    ///Retrieves a vector store file batch.
    ///
    ///REST: `GET /vector_stores/{vector_store_id}/file_batches/{batch_id}`.
    ///Path constant: `OpenAiApiPath::VECTOR_STORES_BY_VECTOR_STORE_ID_BY_FILE_BATCHES_BY_BATCH_ID`.
    async fn get_vector_store_file_batch(
        &self,
        vector_store_id: String,
        batch_id: String,
    ) -> Result<VectorStoreFileBatchObject, Self::Error>;
    ///Cancel a vector store file batch. This attempts to cancel the processing of files in this batch as soon as possible.
    ///
    ///REST: `POST /vector_stores/{vector_store_id}/file_batches/{batch_id}/cancel`.
    ///Path constant: `OpenAiApiPath::VECTOR_STORES_BY_VECTOR_STORE_ID_BY_FILE_BATCHES_BY_BATCH_ID_BY_CANCEL`.
    async fn cancel_vector_store_file_batch(
        &self,
        vector_store_id: String,
        batch_id: String,
    ) -> Result<VectorStoreFileBatchObject, Self::Error>;
    ///Returns a list of vector store files in a batch.
    ///
    ///REST: `GET /vector_stores/{vector_store_id}/file_batches/{batch_id}/files`.
    ///Path constant: `OpenAiApiPath::VECTOR_STORES_BY_VECTOR_STORE_ID_BY_FILE_BATCHES_BY_BATCH_ID_BY_FILES`.
    async fn list_files_in_vector_store_batch(
        &self,
        vector_store_id: String,
        batch_id: String,
        limit: ::std::option::Option<i32>,
        order: ::std::option::Option<String>,
        after: ::std::option::Option<String>,
        before: ::std::option::Option<String>,
        filter: ::std::option::Option<String>,
    ) -> Result<ListVectorStoreFilesResponse, Self::Error>;
    ///Returns a list of vector store files.
    ///
    ///REST: `GET /vector_stores/{vector_store_id}/files`.
    ///Path constant: `OpenAiApiPath::VECTOR_STORES_BY_VECTOR_STORE_ID_BY_FILES`.
    async fn list_vector_store_files(
        &self,
        vector_store_id: String,
        limit: ::std::option::Option<i32>,
        order: ::std::option::Option<String>,
        after: ::std::option::Option<String>,
        before: ::std::option::Option<String>,
        filter: ::std::option::Option<String>,
    ) -> Result<ListVectorStoreFilesResponse, Self::Error>;
    ///Create a vector store file by attaching a [File](/docs/api-reference/files) to a [vector store](/docs/api-reference/vector-stores/object).
    ///
    ///REST: `POST /vector_stores/{vector_store_id}/files`.
    ///Path constant: `OpenAiApiPath::VECTOR_STORES_BY_VECTOR_STORE_ID_BY_FILES`.
    async fn create_vector_store_file(
        &self,
        vector_store_id: String,
        body: CreateVectorStoreFileRequest,
    ) -> Result<VectorStoreFileObject, Self::Error>;
    ///Retrieves a vector store file.
    ///
    ///REST: `GET /vector_stores/{vector_store_id}/files/{file_id}`.
    ///Path constant: `OpenAiApiPath::VECTOR_STORES_BY_VECTOR_STORE_ID_BY_FILES_BY_FILE_ID`.
    async fn get_vector_store_file(
        &self,
        vector_store_id: String,
        file_id: String,
    ) -> Result<VectorStoreFileObject, Self::Error>;
    ///Update attributes on a vector store file.
    ///
    ///REST: `POST /vector_stores/{vector_store_id}/files/{file_id}`.
    ///Path constant: `OpenAiApiPath::VECTOR_STORES_BY_VECTOR_STORE_ID_BY_FILES_BY_FILE_ID`.
    async fn update_vector_store_file_attributes(
        &self,
        vector_store_id: String,
        file_id: String,
        body: UpdateVectorStoreFileAttributesRequest,
    ) -> Result<VectorStoreFileObject, Self::Error>;
    ///Delete a vector store file. This will remove the file from the vector store but the file itself will not be deleted. To delete the file, use the [delete file](/docs/api-reference/files/delete) endpoint.
    ///
    ///REST: `DELETE /vector_stores/{vector_store_id}/files/{file_id}`.
    ///Path constant: `OpenAiApiPath::VECTOR_STORES_BY_VECTOR_STORE_ID_BY_FILES_BY_FILE_ID`.
    async fn delete_vector_store_file(
        &self,
        vector_store_id: String,
        file_id: String,
    ) -> Result<DeleteVectorStoreFileResponse, Self::Error>;
    ///Retrieve the parsed contents of a vector store file.
    ///
    ///REST: `GET /vector_stores/{vector_store_id}/files/{file_id}/content`.
    ///Path constant: `OpenAiApiPath::VECTOR_STORES_BY_VECTOR_STORE_ID_BY_FILES_BY_FILE_ID_BY_CONTENT`.
    async fn retrieve_vector_store_file_content(
        &self,
        vector_store_id: String,
        file_id: String,
    ) -> Result<VectorStoreFileContentResponse, Self::Error>;
    ///Search a vector store for relevant chunks based on a query and file attributes filter.
    ///
    ///REST: `POST /vector_stores/{vector_store_id}/search`.
    ///Path constant: `OpenAiApiPath::VECTOR_STORES_BY_VECTOR_STORE_ID_BY_SEARCH`.
    async fn search_vector_store(
        &self,
        vector_store_id: String,
        body: VectorStoreSearchRequest,
    ) -> Result<VectorStoreSearchResultsPage, Self::Error>;
}
///Videos REST endpoints.
#[::async_trait::async_trait]
pub trait OpenAiVideosApi: Send + Sync {
    type Error: ::std::error::Error + Send + Sync + 'static;
    ///List recently generated videos for the current project.
    ///
    ///REST: `GET /videos`.
    ///Path constant: `OpenAiApiPath::VIDEOS`.
    async fn list_videos(
        &self,
        limit: ::std::option::Option<i32>,
        order: ::std::option::Option<OrderEnum>,
        after: ::std::option::Option<String>,
    ) -> Result<VideoListResource, Self::Error>;
    ///Create a new video generation job from a prompt and optional reference assets.
    ///
    ///REST: `POST /videos`.
    ///Path constant: `OpenAiApiPath::VIDEOS`.
    async fn create_video(
        &self,
        body: ::std::option::Option<CreateVideoJsonBody>,
    ) -> Result<VideoResource, Self::Error>;
    ///Create a character from an uploaded video.
    ///
    ///REST: `POST /videos/characters`.
    ///Path constant: `OpenAiApiPath::VIDEOS_BY_CHARACTERS`.
    async fn create_video_character(
        &self,
        body: ::std::option::Option<CreateVideoCharacterBody>,
    ) -> Result<VideoCharacterResource, Self::Error>;
    ///Fetch a character.
    ///
    ///REST: `GET /videos/characters/{character_id}`.
    ///Path constant: `OpenAiApiPath::VIDEOS_BY_CHARACTERS_BY_CHARACTER_ID`.
    async fn get_video_character(
        &self,
        character_id: String,
    ) -> Result<VideoCharacterResource, Self::Error>;
    ///Create a new video generation job by editing a source video or existing generated video.
    ///
    ///REST: `POST /videos/edits`.
    ///Path constant: `OpenAiApiPath::VIDEOS_BY_EDITS`.
    async fn create_video_edit(
        &self,
        body: ::std::option::Option<CreateVideoEditJsonBody>,
    ) -> Result<VideoResource, Self::Error>;
    ///Create an extension of a completed video.
    ///
    ///REST: `POST /videos/extensions`.
    ///Path constant: `OpenAiApiPath::VIDEOS_BY_EXTENSIONS`.
    async fn create_video_extend(
        &self,
        body: ::std::option::Option<CreateVideoExtendJsonBody>,
    ) -> Result<VideoResource, Self::Error>;
    ///Fetch the latest metadata for a generated video.
    ///
    ///REST: `GET /videos/{video_id}`.
    ///Path constant: `OpenAiApiPath::VIDEOS_BY_VIDEO_ID`.
    async fn get_video(&self, video_id: String) -> Result<VideoResource, Self::Error>;
    ///Permanently delete a completed or failed video and its stored assets.
    ///
    ///REST: `DELETE /videos/{video_id}`.
    ///Path constant: `OpenAiApiPath::VIDEOS_BY_VIDEO_ID`.
    async fn delete_video(
        &self,
        video_id: String,
    ) -> Result<DeletedVideoResource, Self::Error>;
    ///Download the generated video bytes or a derived preview asset. Streams the rendered video content for the specified video job.
    ///
    ///REST: `GET /videos/{video_id}/content`.
    ///Path constant: `OpenAiApiPath::VIDEOS_BY_VIDEO_ID_BY_CONTENT`.
    async fn retrieve_video_content(
        &self,
        video_id: String,
        variant: ::std::option::Option<VideoContentVariant>,
    ) -> Result<String, Self::Error>;
    ///Create a remix of a completed video using a refreshed prompt.
    ///
    ///REST: `POST /videos/{video_id}/remix`.
    ///Path constant: `OpenAiApiPath::VIDEOS_BY_VIDEO_ID_BY_REMIX`.
    async fn create_video_remix(
        &self,
        video_id: String,
        body: ::std::option::Option<CreateVideoRemixBody>,
    ) -> Result<VideoResource, Self::Error>;
}
