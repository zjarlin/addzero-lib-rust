#![cfg(not(target_arch = "wasm32"))]

use std::collections::{BTreeMap, BTreeSet};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use az_minio::MinioClient;
use az_plugin_contract::{
    MarketplaceEntry, MarketplaceSnapshot, NavigationItem, NavigationItemKind, NavigationSection,
    PageScope, PluginCliCommand, PluginDescriptor, PluginInstance, PluginInstanceConfig,
    PluginKind, PluginMetadata, PluginPackageManifest, PluginStatus, ResolvedPage, RuntimeBinding,
};
use chrono::{DateTime, Utc};
use serde_json::Value;
use sha2::{Digest, Sha256};
use sqlx::{PgPool, Row};
use thiserror::Error;
use uuid::Uuid;

use super::minio_files::minio_environment_from_env;

const WASM_PLUGIN_OBJECT_PREFIX: &str = "plugins/wasm";

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub enum WasmFirmwareKind {
    System,
    #[default]
    Business,
}

impl WasmFirmwareKind {
    pub fn as_db_value(&self) -> &'static str {
        match self {
            Self::System => "system",
            Self::Business => "business",
        }
    }
}

#[derive(Debug, Error)]
pub enum WasmPluginStoreError {
    #[error("PostgreSQL is not configured for wasm plugin storage")]
    DatabaseUnavailable,
    #[error("MinIO is not configured for wasm plugin binary storage: {0}")]
    MinioUnavailable(String),
    #[error("plugin package is invalid: {0}")]
    InvalidPackage(String),
    #[error("plugin `{0}` is not installed")]
    NotInstalled(String),
    #[error("plugin instance `{0}` was not found")]
    InstanceNotFound(String),
    #[error("plugin `{plugin_id}` page `{page_id}` was not found")]
    PageNotFound { plugin_id: String, page_id: String },
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),
    #[error(transparent)]
    Serde(#[from] serde_json::Error),
    #[error(transparent)]
    Zip(#[from] zip::result::ZipError),
    #[error(transparent)]
    Io(#[from] std::io::Error),
}

pub type WasmPluginStoreResult<T> = Result<T, WasmPluginStoreError>;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct WasmPluginCliResourceUpload {
    pub command_name: String,
    pub file_name: String,
    pub bytes: Vec<u8>,
    pub content_type: Option<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct InstalledPluginCliCommand {
    pub command_name: String,
    pub install_path: String,
    pub object_key: String,
}

#[derive(Clone)]
pub struct WasmPluginStore {
    pool: PgPool,
    storage: WasmPluginBinaryStorage,
}

#[derive(Clone)]
struct WasmPluginBinaryStorage {
    client: MinioClient,
    bucket: String,
}

impl WasmPluginBinaryStorage {
    async fn put_object_bytes(
        &self,
        object_key: &str,
        bytes: Vec<u8>,
        content_type: Option<String>,
    ) -> WasmPluginStoreResult<()> {
        let client = self.client.clone();
        let bucket = self.bucket.clone();
        let object_key = object_key.to_string();
        let object_key_for_error = object_key.clone();
        tokio::task::spawn_blocking(move || {
            client
                .put_object_bytes(&bucket, &object_key, &bytes, content_type.as_deref())
                .map(|_| ())
                .map_err(|err| WasmPluginStoreError::MinioUnavailable(err.to_string()))
        })
        .await
        .map_err(|err| {
            WasmPluginStoreError::MinioUnavailable(format!(
                "MinIO upload task failed for `{object_key_for_error}`: {err}"
            ))
        })?
    }

    async fn get_object(&self, bucket: &str, object_key: &str) -> WasmPluginStoreResult<Vec<u8>> {
        let client = self.client.clone();
        let bucket = bucket.to_string();
        let object_key = object_key.to_string();
        let object_key_for_error = object_key.clone();
        tokio::task::spawn_blocking(move || {
            client
                .get_object(&bucket, &object_key)
                .map_err(|err| WasmPluginStoreError::MinioUnavailable(err.to_string()))
        })
        .await
        .map_err(|err| {
            WasmPluginStoreError::MinioUnavailable(format!(
                "MinIO download task failed for `{object_key_for_error}`: {err}"
            ))
        })?
    }
}

impl WasmPluginStore {
    pub async fn connect(database_url: &str) -> WasmPluginStoreResult<Self> {
        let pool = PgPool::connect(database_url).await?;
        ensure_wasm_plugin_schema(&pool).await?;
        let environment = tokio::task::spawn_blocking(|| {
            let environment =
                minio_environment_from_env().map_err(WasmPluginStoreError::MinioUnavailable)?;
            environment
                .client
                .ensure_bucket(&environment.bucket)
                .map_err(|err| WasmPluginStoreError::MinioUnavailable(err.to_string()))?;
            Ok::<_, WasmPluginStoreError>(environment)
        })
        .await
        .map_err(|err| {
            WasmPluginStoreError::MinioUnavailable(format!(
                "init MinIO environment task failed: {err}"
            ))
        })??;
        Ok(Self {
            pool,
            storage: WasmPluginBinaryStorage {
                client: environment.client,
                bucket: environment.bucket,
            },
        })
    }

    pub async fn import_wasm_binary(
        &self,
        file_name: &str,
        bytes: &[u8],
        descriptor: PluginDescriptor,
        default_instance_label: Option<String>,
        cli_resources: Vec<WasmPluginCliResourceUpload>,
    ) -> WasmPluginStoreResult<StoredWasmPluginPackage> {
        if bytes.is_empty() {
            return Err(WasmPluginStoreError::InvalidPackage(
                "wasm upload bytes cannot be empty".to_string(),
            ));
        }
        validate_wasm_magic(bytes)?;
        validate_descriptor(&descriptor)?;
        if descriptor.kind != PluginKind::Business {
            return Err(WasmPluginStoreError::InvalidPackage(
                "advanced bare wasm upload only accepts Business plugin descriptors".to_string(),
            ));
        }

        let binary_sha256 = sha256_hex(bytes);
        let binary_object_key = format!(
            "{}/{}/{}/{}.wasm",
            WASM_PLUGIN_OBJECT_PREFIX, descriptor.id, descriptor.version, binary_sha256,
        );
        self.storage
            .put_object_bytes(
                &binary_object_key,
                bytes.to_vec(),
                Some("application/wasm".to_string()),
            )
            .await?;

        let manifest = PluginPackageManifest {
            descriptor,
            runtime: RuntimeBinding {
                binary_path: binary_object_key.clone(),
                checksum_path: format!("sha256:{binary_sha256}"),
                assets_dir: None,
            },
            default_instance_label,
        };
        self.upsert_package_row(
            &manifest,
            &binary_object_key,
            &binary_sha256,
            bytes.len(),
            "wasm",
            WasmFirmwareKind::Business,
        )
        .await?;
        self.replace_cli_commands(&manifest.descriptor.id, cli_resources)
            .await?;

        Ok(StoredWasmPluginPackage {
            plugin_id: manifest.descriptor.id,
            plugin_name: manifest.descriptor.name,
            version: manifest.descriptor.version,
            source_file_name: file_name.to_string(),
            binary_bucket: self.storage.bucket.clone(),
            binary_object_key,
            binary_sha256,
            binary_size_bytes: bytes.len() as u64,
        })
    }

    pub async fn import_firmware_binary(
        &self,
        file_name: &str,
        bytes: &[u8],
        descriptor: PluginDescriptor,
        default_instance_label: Option<String>,
        firmware_kind: WasmFirmwareKind,
        cli_resources: Vec<WasmPluginCliResourceUpload>,
    ) -> WasmPluginStoreResult<StoredWasmPluginPackage> {
        if bytes.is_empty() {
            return Err(WasmPluginStoreError::InvalidPackage(
                "wasm upload bytes cannot be empty".to_string(),
            ));
        }
        validate_wasm_magic(bytes)?;
        validate_descriptor(&descriptor)?;
        if firmware_kind == WasmFirmwareKind::System && descriptor.kind != PluginKind::System {
            return Err(WasmPluginStoreError::InvalidPackage(
                "system firmware must use a System plugin descriptor".to_string(),
            ));
        }
        if firmware_kind == WasmFirmwareKind::Business && descriptor.kind != PluginKind::Business {
            return Err(WasmPluginStoreError::InvalidPackage(
                "business firmware must use a Business plugin descriptor".to_string(),
            ));
        }

        let binary_sha256 = sha256_hex(bytes);
        let binary_object_key = format!(
            "{}/{}/{}/{}.wasm",
            WASM_PLUGIN_OBJECT_PREFIX, descriptor.id, descriptor.version, binary_sha256,
        );
        self.storage
            .put_object_bytes(
                &binary_object_key,
                bytes.to_vec(),
                Some("application/wasm".to_string()),
            )
            .await?;

        let manifest = PluginPackageManifest {
            descriptor,
            runtime: RuntimeBinding {
                binary_path: binary_object_key.clone(),
                checksum_path: format!("sha256:{binary_sha256}"),
                assets_dir: None,
            },
            default_instance_label,
        };
        self.upsert_package_row(
            &manifest,
            &binary_object_key,
            &binary_sha256,
            bytes.len(),
            "wasm",
            firmware_kind,
        )
        .await?;
        self.replace_cli_commands(&manifest.descriptor.id, cli_resources)
            .await?;

        Ok(StoredWasmPluginPackage {
            plugin_id: manifest.descriptor.id,
            plugin_name: manifest.descriptor.name,
            version: manifest.descriptor.version,
            source_file_name: file_name.to_string(),
            binary_bucket: self.storage.bucket.clone(),
            binary_object_key,
            binary_sha256,
            binary_size_bytes: bytes.len() as u64,
        })
    }

    pub async fn marketplace_snapshot(&self) -> WasmPluginStoreResult<MarketplaceSnapshot> {
        let packages = self.packages().await?;
        let instances = self.instances().await?;
        let mut tags = BTreeSet::new();
        let mut entries = Vec::with_capacity(packages.len());
        for package in packages {
            for tag in &package.descriptor.tags {
                tags.insert(tag.clone());
            }
            let kind = package.descriptor.kind.clone();
            let instance_count = instances
                .iter()
                .filter(|instance| instance.plugin_id == package.descriptor.id)
                .count();
            entries.push(MarketplaceEntry {
                plugin_id: package.descriptor.id,
                name: package.descriptor.name,
                version: package.descriptor.version,
                kind: kind.clone(),
                summary: package.descriptor.summary,
                tags: package.descriptor.tags,
                icon: package.descriptor.icon,
                compatibility: package.descriptor.compatibility,
                capabilities: package.descriptor.capabilities,
                status: if instance_count == 0 {
                    if kind == PluginKind::System {
                        PluginStatus::Installed
                    } else {
                        PluginStatus::Available
                    }
                } else {
                    PluginStatus::Installed
                },
                instances: instance_count,
            });
        }
        entries.sort_by(|left, right| left.name.cmp(&right.name));
        Ok(MarketplaceSnapshot {
            entries,
            tags: tags.into_iter().collect(),
        })
    }

    pub async fn plugin_navigation(&self) -> WasmPluginStoreResult<Vec<NavigationSection>> {
        let packages = self.packages().await?;
        let instances = self.instances().await?;
        let mut sections = Vec::new();

        let mut system_items = Vec::new();
        for package in packages
            .iter()
            .filter(|package| package.descriptor.kind == PluginKind::System)
        {
            for menu in sorted_menus(&package.descriptor.menus) {
                system_items.push(NavigationItem {
                    label: menu.label.clone(),
                    href: format!("/system/{}/{}", package.descriptor.id, menu.page_id),
                    plugin_id: Some(package.descriptor.id.clone()),
                    page_id: Some(menu.page_id.clone()),
                    badge: Some("系统".to_string()),
                    kind: NavigationItemKind::SystemPage,
                });
            }
        }
        if !system_items.is_empty() {
            sections.push(NavigationSection {
                label: "系统插件".to_string(),
                items: system_items,
            });
        }

        let business_packages = packages
            .iter()
            .filter(|package| package.descriptor.kind == PluginKind::Business)
            .map(|package| (package.descriptor.id.as_str(), &package.descriptor))
            .collect::<BTreeMap<_, _>>();
        let mut instance_items = Vec::new();
        for instance in instances {
            let Some(descriptor) = business_packages.get(instance.plugin_id.as_str()) else {
                continue;
            };
            for page_id in &instance.page_ids {
                let label = descriptor
                    .pages
                    .iter()
                    .find(|page| &page.id == page_id)
                    .map(|page| format!("{} / {}", instance.label, page.title))
                    .unwrap_or_else(|| format!("{} / {}", instance.label, page_id));
                instance_items.push(NavigationItem {
                    label,
                    href: format!("/apps/{}/{}", instance.slug, page_id),
                    plugin_id: Some(instance.plugin_id.clone()),
                    page_id: Some(page_id.clone()),
                    badge: Some(descriptor.name.clone()),
                    kind: NavigationItemKind::BusinessInstance,
                });
            }
        }
        if !instance_items.is_empty() {
            sections.push(NavigationSection {
                label: "业务应用".to_string(),
                items: instance_items,
            });
        }

        Ok(sections)
    }

    pub async fn create_instance(
        &self,
        plugin_id: &str,
        label: Option<&str>,
    ) -> WasmPluginStoreResult<PluginInstance> {
        let package = self
            .package(plugin_id)
            .await?
            .ok_or_else(|| WasmPluginStoreError::NotInstalled(plugin_id.to_string()))?;
        if package.descriptor.kind != PluginKind::Business {
            return Err(WasmPluginStoreError::InvalidPackage(
                "system firmware cannot be installed as a business instance".to_string(),
            ));
        }
        let cli_commands = self.install_plugin_cli_commands(plugin_id).await?;
        let label = label
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .or(package.default_instance_label.as_deref())
            .unwrap_or(package.descriptor.name.as_str())
            .to_string();
        let slug = self.unique_slug(&label).await?;
        let page_ids = package
            .descriptor
            .pages
            .iter()
            .map(|page| page.id.clone())
            .collect::<Vec<_>>();
        let tags = package.descriptor.tags.clone();
        let config = PluginInstanceConfig {
            label: label.clone(),
            permissions: vec![format!("plugin:{plugin_id}:instance:{slug}:read")],
            dictionary_namespace: Some(format!("{plugin_id}.{slug}")),
            allowed_origins: cli_commands
                .iter()
                .map(|command| format!("cli:{}", command.install_path))
                .collect(),
        };
        let config_json = serde_json::to_value(&config)?;
        let row = sqlx::query(
            r#"
            INSERT INTO wasm_plugin_instances
                (slug, plugin_id, label, status, page_ids, tags, config)
            VALUES ($1, $2, $3, 'installed', $4, $5, $6)
            RETURNING created_at
            "#,
        )
        .bind(&slug)
        .bind(plugin_id)
        .bind(&label)
        .bind(&page_ids)
        .bind(&tags)
        .bind(config_json)
        .fetch_one(&self.pool)
        .await?;
        let created_at: DateTime<Utc> = row.get("created_at");
        Ok(PluginInstance {
            plugin_id: plugin_id.to_string(),
            plugin_name: package.descriptor.name,
            slug,
            label,
            status: PluginStatus::Installed,
            page_ids,
            tags,
            created_at,
            config,
        })
    }

    pub async fn first_instance_for_plugin(
        &self,
        plugin_id: &str,
    ) -> WasmPluginStoreResult<Option<PluginInstance>> {
        let package_names = self.package_names().await?;
        let row = sqlx::query(
            r#"
            SELECT plugin_id, slug, label, status, page_ids, tags, config, created_at
            FROM wasm_plugin_instances
            WHERE plugin_id = $1
            ORDER BY created_at ASC, slug ASC
            LIMIT 1
            "#,
        )
        .bind(plugin_id)
        .fetch_optional(&self.pool)
        .await?;
        row.map(|row| instance_from_row(row, &package_names))
            .transpose()
    }

    pub async fn resolve_instance_page(
        &self,
        instance_slug: &str,
        page_id: &str,
    ) -> WasmPluginStoreResult<ResolvedPage> {
        let instance = self
            .instance(instance_slug)
            .await?
            .ok_or_else(|| WasmPluginStoreError::InstanceNotFound(instance_slug.to_string()))?;
        let package = self
            .package(&instance.plugin_id)
            .await?
            .ok_or_else(|| WasmPluginStoreError::NotInstalled(instance.plugin_id.clone()))?;
        let page = package
            .descriptor
            .pages
            .iter()
            .find(|page| page.id == page_id)
            .ok_or_else(|| WasmPluginStoreError::PageNotFound {
                plugin_id: package.descriptor.id.clone(),
                page_id: page_id.to_string(),
            })?;
        Ok(ResolvedPage {
            scope: az_plugin_contract::PageScope::Instance,
            plugin_id: package.descriptor.id.clone(),
            plugin_name: package.descriptor.name.clone(),
            page_id: page.id.clone(),
            title: format!("{} · {}", instance.label, page.title),
            subtitle: page.subtitle.clone(),
            breadcrumbs: vec!["业务应用".to_string(), instance.label, page.title.clone()],
            schema: page.schema.clone(),
        })
    }

    pub async fn resolve_system_page(
        &self,
        plugin_id: &str,
        page_id: &str,
    ) -> WasmPluginStoreResult<ResolvedPage> {
        let package = self
            .package(plugin_id)
            .await?
            .ok_or_else(|| WasmPluginStoreError::NotInstalled(plugin_id.to_string()))?;
        if package.descriptor.kind != PluginKind::System {
            return Err(WasmPluginStoreError::PageNotFound {
                plugin_id: plugin_id.to_string(),
                page_id: page_id.to_string(),
            });
        }
        let page = package
            .descriptor
            .pages
            .iter()
            .find(|page| page.id == page_id)
            .ok_or_else(|| WasmPluginStoreError::PageNotFound {
                plugin_id: package.descriptor.id.clone(),
                page_id: page_id.to_string(),
            })?;
        Ok(ResolvedPage {
            scope: PageScope::System,
            plugin_id: package.descriptor.id.clone(),
            plugin_name: package.descriptor.name.clone(),
            page_id: page.id.clone(),
            title: page.title.clone(),
            subtitle: page.subtitle.clone(),
            breadcrumbs: vec![
                "系统插件".to_string(),
                package.descriptor.name.clone(),
                page.title.clone(),
            ],
            schema: page.schema.clone(),
        })
    }

    pub async fn instances(&self) -> WasmPluginStoreResult<Vec<PluginInstance>> {
        let package_names = self.package_names().await?;
        let rows = sqlx::query(
            r#"
            SELECT plugin_id, slug, label, status, page_ids, tags, config, created_at
            FROM wasm_plugin_instances
            ORDER BY created_at ASC, slug ASC
            "#,
        )
        .fetch_all(&self.pool)
        .await?;
        rows.into_iter()
            .map(|row| instance_from_row(row, &package_names))
            .collect()
    }

    async fn upsert_package_row(
        &self,
        manifest: &PluginPackageManifest,
        binary_object_key: &str,
        binary_sha256: &str,
        binary_size_bytes: usize,
        source_format: &str,
        firmware_kind: WasmFirmwareKind,
    ) -> WasmPluginStoreResult<()> {
        let descriptor_json = serde_json::to_value(&manifest.descriptor)?;
        let runtime_json = serde_json::to_value(&manifest.runtime)?;
        let metadata_json = serde_json::to_value(&manifest.descriptor.metadata)?;
        sqlx::query(
            r#"
            INSERT INTO wasm_plugin_packages (
                plugin_id,
                name,
                version,
                summary,
                descriptor,
                runtime,
                metadata,
                github_url,
                description,
                maintainer_type,
                maintainer_name,
                primary_language,
                category,
                install_command,
                agent_install_command,
                default_instance_label,
                binary_bucket,
                binary_object_key,
                binary_sha256,
                binary_size_bytes,
                source_format,
                firmware_kind,
                status
            )
            VALUES (
                $1, $2, $3, $4, $5, $6, $7, $8, $9, $10,
                $11, $12, $13, $14, $15, $16, $17, $18,
                $19, $20, $21, $22, 'available'
            )
            ON CONFLICT (plugin_id) DO UPDATE SET
                name = EXCLUDED.name,
                version = EXCLUDED.version,
                summary = EXCLUDED.summary,
                descriptor = EXCLUDED.descriptor,
                runtime = EXCLUDED.runtime,
                metadata = EXCLUDED.metadata,
                github_url = EXCLUDED.github_url,
                description = EXCLUDED.description,
                maintainer_type = EXCLUDED.maintainer_type,
                maintainer_name = EXCLUDED.maintainer_name,
                primary_language = EXCLUDED.primary_language,
                category = EXCLUDED.category,
                install_command = EXCLUDED.install_command,
                agent_install_command = EXCLUDED.agent_install_command,
                default_instance_label = EXCLUDED.default_instance_label,
                binary_bucket = EXCLUDED.binary_bucket,
                binary_object_key = EXCLUDED.binary_object_key,
                binary_sha256 = EXCLUDED.binary_sha256,
                binary_size_bytes = EXCLUDED.binary_size_bytes,
                source_format = EXCLUDED.source_format,
                firmware_kind = EXCLUDED.firmware_kind,
                status = EXCLUDED.status,
                updated_at = NOW()
            "#,
        )
        .bind(&manifest.descriptor.id)
        .bind(&manifest.descriptor.name)
        .bind(&manifest.descriptor.version)
        .bind(&manifest.descriptor.summary)
        .bind(descriptor_json)
        .bind(runtime_json)
        .bind(metadata_json)
        .bind(manifest.descriptor.metadata.github_url.trim())
        .bind(manifest.descriptor.metadata.description.trim())
        .bind(manifest.descriptor.metadata.maintainer_type.trim())
        .bind(manifest.descriptor.metadata.maintainer_name.trim())
        .bind(manifest.descriptor.metadata.primary_language.trim())
        .bind(manifest.descriptor.metadata.category.trim())
        .bind(manifest.descriptor.metadata.install_command.trim())
        .bind(manifest.descriptor.metadata.agent_install_command.trim())
        .bind(&manifest.default_instance_label)
        .bind(&self.storage.bucket)
        .bind(binary_object_key)
        .bind(binary_sha256)
        .bind(i64::try_from(binary_size_bytes).unwrap_or(i64::MAX))
        .bind(source_format)
        .bind(firmware_kind.as_db_value())
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn packages(&self) -> WasmPluginStoreResult<Vec<StoredPackageRow>> {
        let rows = sqlx::query(
            r#"
            SELECT plugin_id, descriptor, metadata, github_url, description, maintainer_type,
                   maintainer_name, primary_language, category, install_command,
                   agent_install_command, default_instance_label
            FROM wasm_plugin_packages
            WHERE status != 'disabled'
            ORDER BY name ASC
            "#,
        )
        .fetch_all(&self.pool)
        .await?;
        let mut packages = Vec::with_capacity(rows.len());
        for row in rows {
            packages.push(self.package_from_row(row).await?);
        }
        Ok(packages)
    }

    async fn package(&self, plugin_id: &str) -> WasmPluginStoreResult<Option<StoredPackageRow>> {
        let row = sqlx::query(
            r#"
            SELECT plugin_id, descriptor, metadata, github_url, description, maintainer_type,
                   maintainer_name, primary_language, category, install_command,
                   agent_install_command, default_instance_label
            FROM wasm_plugin_packages
            WHERE plugin_id = $1 AND status != 'disabled'
            "#,
        )
        .bind(plugin_id)
        .fetch_optional(&self.pool)
        .await?;
        match row {
            Some(row) => Ok(Some(self.package_from_row(row).await?)),
            None => Ok(None),
        }
    }

    async fn package_from_row(
        &self,
        row: sqlx::postgres::PgRow,
    ) -> WasmPluginStoreResult<StoredPackageRow> {
        let mut package = package_from_row(row)?;
        package.descriptor.cli_commands = self.cli_commands(&package.descriptor.id).await?;
        Ok(package)
    }

    async fn cli_commands(&self, plugin_id: &str) -> WasmPluginStoreResult<Vec<PluginCliCommand>> {
        let rows = sqlx::query(
            r#"
            SELECT command_name, file_name, object_bucket, object_key, object_sha256,
                   object_size_bytes, content_type, install_path, status
            FROM wasm_plugin_cli_commands
            WHERE plugin_id = $1
            ORDER BY command_name ASC
            "#,
        )
        .bind(plugin_id)
        .fetch_all(&self.pool)
        .await?;
        Ok(rows
            .into_iter()
            .map(|row| PluginCliCommand {
                command_name: row.get("command_name"),
                file_name: row.get("file_name"),
                object_bucket: row.get("object_bucket"),
                object_key: row.get("object_key"),
                object_sha256: row.get("object_sha256"),
                object_size_bytes: row.get::<i64, _>("object_size_bytes").max(0) as u64,
                content_type: row.get("content_type"),
                install_path: row.get("install_path"),
                status: row.get("status"),
            })
            .collect())
    }

    async fn instance(&self, slug: &str) -> WasmPluginStoreResult<Option<PluginInstance>> {
        let package_names = self.package_names().await?;
        let row = sqlx::query(
            r#"
            SELECT plugin_id, slug, label, status, page_ids, tags, config, created_at
            FROM wasm_plugin_instances
            WHERE slug = $1
            "#,
        )
        .bind(slug)
        .fetch_optional(&self.pool)
        .await?;
        row.map(|row| instance_from_row(row, &package_names))
            .transpose()
    }

    async fn package_names(&self) -> WasmPluginStoreResult<BTreeMap<String, String>> {
        let rows = sqlx::query("SELECT plugin_id, name FROM wasm_plugin_packages")
            .fetch_all(&self.pool)
            .await?;
        Ok(rows
            .into_iter()
            .map(|row| {
                (
                    row.get::<String, _>("plugin_id"),
                    row.get::<String, _>("name"),
                )
            })
            .collect())
    }

    async fn unique_slug(&self, label: &str) -> WasmPluginStoreResult<String> {
        let base = slugify(label);
        let candidate = if base.is_empty() {
            "plugin-instance".to_string()
        } else {
            base
        };
        let exists: bool = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM wasm_plugin_instances WHERE slug = $1)",
        )
        .bind(&candidate)
        .fetch_one(&self.pool)
        .await?;
        if !exists {
            return Ok(candidate);
        }
        let suffix = Uuid::new_v4()
            .simple()
            .to_string()
            .chars()
            .take(6)
            .collect::<String>();
        Ok(format!("{candidate}-{suffix}"))
    }

    async fn replace_cli_commands(
        &self,
        plugin_id: &str,
        resources: Vec<WasmPluginCliResourceUpload>,
    ) -> WasmPluginStoreResult<()> {
        sqlx::query("DELETE FROM wasm_plugin_cli_commands WHERE plugin_id = $1")
            .bind(plugin_id)
            .execute(&self.pool)
            .await?;
        for resource in resources {
            let command_name = normalized_command_name(&resource.command_name)?;
            let file_name = normalized_file_name(&resource.file_name)?;
            if resource.bytes.is_empty() {
                return Err(WasmPluginStoreError::InvalidPackage(format!(
                    "CLI command `{command_name}` bytes cannot be empty"
                )));
            }
            let digest = sha256_hex(&resource.bytes);
            let object_key = format!(
                "{}/{}/cli/{}/{}",
                WASM_PLUGIN_OBJECT_PREFIX, plugin_id, digest, file_name
            );
            let content_type = resource
                .content_type
                .as_deref()
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .unwrap_or("text/x-shellscript")
                .to_string();
            self.storage
                .put_object_bytes(
                    &object_key,
                    resource.bytes.clone(),
                    Some(content_type.clone()),
                )
                .await?;
            let install_path = default_cli_install_dir().join(&command_name);
            sqlx::query(
                r#"
                INSERT INTO wasm_plugin_cli_commands (
                    plugin_id, command_name, file_name, object_bucket, object_key,
                    object_sha256, object_size_bytes, content_type, install_path, status
                )
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, 'available')
                "#,
            )
            .bind(plugin_id)
            .bind(&command_name)
            .bind(&file_name)
            .bind(&self.storage.bucket)
            .bind(&object_key)
            .bind(&digest)
            .bind(i64::try_from(resource.bytes.len()).unwrap_or(i64::MAX))
            .bind(&content_type)
            .bind(install_path.display().to_string())
            .execute(&self.pool)
            .await?;
        }
        Ok(())
    }

    async fn install_plugin_cli_commands(
        &self,
        plugin_id: &str,
    ) -> WasmPluginStoreResult<Vec<InstalledPluginCliCommand>> {
        let commands = self.cli_commands(plugin_id).await?;
        let install_dir = default_cli_install_dir();
        fs::create_dir_all(&install_dir)?;
        let mut installed = Vec::with_capacity(commands.len());
        for command in commands {
            let bytes = self
                .storage
                .get_object(&command.object_bucket, &command.object_key)
                .await?;
            let digest = sha256_hex(&bytes);
            if digest != command.object_sha256 {
                return Err(WasmPluginStoreError::InvalidPackage(format!(
                    "checksum mismatch for CLI command `{}`",
                    command.command_name
                )));
            }
            let target = install_dir.join(&command.command_name);
            ensure_install_target_is_safe(&install_dir, &target)?;
            fs::write(&target, bytes)?;
            make_executable(&target)?;
            let install_path = target.display().to_string();
            sqlx::query(
                r#"
                UPDATE wasm_plugin_cli_commands
                SET status = 'installed', install_path = $3, installed_at = NOW(), updated_at = NOW()
                WHERE plugin_id = $1 AND command_name = $2
                "#,
            )
            .bind(plugin_id)
            .bind(&command.command_name)
            .bind(&install_path)
            .execute(&self.pool)
            .await?;
            installed.push(InstalledPluginCliCommand {
                command_name: command.command_name,
                install_path,
                object_key: command.object_key,
            });
        }
        Ok(installed)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StoredWasmPluginPackage {
    pub plugin_id: String,
    pub plugin_name: String,
    pub version: String,
    pub source_file_name: String,
    pub binary_bucket: String,
    pub binary_object_key: String,
    pub binary_sha256: String,
    pub binary_size_bytes: u64,
}

struct StoredPackageRow {
    descriptor: PluginDescriptor,
    default_instance_label: Option<String>,
}

fn package_from_row(row: sqlx::postgres::PgRow) -> WasmPluginStoreResult<StoredPackageRow> {
    let descriptor_value: Value = row.get("descriptor");
    let mut descriptor: PluginDescriptor = serde_json::from_value(descriptor_value)?;
    descriptor.metadata = metadata_from_package_row(&row);
    Ok(StoredPackageRow {
        descriptor,
        default_instance_label: row.get("default_instance_label"),
    })
}

fn metadata_from_package_row(row: &sqlx::postgres::PgRow) -> PluginMetadata {
    let mut metadata = row
        .try_get::<Value, _>("metadata")
        .ok()
        .and_then(|value| serde_json::from_value::<PluginMetadata>(value).ok())
        .unwrap_or_default();
    metadata.github_url = row.try_get("github_url").unwrap_or(metadata.github_url);
    metadata.description = row.try_get("description").unwrap_or(metadata.description);
    metadata.maintainer_type = row
        .try_get("maintainer_type")
        .unwrap_or(metadata.maintainer_type);
    metadata.maintainer_name = row
        .try_get("maintainer_name")
        .unwrap_or(metadata.maintainer_name);
    metadata.primary_language = row
        .try_get("primary_language")
        .unwrap_or(metadata.primary_language);
    metadata.category = row.try_get("category").unwrap_or(metadata.category);
    metadata.install_command = row
        .try_get("install_command")
        .unwrap_or(metadata.install_command);
    metadata.agent_install_command = row
        .try_get("agent_install_command")
        .unwrap_or(metadata.agent_install_command);
    metadata
}

async fn ensure_wasm_plugin_schema(pool: &PgPool) -> WasmPluginStoreResult<()> {
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS wasm_plugin_packages (
            plugin_id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            version TEXT NOT NULL,
            summary TEXT NOT NULL,
            descriptor JSONB NOT NULL,
            runtime JSONB NOT NULL DEFAULT '{}'::jsonb,
            default_instance_label TEXT,
            binary_bucket TEXT NOT NULL,
            binary_object_key TEXT NOT NULL,
            binary_sha256 TEXT NOT NULL,
            binary_size_bytes BIGINT NOT NULL,
            source_format TEXT NOT NULL DEFAULT 'wasm',
            firmware_kind TEXT NOT NULL DEFAULT 'business',
            status TEXT NOT NULL DEFAULT 'available',
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        )
        "#,
    )
    .execute(pool)
    .await?;
    for statement in [
        "ALTER TABLE wasm_plugin_packages ADD COLUMN IF NOT EXISTS metadata JSONB NOT NULL DEFAULT '{}'::jsonb",
        "ALTER TABLE wasm_plugin_packages ADD COLUMN IF NOT EXISTS github_url TEXT NOT NULL DEFAULT ''",
        "ALTER TABLE wasm_plugin_packages ADD COLUMN IF NOT EXISTS description TEXT NOT NULL DEFAULT ''",
        "ALTER TABLE wasm_plugin_packages ADD COLUMN IF NOT EXISTS maintainer_type TEXT NOT NULL DEFAULT ''",
        "ALTER TABLE wasm_plugin_packages ADD COLUMN IF NOT EXISTS maintainer_name TEXT NOT NULL DEFAULT ''",
        "ALTER TABLE wasm_plugin_packages ADD COLUMN IF NOT EXISTS primary_language TEXT NOT NULL DEFAULT ''",
        "ALTER TABLE wasm_plugin_packages ADD COLUMN IF NOT EXISTS category TEXT NOT NULL DEFAULT ''",
        "ALTER TABLE wasm_plugin_packages ADD COLUMN IF NOT EXISTS install_command TEXT NOT NULL DEFAULT ''",
        "ALTER TABLE wasm_plugin_packages ADD COLUMN IF NOT EXISTS agent_install_command TEXT NOT NULL DEFAULT ''",
        "ALTER TABLE wasm_plugin_packages ADD COLUMN IF NOT EXISTS firmware_kind TEXT NOT NULL DEFAULT 'business'",
    ] {
        sqlx::query(statement).execute(pool).await?;
    }
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS wasm_plugin_instances (
            slug TEXT PRIMARY KEY,
            plugin_id TEXT NOT NULL REFERENCES wasm_plugin_packages(plugin_id) ON DELETE CASCADE,
            label TEXT NOT NULL,
            status TEXT NOT NULL DEFAULT 'installed',
            page_ids TEXT[] NOT NULL DEFAULT '{}',
            tags TEXT[] NOT NULL DEFAULT '{}',
            config JSONB NOT NULL DEFAULT '{}'::jsonb,
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        )
        "#,
    )
    .execute(pool)
    .await?;
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS wasm_plugin_cli_commands (
            plugin_id TEXT NOT NULL REFERENCES wasm_plugin_packages(plugin_id) ON DELETE CASCADE,
            command_name TEXT NOT NULL,
            file_name TEXT NOT NULL,
            object_bucket TEXT NOT NULL,
            object_key TEXT NOT NULL,
            object_sha256 TEXT NOT NULL,
            object_size_bytes BIGINT NOT NULL,
            content_type TEXT NOT NULL DEFAULT 'text/x-shellscript',
            install_path TEXT NOT NULL,
            status TEXT NOT NULL DEFAULT 'available',
            installed_at TIMESTAMPTZ,
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            PRIMARY KEY (plugin_id, command_name)
        )
        "#,
    )
    .execute(pool)
    .await?;
    for statement in [
        "CREATE INDEX IF NOT EXISTS idx_wasm_plugin_packages_status ON wasm_plugin_packages(status)",
        "CREATE INDEX IF NOT EXISTS idx_wasm_plugin_packages_descriptor ON wasm_plugin_packages USING GIN(descriptor)",
        "CREATE INDEX IF NOT EXISTS idx_wasm_plugin_packages_metadata ON wasm_plugin_packages USING GIN(metadata)",
        "CREATE INDEX IF NOT EXISTS idx_wasm_plugin_packages_firmware_kind ON wasm_plugin_packages(firmware_kind)",
        "CREATE INDEX IF NOT EXISTS idx_wasm_plugin_packages_category ON wasm_plugin_packages(category) WHERE category != ''",
        "CREATE INDEX IF NOT EXISTS idx_wasm_plugin_instances_plugin ON wasm_plugin_instances(plugin_id)",
        "CREATE INDEX IF NOT EXISTS idx_wasm_plugin_instances_status ON wasm_plugin_instances(status)",
        "CREATE INDEX IF NOT EXISTS idx_wasm_plugin_cli_commands_status ON wasm_plugin_cli_commands(status)",
    ] {
        sqlx::query(statement).execute(pool).await?;
    }
    Ok(())
}

