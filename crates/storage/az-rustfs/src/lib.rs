#![doc = include_str!("../README.md")]

mod client;
mod helper;
mod progress;
mod types;

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

/// Namespace-style entry point for constructing S3-compatible storage clients.
#[apply(plain_default_copy_eq)]
pub struct Rustfs;

impl Rustfs {
    /// Creates a storage client from explicit S3-compatible configuration.
    pub fn storage_client(config: impl Into<S3ClientConfig>) -> Arc<dyn S3StorageClient> {
        create_storage_client(config)
    }

    /// Creates a storage client from the higher-level RustFS configuration.
    pub fn client(config: RustfsConfig) -> Arc<dyn S3StorageClient> {
        create_client(config)
    }

    /// Creates a storage client using [`RustfsConfig::default`].
    pub fn default_client() -> Arc<dyn S3StorageClient> {
        create_default_client()
    }

    /// Creates a storage client through an injected factory.
    pub fn storage_client_with_factory(
        factory: &dyn S3StorageClientFactory,
        config: impl Into<S3ClientConfig>,
    ) -> Arc<dyn S3StorageClient> {
        factory.create_client(config.into())
    }

    /// Creates the factory-defined default storage client.
    pub fn default_client_with_factory(
        factory: &dyn S3StorageClientFactory,
    ) -> Arc<dyn S3StorageClient> {
        factory.create_default_client()
    }
}

pub fn create_storage_client(config: impl Into<S3ClientConfig>) -> Arc<dyn S3StorageClient> {
    Arc::new(BlockingS3StorageClient::new(config.into()))
}

pub fn create_client(config: RustfsConfig) -> Arc<dyn S3StorageClient> {
    create_storage_client(config)
}

pub fn create_default_client() -> Arc<dyn S3StorageClient> {
    create_storage_client(RustfsConfig::default())
}

pub fn ensure_bucket(client: &dyn S3StorageClient, bucket_name: &str) -> StorageResult<()> {
    if client.bucket_exists(bucket_name)? {
        Ok(())
    } else {
        client.create_bucket(bucket_name)
    }
}

pub fn put_object_bytes(
    client: &dyn S3StorageClient,
    bucket_name: &str,
    key: &str,
    data: &[u8],
    content_type: Option<&str>,
) -> StorageResult<()> {
    client.put_object_bytes(bucket_name, key, data, content_type, &BTreeMap::new())
}

pub fn put_object_file(
    client: &dyn S3StorageClient,
    bucket_name: &str,
    key: &str,
    file: &Path,
    content_type: Option<&str>,
) -> StorageResult<()> {
    client.put_object_file(bucket_name, key, file, content_type, &BTreeMap::new())
}

pub fn get_object(
    client: &dyn S3StorageClient,
    bucket_name: &str,
    key: &str,
) -> StorageResult<Vec<u8>> {
    client.get_object(bucket_name, key)
}

pub fn delete_object(
    client: &dyn S3StorageClient,
    bucket_name: &str,
    key: &str,
) -> StorageResult<()> {
    client.delete_object(bucket_name, key)
}

pub fn delete_objects(
    client: &dyn S3StorageClient,
    bucket_name: &str,
    keys: &[String],
) -> StorageResult<()> {
    client.delete_objects(bucket_name, keys)
}

pub fn object_exists(
    client: &dyn S3StorageClient,
    bucket_name: &str,
    key: &str,
) -> StorageResult<bool> {
    client.object_exists(bucket_name, key)
}

pub fn list_objects(
    client: &dyn S3StorageClient,
    bucket_name: &str,
    prefix: Option<&str>,
    recursive: bool,
    max_keys: usize,
) -> StorageResult<Vec<ObjectMetadata>> {
    client.list_objects(bucket_name, prefix, recursive, max_keys)
}

pub fn copy_object(
    client: &dyn S3StorageClient,
    source_bucket: &str,
    source_key: &str,
    target_bucket: &str,
    target_key: &str,
) -> StorageResult<()> {
    client.copy_object(source_bucket, source_key, target_bucket, target_key)
}

pub fn get_presigned_url(
    client: &dyn S3StorageClient,
    bucket_name: &str,
    key: &str,
    expiration_seconds: u64,
) -> StorageResult<PresignedUrl> {
    client.generate_presigned_url(bucket_name, key, expiration_seconds)
}
