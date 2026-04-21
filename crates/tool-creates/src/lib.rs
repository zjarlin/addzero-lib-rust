#![forbid(unsafe_code)]

use reqwest::blocking::{Client, Response};
use reqwest::header::{
    ACCEPT, AUTHORIZATION, CONTENT_TYPE, HeaderMap, HeaderName, HeaderValue, InvalidHeaderName,
    InvalidHeaderValue,
};
use reqwest::{Method, Url};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::collections::BTreeMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use thiserror::Error;

pub type CreatesResult<T> = Result<T, CreatesError>;

#[derive(Debug, Error)]
pub enum CreatesError {
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
pub struct ApiConfig {
    pub base_url: String,
    pub connect_timeout: Duration,
    pub request_timeout: Duration,
    pub user_agent: Option<String>,
    pub default_headers: BTreeMap<String, String>,
}

impl ApiConfig {
    pub fn builder(base_url: impl Into<String>) -> ApiConfigBuilder {
        ApiConfigBuilder {
            base_url: base_url.into(),
            connect_timeout: Duration::from_secs(10),
            request_timeout: Duration::from_secs(20),
            user_agent: Some(default_user_agent()),
            default_headers: BTreeMap::new(),
        }
    }

    pub fn validate(&self) -> CreatesResult<()> {
        if self.base_url.trim().is_empty() {
            return Err(CreatesError::InvalidConfig(
                "base_url cannot be blank".to_owned(),
            ));
        }
        if self.connect_timeout.is_zero() {
            return Err(CreatesError::InvalidConfig(
                "connect_timeout cannot be zero".to_owned(),
            ));
        }
        if self.request_timeout.is_zero() {
            return Err(CreatesError::InvalidConfig(
                "request_timeout cannot be zero".to_owned(),
            ));
        }
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct ApiConfigBuilder {
    base_url: String,
    connect_timeout: Duration,
    request_timeout: Duration,
    user_agent: Option<String>,
    default_headers: BTreeMap<String, String>,
}

impl ApiConfigBuilder {
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

    pub fn build(self) -> CreatesResult<ApiConfig> {
        let config = ApiConfig {
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

#[derive(Debug, Clone, Copy, Default)]
pub struct Creates;

impl Creates {
    pub fn maven_central() -> CreatesResult<MavenCentralApi> {
        create_maven_central_api()
    }

    pub fn maven_central_with_config(config: ApiConfig) -> CreatesResult<MavenCentralApi> {
        MavenCentralApi::new(config)
    }

    pub fn temp_mail() -> CreatesResult<TempMailApi> {
        create_temp_mail_api()
    }

    pub fn temp_mail_with_config(config: ApiConfig) -> CreatesResult<TempMailApi> {
        TempMailApi::new(config)
    }
}

pub fn create_maven_central_api() -> CreatesResult<MavenCentralApi> {
    let config = ApiConfig::builder("https://search.maven.org").build()?;
    MavenCentralApi::new(config)
}

pub fn create_temp_mail_api() -> CreatesResult<TempMailApi> {
    let config = ApiConfig::builder("https://api.mail.tm")
        .default_header(ACCEPT.as_str(), "application/json")
        .build()?;
    TempMailApi::new(config)
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MavenArtifact {
    pub id: String,
    pub group_id: String,
    pub artifact_id: String,
    pub latest_version: Option<String>,
    pub version: Option<String>,
    pub packaging: Option<String>,
    pub timestamp: Option<i64>,
}

impl MavenArtifact {
    pub fn resolved_version(&self) -> Option<&str> {
        self.version.as_deref().or(self.latest_version.as_deref())
    }
}

#[derive(Debug, Clone)]
pub struct MavenCentralApi {
    http: HttpApiClient,
}

impl MavenCentralApi {
    pub fn new(config: ApiConfig) -> CreatesResult<Self> {
        Ok(Self {
            http: HttpApiClient::new(config)?,
        })
    }

    pub fn search_by_group_id(
        &self,
        group_id: impl AsRef<str>,
        rows: usize,
    ) -> CreatesResult<Vec<MavenArtifact>> {
        self.search(format!("g:{}", group_id.as_ref().trim()), rows, None)
    }

    pub fn search_by_artifact_id(
        &self,
        artifact_id: impl AsRef<str>,
        rows: usize,
    ) -> CreatesResult<Vec<MavenArtifact>> {
        self.search(format!("a:{}", artifact_id.as_ref().trim()), rows, None)
    }

    pub fn search_by_coordinates(
        &self,
        group_id: impl AsRef<str>,
        artifact_id: impl AsRef<str>,
        rows: usize,
    ) -> CreatesResult<Vec<MavenArtifact>> {
        let query = format!(
            "g:{} AND a:{}",
            group_id.as_ref().trim(),
            artifact_id.as_ref().trim()
        );
        self.search(query, rows, None)
    }

    pub fn search_all_versions(
        &self,
        group_id: impl AsRef<str>,
        artifact_id: impl AsRef<str>,
        rows: usize,
    ) -> CreatesResult<Vec<MavenArtifact>> {
        let query = format!(
            "g:{} AND a:{}",
            group_id.as_ref().trim(),
            artifact_id.as_ref().trim()
        );
        self.search(query, rows, Some("gav"))
    }

    pub fn search_by_full_coordinates(
        &self,
        group_id: impl AsRef<str>,
        artifact_id: impl AsRef<str>,
        version: Option<&str>,
        packaging: Option<&str>,
        classifier: Option<&str>,
        rows: usize,
    ) -> CreatesResult<Vec<MavenArtifact>> {
        let mut conditions = vec![
            format!("g:{}", group_id.as_ref().trim()),
            format!("a:{}", artifact_id.as_ref().trim()),
        ];

        if let Some(value) = non_blank(version) {
            conditions.push(format!("v:{value}"));
        }
        if let Some(value) = non_blank(packaging) {
            conditions.push(format!("p:{value}"));
        }
        if let Some(value) = non_blank(classifier) {
            conditions.push(format!("l:{value}"));
        }

        self.search(conditions.join(" AND "), rows, None)
    }

    pub fn search_by_class_name(
        &self,
        class_name: impl AsRef<str>,
        rows: usize,
    ) -> CreatesResult<Vec<MavenArtifact>> {
        self.search(format!("c:{}", class_name.as_ref().trim()), rows, None)
    }

    pub fn search_by_fully_qualified_class_name(
        &self,
        class_name: impl AsRef<str>,
        rows: usize,
    ) -> CreatesResult<Vec<MavenArtifact>> {
        self.search(format!("fc:{}", class_name.as_ref().trim()), rows, None)
    }

    pub fn search_by_sha1(
        &self,
        sha1: impl AsRef<str>,
        rows: usize,
    ) -> CreatesResult<Vec<MavenArtifact>> {
        self.search(format!("1:{}", sha1.as_ref().trim()), rows, None)
    }

    pub fn search_by_tag(
        &self,
        tag: impl AsRef<str>,
        rows: usize,
    ) -> CreatesResult<Vec<MavenArtifact>> {
        self.search(format!("tags:{}", tag.as_ref().trim()), rows, None)
    }

    pub fn search_by_keyword(
        &self,
        keyword: impl AsRef<str>,
        rows: usize,
    ) -> CreatesResult<Vec<MavenArtifact>> {
        self.search(keyword.as_ref().trim().to_owned(), rows, None)
    }

    pub fn get_latest_version(
        &self,
        group_id: impl AsRef<str>,
        artifact_id: impl AsRef<str>,
    ) -> CreatesResult<Option<String>> {
        let artifacts = self.search_by_coordinates(group_id, artifact_id, 1)?;
        Ok(artifacts
            .first()
            .and_then(|artifact| artifact.latest_version.clone().or(artifact.version.clone())))
    }

    pub fn get_latest_version_by_group_id(
        &self,
        group_id: impl AsRef<str>,
        rows: usize,
    ) -> CreatesResult<Option<String>> {
        let artifacts = self.search_by_group_id(group_id, rows)?;
        Ok(artifacts
            .first()
            .and_then(|artifact| artifact.latest_version.clone().or(artifact.version.clone())))
    }

    pub fn download_file(
        &self,
        group_id: impl AsRef<str>,
        artifact_id: impl AsRef<str>,
        version: impl AsRef<str>,
        filename: impl AsRef<str>,
    ) -> CreatesResult<Vec<u8>> {
        let filepath = format!(
            "{}/{}/{}/{}",
            group_id.as_ref().replace('.', "/"),
            artifact_id.as_ref().trim(),
            version.as_ref().trim(),
            filename.as_ref().trim()
        );

        self.http
            .get_bytes("/remotecontent", &[("filepath", filepath)], None)
    }

    fn search(
        &self,
        query: String,
        rows: usize,
        core: Option<&str>,
    ) -> CreatesResult<Vec<MavenArtifact>> {
        let mut params = vec![
            ("q", query),
            ("rows", rows.max(1).to_string()),
            ("wt", "json".to_owned()),
        ];

        if let Some(value) = core {
            params.push(("core", value.to_owned()));
        }

        let response: MavenSearchResponseEnvelope =
            self.http.get_json("/solrsearch/select", &params, None)?;

        Ok(response
            .response
            .docs
            .into_iter()
            .map(MavenArtifact::from)
            .collect())
    }
}

#[derive(Debug, Clone)]
pub struct TempMailApi {
    http: HttpApiClient,
}

impl TempMailApi {
    pub fn new(config: ApiConfig) -> CreatesResult<Self> {
        Ok(Self {
            http: HttpApiClient::new(config)?,
        })
    }

    pub fn get_domains(&self) -> CreatesResult<Vec<TempMailDomain>> {
        let response: HydraCollection<TempMailDomain> =
            self.http.get_json("/domains", &[], None)?;
        Ok(response.items)
    }

    pub fn create_mailbox_and_login(
        &self,
        prefix: impl AsRef<str>,
        password_length: usize,
    ) -> CreatesResult<TempMailMailbox> {
        let domains = self
            .get_domains()?
            .into_iter()
            .filter(|domain| domain.is_active)
            .collect::<Vec<_>>();

        let chosen_domain = domains
            .first()
            .map(|domain| domain.domain.clone())
            .ok_or_else(|| {
                CreatesError::InvalidResponse("no active temp-mail domains available".to_owned())
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
    ) -> CreatesResult<String> {
        let response: TempMailAccountResponse = self.http.post_json(
            "/accounts",
            &json!({
                "address": address.as_ref().trim(),
                "password": password.as_ref(),
            }),
            None,
        )?;

        non_blank(Some(response.id.as_str()))
            .map(ToOwned::to_owned)
            .ok_or_else(|| {
                CreatesError::InvalidResponse(format!(
                    "create account failed: id missing for address={}",
                    address.as_ref().trim()
                ))
            })
    }

    pub fn create_token(
        &self,
        address: impl AsRef<str>,
        password: impl AsRef<str>,
    ) -> CreatesResult<String> {
        let response: TempMailTokenResponse = self.http.post_json(
            "/token",
            &json!({
                "address": address.as_ref().trim(),
                "password": password.as_ref(),
            }),
            None,
        )?;

        non_blank(Some(response.token.as_str()))
            .map(ToOwned::to_owned)
            .ok_or_else(|| {
                CreatesError::InvalidResponse(format!(
                    "create token failed: token missing for address={}",
                    address.as_ref().trim()
                ))
            })
    }

    pub fn list_messages(
        &self,
        token: impl AsRef<str>,
        page: usize,
    ) -> CreatesResult<Vec<TempMailMessageSummary>> {
        let response: HydraCollection<TempMailMessageSummaryRaw> = self.http.get_json(
            "/messages",
            &[("page", page.max(1).to_string())],
            Some(token.as_ref()),
        )?;

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
    ) -> CreatesResult<TempMailMessageDetail> {
        let path = format!("/messages/{}", message_id.as_ref().trim());
        let response: TempMailMessageDetailRaw =
            self.http.get_json(&path, &[], Some(token.as_ref()))?;

        TempMailMessageDetail::try_from_raw(response)
    }
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
    fn try_from_raw(raw: TempMailMessageDetailRaw) -> CreatesResult<Self> {
        let html = match raw.html {
            Value::String(value) => value,
            Value::Array(values) => values
                .into_iter()
                .find_map(|item| item.as_str().map(ToOwned::to_owned))
                .map_or_else(String::new, |value| value),
            Value::Null => String::new(),
            other => {
                return Err(CreatesError::InvalidResponse(format!(
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

#[derive(Debug, Clone)]
struct HttpApiClient {
    base_url: Url,
    client: Client,
}

impl HttpApiClient {
    fn new(config: ApiConfig) -> CreatesResult<Self> {
        config.validate()?;
        let base_url = Url::parse(&config.base_url)
            .map_err(|_| CreatesError::InvalidBaseUrl(config.base_url))?;
        let default_headers = build_default_headers(&config.default_headers)?;

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

    fn get_json<T: DeserializeOwned>(
        &self,
        path: &str,
        query: &[(&str, String)],
        bearer_token: Option<&str>,
    ) -> CreatesResult<T> {
        let url = self.join_url(path)?;
        let mut builder = self.client.request(Method::GET, url.clone());
        if !query.is_empty() {
            builder = builder.query(query);
        }
        if let Some(token) = non_blank(bearer_token) {
            builder = builder.header(AUTHORIZATION, format!("Bearer {token}"));
        }
        let response = builder.send()?;
        self.read_json(url, response)
    }

    fn post_json<T: DeserializeOwned, B: Serialize>(
        &self,
        path: &str,
        body: &B,
        bearer_token: Option<&str>,
    ) -> CreatesResult<T> {
        let url = self.join_url(path)?;
        let body_bytes = serde_json::to_vec(body)?;
        let mut builder = self
            .client
            .request(Method::POST, url.clone())
            .header(CONTENT_TYPE, "application/json")
            .body(body_bytes);
        if let Some(token) = non_blank(bearer_token) {
            builder = builder.header(AUTHORIZATION, format!("Bearer {token}"));
        }
        let response = builder.send()?;
        self.read_json(url, response)
    }

    fn get_bytes(
        &self,
        path: &str,
        query: &[(&str, String)],
        bearer_token: Option<&str>,
    ) -> CreatesResult<Vec<u8>> {
        let url = self.join_url(path)?;
        let mut builder = self.client.request(Method::GET, url.clone());
        if !query.is_empty() {
            builder = builder.query(query);
        }
        if let Some(token) = non_blank(bearer_token) {
            builder = builder.header(AUTHORIZATION, format!("Bearer {token}"));
        }
        let response = builder.send()?;
        let response = self.ensure_success(&url, response)?;
        Ok(response.bytes()?.to_vec())
    }

    fn read_json<T: DeserializeOwned>(&self, url: Url, response: Response) -> CreatesResult<T> {
        let response = self.ensure_success(&url, response)?;
        let bytes = response.bytes()?;
        Ok(serde_json::from_slice(bytes.as_ref())?)
    }

    fn ensure_success(&self, url: &Url, response: Response) -> CreatesResult<Response> {
        let status = response.status();
        if status.is_success() {
            return Ok(response);
        }

        let body = match response.bytes() {
            Ok(bytes) => String::from_utf8_lossy(bytes.as_ref()).into_owned(),
            Err(error) => {
                return Err(CreatesError::Transport(error));
            }
        };

        Err(CreatesError::HttpStatus {
            url: url.to_string(),
            status: status.as_u16(),
            body,
        })
    }

    fn join_url(&self, path: &str) -> CreatesResult<Url> {
        self.base_url
            .join(path)
            .map_err(|_| CreatesError::InvalidPath(path.to_owned()))
    }
}

#[derive(Debug, Deserialize)]
struct MavenSearchResponseEnvelope {
    response: MavenSearchResponse,
}

#[derive(Debug, Deserialize)]
struct MavenSearchResponse {
    #[serde(default)]
    docs: Vec<MavenSearchDocument>,
}

#[derive(Debug, Deserialize)]
struct MavenSearchDocument {
    #[serde(default)]
    id: String,
    #[serde(rename = "g", default)]
    group_id: String,
    #[serde(rename = "a", default)]
    artifact_id: String,
    #[serde(rename = "latestVersion", default)]
    latest_version: Option<String>,
    #[serde(rename = "v", default)]
    version: Option<String>,
    #[serde(rename = "p", default)]
    packaging: Option<String>,
    #[serde(default)]
    timestamp: Option<i64>,
}

impl From<MavenSearchDocument> for MavenArtifact {
    fn from(value: MavenSearchDocument) -> Self {
        Self {
            id: value.id,
            group_id: value.group_id,
            artifact_id: value.artifact_id,
            latest_version: value.latest_version,
            version: value.version,
            packaging: value.packaging,
            timestamp: value.timestamp,
        }
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

fn build_default_headers(headers: &BTreeMap<String, String>) -> CreatesResult<HeaderMap> {
    let mut header_map = HeaderMap::new();
    for (name, value) in headers {
        let header_name = HeaderName::from_bytes(name.as_bytes()).map_err(|source| {
            CreatesError::InvalidHeaderName {
                name: name.clone(),
                source,
            }
        })?;
        let header_value =
            HeaderValue::from_str(value).map_err(|source| CreatesError::InvalidHeaderValue {
                name: name.clone(),
                source,
            })?;
        header_map.insert(header_name, header_value);
    }
    Ok(header_map)
}

fn non_blank(value: Option<&str>) -> Option<&str> {
    value.and_then(|item| {
        let trimmed = item.trim();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed)
        }
    })
}

fn default_user_agent() -> String {
    format!("addzero-creates/{}", env!("CARGO_PKG_VERSION"))
}

fn sanitize_prefix(prefix: &str) -> String {
    let sanitized = prefix
        .chars()
        .filter(|character| character.is_ascii_alphanumeric())
        .collect::<String>();

    if sanitized.is_empty() {
        "az".to_owned()
    } else {
        sanitized
    }
}

fn random_alpha_numeric(length: usize) -> String {
    const ALPHABET: &[u8] = b"abcdefghijklmnopqrstuvwxyz0123456789";
    static COUNTER: AtomicU64 = AtomicU64::new(0);

    let mut state = seed_random_state(COUNTER.fetch_add(1, Ordering::Relaxed));
    let mut output = String::with_capacity(length);

    while output.len() < length {
        state = xorshift64(state);
        let index = (state as usize) % ALPHABET.len();
        output.push(ALPHABET[index] as char);
    }

    output
}

fn seed_random_state(counter: u64) -> u64 {
    let now = match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(duration) => duration.as_nanos() as u64,
        Err(_) => 0,
    };
    let mixed = now ^ counter.rotate_left(19) ^ 0x9E37_79B9_7F4A_7C15;
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
