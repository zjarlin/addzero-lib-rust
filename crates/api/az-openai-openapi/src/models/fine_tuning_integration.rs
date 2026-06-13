// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `FineTuningIntegration` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    FineTuningIntegrationWandb,
};

/// Fine-Tuning Job Integration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FineTuningIntegration {
    /// The type of the integration being enabled for the fine-tuning job
    #[serde(rename = "type")]
    pub type_value: String,
    /// The settings for your integration with Weights and Biases. This payload specifies the project that
    /// metrics will be sent to. Optionally, you can set an explicit display name for your run, add tags to
    /// your run, and set a default entity (team, username, etc) to be associated with your run.
    pub wandb: FineTuningIntegrationWandb,
}
