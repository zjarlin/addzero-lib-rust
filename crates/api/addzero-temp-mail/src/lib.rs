#![forbid(unsafe_code)]

mod config;
mod error;
mod http;
mod util;

pub use config::{ApiConfig, ApiConfigBuilder};
pub use error::{
    CreatesError, CreatesError as TempMailError, CreatesResult, CreatesResult as TempMailResult,
};

use crate::http::HttpApiClient;
use crate::util::{non_blank, random_alpha_numeric, sanitize_prefix};
use reqwest::header::ACCEPT;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

#[derive(Debug, Clone)]
pub struct TempMailApi {
    http: HttpApiClient,
}

impl TempMailApi {
    pub fn new(config: ApiConfig) -> TempMailResult<Self> {
        Ok(Self {
            http: HttpApiClient::new(config)?,
        })
    }

    pub fn get_domains(&self) -> TempMailResult<Vec<TempMailDomain>> {
        let response = self.http.get("/domains")?.send()?;
        let response: HydraCollection<TempMailDomain> = HttpApiClient::read_json(response)?;
        Ok(response.items)
    }

    pub fn create_mailbox_and_login(
        &self,
        prefix: impl AsRef<str>,
        password_length: usize,
    ) -> TempMailResult<TempMailMailbox> {
        let domains = self
            .get_domains()?
            .into_iter()
            .filter(|domain| domain.is_active)
            .collect::<Vec<_>>();

        let chosen_domain = domains
            .first()
            .map(|domain| domain.domain.clone())
            .ok_or_else(|| {
                TempMailError::InvalidResponse("no active temp-mail domains available".to_owned())
            })?;

        let local_part = format!(
            "{}{}",
            sanitize_prefix(prefix.as_ref()),
            random_alpha_numeric(8)
        );
        let address = format!("{local_part}@{chosen_domain}");
        let password = random_alpha_numeric(password_length.max(8));
        let account_id = self.create_account(&address, &password)?;
        let token = self.create_token(&address, &password)?;

        Ok(TempMailMailbox {
            address,
            password,
            account_id,
            token,
        })
    }

    pub fn create_account(
        &self,
        address: impl AsRef<str>,
        password: impl AsRef<str>,
    ) -> TempMailResult<String> {
        let response = self
            .http
            .post("/accounts")?
            .json(&json!({
                "address": address.as_ref().trim(),
                "password": password.as_ref(),
            }))
            .send()?;
        let response: TempMailAccountResponse = HttpApiClient::read_json(response)?;

        non_blank(Some(response.id.as_str()))
            .map(ToOwned::to_owned)
            .ok_or_else(|| {
                TempMailError::InvalidResponse(format!(
                    "create account failed: id missing for address={}",
                    address.as_ref().trim()
                ))
            })
    }

    pub fn create_token(
        &self,
        address: impl AsRef<str>,
        password: impl AsRef<str>,
    ) -> TempMailResult<String> {
        let response = self
            .http
            .post("/token")?
            .json(&json!({
                "address": address.as_ref().trim(),
                "password": password.as_ref(),
            }))
            .send()?;
        let response: TempMailTokenResponse = HttpApiClient::read_json(response)?;

        non_blank(Some(response.token.as_str()))
            .map(ToOwned::to_owned)
            .ok_or_else(|| {
                TempMailError::InvalidResponse(format!(
                    "create token failed: token missing for address={}",
                    address.as_ref().trim()
                ))
            })
    }

    pub fn list_messages(
        &self,
        token: impl AsRef<str>,
        page: usize,
    ) -> TempMailResult<Vec<TempMailMessageSummary>> {
        let response =
            HttpApiClient::with_bearer_auth(self.http.get("/messages")?, Some(token.as_ref()))
                .query(&[("page", page.max(1).to_string())])
                .send()?;
        let response: HydraCollection<TempMailMessageSummaryRaw> =
            HttpApiClient::read_json(response)?;

        Ok(response
            .items
            .into_iter()
            .filter_map(TempMailMessageSummary::try_from_raw)
            .collect())
    }

