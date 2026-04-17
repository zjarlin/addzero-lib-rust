use crate::progress::{InMemoryUploadProgressStorage, PartInfo};
use crate::types::{ObjectMetadata, PresignedUrl, S3ClientConfig};
use std::collections::{BTreeMap, HashMap};
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum StorageError {
    #[error("invalid storage configuration: {0}")]
    InvalidConfig(String),
    #[error("bucket `{bucket}` was not found")]
    BucketNotFound { bucket: String },
    #[error("object `{bucket}/{key}` was not found")]
    ObjectNotFound { bucket: String, key: String },
    #[error("storage backend error: {0}")]
    Backend(String),
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}

pub type StorageResult<T> = Result<T, StorageError>;

pub trait S3StorageClient: Send + Sync {
    fn bucket_exists(&self, bucket_name: &str) -> StorageResult<bool>;
    fn create_bucket(&self, bucket_name: &str) -> StorageResult<()>;
    fn list_buckets(&self) -> StorageResult<Vec<String>>;
    fn delete_bucket(&self, bucket_name: &str) -> StorageResult<()>;

    fn object_exists(&self, bucket_name: &str, key: &str) -> StorageResult<bool>;
    fn get_object_metadata(
        &self,
        bucket_name: &str,
        key: &str,
    ) -> StorageResult<Option<ObjectMetadata>>;
    fn put_object_bytes(
        &self,
        bucket_name: &str,
        key: &str,
        data: &[u8],
        content_type: Option<&str>,
        metadata: &BTreeMap<String, String>,
    ) -> StorageResult<()>;
    fn put_object_file(
        &self,
        bucket_name: &str,
        key: &str,
        path: &Path,
        content_type: Option<&str>,
        metadata: &BTreeMap<String, String>,
    ) -> StorageResult<()>;
    fn get_object(&self, bucket_name: &str, key: &str) -> StorageResult<Vec<u8>>;
    fn get_object_to_file(&self, bucket_name: &str, key: &str, target: &Path) -> StorageResult<()>;
    fn delete_object(&self, bucket_name: &str, key: &str) -> StorageResult<()>;
    fn delete_objects(&self, bucket_name: &str, keys: &[String]) -> StorageResult<()>;
    fn copy_object(
        &self,
        source_bucket: &str,
        source_key: &str,
        target_bucket: &str,
        target_key: &str,
    ) -> StorageResult<()>;
    fn list_objects(
        &self,
        bucket_name: &str,
        prefix: Option<&str>,
        recursive: bool,
        max_keys: usize,
    ) -> StorageResult<Vec<ObjectMetadata>>;

    fn init_multipart_upload(
        &self,
        bucket_name: &str,
        key: &str,
        content_type: Option<&str>,
        metadata: &BTreeMap<String, String>,
    ) -> StorageResult<String>;
    fn upload_part(
        &self,
        bucket_name: &str,
        key: &str,
        upload_id: &str,
        part_number: u32,
        data: &[u8],
        content_type: Option<&str>,
    ) -> StorageResult<String>;
    fn complete_multipart_upload(
        &self,
        bucket_name: &str,
        key: &str,
        upload_id: &str,
        parts: &[PartInfo],
    ) -> StorageResult<()>;
    fn abort_multipart_upload(
        &self,
        bucket_name: &str,
        key: &str,
        upload_id: &str,
    ) -> StorageResult<()>;
    fn list_multipart_uploads(&self, bucket_name: &str) -> StorageResult<Vec<String>>;

    fn generate_presigned_url(
        &self,
        bucket_name: &str,
        key: &str,
        expiration_seconds: u64,
    ) -> StorageResult<PresignedUrl>;
    fn generate_presigned_upload_url(
        &self,
        bucket_name: &str,
        key: &str,
        expiration_seconds: u64,
    ) -> StorageResult<PresignedUrl>;
}

pub trait S3StorageClientFactory: Send + Sync {
    fn create_client(&self, config: S3ClientConfig) -> Arc<dyn S3StorageClient>;
    fn create_default_client(&self) -> Arc<dyn S3StorageClient>;
}

#[derive(Clone)]
pub struct DefaultS3StorageClientFactory {
    default_config: Arc<dyn Fn() -> S3ClientConfig + Send + Sync>,
}

