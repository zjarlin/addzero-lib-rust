// Generated from OpenAPI spec. Do not edit by hand.
//! `CreateModerationResponseResultCategoryAppliedInputTypes` DTO.

use serde::{Deserialize, Serialize};

/// A list of the categories along with the input type(s) that the score applies to.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateModerationResponseResultCategoryAppliedInputTypes {
    /// The applied input type(s) for the category 'hate'.
    pub hate: Vec<String>,
    /// The applied input type(s) for the category 'hate/threatening'.
    #[serde(rename = "hate/threatening")]
    pub hate_threatening: Vec<String>,
    /// The applied input type(s) for the category 'harassment'.
    pub harassment: Vec<String>,
    /// The applied input type(s) for the category 'harassment/threatening'.
    #[serde(rename = "harassment/threatening")]
    pub harassment_threatening: Vec<String>,
    /// The applied input type(s) for the category 'illicit'.
    pub illicit: Vec<String>,
    /// The applied input type(s) for the category 'illicit/violent'.
    #[serde(rename = "illicit/violent")]
    pub illicit_violent: Vec<String>,
    /// The applied input type(s) for the category 'self-harm'.
    #[serde(rename = "self-harm")]
    pub self_harm: Vec<String>,
    /// The applied input type(s) for the category 'self-harm/intent'.
    #[serde(rename = "self-harm/intent")]
    pub self_harm_intent: Vec<String>,
    /// The applied input type(s) for the category 'self-harm/instructions'.
    #[serde(rename = "self-harm/instructions")]
    pub self_harm_instructions: Vec<String>,
    /// The applied input type(s) for the category 'sexual'.
    pub sexual: Vec<String>,
    /// The applied input type(s) for the category 'sexual/minors'.
    #[serde(rename = "sexual/minors")]
    pub sexual_minors: Vec<String>,
    /// The applied input type(s) for the category 'violence'.
    pub violence: Vec<String>,
    /// The applied input type(s) for the category 'violence/graphic'.
    #[serde(rename = "violence/graphic")]
    pub violence_graphic: Vec<String>,
}
