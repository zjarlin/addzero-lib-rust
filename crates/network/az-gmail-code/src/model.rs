use az_derive_aliases::{apply, plain_eq, serde_eq};
use serde_json::Value;
use std::collections::BTreeMap;

/// Verification code extracted from a Gmail message.
#[apply(plain_eq)]
pub struct ExtractedGmailCode {
    /// Numeric verification code.
    pub code: String,
    /// Gmail message id containing the code.
    pub message_id: String,
    /// Optional thread id returned by Gmail.
    pub thread_id: Option<String>,
    /// Best-effort sender header.
    pub from: Option<String>,
    /// Best-effort subject header.
    pub subject: Option<String>,
    /// MIME type of the body candidate that produced the code.
    pub source_mime_type: String,
}

#[apply(serde_eq)]
pub(crate) struct GmailListMessagesResponse {
    #[serde(default)]
    pub(crate) messages: Vec<GmailMessageSummary>,
    #[serde(default, rename = "nextPageToken")]
    pub(crate) next_page_token: Option<String>,
    #[serde(default, rename = "resultSizeEstimate")]
    pub(crate) result_size_estimate: Option<u32>,
}

#[apply(serde_eq)]
pub(crate) struct GmailMessageSummary {
    pub(crate) id: String,
    #[serde(default, rename = "threadId")]
    pub(crate) thread_id: Option<String>,
}

/// Gmail message shape needed for verification-code extraction.
#[apply(serde_eq)]
pub struct GmailMessage {
    /// Gmail message id.
    pub id: String,
    /// Optional Gmail thread id.
    #[serde(default, rename = "threadId")]
    pub thread_id: Option<String>,
    /// MIME tree root.
    #[serde(default)]
    pub payload: Option<GmailMessagePart>,
    /// Short text snippet returned by Gmail.
    #[serde(default)]
    pub snippet: Option<String>,
    /// Extra Gmail fields not modeled by this crate.
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

impl GmailMessage {
    /// Returns a header value from the root payload, case-insensitively.
    #[must_use]
    pub fn header(&self, name: &str) -> Option<&str> {
        self.payload.as_ref()?.header(name)
    }
}

/// One part of a Gmail MIME message tree.
#[apply(serde_eq)]
pub struct GmailMessagePart {
    /// Gmail part id.
    #[serde(default, rename = "partId")]
    pub part_id: Option<String>,
    /// MIME type, for example `text/plain` or `text/html`.
    #[serde(default, rename = "mimeType")]
    pub mime_type: String,
    /// Message headers for this part.
    #[serde(default)]
    pub headers: Vec<GmailMessageHeader>,
    /// Body metadata and inline payload.
    #[serde(default)]
    pub body: Option<GmailMessagePartBody>,
    /// Child MIME parts.
    #[serde(default)]
    pub parts: Vec<GmailMessagePart>,
}

impl GmailMessagePart {
    /// Returns a header value from this part, case-insensitively.
    #[must_use]
    pub fn header(&self, name: &str) -> Option<&str> {
        self.headers
            .iter()
            .find(|header| header.name.eq_ignore_ascii_case(name))
            .map(|header| header.value.as_str())
    }
}

/// Gmail MIME header.
#[apply(serde_eq)]
pub struct GmailMessageHeader {
    /// Header name.
    pub name: String,
    /// Header value.
    pub value: String,
}

/// Gmail message body metadata and inline payload.
#[apply(serde_eq)]
pub struct GmailMessagePartBody {
    #[serde(default)]
    pub data: Option<String>,
    #[serde(default)]
    pub size: Option<u64>,
    #[serde(default, rename = "attachmentId")]
    pub attachment_id: Option<String>,
}
