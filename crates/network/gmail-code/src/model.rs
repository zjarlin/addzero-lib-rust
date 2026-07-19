use serde_json::Value;
use std::collections::BTreeMap;

/// 从 Gmail 邮件中提取出的验证码。
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExtractedGmailCode {
    /// 数字验证码。
    pub code: String,
    /// 包含该验证码的 Gmail 邮件 ID。
    pub message_id: String,
    /// Gmail 返回的可选会话 ID。
    pub thread_id: Option<String>,
    /// 尽力读取到的发件人 header。
    pub from: Option<String>,
    /// 尽力读取到的主题 header。
    pub subject: Option<String>,
    /// 命中该验证码的正文候选 MIME 类型。
    pub source_mime_type: String,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) struct GmailListMessagesResponse {
    #[serde(default)]
    pub(crate) messages: Vec<GmailMessageSummary>,
    #[serde(default, rename = "nextPageToken")]
    pub(crate) next_page_token: Option<String>,
    #[serde(default, rename = "resultSizeEstimate")]
    pub(crate) result_size_estimate: Option<u32>,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) struct GmailMessageSummary {
    pub(crate) id: String,
    #[serde(default, rename = "threadId")]
    pub(crate) thread_id: Option<String>,
}

/// 验证码提取所需的 Gmail 邮件结构。
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct GmailMessage {
    /// Gmail 邮件 ID。
    pub id: String,
    /// 可选的 Gmail 会话 ID。
    #[serde(default, rename = "threadId")]
    pub thread_id: Option<String>,
    /// MIME 树根节点。
    #[serde(default)]
    pub payload: Option<GmailMessagePart>,
    /// Gmail 返回的短文本摘要。
    #[serde(default)]
    pub snippet: Option<String>,
    /// 本 crate 未建模的额外 Gmail 字段。
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

impl GmailMessage {
    /// 从根 payload 中按不区分大小写的名称读取 header 值。
    #[must_use]
    pub fn header(&self, name: &str) -> Option<&str> {
        self.payload.as_ref()?.header(name)
    }
}

/// Gmail MIME 邮件树中的单个 part。
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct GmailMessagePart {
    /// Gmail part ID。
    #[serde(default, rename = "partId")]
    pub part_id: Option<String>,
    /// MIME 类型，例如 `text/plain` 或 `text/html`。
    #[serde(default, rename = "mimeType")]
    pub mime_type: String,
    /// 该 part 的邮件 header 列表。
    #[serde(default)]
    pub headers: Vec<GmailMessageHeader>,
    /// 正文元数据和内联载荷。
    #[serde(default)]
    pub body: Option<GmailMessagePartBody>,
    /// 子 MIME parts。
    #[serde(default)]
    pub parts: Vec<GmailMessagePart>,
}

impl GmailMessagePart {
    /// 从该 part 中按不区分大小写的名称读取 header 值。
    #[must_use]
    pub fn header(&self, name: &str) -> Option<&str> {
        self.headers
            .iter()
            .find(|header| header.name.eq_ignore_ascii_case(name))
            .map(|header| header.value.as_str())
    }
}

/// Gmail MIME header。
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct GmailMessageHeader {
    /// Header 名称。
    pub name: String,
    /// Header 值。
    pub value: String,
}

/// Gmail 邮件正文元数据和内联载荷。
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct GmailMessagePartBody {
    /// Gmail API 返回的 base64url 编码正文；附件正文可能为空。
    #[serde(default)]
    pub data: Option<String>,
    /// Gmail API 报告的正文大小。
    #[serde(default)]
    pub size: Option<u64>,
    /// 外部附件正文 ID；本 crate 当前不自动拉取附件内容。
    #[serde(default, rename = "attachmentId")]
    pub attachment_id: Option<String>,
}
