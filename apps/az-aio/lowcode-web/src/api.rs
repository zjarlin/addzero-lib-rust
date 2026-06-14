use crate::model::*;
use anyhow::{Context, Result};

const BASE: &str = "http://localhost:8081";

async fn get_json<T: serde::de::DeserializeOwned>(url: &str) -> Result<T> {
    let resp = reqwest::get(url).await.context("GET failed")?;
    resp.json().await.context("JSON parse failed")
}

async fn post_json<B: serde::Serialize, T: serde::de::DeserializeOwned>(url: &str, body: &B) -> Result<T> {
    let resp = reqwest::Client::new()
        .post(url)
        .json(body)
        .send()
        .await
        .context("POST failed")?;
    resp.json().await.context("JSON parse failed")
}

async fn delete_json<T: serde::de::DeserializeOwned>(url: &str) -> Result<T> {
    let resp = reqwest::Client::new()
        .delete(url)
        .send()
        .await
        .context("DELETE failed")?;
    resp.json().await.context("JSON parse failed")
}

// ── Models ────────────────────────────────────────────────────────

pub async fn list_models() -> Result<Vec<MetaModelSummary>> {
    get_json(&format!("{BASE}/api/lowcode/models")).await
}

pub async fn create_model(input: &CreateModelInput) -> Result<MetaModel> {
    post_json(&format!("{BASE}/api/lowcode/models"), input).await
}

pub async fn get_model(id: &str) -> Result<Option<MetaModel>> {
    get_json(&format!("{BASE}/api/lowcode/models/{id}")).await
}

pub async fn delete_model(id: &str) -> Result<serde_json::Value> {
    delete_json(&format!("{BASE}/api/lowcode/models/{id}")).await
}

// ── Fields ────────────────────────────────────────────────────────

pub async fn list_fields(model_id: &str) -> Result<Vec<MetaFieldView>> {
    get_json(&format!("{BASE}/api/lowcode/models/{model_id}/fields")).await
}

pub async fn create_field(model_id: &str, input: &CreateFieldInput) -> Result<serde_json::Value> {
    post_json(&format!("{BASE}/api/lowcode/models/{model_id}/fields"), input).await
}

pub async fn update_field(id: &str, input: &UpdateFieldInput) -> Result<serde_json::Value> {
    let resp = reqwest::Client::new()
        .put(&format!("{BASE}/api/lowcode/fields/{id}"))
        .json(input)
        .send()
        .await
        .context("PUT failed")?;
    resp.json().await.context("JSON parse failed")
}

pub async fn delete_field(id: &str) -> Result<serde_json::Value> {
    delete_json(&format!("{BASE}/api/lowcode/fields/{id}")).await
}

// ── Screens ───────────────────────────────────────────────────────

pub async fn list_screens() -> Result<Vec<AppScreenSummary>> {
    get_json(&format!("{BASE}/api/lowcode/screens")).await
}

pub async fn get_screen(id: &str) -> Result<Option<AppScreen>> {
    get_json(&format!("{BASE}/api/lowcode/screens/{id}")).await
}

pub async fn create_screen(input: &CreateScreenInput) -> Result<AppScreen> {
    post_json(&format!("{BASE}/api/lowcode/screens"), input).await
}

pub async fn delete_screen(id: &str) -> Result<serde_json::Value> {
    delete_json(&format!("{BASE}/api/lowcode/screens/{id}")).await
}

// ── Records ───────────────────────────────────────────────────────

pub async fn list_records(model_id: &str) -> Result<Vec<serde_json::Value>> {
    get_json(&format!("{BASE}/api/lowcode/records/{model_id}")).await
}

pub async fn create_record(model_id: &str, fields: &std::collections::HashMap<String, String>) -> Result<serde_json::Value> {
    let client = reqwest::Client::new();
    let resp = client.post(&format!("{BASE}/api/lowcode/records/{model_id}"))
        .json(fields)
        .send().await.context("POST failed")?;
    resp.json().await.context("JSON parse failed")
}

pub async fn update_record(model_id: &str, id: &str, fields: &std::collections::HashMap<String, String>) -> Result<serde_json::Value> {
    let client = reqwest::Client::new();
    let resp = client.put(&format!("{BASE}/api/lowcode/records/{model_id}/{id}"))
        .json(fields)
        .send().await.context("PUT failed")?;
    resp.json().await.context("JSON parse failed")
}

pub async fn delete_record(model_id: &str, id: &str) -> Result<serde_json::Value> {
    delete_json(&format!("{BASE}/api/lowcode/records/{model_id}/{id}")).await
}
