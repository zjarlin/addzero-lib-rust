use crate::progress::{InMemoryUploadProgressStorage, PartInfo};
use crate::types::{ObjectMetadata, PresignedUrl, RustfsConfig, S3ClientConfig};
use anyhow::{anyhow, bail, Context};
use az_derive_aliases::{
    apply, impl_default, plain_clone, plain_clone_debug, plain_default_clone_debug,
    plain_default_debug,
};
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

type HmacSha256 = Hmac<Sha256>;

/// 从中毒 mutex 中恢复，避免测试/内存实现因一次 panic 失去可用性。
fn recover_lock<T>(mutex: &std::sync::Mutex<T>) -> std::sync::MutexGuard<'_, T> {
    mutex.lock().unwrap_or_else(|poisoned| {
        eprintln!("WARN: in-memory storage mutex was poisoned, recovering");
        poisoned.into_inner()
    })
}

const AWS_SERVICE_NAME: &str = "s3";
const UNSIGNED_PAYLOAD: &str = "UNSIGNED-PAYLOAD";

/// S3 兼容对象存储客户端接口。
///
/// trait 覆盖 bucket、对象、分片上传和预签名 URL 四类操作，便于在真实 HTTP
/// 客户端和内存实现之间切换。
pub trait S3StorageClient: Send + Sync {
    fn bucket_exists(&self, bucket_name: &str) -> anyhow::Result<bool>;
    fn create_bucket(&self, bucket_name: &str) -> anyhow::Result<()>;
    fn list_buckets(&self) -> anyhow::Result<Vec<String>>;
    fn delete_bucket(&self, bucket_name: &str) -> anyhow::Result<()>;

    fn object_exists(&self, bucket_name: &str, key: &str) -> anyhow::Result<bool>;
    fn get_object_metadata(
        &self,
        bucket_name: &str,
        key: &str,
    ) -> anyhow::Result<Option<ObjectMetadata>>;
    fn put_object_bytes(
        &self,
        bucket_name: &str,
        key: &str,
        data: &[u8],
        content_type: Option<&str>,
        metadata: &BTreeMap<String, String>,
    ) -> anyhow::Result<()>;
    fn put_object_file(
        &self,
        bucket_name: &str,
        key: &str,
        path: &Path,
        content_type: Option<&str>,
        metadata: &BTreeMap<String, String>,
    ) -> anyhow::Result<()>;
    fn get_object(&self, bucket_name: &str, key: &str) -> anyhow::Result<Vec<u8>>;
    fn get_object_to_file(&self, bucket_name: &str, key: &str, target: &Path) -> anyhow::Result<()>;
    fn delete_object(&self, bucket_name: &str, key: &str) -> anyhow::Result<()>;
    fn delete_objects(&self, bucket_name: &str, keys: &[String]) -> anyhow::Result<()>;
    fn copy_object(
        &self,
        source_bucket: &str,
        source_key: &str,
        target_bucket: &str,
        target_key: &str,
    ) -> anyhow::Result<()>;
    fn list_objects(
        &self,
        bucket_name: &str,
        prefix: Option<&str>,
        recursive: bool,
        max_keys: usize,
    ) -> anyhow::Result<Vec<ObjectMetadata>>;

    fn init_multipart_upload(
        &self,
        bucket_name: &str,
        key: &str,
        content_type: Option<&str>,
        metadata: &BTreeMap<String, String>,
    ) -> anyhow::Result<String>;
    fn upload_part(
        &self,
        bucket_name: &str,
        key: &str,
        upload_id: &str,
        part_number: u32,
        data: &[u8],
        content_type: Option<&str>,
    ) -> anyhow::Result<String>;
    fn complete_multipart_upload(
        &self,
        bucket_name: &str,
        key: &str,
        upload_id: &str,
        parts: &[PartInfo],
    ) -> anyhow::Result<()>;
    fn abort_multipart_upload(
        &self,
        bucket_name: &str,
        key: &str,
        upload_id: &str,
    ) -> anyhow::Result<()>;
    fn list_multipart_uploads(&self, bucket_name: &str) -> anyhow::Result<Vec<String>>;

    fn generate_presigned_url(
        &self,
        bucket_name: &str,
        key: &str,
        expiration_seconds: u64,
    ) -> anyhow::Result<PresignedUrl>;
    fn generate_presigned_upload_url(
        &self,
        bucket_name: &str,
        key: &str,
        expiration_seconds: u64,
    ) -> anyhow::Result<PresignedUrl>;
}

