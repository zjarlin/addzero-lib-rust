use crate::http::HttpApiClient;
use crate::model::{
    CreateMailboxRequest, ListResponse, PageRequest, TempMailMailbox, TempMailMessageDetail,
    TempMailMessageSummary, TempMailProviderKind,
};
use crate::provider::TempMailProvider;
use crate::input::trim_non_blank;
use crate::config::ApiConfig;
use anyhow::{Context, anyhow, bail};
use regex::Regex;
use reqwest::blocking::Response;
use reqwest::header::{COOKIE, SET_COOKIE};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::sync::LazyLock;
use std::sync::Mutex;

const DEFAULT_EMAILNATOR_BASE_URL: &str = "https://www.emailnator.com";

static HTTP_LINK_RE: LazyLock<Option<Regex>> =
    LazyLock::new(|| Regex::new(r#"https?://[^\s<>"{}|\\^`\[\]]+"#).ok());

/// Emailnator `/generate-email` 端点接受的邮箱生成模式。
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, strum::EnumString, strum::IntoStaticStr, strum::VariantArray)]
#[strum(serialize_all = "snake_case")]
pub enum EmailnatorEmailMode {
    /// Gmail 加号地址变体。
    #[strum(serialize = "plusGmail")]
    PlusGmail,
    /// Gmail 点号地址变体。
    #[strum(serialize = "dotGmail")]
    DotGmail,
}

impl EmailnatorEmailMode {
    #[allow(dead_code)]
    pub const ALL: &'static [Self] = <Self as strum::VariantArray>::VARIANTS;

    #[must_use]
    pub fn as_str(self) -> &'static str {
        self.into()
    }

    #[must_use]
    pub fn code(self) -> &'static str {
        self.as_str()
    }

    pub fn from_code(value: &str) -> Option<Self> {
        value.parse().ok()
    }
}

/// 生成 Emailnator 地址的请求选项。
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EmailnatorEmailRequest {
    /// 转发给 Emailnator 的地址生成模式列表。
    pub modes: Vec<EmailnatorEmailMode>,
}

impl Default for EmailnatorEmailRequest {
    fn default() -> Self {
        EmailnatorEmailRequest {
    modes: vec![
        EmailnatorEmailMode::PlusGmail,
        EmailnatorEmailMode::DotGmail,
    ],
}
    }
}

impl EmailnatorEmailRequest {
    /// 根据显式生成模式创建请求。
    #[must_use]
    pub fn new(modes: impl IntoIterator<Item = EmailnatorEmailMode>) -> Self {
        let modes = modes.into_iter().collect::<Vec<_>>();
        if modes.is_empty() {
            Self::default()
        } else {
            Self { modes }
        }
    }

    fn request_body(&self) -> EmailnatorGenerateEmailBody {
        EmailnatorGenerateEmailBody {
            email: self
                .modes
                .iter()
                .map(|mode| mode.code().to_owned())
                .collect(),
        }
    }
}

/// Emailnator 托管临时邮箱 API 的阻塞客户端。
#[derive(Debug)]
pub struct EmailnatorTempMailApi {
    http: HttpApiClient,
    xsrf: Mutex<Option<XsrfToken>>,
}

impl EmailnatorTempMailApi {
    /// 根据显式 API 配置创建客户端。
    pub fn new(config: ApiConfig) -> anyhow::Result<Self> {
        Ok(Self {
            http: HttpApiClient::new(config)?,
            xsrf: Mutex::new(None),
        })
    }

    /// 生成临时邮箱地址。
    pub fn generate_email(&self, request: &EmailnatorEmailRequest) -> anyhow::Result<String> {
        let response = self.post_json("/generate-email", &request.request_body())?;
        let response: EmailnatorGenerateEmailResponse = read_json_response(response)?;
        response.into_email().ok_or_else(|| {
            anyhow!("invalid response: Emailnator response did not include email")
        })
    }

    /// 列出 Emailnator 地址中的邮件。
    pub fn fetch_message_list(
        &self,
        email: impl AsRef<str>,
    ) -> anyhow::Result<Vec<TempMailMessageSummary>> {
        let email = required_email(email.as_ref())?;
        let response = self.post_json("/message-list", &json!({ "email": email }))?;
        let response: EmailnatorMessageListResponse = read_json_response(response)?;
        Ok(response
            .message_data
            .into_iter()
            .filter_map(emailnator_summary_from_raw)
            .collect())
    }

    /// 按 Emailnator 地址和消息 ID 拉取原始邮件正文。
    pub fn fetch_message_body(
        &self,
        email: impl AsRef<str>,
        message_id: impl AsRef<str>,
    ) -> anyhow::Result<String> {
        let email = required_email(email.as_ref())?;
        let message_id = required_message_id(message_id.as_ref())?;
        let response = self.post_json(
            "/message-list",
            &json!({
                "email": email,
                "messageID": message_id,
            }),
        )?;
        read_text_response(response)
    }

