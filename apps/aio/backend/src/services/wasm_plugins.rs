#![cfg(not(target_arch = "wasm32"))]

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use az_plugin_contract::{
    BoardSchema, MarketplaceSnapshot, MetricCard, NotesFragmentsSchema, PageSchema,
    PluginDescriptor, PluginKind, PluginMenuContribution, PluginMetadata, PluginPage, PluginStatus,
    RecordGroup, RecordItem, ResolvedPage, RuntimeOverview, ShellSnapshot,
};
use az_plugin_kernel::PlatformKernel;
use chrono::Utc;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::server::resolved_database_url;

use super::wasm_plugin_store::{WasmFirmwareKind, WasmPluginCliResourceUpload, WasmPluginStore};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct WasmPluginRuntimeSnapshotDto {
    pub shell: ShellSnapshot,
    pub marketplace: MarketplaceSnapshot,
    pub runtime: RuntimeOverview,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct WasmPluginBinaryUploadRequestDto {
    pub file_name: String,
    pub bytes: Vec<u8>,
    pub descriptor: PluginDescriptor,
    pub default_instance_label: Option<String>,
    #[serde(default)]
    pub cli_resources: Vec<WasmPluginCliResourceUploadDto>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct WasmPluginCliResourceUploadDto {
    pub command_name: String,
    pub file_name: String,
    pub bytes: Vec<u8>,
    pub content_type: Option<String>,
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
    #[serde(default)]
    pub metadata: PluginMetadata,
    #[serde(default)]
    pub cli_resources: Vec<WasmPluginCliResourceUploadDto>,
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

impl From<WasmPluginCliResourceUploadDto> for WasmPluginCliResourceUpload {
    fn from(value: WasmPluginCliResourceUploadDto) -> Self {
        Self {
            command_name: value.command_name,
            file_name: value.file_name,
            bytes: value.bytes,
            content_type: value.content_type,
        }
    }
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
            input.cli_resources.into_iter().map(Into::into).collect(),
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
    let descriptor = build_firmware_descriptor(
        &name,
        &description,
        &input.firmware_kind,
        input.metadata,
        input.cli_resources.len(),
    );
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
            input.cli_resources.into_iter().map(Into::into).collect(),
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

pub async fn seed_cloudflare_tunnel_plugin_on_server() -> Result<WasmPluginUploadResultDto, String>
{
    register_cloudflare_tunnel_plugin_on_server().await
}

pub async fn register_cloudflare_tunnel_plugin_on_server()
-> Result<WasmPluginUploadResultDto, String> {
    let descriptor = cloudflare_tunnel_descriptor();
    let resources = cloudflare_tunnel_cli_resources()?;
    let input = WasmPluginBinaryUploadRequestDto {
        file_name: "cloudflare-tunnel.wasm".to_string(),
        bytes: lifecycle_only_wasm(),
        descriptor,
        default_instance_label: Some("Cloudflare Tunnel".to_string()),
        cli_resources: resources,
    };
    upload_wasm_binary_plugin_on_server(input).await
}

pub async fn register_notes_fragments_plugin_on_server()
-> Result<WasmPluginInstallResultDto, String> {
    let descriptor = notes_fragments_descriptor();
    let plugin_id = descriptor.id.clone();
    if let Some(store) = wasm_plugin_store()? {
        if let Some(instance) = store
            .first_instance_for_plugin(&plugin_id)
            .await
            .map_err(|err| format!("load existing notes fragments plugin instance: {err}"))?
        {
            let version = store
                .marketplace_snapshot()
                .await
                .ok()
                .and_then(|snapshot| {
                    snapshot
                        .entries
                        .into_iter()
                        .find(|entry| entry.plugin_id == plugin_id)
                })
                .map(|entry| entry.version)
                .unwrap_or_else(|| descriptor.version.clone());
            return Ok(WasmPluginInstallResultDto {
                plugin_id,
                plugin_name: instance.plugin_name,
                version,
                instance_slug: instance.slug,
                instance_label: instance.label,
                page_ids: instance.page_ids,
            });
        }
    }
    let input = WasmPluginBinaryUploadRequestDto {
        file_name: "notes-fragments.wasm".to_string(),
        bytes: lifecycle_only_wasm(),
        descriptor,
        default_instance_label: Some("碎片笔记".to_string()),
        cli_resources: vec![],
    };
    let uploaded = upload_wasm_binary_plugin_on_server(input).await?;
    install_catalog_wasm_plugin_on_server(WasmPluginInstallRequestDto {
        plugin_id: uploaded.plugin_id,
        instance_label: Some("碎片笔记".to_string()),
    })
    .await
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
    mut metadata: PluginMetadata,
    cli_command_count: usize,
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
                        label: "CLI".to_string(),
                        value: cli_command_count.to_string(),
                        detail: "命令资源由 PostgreSQL 索引并从 MinIO 安装到本机。".to_string(),
                    },
                ],
                groups: vec![RecordGroup {
                    title: "插件说明".to_string(),
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
                        RecordItem {
                            title: "维护者".to_string(),
                            detail: metadata.maintainer_name.clone(),
                            meta: metadata.maintainer_type.clone(),
                        },
                    ],
                }],
            }),
        }],
        metadata: {
            if metadata.description.trim().is_empty() {
                metadata.description = description.to_string();
            }
            if metadata.category.trim().is_empty() {
                metadata.category = category_label.to_string();
            }
            metadata
        },
        cli_commands: vec![],
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

