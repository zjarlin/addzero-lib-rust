// Generated from OpenAPI spec. Do not edit by hand.
//! `CreateEvalCompletionsRunDataSourceInputMessages3ItemReferenceInputMessages` DTO.

use serde::{Deserialize, Serialize};

/// ItemReferenceInputMessages
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateEvalCompletionsRunDataSourceInputMessages3ItemReferenceInputMessages {
    /// The type of input messages. Always `item_reference`.
    #[serde(rename = "type")]
    pub type_value: String,
    /// A reference to a variable in the `item` namespace. Ie, "item.input_trajectory"
    pub item_reference: String,
}
