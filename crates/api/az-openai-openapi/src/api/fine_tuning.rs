// Generated from OpenAPI spec. Do not edit by hand.
//! FineTuning REST endpoint contract.

use async_trait::async_trait;

use crate::models::{
    CreateFineTuningCheckpointPermissionRequest,
    CreateFineTuningJobRequest,
    DeleteFineTuningCheckpointPermissionResponse,
    FineTuningJob,
    ListFineTuningCheckpointPermissionResponse,
    ListFineTuningJobCheckpointsResponse,
    ListFineTuningJobEventsResponse,
    ListPaginatedFineTuningJobsResponse,
    RunGraderRequest,
    RunGraderResponse,
    ValidateGraderRequest,
    ValidateGraderResponse,
};

/// FineTuning REST endpoints.
#[async_trait]
pub trait OpenAiFineTuningApi: Send + Sync {
    /// Error type returned by the application-layer implementation.
    type Error: std::error::Error + Send + Sync + 'static;

    /// Run a grader.
    ///
    /// REST: `POST /fine_tuning/alpha/graders/run`.
    /// Path constant: [`OpenAiApiPath::FINE_TUNING_BY_ALPHA_BY_GRADERS_BY_RUN`](crate::paths::OpenAiApiPath::FINE_TUNING_BY_ALPHA_BY_GRADERS_BY_RUN).
    async fn run_grader(&self, body: RunGraderRequest) -> Result<RunGraderResponse, Self::Error>;

    /// Validate a grader.
    ///
    /// REST: `POST /fine_tuning/alpha/graders/validate`.
    /// Path constant: [`OpenAiApiPath::FINE_TUNING_BY_ALPHA_BY_GRADERS_BY_VALIDATE`](crate::paths::OpenAiApiPath::FINE_TUNING_BY_ALPHA_BY_GRADERS_BY_VALIDATE).
    async fn validate_grader(
        &self,
        body: ValidateGraderRequest,
    ) -> Result<ValidateGraderResponse, Self::Error>;

    /// **NOTE:** This endpoint requires an [admin API key](../admin-api-keys). Organization owners can use
    /// this endpoint to view all permissions for a fine-tuned model checkpoint.
    ///
    /// REST: `GET /fine_tuning/checkpoints/{fine_tuned_model_checkpoint}/permissions`.
    /// Path constant: [`OpenAiApiPath::FINE_TUNING_BY_CHECKPOINTS_BY_FINE_TUNED_MODEL_CHECKPOINT_BY_PERMISSIONS`](crate::paths::OpenAiApiPath::FINE_TUNING_BY_CHECKPOINTS_BY_FINE_TUNED_MODEL_CHECKPOINT_BY_PERMISSIONS).
    async fn list_fine_tuning_checkpoint_permissions(
        &self,
        fine_tuned_model_checkpoint: String,
        project_id: Option<String>,
        after: Option<String>,
        limit: Option<i32>,
        order: Option<String>,
    ) -> Result<ListFineTuningCheckpointPermissionResponse, Self::Error>;

    /// **NOTE:** Calling this endpoint requires an [admin API key](../admin-api-keys). This enables
    /// organization owners to share fine-tuned models with other projects in their organization.
    ///
    /// REST: `POST /fine_tuning/checkpoints/{fine_tuned_model_checkpoint}/permissions`.
    /// Path constant: [`OpenAiApiPath::FINE_TUNING_BY_CHECKPOINTS_BY_FINE_TUNED_MODEL_CHECKPOINT_BY_PERMISSIONS`](crate::paths::OpenAiApiPath::FINE_TUNING_BY_CHECKPOINTS_BY_FINE_TUNED_MODEL_CHECKPOINT_BY_PERMISSIONS).
    async fn create_fine_tuning_checkpoint_permission(
        &self,
        fine_tuned_model_checkpoint: String,
        body: CreateFineTuningCheckpointPermissionRequest,
    ) -> Result<ListFineTuningCheckpointPermissionResponse, Self::Error>;

    /// **NOTE:** This endpoint requires an [admin API key](../admin-api-keys). Organization owners can use
    /// this endpoint to delete a permission for a fine-tuned model checkpoint.
    ///
    /// REST: `DELETE /fine_tuning/checkpoints/{fine_tuned_model_checkpoint}/permissions/{permission_id}`.
    /// Path constant: [`OpenAiApiPath::FINE_TUNING_BY_CHECKPOINTS_BY_FINE_TUNED_MODEL_CHECKPOINT_BY_PERMISSIONS_BY_PERMISSION_ID`](crate::paths::OpenAiApiPath::FINE_TUNING_BY_CHECKPOINTS_BY_FINE_TUNED_MODEL_CHECKPOINT_BY_PERMISSIONS_BY_PERMISSION_ID).
    async fn delete_fine_tuning_checkpoint_permission(
        &self,
        fine_tuned_model_checkpoint: String,
        permission_id: String,
    ) -> Result<DeleteFineTuningCheckpointPermissionResponse, Self::Error>;