fn cloudflare_tunnel_descriptor() -> PluginDescriptor {
    PluginDescriptor {
        id: "cloudflare-tunnel".to_string(),
        name: "Cloudflare Tunnel".to_string(),
        version: Utc::now().format("%Y.%m.%d.%H%M%S").to_string(),
        kind: PluginKind::Business,
        summary: "Expose local HTTP and TCP services through Cloudflare Tunnel and install OS-level host management CLI commands.".to_string(),
        tags: vec![
            "cloudflare".to_string(),
            "tunnel".to_string(),
            "cli".to_string(),
            "network".to_string(),
        ],
        icon: Some("cloud".to_string()),
        compatibility: vec!["desktop".to_string(), "macos".to_string()],
        capabilities: vec![],
        menus: vec![
            PluginMenuContribution {
                section: "运维工具".to_string(),
                label: "Tunnel 总览".to_string(),
                page_id: "overview".to_string(),
                order: 10,
                icon: Some("cloud".to_string()),
            },
            PluginMenuContribution {
                section: "运维工具".to_string(),
                label: "CLI 命令".to_string(),
                page_id: "cli".to_string(),
                order: 11,
                icon: Some("terminal".to_string()),
            },
        ],
        pages: vec![
            PluginPage {
                id: "overview".to_string(),
                title: "Cloudflare Tunnel".to_string(),
                subtitle: "本机 tunnel、DNS host 映射和 CLI 安装状态。".to_string(),
                schema: PageSchema::Board(BoardSchema {
                    metrics: vec![
                        MetricCard {
                            label: "工件".to_string(),
                            value: ".wasm".to_string(),
                            detail: "运行时二进制进入 MinIO，不使用自定义插件包。".to_string(),
                        },
                        MetricCard {
                            label: "元数据".to_string(),
                            value: "PostgreSQL".to_string(),
                            detail: "维护者、安装命令、菜单和页面描述都在数据库。".to_string(),
                        },
                        MetricCard {
                            label: "CLI".to_string(),
                            value: "4".to_string(),
                            detail: "addhost / showhost / rmhost / autohost".to_string(),
                        },
                    ],
                    groups: vec![RecordGroup {
                        title: "交付能力".to_string(),
                        items: vec![
                            RecordItem {
                                title: "addhost".to_string(),
                                detail: "把本机端口映射到 Cloudflare Tunnel hostname。".to_string(),
                                meta: "~/.local/bin/addhost".to_string(),
                            },
                            RecordItem {
                                title: "showhost".to_string(),
                                detail: "查看 tunnel ingress、本地进程、DNS 和 HTTPS 可达性。".to_string(),
                                meta: "~/.local/bin/showhost".to_string(),
                            },
                            RecordItem {
                                title: "autohost".to_string(),
                                detail: "从 Docker 容器发布端口自动生成 host 映射。".to_string(),
                                meta: "~/.local/bin/autohost".to_string(),
                            },
                        ],
                    }],
                }),
            },
            PluginPage {
                id: "cli".to_string(),
                title: "CLI 命令".to_string(),
                subtitle: "插件安装后由宿主从 MinIO 发布到本机 PATH。".to_string(),
                schema: PageSchema::Table(az_plugin_contract::TableSchema {
                    columns: vec![
                        "命令".to_string(),
                        "用途".to_string(),
                        "安装位置".to_string(),
                    ],
                    rows: vec![
                        az_plugin_contract::TableRow {
                            cells: vec![
                                "addhost".to_string(),
                                "新增或更新 tunnel ingress hostname".to_string(),
                                "~/.local/bin/addhost".to_string(),
                            ],
                        },
                        az_plugin_contract::TableRow {
                            cells: vec![
                                "showhost".to_string(),
                                "查询 tunnel host 状态".to_string(),
                                "~/.local/bin/showhost".to_string(),
                            ],
                        },
                        az_plugin_contract::TableRow {
                            cells: vec![
                                "rmhost".to_string(),
                                "移除 tunnel ingress hostname".to_string(),
                                "~/.local/bin/rmhost".to_string(),
                            ],
                        },
                        az_plugin_contract::TableRow {
                            cells: vec![
                                "autohost".to_string(),
                                "按 Docker 运行端口自动维护 host 映射".to_string(),
                                "~/.local/bin/autohost".to_string(),
                            ],
                        },
                    ],
                    empty_message: "暂无 CLI 命令。".to_string(),
                }),
            },
        ],
        metadata: PluginMetadata {
            github_url: "https://github.com/cloudflare/cloudflared".to_string(),
            description: "Local Cloudflare Tunnel hostname manager for AIO desktop environments."
                .to_string(),
            maintainer_type: "local".to_string(),
            maintainer_name: "zjarlin".to_string(),
            primary_language: "Bash/Python".to_string(),
            category: "Network Tools".to_string(),
            install_command: "Install plugin instance to publish addhost/showhost/rmhost/autohost into ~/.local/bin".to_string(),
            agent_install_command: "Upload cloudflare-tunnel.wasm metadata and CLI resources through the AIO DB-first plugin API.".to_string(),
        },
        cli_commands: vec![],
    }
}

