use crate::model::ParsedCurl;
use crate::parse_support::parse_method;
use crate::normalization::normalize_header_name;
use base64::Engine;
use reqwest::Method;
use std::collections::BTreeMap;

#[derive(Clone, Debug)]
pub(crate) struct CurlBuilder {
    pub(crate) method: Option<Method>,
    pub(crate) url: String,
    headers: BTreeMap<String, String>,
    authorization: Option<String>,
    body: Option<String>,
    form_params: BTreeMap<String, String>,
    content_type: Option<String>,
}

impl CurlBuilder {
    pub(crate) fn new(url: impl Into<String>) -> Self {
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

    pub(crate) fn method(mut self, method: impl AsRef<str>) -> anyhow::Result<Self> {
        self.method = Some(parse_method(method.as_ref())?);
        Ok(self)
    }

    pub(crate) fn header(mut self, name: impl AsRef<str>, value: impl Into<String>) -> Self {
        self.headers
            .insert(normalize_header_name(name.as_ref()), value.into());
        self
    }

    pub(crate) fn basic_auth(mut self, user: impl AsRef<str>, password: impl AsRef<str>) -> Self {
        let token = format!("{}:{}", user.as_ref(), password.as_ref());
        let encoded = base64::engine::general_purpose::STANDARD.encode(token);
        let header_value = format!("Basic {encoded}");
        self.authorization = Some(header_value.clone());
        self.headers
            .insert("authorization".to_owned(), header_value);
        self
    }

    pub(crate) fn body(mut self, value: impl Into<String>) -> Self {
        self.body = Some(value.into());
        self
    }

    pub(crate) fn form_field(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
        self.form_params.insert(name.into(), value.into());
        self
    }

    pub(crate) fn build(self) -> anyhow::Result<ParsedCurl> {
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
