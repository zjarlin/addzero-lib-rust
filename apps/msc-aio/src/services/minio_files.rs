use std::{future::Future, pin::Pin, rc::Rc};

#[cfg(not(target_arch = "wasm32"))]
use std::{collections::BTreeMap, path::Path, sync::Mutex};

#[cfg(not(target_arch = "wasm32"))]
use addzero_minio::{DEFAULT_PRESIGNED_EXPIRATION_SECONDS, MinioClient, MinioConfig};
#[cfg(not(target_arch = "wasm32"))]
use base64::Engine as _;
#[cfg(not(target_arch = "wasm32"))]
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use thiserror::Error;

pub type LocalBoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + 'a>>;

#[cfg(not(target_arch = "wasm32"))]
const DOWNLOAD_ROUTE_PREFIX: &str = "/api/admin/storage/files/download/";
pub const MSC_AIO_BUCKET_NAME: &str = "msc-aio";

#[cfg(not(target_arch = "wasm32"))]
static BROWSE_CACHE: Lazy<Mutex<BTreeMap<String, StorageBrowseResultDto>>> =
    Lazy::new(|| Mutex::new(BTreeMap::new()));

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct StorageBrowseRequestDto {
    #[serde(default)]
    pub prefix: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct StorageBreadcrumbDto {
    pub label: String,
    pub prefix: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct StorageFolderDto {
    pub name: String,
    pub prefix: String,
    pub relative_path: String,
    pub object_count: usize,
    pub size_bytes: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct StorageFileDto {
    pub name: String,
    pub object_key: String,
    pub relative_path: String,
    pub size_bytes: u64,
    pub content_type: String,
    pub last_modified: String,
    pub download_path: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct StorageBrowseResultDto {
    pub bucket: String,
    pub current_prefix: String,
    pub parent_prefix: Option<String>,
    pub breadcrumbs: Vec<StorageBreadcrumbDto>,
    pub backend_label: String,
    pub folder_count: usize,
    pub file_count: usize,
    pub folders: Vec<StorageFolderDto>,
    pub files: Vec<StorageFileDto>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct StorageUploadFileDto {
    pub file_name: String,
    pub content_type: Option<String>,
    #[serde(with = "base64_bytes")]
    pub bytes: Vec<u8>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct StorageUploadRequestDto {
    #[serde(default)]
    pub prefix: String,
    #[serde(default)]
    pub files: Vec<StorageUploadFileDto>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct StorageUploadResultDto {
    pub uploaded_count: usize,
    pub prefix: String,
    pub files: Vec<StorageFileDto>,
    pub message: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct StorageCreateFolderDto {
    #[serde(default)]
    pub parent_prefix: String,
    pub relative_path: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct StorageCreateFolderResultDto {
    pub prefix: String,
    pub message: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct StorageShareRequestDto {
    pub object_key: String,
    pub expiration_seconds: Option<u64>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct StorageShareResultDto {
    pub object_key: String,
    pub relative_path: String,
    pub presigned_url: String,
    pub encrypted_url: Option<String>,
    pub expires_in_seconds: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct StorageDeleteObjectDto {
    pub object_key: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct StorageDeleteFolderDto {
    pub prefix: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct StorageDeleteResultDto {
    pub deleted_count: usize,
    pub message: String,
}

#[derive(Clone, Debug, Error, PartialEq, Eq)]
pub enum MinioFilesError {
    #[error("{0}")]
    Message(String),
}

impl MinioFilesError {
    fn new(message: impl Into<String>) -> Self {
        Self::Message(message.into())
    }
}

pub type MinioFilesResult<T> = Result<T, MinioFilesError>;

pub trait MinioFilesApi: 'static {
    fn browse(
        &self,
        input: StorageBrowseRequestDto,
    ) -> LocalBoxFuture<'_, MinioFilesResult<StorageBrowseResultDto>>;

    fn upload_files(
        &self,
        input: StorageUploadRequestDto,
    ) -> LocalBoxFuture<'_, MinioFilesResult<StorageUploadResultDto>>;

    fn create_folder(
        &self,
        input: StorageCreateFolderDto,
    ) -> LocalBoxFuture<'_, MinioFilesResult<StorageCreateFolderResultDto>>;

    fn share_file(
        &self,
        input: StorageShareRequestDto,
    ) -> LocalBoxFuture<'_, MinioFilesResult<StorageShareResultDto>>;

    fn delete_file(
        &self,
        input: StorageDeleteObjectDto,
    ) -> LocalBoxFuture<'_, MinioFilesResult<StorageDeleteResultDto>>;

    fn delete_folder(
        &self,
        input: StorageDeleteFolderDto,
    ) -> LocalBoxFuture<'_, MinioFilesResult<StorageDeleteResultDto>>;
}

pub type SharedMinioFilesApi = Rc<dyn MinioFilesApi>;

pub fn default_minio_files_api() -> SharedMinioFilesApi {
    #[cfg(target_arch = "wasm32")]
    {
        Rc::new(BrowserMinioFilesApi)
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        Rc::new(NativeMinioFilesApi)
    }
}

#[cfg(target_arch = "wasm32")]
struct BrowserMinioFilesApi;

#[cfg(target_arch = "wasm32")]
impl MinioFilesApi for BrowserMinioFilesApi {
    fn browse(
        &self,
        input: StorageBrowseRequestDto,
    ) -> LocalBoxFuture<'_, MinioFilesResult<StorageBrowseResultDto>> {
        Box::pin(async move {
            super::browser_http::post_json("/api/admin/storage/files/browse", &input)
                .await
                .map_err(MinioFilesError::new)
        })
    }

    fn upload_files(
        &self,
        input: StorageUploadRequestDto,
    ) -> LocalBoxFuture<'_, MinioFilesResult<StorageUploadResultDto>> {
        Box::pin(async move {
            super::browser_http::post_json("/api/admin/storage/files/upload", &input)
                .await
                .map_err(MinioFilesError::new)
        })
    }

    fn create_folder(
        &self,
        input: StorageCreateFolderDto,
    ) -> LocalBoxFuture<'_, MinioFilesResult<StorageCreateFolderResultDto>> {
        Box::pin(async move {
            super::browser_http::post_json("/api/admin/storage/files/folders", &input)
                .await
                .map_err(MinioFilesError::new)
        })
    }

    fn share_file(
        &self,
        input: StorageShareRequestDto,
    ) -> LocalBoxFuture<'_, MinioFilesResult<StorageShareResultDto>> {
        Box::pin(async move {
            super::browser_http::post_json("/api/admin/storage/files/share", &input)
                .await
                .map_err(MinioFilesError::new)
        })
    }

    fn delete_file(
        &self,
        input: StorageDeleteObjectDto,
    ) -> LocalBoxFuture<'_, MinioFilesResult<StorageDeleteResultDto>> {
        Box::pin(async move {
            super::browser_http::post_json("/api/admin/storage/files/delete", &input)
                .await
                .map_err(MinioFilesError::new)
        })
    }

    fn delete_folder(
        &self,
        input: StorageDeleteFolderDto,
    ) -> LocalBoxFuture<'_, MinioFilesResult<StorageDeleteResultDto>> {
        Box::pin(async move {
            super::browser_http::post_json("/api/admin/storage/files/folders/delete", &input)
                .await
                .map_err(MinioFilesError::new)
        })
    }
}

#[cfg(not(target_arch = "wasm32"))]
struct NativeMinioFilesApi;

#[cfg(not(target_arch = "wasm32"))]
impl NativeMinioFilesApi {
    fn with_backend<T>(
        operation: impl FnOnce(MinioBackend) -> MinioFilesResult<T>,
    ) -> MinioFilesResult<T> {
        let backend = MinioBackend::from_env()
            .map_err(|reason| MinioFilesError::new(format!("MinIO 未配置：{reason}")))?;
        operation(backend)
    }

    fn browse_blocking(input: StorageBrowseRequestDto) -> MinioFilesResult<StorageBrowseResultDto> {
        Self::with_backend(|backend| backend.browse_blocking(input))
    }

    fn upload_files_blocking(
        input: StorageUploadRequestDto,
    ) -> MinioFilesResult<StorageUploadResultDto> {
        Self::with_backend(|backend| backend.upload_files_blocking(input))
    }

    fn create_folder_blocking(
        input: StorageCreateFolderDto,
    ) -> MinioFilesResult<StorageCreateFolderResultDto> {
        Self::with_backend(|backend| backend.create_folder_blocking(input))
    }

    fn share_file_blocking(
        input: StorageShareRequestDto,
    ) -> MinioFilesResult<StorageShareResultDto> {
        Self::with_backend(|backend| backend.share_file_blocking(input))
    }

    fn delete_file_blocking(
        input: StorageDeleteObjectDto,
    ) -> MinioFilesResult<StorageDeleteResultDto> {
        Self::with_backend(|backend| backend.delete_file_blocking(input))
    }

    fn delete_folder_blocking(
        input: StorageDeleteFolderDto,
    ) -> MinioFilesResult<StorageDeleteResultDto> {
        Self::with_backend(|backend| backend.delete_folder_blocking(input))
    }

    fn download_url_blocking(download_token: String) -> MinioFilesResult<String> {
        Self::with_backend(|backend| backend.download_url_blocking(&download_token))
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl MinioFilesApi for NativeMinioFilesApi {
    fn browse(
        &self,
        input: StorageBrowseRequestDto,
    ) -> LocalBoxFuture<'_, MinioFilesResult<StorageBrowseResultDto>> {
        Box::pin(async move {
            tokio::task::spawn_blocking(move || Self::browse_blocking(input))
                .await
                .map_err(|err| MinioFilesError::new(format!("文件浏览任务失败：{err}")))?
        })
    }

    fn upload_files(
        &self,
        input: StorageUploadRequestDto,
    ) -> LocalBoxFuture<'_, MinioFilesResult<StorageUploadResultDto>> {
        Box::pin(async move {
            tokio::task::spawn_blocking(move || Self::upload_files_blocking(input))
                .await
                .map_err(|err| MinioFilesError::new(format!("文件上传任务失败：{err}")))?
        })
    }

    fn create_folder(
        &self,
        input: StorageCreateFolderDto,
    ) -> LocalBoxFuture<'_, MinioFilesResult<StorageCreateFolderResultDto>> {
        Box::pin(async move {
            tokio::task::spawn_blocking(move || Self::create_folder_blocking(input))
                .await
                .map_err(|err| MinioFilesError::new(format!("创建目录任务失败：{err}")))?
        })
    }

    fn share_file(
        &self,
        input: StorageShareRequestDto,
    ) -> LocalBoxFuture<'_, MinioFilesResult<StorageShareResultDto>> {
        Box::pin(async move {
            tokio::task::spawn_blocking(move || Self::share_file_blocking(input))
                .await
                .map_err(|err| MinioFilesError::new(format!("分享链接生成任务失败：{err}")))?
        })
    }

    fn delete_file(
        &self,
        input: StorageDeleteObjectDto,
    ) -> LocalBoxFuture<'_, MinioFilesResult<StorageDeleteResultDto>> {
        Box::pin(async move {
            tokio::task::spawn_blocking(move || Self::delete_file_blocking(input))
                .await
                .map_err(|err| MinioFilesError::new(format!("删除文件任务失败：{err}")))?
        })
    }

    fn delete_folder(
        &self,
        input: StorageDeleteFolderDto,
    ) -> LocalBoxFuture<'_, MinioFilesResult<StorageDeleteResultDto>> {
        Box::pin(async move {
            tokio::task::spawn_blocking(move || Self::delete_folder_blocking(input))
                .await
                .map_err(|err| MinioFilesError::new(format!("删除目录任务失败：{err}")))?
        })
    }
}

#[cfg(not(target_arch = "wasm32"))]
#[derive(Clone)]
struct MinioBackend {
    client: MinioClient,
    bucket: String,
    backend_label: String,
    share_expiration_seconds: u64,
    share_secret: Option<String>,
}

#[cfg(not(target_arch = "wasm32"))]
impl MinioBackend {
    fn from_env() -> Result<Self, String> {
        let environment = minio_environment_from_env()?;
        let share_secret =
            read_env_any_optional(&["MSC_AIO_MINIO_SHARE_SECRET", "MINIO_URL_ENCRYPTION_SECRET"]);
        let share_expiration_seconds = read_u64_env_any(
            &["MSC_AIO_MINIO_SHARE_EXPIRES_SECONDS"],
            DEFAULT_PRESIGNED_EXPIRATION_SECONDS,
        )?;

        Ok(Self {
            client: environment.client,
            bucket: environment.bucket,
            backend_label: environment.backend_label,
            share_expiration_seconds,
            share_secret,
        })
    }

    fn ensure_bucket(&self) -> MinioFilesResult<()> {
        self.client
            .ensure_bucket(&self.bucket)
            .map_err(|err| MinioFilesError::new(format!("初始化 bucket 失败：{err}")))?;
        Ok(())
    }

    fn browse_blocking(
        &self,
        input: StorageBrowseRequestDto,
    ) -> MinioFilesResult<StorageBrowseResultDto> {
        let prefix = normalize_prefix(&input.prefix)?;
        if let Some(cached) = cached_browse_result(prefix.as_str()) {
            return Ok(cached);
        }
        self.ensure_bucket()?;

        let objects = self
            .client
            .list_objects(
                &self.bucket,
                if prefix.is_empty() {
                    None
                } else {
                    Some(prefix.as_str())
                },
                false,
            )
            .map_err(|err| MinioFilesError::new(format!("读取对象列表失败：{err}")))?;

        let mut folders = BTreeMap::<String, FolderAccumulator>::new();
        let mut files = Vec::new();

        for object in objects {
            let object_key = object.object_name.clone();
            let relative = if prefix.is_empty() {
                object_key.as_str()
            } else {
                let Some(relative) = object_key.strip_prefix(&prefix) else {
                    continue;
                };
                relative
            };

            if relative.is_empty() {
                continue;
            }

            if let Some((segment, _)) = relative.split_once('/') {
                if segment.is_empty() {
                    continue;
                }

                let folder_prefix = format!("{prefix}{segment}/");
                let folder = folders.entry(folder_prefix.clone()).or_insert_with(|| {
                    FolderAccumulator::new(segment.to_string(), folder_prefix.clone())
                });
                if object_key != folder_prefix && !object_key.ends_with('/') {
                    folder.object_count += 1;
                    folder.size_bytes += object.size;
                }
                continue;
            }

            if object_key.ends_with('/') {
                continue;
            }

            files.push(storage_file_dto(
                &object_key,
                object.size,
                object
                    .content_type
                    .unwrap_or_else(|| addzero_rustfs::guess_content_type(Path::new(&object_key))),
                object
                    .last_modified
                    .unwrap_or_else(|| "unknown".to_string()),
            ));
        }

        let mut folders = folders
            .into_values()
            .map(FolderAccumulator::into_dto)
            .collect::<Vec<_>>();
        folders.sort_by(folder_sort_key);
        files.sort_by(file_sort_key);

        let result = StorageBrowseResultDto {
            bucket: self.bucket.clone(),
            current_prefix: prefix.clone(),
            parent_prefix: parent_prefix(&prefix),
            breadcrumbs: build_breadcrumbs(&prefix),
            backend_label: self.backend_label.clone(),
            folder_count: folders.len(),
            file_count: files.len(),
            folders,
            files,
        };
        store_browse_result(prefix.as_str(), &result);
        Ok(result)
    }

    fn upload_files_blocking(
        &self,
        input: StorageUploadRequestDto,
    ) -> MinioFilesResult<StorageUploadResultDto> {
        if input.files.is_empty() {
            return Err(MinioFilesError::new("请至少选择一个文件"));
        }

        let prefix = normalize_prefix(&input.prefix)?;
        self.ensure_bucket()?;

        let uploaded_total = input.files.len();
        let mut uploaded = Vec::with_capacity(input.files.len());

        for file in input.files {
            let file_name = validate_file_name(&file.file_name)?;
            let object_key = format!("{prefix}{file_name}");
            let content_type = normalized_content_type(file.content_type.as_deref(), &file_name);
            self.client
                .put_object_bytes(
                    &self.bucket,
                    &object_key,
                    &file.bytes,
                    Some(content_type.as_str()),
                )
                .map_err(|err| MinioFilesError::new(format!("上传 `{file_name}` 失败：{err}")))?;

            let metadata = self
                .client
                .stat_object(&self.bucket, &object_key)
                .map_err(|err| {
                    MinioFilesError::new(format!("读取 `{file_name}` 元数据失败：{err}"))
                })?;

            let last_modified = metadata
                .as_ref()
                .and_then(|item| item.last_modified.clone())
                .unwrap_or_else(|| "just now".to_string());
            let size_bytes = metadata
                .as_ref()
                .map(|item| item.size)
                .unwrap_or_else(|| u64::try_from(file.bytes.len()).unwrap_or_default());
            let content_type = metadata
                .and_then(|item| item.content_type)
                .unwrap_or(content_type);

            uploaded.push(storage_file_dto(
                &object_key,
                size_bytes,
                content_type,
                last_modified,
            ));
        }

        let target_label = display_prefix(&prefix);
        clear_browse_cache();
        Ok(StorageUploadResultDto {
            uploaded_count: uploaded.len(),
            prefix,
            files: uploaded,
            message: format!("已上传 {uploaded_total} 个文件到 {target_label}"),
        })
    }

    fn create_folder_blocking(
        &self,
        input: StorageCreateFolderDto,
    ) -> MinioFilesResult<StorageCreateFolderResultDto> {
        let parent_prefix = normalize_prefix(&input.parent_prefix)?;
        let folder_prefix = join_prefix(&parent_prefix, &input.relative_path)?;
        self.ensure_bucket()?;

        let existing = self
            .client
            .list_objects(&self.bucket, Some(folder_prefix.as_str()), true)
            .map_err(|err| MinioFilesError::new(format!("检查目录是否存在失败：{err}")))?;
        if !existing.is_empty() {
            return Err(MinioFilesError::new(format!(
                "目录 `{}` 已存在",
                display_prefix(&folder_prefix)
            )));
        }

        self.client
            .put_object_bytes(
                &self.bucket,
                &folder_prefix,
                &[],
                Some("application/x-directory"),
            )
            .map_err(|err| MinioFilesError::new(format!("创建目录失败：{err}")))?;

        clear_browse_cache();
        Ok(StorageCreateFolderResultDto {
            prefix: folder_prefix.clone(),
            message: format!("已创建目录 {}", display_prefix(&folder_prefix)),
        })
    }

    fn share_file_blocking(
        &self,
        input: StorageShareRequestDto,
    ) -> MinioFilesResult<StorageShareResultDto> {
        let object_key = normalize_file_object_key(&input.object_key)?;
        self.ensure_bucket()?;

        if !self
            .client
            .object_exists(&self.bucket, &object_key)
            .map_err(|err| MinioFilesError::new(format!("检查对象是否存在失败：{err}")))?
        {
            return Err(MinioFilesError::new("文件不存在，无法生成分享链接"));
        }

        let expires_in_seconds = input
            .expiration_seconds
            .filter(|value| *value > 0)
            .unwrap_or(self.share_expiration_seconds);
        let presigned = self
            .client
            .get_presigned_object_url_with_expiration(&self.bucket, &object_key, expires_in_seconds)
            .map_err(|err| MinioFilesError::new(format!("生成分享链接失败：{err}")))?;

        let encrypted_url = self.share_secret.as_ref().map(|secret| {
            addzero_minio::encrypt_url(secret, &presigned.url)
                .map_err(|err| MinioFilesError::new(format!("生成加密分享链接失败：{err}")))
        });
        let encrypted_url = match encrypted_url {
            Some(result) => Some(result?),
            None => None,
        };

        Ok(StorageShareResultDto {
            object_key: object_key.clone(),
            relative_path: object_key,
            presigned_url: presigned.url,
            encrypted_url,
            expires_in_seconds,
        })
    }

    fn delete_file_blocking(
        &self,
        input: StorageDeleteObjectDto,
    ) -> MinioFilesResult<StorageDeleteResultDto> {
        let object_key = normalize_file_object_key(&input.object_key)?;
        self.ensure_bucket()?;

        if !self
            .client
            .object_exists(&self.bucket, &object_key)
            .map_err(|err| MinioFilesError::new(format!("检查对象是否存在失败：{err}")))?
        {
            return Err(MinioFilesError::new("文件不存在，无法删除"));
        }

        self.client
            .delete_object(&self.bucket, &object_key)
            .map_err(|err| MinioFilesError::new(format!("删除文件失败：{err}")))?;

        clear_browse_cache();
        Ok(StorageDeleteResultDto {
            deleted_count: 1,
            message: format!("已删除文件 `{object_key}`"),
        })
    }

    fn delete_folder_blocking(
        &self,
        input: StorageDeleteFolderDto,
    ) -> MinioFilesResult<StorageDeleteResultDto> {
        let prefix = normalize_non_root_folder_prefix(&input.prefix)?;
        self.ensure_bucket()?;

        let objects = self
            .client
            .list_objects(&self.bucket, Some(prefix.as_str()), true)
            .map_err(|err| MinioFilesError::new(format!("读取目录对象失败：{err}")))?;

        if objects.is_empty() {
            return Err(MinioFilesError::new("目录不存在或已经为空"));
        }

        let keys = objects
            .into_iter()
            .map(|item| item.object_name)
            .collect::<Vec<_>>();
        self.client
            .delete_objects(&self.bucket, &keys)
            .map_err(|err| MinioFilesError::new(format!("删除目录失败：{err}")))?;

        clear_browse_cache();
        Ok(StorageDeleteResultDto {
            deleted_count: keys.len(),
            message: format!(
                "已删除目录 {} 下的 {} 个对象",
                display_prefix(&prefix),
                keys.len()
            ),
        })
    }

    fn download_url_blocking(&self, download_token: &str) -> MinioFilesResult<String> {
        let object_key = decode_download_token(download_token)?;
        if !self
            .client
            .object_exists(&self.bucket, &object_key)
            .map_err(|err| MinioFilesError::new(format!("检查对象是否存在失败：{err}")))?
        {
            return Err(MinioFilesError::new("文件不存在，无法下载"));
        }

        self.client
            .get_presigned_object_url_with_expiration(
                &self.bucket,
                &object_key,
                self.share_expiration_seconds,
            )
            .map(|details| details.url)
            .map_err(|err| MinioFilesError::new(format!("生成下载链接失败：{err}")))
    }
}

#[cfg(not(target_arch = "wasm32"))]
#[derive(Clone)]
pub(crate) struct MinioEnvironment {
    pub client: MinioClient,
    pub bucket: String,
    pub backend_label: String,
}

#[cfg(not(target_arch = "wasm32"))]
pub(crate) fn minio_environment_from_env() -> Result<MinioEnvironment, String> {
    let endpoint = read_env_any(&[
        "MSC_AIO_MINIO_ENDPOINT",
        "ADMIN_PACKAGE_S3_ENDPOINT",
        "ADMIN_LOGO_S3_ENDPOINT",
    ])?;
    let access_key = read_env_any(&[
        "MSC_AIO_MINIO_ACCESS_KEY",
        "ADMIN_PACKAGE_S3_ACCESS_KEY",
        "ADMIN_LOGO_S3_ACCESS_KEY",
    ])?;
    let secret_key = read_env_any(&[
        "MSC_AIO_MINIO_SECRET_KEY",
        "ADMIN_PACKAGE_S3_SECRET_KEY",
        "ADMIN_LOGO_S3_SECRET_KEY",
    ])?;
    let bucket = canonical_bucket_name()?;
    let region = read_env_any_optional(&[
        "MSC_AIO_MINIO_REGION",
        "ADMIN_PACKAGE_S3_REGION",
        "ADMIN_LOGO_S3_REGION",
    ])
    .unwrap_or_else(|| "us-east-1".to_string());

    let config = MinioConfig::builder(endpoint.clone(), access_key, secret_key)
        .region(region)
        .build()
        .map_err(|err| err.to_string())?;
    let client = addzero_minio::create_client(config).map_err(|err| err.to_string())?;

    Ok(MinioEnvironment {
        client,
        bucket: bucket.clone(),
        backend_label: format!("MinIO · bucket `{bucket}` · endpoint {endpoint}"),
    })
}

#[cfg(not(target_arch = "wasm32"))]
#[derive(Clone)]
struct FolderAccumulator {
    name: String,
    prefix: String,
    object_count: usize,
    size_bytes: u64,
}

#[cfg(not(target_arch = "wasm32"))]
impl FolderAccumulator {
    fn new(name: String, prefix: String) -> Self {
        Self {
            name,
            prefix,
            object_count: 0,
            size_bytes: 0,
        }
    }

    fn into_dto(self) -> StorageFolderDto {
        StorageFolderDto {
            name: self.name,
            relative_path: self.prefix.clone(),
            prefix: self.prefix,
            object_count: self.object_count,
            size_bytes: self.size_bytes,
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn cached_browse_result(prefix: &str) -> Option<StorageBrowseResultDto> {
    BROWSE_CACHE
        .lock()
        .ok()
        .and_then(|cache| cache.get(prefix).cloned())
}

#[cfg(not(target_arch = "wasm32"))]
fn store_browse_result(prefix: &str, result: &StorageBrowseResultDto) {
    if let Ok(mut cache) = BROWSE_CACHE.lock() {
        cache.insert(prefix.to_string(), result.clone());
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn clear_browse_cache() {
    if let Ok(mut cache) = BROWSE_CACHE.lock() {
        cache.clear();
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn folder_sort_key(left: &StorageFolderDto, right: &StorageFolderDto) -> std::cmp::Ordering {
    left.name
        .to_lowercase()
        .cmp(&right.name.to_lowercase())
        .then(left.prefix.cmp(&right.prefix))
}

#[cfg(not(target_arch = "wasm32"))]
fn file_sort_key(left: &StorageFileDto, right: &StorageFileDto) -> std::cmp::Ordering {
    left.name
        .to_lowercase()
        .cmp(&right.name.to_lowercase())
        .then(left.object_key.cmp(&right.object_key))
}

#[cfg(not(target_arch = "wasm32"))]
fn storage_file_dto(
    object_key: &str,
    size_bytes: u64,
    content_type: String,
    last_modified: String,
) -> StorageFileDto {
    StorageFileDto {
        name: object_key
            .rsplit('/')
            .next()
            .unwrap_or(object_key)
            .to_string(),
        object_key: object_key.to_string(),
        relative_path: object_key.to_string(),
        size_bytes,
        content_type,
        last_modified,
        download_path: build_download_path(object_key),
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn normalized_content_type(content_type: Option<&str>, file_name: &str) -> String {
    match content_type {
        Some(value) if !value.trim().is_empty() => value.to_string(),
        _ => addzero_rustfs::guess_content_type(Path::new(file_name)),
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn read_env_any(names: &[&str]) -> Result<String, String> {
    for name in names {
        if let Ok(value) = std::env::var(name) {
            let trimmed = value.trim();
            if !trimmed.is_empty() {
                return Ok(trimmed.to_string());
            }
        }
    }
    Err(format!("缺少环境变量：{}", names.join(" / ")))
}

#[cfg(not(target_arch = "wasm32"))]
fn read_env_any_optional(names: &[&str]) -> Option<String> {
    names.iter().find_map(|name| {
        std::env::var(name)
            .ok()
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty())
    })
}

#[cfg(not(target_arch = "wasm32"))]
fn canonical_bucket_name() -> Result<String, String> {
    enforce_bucket_env("MSC_AIO_MINIO_BUCKET")?;
    enforce_bucket_env("ADMIN_PACKAGE_S3_BUCKET")?;
    enforce_bucket_env("ADMIN_LOGO_S3_BUCKET")?;
    Ok(MSC_AIO_BUCKET_NAME.to_string())
}

#[cfg(not(target_arch = "wasm32"))]
fn enforce_bucket_env(name: &str) -> Result<(), String> {
    let Some(value) = read_env_any_optional(&[name]) else {
        return Ok(());
    };
    if value == MSC_AIO_BUCKET_NAME {
        Ok(())
    } else {
        Err(format!(
            "MinIO bucket 已固定为 `{MSC_AIO_BUCKET_NAME}`；请移除或改正 `{name}={value}`"
        ))
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn read_u64_env_any(names: &[&str], default: u64) -> Result<u64, String> {
    let Some(raw) = read_env_any_optional(names) else {
        return Ok(default);
    };
    raw.parse::<u64>()
        .map_err(|err| format!("无法解析 `{}`：{err}", names.join(" / ")))
}

#[cfg(not(target_arch = "wasm32"))]
fn validate_file_name(raw: &str) -> MinioFilesResult<String> {
    let name = raw.trim();
    if name.is_empty() {
        return Err(MinioFilesError::new("文件名不能为空"));
    }
    if name.contains('/') || name.contains('\\') {
        return Err(MinioFilesError::new("文件名不能包含路径分隔符"));
    }
    validate_segment(name)?;
    Ok(name.to_string())
}

#[cfg(not(target_arch = "wasm32"))]
fn validate_segment(segment: &str) -> MinioFilesResult<()> {
    if segment.trim().is_empty() {
        return Err(MinioFilesError::new("路径段不能为空"));
    }
    if matches!(segment, "." | "..") {
        return Err(MinioFilesError::new("路径段不能是 . 或 .."));
    }
    if segment.contains('\\') {
        return Err(MinioFilesError::new("路径不能包含反斜杠"));
    }
    if segment.chars().any(|ch| ch.is_control()) {
        return Err(MinioFilesError::new("路径不能包含控制字符"));
    }
    Ok(())
}

#[cfg(not(target_arch = "wasm32"))]
fn normalize_prefix(raw: &str) -> MinioFilesResult<String> {
    let trimmed = raw.trim();
    if trimmed.is_empty() || trimmed == "/" {
        return Ok(String::new());
    }

    let mut segments = Vec::new();
    for segment in trimmed.split('/') {
        if segment.is_empty() {
            continue;
        }
        validate_segment(segment)?;
        segments.push(segment.trim().to_string());
    }

    if segments.is_empty() {
        Ok(String::new())
    } else {
        Ok(format!("{}/", segments.join("/")))
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn normalize_non_root_folder_prefix(raw: &str) -> MinioFilesResult<String> {
    let prefix = normalize_prefix(raw)?;
    if prefix.is_empty() {
        Err(MinioFilesError::new("根目录不能直接删除"))
    } else {
        Ok(prefix)
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn normalize_file_object_key(raw: &str) -> MinioFilesResult<String> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Err(MinioFilesError::new("对象 key 不能为空"));
    }
    if trimmed.ends_with('/') {
        return Err(MinioFilesError::new("当前操作仅支持文件，不支持目录"));
    }

    let normalized = normalize_joined_path(trimmed, false)?;
    Ok(normalized)
}

#[cfg(not(target_arch = "wasm32"))]
fn normalize_joined_path(raw: &str, force_trailing_slash: bool) -> MinioFilesResult<String> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Ok(String::new());
    }

    let mut segments = Vec::new();
    for segment in trimmed.split('/') {
        if segment.is_empty() {
            continue;
        }
        validate_segment(segment)?;
        segments.push(segment.trim().to_string());
    }

    if segments.is_empty() {
        return Ok(String::new());
    }

    let joined = segments.join("/");
    if force_trailing_slash {
        Ok(format!("{joined}/"))
    } else {
        Ok(joined)
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn join_prefix(parent_prefix: &str, relative_path: &str) -> MinioFilesResult<String> {
    let relative = relative_path.trim();
    if relative.is_empty() {
        return Err(MinioFilesError::new("请输入目录相对路径"));
    }

    let parent = normalize_prefix(parent_prefix)?;
    let combined = format!("{parent}{relative}");
    let normalized = normalize_joined_path(&combined, true)?;
    if normalized.is_empty() {
        Err(MinioFilesError::new("目录相对路径不能为空"))
    } else {
        Ok(normalized)
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn parent_prefix(prefix: &str) -> Option<String> {
    let normalized = normalize_prefix(prefix).ok()?;
    if normalized.is_empty() {
        return None;
    }

    let trimmed = normalized.trim_end_matches('/');
    let parent = trimmed.rsplit_once('/').map(|(left, _)| left).unwrap_or("");
    if parent.is_empty() {
        Some(String::new())
    } else {
        Some(format!("{parent}/"))
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn build_breadcrumbs(prefix: &str) -> Vec<StorageBreadcrumbDto> {
    let normalized = normalize_prefix(prefix).unwrap_or_default();
    let mut breadcrumbs = vec![StorageBreadcrumbDto {
        label: "根目录".to_string(),
        prefix: String::new(),
    }];

    if normalized.is_empty() {
        return breadcrumbs;
    }

    let mut current = String::new();
    for segment in normalized.trim_end_matches('/').split('/') {
        if segment.is_empty() {
            continue;
        }
        current.push_str(segment);
        current.push('/');
        breadcrumbs.push(StorageBreadcrumbDto {
            label: segment.to_string(),
            prefix: current.clone(),
        });
    }

    breadcrumbs
}

#[cfg(not(target_arch = "wasm32"))]
fn display_prefix(prefix: &str) -> String {
    let normalized = normalize_prefix(prefix).unwrap_or_default();
    if normalized.is_empty() {
        "/".to_string()
    } else {
        format!("/{}", normalized)
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn build_download_token(object_key: &str) -> String {
    base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(object_key.as_bytes())
}

#[cfg(not(target_arch = "wasm32"))]
fn build_download_path(object_key: &str) -> String {
    format!(
        "{DOWNLOAD_ROUTE_PREFIX}{}",
        build_download_token(object_key)
    )
}

#[cfg(not(target_arch = "wasm32"))]
fn decode_download_token(download_token: &str) -> MinioFilesResult<String> {
    let bytes = base64::engine::general_purpose::URL_SAFE_NO_PAD
        .decode(download_token.as_bytes())
        .map_err(|err| MinioFilesError::new(format!("下载 token 非法：{err}")))?;
    String::from_utf8(bytes).map_err(|err| MinioFilesError::new(format!("下载 token 非法：{err}")))
}

#[cfg(not(target_arch = "wasm32"))]
pub fn browse_files_on_server(
    input: StorageBrowseRequestDto,
) -> MinioFilesResult<StorageBrowseResultDto> {
    NativeMinioFilesApi::browse_blocking(input)
}

#[cfg(not(target_arch = "wasm32"))]
pub fn upload_files_on_server(
    input: StorageUploadRequestDto,
) -> MinioFilesResult<StorageUploadResultDto> {
    NativeMinioFilesApi::upload_files_blocking(input)
}

#[cfg(not(target_arch = "wasm32"))]
pub fn create_folder_on_server(
    input: StorageCreateFolderDto,
) -> MinioFilesResult<StorageCreateFolderResultDto> {
    NativeMinioFilesApi::create_folder_blocking(input)
}

#[cfg(not(target_arch = "wasm32"))]
pub fn share_file_on_server(
    input: StorageShareRequestDto,
) -> MinioFilesResult<StorageShareResultDto> {
    NativeMinioFilesApi::share_file_blocking(input)
}

#[cfg(not(target_arch = "wasm32"))]
pub fn delete_file_on_server(
    input: StorageDeleteObjectDto,
) -> MinioFilesResult<StorageDeleteResultDto> {
    NativeMinioFilesApi::delete_file_blocking(input)
}

#[cfg(not(target_arch = "wasm32"))]
pub fn delete_folder_on_server(
    input: StorageDeleteFolderDto,
) -> MinioFilesResult<StorageDeleteResultDto> {
    NativeMinioFilesApi::delete_folder_blocking(input)
}

#[cfg(not(target_arch = "wasm32"))]
pub fn presign_download_url_on_server(download_token: &str) -> MinioFilesResult<String> {
    NativeMinioFilesApi::download_url_blocking(download_token.to_string())
}

mod base64_bytes {
    use base64::Engine as _;
    use serde::{Deserialize, Deserializer, Serializer};

    pub fn serialize<S>(bytes: &[u8], serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&base64::engine::general_purpose::STANDARD.encode(bytes))
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Vec<u8>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let encoded = String::deserialize(deserializer)?;
        base64::engine::general_purpose::STANDARD
            .decode(encoded.as_bytes())
            .map_err(serde::de::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use super::{
        build_breadcrumbs, build_download_path, decode_download_token, normalize_file_object_key,
        normalize_prefix, parent_prefix,
    };

    #[test]
    fn normalize_prefix_should_keep_virtual_directory_shape() {
        assert_eq!(normalize_prefix("").expect("empty"), "");
        assert_eq!(
            normalize_prefix("/assets//images").expect("prefix"),
            "assets/images/"
        );
        assert_eq!(
            normalize_prefix(" reports/2026/05/ ").expect("prefix"),
            "reports/2026/05/"
        );
    }

    #[test]
    fn normalize_file_object_key_should_reject_directory_inputs() {
        assert_eq!(
            normalize_file_object_key("assets/logo.png").expect("file"),
            "assets/logo.png"
        );
        assert!(normalize_file_object_key("assets/logo/").is_err());
    }

    #[test]
    fn breadcrumbs_and_parent_prefix_should_match_current_path() {
        let breadcrumbs = build_breadcrumbs("assets/images/2026/");
        let labels = breadcrumbs
            .iter()
            .map(|item| item.label.as_str())
            .collect::<Vec<_>>();
        assert_eq!(labels, vec!["根目录", "assets", "images", "2026"]);
        assert_eq!(
            parent_prefix("assets/images/2026/").expect("parent"),
            "assets/images/"
        );
    }

    #[test]
    fn download_token_round_trip_should_be_lossless() {
        let path = build_download_path("assets/report final.pdf");
        let token = path
            .rsplit('/')
            .next()
            .expect("download token should exist");
        assert_eq!(
            decode_download_token(token).expect("decode token"),
            "assets/report final.pdf"
        );
    }
}
