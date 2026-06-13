// Generated from OpenAPI spec. Do not edit by hand.
//! `EvalItemContentItem` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    EvalItemContentOutputText,
    EvalItemContentText,
    EvalItemInputImage,
    InputAudio,
    InputTextContent,
};

/// A single content item: input text, output text, input image, or input audio.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum EvalItemContentItem {
    EvalItemContentText(EvalItemContentText),
    InputTextContent(InputTextContent),
    EvalItemContentOutputText(EvalItemContentOutputText),
    EvalItemInputImage(EvalItemInputImage),
    InputAudio(InputAudio),
}
