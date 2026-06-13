// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! Assistants REST endpoint contract.

use async_trait::async_trait;

use crate::models::{
    AssistantObject,
    CreateAssistantRequest,
    CreateMessageRequest,
    CreateRunRequest,
    CreateThreadAndRunRequest,
    CreateThreadRequest,
    DeleteAssistantResponse,
    DeleteMessageResponse,
    DeleteThreadResponse,
    ListAssistantsResponse,
    ListMessagesResponse,
    ListRunStepsResponse,
    ListRunsResponse,
    MessageObject,
    ModifyAssistantRequest,
    ModifyMessageRequest,
    ModifyRunRequest,
    ModifyThreadRequest,
    RunObject,
    RunStepObject,
    SubmitToolOutputsRunRequest,
    ThreadObject,
};

/// Assistants REST endpoints.
#[async_trait]
pub trait OpenAiAssistantsApi: Send + Sync {
    /// Error type returned by the application-layer implementation.
    type Error: std::error::Error + Send + Sync + 'static;

    /// Returns a list of assistants.
    ///
    /// REST: `GET /assistants`.
    /// Path constant: [`OpenAiApiPath::ASSISTANTS`](crate::paths::OpenAiApiPath::ASSISTANTS).
    async fn list_assistants(
        &self,
        limit: Option<i32>,
        order: Option<String>,
        after: Option<String>,
        before: Option<String>,
    ) -> Result<ListAssistantsResponse, Self::Error>;

    /// Create an assistant with a model and instructions.
    ///
    /// REST: `POST /assistants`.
    /// Path constant: [`OpenAiApiPath::ASSISTANTS`](crate::paths::OpenAiApiPath::ASSISTANTS).
    async fn create_assistant(
        &self,
        body: CreateAssistantRequest,
    ) -> Result<AssistantObject, Self::Error>;

    /// Retrieves an assistant.
    ///
    /// REST: `GET /assistants/{assistant_id}`.
    /// Path constant: [`OpenAiApiPath::ASSISTANTS_BY_ASSISTANT_ID`](crate::paths::OpenAiApiPath::ASSISTANTS_BY_ASSISTANT_ID).
    async fn get_assistant(&self, assistant_id: String) -> Result<AssistantObject, Self::Error>;

    /// Modifies an assistant.
    ///
    /// REST: `POST /assistants/{assistant_id}`.
    /// Path constant: [`OpenAiApiPath::ASSISTANTS_BY_ASSISTANT_ID`](crate::paths::OpenAiApiPath::ASSISTANTS_BY_ASSISTANT_ID).
    async fn modify_assistant(
        &self,
        assistant_id: String,
        body: ModifyAssistantRequest,
    ) -> Result<AssistantObject, Self::Error>;

    /// Delete an assistant.
    ///
    /// REST: `DELETE /assistants/{assistant_id}`.
    /// Path constant: [`OpenAiApiPath::ASSISTANTS_BY_ASSISTANT_ID`](crate::paths::OpenAiApiPath::ASSISTANTS_BY_ASSISTANT_ID).
    async fn delete_assistant(
        &self,
        assistant_id: String,
    ) -> Result<DeleteAssistantResponse, Self::Error>;

    /// Create a thread.
    ///
    /// REST: `POST /threads`.
    /// Path constant: [`OpenAiApiPath::THREADS`](crate::paths::OpenAiApiPath::THREADS).
    async fn create_thread(
        &self,
        body: Option<CreateThreadRequest>,
    ) -> Result<ThreadObject, Self::Error>;

    /// Create a thread and run it in one request.
    ///
    /// REST: `POST /threads/runs`.
    /// Path constant: [`OpenAiApiPath::THREADS_BY_RUNS`](crate::paths::OpenAiApiPath::THREADS_BY_RUNS).
    async fn create_thread_and_run(
        &self,
        body: CreateThreadAndRunRequest,
    ) -> Result<RunObject, Self::Error>;

    /// Retrieves a thread.
    ///
    /// REST: `GET /threads/{thread_id}`.
    /// Path constant: [`OpenAiApiPath::THREADS_BY_THREAD_ID`](crate::paths::OpenAiApiPath::THREADS_BY_THREAD_ID).
    async fn get_thread(&self, thread_id: String) -> Result<ThreadObject, Self::Error>;

