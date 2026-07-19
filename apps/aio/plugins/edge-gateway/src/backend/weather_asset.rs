use anyhow::{bail, Context};
use reqwest::{Client, Url};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

/// Stable callable route for current weather.
pub const WEATHER_CURRENT_ROUTE: &str = "/api/edge-gateway/assets/weather/current";
/// Stable asset id for catalog/API metadata.
pub const WEATHER_CURRENT_ASSET_ID: &str = "edge.weather.current";
const OPEN_METEO_FORECAST_URL: &str = "https://api.open-meteo.com/v1/forecast";
const OPEN_METEO_GEOCODING_URL: &str = "https://geocoding-api.open-meteo.com/v1/search";

/// Callable edge asset metadata.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EdgeCallableAsset {
    pub id: String,
    pub name: String,
    pub route: String,
    pub method: String,
    pub auth_scheme: String,
    pub provider: String,
    pub description: String,
    pub request_example: Value,
    pub metadata: Value,
}

/// Request body for the current weather asset.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WeatherCurrentRequest {
    #[serde(default)]
    pub location: Option<String>,
    #[serde(default)]
    pub latitude: Option<f64>,
    #[serde(default)]
    pub longitude: Option<f64>,
    #[serde(default)]
    pub timezone: Option<String>,
    #[serde(default)]
    pub provider: WeatherProvider,
}

/// Weather provider selector.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WeatherProvider {
    #[default]
    OpenMeteo,
}

/// Normalized current weather response.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WeatherCurrentResponse {
    pub provider: WeatherProvider,
    pub location: Option<String>,
    pub latitude: f64,
    pub longitude: f64,
    pub timezone: String,
    pub observed_at: String,
    pub temperature_celsius: f64,
    pub wind_speed_kmh: f64,
    pub wind_direction_degrees: f64,
    pub weather_code: Option<i64>,
    pub raw: Value,
}

/// Returns metadata for the built-in weather current asset.
pub fn weather_current_asset() -> EdgeCallableAsset {
    EdgeCallableAsset {
        id: WEATHER_CURRENT_ASSET_ID.to_string(),
        name: "Weather Current API".to_string(),
        route: WEATHER_CURRENT_ROUTE.to_string(),
        method: "POST".to_string(),
        auth_scheme: "Bearer".to_string(),
        provider: "open_meteo".to_string(),
        description: "Token-gated edge asset for current weather by city or coordinates.".to_string(),
        request_example: json!({
            "location": "Shanghai",
            "timezone": "Asia/Shanghai"
        }),
        metadata: json!({
            "subtype": "callable_api",
            "asset_type": "edge_callable_asset",
            "route": WEATHER_CURRENT_ROUTE,
            "method": "POST",
            "auth": { "scheme": "bearer", "scope": WEATHER_CURRENT_ROUTE },
            "provider": "open_meteo",
            "request_schema": {
                "location": "string | optional when latitude/longitude are supplied",
                "latitude": "number | optional",
                "longitude": "number | optional",
                "timezone": "string | optional, defaults to auto"
            },
            "response_schema": {
                "temperature_celsius": "number",
                "wind_speed_kmh": "number",
                "weather_code": "number | null"
            },
            "upstream_docs": [
                "https://open-meteo.com/en/docs",
                "https://open-meteo.com/en/docs/geocoding-api"
            ]
        }),
    }
}

/// Queries current weather for a callable asset request.
pub async fn query_current_weather(
    client: &Client,
    request: WeatherCurrentRequest,
) -> anyhow::Result<WeatherCurrentResponse> {
    match request.provider {
        WeatherProvider::OpenMeteo => query_open_meteo_current_weather(client, request).await,
    }
}

async fn query_open_meteo_current_weather(
    client: &Client,
    request: WeatherCurrentRequest,
) -> anyhow::Result<WeatherCurrentResponse> {
    let resolved = resolve_location(client, &request).await?;
    let timezone = request
        .timezone
        .as_deref()
        .filter(|value| !value.trim().is_empty())
        .unwrap_or("auto");
    let mut url = Url::parse(OPEN_METEO_FORECAST_URL).context("parse Open-Meteo forecast URL")?;
    url.query_pairs_mut()
        .append_pair("latitude", &resolved.latitude.to_string())
        .append_pair("longitude", &resolved.longitude.to_string())
        .append_pair(
            "current",
            "temperature_2m,weather_code,wind_speed_10m,wind_direction_10m",
        )
        .append_pair("timezone", timezone);

    let payload: OpenMeteoForecastResponse = client
        .get(url)
        .send()
        .await
        .context("request Open-Meteo forecast failed")?
        .error_for_status()
        .context("Open-Meteo forecast returned error status")?
        .json()
        .await
        .context("decode Open-Meteo forecast response")?;
    let current = payload
        .current
        .as_ref()
        .context("Open-Meteo response did not include current weather")?;

    Ok(WeatherCurrentResponse {
        provider: WeatherProvider::OpenMeteo,
        location: resolved.name,
        latitude: payload.latitude,
        longitude: payload.longitude,
        timezone: payload.timezone.clone(),
        observed_at: current.time.clone(),
        temperature_celsius: current.temperature_2m,
        wind_speed_kmh: current.wind_speed_10m,
        wind_direction_degrees: current.wind_direction_10m,
        weather_code: current.weather_code,
        raw: serde_json::to_value(&payload).context("serialize Open-Meteo response")?,
    })
}

