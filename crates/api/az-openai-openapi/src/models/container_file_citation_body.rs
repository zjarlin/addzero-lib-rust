// Generated from OpenAPI spec. Do not edit by hand.
//! `ContainerFileCitationBody` DTO.

use serde::{Deserialize, Serialize};

/// A citation for a container file used to generate a model response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContainerFileCitationBody {
    /// The type of the container file citation. Always `container_file_citation`.
    #[serde(rename = "type")]
    pub type_value: String,
    /// The ID of the container file.
    pub container_id: String,
    /// The ID of the file.
    pub file_id: String,
    /// The index of the first character of the container file citation in the message.
    pub start_index: i32,
    /// The index of the last character of the container file citation in the message.
    pub end_index: i32,
    /// The filename of the container file cited.
    pub filename: String,
}