    /// Modifies a thread.
    ///
    /// REST: `POST /threads/{thread_id}`.
    /// Path constant: [`OpenAiApiPath::THREADS_BY_THREAD_ID`](crate::paths::OpenAiApiPath::THREADS_BY_THREAD_ID).
    async fn modify_thread(
        &self,
        thread_id: String,
        body: ModifyThreadRequest,
    ) -> Result<ThreadObject, Self::Error>;

    /// Delete a thread.
    ///
    /// REST: `DELETE /threads/{thread_id}`.
    /// Path constant: [`OpenAiApiPath::THREADS_BY_THREAD_ID`](crate::paths::OpenAiApiPath::THREADS_BY_THREAD_ID).
    async fn delete_thread(&self, thread_id: String) -> Result<DeleteThreadResponse, Self::Error>;

    /// Returns a list of messages for a given thread.
    ///
    /// REST: `GET /threads/{thread_id}/messages`.
    /// Path constant: [`OpenAiApiPath::THREADS_BY_THREAD_ID_BY_MESSAGES`](crate::paths::OpenAiApiPath::THREADS_BY_THREAD_ID_BY_MESSAGES).
    async fn list_messages(
        &self,
        thread_id: String,
        limit: Option<i32>,
        order: Option<String>,
        after: Option<String>,
        before: Option<String>,
        run_id: Option<String>,
    ) -> Result<ListMessagesResponse, Self::Error>;

    /// Create a message.
    ///
    /// REST: `POST /threads/{thread_id}/messages`.
    /// Path constant: [`OpenAiApiPath::THREADS_BY_THREAD_ID_BY_MESSAGES`](crate::paths::OpenAiApiPath::THREADS_BY_THREAD_ID_BY_MESSAGES).
    async fn create_message(
        &self,
        thread_id: String,
        body: CreateMessageRequest,
    ) -> Result<MessageObject, Self::Error>;

    /// Retrieve a message.
    ///
    /// REST: `GET /threads/{thread_id}/messages/{message_id}`.
    /// Path constant: [`OpenAiApiPath::THREADS_BY_THREAD_ID_BY_MESSAGES_BY_MESSAGE_ID`](crate::paths::OpenAiApiPath::THREADS_BY_THREAD_ID_BY_MESSAGES_BY_MESSAGE_ID).
    async fn get_message(
        &self,
        thread_id: String,
        message_id: String,
    ) -> Result<MessageObject, Self::Error>;

    /// Modifies a message.
    ///
    /// REST: `POST /threads/{thread_id}/messages/{message_id}`.
    /// Path constant: [`OpenAiApiPath::THREADS_BY_THREAD_ID_BY_MESSAGES_BY_MESSAGE_ID`](crate::paths::OpenAiApiPath::THREADS_BY_THREAD_ID_BY_MESSAGES_BY_MESSAGE_ID).
    async fn modify_message(
        &self,
        thread_id: String,
        message_id: String,
        body: ModifyMessageRequest,
    ) -> Result<MessageObject, Self::Error>;

    /// Deletes a message.
    ///
    /// REST: `DELETE /threads/{thread_id}/messages/{message_id}`.
    /// Path constant: [`OpenAiApiPath::THREADS_BY_THREAD_ID_BY_MESSAGES_BY_MESSAGE_ID`](crate::paths::OpenAiApiPath::THREADS_BY_THREAD_ID_BY_MESSAGES_BY_MESSAGE_ID).
    async fn delete_message(
        &self,
        thread_id: String,
        message_id: String,
    ) -> Result<DeleteMessageResponse, Self::Error>;

    /// Returns a list of runs belonging to a thread.
    ///
    /// REST: `GET /threads/{thread_id}/runs`.
    /// Path constant: [`OpenAiApiPath::THREADS_BY_THREAD_ID_BY_RUNS`](crate::paths::OpenAiApiPath::THREADS_BY_THREAD_ID_BY_RUNS).
    async fn list_runs(
        &self,
        thread_id: String,
        limit: Option<i32>,
        order: Option<String>,
        after: Option<String>,
        before: Option<String>,
    ) -> Result<ListRunsResponse, Self::Error>;