fn instance_from_row(
    row: sqlx::postgres::PgRow,
    package_names: &BTreeMap<String, String>,
) -> WasmPluginStoreResult<PluginInstance> {
    let plugin_id: String = row.get("plugin_id");
    let config_value: Value = row.get("config");
    let config: PluginInstanceConfig = serde_json::from_value(config_value)?;
    let status = match row.get::<String, _>("status").as_str() {
        "disabled" => PluginStatus::Disabled,
        _ => PluginStatus::Installed,
    };
    Ok(PluginInstance {
        plugin_name: package_names
            .get(&plugin_id)
            .cloned()
            .unwrap_or_else(|| plugin_id.clone()),
        plugin_id,
        slug: row.get("slug"),
        label: row.get("label"),
        status,
        page_ids: row.get("page_ids"),
        tags: row.get("tags"),
        created_at: row.get("created_at"),
        config,
    })
}

fn sha256_hex(bytes: &[u8]) -> String {
    let mut digest = Sha256::new();
    digest.update(bytes);
    format!("{:x}", digest.finalize())
}

fn validate_wasm_magic(bytes: &[u8]) -> WasmPluginStoreResult<()> {
    if bytes.len() < 8 || &bytes[0..4] != b"\0asm" {
        return Err(WasmPluginStoreError::InvalidPackage(
            "uploaded file is not a WebAssembly module".to_string(),
        ));
    }
    Ok(())
}

