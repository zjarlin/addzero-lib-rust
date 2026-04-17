#![forbid(unsafe_code)]

use std::collections::BTreeMap;
use std::io::Read;
use std::path::Path;
use std::sync::{Arc, OnceLock, RwLock};
use thiserror::Error;
use tool_rustfs::{
    ObjectMetadata, PresignedUrl, RustfsConfig, S3ClientConfig, S3StorageClient, StorageError,
    create_storage_client, guess_content_type,
};

pub const DEFAULT_PRESIGNED_EXPIRATION_SECONDS: u64 = 3600;

#[derive(Debug, Error)]
pub enum MinioError {
    #[error("invalid minio configuration: {0}")]
    InvalidConfig(String),
    #[error("storage backend error: {0}")]
    Storage(#[from] StorageError),
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}

pub type MinioResult<T> = Result<T, MinioError>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MinioConfig {
    pub endpoint: String,
    pub access_key: String,
    pub secret_key: String,
    pub region: Option<String>,
    pub path_style_access: bool,
}

impl MinioConfig {
    pub fn new(
        endpoint: impl Into<String>,
        access_key: impl Into<String>,
        secret_key: impl Into<String>,
    ) -> Self {
        Self {
            endpoint: endpoint.into(),
            access_key: access_key.into(),
            secret_key: secret_key.into(),
            region: None,
            path_style_access: true,
        }
    }

    pub fn builder(
        endpoint: impl Into<String>,
        access_key: impl Into<String>,
        secret_key: impl Into<String>,
    ) -> MinioConfigBuilder {
        MinioConfigBuilder {
            endpoint: endpoint.into(),
            access_key: access_key.into(),
            secret_key: secret_key.into(),
            region: None,
            path_style_access: true,
        }
    }

