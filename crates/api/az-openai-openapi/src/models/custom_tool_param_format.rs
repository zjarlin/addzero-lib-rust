// Generated from OpenAPI spec. Do not edit by hand.
//! `CustomToolParamFormat` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    CustomGrammarFormatParam,
    CustomTextFormatParam,
};

/// The input format for the custom tool. Default is unconstrained text.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum CustomToolParamFormat {
    CustomTextFormatParam(CustomTextFormatParam),
    CustomGrammarFormatParam(CustomGrammarFormatParam),
}
