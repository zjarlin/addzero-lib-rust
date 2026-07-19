//! Shared upload helpers for HTTP endpoints.

use std::path::{Path, PathBuf};

use anyhow::{Context, anyhow, bail};
use axum::extract::{Multipart, multipart::Field};
use axum::routing::get_service;
use axum::Router;
use az_str::sanitize::sanitize_path_segment;
use tokio::{fs, io::AsyncWriteExt};
use tower_http::services::ServeDir;
use uuid::Uuid;

/// Default per-request upload limit for local endpoints.
pub const DEFAULT_UPLOAD_LIMIT_BYTES: usize = 512 * 1024 * 1024;

/// Options for saving a single multipart file field.
#[derive(Clone, Debug)]
pub struct MultipartUploadOptions {
    pub field_name: String,
    pub storage_dir: PathBuf,
    pub public_url_prefix: String,
    pub fallback_file_name: String,
}

/// Result returned after saving a multipart file field.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StoredUpload {
    pub original_file_name: Option<String>,
    pub stored_file_name: String,
    pub storage_path: PathBuf,
    pub public_url: String,
    pub byte_len: u64,
}

/// Save one multipart file field to local storage and return its public URL.
pub async fn save_single_multipart_upload(
    mut multipart: Multipart,
    options: MultipartUploadOptions,
) -> anyhow::Result<StoredUpload> {
    validate_upload_options(&options)?;
    fs::create_dir_all(&options.storage_dir)
        .await
        .with_context(|| format!("创建上传目录失败: {}", options.storage_dir.display()))?;

    while let Some(field) = multipart
        .next_field()
        .await
        .context("读取 multipart 表单失败")?
    {
        if field.name() == Some(options.field_name.as_str()) {
            return save_upload_field(field, &options).await;
        }
    }

    bail!("缺少上传字段: {}", options.field_name)
}

/// Build a static file service for a local upload directory.
pub fn upload_file_service(path: &str, storage_dir: impl Into<PathBuf>) -> Router {
    Router::new().nest_service(path, get_service(ServeDir::new(storage_dir.into())))
}

fn validate_upload_options(options: &MultipartUploadOptions) -> anyhow::Result<()> {
    if options.field_name.trim().is_empty() {
        bail!("上传字段名不能为空");
    }
    if options.public_url_prefix.trim().is_empty() {
        bail!("上传访问前缀不能为空");
    }
    if options.fallback_file_name.trim().is_empty() {
        bail!("上传默认文件名不能为空");
    }
    Ok(())
}

async fn save_upload_field(
    mut field: Field<'_>,
    options: &MultipartUploadOptions,
) -> anyhow::Result<StoredUpload> {
    let original_file_name = field.file_name().map(ToOwned::to_owned);
    let stored_file_name = stored_file_name(original_file_name.as_deref(), &options.fallback_file_name);
    let storage_path = safe_join(&options.storage_dir, &stored_file_name)?;
    let mut file = fs::File::create(&storage_path)
        .await
        .with_context(|| format!("创建上传文件失败: {}", storage_path.display()))?;

    let mut byte_len = 0_u64;
    while let Some(chunk) = field.chunk().await.context("读取上传文件内容失败")? {
        byte_len += chunk.len() as u64;
        file.write_all(&chunk)
            .await
            .with_context(|| format!("写入上传文件失败: {}", storage_path.display()))?;
    }
    file.flush()
        .await
        .with_context(|| format!("刷新上传文件失败: {}", storage_path.display()))?;

    if byte_len == 0 {
        let _ = fs::remove_file(&storage_path).await;
        bail!("上传文件内容为空");
    }

    Ok(StoredUpload {
        original_file_name,
        stored_file_name: stored_file_name.clone(),
        storage_path,
        public_url: public_url(&options.public_url_prefix, &stored_file_name),
        byte_len,
    })
}

fn stored_file_name(original_file_name: Option<&str>, fallback_file_name: &str) -> String {
    let raw_name = original_file_name
        .filter(|name| !name.trim().is_empty())
        .unwrap_or(fallback_file_name);
    let sanitized = sanitize_path_segment(raw_name);
    let file_name = if sanitized.is_empty() {
        sanitize_path_segment(fallback_file_name)
    } else {
        sanitized
    };
    format!("{}-{file_name}", Uuid::new_v4())
}

fn safe_join(base: &Path, file_name: &str) -> anyhow::Result<PathBuf> {
    let candidate = PathBuf::from(file_name);
    if candidate.components().count() != 1 {
        return Err(anyhow!("上传文件名不能包含路径段: {file_name}"));
    }
    Ok(base.join(candidate))
}

fn public_url(prefix: &str, file_name: &str) -> String {
    format!("{}/{}", prefix.trim_end_matches('/'), file_name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stored_file_name_sanitizes_path_segments() {
        let name = stored_file_name(Some("../a b.mp4"), "input.mp4");
        assert!(name.ends_with("-a-b.mp4"));
        assert!(!name.contains('/'));
    }

    #[test]
    fn public_url_joins_prefix_and_name() {
        assert_eq!(
            public_url("/api/demo/uploads/", "video.mp4"),
            "/api/demo/uploads/video.mp4"
        );
    }
}
