#![cfg(not(target_arch = "wasm32"))]

use std::collections::{BTreeMap, BTreeSet};
use std::io::{Cursor, Read};

use az_minio::MinioClient;
use az_plugin_contract::{
    MarketplaceEntry, MarketplaceSnapshot, NavigationItem, NavigationItemKind, NavigationSection,
    PageScope, PluginDescriptor, PluginInstance, PluginInstanceConfig, PluginKind,
    PluginPackageManifest, PluginStatus, ResolvedPage, RuntimeBinding,
};
use az_plugin_runtime::RuntimeError;
use chrono::{DateTime, Utc};
use serde_json::Value;
use sha2::{Digest, Sha256};
use sqlx::{PgPool, Row};
use thiserror::Error;
use uuid::Uuid;
use zip::ZipArchive;

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

impl WasmPluginStore {
    pub async fn connect(database_url: &str) -> WasmPluginStoreResult<Self> {
        let pool = PgPool::connect(database_url).await?;
        let environment =
            minio_environment_from_env().map_err(WasmPluginStoreError::MinioUnavailable)?;
        environment
            .client
            .ensure_bucket(&environment.bucket)
            .map_err(|err| WasmPluginStoreError::MinioUnavailable(err.to_string()))?;
        Ok(Self {
            pool,
            storage: WasmPluginBinaryStorage {
                client: environment.client,
                bucket: environment.bucket,
            },
        })
    }

    pub async fn import_azplugin_package(
        &self,
        file_name: &str,
        bytes: &[u8],
    ) -> WasmPluginStoreResult<StoredWasmPluginPackage> {
        if bytes.is_empty() {
            return Err(WasmPluginStoreError::InvalidPackage(
                "plugin upload bytes cannot be empty".to_string(),
            ));
        }
        let package = ParsedAzPluginPackage::parse(bytes)?;
        if package.manifest.descriptor.kind != PluginKind::Business {
            return Err(WasmPluginStoreError::InvalidPackage(
                "only external Business wasm plugins can be stored".to_string(),
            ));
        }

        let binary_sha256 = sha256_hex(&package.wasm_bytes);
        let binary_object_key = format!(
            "{}/{}/{}/{}.wasm",
            WASM_PLUGIN_OBJECT_PREFIX,
            package.manifest.descriptor.id,
            package.manifest.descriptor.version,
            binary_sha256,
        );
        self.storage
            .client
            .put_object_bytes(
                &self.storage.bucket,
                &binary_object_key,
                &package.wasm_bytes,
                Some("application/wasm"),
            )
            .map_err(|err| WasmPluginStoreError::MinioUnavailable(err.to_string()))?;

        self.upsert_package_row(
            &package.manifest,
            &binary_object_key,
            &binary_sha256,
            package.wasm_bytes.len(),
            "azplugin",
            WasmFirmwareKind::Business,
        )
        .await?;

        Ok(StoredWasmPluginPackage {
            plugin_id: package.manifest.descriptor.id,
            plugin_name: package.manifest.descriptor.name,
            version: package.manifest.descriptor.version,
            source_file_name: file_name.to_string(),
            binary_bucket: self.storage.bucket.clone(),
            binary_object_key,
            binary_sha256,
            binary_size_bytes: package.wasm_bytes.len() as u64,
        })
    }

    pub async fn import_wasm_binary(
        &self,
        file_name: &str,
        bytes: &[u8],
        descriptor: PluginDescriptor,
        default_instance_label: Option<String>,
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
            .client
            .put_object_bytes(
                &self.storage.bucket,
                &binary_object_key,
                bytes,
                Some("application/wasm"),
            )
            .map_err(|err| WasmPluginStoreError::MinioUnavailable(err.to_string()))?;

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
            .client
            .put_object_bytes(
                &self.storage.bucket,
                &binary_object_key,
                bytes,
                Some("application/wasm"),
            )
            .map_err(|err| WasmPluginStoreError::MinioUnavailable(err.to_string()))?;

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
            allowed_origins: vec![],
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
        sqlx::query(
            r#"
            INSERT INTO wasm_plugin_packages (
                plugin_id,
                name,
                version,
                summary,
                descriptor,
                runtime,
                default_instance_label,
                binary_bucket,
                binary_object_key,
                binary_sha256,
                binary_size_bytes,
                source_format,
                firmware_kind,
                status
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, 'available')
            ON CONFLICT (plugin_id) DO UPDATE SET
                name = EXCLUDED.name,
                version = EXCLUDED.version,
                summary = EXCLUDED.summary,
                descriptor = EXCLUDED.descriptor,
                runtime = EXCLUDED.runtime,
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
            SELECT descriptor, default_instance_label
            FROM wasm_plugin_packages
            WHERE status != 'disabled'
            ORDER BY name ASC
            "#,
        )
        .fetch_all(&self.pool)
        .await?;
        rows.into_iter().map(package_from_row).collect()
    }