impl DefaultS3StorageClientFactory {
    pub fn new(default_config: impl Fn() -> S3ClientConfig + Send + Sync + 'static) -> Self {
        Self {
            default_config: Arc::new(default_config),
        }
    }
}

impl S3StorageClientFactory for DefaultS3StorageClientFactory {
    fn create_client(&self, config: S3ClientConfig) -> Arc<dyn S3StorageClient> {
        Arc::new(BlockingS3StorageClient::new(config))
    }

    fn create_default_client(&self) -> Arc<dyn S3StorageClient> {
        Arc::new(BlockingS3StorageClient::new((self.default_config)()))
    }
}

#[derive(Debug, Clone)]
pub struct BlockingS3StorageClient {
    config: S3ClientConfig,
}

impl BlockingS3StorageClient {
    pub fn new(config: S3ClientConfig) -> Self {
        Self { config }
    }

    pub fn config(&self) -> &S3ClientConfig {
        &self.config
    }

    fn backend_unavailable(&self, operation: &str) -> StorageError {
        StorageError::Backend(format!(
            "blocking backend unavailable for `{operation}` with endpoint `{}`; \
             tool-rustfs still targets the older rust-s3 API while the workspace resolves s3 0.1.27",
            self.config.endpoint
        ))
    }
}

impl S3StorageClient for BlockingS3StorageClient {
    fn bucket_exists(&self, _bucket_name: &str) -> StorageResult<bool> {
        Err(self.backend_unavailable("bucket_exists"))
    }

    fn create_bucket(&self, _bucket_name: &str) -> StorageResult<()> {
        Err(self.backend_unavailable("create_bucket"))
    }

    fn list_buckets(&self) -> StorageResult<Vec<String>> {
        Err(self.backend_unavailable("list_buckets"))
    }

    fn delete_bucket(&self, _bucket_name: &str) -> StorageResult<()> {
        Err(self.backend_unavailable("delete_bucket"))
    }

    fn object_exists(&self, _bucket_name: &str, _key: &str) -> StorageResult<bool> {
        Err(self.backend_unavailable("object_exists"))
    }

    fn get_object_metadata(
        &self,
        _bucket_name: &str,
        _key: &str,
    ) -> StorageResult<Option<ObjectMetadata>> {
        Err(self.backend_unavailable("get_object_metadata"))
    }

    fn put_object_bytes(
        &self,
        _bucket_name: &str,
        _key: &str,
        _data: &[u8],
        _content_type: Option<&str>,
        _metadata: &BTreeMap<String, String>,
    ) -> StorageResult<()> {
        Err(self.backend_unavailable("put_object_bytes"))
    }

    fn put_object_file(
        &self,
        _bucket_name: &str,
        _key: &str,
        _path: &Path,
        _content_type: Option<&str>,
        _metadata: &BTreeMap<String, String>,
    ) -> StorageResult<()> {
        Err(self.backend_unavailable("put_object_file"))
    }

    fn get_object(&self, _bucket_name: &str, _key: &str) -> StorageResult<Vec<u8>> {
        Err(self.backend_unavailable("get_object"))
    }

    fn get_object_to_file(
        &self,
        _bucket_name: &str,
        _key: &str,
        _target: &Path,
    ) -> StorageResult<()> {
        Err(self.backend_unavailable("get_object_to_file"))
    }

    fn delete_object(&self, _bucket_name: &str, _key: &str) -> StorageResult<()> {
        Err(self.backend_unavailable("delete_object"))
    }

    fn delete_objects(&self, _bucket_name: &str, _keys: &[String]) -> StorageResult<()> {
        Err(self.backend_unavailable("delete_objects"))
    }

    fn copy_object(
        &self,
        _source_bucket: &str,
        _source_key: &str,
        _target_bucket: &str,
        _target_key: &str,
    ) -> StorageResult<()> {
        Err(self.backend_unavailable("copy_object"))
    }

    fn list_objects(
        &self,
        _bucket_name: &str,
        _prefix: Option<&str>,
        _recursive: bool,
        _max_keys: usize,
    ) -> StorageResult<Vec<ObjectMetadata>> {
        Err(self.backend_unavailable("list_objects"))
    }

    fn init_multipart_upload(
        &self,
        _bucket_name: &str,
        _key: &str,
        _content_type: Option<&str>,
        _metadata: &BTreeMap<String, String>,
    ) -> StorageResult<String> {
        Err(self.backend_unavailable("init_multipart_upload"))
    }

