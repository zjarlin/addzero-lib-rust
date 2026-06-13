// Generated from OpenAPI spec. Do not edit by hand.
//! `FineTuneReinforcementHyperparameters` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    FineTuneReinforcementHyperparametersBatchSize,
    FineTuneReinforcementHyperparametersComputeMultiplier,
    FineTuneReinforcementHyperparametersEvalInterval,
    FineTuneReinforcementHyperparametersEvalSamples,
    FineTuneReinforcementHyperparametersLearningRateMultiplier,
    FineTuneReinforcementHyperparametersNEpochs,
};

/// The hyperparameters used for the reinforcement fine-tuning job.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FineTuneReinforcementHyperparameters {
    /// Number of examples in each batch. A larger batch size means that model parameters are updated less
    /// frequently, but with lower variance.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub batch_size: Option<FineTuneReinforcementHyperparametersBatchSize>,
    /// Scaling factor for the learning rate. A smaller learning rate may be useful to avoid overfitting.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub learning_rate_multiplier: Option<FineTuneReinforcementHyperparametersLearningRateMultiplier>,
    /// The number of epochs to train the model for. An epoch refers to one full cycle through the training
    /// dataset.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub n_epochs: Option<FineTuneReinforcementHyperparametersNEpochs>,
    /// Level of reasoning effort.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reasoning_effort: Option<String>,
    /// Multiplier on amount of compute used for exploring search space during training.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub compute_multiplier: Option<FineTuneReinforcementHyperparametersComputeMultiplier>,
    /// The number of training steps between evaluation runs.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub eval_interval: Option<FineTuneReinforcementHyperparametersEvalInterval>,
    /// Number of evaluation samples to generate per training step.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub eval_samples: Option<FineTuneReinforcementHyperparametersEvalSamples>,
}
