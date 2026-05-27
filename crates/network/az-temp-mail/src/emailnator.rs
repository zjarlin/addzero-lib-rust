use crate::http::HttpApiClient;
use crate::model::{
    CreateMailboxRequest, ListResponse, PageRequest, TempMailMailbox, TempMailMessageDetail,
    TempMailMessageSummary, TempMailProviderKind,
};
use crate::provider::TempMailProvider;
use crate::util::trim_non_blank;
use crate::{ApiConfig, TempMailError, TempMailResult};
use az_derive_aliases::{
    apply, deserialize_debug, impl_default, plain_code_enum, plain_debug, plain_eq, serialize_eq,
};
use regex::Regex;
use reqwest::blocking::Response;
use reqwest::header::{COOKIE, SET_COOKIE};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::sync::LazyLock;
use std::sync::Mutex;

const DEFAULT_EMAILNATOR_BASE_URL: &str = "https://www.emailnator.com";

static HTTP_LINK_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"https?://[^\s<>"{}|\\^`\[\]]+"#).expect("http-link regex should compile")
});

/// Email generation mode accepted by Emailnator's `/generate-email` endpoint.
#[apply(plain_code_enum)]
pub enum EmailnatorEmailMode {
    /// Gmail plus-address variant.
    #[strum(serialize = "plusGmail")]
    PlusGmail,
    /// Gmail dot-address variant.
    #[strum(serialize = "dotGmail")]
    DotGmail,
}

/// Request options for generating an Emailnator address.
#[apply(plain_eq)]
pub struct EmailnatorEmailRequest {
    /// Address generation modes forwarded to Emailnator.
    pub modes: Vec<EmailnatorEmailMode>,
}

impl_default!(EmailnatorEmailRequest => EmailnatorEmailRequest {
    modes: vec![
        EmailnatorEmailMode::PlusGmail,
        EmailnatorEmailMode::DotGmail,
    ],
});

impl EmailnatorEmailRequest {
    /// Creates a request from explicit generation modes.
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

/// Blocking client for Emailnator's hosted temporary mailbox API.
#[apply(plain_debug)]
pub struct EmailnatorTempMailApi {
    http: HttpApiClient,
    xsrf: Mutex<Option<XsrfToken>>,
}

impl EmailnatorTempMailApi {
    /// Creates a client from explicit API configuration.
    pub fn new(config: ApiConfig) -> TempMailResult<Self> {
        Ok(Self {
            http: HttpApiClient::new(config)?,
            xsrf: Mutex::new(None),
        })
    }

    /// Generates a temporary email address.
    pub fn generate_email(&self, request: &EmailnatorEmailRequest) -> TempMailResult<String> {
        let response = self.post_json("/generate-email", &request.request_body())?;
        let response: EmailnatorGenerateEmailResponse = read_json_response(response)?;
        response.into_email().ok_or_else(|| {
            TempMailError::InvalidResponse("Emailnator response did not include email".to_owned())
        })
    }

    /// Lists messages for an Emailnator address.
    pub fn fetch_message_list(
        &self,
        email: impl AsRef<str>,
    ) -> TempMailResult<Vec<TempMailMessageSummary>> {
        let email = required_email(email.as_ref())?;
        let response = self.post_json("/message-list", &json!({ "email": email }))?;
        let response: EmailnatorMessageListResponse = read_json_response(response)?;
        Ok(response
            .message_data
            .into_iter()
            .filter_map(emailnator_summary_from_raw)
            .collect())
    }

    /// Fetches a raw message body for an Emailnator address and message id.
    pub fn fetch_message_body(
        &self,
        email: impl AsRef<str>,
        message_id: impl AsRef<str>,
    ) -> TempMailResult<String> {
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

    fn post_json<T: Serialize>(&self, path: &str, body: &T) -> TempMailResult<Response> {
        let token = self.xsrf_token()?;
        self.http
            .post(path)?
            .header("X-Xsrf-Token", token.decoded)
            .header(COOKIE, format!("XSRF-TOKEN={}", token.raw_cookie_value))
            .json(body)
            .send()
            .map_err(TempMailError::Transport)
    }

    fn xsrf_token(&self) -> TempMailResult<XsrfToken> {
        let existing = self
            .xsrf
            .lock()
            .map_err(|_| TempMailError::InvalidConfig("Emailnator token lock poisoned".to_owned()))?
            .clone();
        if let Some(token) = existing {
            return Ok(token);
        }

        let response = self.http.get("/")?.send()?;
        let token = extract_xsrf_token(&response)?;
        let mut guard = self.xsrf.lock().map_err(|_| {
            TempMailError::InvalidConfig("Emailnator token lock poisoned".to_owned())
        })?;
        *guard = Some(token.clone());
        Ok(token)
    }
}

impl TempMailProvider for EmailnatorTempMailApi {
    fn provider_kind(&self) -> TempMailProviderKind {
        TempMailProviderKind::Emailnator
    }

    fn create_mailbox(&self, _request: &CreateMailboxRequest) -> TempMailResult<TempMailMailbox> {
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
    ) -> TempMailResult<ListResponse<TempMailMessageSummary>> {
        let results = self.fetch_message_list(&mailbox.address)?;
        let count = u64::try_from(results.len()).unwrap_or(u64::MAX);
        Ok(ListResponse { results, count })
    }

    fn get_message(
        &self,
        mailbox: &TempMailMailbox,
        message_id: &str,
    ) -> TempMailResult<Option<TempMailMessageDetail>> {
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

/// Creates a client for the hosted Emailnator API.
pub fn create_emailnator_api() -> TempMailResult<EmailnatorTempMailApi> {
    EmailnatorTempMailApi::new(ApiConfig::builder(DEFAULT_EMAILNATOR_BASE_URL).build()?)
}

/// Extracts the first HTTP(S) link from text, optionally restricted by keyword.
#[must_use]
pub fn extract_first_http_link(content: impl AsRef<str>, keyword: Option<&str>) -> Option<String> {
    let keyword = keyword.and_then(|value| trim_non_blank(Some(value)));
    HTTP_LINK_RE
        .find_iter(content.as_ref())
        .map(|link| link.as_str().trim_end_matches(['.', ',', ';']).to_owned())
        .find(|link| keyword.is_none_or(|keyword| link.contains(keyword)))
}

#[apply(plain_eq)]
struct XsrfToken {
    raw_cookie_value: String,
    decoded: String,
}

#[apply(serialize_eq)]
struct EmailnatorGenerateEmailBody {
    email: Vec<String>,
}

#[apply(deserialize_debug)]
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

#[apply(deserialize_debug)]
#[serde(untagged)]
enum EmailnatorEmailValue {
    Single(String),
    Many(Vec<String>),
}

#[apply(deserialize_debug)]
struct EmailnatorMessageListResponse {
    #[serde(default, rename = "messageData")]
    message_data: Vec<EmailnatorMessageSummaryRaw>,
}

#[apply(deserialize_debug)]
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

fn extract_xsrf_token(response: &Response) -> TempMailResult<XsrfToken> {
    if !response.status().is_success() {
        return Err(TempMailError::HttpStatus {
            url: response.url().to_string(),
            status: response.status().as_u16(),
            body: String::new(),
        });
    }

    for value in response.headers().get_all(SET_COOKIE) {
        let Ok(cookie) = value.to_str() else {
            continue;
        };
        let Some(raw_cookie_value) = extract_cookie_value(cookie, "XSRF-TOKEN") else {
            continue;
        };
        let decoded = urlencoding::decode(raw_cookie_value)
            .map_err(|error| TempMailError::InvalidResponse(error.to_string()))?
            .into_owned();
        return Ok(XsrfToken {
            raw_cookie_value: raw_cookie_value.to_owned(),
            decoded,
        });
    }

    Err(TempMailError::InvalidResponse(
        "Emailnator XSRF-TOKEN cookie missing".to_owned(),
    ))
}

fn extract_cookie_value<'a>(cookie: &'a str, name: &str) -> Option<&'a str> {
    cookie
        .split(';')
        .map(str::trim)
        .find_map(|part| part.strip_prefix(name)?.strip_prefix('='))
        .filter(|value| !value.is_empty())
}

fn read_json_response<T: for<'de> Deserialize<'de>>(response: Response) -> TempMailResult<T> {
    let response = ensure_success_response(response)?;
    let bytes = response.bytes()?;
    Ok(serde_json::from_slice(bytes.as_ref())?)
}

fn read_text_response(response: Response) -> TempMailResult<String> {
    let response = ensure_success_response(response)?;
    Ok(response.text()?)
}

fn ensure_success_response(response: Response) -> TempMailResult<Response> {
    if response.status().is_success() {
        return Ok(response);
    }

    let status = response.status();
    let url = response.url().to_string();
    let body = match response.text() {
        Ok(body) => body,
        Err(error) => return Err(TempMailError::Transport(error)),
    };
    Err(TempMailError::HttpStatus {
        url,
        status: status.as_u16(),
        body,
    })
}

fn required_email(value: &str) -> TempMailResult<&str> {
    trim_non_blank(Some(value))
        .ok_or_else(|| TempMailError::InvalidConfig("email cannot be blank".to_owned()))
}

fn required_message_id(value: &str) -> TempMailResult<&str> {
    trim_non_blank(Some(value))
        .ok_or_else(|| TempMailError::InvalidConfig("message_id cannot be blank".to_owned()))
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
