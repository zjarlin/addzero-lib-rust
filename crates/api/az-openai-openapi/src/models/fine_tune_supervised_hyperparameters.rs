// Generated from OpenAPI spec. Do not edit by hand.
//! `FineTuneSupervisedHyperparameters` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    FineTuneSupervisedHyperparametersBatchSize,
    FineTuneSupervisedHyperparametersLearningRateMultiplier,
    FineTuneSupervisedHyperparametersNEpochs,
};

/// The hyperparameters used for the fine-tuning job.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FineTuneSupervisedHyperparameters {
    /// Number of examples in each batch. A larger batch size means that model parameters are updated less
    /// frequently, but with lower variance.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub batch_size: Option<FineTuneSupervisedHyperparametersBatchSize>,
    /// Scaling factor for the learning rate. A smaller learning rate may be useful to avoid overfitting.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub learning_rate_multiplier: Option<FineTuneSupervisedHyperparametersLearningRateMultiplier>,
    /// The number of epochs to train the model for. An epoch refers to one full cycle through the training
    /// dataset.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub n_epochs: Option<FineTuneSupervisedHyperparametersNEpochs>,
}
