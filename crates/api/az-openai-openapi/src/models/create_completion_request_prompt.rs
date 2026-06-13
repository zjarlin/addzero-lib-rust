// Generated from OpenAPI spec. Do not edit by hand.
//! `CreateCompletionRequestPrompt` DTO.

use serde::{Deserialize, Serialize};

/// The prompt(s) to generate completions for, encoded as a string, array of strings, array of tokens,
/// or array of token arrays. Note that <|endoftext|> is the document separator that the model sees
/// during training, so if a prompt is not specified the model will generate as if from the beginning of
/// a new document.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum CreateCompletionRequestPrompt {
    String(String),
    Array(Vec<String>),
    Array3(Vec<i32>),
    Array4(Vec<Vec<i32>>),
}
