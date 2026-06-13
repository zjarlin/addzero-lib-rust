// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `CustomGrammarFormatParam` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    GrammarSyntax1,
};

/// A grammar defined by the user.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomGrammarFormatParam {
    /// Grammar format. Always `grammar`.
    #[serde(rename = "type")]
    pub type_value: String,
    /// The syntax of the grammar definition. One of `lark` or `regex`.
    pub syntax: GrammarSyntax1,
    /// The grammar definition.
    pub definition: String,
}
