use crate::model::ParsedCurl;
use crate::parse::parse_curl;
use reqwest::blocking::multipart::Form;
use std::borrow::Cow;
use std::collections::BTreeMap;
use std::time::Duration;

/// HTTP response returned by [`execute_curl`].
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CurlResponse {
    /// Numeric HTTP status code.
    pub status: u16,
    /// Response headers normalized into plain string values.
    pub headers: BTreeMap<String, String>,
    /// UTF-8 response body cached for debugger inspection.
    pub text: Option<String>,
    /// Raw response body bytes.
    pub body: Vec<u8>,
}

impl CurlResponse {
    pub fn text(&self) -> anyhow::Result<String> {
        match &self.text {
            Some(text) => Ok(text.clone()),
            None => String::from_utf8(self.body.clone())
                .map_err(|source| anyhow::anyhow!("response body is not valid UTF-8: {source}")),
        }
    }

    pub fn text_lossy(&self) -> Cow<'_, str> {
        match &self.text {
            Some(text) => Cow::Borrowed(text.as_str()),
            None => String::from_utf8_lossy(&self.body),
        }
    }

    pub fn is_success(&self) -> bool {
        (200..300).contains(&self.status)
    }
}

/// Blocking executor for parsed or raw curl commands.
#[derive(Clone, Debug)]
pub(crate) struct CurlExecutor {
    client: reqwest::blocking::Client,
}

impl CurlExecutor {
    pub(crate) fn new() -> anyhow::Result<Self> {
        Ok(Self::with_client(default_client()?))
    }

    pub(crate) fn with_client(client: reqwest::blocking::Client) -> Self {
        Self { client }
    }

    pub(crate) fn execute(&self, curl: impl AsRef<str>) -> anyhow::Result<CurlResponse> {
        let parsed = parse_curl(curl)?;
        self.execute_parsed(&parsed)
    }

    pub(crate) fn build_request(
        &self,
        parsed: &ParsedCurl,
    ) -> anyhow::Result<reqwest::blocking::Request> {
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

        builder
            .build()
            .map_err(|source| anyhow::anyhow!("failed to build request: {source}"))
    }

    pub(crate) fn execute_parsed(&self, parsed: &ParsedCurl) -> anyhow::Result<CurlResponse> {
        let request = self.build_request(parsed)?;
        let response = self
            .client
            .execute(request)
            .map_err(|source| anyhow::anyhow!("failed to execute request: {source}"))?;
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
        let body = response
            .bytes()
            .map_err(|source| anyhow::anyhow!("failed to execute request: {source}"))?
            .to_vec();
        let text = String::from_utf8(body.clone()).ok();

        Ok(CurlResponse {
            status,
            headers,
            text,
            body,
        })
    }
}

fn default_client() -> anyhow::Result<reqwest::blocking::Client> {
    let client = reqwest::blocking::Client::builder()
        .connect_timeout(Duration::from_secs(30))
        .timeout(Duration::from_secs(30))
        .build()
        .map_err(|source| anyhow::anyhow!("failed to build default HTTP client: {source}"))?;
    Ok(client)
}

/// Executes a curl command with a default blocking HTTP client.
///
/// Use this when the caller does not need to reuse an HTTP client across
/// multiple requests.
pub fn execute_curl(curl: impl AsRef<str>) -> anyhow::Result<CurlResponse> {
    CurlExecutor::new()?.execute(curl)
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn executor_accepts_injected_blocking_client() {
        let client = reqwest::blocking::Client::builder()
            .user_agent("az-curl-test")
            .build()
            .expect("test client should build");
        let executor = CurlExecutor::with_client(client);
        let parsed = parse_curl("curl https://example.test -H 'x-test: yes'")
            .expect("curl command should parse");

        let request = executor
            .build_request(&parsed)
            .expect("request should build without executing network IO");

        assert_eq!(request.url().as_str(), "https://example.test/");
        assert_eq!(request.headers()["x-test"], "yes");
    }
}
