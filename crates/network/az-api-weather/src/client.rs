//! Blocking client for the 2345 historical-weather API.

use std::time::Duration;

use anyhow::{Context, bail};
use chrono::{Datelike, Local};
use reqwest::blocking::Client;
use reqwest::header::{ACCEPT, ACCEPT_LANGUAGE, COOKIE, REFERER, USER_AGENT};

use crate::city::{Area, AreaType, CityService};
use crate::model::WeatherData;
use crate::parser::{extract_response_html, parse_weather_html, split_date_and_weekday};

/// Default endpoint used by the JVM module and the 2345 website.
pub const DEFAULT_HISTORY_URL: &str = "https://tianqi.2345.com/Pc/GetHistory";

const DEFAULT_REFERER: &str = "https://tianqi.2345.com/wea_history/57073.htm";
const DEFAULT_COOKIE: &str = "positionCityID=71778; positionCityPinyin=luolong; lastProvinceId=20; lastCityId=57073; lastCountyId=57073; lastTownId=-1; lastCountyPinyin=luoyang";
const DEFAULT_BROWSER_UA: &str = "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/111.0.0.0 Safari/537.36";

/// HTTP configuration for [`WeatherApi`].
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WeatherApiConfig {
    /// 2345 history endpoint URL.
    pub history_url: String,
    /// TCP/HTTPS connect timeout.
    pub connect_timeout: Duration,
    /// Total timeout for each request.
    pub request_timeout: Duration,
    /// Referer header.
    pub referer: String,
    /// Cookie header. The endpoint currently expects browser-like context headers.
    pub cookie: String,
    /// User-Agent header.
    pub user_agent: String,
}

impl Default for WeatherApiConfig {
    fn default() -> Self {
        Self {
            history_url: DEFAULT_HISTORY_URL.to_owned(),
            connect_timeout: Duration::from_secs(10),
            request_timeout: Duration::from_secs(20),
            referer: DEFAULT_REFERER.to_owned(),
            cookie: DEFAULT_COOKIE.to_owned(),
            user_agent: DEFAULT_BROWSER_UA.to_owned(),
        }
    }
}

impl WeatherApiConfig {
    /// Creates a builder initialized with browser-compatible defaults.
    pub fn builder() -> WeatherApiConfigBuilder {
        WeatherApiConfigBuilder::default()
    }

    /// Validates URL and timeout fields before building the HTTP client.
    pub fn validate(&self) -> anyhow::Result<()> {
        if self.history_url.trim().is_empty() {
            bail!("invalid weather config: history_url cannot be blank");
        }
        if self.connect_timeout.is_zero() {
            bail!("invalid weather config: connect_timeout cannot be zero");
        }
        if self.request_timeout.is_zero() {
            bail!("invalid weather config: request_timeout cannot be zero");
        }
        Ok(())
    }
}

/// Builder for [`WeatherApiConfig`].
#[derive(Clone, Debug, Default)]
pub struct WeatherApiConfigBuilder {
    config: WeatherApiConfig,
}

impl WeatherApiConfigBuilder {
    /// Sets the 2345 history endpoint URL.
    #[must_use]
    pub fn history_url(mut self, value: impl Into<String>) -> Self {
        self.config.history_url = value.into();
        self
    }

    /// Sets the TCP/HTTPS connect timeout.
    #[must_use]
    pub fn connect_timeout(mut self, value: Duration) -> Self {
        self.config.connect_timeout = value;
        self
    }

    /// Sets the total timeout for each request.
    #[must_use]
    pub fn request_timeout(mut self, value: Duration) -> Self {
        self.config.request_timeout = value;
        self
    }

    /// Sets the Referer header.
    #[must_use]
    pub fn referer(mut self, value: impl Into<String>) -> Self {
        self.config.referer = value.into();
        self
    }

    /// Sets the Cookie header.
    #[must_use]
    pub fn cookie(mut self, value: impl Into<String>) -> Self {
        self.config.cookie = value.into();
        self
    }

    /// Sets the User-Agent header.
    #[must_use]
    pub fn user_agent(mut self, value: impl Into<String>) -> Self {
        self.config.user_agent = value.into();
        self
    }

    /// Builds and validates the configuration.
    pub fn build(self) -> anyhow::Result<WeatherApiConfig> {
        self.config.validate()?;
        Ok(self.config)
    }
}

/// Blocking weather client plus bundled city lookup service.
#[derive(Clone, Debug)]
pub struct WeatherApi {
    client: Client,
    config: WeatherApiConfig,
    city_service: CityService,
}

