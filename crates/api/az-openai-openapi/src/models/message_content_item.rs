// Generated from OpenAPI spec. Do not edit by hand.
//! `MessageContentItem` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    ComputerScreenshotContent,
    InputFileContent,
    InputImageContent,
    InputTextContent,
    OutputTextContent,
    ReasoningTextContent,
    RefusalContent,
    SummaryTextContent,
    TextContent,
};

/// A content part that makes up an input or output item.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum MessageContentItem {
    InputTextContent(InputTextContent),
    OutputTextContent(OutputTextContent),
    TextContent(TextContent),
    SummaryTextContent(SummaryTextContent),
    ReasoningTextContent(ReasoningTextContent),
    RefusalContent(RefusalContent),
    InputImageContent(InputImageContent),
    ComputerScreenshotContent(ComputerScreenshotContent),
    InputFileContent(InputFileContent),
}
