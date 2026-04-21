use crate::progress::{InMemoryUploadProgressStorage, PartInfo};
use crate::types::{ObjectMetadata, PresignedUrl, S3ClientConfig};
use base64::Engine as _;
use chrono::Utc;
use hmac::{Hmac, Mac};
use quick_xml::Reader;
use quick_xml::events::Event;
use reqwest::blocking::{Client, Response};
use reqwest::header::{CONTENT_LENGTH, CONTENT_TYPE, ETAG, LAST_MODIFIED};
use reqwest::{Method, StatusCode, Url};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, HashMap};
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime};
use thiserror::Error;

type HmacSha256 = Hmac<Sha256>;

const AWS_SERVICE_NAME: &str = "s3";
const UNSIGNED_PAYLOAD: &str = "UNSIGNED-PAYLOAD";

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
    http: Client,
}

#[derive(Debug, Clone)]
struct RequestTarget {
    url: Url,
    canonical_uri: String,
    canonical_query: String,
    host_header: String,
}

#[derive(Debug, Default)]
struct PendingObjectSummary {
    key: String,
    size: u64,
    etag: Option<String>,
    last_modified: Option<String>,
}

impl BlockingS3StorageClient {
    pub fn new(config: S3ClientConfig) -> Self {
        let http = match Client::builder().build() {
            Ok(client) => client,
            Err(_) => Client::new(),
        };
        Self { config, http }
    }

    pub fn config(&self) -> &S3ClientConfig {
        &self.config
    }

    fn endpoint_url(&self) -> StorageResult<Url> {
        Url::parse(&self.config.endpoint).map_err(|error| {
            StorageError::InvalidConfig(format!(
                "invalid endpoint `{}`: {error}",
                self.config.endpoint
            ))
        })
    }

    fn normalized_region(&self) -> &str {
        let trimmed = self.config.region.trim();
        if trimmed.is_empty() {
            "us-east-1"
        } else {
            trimmed
        }
    }

    fn build_request_target(
        &self,
        bucket_name: Option<&str>,
        key: Option<&str>,
        query_pairs: &[(String, String)],
    ) -> StorageResult<RequestTarget> {
        let endpoint = self.endpoint_url()?;
        let scheme = endpoint.scheme();
        let endpoint_host = endpoint.host_str().ok_or_else(|| {
            StorageError::InvalidConfig(format!(
                "endpoint `{}` does not contain a host",
                self.config.endpoint
            ))
        })?;

        let mut host = endpoint_host.to_owned();
        let base_segments = endpoint
            .path()
            .trim_matches('/')
            .split('/')
            .filter(|segment| !segment.is_empty())
            .map(ToOwned::to_owned)
            .collect::<Vec<_>>();

        let mut path_segments = base_segments;
        if self.config.path_style_access {
            if let Some(bucket_name) = bucket_name {
                path_segments.push(bucket_name.to_owned());
            }
        } else if let Some(bucket_name) = bucket_name {
            host = format!("{bucket_name}.{endpoint_host}");
        }

        let mut canonical_uri = if path_segments.is_empty() {
            "/".to_owned()
        } else {
            format!(
                "/{}",
                path_segments
                    .iter()
                    .map(|segment| aws_percent_encode(segment, true))
                    .collect::<Vec<_>>()
                    .join("/")
            )
        };
        if let Some(key) = key {
            if !canonical_uri.ends_with('/') {
                canonical_uri.push('/');
            }
            canonical_uri.push_str(&aws_percent_encode(key.trim_start_matches('/'), false));
        }

        let canonical_query = canonical_query_string(query_pairs);
        let port = endpoint
            .port()
            .map(|value| format!(":{value}"))
            .unwrap_or_default();
        let mut raw_url = format!("{scheme}://{host}{port}{canonical_uri}");
        if !canonical_query.is_empty() {
            raw_url.push('?');
            raw_url.push_str(&canonical_query);
        }

        let url = Url::parse(&raw_url).map_err(|error| {
            StorageError::InvalidConfig(format!("failed to build request URL `{raw_url}`: {error}"))
        })?;

        Ok(RequestTarget {
            host_header: build_host_header(&url)?,
            url,
            canonical_uri,
            canonical_query,
        })
    }

