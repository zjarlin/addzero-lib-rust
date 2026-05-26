use std::{collections::BTreeMap, env, time::Duration};

use anyhow::{Context, anyhow};
use az_aio_plugin_config_center::{
    dotfiles_monitor::scan_dotfiles_status,
    dotfiles_monitor_types::DotfilesMonitorStatus,
    pairing::{PairingLocalInfo, ensure_local_pairing_device_info, local_pairing_info},
    paths::{ConfigCenterPaths, resolve_config_center_paths},
};
use az_aio_plugin_edge_gateway::{
    gateway_runtime::run_gateway_plan,
    gateway_runtime_types::{GatewayRunRequest, GatewayRunResult, GatewayRuntimeStep},
};
use az_aio_plugin_software_center::installer_scanner::{
    InstallerPackage, organize_installers, scan_installers,
};
use az_assets::{AiProviderKind, AssetProviderSecret};
use az_derive_aliases::{apply, deserialize_eq, serialize_eq};
use az_drive_agent::{HostedStatus, ListTrackedOptions, LocalRootState, TrackedItem};
use az_drive_store::{DriveConflict, DriveSyncQueueItem};
use reqwest::{
    Client,
    header::{ACCEPT, AUTHORIZATION, HeaderMap, HeaderValue},
};
use serde_json::Value;

use super::{AiProviderConfigDto, AiProviderConfigUpsertDto, AiProviderKindDto};

#[apply(serialize_eq)]
#[serde(rename_all = "camelCase")]
pub struct ActionResultDto {
    pub message: String,
}

#[apply(serialize_eq)]
#[serde(rename_all = "camelCase")]
pub struct DriveSnapshotDto {
    pub roots: Vec<LocalRootState>,
    pub hosted: Vec<HostedStatus>,
    pub tracked: Vec<TrackedItem>,
    pub conflicts: Vec<DriveConflict>,
    pub queue: Vec<DriveSyncQueueItem>,
}

#[apply(deserialize_eq)]
#[serde(rename_all = "camelCase")]
pub struct DrivePathRequestDto {
    pub path: String,
}

#[apply(serialize_eq)]
#[serde(rename_all = "camelCase")]
pub struct ConfigLocalStatusDto {
    pub dotfiles: DotfilesMonitorStatus,
    pub pairing: PairingLocalInfo,
    pub xdg_paths: ConfigCenterPaths,
    pub providers: Vec<AiProviderConfigDto>,
}

#[apply(deserialize_eq)]
#[serde(rename_all = "camelCase")]
pub struct ProviderTestRequestDto {
    pub provider: AiProviderKindDto,
}

#[apply(serialize_eq)]
#[serde(rename_all = "camelCase")]
pub struct ProviderTestResultDto {
    pub provider: AiProviderKindDto,
    pub ok: bool,
    pub message: String,
}

pub async fn load_drive_snapshot_on_server() -> Result<DriveSnapshotDto, String> {
    let agent = build_drive_agent().await?;
    let roots = agent.list_roots().await.map_err(|err| err.to_string())?;
    let hosted = agent.status(None).await.map_err(|err| err.to_string())?;
    let tracked = agent
        .list_tracked(
            None,
            ListTrackedOptions {
                include_all: true,
                ..ListTrackedOptions::default()
            },
        )
        .await
        .map_err(|err| err.to_string())?;
    let conflicts = agent.conflicts().await.map_err(|err| err.to_string())?;
    let queue = agent
        .sync_queue(None)
        .await
        .map_err(|err| err.to_string())?;
    Ok(DriveSnapshotDto {
        roots,
        hosted,
        tracked,
        conflicts,
        queue,
    })
}

pub async fn drive_host_path_on_server(path: String) -> Result<ActionResultDto, String> {
    let agent = build_drive_agent().await?;
    let statuses = agent
        .host_path(&path, None, None)
        .await
        .map_err(|err| err.to_string())?;
    Ok(ActionResultDto {
        message: format!("hosted {} drive items from {path}", statuses.len()),
    })
}

pub async fn drive_unhost_path_on_server(path: String) -> Result<ActionResultDto, String> {
    let agent = build_drive_agent().await?;
    let removed = agent
        .unhost_path(&path)
        .await
        .map_err(|err| err.to_string())?;
    Ok(ActionResultDto {
        message: format!("unhosted {removed} drive items from {path}"),
    })
}

pub async fn drive_sync_once_on_server() -> Result<ActionResultDto, String> {
    let agent = build_drive_agent().await?;
    let statuses = agent.sync_once().await.map_err(|err| err.to_string())?;
    Ok(ActionResultDto {
        message: format!("sync cycle finished with {} hosted entries", statuses.len()),
    })
}

