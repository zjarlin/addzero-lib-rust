// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `FineTuningJobCheckpointMetrics` DTO.

use serde::{Deserialize, Serialize};

/// Metrics at the step number during the fine-tuning job.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FineTuningJobCheckpointMetrics {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub step: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub train_loss: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub train_mean_token_accuracy: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub valid_loss: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub valid_mean_token_accuracy: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub full_valid_loss: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub full_valid_mean_token_accuracy: Option<f64>,
}