fn validate_descriptor(descriptor: &PluginDescriptor) -> WasmPluginStoreResult<()> {
    for (field, value) in [
        ("descriptor.id", descriptor.id.as_str()),
        ("descriptor.name", descriptor.name.as_str()),
        ("descriptor.version", descriptor.version.as_str()),
        ("descriptor.summary", descriptor.summary.as_str()),
    ] {
        if value.trim().is_empty() {
            return Err(WasmPluginStoreError::InvalidPackage(format!(
                "{field} cannot be empty"
            )));
        }
    }
    if descriptor.pages.is_empty() {
        return Err(WasmPluginStoreError::InvalidPackage(
            "descriptor.pages cannot be empty".to_string(),
        ));
    }
    Ok(())
}

fn normalized_command_name(raw: &str) -> WasmPluginStoreResult<String> {
    let value = raw.trim();
    if value.is_empty() {
        return Err(WasmPluginStoreError::InvalidPackage(
            "CLI command name cannot be empty".to_string(),
        ));
    }
    if !value
        .chars()
        .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_'))
    {
        return Err(WasmPluginStoreError::InvalidPackage(format!(
            "CLI command `{value}` must contain only ASCII letters, digits, dash, or underscore"
        )));
    }
    Ok(value.to_string())
}

fn normalized_file_name(raw: &str) -> WasmPluginStoreResult<String> {
    let value = raw.trim();
    if value.is_empty() {
        return Err(WasmPluginStoreError::InvalidPackage(
            "CLI resource file name cannot be empty".to_string(),
        ));
    }
    let file_name = Path::new(value)
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| {
            WasmPluginStoreError::InvalidPackage("CLI resource file name is invalid".to_string())
        })?;
    if file_name != value {
        return Err(WasmPluginStoreError::InvalidPackage(
            "CLI resource file name must not contain directories".to_string(),
        ));
    }
    Ok(file_name.to_string())
}

