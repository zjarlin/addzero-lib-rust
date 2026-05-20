use crate::{GmailCodeError, GmailCodeResult};
use std::time::Duration;

const DEFAULT_GMAIL_API_BASE_URL: &str = "https://gmail.googleapis.com/gmail/v1/";
const DEFAULT_USER_ID: &str = "me";
const DEFAULT_USER_AGENT: &str = "az-gmail-code/2026.5";
const DEFAULT_CONNECT_TIMEOUT_SECS: u64 = 10;
const DEFAULT_REQUEST_TIMEOUT_SECS: u64 = 30;
const DEFAULT_MAX_RESULTS: u32 = 10;

/// Configuration for authorized Gmail API requests.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GmailCodeConfig {
    /// Gmail API OAuth access token for a mailbox controlled by the caller.
    pub access_token: String,
    /// Gmail API base URL. Keep the default for production use.
    pub base_url: String,
    /// Gmail `userId` path segment. `me` is the normal value for OAuth callers.
    pub user_id: String,
    /// HTTP user agent sent by this client.
    pub user_agent: Option<String>,
    /// TCP connect timeout.
    pub connect_timeout: Duration,
    /// Total request timeout.
    pub request_timeout: Duration,
}

impl GmailCodeConfig {
    /// Starts a config builder with the required Gmail OAuth access token.
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

    pub(crate) fn validate(&self) -> GmailCodeResult<()> {
        if self.access_token.trim().is_empty() {
            return Err(GmailCodeError::InvalidConfig(
                "access_token cannot be blank".to_owned(),
            ));
        }
        if self.base_url.trim().is_empty() {
            return Err(GmailCodeError::InvalidConfig(
                "base_url cannot be blank".to_owned(),
            ));
        }
        if self.user_id.trim().is_empty() {
            return Err(GmailCodeError::InvalidConfig(
                "user_id cannot be blank".to_owned(),
            ));
        }
        Ok(())
    }
}

/// Builder for [`GmailCodeConfig`].
#[derive(Debug, Clone)]
pub struct GmailCodeConfigBuilder {
    config: GmailCodeConfig,
}

impl GmailCodeConfigBuilder {
    /// Overrides the Gmail API base URL, primarily for tests.
    #[must_use]
    pub fn base_url(mut self, value: impl Into<String>) -> Self {
        self.config.base_url = value.into();
        self
    }

    /// Overrides the Gmail `userId` path segment.
    #[must_use]
    pub fn user_id(mut self, value: impl Into<String>) -> Self {
        self.config.user_id = value.into();
        self
    }

    /// Overrides or clears the HTTP user agent.
    #[must_use]
    pub fn user_agent(mut self, value: Option<impl Into<String>>) -> Self {
        self.config.user_agent = value.map(Into::into);
        self
    }

    /// Sets the TCP connect timeout.
    #[must_use]
    pub const fn connect_timeout(mut self, value: Duration) -> Self {
        self.config.connect_timeout = value;
        self
    }

    /// Sets the total request timeout.
    #[must_use]
    pub const fn request_timeout(mut self, value: Duration) -> Self {
        self.config.request_timeout = value;
        self
    }

    /// Validates and returns the config.
    pub fn build(self) -> GmailCodeResult<GmailCodeConfig> {
        self.config.validate()?;
        Ok(self.config)
    }
}

/// Search parameters for finding Gmail verification-code messages.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GmailCodeQuery {
    /// Raw Gmail search expression appended after structured filters.
    pub query: Option<String>,
    /// Optional sender filter.
    pub from: Option<String>,
    /// Optional subject filter.
    pub subject: Option<String>,
    /// Optional Gmail `newer_than:` value such as `10m`, `2h`, or `7d`.
    pub newer_than: Option<String>,
    /// Restricts results to unread messages when true.
    pub unread: bool,
    /// Maximum number of messages to inspect.
    pub max_results: u32,
    /// Gmail label ids, for example `INBOX`.
    pub label_ids: Vec<String>,
}

impl GmailCodeQuery {
    /// Creates a query with a conservative result limit.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets raw Gmail query syntax.
    #[must_use]
    pub fn query(mut self, value: impl Into<String>) -> Self {
        self.query = Some(value.into());
        self
    }

    /// Adds a Gmail `from:` filter.
    #[must_use]
    pub fn from(mut self, value: impl Into<String>) -> Self {
        self.from = Some(value.into());
        self
    }

    /// Adds a Gmail `subject:` filter.
    #[must_use]
    pub fn subject(mut self, value: impl Into<String>) -> Self {
        self.subject = Some(value.into());
        self
    }

    /// Adds a Gmail `newer_than:` filter, for example `10m`, `2h`, or `7d`.
    #[must_use]
    pub fn newer_than(mut self, value: impl Into<String>) -> Self {
        self.newer_than = Some(value.into());
        self
    }

    /// Restricts results to unread messages when true.
    #[must_use]
    pub const fn unread(mut self, value: bool) -> Self {
        self.unread = value;
        self
    }

    /// Sets the maximum number of messages to inspect. Values are clamped to `1..=100`.
    #[must_use]
    pub const fn max_results(mut self, value: u32) -> Self {
        self.max_results = clamp_max_results(value);
        self
    }

    /// Adds a Gmail label id filter, such as `INBOX`.
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

impl Default for GmailCodeQuery {
    fn default() -> Self {
        Self {
            query: None,
            from: None,
            subject: None,
            newer_than: Some("10m".to_owned()),
            unread: false,
            max_results: DEFAULT_MAX_RESULTS,
            label_ids: vec!["INBOX".to_owned()],
        }
    }
}

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
