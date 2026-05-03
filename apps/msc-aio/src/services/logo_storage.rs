use std::rc::Rc;

#[cfg(not(target_arch = "wasm32"))]
use addzero_minio::MinioClient;
#[cfg(not(target_arch = "wasm32"))]
use chrono::Utc;
use serde::{Deserialize, Serialize};
use thiserror::Error;

pub use super::LocalBoxFuture;

pub const LOGO_PREVIEW_BASE_URL: &str = "https://minio-api.addzero.site";

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct LogoUploadRequest {
    pub file_name: String,
    pub content_type: Option<String>,
    #[serde(with = "base64_bytes")]
    pub bytes: Vec<u8>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct StoredLogoDto {
    pub object_key: String,
    pub relative_path: String,
    pub file_name: String,
    pub content_type: String,
    pub backend_label: String,
}

#[derive(Clone, Debug, Error, PartialEq, Eq)]
pub enum LogoStorageError {
    #[error("{0}")]
    Message(String),
}

impl LogoStorageError {
    fn new(message: impl Into<String>) -> Self {
        Self::Message(message.into())
    }
}

pub type LogoStorageResult<T> = Result<T, LogoStorageError>;

pub trait LogoStorageApi: 'static {
    fn upload_logo(
        &self,
        input: LogoUploadRequest,
    ) -> LocalBoxFuture<'_, LogoStorageResult<StoredLogoDto>>;
}

pub type SharedLogoStorageApi = Rc<dyn LogoStorageApi>;

pub fn default_logo_storage_api() -> SharedLogoStorageApi {
    #[cfg(target_arch = "wasm32")]
    {
        Rc::new(BrowserLogoStorageApi)
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        Rc::new(NativeLogoStorage)
    }
}

#[cfg(target_arch = "wasm32")]
struct BrowserLogoStorageApi;

#[cfg(target_arch = "wasm32")]
impl LogoStorageApi for BrowserLogoStorageApi {
    fn upload_logo(
        &self,
        input: LogoUploadRequest,
    ) -> LocalBoxFuture<'_, LogoStorageResult<StoredLogoDto>> {
        Box::pin(async move {
            super::browser_http::post_json("/api/admin/storage/logo", &input)
                .await
                .map_err(LogoStorageError::new)
        })
    }
}

#[cfg(not(target_arch = "wasm32"))]
struct NativeLogoStorage;

#[cfg(not(target_arch = "wasm32"))]
impl NativeLogoStorage {
    fn upload_logo_blocking(input: LogoUploadRequest) -> LogoStorageResult<StoredLogoDto> {
        let backend = MinioBackend::from_env()
            .map_err(|reason| LogoStorageError::new(format!("MinIO / S3 存储未配置：{reason}")))?;
        backend.upload_logo_blocking(input)
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl LogoStorageApi for NativeLogoStorage {
    fn upload_logo(
        &self,
        input: LogoUploadRequest,
    ) -> LocalBoxFuture<'_, LogoStorageResult<StoredLogoDto>> {
        Box::pin(async move {
            tokio::task::spawn_blocking(move || Self::upload_logo_blocking(input))
                .await
                .map_err(|err| LogoStorageError::new(format!("logo 上传任务失败：{err}")))?
        })
    }
}

#[cfg(not(target_arch = "wasm32"))]
struct MinioBackend {
    client: MinioClient,
    bucket: String,
    backend_label: String,
}

#[cfg(not(target_arch = "wasm32"))]
impl MinioBackend {
    fn from_env() -> Result<Self, String> {
        let environment = super::minio_files::minio_environment_from_env()?;
        Ok(Self {
            client: environment.client,
            bucket: environment.bucket,
            backend_label: environment.backend_label,
        })
    }

