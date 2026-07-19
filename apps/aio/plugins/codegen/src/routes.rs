//! 代码生成 REST API 与 SSR 表单操作。

use std::{net::SocketAddr, sync::Arc};

use anyhow::{Context, Result, anyhow, bail};
use axum::{
    Json, Router,
    extract::{ConnectInfo, State},
    response::{IntoResponse, Redirect, Response},
    routing::{get, post},
};
use az_aio_platform::core::api_error::{ApiError, ApiForm, ApiJson, ApiResponse, ok_json};
use serde::Deserialize;

use crate::{
    contract::{
        CodegenExecutionTarget, CodegenStatus, GenerateRustFileRequest, GeneratedRustFile,
        RUST_FILES_PATH, RustEnumVariant, RustStructField, RustTypeDefinition, STATUS_PATH,
        UI_ACTION_PATH,
    },
    generator::ClientRustCodegen,
};

/// 代码生成 API 状态。
#[derive(Clone)]
pub struct CodegenApiState {
    generator: Arc<ClientRustCodegen>,
}

impl CodegenApiState {
    /// 使用当前客户机生成服务构造 API 状态。
    pub fn new(generator: ClientRustCodegen) -> Self {
        Self {
            generator: Arc::new(generator),
        }
    }
}

/// 构建代码生成插件路由。
pub fn codegen_router(state: CodegenApiState) -> Router {
    Router::new()
        .route(STATUS_PATH, get(status_handler))
        .route(RUST_FILES_PATH, post(generate_rust_file_handler))
        .route(UI_ACTION_PATH, post(ui_action_handler))
        .with_state(state)
}

async fn status_handler(
    ConnectInfo(peer): ConnectInfo<SocketAddr>,
    State(state): State<CodegenApiState>,
) -> Result<Json<ApiResponse<CodegenStatus>>, ApiError> {
    ensure_local_client(peer).map_err(ApiError::from)?;
    Ok(ok_json(CodegenStatus {
        execution_target: CodegenExecutionTarget::CurrentClient,
        default_target_directory: state.generator.base_directory().display().to_string(),
        supported_kinds: vec!["enum".to_string(), "struct".to_string()],
    }))
}

async fn generate_rust_file_handler(
    ConnectInfo(peer): ConnectInfo<SocketAddr>,
    State(state): State<CodegenApiState>,
    ApiJson(request): ApiJson<GenerateRustFileRequest>,
) -> Result<Json<ApiResponse<GeneratedRustFile>>, ApiError> {
    ensure_local_client(peer).map_err(ApiError::from)?;
    generate_on_client(state.generator, request)
        .await
        .map(ok_json)
        .map_err(ApiError::from)
}

async fn ui_action_handler(
    ConnectInfo(peer): ConnectInfo<SocketAddr>,
    State(state): State<CodegenApiState>,
    ApiForm(form): ApiForm<GenerateRustFileForm>,
) -> Response {
    let result = match ensure_local_client(peer).and_then(|_| request_from_form(form)) {
        Ok(request) => generate_on_client(state.generator, request).await,
        Err(error) => Err(error),
    };
    let route = match result {
        Ok(generated) => format!(
            "/codegen?generated={}",
            urlencoding::encode(&generated.file_path)
        ),
        Err(error) => format!("/codegen?error={}", urlencoding::encode(&error.to_string())),
    };
    let redirect = format!("/?route={}", urlencoding::encode(&route));
    Redirect::to(&redirect).into_response()
}

async fn generate_on_client(
    generator: Arc<ClientRustCodegen>,
    request: GenerateRustFileRequest,
) -> Result<GeneratedRustFile> {
    tokio::task::spawn_blocking(move || generator.generate(request))
        .await
        .context("客户机代码生成任务异常退出")?
}

fn ensure_local_client(peer: SocketAddr) -> Result<()> {
    if peer.ip().is_loopback() {
        return Ok(());
    }
    Err(anyhow!(
        "forbidden: 客户机文件生成只允许通过本机回环地址调用"
    ))
}

#[derive(Debug, Deserialize)]
struct GenerateRustFileForm {
    target_directory: String,
    file_name: Option<String>,
    type_kind: String,
    type_name: String,
    members: String,
    overwrite: Option<String>,
}