fn notes_fragments_descriptor() -> PluginDescriptor {
    PluginDescriptor {
        id: "notes-fragments".to_string(),
        name: "碎片笔记".to_string(),
        version: Utc::now().format("%Y.%m.%d.%H%M%S").to_string(),
        kind: PluginKind::Business,
        summary: "Capture raw markdown fragments as a DB-backed notes plugin.".to_string(),
        tags: vec![
            "notes".to_string(),
            "fragments".to_string(),
            "markdown".to_string(),
            "knowledge".to_string(),
        ],
        icon: Some("note".to_string()),
        compatibility: vec!["web".to_string(), "desktop".to_string()],
        capabilities: vec![],
        menus: vec![PluginMenuContribution {
            section: "个人资产".to_string(),
            label: "碎片笔记".to_string(),
            page_id: "fragments".to_string(),
            order: 10,
            icon: Some("note".to_string()),
        }],
        pages: vec![PluginPage {
            id: "fragments".to_string(),
            title: "碎片笔记".to_string(),
            subtitle: "只记录原始碎片，整理能力后续作为独立页面接入。".to_string(),
            schema: PageSchema::NotesFragments(NotesFragmentsSchema {
                list_path: "/api/knowledge/entries".to_string(),
                save_path: "/api/knowledge/entries".to_string(),
                delete_path: "/api/knowledge/entries/delete".to_string(),
                placeholder: "记录碎片、命令、结论或上下文。支持 Markdown 和 #标签。"
                    .to_string(),
                empty_message: "还没有笔记。直接记录一条碎片。".to_string(),
            }),
        }],
        metadata: PluginMetadata {
            github_url: "".to_string(),
            description: "DB-first markdown fragment capture plugin for AIO.".to_string(),
            maintainer_type: "local".to_string(),
            maintainer_name: "zjarlin".to_string(),
            primary_language: "Rust/TypeScript/WASM".to_string(),
            category: "Knowledge Tools".to_string(),
            install_command: "Register notes-fragments.wasm through AIO plugin API.".to_string(),
            agent_install_command:
                "POST /api/wasm/plugins/register/notes-fragments to write metadata into PostgreSQL and the .wasm binary into MinIO."
                    .to_string(),
        },
        cli_commands: vec![],
    }
}

