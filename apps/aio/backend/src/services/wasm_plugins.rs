#![cfg(not(target_arch = "wasm32"))]

use std::path::{Path, PathBuf};
use std::sync::Arc;

use addzero_plugin_contract::{MarketplaceSnapshot, ResolvedPage, RuntimeOverview, ShellSnapshot};
use addzero_plugin_kernel::PlatformKernel;
use addzero_plugin_runtime::read_manifest_from_package;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct WasmPluginRuntimeSnapshotDto {
    pub shell: ShellSnapshot,
    pub marketplace: MarketplaceSnapshot,
    pub runtime: RuntimeOverview,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct WasmPluginRegisterDevRequestDto {
    pub source_dir: String,
    pub package_name: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct WasmPluginRegisterDevResultDto {
    pub package_path: String,
    pub plugin_id: String,
    pub plugin_name: String,
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
    addzero_system_starters::link_all();
    PlatformKernel::new(default_catalog_dir(), default_package_root())
        .map(Arc::new)
        .map_err(|err| format!("init wasm plugin kernel: {err}"))
});

pub async fn wasm_plugin_runtime_snapshot_on_server() -> Result<WasmPluginRuntimeSnapshotDto, String>
{
    let kernel = platform_kernel()?;
    let shell = kernel
        .shell_snapshot()
        .map_err(|err| format!("build shell snapshot: {err}"))?;
    let marketplace = kernel
        .marketplace_snapshot()
        .map_err(|err| format!("build marketplace snapshot: {err}"))?;
    let runtime = kernel
        .runtime_overview()
        .map_err(|err| format!("build runtime overview: {err}"))?;
    Ok(WasmPluginRuntimeSnapshotDto {
        shell,
        marketplace,
        runtime,
    })
}

pub async fn register_dev_wasm_plugin_on_server(
    input: WasmPluginRegisterDevRequestDto,
) -> Result<WasmPluginRegisterDevResultDto, String> {
    let kernel = platform_kernel()?;
    let source_dir = normalize_existing_dir(&input.source_dir)?;
    let package_name = normalized_package_name(&input.package_name, &source_dir)?;
    let package_path = kernel
        .ensure_dev_package(&source_dir, &package_name)
        .map_err(|err| format!("package dev plugin `{package_name}`: {err}"))?;
    let manifest = read_manifest_from_package(&package_path)
        .map_err(|err| format!("read packaged manifest {}: {err}", package_path.display()))?;
    Ok(WasmPluginRegisterDevResultDto {
        package_path: package_path.display().to_string(),
        plugin_id: manifest.descriptor.id,
        plugin_name: manifest.descriptor.name,
    })
}

pub async fn install_catalog_wasm_plugin_on_server(
    input: WasmPluginInstallRequestDto,
) -> Result<WasmPluginInstallResultDto, String> {
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
    if current.status != addzero_plugin_contract::PluginStatus::Installed {
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

pub async fn resolve_system_wasm_plugin_page_on_server(
    plugin_id: String,
    page_id: String,
) -> Result<ResolvedPage, String> {
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
    let kernel = platform_kernel()?;
    kernel
        .resolve_instance_page(&instance_slug, &page_id)
        .map_err(|err| format!("resolve instance page `{instance_slug}/{page_id}`: {err}"))?
        .ok_or_else(|| format!("instance page `{instance_slug}/{page_id}` was not found"))
}

fn platform_kernel() -> Result<&'static Arc<PlatformKernel>, String> {
    PLATFORM_KERNEL.as_ref().map_err(Clone::clone)
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

fn normalize_existing_dir(raw: &str) -> Result<PathBuf, String> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Err("source_dir is required".to_string());
    }
    let path = PathBuf::from(trimmed);
    if !path.exists() {
        return Err(format!("source dir does not exist: {}", path.display()));
    }
    if !path.is_dir() {
        return Err(format!("source dir is not a directory: {}", path.display()));
    }
    path.canonicalize()
        .map_err(|err| format!("canonicalize source dir {}: {err}", path.display()))
}

fn normalized_package_name(raw: &str, source_dir: &Path) -> Result<String, String> {
    let candidate = raw
        .trim()
        .strip_suffix(".azplugin")
        .unwrap_or(raw.trim())
        .trim();
    let fallback = source_dir
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or("plugin");
    let chosen = if candidate.is_empty() {
        fallback
    } else {
        candidate
    };
    let normalized = chosen
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_' | '.') {
                ch
            } else {
                '-'
            }
        })
        .collect::<String>()
        .trim_matches('-')
        .to_string();
    if normalized.is_empty() {
        return Err("package_name resolves to an empty identifier".to_string());
    }
    Ok(normalized)
}