    fn upload_logo_blocking(&self, input: LogoUploadRequest) -> LogoStorageResult<StoredLogoDto> {
        validate_logo(&input)?;
        self.client
            .ensure_bucket(&self.bucket)
            .map_err(|err| LogoStorageError::new(format!("创建 bucket 失败：{err}")))?;

        let content_type = normalized_content_type(input.content_type.as_deref());
        let object_key = build_object_key(&input.file_name);

        self.client
            .put_object_bytes(
                &self.bucket,
                &object_key,
                &input.bytes,
                Some(content_type.as_str()),
            )
            .map_err(|err| LogoStorageError::new(format!("上传 logo 到 MinIO 失败：{err}")))?;

        Ok(StoredLogoDto {
            object_key: object_key.clone(),
            relative_path: build_relative_path(&self.bucket, &object_key),
            file_name: input.file_name,
            content_type,
            backend_label: self.backend_label.clone(),
        })
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn validate_logo(input: &LogoUploadRequest) -> LogoStorageResult<()> {
    if input.bytes.is_empty() {
        return Err(LogoStorageError::new("请选择一个非空图片文件"));
    }

    if input.bytes.len() > 4 * 1024 * 1024 {
        return Err(LogoStorageError::new("Logo 文件请控制在 4MB 以内"));
    }

    if let Some(content_type) = input.content_type.as_deref() {
        if !content_type.starts_with("image/") {
            return Err(LogoStorageError::new("Logo 只接受图片文件"));
        }
    }

    Ok(())
}

#[cfg(not(target_arch = "wasm32"))]
fn normalized_content_type(content_type: Option<&str>) -> String {
    match content_type {
        Some(value) if value.starts_with("image/") => value.to_string(),
        _ => "image/png".to_string(),
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn build_object_key(file_name: &str) -> String {
    let extension = file_name
        .rsplit_once('.')
        .map(|(_, ext)| ext)
        .filter(|ext| !ext.trim().is_empty())
        .map(sanitize_segment)
        .filter(|ext| !ext.is_empty())
        .unwrap_or_else(|| "png".to_string());

    format!(
        "branding/logos/logo-{}.{}",
        Utc::now().timestamp_millis(),
        extension
    )
}

pub fn build_preview_url(relative_path: &str) -> String {
    format!(
        "{}/{}",
        LOGO_PREVIEW_BASE_URL.trim_end_matches('/'),
        relative_path.trim_start_matches('/')
    )
}

#[cfg(not(target_arch = "wasm32"))]
fn build_relative_path(bucket: &str, object_key: &str) -> String {
    format!(
        "{}/{}",
        bucket.trim_matches('/'),
        object_key.trim_start_matches('/')
    )
}

#[cfg(not(target_arch = "wasm32"))]
fn sanitize_segment(raw: &str) -> String {
    raw.chars()
        .filter(|ch| ch.is_ascii_alphanumeric())
        .collect::<String>()
        .to_lowercase()
}

#[cfg(not(target_arch = "wasm32"))]
pub fn upload_logo_on_server(input: LogoUploadRequest) -> LogoStorageResult<StoredLogoDto> {
    NativeLogoStorage::upload_logo_blocking(input)
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

#[cfg(test)]
mod tests {
    use super::{LogoUploadRequest, build_preview_url, build_relative_path};

    #[test]
    fn relative_path_should_include_bucket_and_object_key() {
        assert_eq!(
            build_relative_path("branding", "logos/logo-1.png"),
            "branding/logos/logo-1.png"
        );
    }

    #[test]
    fn preview_url_should_use_public_minio_domain() {
        assert_eq!(
            build_preview_url("branding/logos/logo-1.png"),
            "https://minio-api.addzero.site/branding/logos/logo-1.png"
        );
    }

    #[test]
    fn upload_request_should_round_trip_bytes_as_base64_json() {
        let payload = LogoUploadRequest {
            file_name: "logo.png".to_string(),
            content_type: Some("image/png".to_string()),
            bytes: vec![1, 2, 3, 4],
        };

        let encoded = serde_json::to_string(&payload).expect("request should serialize");
        assert!(encoded.contains("\"AQIDBA==\""));

        let decoded: LogoUploadRequest =
            serde_json::from_str(&encoded).expect("request should deserialize");
        assert_eq!(decoded.bytes, vec![1, 2, 3, 4]);
    }
}
