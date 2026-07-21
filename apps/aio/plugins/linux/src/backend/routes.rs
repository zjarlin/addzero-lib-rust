use axum::{
    Json, Router,
    extract::State,
    response::{IntoResponse, Response},
    routing::{get, post},
};
use az_aio_platform::core::api_error::{ApiError, ApiJson, ApiQuery, ApiResponse, ok_json};
use serde::Deserialize;

use crate::backend::{
    contract::{
        BootstrapPlan, BootstrapPlanRequest, ClientPairingSeed, CONTRACT_VERSION,
        LinuxClientStatusResponse, LinuxDistribution, LinuxProfileSummary, LinuxTarget,
        endpoint_catalog,
    },
    env_notes::{load_environment_setup_catalog, setup_source_summary},
    profile::{adapter_for, supported_profiles},
    time::current_timestamp_millis,
};

#[derive(Clone)]
pub struct LinuxApiState {
    default_install_base_url: String,
}

impl LinuxApiState {
    pub fn new(default_install_base_url: String) -> Self {
        Self {
            default_install_base_url,
        }
    }

    pub fn default_install_base_url(&self) -> &str {
        &self.default_install_base_url
    }
}

pub fn linux_router(state: LinuxApiState) -> Router {
    Router::new()
        .route("/api/linux/status", get(status_handler))
        .route("/api/linux/profiles", get(profiles_handler))
        .route("/api/linux/setup-catalog", get(setup_catalog_handler))
        .route("/api/linux/bootstrap-plan", post(bootstrap_plan_handler))
        .route("/api/linux/bootstrap-script", get(bootstrap_script_handler))
        .with_state(state)
}

async fn status_handler(
    State(_state): State<LinuxApiState>,
) -> Json<ApiResponse<LinuxClientStatusResponse>> {
    let catalog = load_environment_setup_catalog();
    let active_profile = supported_profiles()
        .into_iter()
        .next()
        .unwrap_or_else(ubuntu_fallback_profile);
    ok_json(LinuxClientStatusResponse {
        ok: true,
        contract_version: CONTRACT_VERSION.to_string(),
        mode: "client-plugin-first".to_string(),
        server_cli_phase: "planned-after-client-plugin".to_string(),
        active_profile,
        setup_source: setup_source_summary(&catalog),
        endpoints: endpoint_catalog(),
        updated_at_ms: current_timestamp_millis(),
    })
}

async fn profiles_handler() -> Json<ApiResponse<Vec<LinuxProfileSummary>>> {
    ok_json(supported_profiles())
}

async fn setup_catalog_handler() -> Json<ApiResponse<crate::backend::contract::EnvironmentSetupCatalog>> {
    ok_json(load_environment_setup_catalog())
}

async fn bootstrap_plan_handler(
    ApiJson(request): ApiJson<BootstrapPlanRequest>,
) -> Result<Json<ApiResponse<BootstrapPlan>>, Response> {
    let adapter = adapter_for(request.target.distribution);
    adapter
        .build_plan(request)
        .map(ok_json)
        .map_err(linux_error_response)
}

async fn bootstrap_script_handler(
    State(state): State<LinuxApiState>,
    ApiQuery(query): ApiQuery<BootstrapScriptQuery>,
) -> Result<Response, Response> {
    let distribution = query.distribution.unwrap_or(LinuxDistribution::Ubuntu);
    let request = BootstrapPlanRequest {
        target: LinuxTarget {
            host: query.target_host.unwrap_or_else(|| "unknown".to_string()),
            port: query.port.unwrap_or(22),
            user: query.target_user.unwrap_or_else(|| "ubuntu".to_string()),
            distribution,
        },
        client: ClientPairingSeed {
            client_name: query.client_name.unwrap_or_else(|| "aio".to_string()),
            client_endpoint: query.client_endpoint.unwrap_or_default(),
            pair_token: query.pair_token.unwrap_or_default(),
            public_key: query.public_key,
        },
        install_base_url: query
            .install_base_url
            .unwrap_or_else(|| state.default_install_base_url().to_string()),
    };
    let adapter = adapter_for(distribution);
    let script = adapter
        .build_script(request)
        .map_err(linux_error_response)?;

    Ok((
        [("content-type", "text/x-shellscript; charset=utf-8")],
        script,
    )
        .into_response())
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct BootstrapScriptQuery {
    distribution: Option<LinuxDistribution>,
    target_host: Option<String>,
    target_user: Option<String>,
    port: Option<u16>,
    client_name: Option<String>,
    client_endpoint: Option<String>,
    pair_token: Option<String>,
    public_key: Option<String>,
    install_base_url: Option<String>,
}

fn ubuntu_fallback_profile() -> LinuxProfileSummary {
    LinuxProfileSummary {
        distribution: LinuxDistribution::Ubuntu,
        label: "Ubuntu".to_string(),
        package_manager: "apt".to_string(),
        default_user: "ubuntu".to_string(),
        supported_steps: Vec::new(),
    }
}

fn linux_error_response(error: anyhow::Error) -> Response {
    ApiError::from(error).into_response()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn state_keeps_default_install_base_url() {
        let state = LinuxApiState::new("http://127.0.0.1:18080".to_string());

        // 关键断言：curl 脚本生成必须知道客户端插件所在的默认安装入口。
        assert_eq!(state.default_install_base_url(), "http://127.0.0.1:18080");
    }
}
