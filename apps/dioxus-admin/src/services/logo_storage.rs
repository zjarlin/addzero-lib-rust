use std::{future::Future, pin::Pin, rc::Rc};

#[cfg(not(target_arch = "wasm32"))]
use chrono::Utc;
use serde::{Deserialize, Serialize};
use thiserror::Error;

pub type LocalBoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + 'a>>;

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
        Rc::new(NativeLogoStorage::from_env())
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
struct NativeLogoStorage {
    backend: NativeBackend,
}

#[cfg(not(target_arch = "wasm32"))]
enum NativeBackend {
    Rustfs(RustfsBackend),
    MissingConfig(String),
}

#[cfg(not(target_arch = "wasm32"))]
impl NativeLogoStorage {
    fn from_env() -> Self {
        match RustfsBackend::from_env() {
            Ok(backend) => Self {
                backend: NativeBackend::Rustfs(backend),
            },
            Err(reason) => Self {
                backend: NativeBackend::MissingConfig(reason),
            },
        }
    }

    fn upload_logo_blocking(&self, input: LogoUploadRequest) -> LogoStorageResult<StoredLogoDto> {
        match &self.backend {
            NativeBackend::Rustfs(backend) => backend.upload_logo_blocking(input),
            NativeBackend::MissingConfig(reason) => Err(LogoStorageError::new(format!(
                "MinIO / S3 存储未配置：{reason}"
            ))),
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl LogoStorageApi for NativeLogoStorage {
    fn upload_logo(
        &self,
        input: LogoUploadRequest,
    ) -> LocalBoxFuture<'_, LogoStorageResult<StoredLogoDto>> {
        let result = self.upload_logo_blocking(input);
        Box::pin(async move { result })
    }
}

#[cfg(not(target_arch = "wasm32"))]
struct RustfsBackend {
    client: std::sync::Arc<dyn addzero_rustfs::S3StorageClient>,
    bucket: String,
}

#[cfg(not(target_arch = "wasm32"))]
impl RustfsBackend {
    fn from_env() -> Result<Self, String> {
        let endpoint = required_env("ADMIN_LOGO_S3_ENDPOINT")?;
        let access_key = required_env("ADMIN_LOGO_S3_ACCESS_KEY")?;
        let secret_key = required_env("ADMIN_LOGO_S3_SECRET_KEY")?;
        let bucket = required_env("ADMIN_LOGO_S3_BUCKET")?;
        let region = std::env::var("ADMIN_LOGO_S3_REGION").unwrap_or_else(|_| "us-east-1".into());
        let config = addzero_rustfs::S3ClientConfig::new(endpoint.clone(), access_key, secret_key)
            .with_region(region)
            .with_path_style_access(true);

        Ok(Self {
            client: addzero_rustfs::create_storage_client(config),
            bucket,
        })
    }

    fn upload_logo_blocking(&self, input: LogoUploadRequest) -> LogoStorageResult<StoredLogoDto> {
        validate_logo(&input)?;

        addzero_rustfs::ensure_bucket(self.client.as_ref(), &self.bucket)
            .map_err(|err| LogoStorageError::new(format!("创建 bucket 失败：{err}")))?;

        let content_type = normalized_content_type(input.content_type.as_deref());
        let object_key = build_object_key(&input.file_name);

        addzero_rustfs::put_object_bytes(
            self.client.as_ref(),
            &self.bucket,
            &object_key,
            &input.bytes,
            Some(content_type.as_str()),
        )
        .map_err(|err| LogoStorageError::new(format!("上传 logo 到 RustFS 失败：{err}")))?;

        Ok(StoredLogoDto {
            object_key: object_key.clone(),
            relative_path: build_relative_path(&self.bucket, &object_key),
            file_name: input.file_name,
            content_type,
            backend_label: format!("MinIO / S3-compatible · bucket `{}`", self.bucket),
        })
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn required_env(name: &str) -> Result<String, String> {
    std::env::var(name)
        .map(|value| value.trim().to_string())
        .map_err(|_| format!("缺少环境变量 `{name}`"))
        .and_then(|value| {
            if value.is_empty() {
                Err(format!("环境变量 `{name}` 不能为空"))
            } else {
                Ok(value)
            }
        })
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
        .map(|ext| sanitize_segment(ext))
        .filter(|ext| !ext.is_empty())
        .unwrap_or_else(|| "png".to_string());

    format!("logos/logo-{}.{}", Utc::now().timestamp_millis(), extension)
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
    NativeLogoStorage::from_env().upload_logo_blocking(input)
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