    pub fn get_message(
        &self,
        token: impl AsRef<str>,
        message_id: impl AsRef<str>,
    ) -> TempMailResult<TempMailMessageDetail> {
        let path = format!("/messages/{}", message_id.as_ref().trim());
        let response =
            HttpApiClient::with_bearer_auth(self.http.get(&path)?, Some(token.as_ref())).send()?;
        let response: TempMailMessageDetailRaw = HttpApiClient::read_json(response)?;

        TempMailMessageDetail::try_from_raw(response)
    }
}

pub fn create_temp_mail_api() -> TempMailResult<TempMailApi> {
    let config = ApiConfig::builder("https://api.mail.tm")
        .default_header(ACCEPT.as_str(), "application/json")
        .build()?;
    TempMailApi::new(config)
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TempMailDomain {
    pub id: String,
    pub domain: String,
    #[serde(rename = "isActive")]
    pub is_active: bool,
    #[serde(rename = "isPrivate")]
    pub is_private: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TempMailMailbox {
    pub address: String,
    pub password: String,
    pub account_id: String,
    pub token: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TempMailMessageSummary {
    pub id: String,
    pub from_address: String,
    pub from_name: String,
    pub subject: String,
    pub intro: String,
    pub seen: bool,
    pub created_at: String,
}

impl TempMailMessageSummary {
    fn try_from_raw(raw: TempMailMessageSummaryRaw) -> Option<Self> {
        let id = raw.id.trim().to_owned();
        if id.is_empty() {
            return None;
        }

        Some(Self {
            id,
            from_address: raw.from.address,
            from_name: raw.from.name,
            subject: raw.subject,
            intro: raw.intro,
            seen: raw.seen,
            created_at: raw.created_at,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TempMailRecipient {
    pub address: String,
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TempMailMessageDetail {
    pub id: String,
    pub from_address: String,
    pub from_name: String,
    pub to: Vec<TempMailRecipient>,
    pub subject: String,
    pub text: String,
    pub html: String,
    pub created_at: String,
}

impl TempMailMessageDetail {
    fn try_from_raw(raw: TempMailMessageDetailRaw) -> TempMailResult<Self> {
        let html = match raw.html {
            Value::String(value) => value,
            Value::Array(values) => values
                .into_iter()
                .find_map(|item| item.as_str().map(ToOwned::to_owned))
                .unwrap_or_else(String::new),
            Value::Null => String::new(),
            other => {
                return Err(TempMailError::InvalidResponse(format!(
                    "temp-mail html field should be string or array, got {other}"
                )));
            }
        };

        Ok(Self {
            id: raw.id,
            from_address: raw.from.address,
            from_name: raw.from.name,
            to: raw
                .to
                .into_iter()
                .map(|recipient| TempMailRecipient {
                    address: recipient.address,
                    name: recipient.name,
                })
                .collect(),
            subject: raw.subject,
            text: raw.text,
            html,
            created_at: raw.created_at,
        })
    }
}

#[derive(Debug, Deserialize)]
struct HydraCollection<T> {
    #[serde(rename = "hydra:member", default = "Vec::new")]
    items: Vec<T>,
}

#[derive(Debug, Deserialize)]
struct TempMailAccountResponse {
    #[serde(default)]
    id: String,
}

#[derive(Debug, Deserialize)]
struct TempMailTokenResponse {
    #[serde(default)]
    token: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
struct TempMailSenderRaw {
    #[serde(default)]
    address: String,
    #[serde(default)]
    name: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
struct TempMailRecipientRaw {
    #[serde(default)]
    address: String,
    #[serde(default)]
    name: String,
}

#[derive(Debug, Deserialize)]
struct TempMailMessageSummaryRaw {
    #[serde(default)]
    id: String,
    from: TempMailSenderRaw,
    #[serde(default)]
    subject: String,
    #[serde(default)]
    intro: String,
    #[serde(default)]
    seen: bool,
    #[serde(rename = "createdAt", default)]
    created_at: String,
}

#[derive(Debug, Deserialize)]
struct TempMailMessageDetailRaw {
    #[serde(default)]
    id: String,
    from: TempMailSenderRaw,
    #[serde(default)]
    to: Vec<TempMailRecipientRaw>,
    #[serde(default)]
    subject: String,
    #[serde(default)]
    text: String,
    #[serde(default)]
    html: Value,
    #[serde(rename = "createdAt", default)]
    created_at: String,
}
