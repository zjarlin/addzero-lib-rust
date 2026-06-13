// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `CreateFineTuningJobRequestHyperparameters` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    CreateFineTuningJobRequestHyperparametersBatchSize,
    CreateFineTuningJobRequestHyperparametersLearningRateMultiplier,
    CreateFineTuningJobRequestHyperparametersNEpochs,
};

/// The hyperparameters used for the fine-tuning job. This value is now deprecated in favor of `method`,
/// and should be passed in under the `method` parameter.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateFineTuningJobRequestHyperparameters {
    /// Number of examples in each batch. A larger batch size means that model parameters are updated less
    /// frequently, but with lower variance.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub batch_size: Option<CreateFineTuningJobRequestHyperparametersBatchSize>,
    /// Scaling factor for the learning rate. A smaller learning rate may be useful to avoid overfitting.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub learning_rate_multiplier: Option<CreateFineTuningJobRequestHyperparametersLearningRateMultiplier>,
    /// The number of epochs to train the model for. An epoch refers to one full cycle through the training
    /// dataset.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub n_epochs: Option<CreateFineTuningJobRequestHyperparametersNEpochs>,
}
