use crate::http::HttpApiClient;
use crate::model::{
    CreateMailboxRequest, ListResponse, PageRequest, TempMailMailbox, TempMailMessageDetail,
    TempMailMessageSummary, TempMailProviderKind, TempMailRecipient,
};
use crate::provider::TempMailProvider;
use crate::util::{random_alpha_numeric, sanitize_local_part, trim_non_blank};
use crate::{ApiConfig, TempMailError, TempMailResult};
use reqwest::header::ACCEPT;
use serde::Deserialize;
use serde_json::{Value, json};

/// Blocking client for the hosted `mail.tm` temporary email API.
#[derive(Debug, Clone)]
pub struct MailTmTempMailApi {
    http: HttpApiClient,
}

impl MailTmTempMailApi {
    /// Creates a client from explicit API configuration.
    pub fn new(config: ApiConfig) -> TempMailResult<Self> {
        Ok(Self {
            http: HttpApiClient::new(config)?,
        })
    }

    /// Lists available domains.
    pub fn get_domains(&self) -> TempMailResult<Vec<MailTmDomain>> {
        let response = self.http.get("/domains")?.send()?;
        let response: HydraCollection<MailTmDomain> = HttpApiClient::read_json(response)?;
        Ok(response.items)
    }

    /// Creates an account and login token using the first active domain.
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
                TempMailError::InvalidResponse("no active mail.tm domains available".to_owned())
            })?;

        let local_part = format!(
            "{}{}",
            sanitize_local_part(prefix.as_ref()),
            random_alpha_numeric(8)
        );
        let address = format!("{local_part}@{chosen_domain}");
        let password = random_alpha_numeric(password_length.max(8));
        let account_id = self.create_account(&address, &password)?;
        let token = self.create_token(&address, &password)?;

        Ok(TempMailMailbox {
            provider: TempMailProviderKind::MailTm,
            address,
            credential: token,
            account_id: Some(account_id),
            password: Some(password),
        })
    }

    /// Creates a mail.tm account and returns its account id.
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
        let response: MailTmAccountResponse = HttpApiClient::read_json(response)?;

        trim_non_blank(Some(response.id.as_str()))
            .map(ToOwned::to_owned)
            .ok_or_else(|| {
                TempMailError::InvalidResponse(format!(
                    "create account failed: id missing for address={}",
                    address.as_ref().trim()
                ))
            })
    }

    /// Creates a mail.tm bearer token for an account.
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
        let response: MailTmTokenResponse = HttpApiClient::read_json(response)?;

        trim_non_blank(Some(response.token.as_str()))
            .map(ToOwned::to_owned)
            .ok_or_else(|| {
                TempMailError::InvalidResponse(format!(
                    "create token failed: token missing for address={}",
                    address.as_ref().trim()
                ))
            })
    }

    /// Lists messages using a mail.tm bearer token.
    pub fn list_messages_by_token(
        &self,
        token: impl AsRef<str>,
        page: PageRequest,
    ) -> TempMailResult<ListResponse<TempMailMessageSummary>> {
        let query = [("page", mail_tm_page(page).to_string())];
        let response = HttpApiClient::with_bearer_auth(
            self.http.get_with_query("/messages", &query)?,
            Some(token.as_ref()),
        )
        .send()?;
        let response: HydraCollection<MailTmMessageSummaryRaw> =
            HttpApiClient::read_json(response)?;
        let results = response
            .items
            .into_iter()
            .filter_map(mail_tm_summary_from_raw)
            .collect::<Vec<_>>();
        let count = u64::try_from(results.len()).unwrap_or(u64::MAX);
        Ok(ListResponse { results, count })
    }

    /// Fetches a message using a mail.tm bearer token.
    pub fn get_message_by_token(
        &self,
        token: impl AsRef<str>,
        message_id: impl AsRef<str>,
    ) -> TempMailResult<TempMailMessageDetail> {
        let path = format!("/messages/{}", message_id.as_ref().trim());
        let response =
            HttpApiClient::with_bearer_auth(self.http.get(&path)?, Some(token.as_ref())).send()?;
        let response: MailTmMessageDetailRaw = HttpApiClient::read_json(response)?;

        TempMailMessageDetail::try_from(response)
    }
}