fn cloudflare_tunnel_cli_resources() -> Result<Vec<WasmPluginCliResourceUploadDto>, String> {
    let autohost = standalone_autohost_script()?;
    Ok(vec![
        cli_resource_from_path(
            "addhost",
            "addhost",
            Path::new("/Users/zjarlin/.local/bin/addhost"),
        )?,
        cli_resource_from_path(
            "showhost",
            "showhost",
            Path::new("/Users/zjarlin/.local/bin/showhost"),
        )?,
        cli_resource_from_path(
            "rmhost",
            "rmhost",
            Path::new("/Users/zjarlin/.local/bin/rmhost"),
        )?,
        WasmPluginCliResourceUploadDto {
            command_name: "autohost".to_string(),
            file_name: "autohost".to_string(),
            bytes: autohost.into_bytes(),
            content_type: Some("text/x-shellscript".to_string()),
        },
    ])
}

fn cli_resource_from_path(
    command_name: &str,
    file_name: &str,
    path: &Path,
) -> Result<WasmPluginCliResourceUploadDto, String> {
    let bytes =
        fs::read(path).map_err(|err| format!("read CLI resource {}: {err}", path.display()))?;
    Ok(WasmPluginCliResourceUploadDto {
        command_name: command_name.to_string(),
        file_name: file_name.to_string(),
        bytes,
        content_type: Some("text/x-shellscript".to_string()),
    })
}

fn standalone_autohost_script() -> Result<String, String> {
    let source_path = Path::new("/Users/zjarlin/.config/shell/rc.d/22-autohost.sh");
    let source = fs::read_to_string(source_path)
        .map_err(|err| format!("read autohost source {}: {err}", source_path.display()))?;
    Ok(format!(
        "{source}\n\nif [ \"${{BASH_SOURCE[0]:-$0}}\" = \"$0\" ]; then\n  autohost \"$@\"\nfi\n"
    ))
}

fn lifecycle_only_wasm() -> Vec<u8> {
    let exports = [
        "aio_on_load",
        "aio_on_enable",
        "aio_on_disable",
        "aio_on_unload",
    ];
    let header = b"\0asm"
        .iter()
        .copied()
        .chain([1, 0, 0, 0])
        .collect::<Vec<_>>();
    let type_section = wasm_section(1, wasm_vec(vec![vec![0x60, 0x00, 0x01, 0x7f]]));
    let function_section = wasm_section(3, wasm_vec(exports.iter().map(|_| vec![0]).collect()));
    let export_section = wasm_section(
        7,
        wasm_vec(
            exports
                .iter()
                .enumerate()
                .map(|(index, name)| {
                    let mut entry = wasm_name(name);
                    entry.push(0x00);
                    entry.extend(encode_u32(index as u32));
                    entry
                })
                .collect(),
        ),
    );
    let body = vec![0x00, 0x41, 0x00, 0x0b];
    let code_section = wasm_section(
        10,
        wasm_vec(
            exports
                .iter()
                .map(|_| {
                    let mut item = encode_u32(body.len() as u32);
                    item.extend(body.clone());
                    item
                })
                .collect(),
        ),
    );
    [
        header,
        type_section,
        function_section,
        export_section,
        code_section,
    ]
    .concat()
}

fn wasm_section(section_id: u8, payload: Vec<u8>) -> Vec<u8> {
    let mut section = vec![section_id];
    section.extend(encode_u32(payload.len() as u32));
    section.extend(payload);
    section
}

fn wasm_vec(items: Vec<Vec<u8>>) -> Vec<u8> {
    let mut out = encode_u32(items.len() as u32);
    for item in items {
        out.extend(item);
    }
    out
}

fn wasm_name(value: &str) -> Vec<u8> {
    let bytes = value.as_bytes();
    let mut out = encode_u32(bytes.len() as u32);
    out.extend(bytes);
    out
}

fn encode_u32(mut value: u32) -> Vec<u8> {
    let mut out = Vec::new();
    loop {
        let mut byte = (value & 0x7f) as u8;
        value >>= 7;
        if value != 0 {
            byte |= 0x80;
        }
        out.push(byte);
        if value == 0 {
            return out;
        }
    }
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
