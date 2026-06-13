// Generated from OpenAPI spec. Do not edit by hand.
//! `CreateEvalResponsesRunDataSourceInputMessages2` DTO.

use serde::{Deserialize, Serialize};

/// InputMessagesItemReference
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateEvalResponsesRunDataSourceInputMessages2 {
    /// The type of input messages. Always `item_reference`.
    #[serde(rename = "type")]
    pub type_value: String,
    /// A reference to a variable in the `item` namespace. Ie, "item.name"
    pub item_reference: String,
}
