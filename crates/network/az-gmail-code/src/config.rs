use az_derive_aliases::{apply, impl_default, plain_clone_debug, plain_eq};
use std::time::Duration;

const DEFAULT_GMAIL_API_BASE_URL: &str = "https://gmail.googleapis.com/gmail/v1/";
const DEFAULT_USER_ID: &str = "me";
const DEFAULT_USER_AGENT: &str = "az-gmail-code/2026.5";
const DEFAULT_CONNECT_TIMEOUT_SECS: u64 = 10;
const DEFAULT_REQUEST_TIMEOUT_SECS: u64 = 30;
const DEFAULT_MAX_RESULTS: u32 = 10;

/// 已授权 Gmail API 请求的客户端配置。
#[apply(plain_eq)]
pub struct GmailCodeConfig {
    /// 调用方所控制邮箱的 Gmail API OAuth 访问令牌。
    pub access_token: String,
    /// Gmail API 基础 URL；生产环境通常保持默认值。
    pub base_url: String,
    /// Gmail `userId` 路径片段；OAuth 调用方通常使用 `me`。
    pub user_id: String,
    /// 该客户端发送的 HTTP User-Agent。
    pub user_agent: Option<String>,
    /// TCP 连接超时。
    pub connect_timeout: Duration,
    /// 单次请求总超时。
    pub request_timeout: Duration,
}

impl GmailCodeConfig {
    /// 使用必填的 Gmail OAuth 访问令牌创建配置构建器。
    pub fn builder(access_token: impl Into<String>) -> GmailCodeConfigBuilder {
        GmailCodeConfigBuilder {
            config: Self {
                access_token: access_token.into(),
                base_url: DEFAULT_GMAIL_API_BASE_URL.to_owned(),
                user_id: DEFAULT_USER_ID.to_owned(),
                user_agent: Some(DEFAULT_USER_AGENT.to_owned()),
                connect_timeout: Duration::from_secs(DEFAULT_CONNECT_TIMEOUT_SECS),
                request_timeout: Duration::from_secs(DEFAULT_REQUEST_TIMEOUT_SECS),
            },
        }
    }

    pub(crate) fn validate(&self) -> anyhow::Result<()> {
        if self.access_token.trim().is_empty() {
            anyhow::bail!("invalid config: access_token cannot be blank");
        }
        if self.base_url.trim().is_empty() {
            anyhow::bail!("invalid config: base_url cannot be blank");
        }
        if self.user_id.trim().is_empty() {
            anyhow::bail!("invalid config: user_id cannot be blank");
        }
        Ok(())
    }
}

/// [`GmailCodeConfig`] 的链式构建器。
#[apply(plain_clone_debug)]
pub struct GmailCodeConfigBuilder {
    config: GmailCodeConfig,
}

impl GmailCodeConfigBuilder {
    /// 覆盖 Gmail API 基础 URL，主要用于测试。
    #[must_use]
    pub fn base_url(mut self, value: impl Into<String>) -> Self {
        self.config.base_url = value.into();
        self
    }

    /// 覆盖 Gmail `userId` 路径片段。
    #[must_use]
    pub fn user_id(mut self, value: impl Into<String>) -> Self {
        self.config.user_id = value.into();
        self
    }

    /// 覆盖或清除 HTTP User-Agent。
    #[must_use]
    pub fn user_agent(mut self, value: Option<impl Into<String>>) -> Self {
        self.config.user_agent = value.map(Into::into);
        self
    }

    /// 设置 TCP 连接超时。
    #[must_use]
    pub const fn connect_timeout(mut self, value: Duration) -> Self {
        self.config.connect_timeout = value;
        self
    }

    /// 设置单次请求总超时。
    #[must_use]
    pub const fn request_timeout(mut self, value: Duration) -> Self {
        self.config.request_timeout = value;
        self
    }

    /// 校验并返回最终配置。
    pub fn build(self) -> anyhow::Result<GmailCodeConfig> {
        self.config.validate()?;
        Ok(self.config)
    }
}

/// 用于查找 Gmail 验证码邮件的搜索参数。
#[apply(plain_eq)]
pub struct GmailCodeQuery {
    /// 附加到结构化过滤条件之后的原始 Gmail 搜索表达式。
    pub query: Option<String>,
    /// 可选的发件人过滤条件。
    pub from: Option<String>,
    /// 可选的主题过滤条件。
    pub subject: Option<String>,
    /// 可选的 Gmail `newer_than:` 值，例如 `10m`、`2h` 或 `7d`。
    pub newer_than: Option<String>,
    /// 为 `true` 时仅搜索未读邮件。
    pub unread: bool,
    /// 最多检查的邮件数量。
    pub max_results: u32,
    /// Gmail 标签 ID，例如 `INBOX`。
    pub label_ids: Vec<String>,
}

