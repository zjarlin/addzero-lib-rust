// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `CoordParam` DTO.

use serde::{Deserialize, Serialize};

/// An x/y coordinate pair, e.g. `{ x: 100, y: 200 }`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoordParam {
    /// The x-coordinate.
    pub x: i32,
    /// The y-coordinate.
    pub y: i32,
}
