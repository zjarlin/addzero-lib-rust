// Generated from OpenAPI spec. Do not edit by hand.
//! `EvalRun` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    EvalApiError,
    EvalRunDataSource,
    EvalRunPerModelUsageItem,
    EvalRunPerTestingCriteriaResult,
    EvalRunResultCounts,
    Metadata,
};

/// A schema representing an evaluation run.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvalRun {
    /// The type of the object. Always "eval.run".
    pub object: String,
    /// Unique identifier for the evaluation run.
    pub id: String,
    /// The identifier of the associated evaluation.
    pub eval_id: String,
    /// The status of the evaluation run.
    pub status: String,
    /// The model that is evaluated, if applicable.
    pub model: String,
    /// The name of the evaluation run.
    pub name: String,
    /// Unix timestamp (in seconds) when the evaluation run was created.
    pub created_at: i64,
    /// The URL to the rendered evaluation run report on the UI dashboard.
    pub report_url: String,
    /// Counters summarizing the outcomes of the evaluation run.
    pub result_counts: EvalRunResultCounts,
    /// Usage statistics for each model during the evaluation run.
    pub per_model_usage: Vec<EvalRunPerModelUsageItem>,
    /// Results per testing criteria applied during the evaluation run.
    pub per_testing_criteria_results: Vec<EvalRunPerTestingCriteriaResult>,
    /// Information about the run's data source.
    pub data_source: EvalRunDataSource,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub metadata: Option<Metadata>,
    pub error: EvalApiError,
}
