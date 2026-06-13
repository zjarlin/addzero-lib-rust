use crate::http::HttpClient;
use crate::otp::extract_verification_code;
use crate::random::{random_local_part, random_password};
use crate::config::DuckMailConfig;
use anyhow::{anyhow, bail};
use az_derive_aliases::{
    apply, deserialize_debug, plain_clone_debug, plain_eq, serde_eq, serde_eq_default,
};
use serde_json::{Value, json};
use std::time::{Duration, Instant};

/// Blocking DuckMail API client.
#[apply(plain_clone_debug)]
pub struct DuckMailApi {
    config: DuckMailConfig,
    http: HttpClient,
}

impl DuckMailApi {
    /// Creates a client from validated DuckMail API configuration.
    pub fn new(config: DuckMailConfig) -> anyhow::Result<Self> {
        let config = config.build()?;
        Ok(Self {
            http: HttpClient::new(&config)?,
            config,
        })
    }

    /// Creates a client for the public DuckMail API with an optional bearer token or `dk_` key.
    pub fn default_with_auth_token(token: impl Into<String>) -> anyhow::Result<Self> {
        Self::new(DuckMailConfig::default().auth_token(token))
    }

    /// Lists verified DuckMail domains visible to the configured credential.
    pub fn list_domains(&self, page: usize) -> anyhow::Result<Vec<DuckMailDomain>> {
        let response = HttpClient::with_bearer_auth(
            self.http.get("/domains")?,
            self.config.auth_token.as_deref(),
        )
        .query(&[("page", page.max(1).to_string())])
        .send()?;
        let response: HydraCollection<DuckMailDomain> = HttpClient::read_json(response)?;
        Ok(response.items)
    }

    /// Creates a DuckMail account for an explicit address and password.
    pub fn create_account(
        &self,
        address: impl AsRef<str>,
        password: impl AsRef<str>,
    ) -> anyhow::Result<DuckMailAccount> {
        let response = HttpClient::with_bearer_auth(
            self.http.post("/accounts")?,
            self.config.auth_token.as_deref(),
        )
        .json(&json!({
            "address": address.as_ref().trim(),
            "password": password.as_ref(),
        }))
        .send()?;
        HttpClient::read_json(response)
    }

    /// Gets a DuckMail bearer token for an existing account.
    pub fn create_token(
        &self,
        address: impl AsRef<str>,
        password: impl AsRef<str>,
    ) -> anyhow::Result<DuckMailToken> {
        let response = self
            .http
            .post("/token")?
            .json(&json!({
                "address": address.as_ref().trim(),
                "password": password.as_ref(),
            }))
            .send()?;
        HttpClient::read_json(response)
    }

    /// Creates an explicit mailbox and immediately logs in to get its inbox token.
    pub fn create_mailbox_and_login(
        &self,
        address: impl AsRef<str>,
        password: impl AsRef<str>,
    ) -> anyhow::Result<DuckMailMailbox> {
        let address = address.as_ref().trim().to_owned();
        let password = password.as_ref().to_owned();
        let account = self.create_account(&address, &password)?;
        let token = self.create_token(&address, &password)?;

        Ok(DuckMailMailbox {
            address,
            password,
            account_id: account.id,
            token: token.token,
        })
    }

    /// Creates a random DuckMail mailbox on the provided domain, or the first verified domain.
    pub fn create_random_mailbox_and_login(
        &self,
        domain: Option<&str>,
    ) -> anyhow::Result<DuckMailMailbox> {
        let domain = match domain.map(str::trim).filter(|value| !value.is_empty()) {
            Some(domain) => domain.to_owned(),
            None => self
                .list_domains(1)?
                .into_iter()
                .find(|domain| domain.is_verified)
                .map(|domain| domain.domain)
                .ok_or_else(|| {
                    anyhow!("invalid response: no verified DuckMail domains available")
                })?,
        };
        let address = format!("{}@{domain}", random_local_part(12)?);
        let password = random_password(16)?;
        self.create_mailbox_and_login(address, password)
    }

    /// Lists inbox message summaries for a DuckMail bearer token.
    pub fn list_messages(
        &self,
        mail_token: impl AsRef<str>,
        page: usize,
    ) -> anyhow::Result<Vec<DuckMailMessageSummary>> {
        let response =
            HttpClient::with_bearer_auth(self.http.get("/messages")?, Some(mail_token.as_ref()))
                .query(&[("page", page.max(1).to_string())])
                .send()?;
        let response: HydraCollection<DuckMailMessageSummary> = HttpClient::read_json(response)?;
        Ok(response.items)
    }