    fn post_json<T: Serialize>(&self, path: &str, body: &T) -> anyhow::Result<Response> {
        let token = self.xsrf_token()?;
        self.http
            .post(path)?
            .header("X-Xsrf-Token", token.decoded)
            .header(COOKIE, format!("XSRF-TOKEN={}", token.raw_cookie_value))
            .json(body)
            .send()
            .with_context(|| format!("failed to send Emailnator request `{path}`"))
    }

    fn xsrf_token(&self) -> anyhow::Result<XsrfToken> {
        let existing = self
            .xsrf
            .lock()
            .map_err(|_| anyhow!("invalid config: Emailnator token lock poisoned"))?
            .clone();
        if let Some(token) = existing {
            return Ok(token);
        }

        let response = self
            .http
            .get("/")?
            .send()
            .context("failed to fetch Emailnator XSRF token")?;
        let token = extract_xsrf_token(&response)?;
        let mut guard = self.xsrf.lock().map_err(|_| {
            anyhow!("invalid config: Emailnator token lock poisoned")
        })?;
        *guard = Some(token.clone());
        Ok(token)
    }
}

impl TempMailProvider for EmailnatorTempMailApi {
    fn provider_kind(&self) -> TempMailProviderKind {
        TempMailProviderKind::Emailnator
    }

    fn create_mailbox(&self, _request: &CreateMailboxRequest) -> anyhow::Result<TempMailMailbox> {
        let address = self.generate_email(&EmailnatorEmailRequest::default())?;
        Ok(TempMailMailbox {
            provider: TempMailProviderKind::Emailnator,
            address: address.clone(),
            credential: address,
            account_id: None,
            password: None,
        })
    }

    fn list_messages(
        &self,
        mailbox: &TempMailMailbox,
        _page: PageRequest,
    ) -> anyhow::Result<ListResponse<TempMailMessageSummary>> {
        let results = self.fetch_message_list(&mailbox.address)?;
        let count = u64::try_from(results.len()).unwrap_or(u64::MAX);
        Ok(ListResponse { results, count })
    }

    fn get_message(
        &self,
        mailbox: &TempMailMailbox,
        message_id: &str,
    ) -> anyhow::Result<Option<TempMailMessageDetail>> {
        let body = self.fetch_message_body(&mailbox.address, message_id)?;
        Ok(Some(TempMailMessageDetail {
            id: message_id.to_owned(),
            from_address: String::new(),
            from_name: String::new(),
            to: Vec::new(),
            subject: String::new(),
            text: body.clone(),
            html: body.clone(),
            raw: body,
            created_at: String::new(),
        }))
    }
}

/// 创建托管 Emailnator API 客户端。
pub fn create_emailnator_api() -> anyhow::Result<EmailnatorTempMailApi> {
    EmailnatorTempMailApi::new(ApiConfig::builder(DEFAULT_EMAILNATOR_BASE_URL).build()?)
}

/// 从文本中提取第一个 HTTP(S) 链接，并可按关键字限制。
#[must_use]
pub fn extract_first_http_link(content: impl AsRef<str>, keyword: Option<&str>) -> Option<String> {
    let keyword = keyword.and_then(|value| trim_non_blank(Some(value)));
    HTTP_LINK_RE
        .as_ref()?
        .find_iter(content.as_ref())
        .map(|link| link.as_str().trim_end_matches(['.', ',', ';']).to_owned())
        .find(|link| keyword.is_none_or(|keyword| link.contains(keyword)))
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct XsrfToken {
    raw_cookie_value: String,
    decoded: String,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize)]
struct EmailnatorGenerateEmailBody {
    email: Vec<String>,
}

#[derive(Debug, serde::Deserialize)]
struct EmailnatorGenerateEmailResponse {
    email: EmailnatorEmailValue,
}

impl EmailnatorGenerateEmailResponse {
    fn into_email(self) -> Option<String> {
        match self.email {
            EmailnatorEmailValue::Single(value) => {
                trim_non_blank(Some(value.as_str())).map(ToOwned::to_owned)
            }
            EmailnatorEmailValue::Many(values) => values
                .into_iter()
                .find_map(|value| trim_non_blank(Some(value.as_str())).map(ToOwned::to_owned)),
        }
    }
}

#[derive(Debug, serde::Deserialize)]
#[serde(untagged)]
enum EmailnatorEmailValue {
    Single(String),
    Many(Vec<String>),
}

#[derive(Debug, serde::Deserialize)]
struct EmailnatorMessageListResponse {
    #[serde(default, rename = "messageData")]
    message_data: Vec<EmailnatorMessageSummaryRaw>,
}