impl TempMailProvider for MailTmTempMailApi {
    fn provider_kind(&self) -> TempMailProviderKind {
        TempMailProviderKind::MailTm
    }

    fn create_mailbox(&self, request: &CreateMailboxRequest) -> TempMailResult<TempMailMailbox> {
        self.create_mailbox_and_login(
            request.name.as_deref().unwrap_or("az"),
            request.password_length,
        )
    }

    fn list_messages(
        &self,
        mailbox: &TempMailMailbox,
        page: PageRequest,
    ) -> TempMailResult<ListResponse<TempMailMessageSummary>> {
        self.list_messages_by_token(&mailbox.credential, page)
    }

    fn get_message(
        &self,
        mailbox: &TempMailMailbox,
        message_id: &str,
    ) -> TempMailResult<Option<TempMailMessageDetail>> {
        self.get_message_by_token(&mailbox.credential, message_id)
            .map(Some)
    }
}

/// Creates a client for the hosted `mail.tm` API.
pub fn create_mail_tm_api() -> TempMailResult<MailTmTempMailApi> {
    let config = ApiConfig::builder("https://api.mail.tm")
        .default_header(ACCEPT.as_str(), "application/json")
        .build()?;
    MailTmTempMailApi::new(config)
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct MailTmDomain {
    pub id: String,
    pub domain: String,
    #[serde(rename = "isActive")]
    pub is_active: bool,
    #[serde(rename = "isPrivate")]
    pub is_private: bool,
}

#[derive(Debug, Deserialize)]
struct HydraCollection<T> {
    #[serde(rename = "hydra:member", default = "Vec::new")]
    items: Vec<T>,
}

#[derive(Debug, Deserialize)]
struct MailTmAccountResponse {
    #[serde(default)]
    id: String,
}

#[derive(Debug, Deserialize)]
struct MailTmTokenResponse {
    #[serde(default)]
    token: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
struct MailTmSenderRaw {
    #[serde(default)]
    address: String,
    #[serde(default)]
    name: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
struct MailTmRecipientRaw {
    #[serde(default)]
    address: String,
    #[serde(default)]
    name: String,
}

#[derive(Debug, Deserialize)]
struct MailTmMessageSummaryRaw {
    #[serde(default)]
    id: String,
    from: MailTmSenderRaw,
    #[serde(default)]
    subject: String,
    #[serde(default)]
    intro: String,
    #[serde(rename = "createdAt", default)]
    created_at: String,
}

#[derive(Debug, Deserialize)]
struct MailTmMessageDetailRaw {
    #[serde(default)]
    id: String,
    from: MailTmSenderRaw,
    #[serde(default)]
    to: Vec<MailTmRecipientRaw>,
    #[serde(default)]
    subject: String,
    #[serde(default)]
    text: String,
    #[serde(default)]
    html: Value,
    #[serde(rename = "createdAt", default)]
    created_at: String,
}

fn mail_tm_summary_from_raw(raw: MailTmMessageSummaryRaw) -> Option<TempMailMessageSummary> {
    let id = raw.id.trim().to_owned();
    if id.is_empty() {
        return None;
    }

    Some(TempMailMessageSummary {
        id,
        from_address: raw.from.address,
        from_name: raw.from.name,
        subject: raw.subject,
        intro: raw.intro,
        created_at: raw.created_at,
    })
}

impl TryFrom<MailTmMessageDetailRaw> for TempMailMessageDetail {
    type Error = TempMailError;

    fn try_from(raw: MailTmMessageDetailRaw) -> Result<Self, Self::Error> {
        let html = match raw.html {
            Value::String(value) => value,
            Value::Array(values) => values
                .into_iter()
                .find_map(|item| item.as_str().map(ToOwned::to_owned))
                .unwrap_or_else(String::new),
            Value::Null => String::new(),
            other => {
                return Err(TempMailError::InvalidResponse(format!(
                    "mail.tm html field should be string or array, got {other}"
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
            raw: String::new(),
            created_at: raw.created_at,
        })
    }
}

const fn mail_tm_page(page: PageRequest) -> usize {
    let normalized = page.offset / page.limit;
    normalized + 1
}