fn default_cli_install_dir() -> PathBuf {
    env::var_os("AIO_PLUGIN_CLI_BIN_DIR")
        .map(PathBuf::from)
        .or_else(|| dirs::home_dir().map(|home| home.join(".local/bin")))
        .unwrap_or_else(|| PathBuf::from(".aio-plugin-bin"))
}

fn ensure_install_target_is_safe(install_dir: &Path, target: &Path) -> WasmPluginStoreResult<()> {
    let canonical_dir = install_dir
        .canonicalize()
        .unwrap_or_else(|_| install_dir.to_path_buf());
    let check_path = target
        .parent()
        .and_then(|parent| parent.canonicalize().ok())
        .unwrap_or_else(|| canonical_dir.clone());
    if !check_path.starts_with(&canonical_dir) {
        return Err(WasmPluginStoreError::InvalidPackage(format!(
            "CLI install target escapes install dir: {}",
            target.display()
        )));
    }
    Ok(())
}

#[cfg(unix)]
fn make_executable(path: &Path) -> std::io::Result<()> {
    use std::os::unix::fs::PermissionsExt;

    let mut permissions = fs::metadata(path)?.permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(path, permissions)
}

#[cfg(not(unix))]
fn make_executable(_path: &Path) -> std::io::Result<()> {
    Ok(())
}