    fn upload_part(
        &self,
        _bucket_name: &str,
        _key: &str,
        _upload_id: &str,
        _part_number: u32,
        _data: &[u8],
        _content_type: Option<&str>,
    ) -> StorageResult<String> {
        Err(self.backend_unavailable("upload_part"))
    }

    fn complete_multipart_upload(
        &self,
        _bucket_name: &str,
        _key: &str,
        _upload_id: &str,
        _parts: &[PartInfo],
    ) -> StorageResult<()> {
        Err(self.backend_unavailable("complete_multipart_upload"))
    }

    fn abort_multipart_upload(
        &self,
        _bucket_name: &str,
        _key: &str,
        _upload_id: &str,
    ) -> StorageResult<()> {
        Err(self.backend_unavailable("abort_multipart_upload"))
    }

    fn list_multipart_uploads(&self, _bucket_name: &str) -> StorageResult<Vec<String>> {
        Err(self.backend_unavailable("list_multipart_uploads"))
    }

    fn generate_presigned_url(
        &self,
        _bucket_name: &str,
        _key: &str,
        _expiration_seconds: u64,
    ) -> StorageResult<PresignedUrl> {
        Err(self.backend_unavailable("generate_presigned_url"))
    }

    fn generate_presigned_upload_url(
        &self,
        _bucket_name: &str,
        _key: &str,
        _expiration_seconds: u64,
    ) -> StorageResult<PresignedUrl> {
        Err(self.backend_unavailable("generate_presigned_upload_url"))
    }
}

#[derive(Debug, Default, Clone)]
pub struct InMemoryS3StorageClient {
    state: Arc<Mutex<MemoryState>>,
}

#[derive(Debug, Default)]
struct MemoryState {
    buckets: HashMap<String, HashMap<String, MemoryObject>>,
    uploads: HashMap<String, MemoryUpload>,
    next_id: u64,
}

#[derive(Debug, Clone)]
struct MemoryObject {
    bytes: Vec<u8>,
    content_type: Option<String>,
    metadata: BTreeMap<String, String>,
    etag: String,
    last_modified: String,
}

#[derive(Debug, Clone)]
struct MemoryUpload {
    bucket_name: String,
    _object_key: String,
    content_type: Option<String>,
    metadata: BTreeMap<String, String>,
    parts: HashMap<u32, Vec<u8>>,
}

impl InMemoryS3StorageClient {
    fn next_id(state: &mut MemoryState, prefix: &str) -> String {
        state.next_id += 1;
        format!("{prefix}-{}", state.next_id)
    }

    fn current_timestamp() -> String {
        InMemoryUploadProgressStorage::generate_upload_id_key("ts")
    }

    fn object_metadata(key: &str, object: &MemoryObject) -> ObjectMetadata {
        ObjectMetadata {
            key: key.to_owned(),
            size: object.bytes.len() as u64,
            etag: Some(object.etag.clone()),
            last_modified: Some(object.last_modified.clone()),
            content_type: object.content_type.clone(),
            metadata: object.metadata.clone(),
        }
    }
}

impl S3StorageClient for InMemoryS3StorageClient {
    fn bucket_exists(&self, bucket_name: &str) -> StorageResult<bool> {
        Ok(self
            .state
            .lock()
            .expect("in-memory storage mutex should not be poisoned")
            .buckets
            .contains_key(bucket_name))
    }

    fn create_bucket(&self, bucket_name: &str) -> StorageResult<()> {
        self.state
            .lock()
            .expect("in-memory storage mutex should not be poisoned")
            .buckets
            .entry(bucket_name.to_owned())
            .or_default();
        Ok(())
    }

    fn list_buckets(&self) -> StorageResult<Vec<String>> {
        Ok(self
            .state
            .lock()
            .expect("in-memory storage mutex should not be poisoned")
            .buckets
            .keys()
            .cloned()
            .collect())
    }

    fn delete_bucket(&self, bucket_name: &str) -> StorageResult<()> {
        self.state
            .lock()
            .expect("in-memory storage mutex should not be poisoned")
            .buckets
            .remove(bucket_name)
            .ok_or_else(|| StorageError::BucketNotFound {
                bucket: bucket_name.to_owned(),
            })?;
        Ok(())
    }

