use std::{env, time::Duration};

use anyhow::{Context, anyhow};
use az_assets::{
    AiModelProvider, AiModelProviderUpsert, AiProviderKind, Asset, AssetGraph, AssetKind,
    AssetProviderSecret, AssetService, AssetUpsert,
};
use az_desktop_plugin::{DesktopDriveSnapshot, DesktopHostServices, DesktopProviderTestResult};
use az_drive_agent::{ListTrackedOptions, PullRemoteItem, PullRemoteOptions, TrackedItem};
use az_drive_store::{DriveConflict, DriveSyncQueueItem, DriveSyncTaskStatus};
use az_software_catalog::{
    SoftwareCatalogDto, SoftwareCatalogService, SoftwareEntryDto, SoftwareEntryInput,
    SoftwareMetadataDto, SoftwareMetadataFetchInput,
};
use reqwest::{
    Client,
    header::{ACCEPT, AUTHORIZATION, HeaderMap, HeaderValue},
};
use tokio::runtime::{Builder, Runtime};
use uuid::Uuid;

pub struct InProcessHostServices {
    runtime: Runtime,
    assets: AssetService,
    software_catalog: Option<SoftwareCatalogService>,
}

impl InProcessHostServices {
    pub fn new() -> anyhow::Result<Self> {
        let runtime = Builder::new_multi_thread()
            .enable_all()
            .build()
            .context("build desktop host tokio runtime")?;
        let database_url = az_persistence::database_url();
        let secret_master_key = env::var("ADDZERO_SECRET_MASTER_KEY").ok();

        if let Some(url) = database_url.as_deref()
            && is_postgres_database_url(url)
        {
            runtime.block_on(ensure_ai_provider_schema(url))?;
        }

        let assets = runtime.block_on(AssetService::try_attach(
            database_url.as_deref(),
            secret_master_key.as_deref(),
        ));
        let software_catalog = if let Some(url) = database_url.as_deref() {
            runtime.block_on(SoftwareCatalogService::connect(url)).ok()
        } else {
            None
        };

        Ok(Self {
            runtime,
            assets,
            software_catalog,
        })
    }

    fn build_drive_agent(&self) -> std::result::Result<az_drive_agent::DriveAgent, String> {
        self.runtime
            .block_on(az_drive_app::build_agent())
            .map_err(|err| err.to_string())
    }
}

impl DesktopHostServices for InProcessHostServices {
    fn load_drive_snapshot(&self) -> Result<DesktopDriveSnapshot, String> {
        let agent = self.build_drive_agent()?;
        let roots = self
            .runtime
            .block_on(agent.list_roots())
            .map_err(|err| err.to_string())?;
        let hosted = self
            .runtime
            .block_on(agent.status(None))
            .map_err(|err| err.to_string())?;
        let tracked = self
            .runtime
            .block_on(agent.list_tracked(
                None,
                ListTrackedOptions {
                    include_all: true,
                    ..ListTrackedOptions::default()
                },
            ))
            .map_err(|err| err.to_string())?;
        let conflicts = self
            .runtime
            .block_on(agent.conflicts())
            .map_err(|err| err.to_string())?;
        let queue = self
            .runtime
            .block_on(agent.sync_queue(None))
            .map_err(|err| err.to_string())?;
        Ok(DesktopDriveSnapshot {
            roots,
            hosted,
            tracked,
            conflicts,
            queue,
        })
    }

    fn drive_host_path(&self, path: &str) -> Result<String, String> {
        let agent = self.build_drive_agent()?;
        let statuses = self
            .runtime
            .block_on(agent.host_path(path, None, None))
            .map_err(|err| err.to_string())?;
        Ok(format!(
            "hosted {} drive items from {}",
            statuses.len(),
            path
        ))
    }

    fn drive_unhost_path(&self, path: &str) -> Result<String, String> {
        let agent = self.build_drive_agent()?;
        let removed = self
            .runtime
            .block_on(agent.unhost_path(path))
            .map_err(|err| err.to_string())?;
        Ok(format!("unhosted {removed} drive items from {path}"))
    }

    fn drive_sync_once(&self) -> Result<String, String> {
        let agent = self.build_drive_agent()?;
        let statuses = self
            .runtime
            .block_on(agent.sync_once())
            .map_err(|err| err.to_string())?;
        Ok(format!(
            "sync cycle finished with {} hosted entries",
            statuses.len()
        ))
    }

    fn drive_retry_queue(&self) -> Result<String, String> {
        let agent = self.build_drive_agent()?;
        let retried = self
            .runtime
            .block_on(agent.retry_sync_queue())
            .map_err(|err| err.to_string())?;
        Ok(format!("retried {retried} queued drive sync items"))
    }

    fn drive_pull_remote(&self, path: Option<&str>) -> Result<Vec<PullRemoteItem>, String> {
        let agent = self.build_drive_agent()?;
        self.runtime
            .block_on(agent.pull_remote(path, PullRemoteOptions::default()))
            .map_err(|err| err.to_string())
    }

    fn list_tracked(
        &self,
        path: Option<&str>,
        options: ListTrackedOptions,
    ) -> Result<Vec<TrackedItem>, String> {
        let agent = self.build_drive_agent()?;
        self.runtime
            .block_on(agent.list_tracked(path, options))
            .map_err(|err| err.to_string())
    }