fn request_from_form(form: GenerateRustFileForm) -> Result<GenerateRustFileRequest> {
    let definition = match form.type_kind.trim() {
        "enum" => RustTypeDefinition::Enum {
            type_name: form.type_name,
            variants: parse_enum_variants(&form.members)?,
        },
        "struct" => RustTypeDefinition::Struct {
            type_name: form.type_name,
            fields: parse_struct_fields(&form.members)?,
        },
        other => bail!("invalid codegen kind: 不支持的类型 {other}"),
    };
    Ok(GenerateRustFileRequest {
        target_directory: form.target_directory,
        file_name: form.file_name,
        overwrite: form.overwrite.is_some(),
        definition,
    })
}

fn parse_enum_variants(value: &str) -> Result<Vec<RustEnumVariant>> {
    value
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(|line| {
            let Some((name, raw_value)) = line.split_once('=') else {
                return Ok(RustEnumVariant {
                    name: line.to_string(),
                    discriminant: None,
                });
            };
            let discriminant = raw_value
                .trim()
                .parse::<i64>()
                .with_context(|| format!("invalid enum discriminant: {raw_value}"))?;
            Ok(RustEnumVariant {
                name: name.trim().to_string(),
                discriminant: Some(discriminant),
            })
        })
        .collect()
}

fn parse_struct_fields(value: &str) -> Result<Vec<RustStructField>> {
    value
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(|line| {
            let (name, rust_type) = line
                .split_once(':')
                .ok_or_else(|| anyhow!("invalid struct field: 字段必须使用 name: RustType 格式"))?;
            Ok(RustStructField {
                name: name.trim().to_string(),
                rust_type: rust_type.trim().to_string(),
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use axum::{
        body::Body,
        http::{Request, StatusCode},
    };
    use tempfile::TempDir;
    use tower::ServiceExt;

    use super::*;

    #[test]
    fn form_builds_struct_request() -> Result<()> {
        let request = request_from_form(GenerateRustFileForm {
            target_directory: "/tmp/generated".to_string(),
            file_name: None,
            type_kind: "struct".to_string(),
            type_name: "Device".to_string(),
            members: "id: String\nonline: bool".to_string(),
            overwrite: None,
        })?;

        // 关键断言：网页表单和 REST API 必须汇聚到同一结构化操作契约。
        assert!(matches!(
            request.definition,
            RustTypeDefinition::Struct { fields, .. } if fields.len() == 2
        ));
        Ok(())
    }

    #[tokio::test]
    async fn rust_file_api_writes_to_client_directory() -> Result<()> {
        let temp = TempDir::new()?;
        let app = codegen_router(CodegenApiState::new(ClientRustCodegen::new(temp.path())));
        let body = serde_json::to_vec(&GenerateRustFileRequest {
            target_directory: "generated".to_string(),
            file_name: None,
            overwrite: false,
            definition: RustTypeDefinition::Enum {
                type_name: "ConnectionState".to_string(),
                variants: vec![RustEnumVariant {
                    name: "Online".to_string(),
                    discriminant: None,
                }],
            },
        })?;
        let mut request = Request::post(RUST_FILES_PATH)
            .header("content-type", "application/json")
            .body(Body::from(body))?;
        request
            .extensions_mut()
            .insert(ConnectInfo(SocketAddr::from(([127, 0, 0, 1], 18080))));
        let response = app.oneshot(request).await?;

        // 关键断言：REST 操作成功后文件必须出现在当前客户机目录。
        assert_eq!(response.status(), axum::http::StatusCode::OK);
        assert!(temp.path().join("generated/connection_state.rs").exists());
        Ok(())
    }

    #[tokio::test]
    async fn rust_file_api_rejects_remote_peer() -> Result<()> {
        let temp = TempDir::new()?;
        let app = codegen_router(CodegenApiState::new(ClientRustCodegen::new(temp.path())));
        let body = serde_json::to_vec(&GenerateRustFileRequest {
            target_directory: "generated".to_string(),
            file_name: None,
            overwrite: false,
            definition: RustTypeDefinition::Enum {
                type_name: "ConnectionState".to_string(),
                variants: vec![RustEnumVariant {
                    name: "Online".to_string(),
                    discriminant: None,
                }],
            },
        })?;
        let mut request = Request::post(RUST_FILES_PATH)
            .header("content-type", "application/json")
            .body(Body::from(body))?;
        request
            .extensions_mut()
            .insert(ConnectInfo(SocketAddr::from(([192, 168, 1, 20], 18080))));
        let response = app.oneshot(request).await?;

        // 关键断言：云端或局域网请求不能直接获得客户机文件写权限。
        assert_eq!(response.status(), StatusCode::FORBIDDEN);
        assert!(!temp.path().join("generated/connection_state.rs").exists());
        Ok(())
    }
}
