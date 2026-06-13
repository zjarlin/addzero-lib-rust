// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `CreateModerationResponseResultCategoryScores` DTO.

use serde::{Deserialize, Serialize};

/// A list of the categories along with their scores as predicted by model.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateModerationResponseResultCategoryScores {
    /// The score for the category 'hate'.
    pub hate: f64,
    /// The score for the category 'hate/threatening'.
    #[serde(rename = "hate/threatening")]
    pub hate_threatening: f64,
    /// The score for the category 'harassment'.
    pub harassment: f64,
    /// The score for the category 'harassment/threatening'.
    #[serde(rename = "harassment/threatening")]
    pub harassment_threatening: f64,
    /// The score for the category 'illicit'.
    pub illicit: f64,
    /// The score for the category 'illicit/violent'.
    #[serde(rename = "illicit/violent")]
    pub illicit_violent: f64,
    /// The score for the category 'self-harm'.
    #[serde(rename = "self-harm")]
    pub self_harm: f64,
    /// The score for the category 'self-harm/intent'.
    #[serde(rename = "self-harm/intent")]
    pub self_harm_intent: f64,
    /// The score for the category 'self-harm/instructions'.
    #[serde(rename = "self-harm/instructions")]
    pub self_harm_instructions: f64,
    /// The score for the category 'sexual'.
    pub sexual: f64,
    /// The score for the category 'sexual/minors'.
    #[serde(rename = "sexual/minors")]
    pub sexual_minors: f64,
    /// The score for the category 'violence'.
    pub violence: f64,
    /// The score for the category 'violence/graphic'.
    #[serde(rename = "violence/graphic")]
    pub violence_graphic: f64,
}
