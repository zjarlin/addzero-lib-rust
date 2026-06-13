// Generated from OpenAPI spec. Do not edit by hand.
//! `RunGraderRequest` DTO.

use serde::{Deserialize, Serialize};

use crate::bodies::{
    OpenAiJsonObject,
};

use crate::models::{
    RunGraderRequestGrader,
};

/// RunGraderRequest
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunGraderRequest {
    /// The grader used for the fine-tuning job.
    pub grader: RunGraderRequestGrader,
    /// The dataset item provided to the grader. This will be used to populate the `item` namespace. See
    /// [the guide](/docs/guides/graders) for more details.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub item: Option<OpenAiJsonObject>,
    /// The model sample to be evaluated. This value will be used to populate the `sample` namespace. See
    /// [the guide](/docs/guides/graders) for more details. The `output_json` variable will be populated if
    /// the model sample is a valid JSON string.
    pub model_sample: String,
}