    /// Gets a full DuckMail message, including text and HTML body fields.
    pub fn get_message(
        &self,
        mail_token: impl AsRef<str>,
        message_id: impl AsRef<str>,
    ) -> anyhow::Result<DuckMailMessageDetail> {
        let message_id = normalize_message_id(message_id.as_ref());
        let path = format!("/messages/{message_id}");
        let response =
            HttpClient::with_bearer_auth(self.http.get(&path)?, Some(mail_token.as_ref()))
                .send()?;
        let response: DuckMailMessageDetailRaw = HttpClient::read_json(response)?;
        DuckMailMessageDetail::try_from_raw(response)
    }

    /// Polls the inbox until a six-digit verification code is found or the timeout expires.
    pub fn wait_for_verification_code(
        &self,
        mail_token: impl AsRef<str>,
        timeout: Duration,
        poll_interval: Duration,
    ) -> anyhow::Result<Option<String>> {
        let mail_token = mail_token.as_ref();
        let started = Instant::now();

        while started.elapsed() < timeout {
            for message in self.list_messages(mail_token, 1)? {
                let detail = self.get_message(mail_token, &message.id)?;
                if let Some(code) = extract_verification_code(detail.body()) {
                    return Ok(Some(code));
                }
            }
            std::thread::sleep(poll_interval);
        }

        Ok(None)
    }
}

/// DuckMail domain metadata.
#[apply(serde_eq)]
pub struct DuckMailDomain {
    pub id: String,
    pub domain: String,
    #[serde(default, rename = "ownerId")]
    pub owner_id: Option<String>,
    #[serde(default, rename = "isVerified")]
    pub is_verified: bool,
    #[serde(default, rename = "verificationToken")]
    pub verification_token: Option<String>,
    #[serde(default, rename = "createdAt")]
    pub created_at: Option<String>,
    #[serde(default, rename = "updatedAt")]
    pub updated_at: Option<String>,
}

/// DuckMail account creation response.
#[apply(serde_eq)]
pub struct DuckMailAccount {
    pub id: String,
    pub address: String,
    #[serde(default, rename = "authType")]
    pub auth_type: Option<String>,
    #[serde(default, rename = "createdAt")]
    pub created_at: Option<String>,
    #[serde(default, rename = "updatedAt")]
    pub updated_at: Option<String>,
}

/// DuckMail token response.
#[apply(serde_eq)]
pub struct DuckMailToken {
    pub id: String,
    pub token: String,
}

/// A created DuckMail mailbox with the account password and inbox bearer token.
#[apply(plain_eq)]
pub struct DuckMailMailbox {
    pub address: String,
    pub password: String,
    pub account_id: String,
    pub token: String,
}

/// Email address object returned by DuckMail.
#[apply(serde_eq_default)]
pub struct MailAddress {
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub address: String,
}

/// DuckMail inbox message summary.
#[apply(serde_eq)]
pub struct DuckMailMessageSummary {
    pub id: String,
    #[serde(default)]
    pub msgid: Option<String>,
    #[serde(default, rename = "accountId")]
    pub account_id: Option<String>,
    #[serde(default)]
    pub from: MailAddress,
    #[serde(default)]
    pub to: Vec<MailAddress>,
    #[serde(default)]
    pub subject: String,
    #[serde(default)]
    pub seen: bool,
    #[serde(default, rename = "isDeleted")]
    pub is_deleted: bool,
    #[serde(default, rename = "hasAttachments")]
    pub has_attachments: bool,
    #[serde(default)]
    pub size: Option<u64>,
    #[serde(default, rename = "downloadUrl")]
    pub download_url: Option<String>,
    #[serde(default, rename = "createdAt")]
    pub created_at: Option<String>,
    #[serde(default, rename = "updatedAt")]
    pub updated_at: Option<String>,
}

/// DuckMail attachment metadata.
#[apply(serde_eq)]
pub struct DuckMailAttachment {
    pub id: String,
    #[serde(default)]
    pub filename: String,
    #[serde(default, rename = "contentType")]
    pub content_type: String,
    #[serde(default)]
    pub disposition: String,
    #[serde(default, rename = "transferEncoding")]
    pub transfer_encoding: String,
    #[serde(default)]
    pub related: bool,
    #[serde(default)]
    pub size: Option<u64>,
    #[serde(default, rename = "downloadUrl")]
    pub download_url: Option<String>,
}