pub async fn drive_retry_queue_on_server() -> Result<ActionResultDto, String> {
    let agent = build_drive_agent().await?;
    let retried = agent
        .retry_sync_queue()
        .await
        .map_err(|err| err.to_string())?;
    Ok(ActionResultDto {
        message: format!("retried {retried} queued drive sync items"),
    })
}

pub async fn drive_queue_on_server() -> Result<Vec<DriveSyncQueueItem>, String> {
    let agent = build_drive_agent().await?;
    agent.sync_queue(None).await.map_err(|err| err.to_string())
}

pub async fn drive_conflicts_on_server() -> Result<Vec<DriveConflict>, String> {
    let agent = build_drive_agent().await?;
    agent.conflicts().await.map_err(|err| err.to_string())
}

pub async fn drive_tracked_roots_on_server() -> Result<Vec<TrackedItem>, String> {
    let agent = build_drive_agent().await?;
    agent
        .list_tracked(
            None,
            ListTrackedOptions {
                include_all: true,
                ..ListTrackedOptions::default()
            },
        )
        .await
        .map_err(|err| err.to_string())
}

pub fn gateway_example_plan() -> GatewayRunRequest {
    GatewayRunRequest {
        entry_route: "/edge/session-proxy".to_string(),
        input: Value::Null,
        steps: vec![GatewayRuntimeStep {
            body_preview: String::new(),
            capture_path: "$.headers.host".to_string(),
            depends_on: Vec::new(),
            headers: BTreeMap::new(),
            id: "ping".to_string(),
            input_refs: Vec::new(),
            kind: "curl".to_string(),
            label: "GET postman echo".to_string(),
            method: "GET".to_string(),
            notes: "Reference flow".to_string(),
            url: "https://postman-echo.com/get?source=aio-react".to_string(),
        }],
    }
}

pub async fn run_gateway_plan_on_server(
    request: GatewayRunRequest,
) -> Result<GatewayRunResult, String> {
    run_gateway_plan(request)
        .await
        .map_err(|err| err.to_string())
}

pub async fn scan_installers_on_server() -> Result<Vec<InstallerPackage>, String> {
    tokio::task::spawn_blocking(scan_installers)
        .await
        .map_err(|err| err.to_string())?
        .map_err(|err| err.to_string())
}

pub async fn organize_installers_on_server() -> Result<Vec<InstallerPackage>, String> {
    tokio::task::spawn_blocking(organize_installers)
        .await
        .map_err(|err| err.to_string())?
        .map_err(|err| err.to_string())
}

pub async fn load_config_local_status_on_server() -> Result<ConfigLocalStatusDto, String> {
    let dotfiles = tokio::task::spawn_blocking(|| {
        ensure_local_pairing_device_info()?;
        let dotfiles = scan_dotfiles_status()?;
        let pairing = local_pairing_info()?;
        let xdg_paths = resolve_config_center_paths()?;
        anyhow::Ok((dotfiles, pairing, xdg_paths))
    })
    .await
    .map_err(|err| err.to_string())?
    .map_err(|err| err.to_string())?;
    let providers = crate::services::ai_chat::list_provider_configs_on_server().await?;
    Ok(ConfigLocalStatusDto {
        dotfiles: dotfiles.0,
        pairing: dotfiles.1,
        xdg_paths: dotfiles.2,
        providers,
    })
}

pub async fn import_env_providers_on_server() -> Result<ActionResultDto, String> {
    let candidates = [
        (
            AiProviderKindDto::OpenAi,
            env::var("OPENAI_API_KEY").ok(),
            env::var("OPENAI_BASE_URL").ok(),
        ),
        (
            AiProviderKindDto::Anthropic,
            env::var("ANTHROPIC_API_KEY").ok(),
            env::var("ANTHROPIC_BASE_URL").ok(),
        ),
        (
            AiProviderKindDto::Gemini,
            env::var("GEMINI_API_KEY")
                .ok()
                .or_else(|| env::var("GOOGLE_API_KEY").ok()),
            env::var("GEMINI_BASE_URL").ok(),
        ),
    ];

    let mut imported = 0;
    for (provider, api_key, base_url) in candidates {
        let Some(api_key) = api_key.filter(|value| !value.trim().is_empty()) else {
            continue;
        };
        crate::services::ai_chat::upsert_provider_config_on_server(AiProviderConfigUpsertDto {
            provider,
            base_url: base_url.filter(|value| !value.trim().is_empty()),
            default_model: az_ai_agent::default_model_for(provider.into()).to_string(),
            enabled: true,
            api_key: Some(api_key),
        })
        .await?;
        imported += 1;
    }

    Ok(ActionResultDto {
        message: format!("imported {imported} provider configs from env"),
    })
}