    fn object_exists(&self, bucket_name: &str, key: &str) -> StorageResult<bool> {
        Ok(self
            .state
            .lock()
            .expect("in-memory storage mutex should not be poisoned")
            .buckets
            .get(bucket_name)
            .and_then(|bucket| bucket.get(key))
            .is_some())
    }

    fn get_object_metadata(
        &self,
        bucket_name: &str,
        key: &str,
    ) -> StorageResult<Option<ObjectMetadata>> {
        Ok(self
            .state
            .lock()
            .expect("in-memory storage mutex should not be poisoned")
            .buckets
            .get(bucket_name)
            .and_then(|bucket| bucket.get(key))
            .map(|object| Self::object_metadata(key, object)))
    }

    fn put_object_bytes(
        &self,
        bucket_name: &str,
        key: &str,
        data: &[u8],
        content_type: Option<&str>,
        metadata: &BTreeMap<String, String>,
    ) -> StorageResult<()> {
        let mut state = self
            .state
            .lock()
            .expect("in-memory storage mutex should not be poisoned");
        let etag = Self::next_id(&mut state, "etag");
        state
            .buckets
            .entry(bucket_name.to_owned())
            .or_default()
            .insert(
                key.to_owned(),
                MemoryObject {
                    bytes: data.to_vec(),
                    content_type: content_type.map(ToOwned::to_owned),
                    metadata: metadata.clone(),
                    etag,
                    last_modified: Self::current_timestamp(),
                },
            );
        Ok(())
    }

    fn put_object_file(
        &self,
        bucket_name: &str,
        key: &str,
        path: &Path,
        content_type: Option<&str>,
        metadata: &BTreeMap<String, String>,
    ) -> StorageResult<()> {
        let bytes = std::fs::read(path)?;
        self.put_object_bytes(bucket_name, key, &bytes, content_type, metadata)
    }

    fn get_object(&self, bucket_name: &str, key: &str) -> StorageResult<Vec<u8>> {
        self.state
            .lock()
            .expect("in-memory storage mutex should not be poisoned")
            .buckets
            .get(bucket_name)
            .and_then(|bucket| bucket.get(key))
            .map(|object| object.bytes.clone())
            .ok_or_else(|| StorageError::ObjectNotFound {
                bucket: bucket_name.to_owned(),
                key: key.to_owned(),
            })
    }

    fn get_object_to_file(&self, bucket_name: &str, key: &str, target: &Path) -> StorageResult<()> {
        let bytes = self.get_object(bucket_name, key)?;
        if let Some(parent) = target.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(target, bytes)?;
        Ok(())
    }

    fn delete_object(&self, bucket_name: &str, key: &str) -> StorageResult<()> {
        self.state
            .lock()
            .expect("in-memory storage mutex should not be poisoned")
            .buckets
            .get_mut(bucket_name)
            .and_then(|bucket| bucket.remove(key))
            .ok_or_else(|| StorageError::ObjectNotFound {
                bucket: bucket_name.to_owned(),
                key: key.to_owned(),
            })?;
        Ok(())
    }

    fn delete_objects(&self, bucket_name: &str, keys: &[String]) -> StorageResult<()> {
        for key in keys {
            let _ = self.delete_object(bucket_name, key);
        }
        Ok(())
    }

    fn copy_object(
        &self,
        source_bucket: &str,
        source_key: &str,
        target_bucket: &str,
        target_key: &str,
    ) -> StorageResult<()> {
        let metadata = self.get_object_metadata(source_bucket, source_key)?;
        let bytes = self.get_object(source_bucket, source_key)?;
        let content_type = metadata.as_ref().and_then(|meta| meta.content_type.clone());
        let custom_metadata = metadata.map(|meta| meta.metadata).unwrap_or_default();
        self.put_object_bytes(
            target_bucket,
            target_key,
            &bytes,
            content_type.as_deref(),
            &custom_metadata,
        )
    }

    fn list_objects(
        &self,
        bucket_name: &str,
        prefix: Option<&str>,
        recursive: bool,
        max_keys: usize,
    ) -> StorageResult<Vec<ObjectMetadata>> {
        let prefix = prefix.unwrap_or_default();
        let bucket = self
            .state
            .lock()
            .expect("in-memory storage mutex should not be poisoned")
            .buckets
            .get(bucket_name)
            .cloned()
            .unwrap_or_default();

        Ok(bucket
            .into_iter()
            .filter(|(key, _)| key.starts_with(prefix))
            .filter(|(key, _)| {
                recursive || !key[prefix.len()..].trim_start_matches('/').contains('/')
            })
            .take(max_keys)
            .map(|(key, object)| Self::object_metadata(&key, &object))
            .collect())
    }

