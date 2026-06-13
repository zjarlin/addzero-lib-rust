// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `DragParam` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    CoordParam,
};

/// A drag action.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DragParam {
    /// Specifies the event type. For a drag action, this property is always set to `drag`.
    #[serde(rename = "type")]
    pub type_value: String,
    /// An array of coordinates representing the path of the drag action. Coordinates will appear as an
    /// array of objects, eg ``` [ { x: 100, y: 200 }, { x: 200, y: 300 } ] ```
    pub path: Vec<CoordParam>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub keys: Option<Vec<String>>,
}