/// S3 兼容客户端工厂接口。
pub trait S3StorageClientFactory: Send + Sync {
    fn create_client(&self, config: S3ClientConfig) -> Arc<dyn S3StorageClient>;
    fn create_default_client(&self) -> Arc<dyn S3StorageClient>;
}

/// 默认客户端工厂，通过调用方注入的配置源创建阻塞式 S3 客户端。
#[apply(plain_clone)]
pub struct DefaultS3StorageClientFactory {
    default_config: Arc<dyn Fn() -> S3ClientConfig + Send + Sync>,
}

impl DefaultS3StorageClientFactory {
    /// 使用调用方提供的默认配置源创建工厂。
    pub fn new(default_config: impl Fn() -> S3ClientConfig + Send + Sync + 'static) -> Self {
        Self {
            default_config: Arc::new(default_config),
        }
    }
}

impl_default!(
    DefaultS3StorageClientFactory => DefaultS3StorageClientFactory::new(|| {
        S3ClientConfig::from(RustfsConfig::default())
    })
);

impl S3StorageClientFactory for DefaultS3StorageClientFactory {
    fn create_client(&self, config: S3ClientConfig) -> Arc<dyn S3StorageClient> {
        Arc::new(BlockingS3StorageClient::new(config))
    }

    fn create_default_client(&self) -> Arc<dyn S3StorageClient> {
        Arc::new(BlockingS3StorageClient::new((self.default_config)()))
    }
}

/// 基于 `reqwest::blocking` 的 S3 兼容客户端实现。
///
/// 该实现负责 AWS SigV4 签名、XML 响应解析和分片上传协议。
#[apply(plain_clone_debug)]
pub struct BlockingS3StorageClient {
    config: S3ClientConfig,
    http: Client,
}

#[apply(plain_clone_debug)]
struct RequestTarget {
    url: Url,
    canonical_uri: String,
    canonical_query: String,
    host_header: String,
}

#[apply(plain_default_debug)]
struct PendingObjectSummary {
    key: String,
    size: u64,
    etag: Option<String>,
    last_modified: Option<String>,
}

impl BlockingS3StorageClient {
    /// 使用指定配置创建阻塞式客户端。
    pub fn new(config: S3ClientConfig) -> Self {
        let http = match Client::builder().build() {
            Ok(client) => client,
            Err(_) => Client::new(),
        };
        Self { config, http }
    }

    /// 返回当前客户端配置。
    pub fn config(&self) -> &S3ClientConfig {
        &self.config
    }