    fn init_multipart_upload(
        &self,
        bucket_name: &str,
        key: &str,
        content_type: Option<&str>,
        metadata: &BTreeMap<String, String>,
    ) -> StorageResult<String> {
        let mut state = self
            .state
            .lock()
            .expect("in-memory storage mutex should not be poisoned");
        let upload_id = Self::next_id(&mut state, "upload");
        state.uploads.insert(
            upload_id.clone(),
            MemoryUpload {
                bucket_name: bucket_name.to_owned(),
                _object_key: key.to_owned(),
                content_type: content_type.map(ToOwned::to_owned),
                metadata: metadata.clone(),
                parts: HashMap::new(),
            },
        );
        Ok(upload_id)
    }

    fn upload_part(
        &self,
        _bucket_name: &str,
        _key: &str,
        upload_id: &str,
        part_number: u32,
        data: &[u8],
        _content_type: Option<&str>,
    ) -> StorageResult<String> {
        let mut state = self
            .state
            .lock()
            .expect("in-memory storage mutex should not be poisoned");
        let upload = state
            .uploads
            .get_mut(upload_id)
            .ok_or_else(|| StorageError::Backend(format!("unknown upload id `{upload_id}`")))?;
        upload.parts.insert(part_number, data.to_vec());
        Ok(format!("etag-{upload_id}-{part_number}"))
    }

    fn complete_multipart_upload(
        &self,
        bucket_name: &str,
        key: &str,
        upload_id: &str,
        parts: &[PartInfo],
    ) -> StorageResult<()> {
        let mut state = self
            .state
            .lock()
            .expect("in-memory storage mutex should not be poisoned");
        let upload = state
            .uploads
            .remove(upload_id)
            .ok_or_else(|| StorageError::Backend(format!("unknown upload id `{upload_id}`")))?;

        let mut ordered = parts.to_vec();
        ordered.sort_by_key(|part| part.part_number);
        let bytes = ordered
            .into_iter()
            .filter_map(|part| upload.parts.get(&part.part_number).cloned())
            .flatten()
            .collect::<Vec<_>>();

        let etag = Self::next_id(&mut state, "etag");
        state
            .buckets
            .entry(bucket_name.to_owned())
            .or_default()
            .insert(
                key.to_owned(),
                MemoryObject {
                    bytes,
                    content_type: upload.content_type,
                    metadata: upload.metadata,
                    etag,
                    last_modified: Self::current_timestamp(),
                },
            );
        Ok(())
    }

    fn abort_multipart_upload(
        &self,
        _bucket_name: &str,
        _key: &str,
        upload_id: &str,
    ) -> StorageResult<()> {
        self.state
            .lock()
            .expect("in-memory storage mutex should not be poisoned")
            .uploads
            .remove(upload_id);
        Ok(())
    }

    fn list_multipart_uploads(&self, bucket_name: &str) -> StorageResult<Vec<String>> {
        Ok(self
            .state
            .lock()
            .expect("in-memory storage mutex should not be poisoned")
            .uploads
            .iter()
            .filter(|(_, upload)| upload.bucket_name == bucket_name)
            .map(|(upload_id, _)| upload_id.clone())
            .collect())
    }

    fn generate_presigned_url(
        &self,
        bucket_name: &str,
        key: &str,
        expiration_seconds: u64,
    ) -> StorageResult<PresignedUrl> {
        Ok(PresignedUrl {
            url: format!("memory://{bucket_name}/{key}?op=get"),
            expiration: SystemTime::now() + Duration::from_secs(expiration_seconds),
        })
    }

    fn generate_presigned_upload_url(
        &self,
        bucket_name: &str,
        key: &str,
        expiration_seconds: u64,
    ) -> StorageResult<PresignedUrl> {
        Ok(PresignedUrl {
            url: format!("memory://{bucket_name}/{key}?op=put"),
            expiration: SystemTime::now() + Duration::from_secs(expiration_seconds),
        })
    }
}
