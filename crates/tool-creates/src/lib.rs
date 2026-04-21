#![forbid(unsafe_code)]

use chrono::Utc;
use hmac::{Hmac, Mac};
use reqwest::blocking::{Client, Response};
use reqwest::header::{
    ACCEPT, AUTHORIZATION, CONTENT_TYPE, HOST, HeaderMap, HeaderName, HeaderValue,
    InvalidHeaderName, InvalidHeaderValue,
};
use reqwest::{Method, Url};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::thread;
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
    #[error("signature error: {0}")]
    Signature(String),
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

    pub fn music_search() -> CreatesResult<MusicSearchApi> {
        create_music_search_api()
    }

    pub fn music_search_with_config(config: ApiConfig) -> CreatesResult<MusicSearchApi> {
        MusicSearchApi::new(config)
    }

    pub fn suno(api_token: impl Into<String>) -> CreatesResult<SunoApi> {
        create_suno_api(api_token)
    }

    pub fn suno_with_config(
        api_token: impl Into<String>,
        config: ApiConfig,
    ) -> CreatesResult<SunoApi> {
        SunoApi::new(api_token, config)
    }

    pub fn tianyancha(
        authorization: impl Into<String>,
        auth_token: impl Into<String>,
    ) -> CreatesResult<TianyanchaApi> {
        create_tianyancha_api(authorization, auth_token)
    }

    pub fn tianyancha_with_config(
        authorization: impl Into<String>,
        auth_token: impl Into<String>,
        config: ApiConfig,
    ) -> CreatesResult<TianyanchaApi> {
        TianyanchaApi::new(authorization, auth_token, config)
    }

    pub fn tianyancha_huawei(
        access_key: impl Into<String>,
        secret_key: impl Into<String>,
    ) -> CreatesResult<TianyanchaHuaweiApi> {
        create_tianyancha_huawei_api(access_key, secret_key)
    }

    pub fn tianyancha_huawei_with_config(
        access_key: impl Into<String>,
        secret_key: impl Into<String>,
        config: ApiConfig,
    ) -> CreatesResult<TianyanchaHuaweiApi> {
        TianyanchaHuaweiApi::new(access_key, secret_key, config)
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

pub fn create_music_search_api() -> CreatesResult<MusicSearchApi> {
    let config = ApiConfig::builder("https://music.163.com/api/")
        .default_header(ACCEPT.as_str(), "application/json")
        .default_header("Referer", "https://music.163.com/")
        .user_agent(
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 \
             (KHTML, like Gecko) Chrome/124.0.0.0 Safari/537.36",
        )
        .build()?;
    MusicSearchApi::new(config)
}

pub fn create_suno_api(api_token: impl Into<String>) -> CreatesResult<SunoApi> {
    let config = ApiConfig::builder("https://api.vectorengine.ai")
        .default_header(ACCEPT.as_str(), "application/json")
        .build()?;
    SunoApi::new(api_token, config)
}

pub fn create_tianyancha_api(
    authorization: impl Into<String>,
    auth_token: impl Into<String>,
) -> CreatesResult<TianyanchaApi> {
    let config = ApiConfig::builder("https://api9.tianyancha.com")
        .default_header(CONTENT_TYPE.as_str(), "application/json")
        .default_header(HOST.as_str(), "api9.tianyancha.com")
        .default_header(ACCEPT.as_str(), "*/*")
        .default_header("version", "TYC-XCX-WX")
        .default_header(
            "User-Agent",
            "Mozilla/5.0 (iPhone; CPU iPhone OS 12_1_4 like Mac OS X) \
             AppleWebKit/605.1.15 (KHTML, like Gecko) Mobile/16D57 \
             MicroMessenger/7.0.5(0x17000523) NetType/WIFI Language/zh_CN",
        )
        .default_header("Accept-Language", "zh-cn")
        .build()?;
    TianyanchaApi::new(authorization, auth_token, config)
}

pub fn create_tianyancha_huawei_api(
    access_key: impl Into<String>,
    secret_key: impl Into<String>,
) -> CreatesResult<TianyanchaHuaweiApi> {
    let config = ApiConfig::builder("http://kzenterprisewmh.apistore.huaweicloud.com")
        .default_header(ACCEPT.as_str(), "application/json")
        .build()?;
    TianyanchaHuaweiApi::new(access_key, secret_key, config)
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MusicSearchType {
    Song,
    Album,
    Artist,
    Playlist,
    User,
    Mv,
    Lyric,
    Radio,
    Video,
}

impl Default for MusicSearchType {
    fn default() -> Self {
        Self::Song
    }
}

impl MusicSearchType {
    pub const fn value(self) -> u16 {
        match self {
            Self::Song => 1,
            Self::Album => 10,
            Self::Artist => 100,
            Self::Playlist => 1000,
            Self::User => 1002,
            Self::Mv => 1004,
            Self::Lyric => 1006,
            Self::Radio => 1009,
            Self::Video => 1014,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MusicSearchRequest {
    pub keywords: String,
    pub search_type: MusicSearchType,
    pub limit: usize,
    pub offset: usize,
}

impl MusicSearchRequest {
    pub fn new(keywords: impl Into<String>) -> Self {
        Self {
            keywords: keywords.into(),
            search_type: MusicSearchType::Song,
            limit: 30,
            offset: 0,
        }
    }

    pub fn search_type(mut self, value: MusicSearchType) -> Self {
        self.search_type = value;
        self
    }

    pub fn limit(mut self, value: usize) -> Self {
        self.limit = value;
        self
    }

    pub fn offset(mut self, value: usize) -> Self {
        self.offset = value;
        self
    }
}

#[derive(Debug, Clone)]
pub struct MusicSearchApi {
    http: HttpApiClient,
}

pub type NeteaseMusicApi = MusicSearchApi;

impl MusicSearchApi {
    pub fn new(config: ApiConfig) -> CreatesResult<Self> {
        Ok(Self {
            http: HttpApiClient::new(config)?,
        })
    }

    pub fn search(&self, request: MusicSearchRequest) -> CreatesResult<MusicSearchResult> {
        let keywords = trim_non_blank(Some(request.keywords.as_str())).ok_or_else(|| {
            CreatesError::InvalidConfig("music keywords cannot be blank".to_owned())
        })?;
        let response: MusicSearchResponse = self.http.get_json(
            "search/get/web",
            &[
                ("s", keywords.to_owned()),
                ("type", request.search_type.value().to_string()),
                ("limit", request.limit.max(1).to_string()),
                ("offset", request.offset.to_string()),
            ],
            None,
        )?;
        ensure_code_200(response.code, response.msg.as_deref(), "music search")?;
        Ok(response.result.unwrap_or_default())
    }

    pub fn search_songs(
        &self,
        keywords: impl Into<String>,
        limit: usize,
        offset: usize,
    ) -> CreatesResult<Vec<MusicSong>> {
        Ok(self
            .search(
                MusicSearchRequest::new(keywords)
                    .search_type(MusicSearchType::Song)
                    .limit(limit)
                    .offset(offset),
            )?
            .songs)
    }

    pub fn search_artists(
        &self,
        keywords: impl Into<String>,
        limit: usize,
        offset: usize,
    ) -> CreatesResult<Vec<MusicArtist>> {
        Ok(self
            .search(
                MusicSearchRequest::new(keywords)
                    .search_type(MusicSearchType::Artist)
                    .limit(limit)
                    .offset(offset),
            )?
            .artists)
    }

    pub fn search_albums(
        &self,
        keywords: impl Into<String>,
        limit: usize,
        offset: usize,
    ) -> CreatesResult<Vec<MusicAlbum>> {
        Ok(self
            .search(
                MusicSearchRequest::new(keywords)
                    .search_type(MusicSearchType::Album)
                    .limit(limit)
                    .offset(offset),
            )?
            .albums)
    }

    pub fn search_playlists(
        &self,
        keywords: impl Into<String>,
        limit: usize,
        offset: usize,
    ) -> CreatesResult<Vec<MusicPlaylist>> {
        Ok(self
            .search(
                MusicSearchRequest::new(keywords)
                    .search_type(MusicSearchType::Playlist)
                    .limit(limit)
                    .offset(offset),
            )?
            .playlists)
    }

    pub fn get_lyric(&self, song_id: i64) -> CreatesResult<LyricResponse> {
        let response: LyricResponse = self.http.get_json(
            "song/lyric",
            &[
                ("id", song_id.to_string()),
                ("lv", "1".to_owned()),
                ("tv", "1".to_owned()),
            ],
            None,
        )?;
        ensure_code_200(response.code, None, "get lyric")?;
        Ok(response)
    }

    pub fn get_song_detail(&self, song_ids: &[i64]) -> CreatesResult<Vec<MusicSong>> {
        if song_ids.is_empty() {
            return Ok(Vec::new());
        }
        let ids = song_ids
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join(",");
        let response: SongDetailResponse =
            self.http
                .get_json("song/detail", &[("ids", format!("[{ids}]"))], None)?;
        ensure_code_200(response.code, None, "get song detail")?;
        Ok(response.songs)
    }

    pub fn search_by_song_and_artist(
        &self,
        song_name: impl AsRef<str>,
        artist_name: Option<&str>,
    ) -> CreatesResult<Vec<MusicSong>> {
        let keywords = match trim_non_blank(artist_name) {
            Some(artist) => format!("{} {artist}", song_name.as_ref().trim()),
            None => song_name.as_ref().trim().to_owned(),
        };
        let songs = self.search_songs(keywords, 10, 0)?;
        let Some(artist_name) = trim_non_blank(artist_name) else {
            return Ok(songs);
        };
        let needle = artist_name.to_lowercase();
        Ok(songs
            .into_iter()
            .filter(|song| {
                song.artists
                    .iter()
                    .any(|artist| artist.name.to_lowercase().contains(&needle))
            })
            .collect())
    }

    pub fn search_by_lyric(
        &self,
        lyric_fragment: impl Into<String>,
    ) -> CreatesResult<Vec<MusicSong>> {
        Ok(self
            .search(
                MusicSearchRequest::new(lyric_fragment)
                    .search_type(MusicSearchType::Lyric)
                    .limit(20),
            )?
            .songs)
    }

    pub fn get_lyric_by_song_name(
        &self,
        song_name: impl AsRef<str>,
        artist_name: Option<&str>,
    ) -> CreatesResult<Option<LyricResponse>> {
        let songs = self.search_by_song_and_artist(song_name, artist_name)?;
        if let Some(song) = songs.first() {
            return self.get_lyric(song.id).map(Some);
        }
        Ok(None)
    }

    pub fn get_lyrics_by_fragment(
        &self,
        lyric_fragment: impl Into<String>,
        limit: usize,
        filter_empty: bool,
    ) -> CreatesResult<Vec<SongWithLyric>> {
        let songs = self.search_by_lyric(lyric_fragment)?;
        let mut items = Vec::new();
        for song in songs.into_iter().take(limit.max(1)) {
            let lyric = match self.get_lyric(song.id) {
                Ok(value) => value,
                Err(_) => continue,
            };
            if filter_empty
                && lyric
                    .lrc
                    .as_ref()
                    .and_then(|item| trim_non_blank(item.lyric.as_deref()))
                    .is_none()
            {
                continue;
            }
            items.push(SongWithLyric { song, lyric });
        }
        Ok(items)
    }
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct MusicSearchResponse {
    #[serde(default)]
    pub code: i32,
    #[serde(default)]
    pub msg: Option<String>,
    #[serde(default)]
    pub result: Option<MusicSearchResult>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct MusicSearchResult {
    #[serde(default)]
    pub songs: Vec<MusicSong>,
    #[serde(rename = "songCount", default)]
    pub song_count: Option<i64>,
    #[serde(default)]
    pub albums: Vec<MusicAlbum>,
    #[serde(rename = "albumCount", default)]
    pub album_count: Option<i64>,
    #[serde(default)]
    pub artists: Vec<MusicArtist>,
    #[serde(rename = "artistCount", default)]
    pub artist_count: Option<i64>,
    #[serde(default)]
    pub playlists: Vec<MusicPlaylist>,
    #[serde(rename = "playlistCount", default)]
    pub playlist_count: Option<i64>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct MusicSong {
    #[serde(default)]
    pub id: i64,
    #[serde(default)]
    pub name: String,
    #[serde(default, alias = "ar")]
    pub artists: Vec<MusicArtist>,
    #[serde(default, alias = "al")]
    pub album: Option<MusicAlbum>,
    #[serde(default, alias = "dt")]
    pub duration: Option<i64>,
    #[serde(rename = "mvid", default)]
    pub mv_id: Option<i64>,
    #[serde(default)]
    pub fee: Option<i32>,
    #[serde(default)]
    pub privilege: Option<MusicPrivilege>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct MusicArtist {
    #[serde(default)]
    pub id: i64,
    #[serde(default)]
    pub name: String,
    #[serde(rename = "picUrl", default)]
    pub pic_url: Option<String>,
    #[serde(default)]
    pub alias: Vec<String>,
    #[serde(rename = "albumSize", default)]
    pub album_size: Option<i32>,
    #[serde(rename = "musicSize", default)]
    pub music_size: Option<i32>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct MusicAlbum {
    #[serde(default)]
    pub id: i64,
    #[serde(default)]
    pub name: String,
    #[serde(rename = "picUrl", default)]
    pub pic_url: Option<String>,
    #[serde(default)]
    pub artist: Option<MusicArtist>,
    #[serde(rename = "publishTime", default)]
    pub publish_time: Option<i64>,
    #[serde(default)]
    pub size: Option<i32>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct MusicPlaylist {
    #[serde(default)]
    pub id: i64,
    #[serde(default)]
    pub name: String,
    #[serde(rename = "coverImgUrl", default)]
    pub cover_img_url: Option<String>,
    #[serde(default)]
    pub creator: Option<MusicCreator>,
    #[serde(rename = "trackCount", default)]
    pub track_count: Option<i32>,
    #[serde(rename = "playCount", default)]
    pub play_count: Option<i64>,
    #[serde(default)]
    pub description: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct MusicCreator {
    #[serde(rename = "userId", default)]
    pub user_id: i64,
    #[serde(default)]
    pub nickname: String,
    #[serde(rename = "avatarUrl", default)]
    pub avatar_url: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct MusicPrivilege {
    #[serde(default)]
    pub id: Option<i64>,
    #[serde(default)]
    pub fee: Option<i32>,
    #[serde(default)]
    pub st: Option<i32>,
    #[serde(default)]
    pub pl: Option<i32>,
    #[serde(default)]
    pub dl: Option<i32>,
    #[serde(default)]
    pub maxbr: Option<i32>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct LyricResponse {
    #[serde(default)]
    pub code: i32,
    #[serde(default)]
    pub lrc: Option<LyricContent>,
    #[serde(default)]
    pub tlyric: Option<LyricContent>,
    #[serde(default)]
    pub romalrc: Option<LyricContent>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct LyricContent {
    #[serde(default)]
    pub version: Option<i32>,
    #[serde(default)]
    pub lyric: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct SongDetailResponse {
    #[serde(default)]
    pub code: i32,
    #[serde(default)]
    pub songs: Vec<MusicSong>,
    #[serde(default)]
    pub privileges: Vec<MusicPrivilege>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SongWithLyric {
    pub song: MusicSong,
    pub lyric: LyricResponse,
}

#[derive(Debug, Clone)]
pub struct SunoApi {
    api_token: String,
    http: HttpApiClient,
}

impl SunoApi {
    pub fn new(api_token: impl Into<String>, config: ApiConfig) -> CreatesResult<Self> {
        let api_token = api_token.into();
        if trim_non_blank(Some(api_token.as_str())).is_none() {
            return Err(CreatesError::InvalidConfig(
                "suno api_token cannot be blank".to_owned(),
            ));
        }
        Ok(Self {
            api_token,
            http: HttpApiClient::new(config)?,
        })
    }

    pub fn generate_music(&self, request: &SunoMusicRequest) -> CreatesResult<String> {
        let response: ApiEnvelope<String> =
            self.http
                .post_json("suno/submit/music", request, Some(self.api_token.as_str()))?;
        response.into_data("generate suno music")
    }

    pub fn generate_lyrics(&self, prompt: impl AsRef<str>) -> CreatesResult<String> {
        let response: ApiEnvelope<String> = self.http.post_json(
            "suno/lyrics",
            &GenerateLyricsRequest {
                prompt: prompt.as_ref().trim().to_owned(),
            },
            Some(self.api_token.as_str()),
        )?;
        response.into_data("generate suno lyrics")
    }

    pub fn concat_songs(&self, clip_id: impl AsRef<str>) -> CreatesResult<String> {
        let response: ApiEnvelope<String> = self.http.post_json(
            "suno/concat",
            &ConcatSongsRequest {
                clip_id: clip_id.as_ref().trim().to_owned(),
            },
            Some(self.api_token.as_str()),
        )?;
        response.into_data("concat suno songs")
    }

    pub fn fetch_task(&self, task_id: impl AsRef<str>) -> CreatesResult<Option<SunoTask>> {
        let path = format!("suno/fetch/{}", task_id.as_ref().trim());
        let response: ApiEnvelope<SunoTask> =
            self.http
                .get_json(&path, &[], Some(self.api_token.as_str()))?;
        response.into_optional_data("fetch suno task")
    }

    pub fn batch_fetch_tasks<I, S>(&self, task_ids: I) -> CreatesResult<Vec<SunoTask>>
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        let ids = task_ids.into_iter().map(Into::into).collect::<Vec<_>>();
        let response: ApiEnvelope<Vec<SunoTask>> = self.http.post_json(
            "suno/fetch",
            &BatchFetchRequest { ids },
            Some(self.api_token.as_str()),
        )?;
        Ok(response
            .into_optional_data("batch fetch suno task")?
            .unwrap_or_default())
    }

    pub fn wait_for_completion(&self, task_id: impl AsRef<str>) -> CreatesResult<SunoTask> {
        self.wait_for_completion_with(
            task_id,
            Duration::from_secs(600),
            Duration::from_secs(10),
            |_| {},
        )
    }

    pub fn wait_for_completion_with<F>(
        &self,
        task_id: impl AsRef<str>,
        max_wait: Duration,
        poll_interval: Duration,
        mut on_status_update: F,
    ) -> CreatesResult<SunoTask>
    where
        F: FnMut(Option<&str>),
    {
        let task_id = task_id.as_ref().trim().to_owned();
        let started = SystemTime::now();

        loop {
            let task = self.fetch_task(task_id.as_str())?;
            on_status_update(task.as_ref().and_then(|item| item.status.as_deref()));

            match task {
                Some(task) if matches!(task.status.as_deref(), Some("complete" | "streaming")) => {
                    return Ok(task);
                }
                Some(task) if matches!(task.status.as_deref(), Some("error")) => {
                    return Err(CreatesError::InvalidResponse(format!(
                        "suno task failed: {}",
                        task.error
                            .or(task.error_message)
                            .unwrap_or_else(|| "unknown error".to_owned())
                    )));
                }
                _ => {
                    if started.elapsed().unwrap_or_default() >= max_wait {
                        return Err(CreatesError::InvalidResponse(format!(
                            "suno task `{task_id}` timed out after {:?}",
                            max_wait
                        )));
                    }
                    thread::sleep(poll_interval);
                }
            }
        }
    }

    pub fn wait_for_batch_completion<I, S>(&self, task_ids: I) -> CreatesResult<Vec<SunoTask>>
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.wait_for_batch_completion_with(
            task_ids,
            Duration::from_secs(600),
            Duration::from_secs(10),
        )
    }

    pub fn wait_for_batch_completion_with<I, S>(
        &self,
        task_ids: I,
        max_wait: Duration,
        poll_interval: Duration,
    ) -> CreatesResult<Vec<SunoTask>>
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        let task_ids = task_ids.into_iter().map(Into::into).collect::<Vec<_>>();
        let started = SystemTime::now();

        loop {
            let tasks = self.batch_fetch_tasks(task_ids.clone())?;
            if tasks
                .iter()
                .all(|item| matches!(item.status.as_deref(), Some("complete" | "streaming")))
            {
                return Ok(tasks);
            }
            if let Some(task) = tasks
                .iter()
                .find(|item| matches!(item.status.as_deref(), Some("error")))
            {
                return Err(CreatesError::InvalidResponse(format!(
                    "suno task failed: {}",
                    task.error
                        .clone()
                        .or(task.error_message.clone())
                        .unwrap_or_else(|| "unknown error".to_owned())
                )));
            }
            if started.elapsed().unwrap_or_default() >= max_wait {
                return Err(CreatesError::InvalidResponse(format!(
                    "suno tasks timed out after {:?}",
                    max_wait
                )));
            }
            thread::sleep(poll_interval);
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct SunoMusicRequest {
    #[serde(default = "default_suno_mv")]
    pub mv: String,
    #[serde(rename = "gpt_description_prompt", default)]
    pub gpt_description_prompt: Option<String>,
    #[serde(rename = "notify_hook", default)]
    pub notify_hook: Option<String>,
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub tags: Option<String>,
    pub prompt: String,
    #[serde(rename = "make_instrumental", default)]
    pub make_instrumental: Option<bool>,
    #[serde(rename = "task_id", default)]
    pub task_id: Option<String>,
    #[serde(rename = "continue_clip_id", default)]
    pub continue_clip_id: Option<String>,
    #[serde(rename = "continue_at", default)]
    pub continue_at: Option<i32>,
    #[serde(rename = "persona_id", default)]
    pub persona_id: Option<String>,
    #[serde(rename = "artist_clip_id", default)]
    pub artist_clip_id: Option<String>,
    #[serde(rename = "vocal_gender", default)]
    pub vocal_gender: Option<String>,
    #[serde(rename = "generation_type", default)]
    pub generation_type: Option<String>,
    #[serde(rename = "negative_tags", default)]
    pub negative_tags: Option<String>,
    #[serde(rename = "clip_id", default)]
    pub clip_id: Option<String>,
    #[serde(rename = "is_infill", default)]
    pub is_infill: Option<bool>,
    #[serde(default = "default_suno_task")]
    pub task: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct GenerateLyricsRequest {
    pub prompt: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct BatchFetchRequest {
    #[serde(default)]
    pub ids: Vec<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConcatSongsRequest {
    #[serde(rename = "clip_id")]
    pub clip_id: String,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct SunoTask {
    #[serde(default)]
    pub id: Option<String>,
    #[serde(default)]
    pub status: Option<String>,
    #[serde(default)]
    pub prompt: Option<String>,
    #[serde(rename = "gpt_description_prompt", default)]
    pub gpt_description_prompt: Option<String>,
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub tags: Option<String>,
    #[serde(default)]
    pub mv: Option<String>,
    #[serde(default)]
    pub r#type: Option<String>,
    #[serde(default)]
    pub duration: Option<f64>,
    #[serde(rename = "audio_url", default)]
    pub audio_url: Option<String>,
    #[serde(rename = "video_url", default)]
    pub video_url: Option<String>,
    #[serde(rename = "created_at", default)]
    pub created_at: Option<String>,
    #[serde(rename = "error_message", default)]
    pub error_message: Option<String>,
    #[serde(default)]
    pub error: Option<String>,
    #[serde(rename = "clip_id", default)]
    pub clip_id: Option<String>,
    #[serde(rename = "instrumental", default)]
    pub instrumental: Option<bool>,
}

#[derive(Debug, Clone)]
pub struct TianyanchaApi {
    authorization: String,
    auth_token: String,
    http: HttpApiClient,
}

impl TianyanchaApi {
    pub fn new(
        authorization: impl Into<String>,
        auth_token: impl Into<String>,
        config: ApiConfig,
    ) -> CreatesResult<Self> {
        let authorization = authorization.into();
        let auth_token = auth_token.into();
        if trim_non_blank(Some(authorization.as_str())).is_none() {
            return Err(CreatesError::InvalidConfig(
                "tianyancha authorization cannot be blank".to_owned(),
            ));
        }
        if trim_non_blank(Some(auth_token.as_str())).is_none() {
            return Err(CreatesError::InvalidConfig(
                "tianyancha auth_token cannot be blank".to_owned(),
            ));
        }
        Ok(Self {
            authorization,
            auth_token,
            http: HttpApiClient::new(config)?,
        })
    }

    pub fn search_company(
        &self,
        company_name: impl AsRef<str>,
        page_num: usize,
        page_size: usize,
        sort_type: impl AsRef<str>,
    ) -> CreatesResult<TianyanchaCompanySearchData> {
        let company_name = trim_non_blank(Some(company_name.as_ref())).ok_or_else(|| {
            CreatesError::InvalidConfig("company_name cannot be blank".to_owned())
        })?;
        let path = format!(
            "/services/v3/search/sNorV4/{}",
            encode_url_component(company_name)
        );
        let response: TianyanchaSearchResponse = self.http.get_json_with_headers(
            path.as_str(),
            &[
                ("pageNum", page_num.max(1).to_string()),
                ("pageSize", page_size.max(1).to_string()),
                ("sortType", sort_type.as_ref().trim().to_owned()),
            ],
            &self.request_headers(),
        )?;
        response.into_data("search tianyancha company")
    }

    pub fn get_base_info(&self, company_id: i64) -> CreatesResult<TianyanchaCompanyDetail> {
        let path = format!("/services/v3/t/common/baseinfoV5/{company_id}");
        let response: TianyanchaDetailResponse =
            self.http
                .get_json_with_headers(path.as_str(), &[], &self.request_headers())?;
        response.into_data("get tianyancha base info")
    }

    fn request_headers(&self) -> BTreeMap<String, String> {
        BTreeMap::from([
            ("Authorization".to_owned(), self.authorization.clone()),
            ("X-AUTH-TOKEN".to_owned(), self.auth_token.clone()),
        ])
    }
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct TianyanchaCompanySearchData {
    #[serde(rename = "adviceQuery", default)]
    pub advice_query: Option<Value>,
    #[serde(rename = "companyCount", default)]
    pub company_count: Option<i64>,
    #[serde(rename = "companyHumanCount", default)]
    pub company_human_count: Option<i64>,
    #[serde(rename = "companyList", default)]
    pub company_list: Vec<TianyanchaCompany>,
    #[serde(rename = "companyTotal", default)]
    pub company_total: Option<i64>,
    #[serde(rename = "companyTotalPage", default)]
    pub company_total_page: Option<i64>,
    #[serde(rename = "companyTotalStr", default)]
    pub company_total_str: Option<String>,
    #[serde(rename = "humanCount", default)]
    pub human_count: Option<i64>,
    #[serde(rename = "modifiedQuery", default)]
    pub modified_query: Option<Value>,
    #[serde(rename = "searchContent", default)]
    pub search_content: Option<String>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct TianyanchaCompany {
    #[serde(default)]
    pub id: i64,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub alias: Option<String>,
    #[serde(rename = "legalPersonName", default)]
    pub legal_person_name: Option<String>,
    #[serde(rename = "regStatus", default)]
    pub reg_status: Option<String>,
    #[serde(rename = "regCapital", default)]
    pub reg_capital: Option<String>,
    #[serde(rename = "creditCode", default)]
    pub credit_code: Option<String>,
    #[serde(rename = "phoneNum", default)]
    pub phone_num: Option<String>,
    #[serde(rename = "emailList", default)]
    pub email_list: Vec<String>,
    #[serde(rename = "companyOrgType", default)]
    pub company_org_type: Option<String>,
    #[serde(rename = "regLocation", default)]
    pub reg_location: Option<String>,
    #[serde(default)]
    pub logo: Option<String>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct TianyanchaCompanyDetail {
    #[serde(default)]
    pub id: i64,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub alias: Option<String>,
    #[serde(rename = "legalPersonName", default)]
    pub legal_person_name: Option<String>,
    #[serde(rename = "legalPersonId", default)]
    pub legal_person_id: Option<i64>,
    #[serde(rename = "regStatus", default)]
    pub reg_status: Option<String>,
    #[serde(rename = "creditCode", default)]
    pub credit_code: Option<String>,
    #[serde(rename = "companyCreditCode", default)]
    pub company_credit_code: Option<String>,
    #[serde(rename = "regCapital", default)]
    pub reg_capital: Option<String>,
    #[serde(rename = "regNumber", default)]
    pub reg_number: Option<String>,
    #[serde(rename = "companyOrgType", default)]
    pub company_org_type: Option<String>,
    #[serde(rename = "companyProfilePlainText", default)]
    pub company_profile_plain_text: Option<String>,
    #[serde(rename = "businessScope", default)]
    pub business_scope: Option<String>,
    #[serde(rename = "phoneNumber", default)]
    pub phone_number: Option<String>,
    #[serde(rename = "phoneList", default)]
    pub phone_list: Vec<String>,
    #[serde(default)]
    pub email: Option<String>,
    #[serde(rename = "emailList", default)]
    pub email_list: Vec<String>,
    #[serde(rename = "regLocation", default)]
    pub reg_location: Option<String>,
    #[serde(rename = "taxNumber", default)]
    pub tax_number: Option<String>,
    #[serde(rename = "estiblishTime", default)]
    pub estiblish_time: Option<i64>,
    #[serde(rename = "approvedTime", default)]
    pub approved_time: Option<i64>,
    #[serde(default)]
    pub logo: Option<String>,
    #[serde(default)]
    pub tags: Option<String>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

#[derive(Debug, Clone)]
pub struct TianyanchaHuaweiApi {
    access_key: String,
    secret_key: String,
    http: HttpApiClient,
}

impl TianyanchaHuaweiApi {
    pub fn new(
        access_key: impl Into<String>,
        secret_key: impl Into<String>,
        config: ApiConfig,
    ) -> CreatesResult<Self> {
        let access_key = access_key.into();
        let secret_key = secret_key.into();
        if trim_non_blank(Some(access_key.as_str())).is_none() {
            return Err(CreatesError::InvalidConfig(
                "huawei access_key cannot be blank".to_owned(),
            ));
        }
        if trim_non_blank(Some(secret_key.as_str())).is_none() {
            return Err(CreatesError::InvalidConfig(
                "huawei secret_key cannot be blank".to_owned(),
            ));
        }
        Ok(Self {
            access_key,
            secret_key,
            http: HttpApiClient::new(config)?,
        })
    }

    pub fn search_companies(
        &self,
        keyword: impl AsRef<str>,
        page_num: usize,
        page_size: usize,
    ) -> CreatesResult<TianyanchaHuaweiCompanySearchData> {
        let keyword = trim_non_blank(Some(keyword.as_ref()))
            .ok_or_else(|| CreatesError::InvalidConfig("keyword cannot be blank".to_owned()))?;
        let query = vec![
            ("keyword", keyword.to_owned()),
            ("pageNum", page_num.max(1).to_string()),
            ("pageSize", page_size.max(1).to_string()),
        ];
        let url = self
            .http
            .build_url("/api-mall/api/company_search/query", &query)?;
        let signed_headers = self.sign_headers(Method::GET.as_str(), &url, None, None)?;
        let response: TianyanchaHuaweiResponse =
            self.http.get_json_url_with_headers(url, &signed_headers)?;
        response.into_data("search huawei tianyancha company")
    }

    fn sign_headers(
        &self,
        method: &str,
        url: &Url,
        body: Option<&[u8]>,
        timestamp: Option<&str>,
    ) -> CreatesResult<BTreeMap<String, String>> {
        let payload_hash = sha256_hex(body.unwrap_or_default());
        let host = url
            .host_str()
            .map(|host| match url.port() {
                Some(port) => format!("{host}:{port}"),
                None => host.to_owned(),
            })
            .ok_or_else(|| CreatesError::InvalidResponse("huawei url missing host".to_owned()))?;
        let request_time = timestamp
            .map(ToOwned::to_owned)
            .unwrap_or_else(|| Utc::now().format("%Y%m%dT%H%M%SZ").to_string());

        let canonical_uri = canonical_uri(url);
        let canonical_query = canonical_query_string(url);
        let canonical_headers = format!("host:{host}\nx-sdk-date:{request_time}\n");
        let signed_headers = "host;x-sdk-date";
        let canonical_request = format!(
            "{method}\n{canonical_uri}\n{canonical_query}\n{canonical_headers}\n{signed_headers}\n{payload_hash}"
        );
        let hashed_request = sha256_hex(canonical_request.as_bytes());
        let string_to_sign = format!("SDK-HMAC-SHA256\n{request_time}\n{hashed_request}");

        let mut mac = Hmac::<Sha256>::new_from_slice(self.secret_key.as_bytes())
            .map_err(|error| CreatesError::Signature(error.to_string()))?;
        mac.update(string_to_sign.as_bytes());
        let signature = hex_string(&mac.finalize().into_bytes());
        let authorization = format!(
            "SDK-HMAC-SHA256 Access={access_key}, SignedHeaders={signed_headers}, Signature={signature}",
            access_key = self.access_key
        );

        Ok(BTreeMap::from([
            (HOST.as_str().to_owned(), host),
            ("X-Sdk-Date".to_owned(), request_time),
            (AUTHORIZATION.as_str().to_owned(), authorization),
        ]))
    }
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct TianyanchaHuaweiCompanySearchData {
    #[serde(rename = "companyList", default)]
    pub company_list: Vec<TianyanchaHuaweiCompany>,
    #[serde(rename = "orderNo", default)]
    pub order_no: Option<String>,
    #[serde(rename = "pageInfo", default)]
    pub page_info: Option<TianyanchaHuaweiPageInfo>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct TianyanchaHuaweiCompany {
    #[serde(rename = "companyCode", default)]
    pub company_code: String,
    #[serde(rename = "companyName", default)]
    pub company_name: String,
    #[serde(rename = "companyStatus", default)]
    pub company_status: String,
    #[serde(rename = "creditNo", default)]
    pub credit_no: String,
    #[serde(rename = "establishDate", default)]
    pub establish_date: String,
    #[serde(rename = "legalPerson", default)]
    pub legal_person: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct TianyanchaHuaweiPageInfo {
    #[serde(rename = "pageIndex", default)]
    pub page_index: String,
    #[serde(rename = "pageSize", default)]
    pub page_size: String,
    #[serde(rename = "totalRecords", default)]
    pub total_records: String,
}

#[derive(Debug, Deserialize)]
struct ApiEnvelope<T> {
    #[serde(default)]
    code: Value,
    #[serde(default)]
    message: Option<String>,
    #[serde(default)]
    msg: Option<String>,
    #[serde(default)]
    success: Option<bool>,
    #[serde(default)]
    data: Option<T>,
}

impl<T> ApiEnvelope<T> {
    fn into_data(self, action: &str) -> CreatesResult<T> {
        if self.is_success() {
            return self.data.ok_or_else(|| {
                CreatesError::InvalidResponse(format!("{action} returned success without data"))
            });
        }
        Err(CreatesError::InvalidResponse(format!(
            "{action} failed: {}",
            self.message_or_default()
        )))
    }

    fn into_optional_data(self, action: &str) -> CreatesResult<Option<T>> {
        if self.is_success() {
            return Ok(self.data);
        }
        Err(CreatesError::InvalidResponse(format!(
            "{action} failed: {}",
            self.message_or_default()
        )))
    }

    fn is_success(&self) -> bool {
        self.success.unwrap_or(false)
            || matches!(self.code, Value::Number(ref value) if value.as_i64() == Some(200))
            || matches!(self.code, Value::String(ref value) if value == "200" || value.eq_ignore_ascii_case("success"))
            || self.data.is_some() && self.code.is_null()
    }

    fn message_or_default(&self) -> String {
        self.message
            .clone()
            .or_else(|| self.msg.clone())
            .unwrap_or_else(|| format!("code={}", self.code))
    }
}

#[derive(Debug, Deserialize)]
struct TianyanchaSearchResponse {
    #[serde(default)]
    data: Option<TianyanchaCompanySearchData>,
    #[serde(default)]
    message: Option<String>,
    #[serde(default)]
    state: Option<String>,
    #[serde(default)]
    #[serde(rename = "vipMessage")]
    vip_message: Option<String>,
}

impl TianyanchaSearchResponse {
    fn into_data(self, action: &str) -> CreatesResult<TianyanchaCompanySearchData> {
        if matches!(self.state.as_deref(), Some("ok")) {
            return self.data.ok_or_else(|| {
                CreatesError::InvalidResponse(format!("{action} returned ok without data"))
            });
        }
        Err(CreatesError::InvalidResponse(format!(
            "{action} failed: {}",
            self.message
                .or(self.vip_message)
                .unwrap_or_else(|| "unknown error".to_owned())
        )))
    }
}

#[derive(Debug, Deserialize)]
struct TianyanchaDetailResponse {
    #[serde(default)]
    data: Option<TianyanchaCompanyDetail>,
    #[serde(default)]
    message: Option<String>,
    #[serde(default)]
    state: Option<String>,
    #[serde(rename = "errorMessage", default)]
    error_message: Option<Value>,
}

impl TianyanchaDetailResponse {
    fn into_data(self, action: &str) -> CreatesResult<TianyanchaCompanyDetail> {
        if matches!(self.state.as_deref(), Some("ok")) {
            return self.data.ok_or_else(|| {
                CreatesError::InvalidResponse(format!("{action} returned ok without data"))
            });
        }
        let error_message = self.error_message.and_then(|value| match value {
            Value::Null => None,
            Value::String(value) => Some(value),
            other => Some(other.to_string()),
        });
        Err(CreatesError::InvalidResponse(format!(
            "{action} failed: {}",
            self.message
                .or(error_message)
                .unwrap_or_else(|| "unknown error".to_owned())
        )))
    }
}

#[derive(Debug, Deserialize)]
struct TianyanchaHuaweiResponse {
    #[serde(default)]
    code: i32,
    #[serde(default)]
    data: Option<TianyanchaHuaweiCompanySearchData>,
    #[serde(default)]
    msg: Option<String>,
    #[serde(default)]
    success: Option<bool>,
}

impl TianyanchaHuaweiResponse {
    fn into_data(self, action: &str) -> CreatesResult<TianyanchaHuaweiCompanySearchData> {
        if self.code == 200 || self.success == Some(true) {
            return self.data.ok_or_else(|| {
                CreatesError::InvalidResponse(format!("{action} returned success without data"))
            });
        }
        Err(CreatesError::InvalidResponse(format!(
            "{action} failed: {}",
            self.msg.unwrap_or_else(|| format!("code={}", self.code))
        )))
    }
}

fn default_suno_mv() -> String {
    "chirp-v5".to_owned()
}

fn default_suno_task() -> Option<String> {
    Some("extend".to_owned())
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
        let mut headers = BTreeMap::new();
        if let Some(token) = trim_non_blank(bearer_token) {
            headers.insert(AUTHORIZATION.as_str().to_owned(), format!("Bearer {token}"));
        }
        self.get_json_with_headers(path, query, &headers)
    }

    fn get_json_with_headers<T: DeserializeOwned>(
        &self,
        path: &str,
        query: &[(&str, String)],
        headers: &BTreeMap<String, String>,
    ) -> CreatesResult<T> {
        let url = self.build_url(path, query)?;
        self.get_json_url_with_headers(url, headers)
    }

    fn get_json_url_with_headers<T: DeserializeOwned>(
        &self,
        url: Url,
        headers: &BTreeMap<String, String>,
    ) -> CreatesResult<T> {
        let builder = self.client.request(Method::GET, url.clone());
        let builder = apply_headers(builder, headers)?;
        let response = builder.send()?;
        self.read_json(url, response)
    }

    fn post_json<T: DeserializeOwned, B: Serialize>(
        &self,
        path: &str,
        body: &B,
        bearer_token: Option<&str>,
    ) -> CreatesResult<T> {
        let mut headers = BTreeMap::new();
        if let Some(token) = trim_non_blank(bearer_token) {
            headers.insert(AUTHORIZATION.as_str().to_owned(), format!("Bearer {token}"));
        }
        self.post_json_with_headers(path, body, &headers)
    }

    fn post_json_with_headers<T: DeserializeOwned, B: Serialize>(
        &self,
        path: &str,
        body: &B,
        headers: &BTreeMap<String, String>,
    ) -> CreatesResult<T> {
        let url = self.build_url(path, &[])?;
        let body_bytes = serde_json::to_vec(body)?;
        let builder = self
            .client
            .request(Method::POST, url.clone())
            .header(CONTENT_TYPE, "application/json")
            .body(body_bytes);
        let builder = apply_headers(builder, headers)?;
        let response = builder.send()?;
        self.read_json(url, response)
    }

    fn get_bytes(
        &self,
        path: &str,
        query: &[(&str, String)],
        bearer_token: Option<&str>,
    ) -> CreatesResult<Vec<u8>> {
        let mut headers = BTreeMap::new();
        if let Some(token) = trim_non_blank(bearer_token) {
            headers.insert(AUTHORIZATION.as_str().to_owned(), format!("Bearer {token}"));
        }
        let url = self.build_url(path, query)?;
        let builder = self.client.request(Method::GET, url.clone());
        let builder = apply_headers(builder, &headers)?;
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

    fn build_url(&self, path: &str, query: &[(&str, String)]) -> CreatesResult<Url> {
        let mut url = self.join_url(path)?;
        if !query.is_empty() {
            let mut pairs = url.query_pairs_mut();
            for (name, value) in query {
                pairs.append_pair(name, value);
            }
        }
        Ok(url)
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

fn apply_headers(
    mut builder: reqwest::blocking::RequestBuilder,
    headers: &BTreeMap<String, String>,
) -> CreatesResult<reqwest::blocking::RequestBuilder> {
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
        builder = builder.header(header_name, header_value);
    }
    Ok(builder)
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

fn non_blank(value: Option<&str>) -> Option<&str> {
    trim_non_blank(value)
}

fn ensure_code_200(code: i32, message: Option<&str>, action: &str) -> CreatesResult<()> {
    if code == 200 {
        return Ok(());
    }
    Err(CreatesError::InvalidResponse(format!(
        "{action} failed: code={code}, message={}",
        message.unwrap_or_default()
    )))
}

fn encode_url_component(value: &str) -> String {
    const UNRESERVED: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_.~";
    let mut output = String::new();
    for byte in value.as_bytes() {
        if UNRESERVED.contains(byte) {
            output.push(*byte as char);
        } else {
            output.push_str(&format!("%{:02X}", byte));
        }
    }
    output
}

fn sha256_hex(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    hex_string(&hasher.finalize())
}

fn hex_string(bytes: &[u8]) -> String {
    let mut output = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        output.push_str(&format!("{byte:02x}"));
    }
    output
}

fn canonical_uri(url: &Url) -> String {
    let path = if url.path().is_empty() {
        "/"
    } else {
        url.path()
    };
    if path == "/" {
        return "/".to_owned();
    }
    path.split('/')
        .enumerate()
        .map(|(index, segment)| {
            if index == 0 {
                String::new()
            } else {
                encode_url_component(segment)
            }
        })
        .collect::<Vec<_>>()
        .join("/")
}

fn canonical_query_string(url: &Url) -> String {
    let mut pairs = url
        .query_pairs()
        .map(|(name, value)| {
            (
                encode_url_component(name.as_ref()),
                encode_url_component(value.as_ref()),
            )
        })
        .collect::<Vec<_>>();
    pairs.sort_by(|left, right| left.cmp(right));
    pairs
        .into_iter()
        .map(|(name, value)| format!("{name}={value}"))
        .collect::<Vec<_>>()
        .join("&")
}

fn default_user_agent() -> String {
    format!("tool-creates/{}", env!("CARGO_PKG_VERSION"))
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::error::Error;
    use std::io::{Read, Write};
    use std::net::{TcpListener, TcpStream};
    use std::sync::{Arc, Mutex};
    use std::thread::{self, JoinHandle};

    #[test]
    fn maven_search_parses_latest_version() -> Result<(), Box<dyn Error>> {
        let server = TestServer::spawn(vec![TestResponse::json(
            r#"{"response":{"docs":[{"id":"com.google.guava:guava","g":"com.google.guava","a":"guava","latestVersion":"33.2.1-jre","p":"bundle","timestamp":123456}]}}"#,
        )])?;

        let api = MavenCentralApi::new(ApiConfig::builder(server.base_url()).build()?)?;
        let artifacts = api.search_by_coordinates("com.google.guava", "guava", 5)?;

        assert_eq!(artifacts.len(), 1);
        assert_eq!(artifacts[0].group_id, "com.google.guava");
        assert_eq!(artifacts[0].artifact_id, "guava");
        assert_eq!(artifacts[0].resolved_version(), Some("33.2.1-jre"));

        let requests = server.finish()?;
        assert_eq!(requests.len(), 1);
        assert_eq!(requests[0].method, "GET");
        assert!(
            requests[0]
                .path
                .contains("/solrsearch/select?q=g%3Acom.google.guava+AND+a%3Aguava")
        );
        assert!(requests[0].path.contains("rows=5"));
        Ok(())
    }

    #[test]
    fn maven_download_uses_remotecontent_endpoint() -> Result<(), Box<dyn Error>> {
        let server = TestServer::spawn(vec![TestResponse::text("artifact-body")])?;

        let api = MavenCentralApi::new(ApiConfig::builder(server.base_url()).build()?)?;
        let bytes = api.download_file(
            "com.google.guava",
            "guava",
            "33.2.1-jre",
            "guava-33.2.1-jre.pom",
        )?;

        assert_eq!(String::from_utf8(bytes)?, "artifact-body");

        let requests = server.finish()?;
        assert_eq!(requests.len(), 1);
        assert_eq!(
            requests[0].path,
            "/remotecontent?filepath=com%2Fgoogle%2Fguava%2Fguava%2F33.2.1-jre%2Fguava-33.2.1-jre.pom"
        );
        Ok(())
    }

    #[test]
    fn temp_mail_create_mailbox_and_login_runs_full_flow() -> Result<(), Box<dyn Error>> {
        let server = TestServer::spawn(vec![
            TestResponse::json(
                r#"{"hydra:member":[{"id":"domain-1","domain":"mail.tm","isActive":true,"isPrivate":false}]}"#,
            ),
            TestResponse::json(r#"{"id":"account-1"}"#),
            TestResponse::json(r#"{"token":"token-1"}"#),
        ])?;

        let api = TempMailApi::new(ApiConfig::builder(server.base_url()).build()?)?;
        let mailbox = api.create_mailbox_and_login("az_", 10)?;

        assert!(mailbox.address.ends_with("@mail.tm"));
        assert_eq!(mailbox.account_id, "account-1");
        assert_eq!(mailbox.token, "token-1");
        assert_eq!(mailbox.password.len(), 10);

        let requests = server.finish()?;
        assert_eq!(requests.len(), 3);
        assert_eq!(requests[0].path, "/domains");
        assert_eq!(requests[1].path, "/accounts");
        assert_eq!(requests[2].path, "/token");
        assert!(requests[1].body.contains("\"address\""));
        assert!(requests[2].body.contains("\"password\""));
        Ok(())
    }

    #[test]
    fn temp_mail_get_message_flattens_html_array() -> Result<(), Box<dyn Error>> {
        let server = TestServer::spawn(vec![TestResponse::json(
            r#"{"id":"msg-1","from":{"address":"from@mail.tm","name":"Sender"},"to":[{"address":"to@mail.tm","name":"Receiver"}],"subject":"Hello","text":"Plain","html":["<p>Hello</p>"],"createdAt":"2026-04-20T12:00:00.000Z"}"#,
        )])?;

        let api = TempMailApi::new(ApiConfig::builder(server.base_url()).build()?)?;
        let message = api.get_message("token-1", "msg-1")?;

        assert_eq!(message.id, "msg-1");
        assert_eq!(message.html, "<p>Hello</p>");
        assert_eq!(message.to.len(), 1);

        let requests = server.finish()?;
        let authorization = requests[0]
            .headers
            .get("authorization")
            .cloned()
            .ok_or_else(|| std::io::Error::other("missing authorization header"))?;
        assert_eq!(requests[0].path, "/messages/msg-1");
        assert_eq!(authorization, "Bearer token-1");
        Ok(())
    }

    #[test]
    fn music_search_supports_song_artist_album_and_playlist_queries() -> Result<(), Box<dyn Error>>
    {
        let server = TestServer::spawn(vec![
            TestResponse::json(
                r#"{"code":200,"result":{"songs":[{"id":1,"name":"晴天","artists":[{"id":11,"name":"周杰伦"}],"album":{"id":21,"name":"叶惠美"},"duration":269000}]}}"#,
            ),
            TestResponse::json(
                r#"{"code":200,"result":{"artists":[{"id":11,"name":"周杰伦","albumSize":15,"musicSize":200}]}}"#,
            ),
            TestResponse::json(
                r#"{"code":200,"result":{"albums":[{"id":21,"name":"叶惠美","artist":{"id":11,"name":"周杰伦"}}]}}"#,
            ),
            TestResponse::json(
                r#"{"code":200,"result":{"playlists":[{"id":31,"name":"华语经典","trackCount":128,"playCount":9999}]}}"#,
            ),
        ])?;

        let api = MusicSearchApi::new(test_music_config(server.base_url())?)?;
        let songs = api.search_songs("晴天", 5, 0)?;
        let artists = api.search_artists("周杰伦", 3, 0)?;
        let albums = api.search_albums("叶惠美", 3, 0)?;
        let playlists = api.search_playlists("华语经典", 3, 0)?;

        assert_eq!(songs[0].name, "晴天");
        assert_eq!(songs[0].artists[0].name, "周杰伦");
        assert_eq!(artists[0].name, "周杰伦");
        assert_eq!(albums[0].name, "叶惠美");
        assert_eq!(playlists[0].name, "华语经典");

        let requests = server.finish()?;
        assert_eq!(requests.len(), 4);
        assert!(
            requests[0]
                .path
                .contains("/search/get/web?s=%E6%99%B4%E5%A4%A9")
        );
        assert!(requests[0].path.contains("type=1"));
        assert!(requests[1].path.contains("type=100"));
        assert!(requests[2].path.contains("type=10"));
        assert!(requests[3].path.contains("type=1000"));
        assert_eq!(
            requests[0].headers.get("referer").map(String::as_str),
            Some("https://music.163.com/")
        );
        Ok(())
    }

    #[test]
    fn music_lyric_detail_and_filtering_behave_like_jvm_client() -> Result<(), Box<dyn Error>> {
        let server = TestServer::spawn(vec![
            TestResponse::json(r#"{"code":200,"lrc":{"version":1,"lyric":"[00:01]晴天"}}"#),
            TestResponse::json(
                r#"{"code":200,"songs":[{"id":1,"name":"晴天","artists":[{"id":11,"name":"周杰伦"}],"album":{"id":21,"name":"叶惠美"}},{"id":2,"name":"七里香","artists":[{"id":11,"name":"周杰伦"}],"album":{"id":22,"name":"七里香"}}]}"#,
            ),
            TestResponse::json(
                r#"{"code":200,"result":{"songs":[{"id":1,"name":"晴天","artists":[{"id":11,"name":"周杰伦"}]},{"id":2,"name":"晴天","artists":[{"id":12,"name":"别人"}]}]}}"#,
            ),
        ])?;

        let api = MusicSearchApi::new(test_music_config(server.base_url())?)?;
        let lyric = api.get_lyric(1)?;
        let songs = api.get_song_detail(&[1, 2])?;
        let filtered = api.search_by_song_and_artist("晴天", Some("周杰伦"))?;

        assert_eq!(
            lyric.lrc.and_then(|item| item.lyric).as_deref(),
            Some("[00:01]晴天")
        );
        assert_eq!(songs.len(), 2);
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].artists[0].name, "周杰伦");

        let requests = server.finish()?;
        assert_eq!(requests[0].path, "/song/lyric?id=1&lv=1&tv=1");
        assert_eq!(requests[1].path, "/song/detail?ids=%5B1%2C2%5D");
        assert!(
            requests[2]
                .path
                .contains("/search/get/web?s=%E6%99%B4%E5%A4%A9+%E5%91%A8%E6%9D%B0%E4%BC%A6")
        );
        Ok(())
    }

    #[test]
    fn suno_endpoints_use_bearer_token_and_decode_payloads() -> Result<(), Box<dyn Error>> {
        let server = TestServer::spawn(vec![
            TestResponse::json(r#"{"code":200,"message":"ok","data":"task-1"}"#),
            TestResponse::json(r#"{"code":200,"message":"ok","data":"歌词"}"#),
            TestResponse::json(r#"{"code":200,"message":"ok","data":"concat-1"}"#),
            TestResponse::json(
                r#"{"code":200,"message":"ok","data":{"id":"task-1","status":"complete","audio_url":"https://example.com/1.mp3"}}"#,
            ),
            TestResponse::json(
                r#"{"code":200,"message":"ok","data":[{"id":"task-1","status":"complete"},{"id":"task-2","status":"streaming"}]}"#,
            ),
        ])?;

        let api = SunoApi::new("token-123", test_suno_config(server.base_url())?)?;
        let task_id = api.generate_music(&SunoMusicRequest {
            prompt: "写一首歌".to_owned(),
            title: Some("测试".to_owned()),
            tags: Some("pop".to_owned()),
            ..Default::default()
        })?;
        let lyrics = api.generate_lyrics("写一段歌词")?;
        let concat_task = api.concat_songs("clip-1")?;
        let task = api.fetch_task("task-1")?;
        let tasks = api.batch_fetch_tasks(vec!["task-1", "task-2"])?;

        assert_eq!(task_id, "task-1");
        assert_eq!(lyrics, "歌词");
        assert_eq!(concat_task, "concat-1");
        assert_eq!(
            task.and_then(|item| item.audio_url).as_deref(),
            Some("https://example.com/1.mp3")
        );
        assert_eq!(tasks.len(), 2);

        let requests = server.finish()?;
        assert_eq!(requests.len(), 5);
        for request in &requests {
            assert_eq!(
                request.headers.get("authorization").map(String::as_str),
                Some("Bearer token-123")
            );
        }
        assert_eq!(requests[0].path, "/suno/submit/music");
        assert!(requests[0].body.contains("\"prompt\":\"写一首歌\""));
        assert_eq!(requests[1].path, "/suno/lyrics");
        assert_eq!(requests[2].path, "/suno/concat");
        assert_eq!(requests[3].path, "/suno/fetch/task-1");
        assert_eq!(requests[4].path, "/suno/fetch");
        Ok(())
    }

    #[test]
    fn suno_wait_for_completion_polls_until_complete() -> Result<(), Box<dyn Error>> {
        let server = TestServer::spawn(vec![
            TestResponse::json(
                r#"{"code":200,"message":"ok","data":{"id":"task-1","status":"processing"}}"#,
            ),
            TestResponse::json(
                r#"{"code":200,"message":"ok","data":{"id":"task-1","status":"complete","audio_url":"https://example.com/done.mp3"}}"#,
            ),
        ])?;

        let api = SunoApi::new("token-123", test_suno_config(server.base_url())?)?;
        let mut seen_status = Vec::new();
        let task = api.wait_for_completion_with(
            "task-1",
            Duration::from_millis(50),
            Duration::from_millis(1),
            |status| seen_status.push(status.map(ToOwned::to_owned)),
        )?;

        assert_eq!(task.status.as_deref(), Some("complete"));
        assert_eq!(
            seen_status,
            vec![Some("processing".to_owned()), Some("complete".to_owned())]
        );
        Ok(())
    }

    #[test]
    fn tianyancha_search_and_base_info_send_required_headers() -> Result<(), Box<dyn Error>> {
        let server = TestServer::spawn(vec![
            TestResponse::json(
                r#"{"state":"ok","message":"success","data":{"companyList":[{"id":3398690435,"name":"河南中洛佳科技有限公司","alias":"中洛佳","legalPersonName":"马丽北","regStatus":"存续"}],"companyTotal":1,"companyTotalPage":1,"searchContent":"中洛佳"}}"#,
            ),
            TestResponse::json(
                r#"{"state":"ok","message":"success","data":{"id":3398690435,"name":"河南中洛佳科技有限公司","alias":"中洛佳","legalPersonName":"马丽北","businessScope":"软件开发","phoneList":["0379-65199909"],"emailList":["demo@example.com"]}}"#,
            ),
        ])?;

        let api = TianyanchaApi::new(
            "auth-value",
            "token-value",
            test_tianyancha_config(server.base_url())?,
        )?;
        let search = api.search_company("中洛佳", 1, 10, "0")?;
        let detail = api.get_base_info(3398690435)?;

        assert_eq!(search.company_list.len(), 1);
        assert_eq!(detail.name, "河南中洛佳科技有限公司");

        let requests = server.finish()?;
        assert!(
            requests[0]
                .path
                .contains("/services/v3/search/sNorV4/%E4%B8%AD%E6%B4%9B%E4%BD%B3")
        );
        assert!(requests[0].path.contains("pageNum=1"));
        assert_eq!(
            requests[0].headers.get("authorization").map(String::as_str),
            Some("auth-value")
        );
        assert_eq!(
            requests[0].headers.get("x-auth-token").map(String::as_str),
            Some("token-value")
        );
        assert_eq!(
            requests[0].headers.get("version").map(String::as_str),
            Some("TYC-XCX-WX")
        );
        assert_eq!(
            requests[1].path,
            "/services/v3/t/common/baseinfoV5/3398690435"
        );
        Ok(())
    }

    #[test]
    fn huawei_tianyancha_search_generates_signature_headers() -> Result<(), Box<dyn Error>> {
        let server = TestServer::spawn(vec![TestResponse::json(
            r#"{"code":200,"msg":"成功","success":true,"data":{"companyList":[{"companyCode":"410307100005658","companyName":"洛阳古城机械有限公司","companyStatus":"在营（开业）","creditNo":"91410307171359173W","establishDate":"20030415","legalPerson":"王根成"}],"orderNo":"202511032218555320095","pageInfo":{"pageIndex":"2","pageSize":"20","totalRecords":"528"}}}"#,
        )])?;

        let api = TianyanchaHuaweiApi::new(
            "ak-demo",
            "sk-demo",
            ApiConfig::builder(server.base_url()).build()?,
        )?;
        let data = api.search_companies("测试企业", 2, 20)?;

        assert_eq!(data.company_list.len(), 1);
        assert_eq!(
            data.page_info.as_ref().map(|item| item.page_index.as_str()),
            Some("2")
        );

        let requests = server.finish()?;
        assert_eq!(
            requests[0].path,
            "/api-mall/api/company_search/query?keyword=%E6%B5%8B%E8%AF%95%E4%BC%81%E4%B8%9A&pageNum=2&pageSize=20"
        );
        let authorization = requests[0]
            .headers
            .get("authorization")
            .cloned()
            .ok_or_else(|| std::io::Error::other("missing authorization header"))?;
        assert!(authorization.starts_with("SDK-HMAC-SHA256 Access=ak-demo"));
        assert!(authorization.contains("SignedHeaders=host;x-sdk-date"));
        assert!(requests[0].headers.contains_key("x-sdk-date"));
        Ok(())
    }

    #[test]
    fn huawei_signature_canonicalizes_query_string() -> Result<(), Box<dyn Error>> {
        let api = TianyanchaHuaweiApi::new(
            "ak-demo",
            "sk-demo",
            ApiConfig::builder("http://example.com").build()?,
        )?;
        let url = Url::parse(
            "http://example.com/api-mall/api/company_search/query?pageSize=20&keyword=%E6%B5%8B%E8%AF%95&pageNum=2",
        )?;
        let headers = api.sign_headers("GET", &url, None, Some("20260421T120000Z"))?;

        assert_eq!(
            canonical_query_string(&url),
            "keyword=%E6%B5%8B%E8%AF%95&pageNum=2&pageSize=20"
        );
        assert_eq!(
            headers.get("X-Sdk-Date").map(String::as_str),
            Some("20260421T120000Z")
        );
        assert!(
            headers
                .get("authorization")
                .or_else(|| headers.get("Authorization"))
                .is_some()
        );
        Ok(())
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    struct CapturedRequest {
        method: String,
        path: String,
        headers: BTreeMap<String, String>,
        body: String,
    }

    #[derive(Debug, Clone)]
    struct TestResponse {
        status: u16,
        content_type: &'static str,
        body: String,
    }

    impl TestResponse {
        fn json(body: &str) -> Self {
            Self {
                status: 200,
                content_type: "application/json",
                body: body.to_owned(),
            }
        }

        fn text(body: &str) -> Self {
            Self {
                status: 200,
                content_type: "text/plain; charset=utf-8",
                body: body.to_owned(),
            }
        }
    }

    fn test_music_config(base_url: &str) -> CreatesResult<ApiConfig> {
        ApiConfig::builder(base_url)
            .default_header("Referer", "https://music.163.com/")
            .user_agent("Mozilla/5.0")
            .build()
    }

    fn test_suno_config(base_url: &str) -> CreatesResult<ApiConfig> {
        ApiConfig::builder(base_url)
            .default_header(ACCEPT.as_str(), "application/json")
            .build()
    }

    fn test_tianyancha_config(base_url: &str) -> CreatesResult<ApiConfig> {
        ApiConfig::builder(base_url)
            .default_header(CONTENT_TYPE.as_str(), "application/json")
            .default_header(HOST.as_str(), "api9.tianyancha.com")
            .default_header(ACCEPT.as_str(), "*/*")
            .default_header("version", "TYC-XCX-WX")
            .default_header("User-Agent", "Mozilla/5.0")
            .default_header("Accept-Language", "zh-cn")
            .build()
    }

    struct TestServer {
        base_url: String,
        captured: Arc<Mutex<Vec<CapturedRequest>>>,
        handle: Option<JoinHandle<std::io::Result<()>>>,
    }

    impl TestServer {
        fn spawn(responses: Vec<TestResponse>) -> Result<Self, Box<dyn Error>> {
            let listener = TcpListener::bind("127.0.0.1:0")?;
            let address = listener.local_addr()?;
            let captured = Arc::new(Mutex::new(Vec::new()));
            let captured_clone = Arc::clone(&captured);

            let handle = thread::spawn(move || -> std::io::Result<()> {
                for response in responses {
                    let (mut stream, _) = listener.accept()?;
                    let request = read_request(&mut stream)?;
                    let mut guard = captured_clone
                        .lock()
                        .map_err(|_| std::io::Error::other("request capture mutex poisoned"))?;
                    guard.push(request);
                    drop(guard);
                    write_response(&mut stream, response)?;
                }
                Ok(())
            });

            Ok(Self {
                base_url: format!("http://{address}"),
                captured,
                handle: Some(handle),
            })
        }

        fn base_url(&self) -> &str {
            &self.base_url
        }

        fn finish(mut self) -> Result<Vec<CapturedRequest>, Box<dyn Error>> {
            if let Some(handle) = self.handle.take() {
                match handle.join() {
                    Ok(result) => {
                        result?;
                    }
                    Err(_) => {
                        return Err(Box::new(std::io::Error::other(
                            "test server thread panicked",
                        )));
                    }
                }
            }

            let guard = self
                .captured
                .lock()
                .map_err(|_| std::io::Error::other("request capture mutex poisoned"))?;
            Ok(guard.clone())
        }
    }

    fn read_request(stream: &mut TcpStream) -> std::io::Result<CapturedRequest> {
        let mut buffer = Vec::new();
        let mut chunk = [0u8; 1024];
        let header_end = loop {
            let read = stream.read(&mut chunk)?;
            if read == 0 {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::UnexpectedEof,
                    "request ended before headers",
                ));
            }
            buffer.extend_from_slice(&chunk[..read]);
            if let Some(index) = find_bytes(&buffer, b"\r\n\r\n") {
                break index + 4;
            }
        };

        let header_text = String::from_utf8_lossy(&buffer[..header_end]).into_owned();
        let mut lines = header_text.split("\r\n");
        let request_line = lines.next().ok_or_else(|| {
            std::io::Error::new(std::io::ErrorKind::InvalidData, "missing request line")
        })?;
        let mut request_parts = request_line.split_whitespace();
        let method = request_parts
            .next()
            .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::InvalidData, "missing method"))?
            .to_owned();
        let path = request_parts
            .next()
            .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::InvalidData, "missing path"))?
            .to_owned();

        let mut headers = BTreeMap::new();
        let mut content_length = 0usize;
        for line in lines {
            if line.is_empty() {
                continue;
            }

            let Some((name, value)) = line.split_once(':') else {
                continue;
            };
            let normalized_name = name.trim().to_ascii_lowercase();
            let trimmed_value = value.trim().to_owned();
            if normalized_name == "content-length" {
                content_length = match trimmed_value.parse::<usize>() {
                    Ok(value) => value,
                    Err(_) => 0,
                };
            }
            headers.insert(normalized_name, trimmed_value);
        }

        while buffer.len() < header_end + content_length {
            let read = stream.read(&mut chunk)?;
            if read == 0 {
                break;
            }
            buffer.extend_from_slice(&chunk[..read]);
        }

        let body_bytes = if content_length == 0 {
            &[][..]
        } else {
            &buffer[header_end..header_end + content_length]
        };

        Ok(CapturedRequest {
            method,
            path,
            headers,
            body: String::from_utf8_lossy(body_bytes).into_owned(),
        })
    }

    fn write_response(stream: &mut TcpStream, response: TestResponse) -> std::io::Result<()> {
        let body = response.body;
        let payload = format!(
            "HTTP/1.1 {} OK\r\nContent-Type: {}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
            response.status,
            response.content_type,
            body.len(),
            body
        );
        stream.write_all(payload.as_bytes())?;
        stream.flush()
    }

    fn find_bytes(haystack: &[u8], needle: &[u8]) -> Option<usize> {
        haystack
            .windows(needle.len())
            .position(|window| window == needle)
    }
}
