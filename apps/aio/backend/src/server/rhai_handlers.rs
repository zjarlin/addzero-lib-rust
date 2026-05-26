use std::collections::BTreeMap;
use std::path::PathBuf;

use axum::{
    Json,
    extract::Path,
    http::{HeaderMap, StatusCode},
};
use az_derive_aliases::{apply, deserialize_debug};
use az_script_engine::script::{ScriptInput, ScriptLang, ScriptOutput};

use crate::server::{ApiError, ApiResult, ensure_auth, services};

#[apply(deserialize_debug)]
pub struct RunRhaiRequest {
    pub source: String,
    #[serde(default)]
    pub vars: BTreeMap<String, serde_json::Value>,
}

#[apply(deserialize_debug)]
pub struct SaveScriptRequest {
    pub name: String,
    pub source: String,
}

pub async fn run_rhai(
    headers: HeaderMap,
    Json(body): Json<RunRhaiRequest>,
) -> ApiResult<Json<serde_json::Value>> {
    let backend = services().await;
    ensure_auth(&backend.admin_auth, &headers)?;

    let output = run_rhai_script(&backend.script_engines, body, 30).await?;

    Ok(Json(serde_json::json!({
        "exit_code": output.exit_code,
        "stdout": output.stdout,
        "stderr": output.stderr,
        "vars": output.vars,
        "duration_ms": output.duration_ms,
    })))
}

pub async fn list_scripts(headers: HeaderMap) -> ApiResult<Json<Vec<String>>> {
    let backend = services().await;
    ensure_auth(&backend.admin_auth, &headers)?;
    let dir = scripts_dir();
    let mut names = Vec::new();
    if let Ok(mut entries) = tokio::fs::read_dir(&dir).await {
        while let Ok(Some(entry)) = entries.next_entry().await {
            if let Some(name) = entry.file_name().to_str() {
                if name.ends_with(".rhai") {
                    names.push(name.strip_suffix(".rhai").unwrap_or(name).to_string());
                }
            }
        }
    }
    names.sort();
    Ok(Json(names))
}

pub async fn get_script(
    headers: HeaderMap,
    Path(name): Path<String>,
) -> ApiResult<Json<serde_json::Value>> {
    let backend = services().await;
    ensure_auth(&backend.admin_auth, &headers)?;
    let path = scripts_dir().join(format!("{name}.rhai"));
    let source = tokio::fs::read_to_string(&path)
        .await
        .map_err(|_| ApiError::not_found("Script not found"))?;
    Ok(Json(serde_json::json!({"name": name, "source": source})))
}

pub async fn save_script(
    headers: HeaderMap,
    Json(body): Json<SaveScriptRequest>,
) -> ApiResult<StatusCode> {
    let backend = services().await;
    ensure_auth(&backend.admin_auth, &headers)?;
    let dir = scripts_dir();
    tokio::fs::create_dir_all(&dir)
        .await
        .map_err(|e| ApiError::internal(e.to_string()))?;
    let path = dir.join(format!("{}.rhai", body.name));
    tokio::fs::write(&path, body.source)
        .await
        .map_err(|e| ApiError::internal(e.to_string()))?;
    Ok(StatusCode::CREATED)
}

pub async fn delete_script(headers: HeaderMap, Path(name): Path<String>) -> ApiResult<StatusCode> {
    let backend = services().await;
    ensure_auth(&backend.admin_auth, &headers)?;
    let path = scripts_dir().join(format!("{name}.rhai"));
    tokio::fs::remove_file(&path)
        .await
        .map_err(|_| ApiError::not_found("Script not found"))?;
    Ok(StatusCode::NO_CONTENT)
}

fn scripts_dir() -> PathBuf {
    std::env::var("AIO_SCRIPTS_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("scripts"))
}

pub async fn eval_rhai_env(
    headers: HeaderMap,
    Json(body): Json<RunRhaiRequest>,
) -> ApiResult<Json<serde_json::Value>> {
    let backend = services().await;
    ensure_auth(&backend.admin_auth, &headers)?;

    let output = run_rhai_script(&backend.script_engines, body, 10).await?;

    let mut env_vars = serde_json::Map::new();
    for (key, value) in &output.vars {
        if key != "_result" {
            env_vars.insert(key.clone(), value.clone());
        }
    }

    Ok(Json(serde_json::json!({
        "vars": env_vars,
        "stdout": output.stdout,
        "stderr": output.stderr,
    })))
}

async fn run_rhai_script(
    registry: &'static dyn az_script_engine::script::ScriptEngineRegistry,
    body: RunRhaiRequest,
    timeout_secs: u64,
) -> ApiResult<ScriptOutput> {
    let Some(engine) = registry.get(ScriptLang::Rhai) else {
        return Err(ApiError::internal("Rhai script engine is not registered"));
    };
    let input = ScriptInput {
        source: body.source,
        lang: ScriptLang::Rhai,
        vars: body.vars,
        policy: az_sandbox::sandbox::SandboxPolicy::permissive(),
        timeout_secs,
    };

    tokio::task::spawn_blocking(move || engine.run(input))
        .await
        .map_err(|e| ApiError::internal(e.to_string()))
}