#[derive(Debug, serde::Deserialize)]
struct EmailnatorMessageSummaryRaw {
    #[serde(default, rename = "messageID")]
    message_id: Value,
    #[serde(default)]
    from: String,
    #[serde(default)]
    subject: String,
    #[serde(default)]
    time: String,
    #[serde(default)]
    date: String,
}

fn emailnator_summary_from_raw(raw: EmailnatorMessageSummaryRaw) -> Option<TempMailMessageSummary> {
    let id = value_to_non_blank_string(&raw.message_id)?;
    Some(TempMailMessageSummary {
        id,
        from_address: raw.from,
        from_name: String::new(),
        subject: raw.subject,
        intro: String::new(),
        created_at: if raw.time.is_empty() {
            raw.date
        } else {
            raw.time
        },
    })
}

fn extract_xsrf_token(response: &Response) -> anyhow::Result<XsrfToken> {
    if !response.status().is_success() {
        bail!(
            "request to `{}` returned HTTP {}: ",
            response.url(),
            response.status().as_u16()
        );
    }

    for value in response.headers().get_all(SET_COOKIE) {
        let Ok(cookie) = value.to_str() else {
            continue;
        };
        let Some(raw_cookie_value) = extract_cookie_value(cookie, "XSRF-TOKEN") else {
            continue;
        };
        let decoded = urlencoding::decode(raw_cookie_value)
            .with_context(|| format!("invalid response: failed to decode XSRF cookie `{raw_cookie_value}`"))?
            .into_owned();
        return Ok(XsrfToken {
            raw_cookie_value: raw_cookie_value.to_owned(),
            decoded,
        });
    }

    bail!("invalid response: Emailnator XSRF-TOKEN cookie missing")
}

fn extract_cookie_value<'a>(cookie: &'a str, name: &str) -> Option<&'a str> {
    cookie
        .split(';')
        .map(str::trim)
        .find_map(|part| part.strip_prefix(name)?.strip_prefix('='))
        .filter(|value| !value.is_empty())
}

fn read_json_response<T: for<'de> Deserialize<'de>>(response: Response) -> anyhow::Result<T> {
    let response = ensure_success_response(response)?;
    let url = response.url().to_string();
    let bytes = response
        .bytes()
        .with_context(|| format!("failed to read response body from `{url}`"))?;
    serde_json::from_slice(bytes.as_ref())
        .with_context(|| format!("failed to parse JSON response from `{url}`"))
}

fn read_text_response(response: Response) -> anyhow::Result<String> {
    let response = ensure_success_response(response)?;
    let url = response.url().to_string();
    response
        .text()
        .with_context(|| format!("failed to read text response from `{url}`"))
}

fn ensure_success_response(response: Response) -> anyhow::Result<Response> {
    if response.status().is_success() {
        return Ok(response);
    }

    let status = response.status();
    let url = response.url().to_string();
    let body = response
        .text()
        .with_context(|| format!("failed to read error response body from `{url}`"))?;
    bail!("request to `{url}` returned HTTP {}: {body}", status.as_u16())
}

fn required_email(value: &str) -> anyhow::Result<&str> {
    trim_non_blank(Some(value))
        .ok_or_else(|| anyhow!("invalid config: email cannot be blank"))
}

fn required_message_id(value: &str) -> anyhow::Result<&str> {
    trim_non_blank(Some(value))
        .ok_or_else(|| anyhow!("invalid config: message_id cannot be blank"))
}

fn value_to_non_blank_string(value: &Value) -> Option<String> {
    match value {
        Value::String(value) => trim_non_blank(Some(value.as_str())).map(ToOwned::to_owned),
        Value::Number(value) => Some(value.to_string()),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::{EmailnatorEmailMode, EmailnatorEmailRequest, extract_first_http_link};

    #[test]
    fn email_request_defaults_to_gmail_variants() {
        let body = EmailnatorEmailRequest::default().request_body();

        assert_eq!(body.email, vec!["plusGmail", "dotGmail"]);
    }

    #[test]
    fn empty_email_request_modes_fall_back_to_default() {
        let body = EmailnatorEmailRequest::new([]).request_body();

        assert_eq!(body.email, vec!["plusGmail", "dotGmail"]);
    }

    #[test]
    fn explicit_email_request_modes_are_preserved() {
        let body = EmailnatorEmailRequest::new([EmailnatorEmailMode::DotGmail]).request_body();

        assert_eq!(body.email, vec!["dotGmail"]);
    }

    #[test]
    fn link_extractor_can_filter_by_keyword() {
        let content = "open https://example.com/a then https://example.com/activate?id=1";

        assert_eq!(
            extract_first_http_link(content, Some("activate")).as_deref(),
            Some("https://example.com/activate?id=1")
        );
    }
}