impl GmailCodeQuery {
    /// 创建带保守结果数量限制的查询。
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// 设置原始 Gmail 查询语法。
    #[must_use]
    pub fn query(mut self, value: impl Into<String>) -> Self {
        self.query = Some(value.into());
        self
    }

    /// 添加 Gmail `from:` 过滤条件。
    #[must_use]
    pub fn from(mut self, value: impl Into<String>) -> Self {
        self.from = Some(value.into());
        self
    }

    /// 添加 Gmail `subject:` 过滤条件。
    #[must_use]
    pub fn subject(mut self, value: impl Into<String>) -> Self {
        self.subject = Some(value.into());
        self
    }

    /// 添加 Gmail `newer_than:` 过滤条件，例如 `10m`、`2h` 或 `7d`。
    #[must_use]
    pub fn newer_than(mut self, value: impl Into<String>) -> Self {
        self.newer_than = Some(value.into());
        self
    }

    /// 为 `true` 时仅搜索未读邮件。
    #[must_use]
    pub const fn unread(mut self, value: bool) -> Self {
        self.unread = value;
        self
    }

    /// 设置最多检查的邮件数量；取值会被限制到 `1..=100`。
    #[must_use]
    pub const fn max_results(mut self, value: u32) -> Self {
        self.max_results = clamp_max_results(value);
        self
    }

    /// 添加 Gmail 标签 ID 过滤条件，例如 `INBOX`。
    #[must_use]
    pub fn label_id(mut self, value: impl Into<String>) -> Self {
        self.label_ids.push(value.into());
        self
    }

    pub(crate) fn gmail_q(&self) -> String {
        let mut parts = Vec::new();
        push_filter(&mut parts, "from", self.from.as_deref());
        push_filter(&mut parts, "subject", self.subject.as_deref());
        push_filter(&mut parts, "newer_than", self.newer_than.as_deref());
        if self.unread {
            parts.push("is:unread".to_owned());
        }
        if let Some(query) = trimmed(self.query.as_deref()) {
            parts.push(query.to_owned());
        }
        parts.join(" ")
    }
}

impl_default!(GmailCodeQuery => GmailCodeQuery {
    query: None,
    from: None,
    subject: None,
    newer_than: Some("10m".to_owned()),
    unread: false,
    max_results: DEFAULT_MAX_RESULTS,
    label_ids: vec!["INBOX".to_owned()],
});

const fn clamp_max_results(value: u32) -> u32 {
    if value == 0 {
        1
    } else if value > 100 {
        100
    } else {
        value
    }
}

fn push_filter(parts: &mut Vec<String>, name: &str, value: Option<&str>) {
    if let Some(value) = trimmed(value) {
        parts.push(format!("{name}:{}", quote_query_value(value)));
    }
}

fn trimmed(value: Option<&str>) -> Option<&str> {
    value.and_then(|value| {
        let value = value.trim();
        if value.is_empty() { None } else { Some(value) }
    })
}

fn quote_query_value(value: &str) -> String {
    if value
        .chars()
        .any(|ch| ch.is_whitespace() || matches!(ch, '"' | '(' | ')'))
    {
        format!("\"{}\"", value.replace('"', "\\\""))
    } else {
        value.to_owned()
    }
}

#[cfg(test)]
mod tests {
    use super::{GmailCodeConfig, GmailCodeQuery};

    #[test]
    fn query_builder_combines_structured_filters() {
        let query = GmailCodeQuery::new()
            .from("security@example.com")
            .subject("login code")
            .newer_than("2h")
            .unread(true)
            .query("category:primary");

        assert_eq!(
            query.gmail_q(),
            r#"from:security@example.com subject:"login code" newer_than:2h is:unread category:primary"#
        );
    }

    #[test]
    fn max_results_is_clamped_to_gmail_limit() {
        assert_eq!(GmailCodeQuery::new().max_results(0).max_results, 1);
        assert_eq!(GmailCodeQuery::new().max_results(150).max_results, 100);
    }

    #[test]
    fn blank_token_is_rejected_before_network_io() {
        let error = GmailCodeConfig::builder("  ")
            .build()
            .expect_err("blank token should fail");

        assert!(error.to_string().contains("access_token"));
    }
}