async fn resolve_location(
    client: &Client,
    request: &WeatherCurrentRequest,
) -> anyhow::Result<ResolvedLocation> {
    match (request.latitude, request.longitude) {
        (Some(latitude), Some(longitude)) => Ok(ResolvedLocation {
            name: request.location.clone(),
            latitude,
            longitude,
        }),
        (None, None) => {
            let location = request
                .location
                .as_deref()
                .context("location is required when latitude/longitude are omitted")?;
            geocode_location(client, location).await
        }
        _ => bail!("latitude and longitude must be provided together"),
    }
}

async fn geocode_location(client: &Client, location: &str) -> anyhow::Result<ResolvedLocation> {
    let location = location.trim();
    if location.is_empty() {
        bail!("location must not be empty");
    }
    let mut url = Url::parse(OPEN_METEO_GEOCODING_URL).context("parse Open-Meteo geocoding URL")?;
    url.query_pairs_mut()
        .append_pair("name", location)
        .append_pair("count", "1")
        .append_pair("language", "en")
        .append_pair("format", "json");

    let payload: OpenMeteoGeocodingResponse = client
        .get(url)
        .send()
        .await
        .context("request Open-Meteo geocoding failed")?
        .error_for_status()
        .context("Open-Meteo geocoding returned error status")?
        .json()
        .await
        .context("decode Open-Meteo geocoding response")?;
    let first = payload
        .results
        .and_then(|mut values| values.drain(..).next())
        .with_context(|| format!("location not found: {location}"))?;

    Ok(ResolvedLocation {
        name: Some(first.display_name()),
        latitude: first.latitude,
        longitude: first.longitude,
    })
}

#[derive(Debug)]
struct ResolvedLocation {
    name: Option<String>,
    latitude: f64,
    longitude: f64,
}

#[derive(Debug, Deserialize, Serialize)]
struct OpenMeteoForecastResponse {
    latitude: f64,
    longitude: f64,
    timezone: String,
    current: Option<OpenMeteoCurrentWeather>,
}

#[derive(Debug, Deserialize, Serialize)]
struct OpenMeteoCurrentWeather {
    time: String,
    temperature_2m: f64,
    weather_code: Option<i64>,
    wind_speed_10m: f64,
    wind_direction_10m: f64,
}

#[derive(Debug, Deserialize)]
struct OpenMeteoGeocodingResponse {
    results: Option<Vec<OpenMeteoGeocodingItem>>,
}

#[derive(Debug, Deserialize)]
struct OpenMeteoGeocodingItem {
    name: String,
    latitude: f64,
    longitude: f64,
    country: Option<String>,
    admin1: Option<String>,
}

impl OpenMeteoGeocodingItem {
    fn display_name(&self) -> String {
        [
            Some(self.name.as_str()),
            self.admin1.as_deref(),
            self.country.as_deref(),
        ]
        .into_iter()
        .flatten()
        .filter(|part| !part.trim().is_empty())
        .collect::<Vec<_>>()
        .join(", ")
    }
}

#[cfg(test)]
mod tests {
    use reqwest::Client;

    use super::{resolve_location, weather_current_asset, WeatherCurrentRequest, WeatherProvider, WEATHER_CURRENT_ROUTE};

    #[tokio::test]
    async fn resolve_location_accepts_coordinate_pair_without_network() {
        let resolved = resolve_location(
            &Client::new(),
            &WeatherCurrentRequest {
                location: Some("Shanghai".to_string()),
                latitude: Some(31.2304),
                longitude: Some(121.4737),
                timezone: Some("Asia/Shanghai".to_string()),
                provider: WeatherProvider::OpenMeteo,
            },
        )
        .await
        .unwrap();

        assert_eq!(resolved.name.as_deref(), Some("Shanghai"));
    }

    #[test]
    fn weather_asset_metadata_exposes_callable_route() {
        let asset = weather_current_asset();

        assert_eq!(asset.route, WEATHER_CURRENT_ROUTE);
        assert_eq!(
            asset.metadata.get("subtype").and_then(|value| value.as_str()),
            Some("callable_api")
        );
    }
}