    pub fn validate(&self) -> MinioResult<()> {
        if self.endpoint.trim().is_empty() {
            return Err(MinioError::InvalidConfig(
                "endpoint cannot be blank".to_owned(),
            ));
        }
        if self.access_key.trim().is_empty() {
            return Err(MinioError::InvalidConfig(
                "access_key cannot be blank".to_owned(),
            ));
        }
        if self.secret_key.trim().is_empty() {
            return Err(MinioError::InvalidConfig(
                "secret_key cannot be blank".to_owned(),
            ));
        }
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct MinioConfigBuilder {
    endpoint: String,
    access_key: String,
    secret_key: String,
    region: Option<String>,
    path_style_access: bool,
}

impl MinioConfigBuilder {
    pub fn region(mut self, value: impl Into<String>) -> Self {
        self.region = Some(value.into());
        self
    }

    pub fn path_style_access(mut self, value: bool) -> Self {
        self.path_style_access = value;
        self
    }

    pub fn build(self) -> MinioResult<MinioConfig> {
        let config = MinioConfig {
            endpoint: self.endpoint,
            access_key: self.access_key,
            secret_key: self.secret_key,
            region: self.region,
            path_style_access: self.path_style_access,
        };
        config.validate()?;
        Ok(config)
    }
}

impl From<MinioConfig> for S3ClientConfig {
    fn from(value: MinioConfig) -> Self {
        let mut config = S3ClientConfig::new(value.endpoint, value.access_key, value.secret_key)
            .with_path_style_access(value.path_style_access);
        if let Some(region) = value.region {
            config = config.with_region(region);
        }
        config
    }
}

impl From<RustfsConfig> for MinioConfig {
    fn from(value: RustfsConfig) -> Self {
        Self {
            endpoint: value.endpoint,
            access_key: value.access_key,
            secret_key: value.secret_key,
            region: Some(value.region),
            path_style_access: true,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MinioOperationResult {
    pub message: String,
}

impl MinioOperationResult {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ObjectInfo {
    pub object_name: String,
    pub size: u64,
    pub etag: Option<String>,
    pub last_modified: Option<String>,
    pub content_type: Option<String>,
}

impl From<ObjectMetadata> for ObjectInfo {
    fn from(value: ObjectMetadata) -> Self {
        Self {
            object_name: value.key,
            size: value.size,
            etag: value.etag,
            last_modified: value.last_modified,
            content_type: value.content_type,
        }
    }
}

#[derive(Clone)]
pub struct MinioClient {
    config: MinioConfig,
    storage: Arc<dyn S3StorageClient>,
}

impl MinioClient {
    pub fn new(config: MinioConfig) -> MinioResult<Self> {
        config.validate()?;
        let storage = create_storage_client(config.clone());
        Ok(Self { config, storage })
    }

    pub fn from_storage_client(
        config: MinioConfig,
        storage: Arc<dyn S3StorageClient>,
    ) -> MinioResult<Self> {
        config.validate()?;
        Ok(Self { config, storage })
    }

    pub fn config(&self) -> &MinioConfig {
        &self.config
    }

    pub fn storage(&self) -> &Arc<dyn S3StorageClient> {
        &self.storage
    }

    pub fn bucket_exists(&self, bucket_name: &str) -> MinioResult<bool> {
        Ok(self.storage.bucket_exists(bucket_name)?)
    }

    pub fn create_bucket(&self, bucket_name: &str) -> MinioResult<MinioOperationResult> {
        if self.bucket_exists(bucket_name)? {
            return Err(MinioError::Storage(StorageError::InvalidConfig(format!(
                "bucket already exists: {bucket_name}"
            ))));
        }
        self.storage.create_bucket(bucket_name)?;
        Ok(MinioOperationResult::new(format!(
            "Bucket created: {bucket_name}"
        )))
    }

    pub fn ensure_bucket(&self, bucket_name: &str) -> MinioResult<MinioOperationResult> {
        if self.bucket_exists(bucket_name)? {
            Ok(MinioOperationResult::new(format!(
                "Bucket already exists: {bucket_name}"
            )))
        } else {
            self.storage.create_bucket(bucket_name)?;
            Ok(MinioOperationResult::new(format!(
                "Bucket created: {bucket_name}"
            )))
        }
    }

    pub fn list_buckets(&self) -> MinioResult<Vec<String>> {
        self.storage.list_buckets().map_err(MinioError::from)
    }

    pub fn delete_bucket(
        &self,
        bucket_name: &str,
        force: bool,
    ) -> MinioResult<MinioOperationResult> {
        if !self.bucket_exists(bucket_name)? {
            return Err(MinioError::Storage(StorageError::BucketNotFound {
                bucket: bucket_name.to_owned(),
            }));
        }

        if force {
            let objects = self.list_objects(bucket_name, None, true)?;
            if !objects.is_empty() {
                let keys = objects
                    .into_iter()
                    .map(|object| object.object_name)
                    .collect::<Vec<_>>();
                self.storage.delete_objects(bucket_name, &keys)?;
            }
        }

        self.storage.delete_bucket(bucket_name)?;
        Ok(MinioOperationResult::new(format!(
            "Bucket deleted: {bucket_name}"
        )))
    }

    pub fn put_object_bytes(
        &self,
        bucket_name: &str,
        object_name: &str,
        data: &[u8],
        content_type: Option<&str>,
    ) -> MinioResult<MinioOperationResult> {
        self.storage.put_object_bytes(
            bucket_name,
            object_name,
            data,
            Some(content_type.unwrap_or("application/octet-stream")),
            &BTreeMap::new(),
        )?;
        Ok(MinioOperationResult::new(format!(
            "Object uploaded: {bucket_name}/{object_name}"
        )))
    }

    pub fn put_object_file(
        &self,
        bucket_name: &str,
        object_name: &str,
        file: &Path,
        content_type: Option<&str>,
    ) -> MinioResult<MinioOperationResult> {
        let detected = content_type.map(ToOwned::to_owned).unwrap_or_else(|| {
            let guessed_from_object_name = guess_content_type(Path::new(object_name));
            if guessed_from_object_name == "application/octet-stream" {
                guess_content_type(file)
            } else {
                guessed_from_object_name
            }
        });
        self.storage.put_object_file(
            bucket_name,
            object_name,
            file,
            Some(&detected),
            &BTreeMap::new(),
        )?;
        Ok(MinioOperationResult::new(format!(
            "Object uploaded: {bucket_name}/{object_name}"
        )))
    }

    pub fn put_object_reader<R: Read>(
        &self,
        bucket_name: &str,
        object_name: &str,
        mut reader: R,
        content_type: Option<&str>,
    ) -> MinioResult<MinioOperationResult> {
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes)?;
        self.put_object_bytes(bucket_name, object_name, &bytes, content_type)
    }

    pub fn get_object(&self, bucket_name: &str, object_name: &str) -> MinioResult<Vec<u8>> {
        self.storage
            .get_object(bucket_name, object_name)
            .map_err(MinioError::from)
    }

    pub fn get_object_to_file(
        &self,
        bucket_name: &str,
        object_name: &str,
        file: &Path,
    ) -> MinioResult<MinioOperationResult> {
        self.storage
            .get_object_to_file(bucket_name, object_name, file)?;
        Ok(MinioOperationResult::new(format!(
            "Object downloaded to: {}",
            file.display()
        )))
    }

    pub fn stat_object(
        &self,
        bucket_name: &str,
        object_name: &str,
    ) -> MinioResult<Option<ObjectInfo>> {
        self.storage
            .get_object_metadata(bucket_name, object_name)
            .map(|metadata| metadata.map(ObjectInfo::from))
            .map_err(MinioError::from)
    }

    pub fn object_exists(&self, bucket_name: &str, object_name: &str) -> MinioResult<bool> {
        self.storage
            .object_exists(bucket_name, object_name)
            .map_err(MinioError::from)
    }

    pub fn list_objects(
        &self,
        bucket_name: &str,
        prefix: Option<&str>,
        recursive: bool,
    ) -> MinioResult<Vec<ObjectInfo>> {
        self.storage
            .list_objects(bucket_name, prefix, recursive, usize::MAX)
            .map(|objects| objects.into_iter().map(ObjectInfo::from).collect())
            .map_err(MinioError::from)
    }

    pub fn delete_object(
        &self,
        bucket_name: &str,
        object_name: &str,
    ) -> MinioResult<MinioOperationResult> {
        self.storage.delete_object(bucket_name, object_name)?;
        Ok(MinioOperationResult::new(format!(
            "Object deleted: {bucket_name}/{object_name}"
        )))
    }

    pub fn delete_objects(
        &self,
        bucket_name: &str,
        object_names: &[String],
    ) -> MinioResult<MinioOperationResult> {
        self.storage.delete_objects(bucket_name, object_names)?;
        Ok(MinioOperationResult::new(format!(
            "Deleted {} objects",
            object_names.len()
        )))
    }

    pub fn copy_object(
        &self,
        source_bucket: &str,
        source_object: &str,
        target_bucket: &str,
        target_object: &str,
    ) -> MinioResult<MinioOperationResult> {
        self.storage
            .copy_object(source_bucket, source_object, target_bucket, target_object)?;
        Ok(MinioOperationResult::new(format!(
            "Object copied: {source_bucket}/{source_object} -> {target_bucket}/{target_object}"
        )))
    }

    pub fn get_presigned_object_url(
        &self,
        bucket_name: &str,
        object_name: &str,
    ) -> MinioResult<String> {
        self.get_presigned_object_url_with_expiration(
            bucket_name,
            object_name,
            DEFAULT_PRESIGNED_EXPIRATION_SECONDS,
        )
        .map(|details| details.url)
    }

    pub fn get_presigned_object_url_with_expiration(
        &self,
        bucket_name: &str,
        object_name: &str,
        expiration_seconds: u64,
    ) -> MinioResult<PresignedUrl> {
        self.storage
            .generate_presigned_url(bucket_name, object_name, expiration_seconds)
            .map_err(MinioError::from)
    }
}

static CLIENTS: OnceLock<RwLock<BTreeMap<String, MinioClient>>> = OnceLock::new();

pub fn create_client(config: MinioConfig) -> MinioResult<MinioClient> {
    MinioClient::new(config)
}

pub fn create_client_with_credentials(
    endpoint: impl Into<String>,
    access_key: impl Into<String>,
    secret_key: impl Into<String>,
) -> MinioResult<MinioClient> {
    create_client(MinioConfig::new(endpoint, access_key, secret_key))
}

pub fn get_or_create_client(key: &str, config: MinioConfig) -> MinioResult<MinioClient> {
    let lock = CLIENTS.get_or_init(|| RwLock::new(BTreeMap::new()));
    if let Some(existing) = lock
        .read()
        .expect("minio client cache should not be poisoned")
        .get(key)
        .cloned()
    {
        return Ok(existing);
    }

    let client = create_client(config)?;
    lock.write()
        .expect("minio client cache should not be poisoned")
        .insert(key.to_owned(), client.clone());
    Ok(client)
}

pub fn client(config: MinioConfig) -> MinioResult<MinioClient> {
    create_client(config)
}

pub fn bucket_exists(client: &MinioClient, bucket_name: &str) -> MinioResult<bool> {
    client.bucket_exists(bucket_name)
}

pub fn create_bucket(client: &MinioClient, bucket_name: &str) -> MinioResult<MinioOperationResult> {
    client.create_bucket(bucket_name)
}

pub fn ensure_bucket(client: &MinioClient, bucket_name: &str) -> MinioResult<MinioOperationResult> {
    client.ensure_bucket(bucket_name)
}

pub fn list_buckets(client: &MinioClient) -> MinioResult<Vec<String>> {
    client.list_buckets()
}

pub fn delete_bucket(
    client: &MinioClient,
    bucket_name: &str,
    force: bool,
) -> MinioResult<MinioOperationResult> {
    client.delete_bucket(bucket_name, force)
}

pub fn put_object(
    client: &MinioClient,
    bucket_name: &str,
    object_name: &str,
    data: &[u8],
    content_type: Option<&str>,
) -> MinioResult<MinioOperationResult> {
    client.put_object_bytes(bucket_name, object_name, data, content_type)
}

pub fn put_object_file(
    client: &MinioClient,
    bucket_name: &str,
    object_name: &str,
    file: &Path,
    content_type: Option<&str>,
) -> MinioResult<MinioOperationResult> {
    client.put_object_file(bucket_name, object_name, file, content_type)
}

pub fn put_object_reader<R: Read>(
    client: &MinioClient,
    bucket_name: &str,
    object_name: &str,
    reader: R,
    content_type: Option<&str>,
) -> MinioResult<MinioOperationResult> {
    client.put_object_reader(bucket_name, object_name, reader, content_type)
}

pub fn get_object(
    client: &MinioClient,
    bucket_name: &str,
    object_name: &str,
) -> MinioResult<Vec<u8>> {
    client.get_object(bucket_name, object_name)
}

pub fn get_object_to_file(
    client: &MinioClient,
    bucket_name: &str,
    object_name: &str,
    file: &Path,
) -> MinioResult<MinioOperationResult> {
    client.get_object_to_file(bucket_name, object_name, file)
}

pub fn stat_object(
    client: &MinioClient,
    bucket_name: &str,
    object_name: &str,
) -> MinioResult<Option<ObjectInfo>> {
    client.stat_object(bucket_name, object_name)
}

pub fn object_exists(
    client: &MinioClient,
    bucket_name: &str,
    object_name: &str,
) -> MinioResult<bool> {
    client.object_exists(bucket_name, object_name)
}

pub fn list_objects(
    client: &MinioClient,
    bucket_name: &str,
    prefix: Option<&str>,
    recursive: bool,
) -> MinioResult<Vec<ObjectInfo>> {
    client.list_objects(bucket_name, prefix, recursive)
}

pub fn delete_object(
    client: &MinioClient,
    bucket_name: &str,
    object_name: &str,
) -> MinioResult<MinioOperationResult> {
    client.delete_object(bucket_name, object_name)
}

pub fn delete_objects(
    client: &MinioClient,
    bucket_name: &str,
    object_names: &[String],
) -> MinioResult<MinioOperationResult> {
    client.delete_objects(bucket_name, object_names)
}

pub fn copy_object(
    client: &MinioClient,
    source_bucket: &str,
    source_object: &str,
    target_bucket: &str,
    target_object: &str,
) -> MinioResult<MinioOperationResult> {
    client.copy_object(source_bucket, source_object, target_bucket, target_object)
}

pub fn get_presigned_object_url(
    client: &MinioClient,
    bucket_name: &str,
    object_name: &str,
) -> MinioResult<String> {
    client.get_presigned_object_url(bucket_name, object_name)
}

pub fn get_presigned_object_url_with_expiration(
    client: &MinioClient,
    bucket_name: &str,
    object_name: &str,
    expiration_seconds: u64,
) -> MinioResult<PresignedUrl> {
    client.get_presigned_object_url_with_expiration(bucket_name, object_name, expiration_seconds)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::NamedTempFile;
    use tool_rustfs::InMemoryS3StorageClient;

    fn test_client() -> MinioClient {
        let config = MinioConfig::builder("http://localhost:9000", "minioadmin", "minioadmin")
            .region("us-east-1")
            .build()
            .expect("config should build");
        MinioClient::from_storage_client(config, Arc::new(InMemoryS3StorageClient::default()))
            .expect("client should build")
    }

    #[test]
    fn config_builder_matches_minio_defaults() {
        let config = MinioConfig::builder("http://localhost:9000", "minioadmin", "minioadmin")
            .build()
            .expect("config should build");

        assert_eq!(config.endpoint, "http://localhost:9000");
        assert_eq!(config.access_key, "minioadmin");
        assert_eq!(config.secret_key, "minioadmin");
        assert_eq!(config.region, None);
        assert!(config.path_style_access);
    }

    #[test]
    fn bucket_and_object_lifecycle_matches_jvm_api() {
        let client = test_client();
        ensure_bucket(&client, "demo").expect("bucket should be ensured");

        let payload = b"hello minio";
        put_object(&client, "demo", "hello.txt", payload, Some("text/plain"))
            .expect("object should upload");

        assert!(bucket_exists(&client, "demo").expect("bucket check should work"));
        assert!(object_exists(&client, "demo", "hello.txt").expect("object check should work"));
        assert_eq!(
            get_object(&client, "demo", "hello.txt").expect("download should work"),
            payload
        );

        let info = stat_object(&client, "demo", "hello.txt")
            .expect("stat should work")
            .expect("object should exist");
        assert_eq!(info.object_name, "hello.txt");
        assert_eq!(info.size, payload.len() as u64);
        assert_eq!(info.content_type.as_deref(), Some("text/plain"));

        copy_object(&client, "demo", "hello.txt", "demo", "copy.txt").expect("copy should work");
        delete_object(&client, "demo", "hello.txt").expect("delete should work");

        let objects = list_objects(&client, "demo", None, true).expect("list should work");
        assert_eq!(objects.len(), 1);
        assert_eq!(objects[0].object_name, "copy.txt");
    }

    #[test]
    fn file_upload_and_presigned_url_are_supported() {
        let client = test_client();
        ensure_bucket(&client, "demo").expect("bucket should be ensured");

        let file = NamedTempFile::new().expect("tempfile should exist");
        fs::write(file.path(), "{\"ok\":true}").expect("tempfile should be written");

        put_object_file(&client, "demo", "payload.json", file.path(), None)
            .expect("file upload should work");

        let info = stat_object(&client, "demo", "payload.json")
            .expect("stat should work")
            .expect("object should exist");
        assert_eq!(info.content_type.as_deref(), Some("application/json"));

        let download = NamedTempFile::new().expect("download tempfile should exist");
        get_object_to_file(&client, "demo", "payload.json", download.path())
            .expect("download should work");
        assert_eq!(
            fs::read_to_string(download.path()).expect("download should be readable"),
            "{\"ok\":true}"
        );

        let url = get_presigned_object_url(&client, "demo", "payload.json")
            .expect("presigned url should be created");
        assert!(url.contains("demo"));
        assert!(url.contains("payload.json"));
    }

    #[test]
    fn cached_clients_reuse_the_same_key() {
        let config = MinioConfig::new("http://localhost:9000", "minioadmin", "minioadmin");
        let first =
            get_or_create_client("default", config.clone()).expect("first client should build");
        let second = get_or_create_client("default", config).expect("second client should reuse");

        assert_eq!(first.config(), second.config());
    }
}