impl WeatherApi {
    /// Creates a weather client using browser-compatible defaults.
    pub fn new() -> anyhow::Result<Self> {
        Self::with_config(WeatherApiConfig::default())
    }

    /// Creates a weather client from explicit config.
    pub fn with_config(config: WeatherApiConfig) -> anyhow::Result<Self> {
        config.validate()?;
        let client = Client::builder()
            .connect_timeout(config.connect_timeout)
            .timeout(config.request_timeout)
            .build()
            .context("failed to build 2345 weather HTTP client")?;
        Ok(Self {
            client,
            config,
            city_service: CityService,
        })
    }

    /// Searches the bundled city dataset.
    pub fn search_cities(&self, keyword: impl AsRef<str>, area_type: AreaType) -> Vec<Area> {
        self.city_service.search_cities(keyword, area_type)
    }

    /// Searches both domestic and international city datasets.
    pub fn search_all_cities(&self, keyword: impl AsRef<str>) -> Vec<Area> {
        self.city_service.search_all_cities(keyword)
    }

    /// Searches cities by keyword and fetches weather for each match.
    pub fn query_weather(
        &self,
        year: i32,
        month: u32,
        keyword: impl AsRef<str>,
        area_type: AreaType,
    ) -> anyhow::Result<Vec<Vec<WeatherData>>> {
        let endpoint_area_type = area_type.endpoint_code();
        self.search_cities(keyword, area_type)
            .into_iter()
            .map(|area| {
                self.query_weather_by_area_id(year, month, &area.area_code, endpoint_area_type)
            })
            .collect()
    }

    /// Fetches historical weather for an explicit 2345 area ID and endpoint area type.
    pub fn query_weather_by_area_id(
        &self,
        year: i32,
        month: u32,
        area_id: impl AsRef<str>,
        area_type: impl AsRef<str>,
    ) -> anyhow::Result<Vec<WeatherData>> {
        validate_year_month(year, month)?;
        let area_id = area_id.as_ref().trim();
        if area_id.is_empty() {
            bail!("invalid weather query: area_id cannot be blank");
        }
        let area_type = area_type.as_ref().trim();
        if area_type.is_empty() {
            bail!("invalid weather query: area_type cannot be blank");
        }

        let body = self
            .client
            .get(&self.config.history_url)
            .header(ACCEPT, "application/json, text/javascript, */*; q=0.01")
            .header(ACCEPT_LANGUAGE, "zh-CN,zh;q=0.9")
            .header(REFERER, &self.config.referer)
            .header(COOKIE, &self.config.cookie)
            .header(USER_AGENT, &self.config.user_agent)
            .header("X-Requested-With", "XMLHttpRequest")
            .header(
                "sec-ch-ua",
                "\"Google Chrome\";v=\"111\", \"Not(A:Brand\";v=\"8\", \"Chromium\";v=\"111\"",
            )
            .header("sec-ch-ua-mobile", "?0")
            .header("sec-ch-ua-platform", "\"macOS\"")
            .query(&[
                ("areaInfo[areaId]", area_id.to_owned()),
                ("areaInfo[areaType]", area_type.to_owned()),
                ("date[year]", year.to_string()),
                ("date[month]", month.to_string()),
            ])
            .send()
            .context("failed to execute 2345 weather request")?;
        let status = body.status();
        let url = body.url().to_string();
        let response_text = body
            .text()
            .with_context(|| format!("failed to read 2345 weather response body from `{url}`"))?;
        if !status.is_success() {
            bail!(
                "2345 weather request to `{url}` returned HTTP {}: {response_text}",
                status.as_u16()
            );
        }

        let html = extract_response_html(&response_text)?;
        if html.contains("抱歉，暂无") {
            bail!("2345 weather returned no data: {html}");
        }

        let mut data = parse_weather_html(&html)?;
        for row in &mut data {
            let (date, week) = split_date_and_weekday(&row.date);
            row.date = date;
            row.week = week;
            row.area_id = Some(area_id.to_owned());
            row.area_type = Some(area_type.to_owned());
        }
        Ok(data)
    }
}

fn validate_year_month(year: i32, month: u32) -> anyhow::Result<()> {
    if !(1..=12).contains(&month) {
        bail!("invalid weather query: month must be between 1 and 12");
    }
    let today = Local::now().date_naive();
    if year > today.year() || (year == today.year() && month > today.month()) {
        bail!("不能查询未来的天气!");
    }
    Ok(())
}