    fn endpoint_url(&self) -> anyhow::Result<Url> {
        Url::parse(&self.config.endpoint).with_context(|| {
            format!(
                "invalid storage configuration: invalid endpoint `{}`",
                self.config.endpoint
            )
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
    ) -> anyhow::Result<RequestTarget> {
        let endpoint = self.endpoint_url()?;
        let scheme = endpoint.scheme();
        let endpoint_host = endpoint.host_str().ok_or_else(|| {
            anyhow!(
                "invalid storage configuration: endpoint `{}` does not contain a host",
                self.config.endpoint,
            )
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

        let url = Url::parse(&raw_url)
            .with_context(|| format!("invalid storage configuration: failed to build request URL `{raw_url}`"))?;

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
    ) -> anyhow::Result<Response> {
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
            .map_err(|error| anyhow!("storage backend error: request failed: {error}"))
    }

    fn execute_empty_body_request(
        &self,
        method: Method,
        bucket_name: Option<&str>,
        key: Option<&str>,
        query_pairs: Vec<(String, String)>,
        headers: BTreeMap<String, String>,
    ) -> anyhow::Result<Response> {
        self.execute_signed_request(method, bucket_name, key, query_pairs, headers, Vec::new())
    }

    fn ensure_success(
        &self,
        response: Response,
        bucket_name: Option<&str>,
        key: Option<&str>,
    ) -> anyhow::Result<Response> {
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
    ) -> anyhow::Result<PresignedUrl> {
        if expiration_seconds == 0 {
            bail!("invalid storage configuration: presigned URL expiration must be greater than zero");
        }
        if expiration_seconds > 7 * 24 * 60 * 60 {
            bail!("invalid storage configuration: presigned URL expiration cannot exceed 7 days");
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
    fn bucket_exists(&self, bucket_name: &str) -> anyhow::Result<bool> {
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
                let bucket = Some(bucket_name);
                let error = response_to_storage_error(response, bucket, None);

                return Err(error);
            }
        })
    }

    fn create_bucket(&self, bucket_name: &str) -> anyhow::Result<()> {
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

    fn list_buckets(&self) -> anyhow::Result<Vec<String>> {
        let response =
            self.execute_empty_body_request(Method::GET, None, None, Vec::new(), BTreeMap::new())?;
        let body = response_to_text(self.ensure_success(response, None, None)?)?;
        collect_path_texts(
            &body,
            &["ListAllMyBucketsResult", "Buckets", "Bucket", "Name"],
        )
    }

    fn delete_bucket(&self, bucket_name: &str) -> anyhow::Result<()> {
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

    fn object_exists(&self, bucket_name: &str, key: &str) -> anyhow::Result<bool> {
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
                let bucket = Some(bucket_name);
                let object_key = Some(key);
                let error = response_to_storage_error(response, bucket, object_key);

                return Err(error);
            }
        })
    }

    fn get_object_metadata(
        &self,
        bucket_name: &str,
        key: &str,
    ) -> anyhow::Result<Option<ObjectMetadata>> {
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
                let bucket = Some(bucket_name);
                let object_key = Some(key);
                let error = response_to_storage_error(response, bucket, object_key);

                return Err(error);
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
    ) -> anyhow::Result<()> {
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
    ) -> anyhow::Result<()> {
        let bytes = std::fs::read(path)?;
        self.put_object_bytes(bucket_name, key, &bytes, content_type, metadata)
    }

    fn get_object(&self, bucket_name: &str, key: &str) -> anyhow::Result<Vec<u8>> {
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
            .map_err(|error| anyhow!("storage backend error: failed to read response body: {error}"))
    }

    fn get_object_to_file(&self, bucket_name: &str, key: &str, target: &Path) -> anyhow::Result<()> {
        let bytes = self.get_object(bucket_name, key)?;
        if let Some(parent) = target.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(target, bytes)?;
        Ok(())
    }

    fn delete_object(&self, bucket_name: &str, key: &str) -> anyhow::Result<()> {
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

    fn delete_objects(&self, bucket_name: &str, keys: &[String]) -> anyhow::Result<()> {
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
    ) -> anyhow::Result<()> {
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
    ) -> anyhow::Result<Vec<ObjectMetadata>> {
        if max_keys == 0 {
            return Ok(Vec::new());
        }

        let mut remaining = max_keys;
        let mut continuation_token = None::<String>;
        let mut objects = Vec::new();

        loop {
            let mut query_pairs = vec![
                ("list-type".to_owned(), "2".to_owned()),
                ("max-keys".to_owned(), remaining.to_string()),
            ];
            if let Some(prefix) = prefix.filter(|value| !value.is_empty()) {
                query_pairs.push(("prefix".to_owned(), prefix.to_owned()));
            }
            if !recursive {
                query_pairs.push(("delimiter".to_owned(), "/".to_owned()));
            }
            if let Some(token) = continuation_token
                .as_ref()
                .filter(|value| !value.is_empty())
            {
                query_pairs.push(("continuation-token".to_owned(), token.clone()));
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
            let page = parse_list_objects_response(&body)?;
            let page_len = page.objects.len();
            objects.extend(page.objects);
            remaining = remaining.saturating_sub(page_len);

            if remaining == 0 || !page.is_truncated {
                break;
            }

            let Some(token) = page.next_continuation_token else {
                break;
            };
            continuation_token = Some(token);
        }

        Ok(objects)
    }

    fn init_multipart_upload(
        &self,
        bucket_name: &str,
        key: &str,
        content_type: Option<&str>,
        metadata: &BTreeMap<String, String>,
    ) -> anyhow::Result<String> {
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
            || anyhow!("storage backend error: {}", "multipart upload response missing UploadId".to_owned()),
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
    ) -> anyhow::Result<String> {
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
    ) -> anyhow::Result<()> {
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
    ) -> anyhow::Result<()> {
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

    fn list_multipart_uploads(&self, bucket_name: &str) -> anyhow::Result<Vec<String>> {
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
    ) -> anyhow::Result<PresignedUrl> {
        self.presigned_url("GET", bucket_name, key, expiration_seconds)
    }

    fn generate_presigned_upload_url(
        &self,
        bucket_name: &str,
        key: &str,
        expiration_seconds: u64,
    ) -> anyhow::Result<PresignedUrl> {
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

fn build_host_header(url: &Url) -> anyhow::Result<String> {
    let host = url.host_str().ok_or_else(|| {
        anyhow!("invalid storage configuration: request URL `{url}` does not contain a host")
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

fn response_to_text(response: Response) -> anyhow::Result<String> {
    response
        .bytes()
        .map(|bytes| String::from_utf8_lossy(bytes.as_ref()).into_owned())
        .map_err(|error| anyhow!("storage backend error: failed to read response body: {error}"))
}

fn response_to_storage_error(
    response: Response,
    bucket_name: Option<&str>,
    key: Option<&str>,
) -> anyhow::Error {
    let status = response.status();
    let body = match response.bytes() {
        Ok(bytes) => String::from_utf8_lossy(bytes.as_ref()).into_owned(),
        Err(error) => {
            return anyhow!(
                "S3 request failed with HTTP {} and body read error: {error}",
                status.as_u16()
            );
        }
    };
    let error_code = collect_first_local_name_text(&body, "Code").ok().flatten();
    let error_message = collect_first_local_name_text(&body, "Message")
        .ok()
        .flatten();

    match (status, error_code.as_deref(), bucket_name, key) {
        (StatusCode::NOT_FOUND, Some("NoSuchBucket"), Some(bucket_name), _) => {
            anyhow!("bucket `{}` was not found", bucket_name.to_owned())
        }
        (StatusCode::NOT_FOUND, Some("NoSuchKey"), Some(bucket_name), Some(key)) => {
            anyhow!("object `{}/{}` was not found", bucket_name.to_owned(), key.to_owned())
        }
        (StatusCode::NOT_FOUND, _, Some(bucket_name), Some(key)) => {
            anyhow!("object `{}/{}` was not found", bucket_name, key)
        }
        (StatusCode::NOT_FOUND, _, Some(bucket_name), None) => {
            anyhow!("bucket `{}` was not found", bucket_name)
        }
        _ => anyhow!(
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
                }),
        ),
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

struct ParsedListObjectsResponse {
    objects: Vec<ObjectMetadata>,
    is_truncated: bool,
    next_continuation_token: Option<String>,
}

fn parse_list_objects_response(xml: &str) -> anyhow::Result<ParsedListObjectsResponse> {
    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(true);
    let mut buffer = Vec::new();
    let mut path = Vec::<String>::new();
    let mut current = None::<PendingObjectSummary>;
    let mut objects = Vec::new();
    let mut common_prefixes = Vec::<String>::new();
    let mut is_truncated = false;
    let mut next_continuation_token = None::<String>;

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
                if name == "Contents" && let Some(current) = current.take() {
                    objects.push(ObjectMetadata {
                        key: current.key,
                        size: current.size,
                        etag: current.etag,
                        last_modified: current.last_modified,
                        content_type: None,
                        metadata: BTreeMap::new(),
                    });
                }
                if path.last().map(|item| item.as_str()) == Some(name.as_str()) {
                    let _ = path.pop();
                }
            }
            Ok(Event::Text(text)) => {
                let value = text.xml_content().map_err(xml_parse_error)?.into_owned();
                if let Some(current) = current.as_mut() {
                    match path.as_slice() {
                        [.., contents, key_name] if contents == "Contents" && key_name == "Key" => {
                            current.key = value.clone();
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
                            current.last_modified = Some(value.clone());
                        }
                        _ => {}
                    }
                }

                match path.as_slice() {
                    [.., prefixes, prefix_name]
                        if prefixes == "CommonPrefixes" && prefix_name == "Prefix" =>
                    {
                        common_prefixes.push(value.clone());
                    }
                    [.., node] if node == "IsTruncated" => {
                        is_truncated = value.eq_ignore_ascii_case("true");
                    }
                    [.., node] if node == "NextContinuationToken" => {
                        next_continuation_token = Some(value);
                    }
                    _ => {}
                }
            }
            Ok(Event::Eof) => break,
            Ok(_) => {}
            Err(error) => {
                let error = xml_parse_error(error);

                return Err(error);
            }
        }
        buffer.clear();
    }

    objects.extend(common_prefixes.into_iter().map(|prefix| ObjectMetadata {
        key: prefix,
        size: 0,
        etag: None,
        last_modified: None,
        content_type: Some("application/x-directory".to_string()),
        metadata: BTreeMap::new(),
    }));

    Ok(ParsedListObjectsResponse {
        objects,
        is_truncated,
        next_continuation_token,
    })
}

#[cfg(test)]
mod tests {
    use super::parse_list_objects_response;

    #[test]
    fn parse_list_objects_response_should_capture_pagination_fields() {
        let xml = r#"
            <ListBucketResult>
              <IsTruncated>true</IsTruncated>
              <NextContinuationToken>token-2</NextContinuationToken>
              <Contents>
                <Key>archive/foo.zip</Key>
                <LastModified>2026-05-02T00:00:00.000Z</LastModified>
                <ETag>"abc"</ETag>
                <Size>123</Size>
              </Contents>
            </ListBucketResult>
        "#;

        let parsed = parse_list_objects_response(xml).expect("xml should parse");

        assert!(parsed.is_truncated);
        assert_eq!(parsed.next_continuation_token.as_deref(), Some("token-2"));
        assert_eq!(parsed.objects.len(), 1);
        assert_eq!(parsed.objects[0].key, "archive/foo.zip");
        assert_eq!(parsed.objects[0].size, 123);
        assert_eq!(parsed.objects[0].etag.as_deref(), Some("abc"));
    }

    #[test]
    fn parse_list_objects_response_should_capture_common_prefixes() {
        let xml = r#"
            <ListBucketResult>
              <CommonPrefixes>
                <Prefix>branding/</Prefix>
              </CommonPrefixes>
              <CommonPrefixes>
                <Prefix>dotfiles/</Prefix>
              </CommonPrefixes>
            </ListBucketResult>
        "#;

        let parsed = parse_list_objects_response(xml).expect("xml should parse");

        assert_eq!(parsed.objects.len(), 2);
        assert_eq!(parsed.objects[0].key, "branding/");
        assert_eq!(
            parsed.objects[0].content_type.as_deref(),
            Some("application/x-directory")
        );
        assert_eq!(parsed.objects[1].key, "dotfiles/");
    }
}

fn collect_path_texts(xml: &str, target_path: &[&str]) -> anyhow::Result<Vec<String>> {
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
            Err(error) => {
                let error = xml_parse_error(error);

                return Err(error);
            }
        }
        buffer.clear();
    }

    Ok(values)
}

fn collect_first_path_text(xml: &str, target_path: &[&str]) -> anyhow::Result<Option<String>> {
    Ok(collect_path_texts(xml, target_path)?.into_iter().next())
}

fn collect_first_local_name_text(xml: &str, name: &str) -> anyhow::Result<Option<String>> {
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
            Err(error) => {
                let error = xml_parse_error(error);

                return Err(error);
            }
        }
        buffer.clear();
    }

    Ok(None)
}

fn xml_parse_error(error: impl std::fmt::Display) -> anyhow::Error {
    anyhow!("storage backend error: failed to parse S3 XML response: {error}")
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
) -> anyhow::Result<Vec<u8>> {
    let k_date = sign_hmac(
        format!("AWS4{secret_key}").as_bytes(),
        date_stamp.as_bytes(),
    )?;
    let k_region = sign_hmac(&k_date, region.as_bytes())?;
    let k_service = sign_hmac(&k_region, service.as_bytes())?;
    sign_hmac(&k_service, b"aws4_request")
}

fn sign_hmac(key: &[u8], data: &[u8]) -> anyhow::Result<Vec<u8>> {
    let mut mac = HmacSha256::new_from_slice(key)
        .map_err(|error| anyhow!("storage backend error: failed to initialize HMAC: {error}"))?;
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

/// 内存对象存储实现，主要用于测试和本地冒烟验证。
#[apply(plain_default_clone_debug)]
pub struct InMemoryS3StorageClient {
    state: Arc<Mutex<MemoryState>>,
}

#[apply(plain_default_debug)]
struct MemoryState {
    buckets: HashMap<String, HashMap<String, MemoryObject>>,
    uploads: HashMap<String, MemoryUpload>,
    next_id: u64,
}

#[apply(plain_clone_debug)]
struct MemoryObject {
    bytes: Vec<u8>,
    content_type: Option<String>,
    metadata: BTreeMap<String, String>,
    etag: String,
    last_modified: String,
}

#[apply(plain_clone_debug)]
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
    fn bucket_exists(&self, bucket_name: &str) -> anyhow::Result<bool> {
        Ok(recover_lock(&self.state).buckets.contains_key(bucket_name))
    }

    fn create_bucket(&self, bucket_name: &str) -> anyhow::Result<()> {
        recover_lock(&self.state)
            .buckets
            .entry(bucket_name.to_owned())
            .or_default();
        Ok(())
    }

    fn list_buckets(&self) -> anyhow::Result<Vec<String>> {
        Ok(recover_lock(&self.state).buckets.keys().cloned().collect())
    }

    fn delete_bucket(&self, bucket_name: &str) -> anyhow::Result<()> {
        recover_lock(&self.state)
            .buckets
            .remove(bucket_name)
            .ok_or_else(|| anyhow!("bucket `{}` was not found", bucket_name.to_owned()))?;
        Ok(())
    }

    fn object_exists(&self, bucket_name: &str, key: &str) -> anyhow::Result<bool> {
        Ok(recover_lock(&self.state)
            .buckets
            .get(bucket_name)
            .and_then(|bucket| bucket.get(key))
            .is_some())
    }

    fn get_object_metadata(
        &self,
        bucket_name: &str,
        key: &str,
    ) -> anyhow::Result<Option<ObjectMetadata>> {
        Ok(recover_lock(&self.state)
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
    ) -> anyhow::Result<()> {
        let mut state = recover_lock(&self.state);
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
    ) -> anyhow::Result<()> {
        let bytes = std::fs::read(path)?;
        self.put_object_bytes(bucket_name, key, &bytes, content_type, metadata)
    }

    fn get_object(&self, bucket_name: &str, key: &str) -> anyhow::Result<Vec<u8>> {
        recover_lock(&self.state)
            .buckets
            .get(bucket_name)
            .and_then(|bucket| bucket.get(key))
            .map(|object| object.bytes.clone())
            .ok_or_else(|| anyhow!("object `{}/{}` was not found", bucket_name.to_owned(), key.to_owned()))
    }

    fn get_object_to_file(&self, bucket_name: &str, key: &str, target: &Path) -> anyhow::Result<()> {
        let bytes = self.get_object(bucket_name, key)?;
        if let Some(parent) = target.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(target, bytes)?;
        Ok(())
    }

    fn delete_object(&self, bucket_name: &str, key: &str) -> anyhow::Result<()> {
        recover_lock(&self.state)
            .buckets
            .get_mut(bucket_name)
            .and_then(|bucket| bucket.remove(key))
            .ok_or_else(|| anyhow!("object `{}/{}` was not found", bucket_name.to_owned(), key.to_owned()))?;
        Ok(())
    }

    fn delete_objects(&self, bucket_name: &str, keys: &[String]) -> anyhow::Result<()> {
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
    ) -> anyhow::Result<()> {
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
    ) -> anyhow::Result<Vec<ObjectMetadata>> {
        let prefix = prefix.unwrap_or_default();
        let bucket = recover_lock(&self.state)
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
    ) -> anyhow::Result<String> {
        let mut state = recover_lock(&self.state);
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
    ) -> anyhow::Result<String> {
        let mut state = recover_lock(&self.state);
        let upload = state
            .uploads
            .get_mut(upload_id)
            .ok_or_else(|| anyhow!("storage backend error: unknown upload id `{upload_id}`"))?;
        upload.parts.insert(part_number, data.to_vec());
        Ok(format!("etag-{upload_id}-{part_number}"))
    }

    fn complete_multipart_upload(
        &self,
        bucket_name: &str,
        key: &str,
        upload_id: &str,
        parts: &[PartInfo],
    ) -> anyhow::Result<()> {
        let mut state = recover_lock(&self.state);
        let upload = state
            .uploads
            .remove(upload_id)
            .ok_or_else(|| anyhow!("storage backend error: unknown upload id `{upload_id}`"))?;

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
    ) -> anyhow::Result<()> {
        recover_lock(&self.state).uploads.remove(upload_id);
        Ok(())
    }

    fn list_multipart_uploads(&self, bucket_name: &str) -> anyhow::Result<Vec<String>> {
        Ok(recover_lock(&self.state)
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
    ) -> anyhow::Result<PresignedUrl> {
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
    ) -> anyhow::Result<PresignedUrl> {
        Ok(PresignedUrl {
            url: format!("memory://{bucket_name}/{key}?op=put"),
            expiration: SystemTime::now() + Duration::from_secs(expiration_seconds),
        })
    }
}