fn sorted_menus(
    menus: &[az_plugin_contract::PluginMenuContribution],
) -> Vec<&az_plugin_contract::PluginMenuContribution> {
    let mut sorted = menus.iter().collect::<Vec<_>>();
    sorted.sort_by(|left, right| {
        left.order
            .cmp(&right.order)
            .then(left.label.cmp(&right.label))
    });
    sorted
}

fn slugify(value: &str) -> String {
    let mut slug = String::new();
    let mut last_dash = false;
    for ch in value.chars() {
        let mapped = if ch.is_ascii_alphanumeric() {
            last_dash = false;
            Some(ch.to_ascii_lowercase())
        } else if !last_dash {
            last_dash = true;
            Some('-')
        } else {
            None
        };
        if let Some(ch) = mapped {
            slug.push(ch);
        }
    }
    slug.trim_matches('-').to_string()
}

#[cfg(test)]
mod tests {
    use az_plugin_contract::{
        MarkdownSchema, PageSchema, PluginDescriptor, PluginKind, PluginPage,
    };

    use super::{validate_descriptor, validate_wasm_magic};

    #[test]
    fn validate_wasm_magic_should_accept_wasm_binary_header() {
        let wasm = [0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00];

        assert!(validate_wasm_magic(&wasm).is_ok());
    }

