use std::rc::Rc;

#[cfg(not(target_arch = "wasm32"))]
use std::{
    fs,
    path::{Component, Path, PathBuf},
};

use az_derive_aliases::{apply, error_eq, serde_eq};
#[cfg(not(target_arch = "wasm32"))]
use chrono::Utc;

pub use super::LocalBoxFuture;

pub const LOGO_PREVIEW_BASE_URL: &str = "/api/admin/storage/logo";

#[apply(serde_eq)]
pub struct LogoUploadRequest {
    pub file_name: String,
    pub content_type: Option<String>,
    #[serde(with = "base64_bytes")]
    pub bytes: Vec<u8>,
}

#[apply(serde_eq)]
pub struct StoredLogoDto {
    pub object_key: String,
    pub relative_path: String,
    pub file_name: String,
    pub content_type: String,
    pub backend_label: String,
}

#[apply(error_eq)]
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
        LocalLogoBackend::open()?.upload_logo_blocking(input)
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
struct LocalLogoBackend {
    config_root: PathBuf,
    backend_label: String,
}

#[cfg(not(target_arch = "wasm32"))]
impl LocalLogoBackend {
    fn open() -> LogoStorageResult<Self> {
        let config_root = config_root()?;
        fs::create_dir_all(config_root.join("branding/logos"))
            .map_err(|err| LogoStorageError::new(format!("创建本地 logo 目录失败：{err}")))?;
        Ok(Self {
            backend_label: format!("local fs · {}", config_root.display()),
            config_root,
        })
    }

    fn upload_logo_blocking(&self, input: LogoUploadRequest) -> LogoStorageResult<StoredLogoDto> {
        validate_logo(&input)?;
        let content_type = normalized_content_type(input.content_type.as_deref(), &input.file_name);
        let relative_path = build_object_key(&input.file_name);
        let target_path = self.config_root.join(&relative_path);
        if let Some(parent) = target_path.parent() {
            fs::create_dir_all(parent)
                .map_err(|err| LogoStorageError::new(format!("创建 logo 父目录失败：{err}")))?;
        }
        fs::write(&target_path, &input.bytes)
            .map_err(|err| LogoStorageError::new(format!("写入 logo 文件失败：{err}")))?;

        Ok(StoredLogoDto {
            object_key: relative_path.clone(),
            relative_path,
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

    if let Some(content_type) = input.content_type.as_deref()
        && !content_type.starts_with("image/")
    {
        return Err(LogoStorageError::new("Logo 只接受图片文件"));
    }

    Ok(())
}

#[cfg(not(target_arch = "wasm32"))]
fn normalized_content_type(content_type: Option<&str>, file_name: &str) -> String {
    match content_type {
        Some(value) if value.starts_with("image/") => value.to_string(),
        _ => mime_guess::from_path(file_name)
            .first_raw()
            .unwrap_or("image/png")
            .to_string(),
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
fn sanitize_segment(raw: &str) -> String {
    raw.chars()
        .filter(|ch| ch.is_ascii_alphanumeric())
        .collect::<String>()
        .to_lowercase()
}

#[cfg(not(target_arch = "wasm32"))]
fn config_root() -> LogoStorageResult<PathBuf> {
    let env_path = az_persistence::local_env_path()
        .ok_or_else(|| LogoStorageError::new("无法定位 ~/.config/aio 目录"))?;
    env_path
        .parent()
        .map(Path::to_path_buf)
        .ok_or_else(|| LogoStorageError::new("无法定位 ~/.config/aio 目录"))
}

#[cfg(not(target_arch = "wasm32"))]
fn sanitize_relative_path(relative_path: &str) -> LogoStorageResult<PathBuf> {
    let path = Path::new(relative_path);
    if path.is_absolute() {
        return Err(LogoStorageError::new("logo 路径不能是绝对路径"));
    }
    let mut sanitized = PathBuf::new();
    for component in path.components() {
        match component {
            Component::Normal(part) => sanitized.push(part),
            _ => return Err(LogoStorageError::new("logo 路径包含非法路径段")),
        }
    }
    if sanitized.as_os_str().is_empty() {
        return Err(LogoStorageError::new("logo 路径不能为空"));
    }
    Ok(sanitized)
}

#[cfg(not(target_arch = "wasm32"))]
pub fn upload_logo_on_server(input: LogoUploadRequest) -> LogoStorageResult<StoredLogoDto> {
    NativeLogoStorage::upload_logo_blocking(input)
}

#[cfg(not(target_arch = "wasm32"))]
pub fn read_logo_on_server(relative_path: &str) -> LogoStorageResult<(String, Vec<u8>)> {
    let root = config_root()?;
    let relative_path = sanitize_relative_path(relative_path)?;
    let path = root.join(&relative_path);
    let bytes = fs::read(&path)
        .map_err(|err| LogoStorageError::new(format!("读取本地 logo 失败：{err}")))?;
    let content_type = mime_guess::from_path(&path)
        .first_raw()
        .unwrap_or("application/octet-stream")
        .to_string();
    Ok((content_type, bytes))
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
    use super::LogoUploadRequest;
    use super::build_preview_url;

    #[test]
    fn preview_url_should_use_local_logo_route() {
        assert_eq!(
            build_preview_url("branding/logos/logo-1.png"),
            "/api/admin/storage/logo/branding/logos/logo-1.png"
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