    /// List your organization's fine-tuning jobs
    ///
    /// REST: `GET /fine_tuning/jobs`.
    /// Path constant: [`OpenAiApiPath::FINE_TUNING_BY_JOBS`](crate::paths::OpenAiApiPath::FINE_TUNING_BY_JOBS).
    async fn list_paginated_fine_tuning_jobs(
        &self,
        after: Option<String>,
        limit: Option<i32>,
        metadata: Option<std::collections::BTreeMap<String, String>>,
    ) -> Result<ListPaginatedFineTuningJobsResponse, Self::Error>;

    /// Creates a fine-tuning job which begins the process of creating a new model from a given dataset.
    /// Response includes details of the enqueued job including job status and the name of the fine-tuned
    /// models once complete. [Learn more about fine-tuning](/docs/guides/model-optimization)
    ///
    /// REST: `POST /fine_tuning/jobs`.
    /// Path constant: [`OpenAiApiPath::FINE_TUNING_BY_JOBS`](crate::paths::OpenAiApiPath::FINE_TUNING_BY_JOBS).
    async fn create_fine_tuning_job(
        &self,
        body: CreateFineTuningJobRequest,
    ) -> Result<FineTuningJob, Self::Error>;

    /// Get info about a fine-tuning job. [Learn more about fine-tuning](/docs/guides/model-optimization)
    ///
    /// REST: `GET /fine_tuning/jobs/{fine_tuning_job_id}`.
    /// Path constant: [`OpenAiApiPath::FINE_TUNING_BY_JOBS_BY_FINE_TUNING_JOB_ID`](crate::paths::OpenAiApiPath::FINE_TUNING_BY_JOBS_BY_FINE_TUNING_JOB_ID).
    async fn retrieve_fine_tuning_job(
        &self,
        fine_tuning_job_id: String,
    ) -> Result<FineTuningJob, Self::Error>;

    /// Immediately cancel a fine-tune job.
    ///
    /// REST: `POST /fine_tuning/jobs/{fine_tuning_job_id}/cancel`.
    /// Path constant: [`OpenAiApiPath::FINE_TUNING_BY_JOBS_BY_FINE_TUNING_JOB_ID_BY_CANCEL`](crate::paths::OpenAiApiPath::FINE_TUNING_BY_JOBS_BY_FINE_TUNING_JOB_ID_BY_CANCEL).
    async fn cancel_fine_tuning_job(
        &self,
        fine_tuning_job_id: String,
    ) -> Result<FineTuningJob, Self::Error>;

    /// List checkpoints for a fine-tuning job.
    ///
    /// REST: `GET /fine_tuning/jobs/{fine_tuning_job_id}/checkpoints`.
    /// Path constant: [`OpenAiApiPath::FINE_TUNING_BY_JOBS_BY_FINE_TUNING_JOB_ID_BY_CHECKPOINTS`](crate::paths::OpenAiApiPath::FINE_TUNING_BY_JOBS_BY_FINE_TUNING_JOB_ID_BY_CHECKPOINTS).
    async fn list_fine_tuning_job_checkpoints(
        &self,
        fine_tuning_job_id: String,
        after: Option<String>,
        limit: Option<i32>,
    ) -> Result<ListFineTuningJobCheckpointsResponse, Self::Error>;

    /// Get status updates for a fine-tuning job.
    ///
    /// REST: `GET /fine_tuning/jobs/{fine_tuning_job_id}/events`.
    /// Path constant: [`OpenAiApiPath::FINE_TUNING_BY_JOBS_BY_FINE_TUNING_JOB_ID_BY_EVENTS`](crate::paths::OpenAiApiPath::FINE_TUNING_BY_JOBS_BY_FINE_TUNING_JOB_ID_BY_EVENTS).
    async fn list_fine_tuning_events(
        &self,
        fine_tuning_job_id: String,
        after: Option<String>,
        limit: Option<i32>,
    ) -> Result<ListFineTuningJobEventsResponse, Self::Error>;

    /// Pause a fine-tune job.
    ///
    /// REST: `POST /fine_tuning/jobs/{fine_tuning_job_id}/pause`.
    /// Path constant: [`OpenAiApiPath::FINE_TUNING_BY_JOBS_BY_FINE_TUNING_JOB_ID_BY_PAUSE`](crate::paths::OpenAiApiPath::FINE_TUNING_BY_JOBS_BY_FINE_TUNING_JOB_ID_BY_PAUSE).
    async fn pause_fine_tuning_job(
        &self,
        fine_tuning_job_id: String,
    ) -> Result<FineTuningJob, Self::Error>;

    /// Resume a fine-tune job.
    ///
    /// REST: `POST /fine_tuning/jobs/{fine_tuning_job_id}/resume`.
    /// Path constant: [`OpenAiApiPath::FINE_TUNING_BY_JOBS_BY_FINE_TUNING_JOB_ID_BY_RESUME`](crate::paths::OpenAiApiPath::FINE_TUNING_BY_JOBS_BY_FINE_TUNING_JOB_ID_BY_RESUME).
    async fn resume_fine_tuning_job(
        &self,
        fine_tuning_job_id: String,
    ) -> Result<FineTuningJob, Self::Error>;
}