    /// Create a run.
    ///
    /// REST: `POST /threads/{thread_id}/runs`.
    /// Path constant: [`OpenAiApiPath::THREADS_BY_THREAD_ID_BY_RUNS`](crate::paths::OpenAiApiPath::THREADS_BY_THREAD_ID_BY_RUNS).
    async fn create_run(
        &self,
        thread_id: String,
        include: Option<Vec<String>>,
        body: CreateRunRequest,
    ) -> Result<RunObject, Self::Error>;

    /// Retrieves a run.
    ///
    /// REST: `GET /threads/{thread_id}/runs/{run_id}`.
    /// Path constant: [`OpenAiApiPath::THREADS_BY_THREAD_ID_BY_RUNS_BY_RUN_ID`](crate::paths::OpenAiApiPath::THREADS_BY_THREAD_ID_BY_RUNS_BY_RUN_ID).
    async fn get_run(&self, thread_id: String, run_id: String) -> Result<RunObject, Self::Error>;

    /// Modifies a run.
    ///
    /// REST: `POST /threads/{thread_id}/runs/{run_id}`.
    /// Path constant: [`OpenAiApiPath::THREADS_BY_THREAD_ID_BY_RUNS_BY_RUN_ID`](crate::paths::OpenAiApiPath::THREADS_BY_THREAD_ID_BY_RUNS_BY_RUN_ID).
    async fn modify_run(
        &self,
        thread_id: String,
        run_id: String,
        body: ModifyRunRequest,
    ) -> Result<RunObject, Self::Error>;

    /// Cancels a run that is `in_progress`.
    ///
    /// REST: `POST /threads/{thread_id}/runs/{run_id}/cancel`.
    /// Path constant: [`OpenAiApiPath::THREADS_BY_THREAD_ID_BY_RUNS_BY_RUN_ID_BY_CANCEL`](crate::paths::OpenAiApiPath::THREADS_BY_THREAD_ID_BY_RUNS_BY_RUN_ID_BY_CANCEL).
    async fn cancel_run(&self, thread_id: String, run_id: String) -> Result<RunObject, Self::Error>;

    /// Returns a list of run steps belonging to a run.
    ///
    /// REST: `GET /threads/{thread_id}/runs/{run_id}/steps`.
    /// Path constant: [`OpenAiApiPath::THREADS_BY_THREAD_ID_BY_RUNS_BY_RUN_ID_BY_STEPS`](crate::paths::OpenAiApiPath::THREADS_BY_THREAD_ID_BY_RUNS_BY_RUN_ID_BY_STEPS).
    async fn list_run_steps(
        &self,
        thread_id: String,
        run_id: String,
        limit: Option<i32>,
        order: Option<String>,
        after: Option<String>,
        before: Option<String>,
        include: Option<Vec<String>>,
    ) -> Result<ListRunStepsResponse, Self::Error>;

    /// Retrieves a run step.
    ///
    /// REST: `GET /threads/{thread_id}/runs/{run_id}/steps/{step_id}`.
    /// Path constant: [`OpenAiApiPath::THREADS_BY_THREAD_ID_BY_RUNS_BY_RUN_ID_BY_STEPS_BY_STEP_ID`](crate::paths::OpenAiApiPath::THREADS_BY_THREAD_ID_BY_RUNS_BY_RUN_ID_BY_STEPS_BY_STEP_ID).
    async fn get_run_step(
        &self,
        thread_id: String,
        run_id: String,
        step_id: String,
        include: Option<Vec<String>>,
    ) -> Result<RunStepObject, Self::Error>;

    /// When a run has the `status: "requires_action"` and `required_action.type` is `submit_tool_outputs`,
    /// this endpoint can be used to submit the outputs from the tool calls once they're all completed. All
    /// outputs must be submitted in a single request.
    ///
    /// REST: `POST /threads/{thread_id}/runs/{run_id}/submit_tool_outputs`.
    /// Path constant: [`OpenAiApiPath::THREADS_BY_THREAD_ID_BY_RUNS_BY_RUN_ID_BY_SUBMIT_TOOL_OUTPUTS`](crate::paths::OpenAiApiPath::THREADS_BY_THREAD_ID_BY_RUNS_BY_RUN_ID_BY_SUBMIT_TOOL_OUTPUTS).
    async fn submit_tool_ouputs_to_run(
        &self,
        thread_id: String,
        run_id: String,
        body: SubmitToolOutputsRunRequest,
    ) -> Result<RunObject, Self::Error>;
}