/// Full DuckMail message detail.
#[apply(plain_eq)]
pub struct DuckMailMessageDetail {
    pub id: String,
    pub msgid: Option<String>,
    pub account_id: Option<String>,
    pub from: MailAddress,
    pub to: Vec<MailAddress>,
    pub subject: String,
    pub text: String,
    pub html: Vec<String>,
    pub seen: bool,
    pub is_deleted: bool,
    pub has_attachments: bool,
    pub size: Option<u64>,
    pub download_url: Option<String>,
    pub attachments: Vec<DuckMailAttachment>,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}

impl DuckMailMessageDetail {
    /// Returns the plain text body when present, otherwise concatenated HTML.
    pub fn body(&self) -> String {
        if !self.text.trim().is_empty() {
            return self.text.clone();
        }
        self.html.join("")
    }

    fn try_from_raw(raw: DuckMailMessageDetailRaw) -> anyhow::Result<Self> {
        Ok(Self {
            id: raw.id,
            msgid: raw.msgid,
            account_id: raw.account_id,
            from: raw.from,
            to: raw.to,
            subject: raw.subject,
            text: raw.text.unwrap_or_default(),
            html: html_to_vec(raw.html)?,
            seen: raw.seen,
            is_deleted: raw.is_deleted,
            has_attachments: raw.has_attachments,
            size: raw.size,
            download_url: raw.download_url,
            attachments: raw.attachments,
            created_at: raw.created_at,
            updated_at: raw.updated_at,
        })
    }
}

#[apply(deserialize_debug)]
#[serde(bound(deserialize = "T: ::serde::Deserialize<'de>"))]
struct HydraCollection<T> {
    #[serde(
        default = "empty_items",
        rename = "hydra:member",
        alias = "member",
        alias = "data",
        alias = "items"
    )]
    items: Vec<T>,
}

fn empty_items<T>() -> Vec<T> {
    Vec::new()
}

#[apply(deserialize_debug)]
struct DuckMailMessageDetailRaw {
    id: String,
    #[serde(default)]
    msgid: Option<String>,
    #[serde(default, rename = "accountId")]
    account_id: Option<String>,
    #[serde(default)]
    from: MailAddress,
    #[serde(default)]
    to: Vec<MailAddress>,
    #[serde(default)]
    subject: String,
    #[serde(default)]
    text: Option<String>,
    #[serde(default)]
    html: Value,
    #[serde(default)]
    seen: bool,
    #[serde(default, rename = "isDeleted")]
    is_deleted: bool,
    #[serde(default, rename = "hasAttachments")]
    has_attachments: bool,
    #[serde(default)]
    size: Option<u64>,
    #[serde(default, rename = "downloadUrl")]
    download_url: Option<String>,
    #[serde(default)]
    attachments: Vec<DuckMailAttachment>,
    #[serde(default, rename = "createdAt")]
    created_at: Option<String>,
    #[serde(default, rename = "updatedAt")]
    updated_at: Option<String>,
}

fn html_to_vec(value: Value) -> anyhow::Result<Vec<String>> {
    match value {
        Value::Null => Ok(Vec::new()),
        Value::String(value) => Ok(vec![value]),
        Value::Array(values) => values
            .into_iter()
            .map(|value| {
                value.as_str().map(ToOwned::to_owned).ok_or_else(|| {
                    anyhow!("invalid response: message html array must contain only strings")
                })
            })
            .collect(),
        other => bail!("invalid response: message html field should be a string or array, got {other}"),
    }
}

fn normalize_message_id(value: &str) -> String {
    value
        .trim()
        .strip_prefix("/messages/")
        .unwrap_or_else(|| value.trim())
        .to_owned()
}

#[cfg(test)]
mod tests {
    use super::{
        DuckMailMessageDetail, DuckMailMessageDetailRaw, html_to_vec, normalize_message_id,
    };
    use serde_json::json;

    #[test]
    fn normalize_message_id_strips_api_prefix() {
        assert_eq!(normalize_message_id("/messages/msg-1"), "msg-1");
    }

    #[test]
    fn html_to_vec_accepts_string_and_array() {
        assert_eq!(
            html_to_vec(json!("<p>one</p>")).expect("html string"),
            vec!["<p>one</p>"]
        );
        assert_eq!(
            html_to_vec(json!(["<p>one</p>", "<p>two</p>"])).expect("html array"),
            vec!["<p>one</p>", "<p>two</p>"]
        );
    }

    #[test]
    fn message_body_prefers_text() {
        let raw: DuckMailMessageDetailRaw = serde_json::from_value(json!({
            "id": "msg-1",
            "text": "plain",
            "html": ["<p>html</p>"]
        }))
        .expect("raw detail");
        let detail = DuckMailMessageDetail::try_from_raw(raw).expect("detail");

        assert_eq!(detail.body(), "plain");
    }
}
