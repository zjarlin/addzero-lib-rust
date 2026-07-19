use crate::backend::gateway_runtime_types::{GatewayRunStepResult, GatewayRuntimeStep};
use anyhow::{Context, Result};
use reqwest::{Response, header::HeaderMap};
use serde_json::Value;
use serde_json_path::JsonPath;
use std::collections::BTreeMap;

pub async fn response_result(
    step: &GatewayRuntimeStep,
    request_url: String,
    response: Response,
) -> Result<GatewayRunStepResult> {
    let status = response.status();
    let headers = collect_response_headers(response.headers());
    let body = response_body(response).await?;
    let captured = capture_value(&step.capture_path, &body)?;
    let error = response_error(status.as_u16(), status.is_success(), &body);

    Ok(GatewayRunStepResult {
        capture_path: step.capture_path.clone(),
        captured,
        duration_ms: 0,
        error,
        id: step.id.clone(),
        label: step.label.clone(),
        ok: status.is_success(),
        request_url,
        response_body: body,
        response_headers: headers,
        status_code: Some(status.as_u16()),
    })
}

pub fn failed_step(
    step: &GatewayRuntimeStep,
    request_url: &str,
    duration_ms: u128,
    error: String,
) -> GatewayRunStepResult {
    GatewayRunStepResult {
        capture_path: step.capture_path.clone(),
        captured: None,
        duration_ms,
        error: Some(error),
        id: step.id.clone(),
        label: step.label.clone(),
        ok: false,
        request_url: request_url.to_string(),
        response_body: Value::Null,
        response_headers: BTreeMap::new(),
        status_code: None,
    }
}

pub fn final_step_value(step: &GatewayRunStepResult) -> Value {
    step.captured
        .clone()
        .unwrap_or_else(|| step.response_body.clone())
}

fn collect_response_headers(headers: &HeaderMap) -> BTreeMap<String, String> {
    headers
        .iter()
        .map(|(key, value)| {
            (
                key.as_str().to_string(),
                value.to_str().unwrap_or("").to_string(),
            )
        })
        .collect()
}

fn capture_value(path: &str, body: &Value) -> Result<Option<Value>> {
    if path.trim().is_empty() {
        return Ok(None);
    }
    let query = JsonPath::parse(path).with_context(|| format!("invalid JSONPath: {path}"))?;
    let nodes = query.query(body).all();
    Ok(match nodes.as_slice() {
        [] => None,
        [single] => Some((*single).clone()),
        many => Some(Value::Array(
            many.iter().map(|value| (*value).clone()).collect(),
        )),
    })
}

async fn response_body(response: Response) -> Result<Value> {
    let text = response.text().await.context("read response body failed")?;
    if text.trim().is_empty() {
        return Ok(Value::Null);
    }
    Ok(serde_json::from_str(&text).unwrap_or(Value::String(text)))
}

fn response_error(status: u16, success: bool, body: &Value) -> Option<String> {
    if success {
        return None;
    }
    Some(format!("HTTP {status}: {}", compact_value(body)))
}

fn compact_value(value: &Value) -> String {
    match value {
        Value::String(text) => text.chars().take(240).collect(),
        other => other.to_string().chars().take(240).collect(),
    }
}
