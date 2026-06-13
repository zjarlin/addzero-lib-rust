// Generated from OpenAPI spec. Do not edit by hand.
//! `CreateModerationResponseResultCategories` DTO.

use serde::{Deserialize, Serialize};

/// A list of the categories, and whether they are flagged or not.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateModerationResponseResultCategories {
    /// Content that expresses, incites, or promotes hate based on race, gender, ethnicity, religion,
    /// nationality, sexual orientation, disability status, or caste. Hateful content aimed at non-protected
    /// groups (e.g., chess players) is harassment.
    pub hate: bool,
    /// Hateful content that also includes violence or serious harm towards the targeted group based on
    /// race, gender, ethnicity, religion, nationality, sexual orientation, disability status, or caste.
    #[serde(rename = "hate/threatening")]
    pub hate_threatening: bool,
    /// Content that expresses, incites, or promotes harassing language towards any target.
    pub harassment: bool,
    /// Harassment content that also includes violence or serious harm towards any target.
    #[serde(rename = "harassment/threatening")]
    pub harassment_threatening: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub illicit: Option<bool>,
    #[serde(rename = "illicit/violent", default, skip_serializing_if = "Option::is_none")]
    pub illicit_violent: Option<bool>,
    /// Content that promotes, encourages, or depicts acts of self-harm, such as suicide, cutting, and
    /// eating disorders.
    #[serde(rename = "self-harm")]
    pub self_harm: bool,
    /// Content where the speaker expresses that they are engaging or intend to engage in acts of self-harm,
    /// such as suicide, cutting, and eating disorders.
    #[serde(rename = "self-harm/intent")]
    pub self_harm_intent: bool,
    /// Content that encourages performing acts of self-harm, such as suicide, cutting, and eating
    /// disorders, or that gives instructions or advice on how to commit such acts.
    #[serde(rename = "self-harm/instructions")]
    pub self_harm_instructions: bool,
    /// Content meant to arouse sexual excitement, such as the description of sexual activity, or that
    /// promotes sexual services (excluding sex education and wellness).
    pub sexual: bool,
    /// Sexual content that includes an individual who is under 18 years old.
    #[serde(rename = "sexual/minors")]
    pub sexual_minors: bool,
    /// Content that depicts death, violence, or physical injury.
    pub violence: bool,
    /// Content that depicts death, violence, or physical injury in graphic detail.
    #[serde(rename = "violence/graphic")]
    pub violence_graphic: bool,
}
