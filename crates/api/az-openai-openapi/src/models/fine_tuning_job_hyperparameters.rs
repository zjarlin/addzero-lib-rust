// Generated from OpenAPI spec. Do not edit by hand.
//! `FineTuningJobHyperparameters` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    FineTuningJobHyperparametersBatchSize,
    FineTuningJobHyperparametersLearningRateMultiplier,
    FineTuningJobHyperparametersNEpochs,
};

/// The hyperparameters used for the fine-tuning job. This value will only be returned when running
/// `supervised` jobs.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FineTuningJobHyperparameters {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub batch_size: Option<FineTuningJobHyperparametersBatchSize>,
    /// Scaling factor for the learning rate. A smaller learning rate may be useful to avoid overfitting.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub learning_rate_multiplier: Option<FineTuningJobHyperparametersLearningRateMultiplier>,
    /// The number of epochs to train the model for. An epoch refers to one full cycle through the training
    /// dataset.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub n_epochs: Option<FineTuningJobHyperparametersNEpochs>,
}