    fn execute_signed_request(
        &self,
        method: Method,
        bucket_name: Option<&str>,
        key: Option<&str>,
        query_pairs: Vec<(String, String)>,
        extra_headers: BTreeMap<String, String>,
        body: Vec<u8>,
    ) -> StorageResult<Response> {
        let request_target = self.build_request_target(bucket_name, key, &query_pairs)?;
        let now = Utc::now();
        let amz_date = now.format("%Y%m%dT%H%M%SZ").to_string();
        let date_stamp = now.format("%Y%m%d").to_string();
        let payload_hash = sha256_hex(&body);
        let mut headers = BTreeMap::from([
            ("host".to_owned(), request_target.host_header.clone()),
            ("x-amz-content-sha256".to_owned(), payload_hash.clone()),
            ("x-amz-date".to_owned(), amz_date.clone()),
        ]);
        for (name, value) in extra_headers {
            if !value.trim().is_empty() {
                headers.insert(name.to_ascii_lowercase(), value);
            }
        }

        let (canonical_headers, signed_headers) = build_canonical_headers(&headers);
        let method_name = method.as_str().to_owned();
        let credential_scope = format!(
            "{}/{}/{}/aws4_request",
            date_stamp,
            self.normalized_region(),
            AWS_SERVICE_NAME
        );
        let canonical_request = format!(
            "{method_name}\n{}\n{}\n{}\n{}\n{}",
            request_target.canonical_uri,
            request_target.canonical_query,
            canonical_headers,
            signed_headers,
            payload_hash
        );
        let string_to_sign = format!(
            "AWS4-HMAC-SHA256\n{amz_date}\n{credential_scope}\n{}",
            sha256_hex(canonical_request.as_bytes())
        );
        let signing_key = derive_signing_key(
            &self.config.secret_key,
            &date_stamp,
            self.normalized_region(),
            AWS_SERVICE_NAME,
        )?;
        let signature = hex_lower(&sign_hmac(&signing_key, string_to_sign.as_bytes())?);
        let authorization = format!(
            "AWS4-HMAC-SHA256 Credential={}/{credential_scope}, SignedHeaders={signed_headers}, Signature={signature}",
            self.config.access_key
        );

        let mut request = self.http.request(method, request_target.url);
        for (name, value) in headers {
            request = request.header(name, value);
        }
        request
            .header("Authorization", authorization)
            .body(body)
            .send()
            .map_err(|error| StorageError::Backend(format!("request failed: {error}")))
    }

    fn execute_empty_body_request(
        &self,
        method: Method,
        bucket_name: Option<&str>,
        key: Option<&str>,
        query_pairs: Vec<(String, String)>,
        headers: BTreeMap<String, String>,
    ) -> StorageResult<Response> {
        self.execute_signed_request(method, bucket_name, key, query_pairs, headers, Vec::new())
    }

    fn ensure_success(
        &self,
        response: Response,
        bucket_name: Option<&str>,
        key: Option<&str>,
    ) -> StorageResult<Response> {
        if response.status().is_success() {
            return Ok(response);
        }
        Err(response_to_storage_error(response, bucket_name, key))
    }

