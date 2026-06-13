// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `FineTuningJob` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    FineTuneMethod,
    FineTuningIntegration,
    FineTuningJobError,
    FineTuningJobHyperparameters,
    Metadata,
};

/// The `fine_tuning.job` object represents a fine-tuning job that has been created through the API.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FineTuningJob {
    /// The object identifier, which can be referenced in the API endpoints.
    pub id: String,
    /// The Unix timestamp (in seconds) for when the fine-tuning job was created.
    pub created_at: i64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<FineTuningJobError>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fine_tuned_model: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub finished_at: Option<i64>,
    /// The hyperparameters used for the fine-tuning job. This value will only be returned when running
    /// `supervised` jobs.
    pub hyperparameters: FineTuningJobHyperparameters,
    /// The base model that is being fine-tuned.
    pub model: String,
    /// The object type, which is always "fine_tuning.job".
    pub object: String,
    /// The organization that owns the fine-tuning job.
    pub organization_id: String,
    /// The compiled results file ID(s) for the fine-tuning job. You can retrieve the results with the
    /// [Files API](/docs/api-reference/files/retrieve-contents).
    pub result_files: Vec<String>,
    /// The current status of the fine-tuning job, which can be either `validating_files`, `queued`,
    /// `running`, `succeeded`, `failed`, or `cancelled`.
    pub status: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub trained_tokens: Option<i32>,
    /// The file ID used for training. You can retrieve the training data with the [Files API](/docs/api-
    /// reference/files/retrieve-contents).
    pub training_file: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub validation_file: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub integrations: Option<Vec<FineTuningIntegration>>,
    /// The seed used for the fine-tuning job.
    pub seed: i32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub estimated_finish: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub method: Option<FineTuneMethod>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub metadata: Option<Metadata>,
}
