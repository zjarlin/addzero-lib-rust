use std::{
    path::{Path, PathBuf},
    rc::Rc,
};

#[cfg(not(target_arch = "wasm32"))]
use std::{collections::BTreeMap, ffi::OsStr, sync::Arc};

#[cfg(not(target_arch = "wasm32"))]
use chrono::Utc;
#[cfg(not(target_arch = "wasm32"))]
use std::sync::OnceLock;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[cfg(not(target_arch = "wasm32"))]
use tokio::sync::broadcast;

pub use super::LocalBoxFuture;

pub const PACKAGE_BUCKET: &str = "msc-aio";
pub const PACKAGE_RELATIVE_PREFIX: &str = "installation";
pub const PACKAGE_LOCAL_ROOT_ENV: &str = "ADMIN_PACKAGE_LOCAL_ROOT";
pub const DEFAULT_PACKAGE_LOCAL_ROOT_INPUT: &str = "~/Nextcloud";

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PackageStorageBackendDto {
    LocalFs,
    Minio,
}

impl PackageStorageBackendDto {
    pub fn label(self) -> &'static str {
        match self {
            Self::LocalFs => "本地目录",
            Self::Minio => "MinIO / S3",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PackageUploadRequest {
    pub bucket: String,
    pub relative_path_prefix: String,
    #[serde(default = "default_package_backend")]
    pub backend: PackageStorageBackendDto,
}

impl Default for PackageUploadRequest {
    fn default() -> Self {
        Self {
            bucket: PACKAGE_BUCKET.to_string(),
            relative_path_prefix: PACKAGE_RELATIVE_PREFIX.to_string(),
            backend: default_package_backend(),
        }
    }
}

fn default_package_backend() -> PackageStorageBackendDto {
    PackageStorageBackendDto::LocalFs
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct BrowserPackageImportItemDto {
    pub file_name: String,
    pub content_type: Option<String>,
    #[serde(with = "base64_bytes")]
    pub bytes: Vec<u8>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct BrowserPackageImportRequest {
    #[serde(default = "default_package_backend")]
    pub backend: PackageStorageBackendDto,
    #[serde(default)]
    pub relative_path_prefix: Option<String>,
    pub items: Vec<BrowserPackageImportItemDto>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PackageUploadItemDto {
    pub file_name: String,
    pub local_path: String,
    pub relative_path: Option<String>,
    pub download_url: Option<String>,
    pub content_hash: Option<String>,
    pub hash_algorithm: Option<String>,
    pub size_bytes: u64,
    pub status: String,
    pub error: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PackageUploadReportDto {
    pub bucket: String,
    pub relative_path_prefix: String,
    pub backend: PackageStorageBackendDto,
    pub backend_label: String,
    pub scanned_roots: Vec<String>,
    pub uploaded_count: usize,
    pub failed_count: usize,
    pub items: Vec<PackageUploadItemDto>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PackageStorageOverviewDto {
    pub backend: PackageStorageBackendDto,
    pub backend_label: String,
    pub local_root: Option<String>,
    pub local_root_input: Option<String>,
    pub bucket: Option<String>,
    pub relative_path_prefix: String,
    pub env_file_path: Option<String>,
    pub effective_source: Option<String>,
    pub warnings: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PackageStorageConfigUpsertDto {
    pub local_root_input: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PackageEventKindDto {
    ImportStarted,
    ImportFinished,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PackageEventDto {
    pub kind: PackageEventKindDto,
    pub message: String,
    pub report: Option<PackageUploadReportDto>,
    pub emitted_at: String,
}

#[cfg(not(target_arch = "wasm32"))]
static PACKAGE_EVENT_BUS: OnceLock<broadcast::Sender<PackageEventDto>> = OnceLock::new();

#[derive(Clone, Debug, Error, PartialEq, Eq)]
pub enum PackageStorageError {
    #[error("{0}")]
    Message(String),
}

impl PackageStorageError {
    fn new(message: impl Into<String>) -> Self {
        Self::Message(message.into())
    }
}

pub type PackageStorageResult<T> = Result<T, PackageStorageError>;

#[cfg(not(target_arch = "wasm32"))]
pub fn subscribe_package_events() -> broadcast::Receiver<PackageEventDto> {
    package_event_sender().subscribe()
}

#[cfg(not(target_arch = "wasm32"))]
fn emit_package_event(kind: PackageEventKindDto, message: impl Into<String>, report: Option<PackageUploadReportDto>) {
    let _ = package_event_sender().send(PackageEventDto {
        kind,
        message: message.into(),
        report,
        emitted_at: Utc::now().to_rfc3339(),
    });
}

#[cfg(not(target_arch = "wasm32"))]
fn package_event_sender() -> &'static broadcast::Sender<PackageEventDto> {
    PACKAGE_EVENT_BUS.get_or_init(|| {
        let (sender, _) = broadcast::channel(64);
        sender
    })
}

pub trait PackageStorageApi: 'static {
    fn scan_and_upload_installers(
        &self,
        input: PackageUploadRequest,
    ) -> LocalBoxFuture<'_, PackageStorageResult<PackageUploadReportDto>>;

    fn import_browser_files(
        &self,
        input: BrowserPackageImportRequest,
    ) -> LocalBoxFuture<'_, PackageStorageResult<PackageUploadReportDto>>;

    fn storage_overview(&self) -> LocalBoxFuture<'_, PackageStorageResult<PackageStorageOverviewDto>>;

    fn save_storage_config(
        &self,
        input: PackageStorageConfigUpsertDto,
    ) -> LocalBoxFuture<'_, PackageStorageResult<PackageStorageOverviewDto>>;
}

pub type SharedPackageStorageApi = Rc<dyn PackageStorageApi>;

pub fn default_package_storage_api() -> SharedPackageStorageApi {
    #[cfg(target_arch = "wasm32")]
    {
        Rc::new(BrowserPackageStorageApi)
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        Rc::new(NativePackageStorage::from_env())
    }
}

#[cfg(target_arch = "wasm32")]
struct BrowserPackageStorageApi;

#[cfg(target_arch = "wasm32")]
impl PackageStorageApi for BrowserPackageStorageApi {
    fn scan_and_upload_installers(
        &self,
        input: PackageUploadRequest,
    ) -> LocalBoxFuture<'_, PackageStorageResult<PackageUploadReportDto>> {
        Box::pin(async move {
            super::browser_http::post_json("/api/admin/storage/packages/scan-upload", &input)
                .await
                .map_err(PackageStorageError::new)
        })
    }

    fn import_browser_files(
        &self,
        input: BrowserPackageImportRequest,
    ) -> LocalBoxFuture<'_, PackageStorageResult<PackageUploadReportDto>> {
        Box::pin(async move {
            super::browser_http::post_json("/api/admin/storage/packages/import", &input)
                .await
                .map_err(PackageStorageError::new)
        })
    }

    fn storage_overview(
        &self,
    ) -> LocalBoxFuture<'_, PackageStorageResult<PackageStorageOverviewDto>> {
        Box::pin(async move {
            super::browser_http::get_json("/api/admin/storage/packages/overview")
                .await
                .map_err(PackageStorageError::new)
        })
    }

    fn save_storage_config(
        &self,
        input: PackageStorageConfigUpsertDto,
    ) -> LocalBoxFuture<'_, PackageStorageResult<PackageStorageOverviewDto>> {
        Box::pin(async move {
            super::browser_http::post_json("/api/admin/storage/packages/config", &input)
                .await
                .map_err(PackageStorageError::new)
        })
    }
}

#[cfg(not(target_arch = "wasm32"))]
struct NativePackageStorage {
    backend: NativeBackend,
}

#[cfg(not(target_arch = "wasm32"))]
enum NativeBackend {
    Rustfs(RustfsBackend),
    Local(LocalFsBackend),
    MissingConfig(String),
}

#[cfg(not(target_arch = "wasm32"))]
impl NativePackageStorage {
    fn from_env() -> Self {
        if let Ok(local) = LocalFsBackend::from_env() {
            return Self {
                backend: NativeBackend::Local(local),
            };
        }

        match RustfsBackend::from_env() {
            Ok(backend) => Self {
                backend: NativeBackend::Rustfs(backend),
            },
            Err(reason) => Self {
                backend: NativeBackend::MissingConfig(reason),
            },
        }
    }

    fn scan_and_upload_installers_blocking(
        &self,
        input: PackageUploadRequest,
    ) -> PackageStorageResult<PackageUploadReportDto> {
        match (&self.backend, input.backend) {
            (NativeBackend::Local(backend), PackageStorageBackendDto::LocalFs) => {
                backend.scan_and_import_installers_blocking(input)
            }
            (NativeBackend::Rustfs(backend), PackageStorageBackendDto::Minio) => {
                backend.scan_and_upload_installers_blocking(input)
            }
            (NativeBackend::Local(backend), PackageStorageBackendDto::Minio) => {
                backend.scan_and_import_installers_blocking(PackageUploadRequest {
                    backend: PackageStorageBackendDto::LocalFs,
                    ..input
                })
            }
            (NativeBackend::Rustfs(backend), PackageStorageBackendDto::LocalFs) => {
                let _ = backend;
                Err(PackageStorageError::new(
                    "当前未配置本地安装包目录 backend，请设置 ADMIN_PACKAGE_LOCAL_ROOT",
                ))
            }
            (NativeBackend::MissingConfig(reason), PackageStorageBackendDto::Minio) => {
                Err(PackageStorageError::new(format!("MinIO / S3 存储未配置：{reason}")))
            }
            (NativeBackend::MissingConfig(_), PackageStorageBackendDto::LocalFs) => Err(
                PackageStorageError::new(
                    "当前未配置本地安装包目录 backend，请设置 ADMIN_PACKAGE_LOCAL_ROOT",
                ),
            ),
        }
    }

    fn import_browser_files_blocking(
        &self,
        input: BrowserPackageImportRequest,
    ) -> PackageStorageResult<PackageUploadReportDto> {
        match (&self.backend, input.backend) {
            (NativeBackend::Local(backend), PackageStorageBackendDto::LocalFs) => {
                backend.import_browser_files_blocking(input)
            }
            (NativeBackend::Rustfs(_), PackageStorageBackendDto::LocalFs) => Err(
                PackageStorageError::new(
                    "当前未配置本地安装包目录 backend，请设置 ADMIN_PACKAGE_LOCAL_ROOT",
                ),
            ),
            (NativeBackend::Rustfs(_), PackageStorageBackendDto::Minio) => Err(
                PackageStorageError::new("浏览器拖拽导入暂不支持直接落 MinIO，请先使用本地目录 backend"),
            ),
            (NativeBackend::Local(_), PackageStorageBackendDto::Minio) => Err(
                PackageStorageError::new("浏览器拖拽导入暂不支持直接落 MinIO，请先使用本地目录 backend"),
            ),
            (NativeBackend::MissingConfig(reason), PackageStorageBackendDto::Minio) => Err(
                PackageStorageError::new(format!("MinIO / S3 存储未配置：{reason}")),
            ),
            (NativeBackend::MissingConfig(_), PackageStorageBackendDto::LocalFs) => Err(
                PackageStorageError::new(
                    "当前未配置本地安装包目录 backend，请设置 ADMIN_PACKAGE_LOCAL_ROOT",
                ),
            ),
        }
    }

    fn storage_overview_blocking(&self) -> PackageStorageResult<PackageStorageOverviewDto> {
        match &self.backend {
            NativeBackend::Local(backend) => Ok(backend.overview()),
            NativeBackend::Rustfs(backend) => Ok(backend.overview()),
            NativeBackend::MissingConfig(reason) => Err(PackageStorageError::new(format!(
                "安装包存储未配置：{reason}"
            ))),
        }
    }

    fn save_storage_config_blocking(
        &self,
        input: PackageStorageConfigUpsertDto,
    ) -> PackageStorageResult<PackageStorageOverviewDto> {
        save_package_local_root_input(&input.local_root_input)?;
        NativePackageStorage::from_env().storage_overview_blocking()
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl PackageStorageApi for NativePackageStorage {
    fn scan_and_upload_installers(
        &self,
        input: PackageUploadRequest,
    ) -> LocalBoxFuture<'_, PackageStorageResult<PackageUploadReportDto>> {
        let result = self.scan_and_upload_installers_blocking(input);
        Box::pin(async move { result })
    }

    fn import_browser_files(
        &self,
        input: BrowserPackageImportRequest,
    ) -> LocalBoxFuture<'_, PackageStorageResult<PackageUploadReportDto>> {
        let result = self.import_browser_files_blocking(input);
        Box::pin(async move { result })
    }

    fn storage_overview(
        &self,
    ) -> LocalBoxFuture<'_, PackageStorageResult<PackageStorageOverviewDto>> {
        let result = self.storage_overview_blocking();
        Box::pin(async move { result })
    }

    fn save_storage_config(
        &self,
        input: PackageStorageConfigUpsertDto,
    ) -> LocalBoxFuture<'_, PackageStorageResult<PackageStorageOverviewDto>> {
        let result = self.save_storage_config_blocking(input);
        Box::pin(async move { result })
    }
}

#[cfg(not(target_arch = "wasm32"))]
struct RustfsBackend {
    client: Arc<dyn addzero_rustfs::S3StorageClient>,
    default_bucket: String,
}

#[cfg(not(target_arch = "wasm32"))]
struct LocalFsBackend {
    root: PathBuf,
    root_input: String,
    source_label: String,
    env_file_path: Option<String>,
    warnings: Vec<String>,
}

#[cfg(not(target_arch = "wasm32"))]
impl RustfsBackend {
    fn from_env() -> Result<Self, String> {
        let endpoint =
            read_env_with_fallback("ADMIN_PACKAGE_S3_ENDPOINT", "ADMIN_LOGO_S3_ENDPOINT")?;
        let access_key =
            read_env_with_fallback("ADMIN_PACKAGE_S3_ACCESS_KEY", "ADMIN_LOGO_S3_ACCESS_KEY")?;
        let secret_key =
            read_env_with_fallback("ADMIN_PACKAGE_S3_SECRET_KEY", "ADMIN_LOGO_S3_SECRET_KEY")?;
        let bucket = std::env::var("ADMIN_PACKAGE_S3_BUCKET")
            .ok()
            .filter(|value| !value.trim().is_empty())
            .unwrap_or_else(|| PACKAGE_BUCKET.to_string());
        let region = std::env::var("ADMIN_PACKAGE_S3_REGION")
            .ok()
            .filter(|value| !value.trim().is_empty())
            .or_else(|| std::env::var("ADMIN_LOGO_S3_REGION").ok())
            .unwrap_or_else(|| "us-east-1".into());
        let config = addzero_rustfs::S3ClientConfig::new(endpoint, access_key, secret_key)
            .with_region(region)
            .with_path_style_access(true);

        Ok(Self {
            client: addzero_rustfs::create_storage_client(config),
            default_bucket: bucket,
        })
    }

    fn scan_and_upload_installers_blocking(
        &self,
        input: PackageUploadRequest,
    ) -> PackageStorageResult<PackageUploadReportDto> {
        let bucket = normalized_bucket(&input.bucket, &self.default_bucket)?;
        let relative_prefix = normalized_prefix(&input.relative_path_prefix);
        let scanned_roots = installer_roots();
        let installer_files = discover_installer_files(&scanned_roots);

        addzero_rustfs::ensure_bucket(self.client.as_ref(), &bucket)
            .map_err(|err| PackageStorageError::new(format!("创建 bucket 失败：{err}")))?;

        let mut items = Vec::with_capacity(installer_files.len());
        let mut uploaded_count = 0usize;
        let mut failed_count = 0usize;

        for path in installer_files {
            let file_name = path
                .file_name()
                .and_then(OsStr::to_str)
                .unwrap_or_default()
                .to_string();
            let size_bytes = std::fs::metadata(&path)
                .map(|meta| meta.len())
                .unwrap_or_default();
            let content_hash = match super::asset_graph::blake3_file_hex(&path) {
                Ok(hash) => hash,
                Err(err) => {
                    failed_count += 1;
                    items.push(PackageUploadItemDto {
                        file_name,
                        local_path: path.display().to_string(),
                        relative_path: None,
                        download_url: None,
                        content_hash: None,
                        hash_algorithm: Some("blake3".to_string()),
                        size_bytes,
                        status: "计算 hash 失败".to_string(),
                        error: Some(err.to_string()),
                    });
                    continue;
                }
            };
            match run_asset_async(super::asset_graph::existing_package_by_hash(
                "blake3",
                &content_hash,
            )) {
                Ok(Some(existing)) => {
                    uploaded_count += 1;
                    items.push(PackageUploadItemDto {
                        file_name,
                        local_path: path.display().to_string(),
                        relative_path: existing.relative_path,
                        download_url: existing.download_url,
                        content_hash: Some(content_hash),
                        hash_algorithm: Some("blake3".to_string()),
                        size_bytes,
                        status: "已存在".to_string(),
                        error: None,
                    });
                    continue;
                }
                Ok(None) => {}
                Err(err) => {
                    failed_count += 1;
                    items.push(PackageUploadItemDto {
                        file_name,
                        local_path: path.display().to_string(),
                        relative_path: None,
                        download_url: None,
                        content_hash: Some(content_hash),
                        hash_algorithm: Some("blake3".to_string()),
                        size_bytes,
                        status: "查询 hash 失败".to_string(),
                        error: Some(err.to_string()),
                    });
                    continue;
                }
            }

            let object_key = super::asset_graph::build_package_object_key(
                &relative_prefix,
                &content_hash,
                &file_name,
            );
            let relative_path = super::asset_graph::build_relative_path(&bucket, &object_key);
            let download_url = super::asset_graph::build_download_url(&relative_path);
            let content_type = addzero_rustfs::guess_content_type(&path);
            let mut metadata = BTreeMap::new();
            metadata.insert("hash-algorithm".to_string(), "blake3".to_string());
            metadata.insert("content-hash".to_string(), content_hash.clone());
            metadata.insert("source-file-name".to_string(), file_name.clone());

            let upload_result = match self.client.object_exists(&bucket, &object_key) {
                Ok(true) => Ok(()),
                Ok(false) => self.client.put_object_file(
                    &bucket,
                    &object_key,
                    &path,
                    Some(content_type.as_str()),
                    &metadata,
                ),
                Err(err) => Err(err),
            };

            match upload_result {
                Ok(()) => {
                    let record = super::asset_graph::AssetRecordInput {
                        id: format!("package-blake3-{content_hash}"),
                        kind: super::asset_graph::AssetKindDto::Package,
                        title: file_name.clone(),
                        detail: path.display().to_string(),
                        source: "MinIO 安装包上传".to_string(),
                        local_path: Some(path.display().to_string()),
                        relative_path: Some(relative_path.clone()),
                        download_url: Some(download_url.clone()),
                        content_hash: Some(content_hash.clone()),
                        hash_algorithm: Some("blake3".to_string()),
                        size_bytes: Some(size_bytes),
                        tags: vec![
                            "安装包".to_string(),
                            package_format_tag(&path),
                            "MinIO".to_string(),
                        ],
                        raw: serde_json::json!({
                            "bucket": bucket.clone(),
                            "object_key": object_key.clone(),
                            "file_name": file_name.clone(),
                        }),
                    };
                    if let Err(err) =
                        run_asset_async(super::asset_graph::upsert_asset_record_on_server(record))
                    {
                        failed_count += 1;
                        items.push(PackageUploadItemDto {
                            file_name,
                            local_path: path.display().to_string(),
                            relative_path: Some(relative_path),
                            download_url: Some(download_url),
                            content_hash: Some(content_hash),
                            hash_algorithm: Some("blake3".to_string()),
                            size_bytes,
                            status: "上传成功但写入 PG 失败".to_string(),
                            error: Some(err.to_string()),
                        });
                        continue;
                    }

                    uploaded_count += 1;
                    items.push(PackageUploadItemDto {
                        file_name,
                        local_path: path.display().to_string(),
                        relative_path: Some(relative_path),
                        download_url: Some(download_url),
                        content_hash: Some(content_hash),
                        hash_algorithm: Some("blake3".to_string()),
                        size_bytes,
                        status: "已上传".to_string(),
                        error: None,
                    });
                }
                Err(err) => {
                    failed_count += 1;
                    items.push(PackageUploadItemDto {
                        file_name,
                        local_path: path.display().to_string(),
                        relative_path: None,
                        download_url: None,
                        content_hash: Some(content_hash),
                        hash_algorithm: Some("blake3".to_string()),
                        size_bytes,
                        status: "上传失败".to_string(),
                        error: Some(err.to_string()),
                    });
                }
            }
        }

        Ok(PackageUploadReportDto {
            bucket,
            relative_path_prefix: relative_prefix,
            backend: PackageStorageBackendDto::Minio,
            backend_label: self.overview().backend_label,
            scanned_roots: scanned_roots
                .into_iter()
                .map(|path| path.display().to_string())
                .collect(),
            uploaded_count,
            failed_count,
            items,
        })
    }

    fn overview(&self) -> PackageStorageOverviewDto {
        PackageStorageOverviewDto {
            backend: PackageStorageBackendDto::Minio,
            backend_label: format!("MinIO / S3-compatible · bucket `{}`", self.default_bucket),
            local_root: None,
            local_root_input: None,
            bucket: Some(self.default_bucket.clone()),
            relative_path_prefix: PACKAGE_RELATIVE_PREFIX.to_string(),
            env_file_path: addzero_persistence::local_env_path()
                .map(|path| path.display().to_string()),
            effective_source: Some("环境变量 / 对象存储配置".to_string()),
            warnings: vec![
                "当前对象存储只保留兼容能力；安装包正式路径请优先配置本地目录。".to_string(),
            ],
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl LocalFsBackend {
    fn from_env() -> Result<Self, String> {
        let resolved = resolve_local_package_root()?;
        let root = resolved.expanded_path;

        std::fs::create_dir_all(&root)
            .map_err(|err| format!("创建本地安装包目录失败 `{}`: {err}", root.display()))?;

        Ok(Self {
            root,
            root_input: resolved.raw_input,
            source_label: resolved.source_label,
            env_file_path: resolved.env_file_path,
            warnings: resolved.warnings,
        })
    }

    fn overview(&self) -> PackageStorageOverviewDto {
        PackageStorageOverviewDto {
            backend: PackageStorageBackendDto::LocalFs,
            backend_label: format!("本地目录 · {}", self.root.display()),
            local_root: Some(self.root.display().to_string()),
            local_root_input: Some(self.root_input.clone()),
            bucket: None,
            relative_path_prefix: PACKAGE_RELATIVE_PREFIX.to_string(),
            env_file_path: self.env_file_path.clone(),
            effective_source: Some(self.source_label.clone()),
            warnings: self.warnings.clone(),
        }
    }

    fn scan_and_import_installers_blocking(
        &self,
        input: PackageUploadRequest,
    ) -> PackageStorageResult<PackageUploadReportDto> {
        emit_package_event(
            PackageEventKindDto::ImportStarted,
            format!("开始扫描本机安装包并归档到 {}", self.root.display()),
            None,
        );
        let relative_prefix = normalized_prefix(&input.relative_path_prefix);
        let scanned_roots = installer_roots();
        let installer_files = discover_installer_files(&scanned_roots);

        let mut items = Vec::with_capacity(installer_files.len());
        let mut uploaded_count = 0usize;
        let mut failed_count = 0usize;

        for path in installer_files {
            match self.import_file_from_path(&path, &relative_prefix) {
                Ok(item) => {
                    uploaded_count += 1;
                    items.push(item);
                }
                Err(err) => {
                    failed_count += 1;
                    let file_name = path
                        .file_name()
                        .and_then(OsStr::to_str)
                        .unwrap_or_default()
                        .to_string();
                    let size_bytes = std::fs::metadata(&path)
                        .map(|meta| meta.len())
                        .unwrap_or_default();
                    items.push(PackageUploadItemDto {
                        file_name,
                        local_path: path.display().to_string(),
                        relative_path: None,
                        download_url: None,
                        content_hash: None,
                        hash_algorithm: Some("blake3".to_string()),
                        size_bytes,
                        status: "导入失败".to_string(),
                        error: Some(err.to_string()),
                    });
                }
            }
        }

        let report = PackageUploadReportDto {
            bucket: String::new(),
            relative_path_prefix: relative_prefix,
            backend: PackageStorageBackendDto::LocalFs,
            backend_label: self.overview().backend_label,
            scanned_roots: scanned_roots
                .into_iter()
                .map(|path| path.display().to_string())
                .collect(),
            uploaded_count,
            failed_count,
            items,
        };
        emit_package_event(
            PackageEventKindDto::ImportFinished,
            format!(
                "本地安装包归档完成：{} 成功，{} 失败",
                report.uploaded_count, report.failed_count
            ),
            Some(report.clone()),
        );
        Ok(report)
    }

    fn import_browser_files_blocking(
        &self,
        input: BrowserPackageImportRequest,
    ) -> PackageStorageResult<PackageUploadReportDto> {
        emit_package_event(
            PackageEventKindDto::ImportStarted,
            format!("开始接收浏览器拖拽文件到 {}", self.root.display()),
            None,
        );
        let relative_prefix = input
            .relative_path_prefix
            .as_deref()
            .map(normalized_prefix)
            .unwrap_or_else(|| PACKAGE_RELATIVE_PREFIX.to_string());
        let mut items = Vec::with_capacity(input.items.len());
        let mut uploaded_count = 0usize;
        let mut failed_count = 0usize;

        for item in input.items {
            match self.import_browser_item(&item, &relative_prefix) {
                Ok(row) => {
                    uploaded_count += 1;
                    items.push(row);
                }
                Err(err) => {
                    failed_count += 1;
                    items.push(PackageUploadItemDto {
                        file_name: item.file_name.clone(),
                        local_path: self.root.display().to_string(),
                        relative_path: None,
                        download_url: None,
                        content_hash: None,
                        hash_algorithm: Some("blake3".to_string()),
                        size_bytes: u64::try_from(item.bytes.len()).unwrap_or_default(),
                        status: "导入失败".to_string(),
                        error: Some(err.to_string()),
                    });
                }
            }
        }

        let report = PackageUploadReportDto {
            bucket: String::new(),
            relative_path_prefix: relative_prefix,
            backend: PackageStorageBackendDto::LocalFs,
            backend_label: self.overview().backend_label,
            scanned_roots: vec![self.root.display().to_string()],
            uploaded_count,
            failed_count,
            items,
        };
        emit_package_event(
            PackageEventKindDto::ImportFinished,
            format!(
                "浏览器拖拽导入完成：{} 成功，{} 失败",
                report.uploaded_count, report.failed_count
            ),
            Some(report.clone()),
        );
        Ok(report)
    }

    fn import_browser_item(
        &self,
        item: &BrowserPackageImportItemDto,
        relative_prefix: &str,
    ) -> PackageStorageResult<PackageUploadItemDto> {
        validate_installer_name(&item.file_name)?;
        if item.bytes.is_empty() {
            return Err(PackageStorageError::new("拖拽的文件为空"));
        }

        let content_hash = blake3::hash(&item.bytes).to_hex().to_string();
        if let Some(existing) = run_asset_async(super::asset_graph::existing_package_by_hash(
            "blake3",
            &content_hash,
        ))
        .map_err(|err| PackageStorageError::new(err.to_string()))?
        {
            return Ok(PackageUploadItemDto {
                file_name: item.file_name.clone(),
                local_path: existing.local_path.unwrap_or_else(|| self.root.display().to_string()),
                relative_path: existing.relative_path,
                download_url: existing.download_url,
                content_hash: Some(content_hash),
                hash_algorithm: Some("blake3".to_string()),
                size_bytes: u64::try_from(item.bytes.len()).unwrap_or_default(),
                status: "已存在".to_string(),
                error: None,
            });
        }

        let final_path = build_local_package_path(&self.root, relative_prefix, &content_hash, &item.file_name);
        if let Some(parent) = final_path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|err| PackageStorageError::new(format!("创建目录失败：{err}")))?;
        }
        std::fs::write(&final_path, &item.bytes)
            .map_err(|err| PackageStorageError::new(format!("写入本地安装包失败：{err}")))?;

        register_local_package_asset(
            &self.root,
            &final_path,
            &content_hash,
            "浏览器拖拽导入",
            relative_prefix,
            Some(serde_json::json!({
                "content_type": item.content_type,
                "imported_via": "browser_drag_drop",
            })),
        )?;

        Ok(PackageUploadItemDto {
            file_name: item.file_name.clone(),
            local_path: final_path.display().to_string(),
            relative_path: Some(path_relative_to_root(&self.root, &final_path)),
            download_url: None,
            content_hash: Some(content_hash),
            hash_algorithm: Some("blake3".to_string()),
            size_bytes: u64::try_from(item.bytes.len()).unwrap_or_default(),
            status: "已导入本地目录".to_string(),
            error: None,
        })
    }

    fn import_file_from_path(
        &self,
        source_path: &Path,
        relative_prefix: &str,
    ) -> PackageStorageResult<PackageUploadItemDto> {
        let file_name = source_path
            .file_name()
            .and_then(OsStr::to_str)
            .unwrap_or_default()
            .to_string();
        validate_installer_name(&file_name)?;
        let size_bytes = std::fs::metadata(source_path)
            .map(|meta| meta.len())
            .unwrap_or_default();
        let content_hash = super::asset_graph::blake3_file_hex(source_path)
            .map_err(|err| PackageStorageError::new(err.to_string()))?;

        if let Some(existing) = run_asset_async(super::asset_graph::existing_package_by_hash(
            "blake3",
            &content_hash,
        ))
        .map_err(|err| PackageStorageError::new(err.to_string()))?
        {
            return Ok(PackageUploadItemDto {
                file_name,
                local_path: existing.local_path.unwrap_or_else(|| source_path.display().to_string()),
                relative_path: existing.relative_path,
                download_url: existing.download_url,
                content_hash: Some(content_hash),
                hash_algorithm: Some("blake3".to_string()),
                size_bytes,
                status: "已存在".to_string(),
                error: None,
            });
        }

        let final_path = build_local_package_path(&self.root, relative_prefix, &content_hash, &file_name);
        if source_path != final_path {
            if let Some(parent) = final_path.parent() {
                std::fs::create_dir_all(parent)
                    .map_err(|err| PackageStorageError::new(format!("创建目录失败：{err}")))?;
            }
            std::fs::copy(source_path, &final_path)
                .map_err(|err| PackageStorageError::new(format!("复制安装包失败：{err}")))?;
        }

        register_local_package_asset(
            &self.root,
            &final_path,
            &content_hash,
            "本地安装包归档",
            relative_prefix,
            Some(serde_json::json!({
                "source_path": source_path.display().to_string(),
                "imported_via": "scanner",
            })),
        )?;

        Ok(PackageUploadItemDto {
            file_name,
            local_path: final_path.display().to_string(),
            relative_path: Some(path_relative_to_root(&self.root, &final_path)),
            download_url: None,
            content_hash: Some(content_hash),
            hash_algorithm: Some("blake3".to_string()),
            size_bytes,
            status: "已归档到本地目录".to_string(),
            error: None,
        })
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn read_env_with_fallback(primary: &str, fallback: &str) -> Result<String, String> {
    std::env::var(primary)
        .ok()
        .filter(|value| !value.trim().is_empty())
        .or_else(|| {
            std::env::var(fallback)
                .ok()
                .filter(|value| !value.trim().is_empty())
        })
        .map(|value| value.trim().to_string())
        .ok_or_else(|| format!("缺少环境变量 `{primary}`，且 `{fallback}` 也不可用"))
}

#[cfg(not(target_arch = "wasm32"))]
struct ResolvedLocalRoot {
    raw_input: String,
    expanded_path: PathBuf,
    source_label: String,
    env_file_path: Option<String>,
    warnings: Vec<String>,
}

#[cfg(not(target_arch = "wasm32"))]
fn resolve_local_package_root() -> Result<ResolvedLocalRoot, String> {
    let env_file_path = addzero_persistence::local_env_path().map(|path| path.display().to_string());
    let from_process = std::env::var(PACKAGE_LOCAL_ROOT_ENV)
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty());
    let from_file = read_local_env_value(PACKAGE_LOCAL_ROOT_ENV);

    let (raw_input, source_label, warnings) = if let Some(value) = from_process {
        (value, format!("环境变量 `{PACKAGE_LOCAL_ROOT_ENV}`"), Vec::new())
    } else if let Some(value) = from_file {
        (
            value,
            "本地配置文件 ~/.config/msc-aio/msc-aio.env".to_string(),
            Vec::new(),
        )
    } else {
        (
            DEFAULT_PACKAGE_LOCAL_ROOT_INPUT.to_string(),
            "默认值 ~/Nextcloud".to_string(),
            vec!["尚未保存安装包本地目录，当前使用默认值 ~/Nextcloud。".to_string()],
        )
    };

    let expanded_path = expand_local_root_input(&raw_input)
        .ok_or_else(|| format!("无法展开本地目录配置：`{raw_input}`"))?;

    Ok(ResolvedLocalRoot {
        raw_input,
        expanded_path,
        source_label,
        env_file_path,
        warnings,
    })
}

#[cfg(not(target_arch = "wasm32"))]
fn expand_local_root_input(raw: &str) -> Option<PathBuf> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return None;
    }

    let home = std::env::var("HOME").ok().filter(|value| !value.trim().is_empty())?;

    let expanded = if trimmed == "~" {
        home
    } else if let Some(rest) = trimmed.strip_prefix("~/") {
        format!("{home}/{rest}")
    } else if let Some(rest) = trimmed.strip_prefix("$HOME/") {
        format!("{home}/{rest}")
    } else if trimmed == "$HOME" {
        home
    } else {
        trimmed.to_string()
    };

    Some(PathBuf::from(expanded))
}

#[cfg(not(target_arch = "wasm32"))]
fn read_local_env_value(key: &str) -> Option<String> {
    let path = addzero_persistence::local_env_path()?;
    let content = std::fs::read_to_string(path).ok()?;
    content
        .lines()
        .filter_map(|line| {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                return None;
            }
            let (current_key, current_value) = trimmed.split_once('=')?;
            if current_key.trim() == key {
                Some(current_value.trim().to_string())
            } else {
                None
            }
        })
        .next()
}

#[cfg(not(target_arch = "wasm32"))]
fn save_package_local_root_input(raw_input: &str) -> PackageStorageResult<()> {
    let trimmed = raw_input.trim();
    if trimmed.is_empty() {
        return Err(PackageStorageError::new("安装包本地目录不能为空"));
    }
    let expanded = expand_local_root_input(trimmed)
        .ok_or_else(|| PackageStorageError::new("无法展开安装包本地目录，请检查 `~` 或 `$HOME`"))?;
    std::fs::create_dir_all(&expanded)
        .map_err(|err| PackageStorageError::new(format!("创建本地目录失败：{err}")))?;

    let path = addzero_persistence::local_env_path()
        .ok_or_else(|| PackageStorageError::new("无法定位本地配置文件路径"))?;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|err| PackageStorageError::new(format!("创建配置目录失败：{err}")))?;
    }

    let existing = std::fs::read_to_string(&path).unwrap_or_default();
    let mut lines = existing.lines().map(str::to_string).collect::<Vec<_>>();
    let mut replaced = false;
    for line in &mut lines {
        let trimmed_line = line.trim();
        if trimmed_line.starts_with('#') || !trimmed_line.contains('=') {
            continue;
        }
        let Some((current_key, _)) = trimmed_line.split_once('=') else {
            continue;
        };
        if current_key.trim() == PACKAGE_LOCAL_ROOT_ENV {
            *line = format!("{PACKAGE_LOCAL_ROOT_ENV}={trimmed}");
            replaced = true;
        }
    }
    if !replaced {
        lines.push(format!("{PACKAGE_LOCAL_ROOT_ENV}={trimmed}"));
    }
    let mut rendered = lines.join("\n");
    if !rendered.ends_with('\n') {
        rendered.push('\n');
    }
    std::fs::write(&path, rendered)
        .map_err(|err| PackageStorageError::new(format!("写入本地配置文件失败：{err}")))?;
    Ok(())
}

