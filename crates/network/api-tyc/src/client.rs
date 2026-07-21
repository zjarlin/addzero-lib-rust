//! Blocking Tianyancha API client.

use anyhow::{Context, bail};
use reqwest::Url;
use reqwest::blocking::{Client, Response};
use serde::de::DeserializeOwned;

use crate::config::TycConfig;
use crate::constant::{BASE_INFO_URL, SEARCH_URL};
use crate::detail::{CompanyDetailData, CompanyInfoRes};
use crate::headers::{TycCredentials, to_header_map, tyc_headers};
use crate::search::{CompanyData, SearchRes};

/// Blocking client for Tianyancha company search and base-info endpoints.
#[derive(Clone, Debug)]
pub struct TycApi {
    client: Client,
}

impl TycApi {
    /// Creates a client from explicit credentials with default timeout settings.
    pub fn new(credentials: TycCredentials) -> anyhow::Result<Self> {
        Self::with_config(credentials, TycConfig::default())
    }

    /// Creates a client from `TYC_AUTHORIZATION` and `TYC_X_AUTH_TOKEN`.
    pub fn from_env() -> anyhow::Result<Self> {
        Self::new(TycCredentials::from_env()?)
    }

    /// Creates a client from explicit credentials and HTTP config.
    pub fn with_config(credentials: TycCredentials, config: TycConfig) -> anyhow::Result<Self> {
        credentials.validate()?;
        config.validate()?;
        let default_headers = to_header_map(&tyc_headers(&credentials))?;
        let client = Client::builder()
            .connect_timeout(config.connect_timeout)
            .timeout(config.request_timeout)
            .default_headers(default_headers)
            .build()
            .context("failed to build Tianyancha HTTP client")?;
        Ok(Self { client })
    }

    /// Fetches company base information by Tianyancha company ID.
    pub fn get_base_info(&self, company_id: i64) -> anyhow::Result<CompanyDetailData> {
        if company_id <= 0 {
            bail!("invalid Tianyancha company_id: must be greater than zero");
        }
        let company_id = company_id.to_string();
        let encoded = urlencoding::encode(&company_id);
        let url = format!("{BASE_INFO_URL}{encoded}");
        let response: CompanyInfoRes = self.read_json(self.client.get(url).send()?)?;
        Ok(response.data)
    }

    /// Searches companies by name using default pagination and sort options.
    pub fn search_company(&self, company_name: impl AsRef<str>) -> anyhow::Result<CompanyData> {
        self.search_company_with_options(company_name, SearchCompanyOptions::default())
    }

    /// Searches companies by name with explicit pagination and sort options.
    pub fn search_company_with_options(
        &self,
        company_name: impl AsRef<str>,
        options: SearchCompanyOptions,
    ) -> anyhow::Result<CompanyData> {
        let company_name = company_name.as_ref().trim();
        if company_name.is_empty() {
            bail!("invalid Tianyancha search: company_name cannot be blank");
        }
        options.validate()?;

        let encoded = urlencoding::encode(company_name);
        let mut url = Url::parse(&format!("{SEARCH_URL}{encoded}"))
            .context("failed to build Tianyancha search URL")?;
        url.query_pairs_mut()
            .append_pair("pageNum", &options.page_num.to_string())
            .append_pair("pageSize", &options.page_size.to_string())
            .append_pair("sortType", &options.sort_type.to_string());

        let response: SearchRes = self.read_json(self.client.get(url).send()?)?;
        Ok(response.data)
    }

    fn read_json<T: DeserializeOwned>(&self, response: Response) -> anyhow::Result<T> {
        let response = Self::ensure_success(response)?;
        let url = response.url().to_string();
        let bytes = response
            .bytes()
            .with_context(|| format!("failed to read Tianyancha response body from `{url}`"))?;
        serde_json::from_slice(bytes.as_ref())
            .with_context(|| format!("failed to parse Tianyancha JSON response from `{url}`"))
    }

    fn ensure_success(response: Response) -> anyhow::Result<Response> {
        let status = response.status();
        if status.is_success() {
            return Ok(response);
        }

        let url = response.url().to_string();
        let bytes = response.bytes().with_context(|| {
            format!("failed to read Tianyancha error response body from `{url}`")
        })?;
        bail!(
            "Tianyancha request to `{url}` returned HTTP {}: {}",
            status.as_u16(),
            String::from_utf8_lossy(bytes.as_ref())
        )
    }
}

/// Search pagination and sorting options for [`TycApi::search_company_with_options`].
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SearchCompanyOptions {
    /// One-based page number.
    pub page_num: u32,
    /// Page size sent to Tianyancha.
    pub page_size: u32,
    /// Tianyancha sort type. `0` matches the JVM module default.
    pub sort_type: u32,
}

impl Default for SearchCompanyOptions {
    fn default() -> Self {
        Self {
            page_num: 1,
            page_size: 10,
            sort_type: 0,
        }
    }
}

impl SearchCompanyOptions {
    /// Creates search options with an explicit page and size.
    pub const fn new(page_num: u32, page_size: u32) -> Self {
        Self {
            page_num,
            page_size,
            sort_type: 0,
        }
    }

    /// Sets the upstream sort type.
    #[must_use]
    pub const fn sort_type(mut self, value: u32) -> Self {
        self.sort_type = value;
        self
    }

    /// Validates local pagination constraints.
    pub fn validate(&self) -> anyhow::Result<()> {
        if self.page_num == 0 {
            bail!("invalid Tianyancha search: page_num must be greater than zero");
        }
        if self.page_size == 0 {
            bail!("invalid Tianyancha search: page_size must be greater than zero");
        }
        Ok(())
    }
}
