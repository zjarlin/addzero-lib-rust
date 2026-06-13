use crate::client::S3StorageClient;
use crate::types::{ObjectMetadata, PresignedUrl};
use std::collections::BTreeMap;
use std::path::Path;

/// 确保 bucket 存在；不存在时自动创建。
pub fn ensure_bucket(client: &dyn S3StorageClient, bucket_name: &str) -> anyhow::Result<()> {
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
) -> anyhow::Result<()> {
    client.put_object_bytes(bucket_name, key, data, content_type, &BTreeMap::new())
}

/// 上传本地文件为对象。
pub fn put_object_file(
    client: &dyn S3StorageClient,
    bucket_name: &str,
    key: &str,
    file: &Path,
    content_type: Option<&str>,
) -> anyhow::Result<()> {
    client.put_object_file(bucket_name, key, file, content_type, &BTreeMap::new())
}

/// 读取对象全部字节。
pub fn get_object(
    client: &dyn S3StorageClient,
    bucket_name: &str,
    key: &str,
) -> anyhow::Result<Vec<u8>> {
    client.get_object(bucket_name, key)
}

/// 删除单个对象。
pub fn delete_object(
    client: &dyn S3StorageClient,
    bucket_name: &str,
    key: &str,
) -> anyhow::Result<()> {
    client.delete_object(bucket_name, key)
}

/// 批量删除对象。
pub fn delete_objects(
    client: &dyn S3StorageClient,
    bucket_name: &str,
    keys: &[String],
) -> anyhow::Result<()> {
    client.delete_objects(bucket_name, keys)
}

/// 判断对象是否存在。
pub fn object_exists(
    client: &dyn S3StorageClient,
    bucket_name: &str,
    key: &str,
) -> anyhow::Result<bool> {
    client.object_exists(bucket_name, key)
}

/// 按 prefix 列出对象。
pub fn list_objects(
    client: &dyn S3StorageClient,
    bucket_name: &str,
    prefix: Option<&str>,
    recursive: bool,
    max_keys: usize,
) -> anyhow::Result<Vec<ObjectMetadata>> {
    client.list_objects(bucket_name, prefix, recursive, max_keys)
}

/// 复制对象到目标 bucket/key。
pub fn copy_object(
    client: &dyn S3StorageClient,
    source_bucket: &str,
    source_key: &str,
    target_bucket: &str,
    target_key: &str,
) -> anyhow::Result<()> {
    client.copy_object(source_bucket, source_key, target_bucket, target_key)
}

/// 生成对象下载预签名 URL。
pub fn get_presigned_url(
    client: &dyn S3StorageClient,
    bucket_name: &str,
    key: &str,
    expiration_seconds: u64,
) -> anyhow::Result<PresignedUrl> {
    client.generate_presigned_url(bucket_name, key, expiration_seconds)
}
