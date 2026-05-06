use std::path::PathBuf;

use aio_engine::script::ScriptEngine;
use axum::{
    Json,
    extract::Path,
    http::{HeaderMap, StatusCode},
};
use serde::Deserialize;

use crate::server::{ApiError, ApiResult, ensure_auth, services};

#[derive(Deserialize)]
pub struct SaveScriptRequest {
    pub name: String,
    pub source: String,
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

// ─── Rhai Env Config ──────────────────────────────────────────────────

use crate::server::RunRhaiRequest;

pub async fn eval_rhai_env(
    headers: HeaderMap,
    Json(body): Json<RunRhaiRequest>,
) -> ApiResult<Json<serde_json::Value>> {
    let backend = services().await;
    ensure_auth(&backend.admin_auth, &headers)?;

    let engine = aio_engine_rhai::RhaiEngine::new();
    let input = aio_engine::script::ScriptInput {
        source: body.source,
        lang: aio_engine::script::ScriptLang::Rhai,
        vars: body.vars,
        policy: aio_core::sandbox::SandboxPolicy::permissive(),
        timeout_secs: 10,
    };

    let output = tokio::task::spawn_blocking(move || engine.run(input))
        .await
        .map_err(|e| ApiError::internal(e.to_string()))?;

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
