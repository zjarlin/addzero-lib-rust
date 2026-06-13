// Generated from OpenAPI spec. Do not edit by hand.
//! `MCPTool` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    MCPToolAllowedTools,
    MCPToolRequireApproval2,
};

/// Give the model access to additional tools via remote Model Context Protocol (MCP) servers. [Learn
/// more about MCP](/docs/guides/tools-remote-mcp).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MCPTool {
    /// The type of the MCP tool. Always `mcp`.
    #[serde(rename = "type")]
    pub type_value: String,
    /// A label for this MCP server, used to identify it in tool calls.
    pub server_label: String,
    /// The URL for the MCP server. One of `server_url` or `connector_id` must be provided.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub server_url: Option<String>,
    /// Identifier for service connectors, like those available in ChatGPT. One of `server_url` or
    /// `connector_id` must be provided. Learn more about service connectors [here](/docs/guides/tools-
    /// remote-mcp#connectors). Currently supported `connector_id` values are: - Dropbox:
    /// `connector_dropbox` - Gmail: `connector_gmail` - Google Calendar: `connector_googlecalendar` -
    /// Google Drive: `connector_googledrive` - Microsoft Teams: `connector_microsoftteams` - Outlook
    /// Calendar: `connector_outlookcalendar` - Outlook Email: `connector_outlookemail` - SharePoint:
    /// `connector_sharepoint`
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub connector_id: Option<String>,
    /// An OAuth access token that can be used with a remote MCP server, either with a custom MCP server URL
    /// or a service connector. Your application must handle the OAuth authorization flow and provide the
    /// token here.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub authorization: Option<String>,
    /// Optional description of the MCP server, used to provide more context.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub server_description: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub headers: Option<std::collections::BTreeMap<String, String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub allowed_tools: Option<MCPToolAllowedTools>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub require_approval: Option<MCPToolRequireApproval2>,
    /// Whether this MCP tool is deferred and discovered via tool search.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub defer_loading: Option<bool>,
}