    #[test]
    fn validate_descriptor_should_reject_empty_pages() {
        let descriptor = PluginDescriptor {
            id: "bare-wasm".to_string(),
            name: "Bare Wasm".to_string(),
            version: "0.1.0".to_string(),
            kind: PluginKind::Business,
            summary: "bare wasm upload".to_string(),
            tags: vec![],
            icon: None,
            compatibility: vec![],
            capabilities: vec![],
            menus: vec![],
            pages: vec![],
            metadata: Default::default(),
            cli_commands: vec![],
        };

        // The DB row needs at least one page so the admin shell can route the plugin.
        assert!(validate_descriptor(&descriptor).is_err());
    }

    #[test]
    fn validate_descriptor_should_accept_business_plugin_metadata() {
        let descriptor = PluginDescriptor {
            id: "bare-wasm".to_string(),
            name: "Bare Wasm".to_string(),
            version: "0.1.0".to_string(),
            kind: PluginKind::Business,
            summary: "bare wasm upload".to_string(),
            tags: vec!["wasm".to_string()],
            icon: None,
            compatibility: vec!["web".to_string()],
            capabilities: vec![],
            menus: vec![],
            pages: vec![PluginPage {
                id: "overview".to_string(),
                title: "Overview".to_string(),
                subtitle: "Bare wasm metadata".to_string(),
                schema: PageSchema::Markdown(MarkdownSchema {
                    body: "Ready".to_string(),
                }),
            }],
            metadata: Default::default(),
            cli_commands: vec![],
        };

        assert!(validate_descriptor(&descriptor).is_ok());
    }
}
