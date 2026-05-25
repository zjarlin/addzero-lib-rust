use crate::error::{CurlError, CurlResult};
use crate::model::ParsedCurl;
use crate::parse_curl;
use az_derive_aliases::{apply, plain_clone_debug, plain_eq};
use reqwest::blocking::multipart::Form;
use std::borrow::Cow;
use std::collections::BTreeMap;
use std::time::Duration;

/// HTTP response returned by [`CurlExecutor`].
#[apply(plain_eq)]
pub struct CurlResponse {
    pub status: u16,
    pub headers: BTreeMap<String, String>,
    pub body: Vec<u8>,
}

impl CurlResponse {
    pub fn text(&self) -> CurlResult<String> {
        String::from_utf8(self.body.clone()).map_err(CurlError::Utf8)
    }

    pub fn text_lossy(&self) -> Cow<'_, str> {
        String::from_utf8_lossy(&self.body)
    }

    pub fn is_success(&self) -> bool {
        (200..300).contains(&self.status)
    }
}

/// Blocking executor for parsed or raw curl commands.
#[apply(plain_clone_debug)]
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

    pub fn execute(&self, curl: impl AsRef<str>) -> CurlResult<CurlResponse> {
        let parsed = parse_curl(curl)?;
        self.execute_parsed(&parsed)
    }

    pub(crate) fn build_request(
        &self,
        parsed: &ParsedCurl,
    ) -> CurlResult<reqwest::blocking::Request> {
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

    pub fn execute_parsed(&self, parsed: &ParsedCurl) -> CurlResult<CurlResponse> {
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