pub async fn test_provider_on_server(
    provider: AiProviderKindDto,
) -> Result<ProviderTestResultDto, String> {
    let backend = crate::server::services().await;
    let secret = backend
        .assets
        .provider_secret(provider.into())
        .await
        .map_err(|err| err.to_string())?
        .ok_or_else(|| format!("{} is not configured with an API key", provider))?;
    test_provider_connection(provider, secret)
        .await
        .map_err(|err| err.to_string())
}

async fn build_drive_agent() -> Result<az_drive_agent::DriveAgent, String> {
    az_drive_app::build_agent()
        .await
        .map_err(|err| err.to_string())
}

async fn test_provider_connection(
    provider: AiProviderKindDto,
    secret: AssetProviderSecret,
) -> anyhow::Result<ProviderTestResultDto> {
    let client = Client::builder()
        .timeout(Duration::from_secs(15))
        .build()
        .context("create model provider http client")?;
    let endpoint = provider_endpoint(&secret)?;
    let response = client
        .get(endpoint)
        .headers(provider_headers(&secret)?)
        .send()
        .await
        .with_context(|| format!("request {} provider endpoint", secret.provider.as_str()))?;
    let status = response.status();
    let body = response
        .text()
        .await
        .context("read provider response body")?;

    if !status.is_success() {
        return Ok(ProviderTestResultDto {
            provider,
            ok: false,
            message: format!("HTTP {}: {}", status.as_u16(), compact_error_message(&body)),
        });
    }

    Ok(ProviderTestResultDto {
        provider,
        ok: true,
        message: format!(
            "connected with model {} via {}",
            secret.default_model,
            secret
                .base_url
                .clone()
                .unwrap_or_else(|| default_base_url(secret.provider).to_string())
        ),
    })
}

fn provider_endpoint(secret: &AssetProviderSecret) -> anyhow::Result<reqwest::Url> {
    let base_url = secret
        .base_url
        .clone()
        .unwrap_or_else(|| default_base_url(secret.provider).to_string());
    let mut url = reqwest::Url::parse(base_url.trim_end_matches('/'))
        .context("provider base url is invalid")?;
    {
        let mut segments = url
            .path_segments_mut()
            .map_err(|_| anyhow!("provider base url cannot accept path segments"))?;
        segments.pop_if_empty().push("models");
        if !secret.default_model.trim().is_empty() {
            segments.push(&secret.default_model);
        }
    }
    Ok(url)
}

fn provider_headers(secret: &AssetProviderSecret) -> anyhow::Result<HeaderMap> {
    let mut headers = HeaderMap::new();
    headers.insert(ACCEPT, HeaderValue::from_static("application/json"));

    match secret.provider {
        AiProviderKind::OpenAi | AiProviderKind::Gemini => {
            let value = format!("Bearer {}", secret.api_key);
            headers.insert(
                AUTHORIZATION,
                HeaderValue::from_str(&value).context("api key contains invalid characters")?,
            );
        }
        AiProviderKind::Anthropic => {
            headers.insert(
                "x-api-key",
                HeaderValue::from_str(&secret.api_key)
                    .context("anthropic api key contains invalid characters")?,
            );
            headers.insert("anthropic-version", HeaderValue::from_static("2023-06-01"));
        }
    }

    Ok(headers)
}

fn compact_error_message(body: &str) -> String {
    serde_json::from_str::<serde_json::Value>(body)
        .ok()
        .and_then(|value| {
            value
                .pointer("/error/message")
                .or_else(|| value.get("message"))
                .and_then(|value| value.as_str())
                .map(str::to_string)
        })
        .unwrap_or_else(|| body.chars().take(240).collect())
}

fn default_base_url(provider: AiProviderKind) -> &'static str {
    match provider {
        AiProviderKind::OpenAi => "https://api.openai.com/v1",
        AiProviderKind::Anthropic => "https://api.anthropic.com/v1",
        AiProviderKind::Gemini => "https://generativelanguage.googleapis.com/v1beta/openai",
    }
}

#[cfg(test)]
mod tests {
    use super::{gateway_example_plan, load_drive_snapshot_on_server};

    #[test]
    fn gateway_example_keeps_reference_step() {
        let plan = gateway_example_plan();

        assert_eq!(plan.entry_route, "/edge/session-proxy");
        assert_eq!(plan.steps.len(), 1);
        assert_eq!(plan.steps[0].capture_path, "$.headers.host");
    }

    #[tokio::test]
    async fn drive_snapshot_contract_is_callable() {
        let result = load_drive_snapshot_on_server().await;

        assert!(
            result.is_ok() || result.err().is_some_and(|err| !err.trim().is_empty()),
            "drive contract should either return data or a diagnosable setup error"
        );
    }
}
