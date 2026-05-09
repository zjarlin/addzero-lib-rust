#![cfg(not(target_arch = "wasm32"))]

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use az_plugin_contract::{
    MarketplaceSnapshot, PluginKind, ResolvedPage, RuntimeOverview, ShellSnapshot,
};
use az_plugin_kernel::PlatformKernel;
use az_plugin_runtime::{read_manifest_from_package, validate_package};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};

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
pub struct WasmPluginUploadResultDto {
    pub package_path: String,
    pub plugin_id: String,
    pub plugin_name: String,
    pub version: String,
    pub validated: bool,
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
    let kernel = platform_kernel()?;
    let file_name = normalized_upload_file_name(&input.file_name)?;
    if input.bytes.is_empty() {
        return Err("plugin upload bytes cannot be empty".to_string());
    }

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
        })
    })();

    let _ = fs::remove_file(&temp_path);
    result
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
