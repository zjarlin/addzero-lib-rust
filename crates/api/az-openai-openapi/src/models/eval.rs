// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `Eval` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    EvalDataSourceConfig,
    EvalTestingCriteriaItem,
    Metadata,
};

/// An Eval object with a data source config and testing criteria. An Eval represents a task to be done
/// for your LLM integration. Like: - Improve the quality of my chatbot - See how well my chatbot
/// handles customer support - Check if o4-mini is better at my usecase than gpt-4o
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Eval {
    /// The object type.
    pub object: String,
    /// Unique identifier for the evaluation.
    pub id: String,
    /// The name of the evaluation.
    pub name: String,
    /// Configuration of data sources used in runs of the evaluation.
    pub data_source_config: EvalDataSourceConfig,
    /// A list of testing criteria.
    pub testing_criteria: Vec<EvalTestingCriteriaItem>,
    /// The Unix timestamp (in seconds) for when the eval was created.
    pub created_at: i64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub metadata: Option<Metadata>,
}
