use gloo_net::http::Request;
use serde::de::DeserializeOwned;

use crate::bootstrap::ApiResponse;

pub async fn fetch_data<T>(api_base_url: &str, path: &str) -> Result<Vec<T>, String>
where
    T: DeserializeOwned,
{
    let response: ApiResponse<Vec<T>> = fetch_json(api_base_url, path).await?;
    if response.code == 200 {
        Ok(response.data.unwrap_or_default())
    } else {
        Err(response.msg)
    }
}

pub async fn fetch_json<T>(api_base_url: &str, path: &str) -> Result<T, String>
where
    T: DeserializeOwned,
{
    let url = api_url(api_base_url, path);
    let response = Request::get(&url)
        .send()
        .await
        .map_err(|error| format!("GET {url} failed: {error}"))?;
    if !response.ok() {
        return Err(format!("GET {url} returned HTTP {}", response.status()));
    }
    response
        .json::<T>()
        .await
        .map_err(|error| format!("GET {url} returned invalid JSON: {error}"))
}

fn api_url(base: &str, path: &str) -> String {
    let base = base.trim_end_matches('/');
    if base.is_empty() {
        path.to_string()
    } else {
        format!("{base}{path}")
    }
}
