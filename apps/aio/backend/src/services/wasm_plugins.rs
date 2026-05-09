#![cfg(not(target_arch = "wasm32"))]

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use az_plugin_contract::{
    BoardSchema, MarketplaceSnapshot, MetricCard, PageSchema, PluginDescriptor, PluginKind,
    PluginMenuContribution, PluginPage, PluginStatus, RecordGroup, RecordItem, ResolvedPage,
    RuntimeOverview, ShellSnapshot,
};
use az_plugin_kernel::PlatformKernel;
use az_plugin_runtime::{read_manifest_from_package, validate_package};
use chrono::Utc;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::server::resolved_database_url;

use super::wasm_plugin_store::{WasmFirmwareKind, WasmPluginStore};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct WasmPluginRuntimeSnapshotDto {
    pub shell: ShellSnapshot,
    pub marketplace: MarketplaceSnapshot,
    pub runtime: RuntimeOverview,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct WasmPluginUploadRequestDto {
    pub file_name: String,
    pub bytes: Vec<u8>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct WasmPluginBinaryUploadRequestDto {
    pub file_name: String,
    pub bytes: Vec<u8>,
    pub descriptor: PluginDescriptor,
    pub default_instance_label: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum WasmPluginFirmwareKindDto {
    System,
    Business,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct WasmPluginFirmwareUploadRequestDto {
    pub name: String,
    pub description: String,
    pub firmware_kind: WasmPluginFirmwareKindDto,
    pub file_name: String,
    pub bytes: Vec<u8>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct WasmPluginUploadResultDto {
    pub package_path: String,
    pub plugin_id: String,
    pub plugin_name: String,
    pub version: String,
    pub validated: bool,
    pub storage_backend: String,
    pub binary_object_key: Option<String>,
    pub binary_sha256: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct WasmPluginInstallRequestDto {
    pub plugin_id: String,
    pub instance_label: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct WasmPluginInstallResultDto {
    pub plugin_id: String,
    pub plugin_name: String,
    pub version: String,
    pub instance_slug: String,
    pub instance_label: String,
    pub page_ids: Vec<String>,
}

static PLATFORM_KERNEL: Lazy<Result<Arc<PlatformKernel>, String>> = Lazy::new(|| {
    az_system_starters::link_all();
    PlatformKernel::new(default_catalog_dir(), default_package_root())
        .map(Arc::new)
        .map_err(|err| format!("init wasm plugin kernel: {err}"))
});

static WASM_PLUGIN_STORE: Lazy<Result<Option<Arc<WasmPluginStore>>, String>> =
    Lazy::new(|| match resolved_database_url() {
        Some(database_url) => {
            tokio::task::block_in_place(|| {
                match tokio::runtime::Handle::current()
                    .block_on(WasmPluginStore::connect(&database_url))
                {
                    Ok(store) => Ok(Some(Arc::new(store))),
                    Err(err) => {
                        log::warn!("wasm plugin PG/MinIO store is unavailable: {err}");
                        Ok(None)
                    }
                }
            })
        }
        None => Ok(None),
    });

pub async fn wasm_plugin_runtime_snapshot_on_server() -> Result<WasmPluginRuntimeSnapshotDto, String>
{
    let kernel = platform_kernel()?;
    let mut shell = kernel
        .shell_snapshot()
        .map_err(|err| format!("build shell snapshot: {err}"))?;
    let mut marketplace = kernel
        .marketplace_snapshot()
        .map_err(|err| format!("build file-backed marketplace snapshot: {err}"))?;
    let mut runtime = kernel
        .runtime_overview()
        .map_err(|err| format!("build runtime overview: {err}"))?;
    if let Some(store) = wasm_plugin_store()? {
        let persistent_marketplace = store
            .marketplace_snapshot()
            .await
            .map_err(|err| format!("build PG/MinIO marketplace snapshot: {err}"))?;
        merge_marketplace(&mut marketplace, persistent_marketplace);
        let persistent_sections = store
            .plugin_navigation()
            .await
            .map_err(|err| format!("build PG/MinIO plugin navigation: {err}"))?;
        merge_shell_sections(&mut shell, persistent_sections);
        let persistent_instances = store
            .instances()
            .await
            .map_err(|err| format!("load PG/MinIO plugin instances: {err}"))?;
        let kernel_instances = shell.counts.plugin_instances;
        shell.counts.system_plugins = marketplace
            .entries
            .iter()
            .filter(|entry| entry.kind == PluginKind::System)
            .count();
        shell.counts.installed_business_plugins = marketplace
            .entries
            .iter()
            .filter(|entry| entry.kind == PluginKind::Business && entry.instances > 0)
            .count();
        shell.counts.plugin_instances = kernel_instances + persistent_instances.len();
        runtime.counts = shell.counts.clone();
    }
    Ok(WasmPluginRuntimeSnapshotDto {
        shell,
        marketplace,
        runtime,
    })
}

pub async fn install_catalog_wasm_plugin_on_server(
    input: WasmPluginInstallRequestDto,
) -> Result<WasmPluginInstallResultDto, String> {
    if let Some(store) = wasm_plugin_store()? {
        let snapshot = store
            .marketplace_snapshot()
            .await
            .map_err(|err| format!("load PG/MinIO plugin marketplace: {err}"))?;
        if let Some(current) = snapshot
            .entries
            .iter()
            .find(|entry| entry.plugin_id == input.plugin_id)
            .cloned()
        {
            let instance = store
                .create_instance(&input.plugin_id, input.instance_label.as_deref())
                .await
                .map_err(|err| {
                    format!(
                        "create persisted plugin instance `{}`: {err}",
                        input.plugin_id
                    )
                })?;
            return Ok(WasmPluginInstallResultDto {
                plugin_id: input.plugin_id,
                plugin_name: current.name,
                version: current.version,
                instance_slug: instance.slug,
                instance_label: instance.label,
                page_ids: instance.page_ids,
            });
        }
    }

    let kernel = platform_kernel()?;
    let snapshot = kernel
        .marketplace_snapshot()
        .map_err(|err| format!("load marketplace snapshot: {err}"))?;
    let current = snapshot
        .entries
        .iter()
        .find(|entry| entry.plugin_id == input.plugin_id)
        .cloned()
        .ok_or_else(|| format!("plugin `{}` not found in catalog", input.plugin_id))?;

    let plugin_name = current.name.clone();
    let version = current.version.clone();
    if current.status != az_plugin_contract::PluginStatus::Installed {
        kernel
            .install_catalog_plugin(&input.plugin_id)
            .map_err(|err| format!("install catalog plugin `{}`: {err}", input.plugin_id))?;
    }

    let instance_label = input
        .instance_label
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or(plugin_name.as_str())
        .to_string();
    let instance = kernel
        .create_instance(&input.plugin_id, &instance_label)
        .map_err(|err| format!("create plugin instance `{}`: {err}", input.plugin_id))?;

    Ok(WasmPluginInstallResultDto {
        plugin_id: input.plugin_id,
        plugin_name,
        version,
        instance_slug: instance.slug,
        instance_label: instance.label,
        page_ids: instance.page_ids,
    })
}

pub async fn upload_wasm_plugin_on_server(
    input: WasmPluginUploadRequestDto,
) -> Result<WasmPluginUploadResultDto, String> {
    let file_name = normalized_upload_file_name(&input.file_name)?;
    if input.bytes.is_empty() {
        return Err("plugin upload bytes cannot be empty".to_string());
    }

    if let Some(store) = wasm_plugin_store()? {
        let stored = store
            .import_azplugin_package(&file_name, &input.bytes)
            .await
            .map_err(|err| format!("store uploaded plugin in PG/MinIO: {err}"))?;
        return Ok(WasmPluginUploadResultDto {
            package_path: format!("{}/{}", stored.binary_bucket, stored.binary_object_key),
            plugin_id: stored.plugin_id,
            plugin_name: stored.plugin_name,
            version: stored.version,
            validated: true,
            storage_backend: "pg+minio".to_string(),
            binary_object_key: Some(stored.binary_object_key),
            binary_sha256: Some(stored.binary_sha256),
        });
    }

    let kernel = platform_kernel()?;
    let temp_dir = default_plugins_root().join(".tmp");
    fs::create_dir_all(&temp_dir)
        .map_err(|err| format!("create temp plugin dir {}: {err}", temp_dir.display()))?;
    let temp_path = temp_dir.join(format!("upload-{file_name}"));
    fs::write(&temp_path, &input.bytes)
        .map_err(|err| format!("write uploaded plugin {}: {err}", temp_path.display()))?;

    let result = (|| {
        validate_package(&temp_path)
            .map_err(|err| format!("validate uploaded package {}: {err}", temp_path.display()))?;
        let manifest = read_manifest_from_package(&temp_path)
            .map_err(|err| format!("read uploaded manifest {}: {err}", temp_path.display()))?;
        if manifest.descriptor.kind != PluginKind::Business {
            return Err(
                "only external Business wasm plugins can be uploaded to marketplace".to_string(),
            );
        }
        let package_path = store_uploaded_package(kernel, &manifest.descriptor.id, &temp_path)?;
        Ok(WasmPluginUploadResultDto {
            package_path: package_path.display().to_string(),
            plugin_id: manifest.descriptor.id,
            plugin_name: manifest.descriptor.name,
            version: manifest.descriptor.version,
            validated: true,
            storage_backend: "catalog-file".to_string(),
            binary_object_key: None,
            binary_sha256: None,
        })
    })();

    let _ = fs::remove_file(&temp_path);
    result
}

pub async fn upload_wasm_binary_plugin_on_server(
    input: WasmPluginBinaryUploadRequestDto,
) -> Result<WasmPluginUploadResultDto, String> {
    let file_name = normalized_binary_upload_file_name(&input.file_name)?;
    if input.bytes.is_empty() {
        return Err("wasm upload bytes cannot be empty".to_string());
    }
    let Some(store) = wasm_plugin_store()? else {
        return Err(
            "bare wasm upload requires PostgreSQL metadata storage and MinIO binary storage"
                .to_string(),
        );
    };
    let stored = store
        .import_wasm_binary(
            &file_name,
            &input.bytes,
            input.descriptor,
            input.default_instance_label,
        )
        .await
        .map_err(|err| format!("store uploaded wasm in PG/MinIO: {err}"))?;
    Ok(WasmPluginUploadResultDto {
        package_path: format!("{}/{}", stored.binary_bucket, stored.binary_object_key),
        plugin_id: stored.plugin_id,
        plugin_name: stored.plugin_name,
        version: stored.version,
        validated: true,
        storage_backend: "pg+minio".to_string(),
        binary_object_key: Some(stored.binary_object_key),
        binary_sha256: Some(stored.binary_sha256),
    })
}

pub async fn upload_wasm_firmware_plugin_on_server(
    input: WasmPluginFirmwareUploadRequestDto,
) -> Result<WasmPluginUploadResultDto, String> {
    let name = normalized_text_field("plugin name", &input.name)?;
    let description = normalized_text_field("plugin description", &input.description)?;
    let file_name = normalized_binary_upload_file_name(&input.file_name)?;
    if input.bytes.is_empty() {
        return Err("wasm upload bytes cannot be empty".to_string());
    }
    let Some(store) = wasm_plugin_store()? else {
        return Err(
            "wasm firmware upload requires PostgreSQL metadata storage and MinIO binary storage"
                .to_string(),
        );
    };
    let descriptor = build_firmware_descriptor(&name, &description, &input.firmware_kind);
    let firmware_kind = match input.firmware_kind {
        WasmPluginFirmwareKindDto::System => WasmFirmwareKind::System,
        WasmPluginFirmwareKindDto::Business => WasmFirmwareKind::Business,
    };
    let stored = store
        .import_firmware_binary(
            &file_name,
            &input.bytes,
            descriptor,
            Some(name),
            firmware_kind,
        )
        .await
        .map_err(|err| format!("store uploaded firmware in PG/MinIO: {err}"))?;
    Ok(WasmPluginUploadResultDto {
        package_path: format!("{}/{}", stored.binary_bucket, stored.binary_object_key),
        plugin_id: stored.plugin_id,
        plugin_name: stored.plugin_name,
        version: stored.version,
        validated: true,
        storage_backend: "pg+minio".to_string(),
        binary_object_key: Some(stored.binary_object_key),
        binary_sha256: Some(stored.binary_sha256),
    })
}

pub async fn resolve_system_wasm_plugin_page_on_server(
    plugin_id: String,
    page_id: String,
) -> Result<ResolvedPage, String> {
    if let Some(store) = wasm_plugin_store()? {
        match store.resolve_system_page(&plugin_id, &page_id).await {
            Ok(page) => return Ok(page),
            Err(super::wasm_plugin_store::WasmPluginStoreError::PageNotFound { .. }) => {}
            Err(super::wasm_plugin_store::WasmPluginStoreError::NotInstalled(_)) => {}
            Err(err) => {
                return Err(format!(
                    "resolve persisted system page `{plugin_id}/{page_id}`: {err}"
                ));
            }
        }
    }

    let kernel = platform_kernel()?;
    kernel
        .resolve_system_page(&plugin_id, &page_id)
        .map_err(|err| format!("resolve system page `{plugin_id}/{page_id}`: {err}"))?
        .ok_or_else(|| format!("system page `{plugin_id}/{page_id}` was not found"))
}

pub async fn resolve_instance_wasm_plugin_page_on_server(
    instance_slug: String,
    page_id: String,
) -> Result<ResolvedPage, String> {
    if let Some(store) = wasm_plugin_store()? {
        match store.resolve_instance_page(&instance_slug, &page_id).await {
            Ok(page) => return Ok(page),
            Err(super::wasm_plugin_store::WasmPluginStoreError::InstanceNotFound(_)) => {}
            Err(err) => {
                return Err(format!(
                    "resolve persisted instance page `{instance_slug}/{page_id}`: {err}"
                ));
            }
        }
    }

    let kernel = platform_kernel()?;
    kernel
        .resolve_instance_page(&instance_slug, &page_id)
        .map_err(|err| format!("resolve instance page `{instance_slug}/{page_id}`: {err}"))?
        .ok_or_else(|| format!("instance page `{instance_slug}/{page_id}` was not found"))
}

fn platform_kernel() -> Result<&'static Arc<PlatformKernel>, String> {
    PLATFORM_KERNEL.as_ref().map_err(Clone::clone)
}

fn wasm_plugin_store() -> Result<Option<&'static Arc<WasmPluginStore>>, String> {
    WASM_PLUGIN_STORE
        .as_ref()
        .map(|store| store.as_ref())
        .map_err(Clone::clone)
}

fn default_catalog_dir() -> PathBuf {
    default_plugins_root().join("catalog")
}

fn default_package_root() -> PathBuf {
    default_plugins_root().join("host")
}

fn default_plugins_root() -> PathBuf {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../plugins");
    root.canonicalize().unwrap_or(root)
}

fn store_uploaded_package(
    kernel: &Arc<PlatformKernel>,
    plugin_id: &str,
    source_path: &Path,
) -> Result<PathBuf, String> {
    let catalog_dir = kernel
        .catalog_dir()
        .map_err(|err| format!("resolve wasm plugin catalog dir: {err}"))?;
    fs::create_dir_all(&catalog_dir)
        .map_err(|err| format!("create catalog dir {}: {err}", catalog_dir.display()))?;
    let target_path = catalog_dir.join(format!("{plugin_id}.azplugin"));
    fs::copy(source_path, &target_path).map_err(|err| {
        format!(
            "copy uploaded package {} -> {}: {err}",
            source_path.display(),
            target_path.display()
        )
    })?;
    kernel
        .refresh_catalog()
        .map_err(|err| format!("refresh wasm plugin catalog after upload: {err}"))?;
    Ok(target_path)
}

fn normalized_upload_file_name(raw: &str) -> Result<String, String> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Err("upload file name is required".to_string());
    }
    if !trimmed.ends_with(".azplugin") {
        return Err("only `.azplugin` plugin packages can be uploaded".to_string());
    }
    let file_name = Path::new(trimmed)
        .file_name()
        .and_then(|value| value.to_str())
        .ok_or_else(|| "upload file name is invalid".to_string())?;
    if file_name != trimmed {
        return Err("upload file name must not contain parent directories".to_string());
    }
    Ok(file_name.to_string())
}

fn normalized_binary_upload_file_name(raw: &str) -> Result<String, String> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Err("upload file name is required".to_string());
    }
    if !trimmed.ends_with(".wasm") {
        return Err("only `.wasm` plugin binaries can be uploaded".to_string());
    }
    let file_name = Path::new(trimmed)
        .file_name()
        .and_then(|value| value.to_str())
        .ok_or_else(|| "upload file name is invalid".to_string())?;
    if file_name != trimmed {
        return Err("upload file name must not contain parent directories".to_string());
    }
    Ok(file_name.to_string())
}

fn normalized_text_field(label: &str, raw: &str) -> Result<String, String> {
    let normalized = raw.trim();
    if normalized.is_empty() {
        return Err(format!("{label} is required"));
    }
    Ok(normalized.to_string())
}

fn build_firmware_descriptor(
    name: &str,
    description: &str,
    firmware_kind: &WasmPluginFirmwareKindDto,
) -> PluginDescriptor {
    let slug = slugify(name);
    let plugin_id = if slug.is_empty() {
        short_fingerprint(name)
    } else {
        format!("firmware-{slug}")
    };
    let category_tag = match firmware_kind {
        WasmPluginFirmwareKindDto::System => "system-firmware",
        WasmPluginFirmwareKindDto::Business => "business-firmware",
    };
    let kind = match firmware_kind {
        WasmPluginFirmwareKindDto::System => PluginKind::System,
        WasmPluginFirmwareKindDto::Business => PluginKind::Business,
    };
    let section = match firmware_kind {
        WasmPluginFirmwareKindDto::System => "系统固件",
        WasmPluginFirmwareKindDto::Business => "业务固件",
    };
    let category_label = match firmware_kind {
        WasmPluginFirmwareKindDto::System => "系统固件",
        WasmPluginFirmwareKindDto::Business => "业务固件",
    };

    PluginDescriptor {
        id: plugin_id,
        name: name.to_string(),
        version: Utc::now().format("%Y.%m.%d.%H%M%S").to_string(),
        kind,
        summary: description.to_string(),
        tags: vec![
            "wasm".to_string(),
            "firmware".to_string(),
            category_tag.to_string(),
        ],
        icon: Some("cpu".to_string()),
        compatibility: vec!["web".to_string(), "desktop".to_string()],
        capabilities: vec![],
        menus: vec![PluginMenuContribution {
            section: section.to_string(),
            label: name.to_string(),
            page_id: "overview".to_string(),
            order: 10,
            icon: Some("cpu".to_string()),
        }],
        pages: vec![PluginPage {
            id: "overview".to_string(),
            title: name.to_string(),
            subtitle: category_label.to_string(),
            schema: PageSchema::Board(BoardSchema {
                metrics: vec![
                    MetricCard {
                        label: "分类".to_string(),
                        value: category_label.to_string(),
                        detail: "由上传入口写入 PostgreSQL 元数据。".to_string(),
                    },
                    MetricCard {
                        label: "二进制".to_string(),
                        value: "MinIO".to_string(),
                        detail: "WASM 固件二进制不落本地 catalog 文件。".to_string(),
                    },
                    MetricCard {
                        label: "入口".to_string(),
                        value: "插件化".to_string(),
                        detail: "菜单、页面和实例都由插件描述驱动。".to_string(),
                    },
                ],
                groups: vec![RecordGroup {
                    title: "固件说明".to_string(),
                    items: vec![
                        RecordItem {
                            title: "描述".to_string(),
                            detail: description.to_string(),
                            meta: "metadata.description".to_string(),
                        },
                        RecordItem {
                            title: "存储模型".to_string(),
                            detail: "插件元数据写入 PostgreSQL，WASM 二进制写入 MinIO。"
                                .to_string(),
                            meta: "pg+minio".to_string(),
                        },
                    ],
                }],
            }),
        }],
    }
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

fn short_fingerprint(value: &str) -> String {
    let mut digest = Sha256::new();
    digest.update(value.as_bytes());
    let hash = format!("{:x}", digest.finalize());
    format!("firmware-{}", &hash[..12])
}

fn merge_marketplace(base: &mut MarketplaceSnapshot, persistent: MarketplaceSnapshot) {
    for entry in persistent.entries {
        if let Some(existing) = base
            .entries
            .iter_mut()
            .find(|existing| existing.plugin_id == entry.plugin_id)
        {
            *existing = entry;
        } else {
            base.entries.push(entry);
        }
    }
    base.entries
        .sort_by(|left, right| left.name.cmp(&right.name));

    let mut tags = base
        .tags
        .iter()
        .cloned()
        .collect::<std::collections::BTreeSet<_>>();
    tags.extend(persistent.tags);
    base.tags = tags.into_iter().collect();

    for entry in &mut base.entries {
        if entry.status == PluginStatus::Available && entry.instances > 0 {
            entry.status = PluginStatus::Installed;
        }
    }
}

fn merge_shell_sections(
    shell: &mut ShellSnapshot,
    persistent: Vec<az_plugin_contract::NavigationSection>,
) {
    for section in persistent {
        if let Some(existing) = shell
            .nav_sections
            .iter_mut()
            .find(|existing| existing.label == section.label)
        {
            for item in section.items {
                if !existing.items.iter().any(|existing_item| {
                    existing_item.href == item.href
                        && existing_item.plugin_id == item.plugin_id
                        && existing_item.page_id == item.page_id
                }) {
                    existing.items.push(item);
                }
            }
        } else {
            shell.nav_sections.push(section);
        }
    }
}
