// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `FineTuneDPOHyperparameters` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    FineTuneDPOHyperparametersBatchSize,
    FineTuneDPOHyperparametersBeta,
    FineTuneDPOHyperparametersLearningRateMultiplier,
    FineTuneDPOHyperparametersNEpochs,
};

/// The hyperparameters used for the DPO fine-tuning job.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FineTuneDPOHyperparameters {
    /// The beta value for the DPO method. A higher beta value will increase the weight of the penalty
    /// between the policy and reference model.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub beta: Option<FineTuneDPOHyperparametersBeta>,
    /// Number of examples in each batch. A larger batch size means that model parameters are updated less
    /// frequently, but with lower variance.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub batch_size: Option<FineTuneDPOHyperparametersBatchSize>,
    /// Scaling factor for the learning rate. A smaller learning rate may be useful to avoid overfitting.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub learning_rate_multiplier: Option<FineTuneDPOHyperparametersLearningRateMultiplier>,
    /// The number of epochs to train the model for. An epoch refers to one full cycle through the training
    /// dataset.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub n_epochs: Option<FineTuneDPOHyperparametersNEpochs>,
}