    fn presigned_url(
        &self,
        method: &str,
        bucket_name: &str,
        key: &str,
        expiration_seconds: u64,
    ) -> StorageResult<PresignedUrl> {
        if expiration_seconds == 0 {
            return Err(StorageError::InvalidConfig(
                "presigned URL expiration must be greater than zero".to_owned(),
            ));
        }
        if expiration_seconds > 7 * 24 * 60 * 60 {
            return Err(StorageError::InvalidConfig(
                "presigned URL expiration cannot exceed 7 days".to_owned(),
            ));
        }

        let now = Utc::now();
        let amz_date = now.format("%Y%m%dT%H%M%SZ").to_string();
        let date_stamp = now.format("%Y%m%d").to_string();
        let credential_scope = format!(
            "{}/{}/{}/aws4_request",
            date_stamp,
            self.normalized_region(),
            AWS_SERVICE_NAME
        );
        let signed_headers = "host".to_owned();
        let mut query_pairs = vec![
            ("X-Amz-Algorithm".to_owned(), "AWS4-HMAC-SHA256".to_owned()),
            (
                "X-Amz-Credential".to_owned(),
                format!("{}/{}", self.config.access_key, credential_scope),
            ),
            ("X-Amz-Date".to_owned(), amz_date.clone()),
            ("X-Amz-Expires".to_owned(), expiration_seconds.to_string()),
            ("X-Amz-SignedHeaders".to_owned(), signed_headers.clone()),
        ];
        let request_target =
            self.build_request_target(Some(bucket_name), Some(key), &query_pairs)?;
        let canonical_request = format!(
            "{method}\n{}\n{}\nhost:{}\n\n{signed_headers}\n{UNSIGNED_PAYLOAD}",
            request_target.canonical_uri,
            request_target.canonical_query,
            request_target.host_header
        );
        let string_to_sign = format!(
            "AWS4-HMAC-SHA256\n{amz_date}\n{credential_scope}\n{}",
            sha256_hex(canonical_request.as_bytes())
        );
        let signing_key = derive_signing_key(
            &self.config.secret_key,
            &date_stamp,
            self.normalized_region(),
            AWS_SERVICE_NAME,
        )?;
        let signature = hex_lower(&sign_hmac(&signing_key, string_to_sign.as_bytes())?);
        query_pairs.push(("X-Amz-Signature".to_owned(), signature));

        Ok(PresignedUrl {
            url: self
                .build_request_target(Some(bucket_name), Some(key), &query_pairs)?
                .url
                .to_string(),
            expiration: SystemTime::now() + Duration::from_secs(expiration_seconds),
        })
    }
}

impl S3StorageClient for BlockingS3StorageClient {
    fn bucket_exists(&self, bucket_name: &str) -> StorageResult<bool> {
        let response = self.execute_empty_body_request(
            Method::HEAD,
            Some(bucket_name),
            None,
            Vec::new(),
            BTreeMap::new(),
        )?;
        Ok(match response.status() {
            StatusCode::OK | StatusCode::NO_CONTENT => true,
            StatusCode::NOT_FOUND => false,
            _ => {
                return Err(response_to_storage_error(response, Some(bucket_name), None));
            }
        })
    }

    fn create_bucket(&self, bucket_name: &str) -> StorageResult<()> {
        let body = if self.normalized_region() == "us-east-1" {
            Vec::new()
        } else {
            format!(
                "<CreateBucketConfiguration xmlns=\"http://s3.amazonaws.com/doc/2006-03-01/\"><LocationConstraint>{}</LocationConstraint></CreateBucketConfiguration>",
                escape_xml(self.normalized_region())
            )
            .into_bytes()
        };
        let mut headers = BTreeMap::new();
        if !body.is_empty() {
            headers.insert(
                CONTENT_TYPE.as_str().to_owned(),
                "application/xml".to_owned(),
            );
        }
        let response = self.execute_signed_request(
            Method::PUT,
            Some(bucket_name),
            None,
            Vec::new(),
            headers,
            body,
        )?;
        self.ensure_success(response, Some(bucket_name), None)?;
        Ok(())
    }

    fn list_buckets(&self) -> StorageResult<Vec<String>> {
        let response =
            self.execute_empty_body_request(Method::GET, None, None, Vec::new(), BTreeMap::new())?;
        let body = response_to_text(self.ensure_success(response, None, None)?)?;
        collect_path_texts(
            &body,
            &["ListAllMyBucketsResult", "Buckets", "Bucket", "Name"],
        )
    }

    fn delete_bucket(&self, bucket_name: &str) -> StorageResult<()> {
        let response = self.execute_empty_body_request(
            Method::DELETE,
            Some(bucket_name),
            None,
            Vec::new(),
            BTreeMap::new(),
        )?;
        self.ensure_success(response, Some(bucket_name), None)?;
        Ok(())
    }

    fn object_exists(&self, bucket_name: &str, key: &str) -> StorageResult<bool> {
        let response = self.execute_empty_body_request(
            Method::HEAD,
            Some(bucket_name),
            Some(key),
            Vec::new(),
            BTreeMap::new(),
        )?;
        Ok(match response.status() {
            StatusCode::OK => true,
            StatusCode::NOT_FOUND => false,
            _ => {
                return Err(response_to_storage_error(
                    response,
                    Some(bucket_name),
                    Some(key),
                ));
            }
        })
    }