    fn drive_conflicts(&self) -> Result<Vec<DriveConflict>, String> {
        let agent = self.build_drive_agent()?;
        self.runtime
            .block_on(agent.conflicts())
            .map_err(|err| err.to_string())
    }

    fn drive_sync_queue(
        &self,
        status: Option<DriveSyncTaskStatus>,
    ) -> Result<Vec<DriveSyncQueueItem>, String> {
        let agent = self.build_drive_agent()?;
        self.runtime
            .block_on(agent.sync_queue(status))
            .map_err(|err| err.to_string())
    }

    fn list_assets(&self, kind: Option<AssetKind>) -> Result<Vec<Asset>, String> {
        self.runtime
            .block_on(self.assets.list_assets(kind))
            .map_err(|err| err.to_string())
    }

    fn asset_graph(&self) -> Result<AssetGraph, String> {
        self.runtime
            .block_on(self.assets.graph())
            .map_err(|err| err.to_string())
    }

    fn upsert_asset(&self, input: AssetUpsert) -> Result<Asset, String> {
        self.runtime
            .block_on(self.assets.upsert_asset(input))
            .map_err(|err| err.to_string())
    }

    fn delete_asset(&self, id: Uuid) -> Result<(), String> {
        self.runtime
            .block_on(self.assets.delete_asset(id))
            .map_err(|err| err.to_string())
    }

    fn list_provider_configs(&self) -> Result<Vec<AiModelProvider>, String> {
        self.runtime
            .block_on(self.assets.list_providers())
            .map_err(|err| err.to_string())
    }

    fn upsert_provider(&self, input: AiModelProviderUpsert) -> Result<AiModelProvider, String> {
        self.runtime
            .block_on(self.assets.upsert_provider(input))
            .map_err(|err| err.to_string())
    }

    fn test_provider(&self, provider: AiProviderKind) -> Result<DesktopProviderTestResult, String> {
        let secret = self
            .runtime
            .block_on(self.assets.provider_secret(provider))
            .map_err(|err| err.to_string())?
            .ok_or_else(|| format!("{} is not configured with an API key", provider.as_str()))?;
        self.runtime
            .block_on(test_provider_connection(secret))
            .map_err(|err| err.to_string())
    }

    fn software_catalog(&self) -> Result<SoftwareCatalogDto, String> {
        let service = self.software_catalog.as_ref().ok_or_else(|| {
            "software catalog is unavailable until a database is configured".to_string()
        })?;
        self.runtime
            .block_on(service.catalog())
            .map_err(|err| err.to_string())
    }

    fn software_save_entry(&self, input: SoftwareEntryInput) -> Result<SoftwareEntryDto, String> {
        let service = self.software_catalog.as_ref().ok_or_else(|| {
            "software catalog is unavailable until a database is configured".to_string()
        })?;
        self.runtime
            .block_on(service.save_entry(input))
            .map_err(|err| err.to_string())
    }

    fn software_fetch_metadata(
        &self,
        input: SoftwareMetadataFetchInput,
    ) -> Result<SoftwareMetadataDto, String> {
        let service = self.software_catalog.as_ref().ok_or_else(|| {
            "software catalog is unavailable until a database is configured".to_string()
        })?;
        self.runtime
            .block_on(service.fetch_metadata(input))
            .map_err(|err| err.to_string())
    }

    fn open_path(&self, path: &str) -> Result<(), String> {
        #[cfg(target_os = "macos")]
        let mut command = std::process::Command::new("open");
        #[cfg(target_os = "linux")]
        let mut command = std::process::Command::new("xdg-open");
        #[cfg(target_os = "windows")]
        let mut command = {
            let mut command = std::process::Command::new("cmd");
            command.arg("/C").arg("start");
            command
        };

        command.arg(path);
        command
            .spawn()
            .map_err(|err| format!("open path {path}: {err}"))?;
        Ok(())
    }
}

async fn test_provider_connection(
    secret: AssetProviderSecret,
) -> anyhow::Result<DesktopProviderTestResult> {
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
        return Ok(DesktopProviderTestResult {
            provider: secret.provider.as_str().to_string(),
            ok: false,
            message: format!("HTTP {}: {}", status.as_u16(), compact_error_message(&body)),
        });
    }

    Ok(DesktopProviderTestResult {
        provider: secret.provider.as_str().to_string(),
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

fn is_postgres_database_url(database_url: &str) -> bool {
    database_url.starts_with("postgres://") || database_url.starts_with("postgresql://")
}

async fn ensure_ai_provider_schema(database_url: &str) -> anyhow::Result<()> {
    let pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(1)
        .connect(database_url)
        .await?;
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS ai_model_providers (
            provider TEXT PRIMARY KEY,
            base_url TEXT,
            default_model TEXT NOT NULL,
            enabled BOOLEAN NOT NULL DEFAULT FALSE,
            key_id TEXT NOT NULL DEFAULT 'default',
            encrypted_api_key TEXT,
            api_key_configured BOOLEAN NOT NULL DEFAULT FALSE,
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        )
        "#,
    )
    .execute(&pool)
    .await?;
    sqlx::query("ALTER TABLE ai_model_providers ADD COLUMN IF NOT EXISTS base_url TEXT")
        .execute(&pool)
        .await?;
    Ok(())
}