#[cfg(not(target_arch = "wasm32"))]
fn normalized_bucket(raw: &str, fallback: &str) -> Result<String, PackageStorageError> {
    let bucket = if raw.trim().is_empty() { fallback } else { raw };
    let bucket = bucket.trim().trim_matches('/').to_string();
    if bucket.is_empty() {
        Err(PackageStorageError::new("bucket 不能为空"))
    } else {
        Ok(bucket)
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn normalized_prefix(raw: &str) -> String {
    let cleaned = raw.trim().trim_matches('/');
    if cleaned.is_empty() {
        PACKAGE_RELATIVE_PREFIX.to_string()
    } else {
        cleaned.to_string()
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn validate_installer_name(file_name: &str) -> PackageStorageResult<()> {
    if file_name.trim().is_empty() {
        return Err(PackageStorageError::new("文件名不能为空"));
    }
    if !is_installer_file(Path::new(file_name)) {
        return Err(PackageStorageError::new(format!(
            "不支持的安装包格式：{file_name}"
        )));
    }
    Ok(())
}

#[cfg(not(target_arch = "wasm32"))]
fn build_local_package_path(
    root: &Path,
    relative_prefix: &str,
    content_hash: &str,
    file_name: &str,
) -> PathBuf {
    let extension = installer_extension(file_name);
    let stem = sanitize_file_stem(file_name);
    let final_name = if extension.is_empty() {
        format!("{stem}-{content_hash}")
    } else {
        format!("{stem}-{content_hash}.{extension}")
    };
    root.join(relative_prefix).join(final_name)
}

#[cfg(not(target_arch = "wasm32"))]
fn installer_extension(file_name: &str) -> String {
    let lower = file_name.to_ascii_lowercase();
    if lower.ends_with(".tar.gz") {
        "tar.gz".to_string()
    } else {
        Path::new(file_name)
            .extension()
            .and_then(OsStr::to_str)
            .unwrap_or_default()
            .to_ascii_lowercase()
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn sanitize_file_stem(file_name: &str) -> String {
    let stem = if file_name.to_ascii_lowercase().ends_with(".tar.gz") {
        file_name.trim_end_matches(".tar.gz")
    } else {
        Path::new(file_name)
            .file_stem()
            .and_then(OsStr::to_str)
            .unwrap_or("installer")
    };
    let sanitized = stem
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() {
                ch.to_ascii_lowercase()
            } else {
                '-'
            }
        })
        .collect::<String>();
    sanitized
        .split('-')
        .filter(|segment| !segment.is_empty())
        .collect::<Vec<_>>()
        .join("-")
}

#[cfg(not(target_arch = "wasm32"))]
fn path_relative_to_root(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .display()
        .to_string()
}

#[cfg(not(target_arch = "wasm32"))]
fn register_local_package_asset(
    root: &Path,
    final_path: &Path,
    content_hash: &str,
    source: &str,
    relative_prefix: &str,
    extra_raw: Option<serde_json::Value>,
) -> PackageStorageResult<()> {
    let file_name = final_path
        .file_name()
        .map(|value| value.to_string_lossy().to_string())
        .unwrap_or_else(|| "installer".to_string());
    let size_bytes = std::fs::metadata(final_path)
        .map(|meta| meta.len())
        .unwrap_or_default();
    let relative_path = path_relative_to_root(root, final_path);

    let mut raw = serde_json::json!({
        "storage_backend": "local_fs",
        "relative_prefix": relative_prefix,
        "imported_at": Utc::now().to_rfc3339(),
    });
    if let Some(extra) = extra_raw {
        raw["extra"] = extra;
    }

    run_asset_async(super::asset_graph::upsert_asset_record_on_server(
        super::asset_graph::AssetRecordInput {
            id: format!("package-blake3-{content_hash}"),
            kind: super::asset_graph::AssetKindDto::Package,
            title: file_name.clone(),
            detail: final_path.display().to_string(),
            source: source.to_string(),
            local_path: Some(final_path.display().to_string()),
            relative_path: Some(relative_path),
            download_url: None,
            content_hash: Some(content_hash.to_string()),
            hash_algorithm: Some("blake3".to_string()),
            size_bytes: Some(size_bytes),
            tags: vec![
                "安装包".to_string(),
                package_format_tag(final_path),
                "LocalFS".to_string(),
            ],
            raw,
        },
    ))
    .map_err(|err| PackageStorageError::new(err.to_string()))
}

#[cfg(not(target_arch = "wasm32"))]
fn installer_roots() -> Vec<PathBuf> {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    let mut roots = ["Downloads", "Desktop", "Documents"]
        .into_iter()
        .map(|segment| Path::new(&home).join(segment))
        .filter(|path| path.exists())
        .collect::<Vec<_>>();

    if let Ok(extra) = std::env::var("ADMIN_PACKAGE_SCAN_ROOTS") {
        roots.extend(
            extra
                .split([';', '\n'])
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(PathBuf::from)
                .filter(|path| path.exists()),
        );
    }

    roots
}

#[cfg(not(target_arch = "wasm32"))]
fn discover_installer_files(roots: &[PathBuf]) -> Vec<PathBuf> {
    let mut found = Vec::new();
    let mut stack = roots.to_vec();

    while let Some(path) = stack.pop() {
        let Ok(metadata) = std::fs::metadata(&path) else {
            continue;
        };
        if metadata.is_file() {
            if is_installer_file(&path) {
                found.push(path);
            }
            continue;
        }
        if !metadata.is_dir() || should_skip_dir(&path) {
            continue;
        }
        let Ok(entries) = std::fs::read_dir(&path) else {
            continue;
        };
        for entry in entries.flatten() {
            stack.push(entry.path());
        }
    }

    found.sort();
    found
}

#[cfg(not(target_arch = "wasm32"))]
fn should_skip_dir(path: &Path) -> bool {
    matches!(
        path.file_name().and_then(OsStr::to_str),
        Some(".git" | "node_modules" | "target" | ".Trash" | "Library" | "Caches" | "DerivedData")
    )
}

#[cfg(not(target_arch = "wasm32"))]
fn is_installer_file(path: &Path) -> bool {
    let name = path
        .file_name()
        .and_then(OsStr::to_str)
        .unwrap_or_default()
        .to_ascii_lowercase();

    [
        ".dmg",
        ".pkg",
        ".zip",
        ".tar.gz",
        ".tgz",
        ".appimage",
        ".exe",
        ".msi",
    ]
    .iter()
    .any(|suffix| name.ends_with(suffix))
}

#[cfg(not(target_arch = "wasm32"))]
fn package_format_tag(path: &Path) -> String {
    let name = path
        .file_name()
        .map(|value| value.to_string_lossy().to_ascii_lowercase())
        .unwrap_or_default();
    if name.ends_with(".tar.gz") {
        "tar.gz".to_string()
    } else {
        path.extension()
            .map(|value| value.to_string_lossy().to_ascii_lowercase())
            .unwrap_or_else(|| "package".to_string())
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn run_asset_async<T>(
    future: impl std::future::Future<Output = super::asset_graph::AssetGraphResult<T>>,
) -> super::asset_graph::AssetGraphResult<T> {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|err| super::asset_graph::AssetGraphError::Message(err.to_string()))?
        .block_on(future)
}

#[cfg(not(target_arch = "wasm32"))]
pub fn upload_packages_on_server(
    input: PackageUploadRequest,
) -> PackageStorageResult<PackageUploadReportDto> {
    NativePackageStorage::from_env().scan_and_upload_installers_blocking(input)
}

#[cfg(not(target_arch = "wasm32"))]
pub fn import_browser_packages_on_server(
    input: BrowserPackageImportRequest,
) -> PackageStorageResult<PackageUploadReportDto> {
    NativePackageStorage::from_env().import_browser_files_blocking(input)
}

#[cfg(not(target_arch = "wasm32"))]
pub fn package_storage_overview_on_server() -> PackageStorageResult<PackageStorageOverviewDto> {
    NativePackageStorage::from_env().storage_overview_blocking()
}

#[cfg(not(target_arch = "wasm32"))]
pub fn save_package_storage_config_on_server(
    input: PackageStorageConfigUpsertDto,
) -> PackageStorageResult<PackageStorageOverviewDto> {
    NativePackageStorage::from_env().save_storage_config_blocking(input)
}

mod base64_bytes {
    use base64::{Engine as _, engine::general_purpose::STANDARD};
    use serde::{Deserialize, Deserializer, Serializer, de::Error as _};

    pub fn serialize<S>(bytes: &[u8], serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&STANDARD.encode(bytes))
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Vec<u8>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let encoded = String::deserialize(deserializer)?;
        STANDARD
            .decode(encoded.as_bytes())
            .map_err(D::Error::custom)
    }
}