    fn get_object_metadata(
        &self,
        bucket_name: &str,
        key: &str,
    ) -> StorageResult<Option<ObjectMetadata>> {
        let response = self.execute_empty_body_request(
            Method::HEAD,
            Some(bucket_name),
            Some(key),
            Vec::new(),
            BTreeMap::new(),
        )?;
        Ok(match response.status() {
            StatusCode::OK => Some(metadata_from_headers(key, response.headers())),
            StatusCode::NOT_FOUND => None,
            _ => {
                return Err(response_to_storage_error(
                    response,
                    Some(bucket_name),
                    Some(key),
                ));
            }
        })
    }

    fn put_object_bytes(
        &self,
        bucket_name: &str,
        key: &str,
        data: &[u8],
        content_type: Option<&str>,
        metadata: &BTreeMap<String, String>,
    ) -> StorageResult<()> {
        let mut headers = metadata_headers(metadata);
        if let Some(content_type) = content_type.filter(|value| !value.trim().is_empty()) {
            headers.insert(CONTENT_TYPE.as_str().to_owned(), content_type.to_owned());
        }
        let response = self.execute_signed_request(
            Method::PUT,
            Some(bucket_name),
            Some(key),
            Vec::new(),
            headers,
            data.to_vec(),
        )?;
        self.ensure_success(response, Some(bucket_name), Some(key))?;
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
        let response = self.execute_empty_body_request(
            Method::GET,
            Some(bucket_name),
            Some(key),
            Vec::new(),
            BTreeMap::new(),
        )?;
        let response = self.ensure_success(response, Some(bucket_name), Some(key))?;
        response
            .bytes()
            .map(|bytes| bytes.to_vec())
            .map_err(|error| {
                StorageError::Backend(format!("failed to read response body: {error}"))
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
        let response = self.execute_empty_body_request(
            Method::DELETE,
            Some(bucket_name),
            Some(key),
            Vec::new(),
            BTreeMap::new(),
        )?;
        self.ensure_success(response, Some(bucket_name), Some(key))?;
        Ok(())
    }

    fn delete_objects(&self, bucket_name: &str, keys: &[String]) -> StorageResult<()> {
        if keys.is_empty() {
            return Ok(());
        }
        let body = build_delete_objects_body(keys);
        let headers = BTreeMap::from([
            (
                CONTENT_TYPE.as_str().to_owned(),
                "application/xml".to_owned(),
            ),
            (
                "content-md5".to_owned(),
                base64::engine::general_purpose::STANDARD.encode(md5::compute(&body).0),
            ),
        ]);
        let response = self.execute_signed_request(
            Method::POST,
            Some(bucket_name),
            None,
            vec![("delete".to_owned(), String::new())],
            headers,
            body.into_bytes(),
        )?;
        self.ensure_success(response, Some(bucket_name), None)?;
        Ok(())
    }

    fn copy_object(
        &self,
        source_bucket: &str,
        source_key: &str,
        target_bucket: &str,
        target_key: &str,
    ) -> StorageResult<()> {
        let response = self.execute_empty_body_request(
            Method::PUT,
            Some(target_bucket),
            Some(target_key),
            Vec::new(),
            BTreeMap::from([(
                "x-amz-copy-source".to_owned(),
                format!(
                    "/{}/{}",
                    aws_percent_encode(source_bucket, true),
                    aws_percent_encode(source_key, false)
                ),
            )]),
        )?;
        self.ensure_success(response, Some(target_bucket), Some(target_key))?;
        Ok(())
    }

    fn list_objects(
        &self,
        bucket_name: &str,
        prefix: Option<&str>,
        recursive: bool,
        max_keys: usize,
    ) -> StorageResult<Vec<ObjectMetadata>> {
        let mut query_pairs = vec![
            ("list-type".to_owned(), "2".to_owned()),
            ("max-keys".to_owned(), max_keys.to_string()),
        ];
        if let Some(prefix) = prefix.filter(|value| !value.is_empty()) {
            query_pairs.push(("prefix".to_owned(), prefix.to_owned()));
        }
        if !recursive {
            query_pairs.push(("delimiter".to_owned(), "/".to_owned()));
        }
        let response = self.execute_empty_body_request(
            Method::GET,
            Some(bucket_name),
            None,
            query_pairs,
            BTreeMap::new(),
        )?;
        if response.status() == StatusCode::NOT_FOUND {
            return Ok(Vec::new());
        }
        let body = response_to_text(self.ensure_success(response, Some(bucket_name), None)?)?;
        parse_list_objects_response(&body)
    }

    fn init_multipart_upload(
        &self,
        bucket_name: &str,
        key: &str,
        content_type: Option<&str>,
        metadata: &BTreeMap<String, String>,
    ) -> StorageResult<String> {
        let mut headers = metadata_headers(metadata);
        if let Some(content_type) = content_type.filter(|value| !value.trim().is_empty()) {
            headers.insert(CONTENT_TYPE.as_str().to_owned(), content_type.to_owned());
        }
        let response = self.execute_signed_request(
            Method::POST,
            Some(bucket_name),
            Some(key),
            vec![("uploads".to_owned(), String::new())],
            headers,
            Vec::new(),
        )?;
        let body =
            response_to_text(self.ensure_success(response, Some(bucket_name), Some(key))?)?;
        collect_first_path_text(&body, &["InitiateMultipartUploadResult", "UploadId"])?.ok_or_else(
            || StorageError::Backend("multipart upload response missing UploadId".to_owned()),
        )
    }

    fn upload_part(
        &self,
        bucket_name: &str,
        key: &str,
        upload_id: &str,
        part_number: u32,
        data: &[u8],
        content_type: Option<&str>,
    ) -> StorageResult<String> {
        let mut headers = BTreeMap::new();
        if let Some(content_type) = content_type.filter(|value| !value.trim().is_empty()) {
            headers.insert(CONTENT_TYPE.as_str().to_owned(), content_type.to_owned());
        }
        let response = self.execute_signed_request(
            Method::PUT,
            Some(bucket_name),
            Some(key),
            vec![
                ("partNumber".to_owned(), part_number.to_string()),
                ("uploadId".to_owned(), upload_id.to_owned()),
            ],
            headers,
            data.to_vec(),
        )?;
        let response = self.ensure_success(response, Some(bucket_name), Some(key))?;
        Ok(response
            .headers()
            .get(ETAG)
            .and_then(|value| value.to_str().ok())
            .map(trim_surrounding_quotes)
            .unwrap_or_default())
    }

    fn complete_multipart_upload(
        &self,
        bucket_name: &str,
        key: &str,
        upload_id: &str,
        parts: &[PartInfo],
    ) -> StorageResult<()> {
        let body = build_complete_multipart_body(parts);
        let response = self.execute_signed_request(
            Method::POST,
            Some(bucket_name),
            Some(key),
            vec![("uploadId".to_owned(), upload_id.to_owned())],
            BTreeMap::from([(
                CONTENT_TYPE.as_str().to_owned(),
                "application/xml".to_owned(),
            )]),
            body.into_bytes(),
        )?;
        self.ensure_success(response, Some(bucket_name), Some(key))?;
        Ok(())
    }

    fn abort_multipart_upload(
        &self,
        bucket_name: &str,
        key: &str,
        upload_id: &str,
    ) -> StorageResult<()> {
        let response = self.execute_empty_body_request(
            Method::DELETE,
            Some(bucket_name),
            Some(key),
            vec![("uploadId".to_owned(), upload_id.to_owned())],
            BTreeMap::new(),
        )?;
        self.ensure_success(response, Some(bucket_name), Some(key))?;
        Ok(())
    }

    fn list_multipart_uploads(&self, bucket_name: &str) -> StorageResult<Vec<String>> {
        let response = self.execute_empty_body_request(
            Method::GET,
            Some(bucket_name),
            None,
            vec![("uploads".to_owned(), String::new())],
            BTreeMap::new(),
        )?;
        if response.status() == StatusCode::NOT_FOUND {
            return Ok(Vec::new());
        }
        let body = response_to_text(self.ensure_success(response, Some(bucket_name), None)?)?;
        collect_path_texts(&body, &["ListMultipartUploadsResult", "Upload", "UploadId"])
    }

    fn generate_presigned_url(
        &self,
        bucket_name: &str,
        key: &str,
        expiration_seconds: u64,
    ) -> StorageResult<PresignedUrl> {
        self.presigned_url("GET", bucket_name, key, expiration_seconds)
    }

    fn generate_presigned_upload_url(
        &self,
        bucket_name: &str,
        key: &str,
        expiration_seconds: u64,
    ) -> StorageResult<PresignedUrl> {
        self.presigned_url("PUT", bucket_name, key, expiration_seconds)
    }
}

fn metadata_headers(metadata: &BTreeMap<String, String>) -> BTreeMap<String, String> {
    metadata
        .iter()
        .map(|(key, value)| {
            (
                format!("x-amz-meta-{}", key.to_ascii_lowercase()),
                value.clone(),
            )
        })
        .collect()
}

fn build_host_header(url: &Url) -> StorageResult<String> {
    let host = url.host_str().ok_or_else(|| {
        StorageError::InvalidConfig(format!("request URL `{url}` does not contain a host"))
    })?;
    let include_port = match (url.scheme(), url.port()) {
        ("http", Some(80)) | ("https", Some(443)) | (_, None) => false,
        (_, Some(_)) => true,
    };
    Ok(if include_port {
        format!("{host}:{}", url.port().unwrap_or_default())
    } else {
        host.to_owned()
    })
}

fn response_to_text(response: Response) -> StorageResult<String> {
    response
        .bytes()
        .map(|bytes| String::from_utf8_lossy(bytes.as_ref()).into_owned())
        .map_err(|error| StorageError::Backend(format!("failed to read response body: {error}")))
}

fn response_to_storage_error(
    response: Response,
    bucket_name: Option<&str>,
    key: Option<&str>,
) -> StorageError {
    let status = response.status();
    let body = match response.bytes() {
        Ok(bytes) => String::from_utf8_lossy(bytes.as_ref()).into_owned(),
        Err(error) => {
            return StorageError::Backend(format!(
                "S3 request failed with HTTP {} and body read error: {error}",
                status.as_u16()
            ));
        }
    };
    let error_code = collect_first_local_name_text(&body, "Code").ok().flatten();
    let error_message = collect_first_local_name_text(&body, "Message")
        .ok()
        .flatten();

    match (status, error_code.as_deref(), bucket_name, key) {
        (StatusCode::NOT_FOUND, Some("NoSuchBucket"), Some(bucket_name), _) => {
            StorageError::BucketNotFound {
                bucket: bucket_name.to_owned(),
            }
        }
        (StatusCode::NOT_FOUND, Some("NoSuchKey"), Some(bucket_name), Some(key)) => {
            StorageError::ObjectNotFound {
                bucket: bucket_name.to_owned(),
                key: key.to_owned(),
            }
        }
        (StatusCode::NOT_FOUND, _, Some(bucket_name), Some(key)) => StorageError::ObjectNotFound {
            bucket: bucket_name.to_owned(),
            key: key.to_owned(),
        },
        (StatusCode::NOT_FOUND, _, Some(bucket_name), None) => StorageError::BucketNotFound {
            bucket: bucket_name.to_owned(),
        },
        _ => StorageError::Backend(format!(
            "S3 request failed with HTTP {}{}{}",
            status.as_u16(),
            error_code
                .as_ref()
                .map(|value| format!(", code={value}"))
                .unwrap_or_default(),
            error_message
                .as_ref()
                .map(|value| format!(", message={value}"))
                .unwrap_or_else(|| {
                    if body.trim().is_empty() {
                        String::new()
                    } else {
                        format!(", body={body}")
                    }
                })
        )),
    }
}

fn metadata_from_headers(key: &str, headers: &reqwest::header::HeaderMap) -> ObjectMetadata {
    let metadata = headers
        .iter()
        .filter_map(|(name, value)| {
            name.as_str()
                .strip_prefix("x-amz-meta-")
                .and_then(|suffix| {
                    value
                        .to_str()
                        .ok()
                        .map(|text| (suffix.to_owned(), text.to_owned()))
                })
        })
        .collect();
    ObjectMetadata {
        key: key.to_owned(),
        size: headers
            .get(CONTENT_LENGTH)
            .and_then(|value| value.to_str().ok())
            .and_then(|value| value.parse::<u64>().ok())
            .unwrap_or_default(),
        etag: headers
            .get(ETAG)
            .and_then(|value| value.to_str().ok())
            .map(trim_surrounding_quotes),
        last_modified: headers
            .get(LAST_MODIFIED)
            .and_then(|value| value.to_str().ok())
            .map(ToOwned::to_owned),
        content_type: headers
            .get(CONTENT_TYPE)
            .and_then(|value| value.to_str().ok())
            .map(ToOwned::to_owned),
        metadata,
    }
}

fn build_delete_objects_body(keys: &[String]) -> String {
    let objects = keys
        .iter()
        .map(|key| format!("<Object><Key>{}</Key></Object>", escape_xml(key)))
        .collect::<Vec<_>>()
        .join("");
    format!("<Delete>{objects}</Delete>")
}

fn build_complete_multipart_body(parts: &[PartInfo]) -> String {
    let serialized = parts
        .iter()
        .map(|part| {
            format!(
                "<Part><PartNumber>{}</PartNumber><ETag>{}</ETag></Part>",
                part.part_number,
                escape_xml(&quoted_etag(part.etag.as_deref().unwrap_or_default()))
            )
        })
        .collect::<Vec<_>>()
        .join("");
    format!("<CompleteMultipartUpload>{serialized}</CompleteMultipartUpload>")
}

fn parse_list_objects_response(xml: &str) -> StorageResult<Vec<ObjectMetadata>> {
    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(true);
    let mut buffer = Vec::new();
    let mut path = Vec::<String>::new();
    let mut current = None::<PendingObjectSummary>;
    let mut objects = Vec::new();

    loop {
        match reader.read_event_into(&mut buffer) {
            Ok(Event::Start(element)) => {
                let name = local_name(element.name().as_ref());
                if name == "Contents" {
                    current = Some(PendingObjectSummary::default());
                }
                path.push(name);
            }
            Ok(Event::End(element)) => {
                let name = local_name(element.name().as_ref());
                if name == "Contents" {
                    if let Some(current) = current.take() {
                        objects.push(ObjectMetadata {
                            key: current.key,
                            size: current.size,
                            etag: current.etag,
                            last_modified: current.last_modified,
                            content_type: None,
                            metadata: BTreeMap::new(),
                        });
                    }
                }
                if path.last().map(|item| item.as_str()) == Some(name.as_str()) {
                    let _ = path.pop();
                }
            }
            Ok(Event::Text(text)) => {
                if let Some(current) = current.as_mut() {
                    let value = text.xml_content().map_err(xml_parse_error)?.into_owned();
                    match path.as_slice() {
                        [.., contents, key_name] if contents == "Contents" && key_name == "Key" => {
                            current.key = value;
                        }
                        [.., contents, size_name]
                            if contents == "Contents" && size_name == "Size" =>
                        {
                            current.size = value.parse::<u64>().unwrap_or_default();
                        }
                        [.., contents, etag_name]
                            if contents == "Contents" && etag_name == "ETag" =>
                        {
                            current.etag = Some(trim_surrounding_quotes(value.as_str()));
                        }
                        [.., contents, modified_name]
                            if contents == "Contents" && modified_name == "LastModified" =>
                        {
                            current.last_modified = Some(value);
                        }
                        _ => {}
                    }
                }
            }
            Ok(Event::Eof) => break,
            Ok(_) => {}
            Err(error) => return Err(xml_parse_error(error)),
        }
        buffer.clear();
    }

    Ok(objects)
}

fn collect_path_texts(xml: &str, target_path: &[&str]) -> StorageResult<Vec<String>> {
    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(true);
    let mut buffer = Vec::new();
    let mut path = Vec::<String>::new();
    let mut values = Vec::new();

    loop {
        match reader.read_event_into(&mut buffer) {
            Ok(Event::Start(element)) => {
                path.push(local_name(element.name().as_ref()));
            }
            Ok(Event::End(element)) => {
                let name = local_name(element.name().as_ref());
                if path.last().map(|item| item.as_str()) == Some(name.as_str()) {
                    let _ = path.pop();
                }
            }
            Ok(Event::Text(text)) => {
                if path_matches(&path, target_path) {
                    values.push(text.xml_content().map_err(xml_parse_error)?.into_owned());
                }
            }
            Ok(Event::Eof) => break,
            Ok(_) => {}
            Err(error) => return Err(xml_parse_error(error)),
        }
        buffer.clear();
    }

    Ok(values)
}

fn collect_first_path_text(xml: &str, target_path: &[&str]) -> StorageResult<Option<String>> {
    Ok(collect_path_texts(xml, target_path)?.into_iter().next())
}

fn collect_first_local_name_text(xml: &str, name: &str) -> StorageResult<Option<String>> {
    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(true);
    let mut buffer = Vec::new();
    let mut current_name = None::<String>;

    loop {
        match reader.read_event_into(&mut buffer) {
            Ok(Event::Start(element)) => {
                current_name = Some(local_name(element.name().as_ref()));
            }
            Ok(Event::Text(text)) => {
                if current_name.as_deref() == Some(name) {
                    return text
                        .xml_content()
                        .map(|value| Some(value.into_owned()))
                        .map_err(xml_parse_error);
                }
            }
            Ok(Event::End(_)) => {
                current_name = None;
            }
            Ok(Event::Eof) => break,
            Ok(_) => {}
            Err(error) => return Err(xml_parse_error(error)),
        }
        buffer.clear();
    }

    Ok(None)
}

fn xml_parse_error(error: impl std::fmt::Display) -> StorageError {
    StorageError::Backend(format!("failed to parse S3 XML response: {error}"))
}

fn path_matches(path: &[String], target_path: &[&str]) -> bool {
    if path.len() != target_path.len() {
        return false;
    }
    path.iter()
        .map(String::as_str)
        .zip(target_path.iter().copied())
        .all(|(left, right)| left == right)
}

fn local_name(raw: &[u8]) -> String {
    let name = String::from_utf8_lossy(raw);
    name.rsplit(':').next().unwrap_or_default().to_owned()
}

fn normalize_header_value(value: &str) -> String {
    value.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn build_canonical_headers(headers: &BTreeMap<String, String>) -> (String, String) {
    let canonical_headers = headers
        .iter()
        .map(|(name, value)| {
            format!(
                "{}:{}\n",
                name.to_ascii_lowercase(),
                normalize_header_value(value)
            )
        })
        .collect::<String>();
    let signed_headers = headers
        .keys()
        .map(|name| name.to_ascii_lowercase())
        .collect::<Vec<_>>()
        .join(";");
    (canonical_headers, signed_headers)
}

fn derive_signing_key(
    secret_key: &str,
    date_stamp: &str,
    region: &str,
    service: &str,
) -> StorageResult<Vec<u8>> {
    let k_date = sign_hmac(
        format!("AWS4{secret_key}").as_bytes(),
        date_stamp.as_bytes(),
    )?;
    let k_region = sign_hmac(&k_date, region.as_bytes())?;
    let k_service = sign_hmac(&k_region, service.as_bytes())?;
    sign_hmac(&k_service, b"aws4_request")
}

fn sign_hmac(key: &[u8], data: &[u8]) -> StorageResult<Vec<u8>> {
    let mut mac = HmacSha256::new_from_slice(key)
        .map_err(|error| StorageError::Backend(format!("failed to initialize HMAC: {error}")))?;
    mac.update(data);
    Ok(mac.finalize().into_bytes().to_vec())
}

fn sha256_hex(data: &[u8]) -> String {
    hex_lower(Sha256::digest(data).as_slice())
}

fn hex_lower(data: &[u8]) -> String {
    data.iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>()
}

fn canonical_query_string(query_pairs: &[(String, String)]) -> String {
    let mut encoded = query_pairs
        .iter()
        .map(|(name, value)| {
            (
                aws_percent_encode(name, true),
                aws_percent_encode(value, true),
            )
        })
        .collect::<Vec<_>>();
    encoded.sort();
    encoded
        .into_iter()
        .map(|(name, value)| format!("{name}={value}"))
        .collect::<Vec<_>>()
        .join("&")
}

fn aws_percent_encode(value: &str, encode_slash: bool) -> String {
    let mut encoded = String::new();
    for byte in value.as_bytes() {
        let is_unreserved =
            byte.is_ascii_alphanumeric() || matches!(*byte, b'-' | b'_' | b'.' | b'~');
        if is_unreserved || (!encode_slash && *byte == b'/') {
            encoded.push(char::from(*byte));
        } else {
            encoded.push_str(&format!("%{byte:02X}"));
        }
    }
    encoded
}

fn trim_surrounding_quotes(value: &str) -> String {
    value.trim_matches('"').to_owned()
}

fn quoted_etag(value: &str) -> String {
    let trimmed = value.trim();
    if trimmed.starts_with('"') && trimmed.ends_with('"') {
        trimmed.to_owned()
    } else {
        format!("\"{trimmed}\"")
    }
}

fn escape_xml(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
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
