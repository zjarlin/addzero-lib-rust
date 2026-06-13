use crate::url_params::{extract_path_params, extract_query_params};
use crate::util::{looks_like_json, normalize_header_name};
use az_derive_aliases::{apply, plain_eq};
use reqwest::{Method, Url};
use std::collections::BTreeMap;

/// Parsed HTTP request extracted from a curl command.
#[apply(plain_eq)]
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

    pub(crate) fn finalize(mut self) -> anyhow::Result<Self> {
        if self.url.trim().is_empty() {
            anyhow::bail!("curl command does not contain a URL");
        }

        let parsed_url =
            Url::parse(&self.url).map_err(|_| anyhow::anyhow!("invalid URL `{}`", self.url))?;

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
