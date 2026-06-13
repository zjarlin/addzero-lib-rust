// Generated from OpenAPI spec. Do not edit by hand.
//! `CreateFineTuningJobRequest` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    CreateFineTuningJobRequestHyperparameters,
    CreateFineTuningJobRequestIntegration,
    FineTuneMethod,
    Metadata,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateFineTuningJobRequest {
    /// The name of the model to fine-tune. You can select one of the [supported models](/docs/guides/fine-
    /// tuning#which-models-can-be-fine-tuned).
    pub model: String,
    /// The ID of an uploaded file that contains training data. See [upload file](/docs/api-
    /// reference/files/create) for how to upload a file. Your dataset must be formatted as a JSONL file.
    /// Additionally, you must upload your file with the purpose `fine-tune`. The contents of the file
    /// should differ depending on if the model uses the [chat](/docs/api-reference/fine-tuning/chat-input),
    /// [completions](/docs/api-reference/fine-tuning/completions-input) format, or if the fine-tuning
    /// method uses the [preference](/docs/api-reference/fine-tuning/preference-input) format. See the
    /// [fine-tuning guide](/docs/guides/model-optimization) for more details.
    pub training_file: String,
    /// The hyperparameters used for the fine-tuning job. This value is now deprecated in favor of `method`,
    /// and should be passed in under the `method` parameter.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hyperparameters: Option<CreateFineTuningJobRequestHyperparameters>,
    /// A string of up to 64 characters that will be added to your fine-tuned model name. For example, a
    /// `suffix` of "custom-model-name" would produce a model name like `ft:gpt-4o-mini:openai:custom-model-
    /// name:7p4lURel`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub suffix: Option<String>,
    /// The ID of an uploaded file that contains validation data. If you provide this file, the data is used
    /// to generate validation metrics periodically during fine-tuning. These metrics can be viewed in the
    /// fine-tuning results file. The same data should not be present in both train and validation files.
    /// Your dataset must be formatted as a JSONL file. You must upload your file with the purpose `fine-
    /// tune`. See the [fine-tuning guide](/docs/guides/model-optimization) for more details.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub validation_file: Option<String>,
    /// A list of integrations to enable for your fine-tuning job.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub integrations: Option<Vec<CreateFineTuningJobRequestIntegration>>,
    /// The seed controls the reproducibility of the job. Passing in the same seed and job parameters should
    /// produce the same results, but may differ in rare cases. If a seed is not specified, one will be
    /// generated for you.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub seed: Option<i32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub method: Option<FineTuneMethod>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub metadata: Option<Metadata>,
}
