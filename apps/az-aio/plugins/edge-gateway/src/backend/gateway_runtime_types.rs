use serde_json::Value;
use std::collections::BTreeMap;

#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GatewayRunRequest {
    pub entry_route: String,
    pub input: Value,
    pub steps: Vec<GatewayRuntimeStep>,
}

#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GatewayRuntimeStep {
    pub body_preview: String,
    pub capture_path: String,
    pub depends_on: Vec<String>,
    pub headers: BTreeMap<String, String>,
    pub id: String,
    pub input_refs: Vec<String>,
    pub kind: String,
    pub label: String,
    pub method: String,
    pub notes: String,
    pub url: String,
}

#[derive(Clone, Debug, PartialEq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GatewayRunResult {
    pub entry_route: String,
    pub final_result: Option<Value>,
    pub message: String,
    pub ok: bool,
    pub status: String,
    pub steps: Vec<GatewayRunStepResult>,
}

#[derive(Clone, Debug, PartialEq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GatewayRunStepResult {
    pub capture_path: String,
    pub captured: Option<Value>,
    pub duration_ms: u128,
    pub error: Option<String>,
    pub id: String,
    pub label: String,
    pub ok: bool,
    pub request_url: String,
    pub response_body: Value,
    pub response_headers: BTreeMap<String, String>,
    pub status_code: Option<u16>,
}