    async fn package(&self, plugin_id: &str) -> WasmPluginStoreResult<Option<StoredPackageRow>> {
        let row = sqlx::query(
            r#"
            SELECT descriptor, default_instance_label
            FROM wasm_plugin_packages
            WHERE plugin_id = $1 AND status != 'disabled'
            "#,
        )
        .bind(plugin_id)
        .fetch_optional(&self.pool)
        .await?;
        row.map(package_from_row).transpose()
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

struct ParsedAzPluginPackage {
    manifest: PluginPackageManifest,
    wasm_bytes: Vec<u8>,
}

impl ParsedAzPluginPackage {
    fn parse(bytes: &[u8]) -> WasmPluginStoreResult<Self> {
        let mut archive = ZipArchive::new(Cursor::new(bytes))?;
        let mut manifest_source = String::new();
        archive
            .by_name("plugin.toml")
            .map_err(|_| {
                WasmPluginStoreError::InvalidPackage("plugin.toml is required".to_string())
            })?
            .read_to_string(&mut manifest_source)?;
        let manifest: PluginPackageManifest = toml_edit::de::from_str(&manifest_source)
            .map_err(|err| WasmPluginStoreError::InvalidPackage(err.to_string()))?;

        validate_checksums(&mut archive)?;
        let mut wasm_bytes = Vec::new();
        archive
            .by_name(&manifest.runtime.binary_path)
            .map_err(|_| {
                WasmPluginStoreError::InvalidPackage(format!(
                    "runtime binary `{}` is required",
                    manifest.runtime.binary_path
                ))
            })?
            .read_to_end(&mut wasm_bytes)?;
        if wasm_bytes.is_empty() {
            return Err(WasmPluginStoreError::InvalidPackage(
                "runtime wasm bytes cannot be empty".to_string(),
            ));
        }
        Ok(Self {
            manifest,
            wasm_bytes,
        })
    }
}

fn validate_checksums(archive: &mut ZipArchive<Cursor<&[u8]>>) -> WasmPluginStoreResult<()> {
    let mut content = String::new();
    archive
        .by_name("checksums.sha256")
        .map_err(|_| {
            WasmPluginStoreError::InvalidPackage("checksums.sha256 is required".to_string())
        })?
        .read_to_string(&mut content)?;
    let checksums = parse_checksums(&content);
    if checksums.is_empty() {
        return Err(WasmPluginStoreError::InvalidPackage(
            "checksums.sha256 did not contain any entries".to_string(),
        ));
    }
    for (entry_path, expected) in checksums {
        let mut entry = archive.by_name(&entry_path).map_err(|_| {
            WasmPluginStoreError::InvalidPackage(format!("missing packaged file `{entry_path}`"))
        })?;
        let mut bytes = Vec::new();
        entry.read_to_end(&mut bytes)?;
        if sha256_hex(&bytes) != expected {
            return Err(WasmPluginStoreError::InvalidPackage(format!(
                "checksum mismatch for `{entry_path}`"
            )));
        }
    }
    Ok(())
}

fn parse_checksums(content: &str) -> BTreeMap<String, String> {
    content
        .lines()
        .filter_map(|line| {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                return None;
            }
            let (hash, path) = trimmed.split_once("  ")?;
            Some((path.to_string(), hash.to_string()))
        })
        .collect()
}

fn package_from_row(row: sqlx::postgres::PgRow) -> WasmPluginStoreResult<StoredPackageRow> {
    let descriptor_value: Value = row.get("descriptor");
    Ok(StoredPackageRow {
        descriptor: serde_json::from_value(descriptor_value)?,
        default_instance_label: row.get("default_instance_label"),
    })
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

impl From<RuntimeError> for WasmPluginStoreError {
    fn from(value: RuntimeError) -> Self {
        WasmPluginStoreError::InvalidPackage(value.to_string())
    }
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
        };

        assert!(validate_descriptor(&descriptor).is_ok());
    }
}
