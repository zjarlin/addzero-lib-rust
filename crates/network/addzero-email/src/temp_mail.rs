use reqwest::Url;
use reqwest::blocking::{Client, RequestBuilder, Response};
use reqwest::header::{
    ACCEPT, HeaderMap, HeaderName, HeaderValue, InvalidHeaderName, InvalidHeaderValue,
};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::collections::{BTreeMap, HashSet};
use std::ops::Deref;
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use thiserror::Error;

pub type EmailResult<T> = Result<T, EmailError>;

#[derive(Debug, Error)]
pub enum EmailError {
    #[error("invalid config: {0}")]
    InvalidConfig(String),
    #[error("invalid base url `{0}`")]
    InvalidBaseUrl(String),
    #[error("invalid request path `{0}`")]
    InvalidPath(String),
    #[error("invalid header name `{name}`: {source}")]
    InvalidHeaderName {
        name: String,
        #[source]
        source: InvalidHeaderName,
    },
    #[error("invalid header value for `{name}`: {source}")]
    InvalidHeaderValue {
        name: String,
        #[source]
        source: InvalidHeaderValue,
    },
    #[error("request failed: {0}")]
    Transport(#[from] reqwest::Error),
    #[error("failed to parse json payload: {0}")]
    Json(#[from] serde_json::Error),
    #[error("request to `{url}` returned HTTP {status}: {body}")]
    HttpStatus {
        url: String,
        status: u16,
        body: String,
    },
    #[error("invalid response: {0}")]
    InvalidResponse(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TempMailConfig {
    pub base_url: String,
    pub connect_timeout: Duration,
    pub request_timeout: Duration,
    pub user_agent: Option<String>,
    pub default_headers: BTreeMap<String, String>,
}

impl TempMailConfig {
    pub fn builder(base_url: impl Into<String>) -> TempMailConfigBuilder {
        TempMailConfigBuilder {
            base_url: base_url.into(),
            connect_timeout: Duration::from_secs(10),
            request_timeout: Duration::from_secs(20),
            user_agent: Some(default_user_agent()),
            default_headers: BTreeMap::new(),
        }
    }

    pub fn validate(&self) -> EmailResult<()> {
        if self.base_url.trim().is_empty() {
            return Err(EmailError::InvalidConfig(
                "base_url cannot be blank".to_owned(),
            ));
        }
        if self.connect_timeout.is_zero() {
            return Err(EmailError::InvalidConfig(
                "connect_timeout cannot be zero".to_owned(),
            ));
        }
        if self.request_timeout.is_zero() {
            return Err(EmailError::InvalidConfig(
                "request_timeout cannot be zero".to_owned(),
            ));
        }
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct TempMailConfigBuilder {
    base_url: String,
    connect_timeout: Duration,
    request_timeout: Duration,
    user_agent: Option<String>,
    default_headers: BTreeMap<String, String>,
}

impl TempMailConfigBuilder {
    pub fn connect_timeout(mut self, value: Duration) -> Self {
        self.connect_timeout = value;
        self
    }

    pub fn request_timeout(mut self, value: Duration) -> Self {
        self.request_timeout = value;
        self
    }

    pub fn user_agent(mut self, value: impl Into<String>) -> Self {
        self.user_agent = Some(value.into());
        self
    }

    pub fn clear_user_agent(mut self) -> Self {
        self.user_agent = None;
        self
    }

    pub fn default_header(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
        self.default_headers.insert(name.into(), value.into());
        self
    }

    pub fn build(self) -> EmailResult<TempMailConfig> {
        let config = TempMailConfig {
            base_url: self.base_url,
            connect_timeout: self.connect_timeout,
            request_timeout: self.request_timeout,
            user_agent: self.user_agent,
            default_headers: self.default_headers,
        };
        config.validate()?;
        Ok(config)
    }
}

#[derive(Debug, Clone)]
pub struct TempMailClient {
    inner: TempMailApi,
}

impl TempMailClient {
    pub fn new() -> EmailResult<Self> {
        Ok(Self {
            inner: create_mail_tm_api()?,
        })
    }

    pub fn with_base_url(base_url: impl Into<String>) -> EmailResult<Self> {
        let config = TempMailConfig::builder(base_url).build()?;
        Self::with_config(config)
    }

    pub fn with_config(config: TempMailConfig) -> EmailResult<Self> {
        Ok(Self {
            inner: TempMailApi::new(config)?,
        })
    }

    pub fn into_api(self) -> TempMailApi {
        self.inner
    }
}

impl Deref for TempMailClient {
    type Target = TempMailApi;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

#[derive(Debug, Clone)]
pub struct TempMailApi {
    config: TempMailConfig,
    http: HttpApiClient,
}

impl TempMailApi {
    pub fn new(config: TempMailConfig) -> EmailResult<Self> {
        Ok(Self {
            http: HttpApiClient::new(config.clone())?,
            config,
        })
    }

    pub fn mail_tm() -> EmailResult<Self> {
        create_mail_tm_api()
    }

    pub fn config(&self) -> &TempMailConfig {
        &self.config
    }

    pub fn get_domains(&self) -> EmailResult<Vec<TempMailDomain>> {
        let response = self.http.get("/domains")?.send()?;
        let response: HydraCollection<TempMailDomain> = HttpApiClient::read_json(response)?;
        Ok(response.items)
    }

    pub fn create_mailbox_and_login(
        &self,
        prefix: impl AsRef<str>,
        password_length: usize,
    ) -> EmailResult<TempMailMailbox> {
        let chosen_domain = self
            .get_domains()?
            .into_iter()
            .find(|domain| domain.is_active && !domain.domain.trim().is_empty())
            .map(|domain| domain.domain)
            .ok_or_else(|| {
                EmailError::InvalidResponse("no active temp-mail domains available".to_owned())
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
    ) -> EmailResult<String> {
        let address = trim_required(address.as_ref(), "address")?;
        let password = trim_required(password.as_ref(), "password")?;

        let response = self
            .http
            .post("/accounts")?
            .json(&json!({
                "address": address,
                "password": password,
            }))
            .send()?;
        let response: TempMailAccountResponse = HttpApiClient::read_json(response)?;

        non_blank(Some(response.id.as_str()))
            .map(ToOwned::to_owned)
            .ok_or_else(|| {
                EmailError::InvalidResponse(format!(
                    "create account failed: id missing for address={address}"
                ))
            })
    }

    pub fn create_token(
        &self,
        address: impl AsRef<str>,
        password: impl AsRef<str>,
    ) -> EmailResult<String> {
        let address = trim_required(address.as_ref(), "address")?;
        let password = trim_required(password.as_ref(), "password")?;

        let response = self
            .http
            .post("/token")?
            .json(&json!({
                "address": address,
                "password": password,
            }))
            .send()?;
        let response: TempMailTokenResponse = HttpApiClient::read_json(response)?;

        non_blank(Some(response.token.as_str()))
            .map(ToOwned::to_owned)
            .ok_or_else(|| {
                EmailError::InvalidResponse(format!(
                    "create token failed: token missing for address={address}"
                ))
            })
    }

    pub fn list_messages(
        &self,
        token: impl AsRef<str>,
        page: usize,
    ) -> EmailResult<Vec<TempMailMessageSummary>> {
        let token = trim_required(token.as_ref(), "token")?;
        let response = HttpApiClient::with_bearer_auth(self.http.get("/messages")?, Some(token))
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
    ) -> EmailResult<TempMailMessageDetail> {
        let token = trim_required(token.as_ref(), "token")?;
        let message_id = trim_required(message_id.as_ref(), "message_id")?;
        let path = format!("/messages/{message_id}");
        let response =
            HttpApiClient::with_bearer_auth(self.http.get(&path)?, Some(token)).send()?;
        let response: TempMailMessageDetailRaw = HttpApiClient::read_json(response)?;
        TempMailMessageDetail::try_from_raw(response)
    }

    pub fn wait_for_message(
        &self,
        token: impl AsRef<str>,
        max_wait: Duration,
        poll_interval: Duration,
    ) -> EmailResult<TempMailMessageDetail> {
        let token = trim_required(token.as_ref(), "token")?;
        validate_poll_durations(max_wait, poll_interval)?;
        let started = Instant::now();

        loop {
            if let Some(summary) = self.list_messages(token, 1)?.into_iter().next() {
                return self.get_message(token, summary.id);
            }

            if started.elapsed() >= max_wait {
                return Err(EmailError::InvalidResponse(format!(
                    "timed out waiting for message after {:?}",
                    max_wait
                )));
            }

            thread::sleep(poll_interval);
        }
    }

    pub fn wait_for_code(
        &self,
        token: impl AsRef<str>,
        max_wait: Duration,
        poll_interval: Duration,
    ) -> EmailResult<TempMailVerificationCode> {
        let token = trim_required(token.as_ref(), "token")?;
        validate_poll_durations(max_wait, poll_interval)?;
        let started = Instant::now();
        let mut inspected_ids = HashSet::new();

        loop {
            let messages = self.list_messages(token, 1)?;
            for summary in messages {
                if !inspected_ids.insert(summary.id.clone()) {
                    continue;
                }

                let message = self.get_message(token, summary.id.as_str())?;
                if let Some(code) = extract_verification_code_from_message(&message) {
                    return Ok(TempMailVerificationCode {
                        code,
                        message_id: message.id,
                        subject: message.subject,
                        from_address: message.from_address,
                    });
                }
            }

            if started.elapsed() >= max_wait {
                return Err(EmailError::InvalidResponse(format!(
                    "timed out waiting for verification code after {:?}",
                    max_wait
                )));
            }

            thread::sleep(poll_interval);
        }
    }

    pub fn wait_for_mailbox_code(
        &self,
        mailbox: &TempMailMailbox,
        max_wait: Duration,
        poll_interval: Duration,
    ) -> EmailResult<TempMailVerificationCode> {
        self.wait_for_code(mailbox.token.as_str(), max_wait, poll_interval)
    }
}

pub fn create_mail_tm_api() -> EmailResult<TempMailApi> {
    let config = TempMailConfig::builder("https://api.mail.tm")
        .default_header(ACCEPT.as_str(), "application/ld+json")
        .build()?;
    TempMailApi::new(config)
}

pub fn create_temp_mail_client() -> EmailResult<TempMailClient> {
    TempMailClient::new()
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
    fn try_from_raw(raw: TempMailMessageDetailRaw) -> EmailResult<Self> {
        let html = match raw.html {
            Value::String(value) => value,
            Value::Array(values) => values
                .into_iter()
                .find_map(|item| item.as_str().map(ToOwned::to_owned))
                .unwrap_or_default(),
            Value::Null => String::new(),
            other => {
                return Err(EmailError::InvalidResponse(format!(
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TempMailVerificationCode {
    pub code: String,
    pub message_id: String,
    pub subject: String,
    pub from_address: String,
}

pub fn extract_verification_code(value: &str) -> Option<String> {
    extract_verification_code_with(value, 4, 8)
}

pub fn extract_verification_code_with(
    value: &str,
    min_length: usize,
    max_length: usize,
) -> Option<String> {
    if min_length == 0 || min_length > max_length {
        return None;
    }

    let mut digits = String::new();
    for ch in value.chars() {
        if ch.is_ascii_digit() {
            digits.push(ch);
            continue;
        }

        if (min_length..=max_length).contains(&digits.len()) {
            return Some(digits);
        }
        digits.clear();
    }

    if (min_length..=max_length).contains(&digits.len()) {
        Some(digits)
    } else {
        None
    }
}

fn extract_verification_code_from_message(message: &TempMailMessageDetail) -> Option<String> {
    [
        message.subject.as_str(),
        message.text.as_str(),
        message.html.as_str(),
    ]
    .into_iter()
    .find_map(extract_verification_code)
}

#[derive(Debug, Clone)]
struct HttpApiClient {
    base_url: Url,
    client: Client,
}

impl HttpApiClient {
    fn new(config: TempMailConfig) -> EmailResult<Self> {
        config.validate()?;
        let base_url = Url::parse(&config.base_url)
            .map_err(|_| EmailError::InvalidBaseUrl(config.base_url.clone()))?;
        let default_headers = build_header_map(&config.default_headers)?;

        let mut builder = Client::builder()
            .connect_timeout(config.connect_timeout)
            .timeout(config.request_timeout)
            .default_headers(default_headers);

        if let Some(user_agent) = config.user_agent {
            builder = builder.user_agent(user_agent);
        }

        Ok(Self {
            base_url,
            client: builder.build()?,
        })
    }

    fn get(&self, path: &str) -> EmailResult<RequestBuilder> {
        Ok(self.client.get(self.join_url(path)?))
    }

    fn post(&self, path: &str) -> EmailResult<RequestBuilder> {
        Ok(self.client.post(self.join_url(path)?))
    }

    fn with_bearer_auth(builder: RequestBuilder, bearer_token: Option<&str>) -> RequestBuilder {
        match trim_non_blank(bearer_token) {
            Some(token) => builder.bearer_auth(token),
            None => builder,
        }
    }

    fn read_json<T: DeserializeOwned>(response: Response) -> EmailResult<T> {
        let response = Self::ensure_success(response)?;
        let bytes = response.bytes()?;
        Ok(serde_json::from_slice(bytes.as_ref())?)
    }

    fn ensure_success(response: Response) -> EmailResult<Response> {
        let status = response.status();
        if status.is_success() {
            return Ok(response);
        }

        let url = response.url().to_string();
        let body = match response.bytes() {
            Ok(bytes) => String::from_utf8_lossy(bytes.as_ref()).into_owned(),
            Err(error) => return Err(EmailError::Transport(error)),
        };

        Err(EmailError::HttpStatus {
            url,
            status: status.as_u16(),
            body,
        })
    }

    fn join_url(&self, path: &str) -> EmailResult<Url> {
        self.base_url
            .join(path)
            .map_err(|_| EmailError::InvalidPath(path.to_owned()))
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

#[derive(Debug, Default, Deserialize)]
struct TempMailNamedAddressRaw {
    #[serde(default)]
    address: String,
    #[serde(default)]
    name: String,
}

#[derive(Debug, Deserialize)]
struct TempMailMessageSummaryRaw {
    #[serde(default)]
    id: String,
    #[serde(default)]
    from: TempMailNamedAddressRaw,
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
    #[serde(default)]
    from: TempMailNamedAddressRaw,
    #[serde(default)]
    to: Vec<TempMailNamedAddressRaw>,
    #[serde(default)]
    subject: String,
    #[serde(default)]
    text: String,
    #[serde(default)]
    html: Value,
    #[serde(rename = "createdAt", default)]
    created_at: String,
}

fn build_header_map(headers: &BTreeMap<String, String>) -> EmailResult<HeaderMap> {
    let mut header_map = HeaderMap::new();
    for (name, value) in headers {
        let header_name = HeaderName::from_bytes(name.as_bytes()).map_err(|source| {
            EmailError::InvalidHeaderName {
                name: name.clone(),
                source,
            }
        })?;
        let header_value =
            HeaderValue::from_str(value).map_err(|source| EmailError::InvalidHeaderValue {
                name: name.clone(),
                source,
            })?;
        header_map.insert(header_name, header_value);
    }
    Ok(header_map)
}

fn default_user_agent() -> String {
    format!("{}/{}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"))
}

fn non_blank(value: Option<&str>) -> Option<&str> {
    trim_non_blank(value)
}

fn trim_non_blank(value: Option<&str>) -> Option<&str> {
    value.and_then(|item| {
        let trimmed = item.trim();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed)
        }
    })
}

fn trim_required<'a>(value: &'a str, field: &str) -> EmailResult<&'a str> {
    trim_non_blank(Some(value))
        .ok_or_else(|| EmailError::InvalidConfig(format!("{field} cannot be blank")))
}

fn validate_poll_durations(max_wait: Duration, poll_interval: Duration) -> EmailResult<()> {
    if max_wait.is_zero() {
        return Err(EmailError::InvalidConfig(
            "max_wait cannot be zero".to_owned(),
        ));
    }
    if poll_interval.is_zero() {
        return Err(EmailError::InvalidConfig(
            "poll_interval cannot be zero".to_owned(),
        ));
    }
    Ok(())
}

fn sanitize_prefix(prefix: &str) -> String {
    let sanitized = prefix
        .chars()
        .filter(char::is_ascii_alphanumeric)
        .collect::<String>()
        .to_ascii_lowercase();

    if sanitized.is_empty() {
        "az".to_owned()
    } else {
        sanitized
    }
}

fn random_alpha_numeric(length: usize) -> String {
    const ALPHABET: &[u8] = b"abcdefghijklmnopqrstuvwxyz0123456789";
    const ALPHABET_LEN: u64 = 36;

    let mut state = seed_random_state();
    let mut output = String::with_capacity(length);

    while output.len() < length {
        state = xorshift64(state);
        let reduced = state % ALPHABET_LEN;
        let Ok(index) = usize::try_from(reduced) else {
            continue;
        };
        output.push(ALPHABET[index] as char);
    }

    output
}

fn seed_random_state() -> u64 {
    let now = match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(duration) => duration.as_secs() ^ u64::from(duration.subsec_nanos()).rotate_left(32),
        Err(_) => 0,
    };
    let mixed = now ^ 0x9E37_79B9_7F4A_7C15;
    if mixed == 0 {
        0xA5A5_A5A5_A5A5_A5A5
    } else {
        mixed
    }
}

fn xorshift64(mut state: u64) -> u64 {
    state ^= state << 13;
    state ^= state >> 7;
    state ^= state << 17;
    state
}
