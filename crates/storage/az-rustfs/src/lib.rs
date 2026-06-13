#![doc = include_str!("../README.md")]

automod::dir!("src");

pub use client::{
    BlockingS3StorageClient, DefaultS3StorageClientFactory, InMemoryS3StorageClient,
    S3StorageClient, S3StorageClientFactory, StorageError, StorageResult,
};
pub use helper::{
    ListRequest, RustfsResult, build_list_request, calculate_optimal_part_size,
    generate_part_infos, get_presigned_object_url, guess_content_type, metadata_keys,
    resume_or_upload, should_use_multipart_upload, smart_upload, upload_multipart,
};
pub use progress::{
    InMemoryUploadProgressStorage, MultipartUploadConfig, MultipartUploadResult, PartInfo,
    PartStatus, SpeedTrackingProgressListener, UploadProgress, UploadProgressData,
    UploadProgressListener, UploadProgressStorage, UploadStatus, UploadStatusType,
};
pub use types::{ObjectMetadata, PresignedUrl, RustfsConfig, S3ClientConfig};

use az_derive_aliases::{apply, plain_default_copy_eq};
use std::collections::BTreeMap;
use std::path::Path;
use std::sync::Arc;

/// 构造 S3 兼容对象存储客户端的命名空间入口。
#[apply(plain_default_copy_eq)]
pub struct Rustfs;

impl Rustfs {
    /// 使用显式 S3 兼容配置创建存储客户端。
    pub fn storage_client(config: impl Into<S3ClientConfig>) -> Arc<dyn S3StorageClient> {
        create_storage_client(config)
    }

    /// 使用高层 RustFS 配置创建存储客户端。
    pub fn client(config: RustfsConfig) -> Arc<dyn S3StorageClient> {
        create_client(config)
    }

    /// 使用 [`RustfsConfig::default`] 创建默认存储客户端。
    pub fn default_client() -> Arc<dyn S3StorageClient> {
        create_default_client()
    }

    /// 通过注入的工厂创建存储客户端。
    pub fn storage_client_with_factory(
        factory: &dyn S3StorageClientFactory,
        config: impl Into<S3ClientConfig>,
    ) -> Arc<dyn S3StorageClient> {
        factory.create_client(config.into())
    }

    /// 创建工厂定义的默认存储客户端。
    pub fn default_client_with_factory(
        factory: &dyn S3StorageClientFactory,
    ) -> Arc<dyn S3StorageClient> {
        factory.create_default_client()
    }
}

/// 使用 S3 兼容配置创建存储客户端。
pub fn create_storage_client(config: impl Into<S3ClientConfig>) -> Arc<dyn S3StorageClient> {
    Arc::new(BlockingS3StorageClient::new(config.into()))
}

/// 使用 RustFS 配置创建存储客户端。
pub fn create_client(config: RustfsConfig) -> Arc<dyn S3StorageClient> {
    create_storage_client(config)
}

/// 创建默认本地存储客户端。
pub fn create_default_client() -> Arc<dyn S3StorageClient> {
    create_storage_client(RustfsConfig::default())
}

/// 确保 bucket 存在；不存在时自动创建。
pub fn ensure_bucket(client: &dyn S3StorageClient, bucket_name: &str) -> StorageResult<()> {
    if client.bucket_exists(bucket_name)? {
        Ok(())
    } else {
        client.create_bucket(bucket_name)
    }
}

/// 上传内存字节为对象。
pub fn put_object_bytes(
    client: &dyn S3StorageClient,
    bucket_name: &str,
    key: &str,
    data: &[u8],
    content_type: Option<&str>,
) -> StorageResult<()> {
    client.put_object_bytes(bucket_name, key, data, content_type, &BTreeMap::new())
}

/// 上传本地文件为对象。
pub fn put_object_file(
    client: &dyn S3StorageClient,
    bucket_name: &str,
    key: &str,
    file: &Path,
    content_type: Option<&str>,
) -> StorageResult<()> {
    client.put_object_file(bucket_name, key, file, content_type, &BTreeMap::new())
}

/// 读取对象全部字节。
pub fn get_object(
    client: &dyn S3StorageClient,
    bucket_name: &str,
    key: &str,
) -> StorageResult<Vec<u8>> {
    client.get_object(bucket_name, key)
}

/// 删除单个对象。
pub fn delete_object(
    client: &dyn S3StorageClient,
    bucket_name: &str,
    key: &str,
) -> StorageResult<()> {
    client.delete_object(bucket_name, key)
}

/// 批量删除对象。
pub fn delete_objects(
    client: &dyn S3StorageClient,
    bucket_name: &str,
    keys: &[String],
) -> StorageResult<()> {
    client.delete_objects(bucket_name, keys)
}

/// 判断对象是否存在。
pub fn object_exists(
    client: &dyn S3StorageClient,
    bucket_name: &str,
    key: &str,
) -> StorageResult<bool> {
    client.object_exists(bucket_name, key)
}

/// 按 prefix 列出对象。
pub fn list_objects(
    client: &dyn S3StorageClient,
    bucket_name: &str,
    prefix: Option<&str>,
    recursive: bool,
    max_keys: usize,
) -> StorageResult<Vec<ObjectMetadata>> {
    client.list_objects(bucket_name, prefix, recursive, max_keys)
}

/// 复制对象到目标 bucket/key。
pub fn copy_object(
    client: &dyn S3StorageClient,
    source_bucket: &str,
    source_key: &str,
    target_bucket: &str,
    target_key: &str,
) -> StorageResult<()> {
    client.copy_object(source_bucket, source_key, target_bucket, target_key)
}

/// 生成对象下载预签名 URL。
pub fn get_presigned_url(
    client: &dyn S3StorageClient,
    bucket_name: &str,
    key: &str,
    expiration_seconds: u64,
) -> StorageResult<PresignedUrl> {
    client.generate_presigned_url(bucket_name, key, expiration_seconds)
}
