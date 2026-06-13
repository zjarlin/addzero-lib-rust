// Generated from OpenAPI spec. Do not edit by hand.
//! `FineTuningJobCheckpoint` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    FineTuningJobCheckpointMetrics,
};

/// The `fine_tuning.job.checkpoint` object represents a model checkpoint for a fine-tuning job that is
/// ready to use.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FineTuningJobCheckpoint {
    /// The checkpoint identifier, which can be referenced in the API endpoints.
    pub id: String,
    /// The Unix timestamp (in seconds) for when the checkpoint was created.
    pub created_at: i64,
    /// The name of the fine-tuned checkpoint model that is created.
    pub fine_tuned_model_checkpoint: String,
    /// The step number that the checkpoint was created at.
    pub step_number: i32,
    /// Metrics at the step number during the fine-tuning job.
    pub metrics: FineTuningJobCheckpointMetrics,
    /// The name of the fine-tuning job that this checkpoint was created from.
    pub fine_tuning_job_id: String,
    /// The object type, which is always "fine_tuning.job.checkpoint".
    pub object: String,
}
