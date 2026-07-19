//! linux SSR 页面状态。

use crate::backend::{
    contract::{
        BootstrapPlan, BootstrapPlanRequest, ClientPairingSeed, EnvironmentSetupCatalog,
        LinuxClientStatusResponse, LinuxDistribution, LinuxTarget,
    },
    env_notes::{load_environment_setup_catalog, setup_source_summary},
    profile::{adapter_for, supported_profiles},
    time::current_timestamp_millis,
};

pub struct LinuxPageSnapshot {
    pub status: LinuxClientStatusResponse,
    pub catalog: EnvironmentSetupCatalog,
    pub plan: Option<BootstrapPlan>,
    pub errors: Vec<String>,
}

pub struct LinuxPlanParams {
    pub host: Option<String>,
    pub port: u16,
    pub user: String,
    pub client_endpoint: String,
    pub install_base_url: String,
    pub pair_token: Option<String>,
    pub public_key: Option<String>,
}

pub fn load_snapshot(params: LinuxPlanParams) -> LinuxPageSnapshot {
    let catalog = load_environment_setup_catalog();
    let profiles = supported_profiles();
    let active_profile = profiles
        .first()
        .cloned()
        .unwrap_or_else(ubuntu_fallback_profile);
    let status = LinuxClientStatusResponse {
        ok: true,
        contract_version: crate::backend::contract::CONTRACT_VERSION.to_string(),
        mode: "client-plugin-first".to_string(),
        server_cli_phase: "planned-after-client-plugin".to_string(),
        active_profile,
        setup_source: setup_source_summary(&catalog),
        endpoints: crate::backend::contract::endpoint_catalog(),
        updated_at_ms: current_timestamp_millis(),
    };

    let mut errors = Vec::new();
    let host = params
        .host
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string);
    let plan = match host {
        Some(host) if !params.install_base_url.trim().is_empty() => {
            match build_plan(&host, params) {
                Ok(value) => Some(value),
                Err(error) => {
                    errors.push(error.to_string());
                    None
                }
            }
        }
        Some(_) if params.install_base_url.trim().is_empty() => {
            errors.push("api_base_url 为空，无法生成远端可访问的安装入口".to_string());
            None
        }
        _ => None,
    };

    LinuxPageSnapshot {
        status,
        catalog,
        plan,
        errors,
    }
}

fn build_plan(host: &str, params: LinuxPlanParams) -> anyhow::Result<BootstrapPlan> {
    let request = BootstrapPlanRequest {
        target: LinuxTarget {
            host: host.trim().to_string(),
            port: params.port,
            user: params.user,
            distribution: LinuxDistribution::Ubuntu,
        },
        client: ClientPairingSeed {
            client_name: "aio".to_string(),
            client_endpoint: params.client_endpoint,
            pair_token: params.pair_token.unwrap_or_default(),
            public_key: params.public_key,
        },
        install_base_url: params.install_base_url,
    };
    adapter_for(LinuxDistribution::Ubuntu).build_plan(request)
}

fn ubuntu_fallback_profile() -> crate::backend::contract::LinuxProfileSummary {
    crate::backend::contract::LinuxProfileSummary {
        distribution: LinuxDistribution::Ubuntu,
        label: "Ubuntu".to_string(),
        package_manager: "apt".to_string(),
        default_user: "ubuntu".to_string(),
        supported_steps: Vec::new(),
    }
}
