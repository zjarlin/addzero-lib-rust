use base64::Engine;
use regex::Regex;
use reqwest::Method;
use reqwest::Url;
use reqwest::blocking::multipart::Form;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::borrow::Cow;
use std::collections::BTreeMap;
use std::sync::LazyLock;
use std::time::Duration;
use thiserror::Error;

static LINE_CONTINUATION_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\\\s*\r?\n").expect("line continuation regex should compile"));
static UUID_LIKE_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?i)^[a-f0-9\-]{20,}$").expect("uuid regex should compile"));
static NUMERIC_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^\d+$").expect("numeric regex should compile"));

#[derive(Debug, Error)]
pub enum CurlError {
    #[error("failed to tokenize curl command")]
    Tokenize,
    #[error("flag `{0}` requires a value")]
    MissingFlagValue(&'static str),
    #[error("curl command does not contain a URL")]
    MissingUrl,
    #[error("invalid HTTP method `{0}`")]
    InvalidMethod(String),
    #[error("invalid URL `{0}`")]
    InvalidUrl(String),
    #[error("invalid header expression `{0}`")]
    InvalidHeader(String),
    #[error("invalid form expression `{0}`")]
    InvalidFormField(String),
    #[error("failed to parse JSON payload: {0}")]
    Json(#[from] serde_json::Error),
    #[error("failed to build request: {0}")]
    RequestBuild(#[source] reqwest::Error),
    #[error("failed to execute request: {0}")]
    Execute(#[source] reqwest::Error),
    #[error("response body is not valid UTF-8: {0}")]
    Utf8(#[from] std::string::FromUtf8Error),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MutationRule {
    String,
    Number,
    Null,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedCurl {
    pub method: Method,
    pub url: String,
    pub headers: BTreeMap<String, String>,
    pub authorization: Option<String>,
    pub body: Option<String>,
    pub query_params: BTreeMap<String, String>,
    pub path_params: Vec<String>,
    pub form_params: BTreeMap<String, String>,
    pub content_type: Option<String>,
}

impl ParsedCurl {
    pub fn builder(url: impl Into<String>) -> CurlBuilder {
        CurlBuilder::new(url)
    }

    pub fn header(&self, name: impl AsRef<str>) -> Option<&str> {
        self.headers
            .get(&normalize_header_name(name.as_ref()))
            .map(String::as_str)
    }

    pub fn inferred_content_type(&self) -> Option<&str> {
        self.content_type
            .as_deref()
            .or_else(|| self.header("content-type"))
    }

    fn finalize(mut self) -> Result<Self, CurlError> {
        if self.url.trim().is_empty() {
            return Err(CurlError::MissingUrl);
        }

        let parsed_url =
            Url::parse(&self.url).map_err(|_| CurlError::InvalidUrl(self.url.clone()))?;

        if self.content_type.is_none() {
            self.content_type = self.header("content-type").map(ToOwned::to_owned);
        }

        if self.content_type.is_none() && self.body.as_deref().is_some_and(looks_like_json) {
            self.content_type = Some("application/json".to_owned());
            self.headers
                .entry("content-type".to_owned())
                .or_insert_with(|| "application/json".to_owned());
        }

        if self.content_type.is_none() && !self.form_params.is_empty() {
            self.content_type = Some("multipart/form-data".to_owned());
            self.headers
                .entry("content-type".to_owned())
                .or_insert_with(|| "multipart/form-data".to_owned());
        }

        self.query_params = extract_query_params(&parsed_url);
        self.path_params = extract_path_params(&parsed_url);
        Ok(self)
    }
}

#[derive(Debug, Clone)]
pub struct CurlBuilder {
    method: Option<Method>,
    url: String,
    headers: BTreeMap<String, String>,
    authorization: Option<String>,
    body: Option<String>,
    form_params: BTreeMap<String, String>,
    content_type: Option<String>,
}

impl CurlBuilder {
    pub fn new(url: impl Into<String>) -> Self {
        Self {
            method: None,
            url: url.into(),
            headers: BTreeMap::new(),
            authorization: None,
            body: None,
            form_params: BTreeMap::new(),
            content_type: None,
        }
    }

    pub fn method(mut self, method: impl AsRef<str>) -> Result<Self, CurlError> {
        self.method = Some(parse_method(method.as_ref())?);
        Ok(self)
    }

    pub fn header(mut self, name: impl AsRef<str>, value: impl Into<String>) -> Self {
        self.headers
            .insert(normalize_header_name(name.as_ref()), value.into());
        self
    }

    pub fn content_type(mut self, value: impl Into<String>) -> Self {
        let value = value.into();
        self.content_type = Some(value.clone());
        self.headers.insert("content-type".to_owned(), value);
        self
    }

    pub fn basic_auth(mut self, user: impl AsRef<str>, password: impl AsRef<str>) -> Self {
        let token = format!("{}:{}", user.as_ref(), password.as_ref());
        let encoded = base64::engine::general_purpose::STANDARD.encode(token);
        let header_value = format!("Basic {encoded}");
        self.authorization = Some(header_value.clone());
        self.headers
            .insert("authorization".to_owned(), header_value);
        self
    }

    pub fn body(mut self, value: impl Into<String>) -> Self {
        self.body = Some(value.into());
        self
    }

    pub fn form_field(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
        self.form_params.insert(name.into(), value.into());
        self
    }

    pub fn build(self) -> Result<ParsedCurl, CurlError> {
        let method = match self.method {
            Some(method) => method,
            None if self.body.is_some() || !self.form_params.is_empty() => Method::POST,
            None => Method::GET,
        };

        ParsedCurl {
            method,
            url: self.url,
            headers: self.headers,
            authorization: self.authorization,
            body: self.body,
            query_params: BTreeMap::new(),
            path_params: Vec::new(),
            form_params: self.form_params,
            content_type: self.content_type,
        }
        .finalize()
    }
}

#[macro_export]
macro_rules! curl {
    ($command:expr) => {
        $crate::CurlParser::parse($command)
    };
}

pub struct CurlParser;

impl CurlParser {
    pub fn parse(command: impl AsRef<str>) -> Result<ParsedCurl, CurlError> {
        let normalized = normalize_command(command.as_ref());
        let tokens = shlex::split(&normalized).ok_or(CurlError::Tokenize)?;
        let mut iter = tokens.into_iter().peekable();

        if iter.peek().is_some_and(|token| token == "curl") {
            iter.next();
        }

        let mut builder = CurlBuilder::new(String::new());
        let mut pending_data = Vec::new();
        let mut explicit_method = None::<Method>;
        let mut saw_head = false;

        while let Some(token) = iter.next() {
            match token.as_str() {
                "-X" | "--request" => {
                    let value = iter
                        .next()
                        .ok_or(CurlError::MissingFlagValue("--request"))?;
                    explicit_method = Some(parse_method(&value)?);
                }
                "-I" | "--head" => {
                    saw_head = true;
                    explicit_method = Some(Method::HEAD);
                }
                "-H" | "--header" => {
                    let value = iter.next().ok_or(CurlError::MissingFlagValue("--header"))?;
                    let (name, header_value) = split_header(&value)?;
                    builder = builder.header(name, header_value);
                }
                "-b" | "--cookie" => {
                    let value = iter.next().ok_or(CurlError::MissingFlagValue("--cookie"))?;
                    builder = builder.header("cookie", value);
                }
                "-u" | "--user" => {
                    let value = iter.next().ok_or(CurlError::MissingFlagValue("--user"))?;
                    let (user, password) = value.split_once(':').unwrap_or((value.as_str(), ""));
                    builder = builder.basic_auth(user, password);
                }
                "-d" | "--data" | "--data-raw" | "--data-binary" | "--data-urlencode" => {
                    let value = iter.next().ok_or(CurlError::MissingFlagValue("--data"))?;
                    pending_data.push(value);
                }
                "-F" | "--form" => {
                    let value = iter.next().ok_or(CurlError::MissingFlagValue("--form"))?;
                    let (name, form_value) = split_form_field(&value)?;
                    builder = builder.form_field(name, form_value);
                }
                "--url" => {
                    let value = iter.next().ok_or(CurlError::MissingFlagValue("--url"))?;
                    builder.url = value;
                }
                "--compressed" | "--location" | "-L" | "--silent" | "-s" | "--insecure" | "-k"
                | "--globoff" | "--verbose" | "-v" => {}
                _ if token.starts_with("http://") || token.starts_with("https://") => {
                    if builder.url.is_empty() {
                        builder.url = token;
                    }
                }
                _ if token.starts_with('-') => {}
                _ => {
                    if builder.url.is_empty() {
                        builder.url = token;
                    }
                }
            }
        }

        if !pending_data.is_empty() {
            builder = builder.body(pending_data.join("&"));
        }

        if let Some(method) = explicit_method {
            builder.method = Some(method);
        } else if saw_head {
            builder.method = Some(Method::HEAD);
        }

        builder.build()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CurlResponse {
    pub status: u16,
    pub headers: BTreeMap<String, String>,
    pub body: Vec<u8>,
}

impl CurlResponse {
    pub fn text(&self) -> Result<String, CurlError> {
        String::from_utf8(self.body.clone()).map_err(CurlError::Utf8)
    }

    pub fn text_lossy(&self) -> Cow<'_, str> {
        String::from_utf8_lossy(&self.body)
    }

    pub fn is_success(&self) -> bool {
        (200..300).contains(&self.status)
    }
}

#[derive(Debug, Clone)]
pub struct CurlExecutor {
    client: reqwest::blocking::Client,
    pub enable_debug_log: bool,
}

impl Default for CurlExecutor {
    fn default() -> Self {
        let client = reqwest::blocking::Client::builder()
            .connect_timeout(Duration::from_secs(30))
            .timeout(Duration::from_secs(30))
            .build()
            .expect("blocking reqwest client should build");

        Self {
            client,
            enable_debug_log: false,
        }
    }
}

impl CurlExecutor {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn execute(&self, curl: impl AsRef<str>) -> Result<CurlResponse, CurlError> {
        let parsed = CurlParser::parse(curl)?;
        self.execute_parsed(&parsed)
    }

    pub fn build_request(
        &self,
        parsed: &ParsedCurl,
    ) -> Result<reqwest::blocking::Request, CurlError> {
        let mut builder = self.client.request(parsed.method.clone(), &parsed.url);

        let skip_content_type = !parsed.form_params.is_empty();
        for (name, value) in &parsed.headers {
            if skip_content_type && name.eq_ignore_ascii_case("content-type") {
                continue;
            }
            builder = builder.header(name, value);
        }

        if !parsed.form_params.is_empty() {
            let form = parsed
                .form_params
                .iter()
                .fold(Form::new(), |form, (name, value)| {
                    form.text(name.clone(), value.clone())
                });
            builder = builder.multipart(form);
        } else if let Some(body) = &parsed.body {
            builder = builder.body(body.clone());
        }

        builder.build().map_err(CurlError::RequestBuild)
    }

    pub fn execute_parsed(&self, parsed: &ParsedCurl) -> Result<CurlResponse, CurlError> {
        let request = self.build_request(parsed)?;
        let response = self.client.execute(request).map_err(CurlError::Execute)?;
        let status = response.status().as_u16();
        let headers = response
            .headers()
            .iter()
            .map(|(name, value)| {
                let value = value
                    .to_str()
                    .map(ToOwned::to_owned)
                    .unwrap_or_else(|_| String::from_utf8_lossy(value.as_bytes()).into_owned());
                (name.as_str().to_owned(), value)
            })
            .collect::<BTreeMap<_, _>>();
        let body = response.bytes().map_err(CurlError::Execute)?.to_vec();

        Ok(CurlResponse {
            status,
            headers,
            body,
        })
    }
}

pub fn generate_mutation_rules(
    payload: impl AsRef<str>,
) -> Result<BTreeMap<String, MutationRule>, CurlError> {
    let value = serde_json::from_str::<Value>(payload.as_ref())?;
    let mut rules = BTreeMap::new();
    collect_mutation_rules(&value, &mut rules);
    Ok(rules)
}

pub fn mutate_payload(
    payload: impl AsRef<str>,
    rules: &BTreeMap<String, MutationRule>,
) -> Result<String, CurlError> {
    let mut value = serde_json::from_str::<Value>(payload.as_ref())?;
    mutate_value(&mut value, rules);
    serde_json::to_string_pretty(&value).map_err(CurlError::Json)
}

pub fn update_payload(command: impl AsRef<str>) -> Result<Option<String>, CurlError> {
    let parsed = CurlParser::parse(command)?;
    let Some(body) = parsed.body else {
        return Ok(None);
    };

    if !looks_like_json(&body) {
        return Ok(None);
    }

    let rules = generate_mutation_rules(&body)?;
    mutate_payload(body, &rules).map(Some)
}

pub fn modify_existing_query_params(url: impl AsRef<str>) -> Result<String, CurlError> {
    let source = url.as_ref();
    let mut parsed = Url::parse(source).map_err(|_| CurlError::InvalidUrl(source.to_owned()))?;
    if parsed.query().is_none() {
        return Ok(parsed.to_string());
    }

    let query_pairs = parsed
        .query_pairs()
        .map(|(key, value)| {
            let key = key.into_owned();
            let lower = key.to_ascii_lowercase();
            let value = if lower.contains("id") || lower.contains("user") {
                "invalid_id_123".to_owned()
            } else if lower.contains("page") || lower.contains("limit") {
                "-1".to_owned()
            } else if lower.contains("status") || lower.contains("type") {
                "invalid_status".to_owned()
            } else {
                format!("modified_{}", value)
            };
            (key, value)
        })
        .collect::<Vec<_>>();

    parsed.query_pairs_mut().clear().extend_pairs(query_pairs);
    Ok(parsed.to_string())
}

fn normalize_command(command: &str) -> String {
    LINE_CONTINUATION_RE.replace_all(command, " ").into_owned()
}

fn parse_method(value: &str) -> Result<Method, CurlError> {
    Method::from_bytes(value.trim().to_ascii_uppercase().as_bytes())
        .map_err(|_| CurlError::InvalidMethod(value.to_owned()))
}

fn normalize_header_name(name: &str) -> String {
    name.trim().to_ascii_lowercase()
}

fn split_header(value: &str) -> Result<(String, String), CurlError> {
    let (name, body) = value
        .split_once(':')
        .ok_or_else(|| CurlError::InvalidHeader(value.to_owned()))?;
    Ok((name.trim().to_owned(), body.trim().to_owned()))
}

fn split_form_field(value: &str) -> Result<(String, String), CurlError> {
    let (name, body) = value
        .split_once('=')
        .ok_or_else(|| CurlError::InvalidFormField(value.to_owned()))?;
    Ok((name.trim().to_owned(), body.trim().to_owned()))
}

fn looks_like_json(value: &str) -> bool {
    let trimmed = value.trim();
    (trimmed.starts_with('{') && trimmed.ends_with('}'))
        || (trimmed.starts_with('[') && trimmed.ends_with(']'))
}

fn extract_query_params(url: &Url) -> BTreeMap<String, String> {
    url.query_pairs()
        .map(|(key, value)| (key.into_owned(), value.into_owned()))
        .collect()
}

fn extract_path_params(url: &Url) -> Vec<String> {
    url.path_segments()
        .into_iter()
        .flatten()
        .filter(|segment| !segment.is_empty())
        .filter(|segment| !is_version_segment(segment))
        .filter(|segment| {
            UUID_LIKE_RE.is_match(segment)
                || NUMERIC_RE.is_match(segment)
                || is_dynamic_segment(segment)
        })
        .map(ToOwned::to_owned)
        .collect()
}

fn is_version_segment(segment: &str) -> bool {
    let Some(rest) = segment.strip_prefix('v') else {
        return false;
    };
    !rest.is_empty() && rest.chars().all(|ch| ch.is_ascii_digit())
}

fn is_dynamic_segment(segment: &str) -> bool {
    let is_token = segment
        .chars()
        .all(|ch| ch.is_ascii_alphanumeric() || ch == '_' || ch == '-');
    let has_letter = segment.chars().any(|ch| ch.is_ascii_alphabetic());
    let has_digit = segment.chars().any(|ch| ch.is_ascii_digit());
    is_token && has_letter && has_digit
}

fn collect_mutation_rules(value: &Value, rules: &mut BTreeMap<String, MutationRule>) {
    match value {
        Value::Object(object) => {
            for (key, value) in object {
                match value {
                    Value::String(_) => {
                        rules.insert(key.clone(), MutationRule::Number);
                    }
                    Value::Number(_) => {
                        rules.insert(key.clone(), MutationRule::String);
                    }
                    Value::Bool(_) | Value::Null => {
                        rules.insert(key.clone(), MutationRule::Null);
                    }
                    _ => collect_mutation_rules(value, rules),
                }
            }
        }
        Value::Array(values) => {
            for value in values {
                collect_mutation_rules(value, rules);
            }
        }
        _ => {}
    }
}

fn mutate_value(value: &mut Value, rules: &BTreeMap<String, MutationRule>) {
    match value {
        Value::Object(object) => {
            for (key, value) in object {
                if let Some(rule) = rules.get(key) {
                    *value = match rule {
                        MutationRule::String => Value::String("mutated_string".to_owned()),
                        MutationRule::Number => Value::Number(0.into()),
                        MutationRule::Null => Value::Null,
                    };
                } else {
                    mutate_value(value, rules);
                }
            }
        }
        Value::Array(values) => {
            for value in values {
                mutate_value(value, rules);
            }
        }
        _ => {}
    }
}
