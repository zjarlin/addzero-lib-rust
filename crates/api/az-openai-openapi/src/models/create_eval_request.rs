// Generated from OpenAPI spec. Do not edit by hand.
//! `CreateEvalRequest` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    CreateEvalRequestDataSourceConfig,
    CreateEvalRequestTestingCriteriaItem,
    Metadata,
};

/// CreateEvalRequest
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateEvalRequest {
    /// The name of the evaluation.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub metadata: Option<Metadata>,
    /// The configuration for the data source used for the evaluation runs. Dictates the schema of the data
    /// used in the evaluation.
    pub data_source_config: CreateEvalRequestDataSourceConfig,
    /// A list of graders for all eval runs in this group. Graders can reference variables in the data
    /// source using double curly braces notation, like `{{item.variable_name}}`. To reference the model's
    /// output, use the `sample` namespace (ie, `{{sample.output_text}}`).
    pub testing_criteria: Vec<CreateEvalRequestTestingCriteriaItem>,
}
