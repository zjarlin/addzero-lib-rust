use addzero_rustfs::{
    BlockingS3StorageClient, PartInfo, PartStatus, S3ClientConfig, S3StorageClient,
};
use quick_xml::Reader;
use quick_xml::events::Event;
use std::collections::{BTreeMap, HashMap};
use std::error::Error;
use std::io::{Read, Write};
use std::net::TcpListener;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::Duration;
use tempfile::NamedTempFile;

#[test]
fn blocking_client_supports_bucket_and_object_lifecycle() -> Result<(), Box<dyn Error>> {
    let server = FakeS3Server::spawn()?;
    let client = BlockingS3StorageClient::new(S3ClientConfig::new(
        server.base_url(),
        "test-access",
        "test-secret",
    ));

    client.create_bucket("demo")?;
    assert!(client.bucket_exists("demo")?);

    let metadata = BTreeMap::from([("app".to_owned(), "rustfs".to_owned())]);
    client.put_object_bytes("demo", "hello.txt", b"hello", Some("text/plain"), &metadata)?;

    assert!(client.object_exists("demo", "hello.txt")?);

    let object_metadata = client
        .get_object_metadata("demo", "hello.txt")?
        .ok_or_else(|| std::io::Error::other("missing object metadata"))?;
    assert_eq!(object_metadata.size, 5);
    assert_eq!(object_metadata.content_type.as_deref(), Some("text/plain"));
    assert_eq!(
        object_metadata.metadata.get("app").map(String::as_str),
        Some("rustfs"),
    );

    let objects = client.list_objects("demo", Some("hel"), true, 100)?;
    assert_eq!(objects.len(), 1);
    assert_eq!(objects[0].key, "hello.txt");

    client.copy_object("demo", "hello.txt", "demo", "copy.txt")?;
    assert_eq!(client.get_object("demo", "copy.txt")?, b"hello");

    let tempfile = NamedTempFile::new()?;
    client.get_object_to_file("demo", "copy.txt", tempfile.path())?;
    assert_eq!(std::fs::read(tempfile.path())?, b"hello");

    client.delete_objects("demo", &["hello.txt".to_owned(), "copy.txt".to_owned()])?;
    assert!(!client.object_exists("demo", "hello.txt")?);

    let buckets = client.list_buckets()?;
    assert_eq!(buckets, vec!["demo".to_owned()]);

    let download_url = client.generate_presigned_url("demo", "hello.txt", 120)?;
    let upload_url = client.generate_presigned_upload_url("demo", "upload.bin", 120)?;
    assert!(
        download_url
            .url
            .contains("X-Amz-Algorithm=AWS4-HMAC-SHA256")
    );
    assert!(download_url.url.contains("X-Amz-SignedHeaders=host"));
    assert!(upload_url.url.contains("X-Amz-Expires=120"));

    let requests = server.finish()?;
    assert!(
        requests
            .iter()
            .all(|request| request.headers.contains_key("authorization"))
    );
    assert!(
        requests
            .iter()
            .all(|request| request.headers.contains_key("x-amz-date"))
    );
    assert!(
        requests
            .iter()
            .all(|request| request.headers.contains_key("x-amz-content-sha256"))
    );

    let put_request = requests
        .iter()
        .find(|request| request.method == "PUT" && request.path == "/demo/hello.txt")
        .ok_or_else(|| std::io::Error::other("missing put object request"))?;
    assert_eq!(
        put_request
            .headers
            .get("x-amz-meta-app")
            .map(String::as_str),
        Some("rustfs"),
    );

    let copy_request = requests
        .iter()
        .find(|request| request.method == "PUT" && request.path == "/demo/copy.txt")
        .ok_or_else(|| std::io::Error::other("missing copy object request"))?;
    assert_eq!(
        copy_request
            .headers
            .get("x-amz-copy-source")
            .map(String::as_str),
        Some("/demo/hello.txt"),
    );

    let delete_many_request = requests
        .iter()
        .find(|request| request.method == "POST" && request.path == "/demo?delete=")
        .ok_or_else(|| std::io::Error::other("missing delete objects request"))?;
    let delete_body = delete_many_request.body_text();
    assert!(delete_body.contains("<Key>hello.txt</Key>"));
    assert!(delete_body.contains("<Key>copy.txt</Key>"));
    assert!(delete_many_request.headers.contains_key("content-md5"));
    Ok(())
}

#[test]
fn blocking_client_supports_multipart_upload_flow() -> Result<(), Box<dyn Error>> {
    let server = FakeS3Server::spawn()?;
    let client = BlockingS3StorageClient::new(S3ClientConfig::new(
        server.base_url(),
        "test-access",
        "test-secret",
    ));
    client.create_bucket("demo")?;

    let metadata = BTreeMap::from([("mode".to_owned(), "multipart".to_owned())]);
    let upload_id = client.init_multipart_upload(
        "demo",
        "big.bin",
        Some("application/octet-stream"),
        &metadata,
    )?;
    assert_eq!(
        client.list_multipart_uploads("demo")?,
        vec![upload_id.clone()]
    );

    let etag1 = client.upload_part("demo", "big.bin", &upload_id, 1, b"abc", None)?;
    let etag2 = client.upload_part("demo", "big.bin", &upload_id, 2, b"def", None)?;
    let parts = vec![
        PartInfo {
            part_number: 1,
            start: 0,
            end: 3,
            size: 3,
            etag: Some(etag1.clone()),
            status: PartStatus::Completed,
        },
        PartInfo {
            part_number: 2,
            start: 3,
            end: 6,
            size: 3,
            etag: Some(etag2.clone()),
            status: PartStatus::Completed,
        },
    ];
    client.complete_multipart_upload("demo", "big.bin", &upload_id, &parts)?;

    assert!(client.list_multipart_uploads("demo")?.is_empty());
    assert_eq!(client.get_object("demo", "big.bin")?, b"abcdef");

    let requests = server.finish()?;
    let init_request = requests
        .iter()
        .find(|request| request.method == "POST" && request.path == "/demo/big.bin?uploads=")
        .ok_or_else(|| std::io::Error::other("missing init multipart request"))?;
    assert_eq!(
        init_request
            .headers
            .get("x-amz-meta-mode")
            .map(String::as_str),
        Some("multipart"),
    );

    let complete_request = requests
        .iter()
        .find(|request| {
            request.method == "POST"
                && request
                    .path
                    .starts_with(&format!("/demo/big.bin?uploadId={upload_id}"))
                && request.body_text().contains("<CompleteMultipartUpload>")
        })
        .ok_or_else(|| std::io::Error::other("missing complete multipart request"))?;
    let complete_body = complete_request.body_text();
    assert!(complete_body.contains(&format!("<ETag>&quot;{etag1}&quot;</ETag>")));
    assert!(complete_body.contains(&format!("<ETag>&quot;{etag2}&quot;</ETag>")));
    Ok(())
}

#[test]
fn presigned_url_uses_virtual_host_when_path_style_disabled() -> Result<(), Box<dyn Error>> {
    let config = S3ClientConfig::new("https://storage.example.com:9443", "ak", "sk")
        .with_region("ap-southeast-1")
        .with_path_style_access(false);
    let client = BlockingS3StorageClient::new(config);

    let url = client.generate_presigned_url("demo", "nested/file.txt", 90)?;
    assert!(
        url.url
            .starts_with("https://demo.storage.example.com:9443/nested/file.txt?")
    );
    assert!(url.url.contains("X-Amz-Credential=ak%2F"));
    Ok(())
}

#[derive(Debug, Clone)]
struct CapturedRequest {
    method: String,
    path: String,
    headers: BTreeMap<String, String>,
    body: Vec<u8>,
}

impl CapturedRequest {
    fn body_text(&self) -> String {
        String::from_utf8_lossy(&self.body).into_owned()
    }
}

#[derive(Debug, Clone)]
struct StoredObject {
    bytes: Vec<u8>,
    content_type: Option<String>,
    metadata: BTreeMap<String, String>,
    etag: String,
}

#[derive(Debug, Clone)]
struct PendingUpload {
    bucket: String,
    key: String,
    content_type: Option<String>,
    metadata: BTreeMap<String, String>,
    parts: BTreeMap<u32, Vec<u8>>,
    etags: BTreeMap<u32, String>,
}

#[derive(Debug, Default)]
struct ServerState {
    buckets: BTreeMap<String, BTreeMap<String, StoredObject>>,
    uploads: BTreeMap<String, PendingUpload>,
    captured: Vec<CapturedRequest>,
    next_id: u64,
}

#[derive(Debug)]
struct TestResponse {
    status: u16,
    headers: Vec<(String, String)>,
    body: Vec<u8>,
}

impl TestResponse {
    fn empty(status: u16) -> Self {
        Self {
            status,
            headers: Vec::new(),
            body: Vec::new(),
        }
    }

    fn xml(status: u16, body: String) -> Self {
        Self {
            status,
            headers: vec![("Content-Type".to_owned(), "application/xml".to_owned())],
            body: body.into_bytes(),
        }
    }

    fn bytes(status: u16, body: Vec<u8>) -> Self {
        Self {
            status,
            headers: Vec::new(),
            body,
        }
    }
}

struct FakeS3Server {
    base_url: String,
    address: String,
    state: Arc<Mutex<ServerState>>,
    stop: Arc<AtomicBool>,
    handle: Option<JoinHandle<std::io::Result<()>>>,
}

impl FakeS3Server {
    fn spawn() -> Result<Self, Box<dyn Error>> {
        let listener = TcpListener::bind("127.0.0.1:0")?;
        listener.set_nonblocking(true)?;
        let address = listener.local_addr()?.to_string();
        let state = Arc::new(Mutex::new(ServerState::default()));
        let state_clone = Arc::clone(&state);
        let stop = Arc::new(AtomicBool::new(false));
        let stop_clone = Arc::clone(&stop);

        let handle = thread::spawn(move || -> std::io::Result<()> {
            while !stop_clone.load(Ordering::Relaxed) {
                match listener.accept() {
                    Ok((mut stream, _)) => {
                        let request = read_request(&mut stream)?;
                        let response = {
                            let mut guard = state_clone
                                .lock()
                                .map_err(|_| std::io::Error::other("fake server mutex poisoned"))?;
                            guard.captured.push(request.clone());
                            handle_request(&mut guard, &request)
                        };
                        write_response(&mut stream, response)?;
                    }
                    Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                        thread::sleep(Duration::from_millis(10));
                    }
                    Err(error) => {
                        return Err(error);
                    }
                }
            }
            Ok(())
        });

        Ok(Self {
            base_url: format!("http://{address}"),
            address,
            state,
            stop,
            handle: Some(handle),
        })
    }

    fn base_url(&self) -> &str {
        &self.base_url
    }

    fn finish(mut self) -> Result<Vec<CapturedRequest>, Box<dyn Error>> {
        self.stop.store(true, Ordering::Relaxed);
        let _ = std::net::TcpStream::connect(&self.address);
        if let Some(handle) = self.handle.take() {
            match handle.join() {
                Ok(result) => {
                    result?;
                }
                Err(_) => {
                    return Err(Box::new(std::io::Error::other(
                        "fake server thread panicked",
                    )));
                }
            }
        }
        let guard = self
            .state
            .lock()
            .map_err(|_| std::io::Error::other("fake server mutex poisoned"))?;
        Ok(guard.captured.clone())
    }
}

impl Drop for FakeS3Server {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Relaxed);
        let _ = std::net::TcpStream::connect(&self.address);
        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
        }
    }
}

fn handle_request(state: &mut ServerState, request: &CapturedRequest) -> TestResponse {
    let (path, query) = split_path_and_query(&request.path);
    let query_map = parse_query(query);
    let (bucket_name, object_key) = parse_path_style_bucket_and_key(path);

    match (
        request.method.as_str(),
        bucket_name.as_deref(),
        object_key.as_deref(),
    ) {
        ("GET", None, None) => list_buckets_response(state),
        ("HEAD", Some(bucket_name), None) => {
            if state.buckets.contains_key(bucket_name) {
                TestResponse::empty(200)
            } else {
                not_found_bucket(bucket_name)
            }
        }
        ("PUT", Some(bucket_name), None) if query_map.is_empty() => {
            state.buckets.entry(bucket_name.to_owned()).or_default();
            TestResponse::empty(200)
        }
        ("DELETE", Some(bucket_name), None) if query_map.is_empty() => {
            if state.buckets.remove(bucket_name).is_some() {
                TestResponse::empty(204)
            } else {
                not_found_bucket(bucket_name)
            }
        }
        ("GET", Some(bucket_name), None) if query_map.contains_key("list-type") => {
            list_objects_response(state, bucket_name, &query_map)
        }
        ("GET", Some(bucket_name), None) if query_map.contains_key("uploads") => {
            list_uploads_response(state, bucket_name)
        }
        ("POST", Some(bucket_name), None) if query_map.contains_key("delete") => {
            delete_objects_response(state, bucket_name, &request.body)
        }
        ("HEAD", Some(bucket_name), Some(object_key)) => {
            if let Some(object) = state
                .buckets
                .get(bucket_name)
                .and_then(|bucket| bucket.get(object_key))
            {
                let mut response = TestResponse::empty(200);
                response
                    .headers
                    .push(("Content-Length".to_owned(), object.bytes.len().to_string()));
                if let Some(content_type) = object.content_type.as_ref() {
                    response
                        .headers
                        .push(("Content-Type".to_owned(), content_type.clone()));
                }
                response
                    .headers
                    .push(("ETag".to_owned(), format!("\"{}\"", object.etag)));
                for (name, value) in &object.metadata {
                    response
                        .headers
                        .push((format!("x-amz-meta-{name}"), value.clone()));
                }
                response
            } else {
                not_found_object(bucket_name, object_key)
            }
        }
        ("PUT", Some(bucket_name), Some(object_key))
            if query_map.contains_key("partNumber") && query_map.contains_key("uploadId") =>
        {
            upload_part_response(state, bucket_name, object_key, &query_map, &request.body)
        }
        ("POST", Some(bucket_name), Some(object_key)) if query_map.contains_key("uploads") => {
            init_upload_response(state, bucket_name, object_key, &request.headers)
        }
        ("POST", Some(bucket_name), Some(object_key)) if query_map.contains_key("uploadId") => {
            complete_upload_response(state, bucket_name, object_key, &query_map)
        }
        ("DELETE", Some(bucket_name), Some(object_key)) if query_map.contains_key("uploadId") => {
            abort_upload_response(state, bucket_name, object_key, &query_map)
        }
        ("PUT", Some(bucket_name), Some(object_key))
            if request.headers.contains_key("x-amz-copy-source") =>
        {
            copy_object_response(state, bucket_name, object_key, &request.headers)
        }
        ("PUT", Some(bucket_name), Some(object_key)) => {
            let etag = next_id(state, "etag");
            let object = StoredObject {
                bytes: request.body.clone(),
                content_type: request.headers.get("content-type").cloned(),
                metadata: collect_metadata_headers(&request.headers),
                etag: etag.clone(),
            };
            state
                .buckets
                .entry(bucket_name.to_owned())
                .or_default()
                .insert(object_key.to_owned(), object);
            let mut response = TestResponse::empty(200);
            response
                .headers
                .push(("ETag".to_owned(), format!("\"{etag}\"")));
            response
        }
        ("GET", Some(bucket_name), Some(object_key)) => {
            if let Some(object) = state
                .buckets
                .get(bucket_name)
                .and_then(|bucket| bucket.get(object_key))
            {
                let mut response = TestResponse::bytes(200, object.bytes.clone());
                if let Some(content_type) = object.content_type.as_ref() {
                    response
                        .headers
                        .push(("Content-Type".to_owned(), content_type.clone()));
                }
                response
            } else {
                not_found_object(bucket_name, object_key)
            }
        }
        ("DELETE", Some(bucket_name), Some(object_key)) => {
            if let Some(bucket) = state.buckets.get_mut(bucket_name) {
                let _ = bucket.remove(object_key);
            }
            TestResponse::empty(204)
        }
        _ => TestResponse::xml(
            400,
            format!(
                "<Error><Code>Unsupported</Code><Message>{}</Message></Error>",
                escape_xml(&request.path)
            ),
        ),
    }
}

fn list_buckets_response(state: &ServerState) -> TestResponse {
    let buckets = state
        .buckets
        .keys()
        .map(|name| format!("<Bucket><Name>{}</Name></Bucket>", escape_xml(name)))
        .collect::<Vec<_>>()
        .join("");
    TestResponse::xml(
        200,
        format!("<ListAllMyBucketsResult><Buckets>{buckets}</Buckets></ListAllMyBucketsResult>"),
    )
}

fn list_objects_response(
    state: &ServerState,
    bucket_name: &str,
    query_map: &HashMap<String, String>,
) -> TestResponse {
    let Some(bucket) = state.buckets.get(bucket_name) else {
        return not_found_bucket(bucket_name);
    };
    let prefix = query_map
        .get("prefix")
        .map(String::as_str)
        .unwrap_or_default();
    let recursive = !query_map.contains_key("delimiter");
    let max_keys = query_map
        .get("max-keys")
        .and_then(|value| value.parse::<usize>().ok())
        .unwrap_or(1000);

    let contents = bucket
        .iter()
        .filter(|(key, _)| key.starts_with(prefix))
        .filter(|(key, _)| recursive || !key[prefix.len()..].trim_start_matches('/').contains('/'))
        .take(max_keys)
        .map(|(key, object)| {
            format!(
                "<Contents><Key>{}</Key><LastModified>2026-04-20T12:00:00Z</LastModified><ETag>\"{}\"</ETag><Size>{}</Size></Contents>",
                escape_xml(key),
                object.etag,
                object.bytes.len()
            )
        })
        .collect::<Vec<_>>()
        .join("");

    TestResponse::xml(
        200,
        format!("<ListBucketResult>{contents}</ListBucketResult>"),
    )
}

fn list_uploads_response(state: &ServerState, bucket_name: &str) -> TestResponse {
    let uploads = state
        .uploads
        .iter()
        .filter(|(_, upload)| upload.bucket == bucket_name)
        .map(|(upload_id, upload)| {
            format!(
                "<Upload><Key>{}</Key><UploadId>{}</UploadId></Upload>",
                escape_xml(&upload.key),
                escape_xml(upload_id)
            )
        })
        .collect::<Vec<_>>()
        .join("");
    TestResponse::xml(
        200,
        format!("<ListMultipartUploadsResult>{uploads}</ListMultipartUploadsResult>"),
    )
}

fn delete_objects_response(
    state: &mut ServerState,
    bucket_name: &str,
    body: &[u8],
) -> TestResponse {
    let Some(bucket) = state.buckets.get_mut(bucket_name) else {
        return not_found_bucket(bucket_name);
    };
    for key in extract_xml_tag_values(&String::from_utf8_lossy(body), "Key") {
        let _ = bucket.remove(&key);
    }
    TestResponse::xml(200, "<DeleteResult/>".to_owned())
}

fn init_upload_response(
    state: &mut ServerState,
    bucket_name: &str,
    object_key: &str,
    headers: &BTreeMap<String, String>,
) -> TestResponse {
    let upload_id = next_id(state, "upload");
    let upload = PendingUpload {
        bucket: bucket_name.to_owned(),
        key: object_key.to_owned(),
        content_type: headers.get("content-type").cloned(),
        metadata: collect_metadata_headers(headers),
        parts: BTreeMap::new(),
        etags: BTreeMap::new(),
    };
    state.uploads.insert(upload_id.clone(), upload);
    TestResponse::xml(
        200,
        format!(
            "<InitiateMultipartUploadResult><UploadId>{}</UploadId></InitiateMultipartUploadResult>",
            escape_xml(&upload_id)
        ),
    )
}

fn upload_part_response(
    state: &mut ServerState,
    bucket_name: &str,
    object_key: &str,
    query_map: &HashMap<String, String>,
    body: &[u8],
) -> TestResponse {
    let Some(upload_id) = query_map.get("uploadId") else {
        return TestResponse::xml(
            400,
            "<Error><Code>InvalidRequest</Code><Message>missing uploadId</Message></Error>"
                .to_owned(),
        );
    };
    let part_number = query_map
        .get("partNumber")
        .and_then(|value| value.parse::<u32>().ok())
        .unwrap_or(0);
    let etag = next_id(state, "etag");
    let Some(upload) = state.uploads.get_mut(upload_id) else {
        return TestResponse::xml(
            404,
            "<Error><Code>NoSuchUpload</Code><Message>upload not found</Message></Error>"
                .to_owned(),
        );
    };
    if upload.bucket != bucket_name || upload.key != object_key {
        return TestResponse::xml(
            404,
            "<Error><Code>NoSuchUpload</Code><Message>upload not found</Message></Error>"
                .to_owned(),
        );
    }

    upload.parts.insert(part_number, body.to_vec());
    upload.etags.insert(part_number, etag.clone());
    let mut response = TestResponse::empty(200);
    response
        .headers
        .push(("ETag".to_owned(), format!("\"{etag}\"")));
    response
}

fn complete_upload_response(
    state: &mut ServerState,
    bucket_name: &str,
    object_key: &str,
    query_map: &HashMap<String, String>,
) -> TestResponse {
    let Some(upload_id) = query_map.get("uploadId") else {
        return TestResponse::xml(
            400,
            "<Error><Code>InvalidRequest</Code><Message>missing uploadId</Message></Error>"
                .to_owned(),
        );
    };
    let Some(upload) = state.uploads.remove(upload_id) else {
        return TestResponse::xml(
            404,
            "<Error><Code>NoSuchUpload</Code><Message>upload not found</Message></Error>"
                .to_owned(),
        );
    };
    if upload.bucket != bucket_name || upload.key != object_key {
        return TestResponse::xml(
            404,
            "<Error><Code>NoSuchUpload</Code><Message>upload not found</Message></Error>"
                .to_owned(),
        );
    }

    let bytes = upload
        .parts
        .values()
        .flat_map(|part| part.iter().copied())
        .collect::<Vec<_>>();
    let etag = next_id(state, "etag");
    let object = StoredObject {
        bytes,
        content_type: upload.content_type,
        metadata: upload.metadata,
        etag: etag.clone(),
    };
    state
        .buckets
        .entry(bucket_name.to_owned())
        .or_default()
        .insert(object_key.to_owned(), object);

    TestResponse::xml(
        200,
        format!(
            "<CompleteMultipartUploadResult><ETag>\"{}\"</ETag></CompleteMultipartUploadResult>",
            escape_xml(&etag)
        ),
    )
}

fn abort_upload_response(
    state: &mut ServerState,
    bucket_name: &str,
    object_key: &str,
    query_map: &HashMap<String, String>,
) -> TestResponse {
    let Some(upload_id) = query_map.get("uploadId") else {
        return TestResponse::xml(
            400,
            "<Error><Code>InvalidRequest</Code><Message>missing uploadId</Message></Error>"
                .to_owned(),
        );
    };
    if state
        .uploads
        .remove(upload_id)
        .filter(|upload| upload.bucket == bucket_name && upload.key == object_key)
        .is_some()
    {
        TestResponse::empty(204)
    } else {
        TestResponse::xml(
            404,
            "<Error><Code>NoSuchUpload</Code><Message>upload not found</Message></Error>"
                .to_owned(),
        )
    }
}

fn copy_object_response(
    state: &mut ServerState,
    bucket_name: &str,
    object_key: &str,
    headers: &BTreeMap<String, String>,
) -> TestResponse {
    let Some(source) = headers.get("x-amz-copy-source") else {
        return TestResponse::xml(
            400,
            "<Error><Code>InvalidRequest</Code><Message>missing copy source</Message></Error>"
                .to_owned(),
        );
    };
    let (source_bucket, source_key) = parse_copy_source(source);
    let Some(source_object) = state
        .buckets
        .get(&source_bucket)
        .and_then(|bucket| bucket.get(&source_key))
        .cloned()
    else {
        return not_found_object(&source_bucket, &source_key);
    };

    state
        .buckets
        .entry(bucket_name.to_owned())
        .or_default()
        .insert(object_key.to_owned(), source_object);
    TestResponse::xml(200, "<CopyObjectResult/>".to_owned())
}

fn next_id(state: &mut ServerState, prefix: &str) -> String {
    state.next_id += 1;
    format!("{prefix}-{}", state.next_id)
}

fn collect_metadata_headers(headers: &BTreeMap<String, String>) -> BTreeMap<String, String> {
    headers
        .iter()
        .filter_map(|(name, value)| {
            name.strip_prefix("x-amz-meta-")
                .map(|suffix| (suffix.to_owned(), value.clone()))
        })
        .collect()
}

fn parse_copy_source(value: &str) -> (String, String) {
    let trimmed = value.trim_start_matches('/');
    let mut parts = trimmed.splitn(2, '/');
    let bucket = parts.next().map(percent_decode).unwrap_or_default();
    let key = parts.next().map(percent_decode).unwrap_or_default();
    (bucket, key)
}

fn parse_path_style_bucket_and_key(path: &str) -> (Option<String>, Option<String>) {
    let trimmed = path.trim_start_matches('/');
    if trimmed.is_empty() {
        return (None, None);
    }
    match trimmed.split_once('/') {
        Some((bucket_name, object_key)) => (
            Some(percent_decode(bucket_name)),
            Some(percent_decode(object_key)),
        ),
        None => (Some(percent_decode(trimmed)), None),
    }
}

fn split_path_and_query(path: &str) -> (&str, &str) {
    match path.split_once('?') {
        Some((path, query)) => (path, query),
        None => (path, ""),
    }
}

fn parse_query(query: &str) -> HashMap<String, String> {
    let mut values = HashMap::new();
    if query.is_empty() {
        return values;
    }
    for pair in query.split('&') {
        let (name, value) = match pair.split_once('=') {
            Some((name, value)) => (name, value),
            None => (pair, ""),
        };
        values.insert(percent_decode(name), percent_decode(value));
    }
    values
}

fn percent_decode(value: &str) -> String {
    let bytes = value.as_bytes();
    let mut output = Vec::with_capacity(bytes.len());
    let mut index = 0usize;
    while index < bytes.len() {
        match bytes[index] {
            b'%' if index + 2 < bytes.len() => {
                let high = hex_value(bytes[index + 1]);
                let low = hex_value(bytes[index + 2]);
                if let (Some(high), Some(low)) = (high, low) {
                    output.push((high << 4) | low);
                    index += 3;
                    continue;
                }
                output.push(bytes[index]);
                index += 1;
            }
            b'+' => {
                output.push(b' ');
                index += 1;
            }
            byte => {
                output.push(byte);
                index += 1;
            }
        }
    }
    String::from_utf8_lossy(&output).into_owned()
}

fn hex_value(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        b'A'..=b'F' => Some(byte - b'A' + 10),
        _ => None,
    }
}

fn extract_xml_tag_values(xml: &str, tag_name: &str) -> Vec<String> {
    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(true);
    let mut buffer = Vec::new();
    let mut current = None::<String>;
    let mut values = Vec::new();

    loop {
        match reader.read_event_into(&mut buffer) {
            Ok(Event::Start(element)) => {
                current = Some(local_name(element.name().as_ref()));
            }
            Ok(Event::Text(text)) => {
                if current.as_deref() == Some(tag_name) {
                    if let Ok(value) = text.xml_content() {
                        values.push(value.into_owned());
                    }
                }
            }
            Ok(Event::End(_)) => {
                current = None;
            }
            Ok(Event::Eof) => break,
            Ok(_) => {}
            Err(_) => break,
        }
        buffer.clear();
    }

    values
}

fn local_name(raw: &[u8]) -> String {
    let name = String::from_utf8_lossy(raw);
    name.rsplit(':').next().unwrap_or_default().to_owned()
}

fn not_found_bucket(bucket_name: &str) -> TestResponse {
    TestResponse::xml(
        404,
        format!(
            "<Error><Code>NoSuchBucket</Code><Message>{}</Message></Error>",
            escape_xml(bucket_name)
        ),
    )
}

fn not_found_object(bucket_name: &str, object_key: &str) -> TestResponse {
    TestResponse::xml(
        404,
        format!(
            "<Error><Code>NoSuchKey</Code><Message>{}/{}</Message></Error>",
            escape_xml(bucket_name),
            escape_xml(object_key)
        ),
    )
}

fn read_request(stream: &mut std::net::TcpStream) -> std::io::Result<CapturedRequest> {
    let mut buffer = Vec::new();
    let mut chunk = [0u8; 4096];
    let header_end = loop {
        let read = stream.read(&mut chunk)?;
        if read == 0 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::UnexpectedEof,
                "request ended before headers",
            ));
        }
        buffer.extend_from_slice(&chunk[..read]);
        if let Some(index) = find_bytes(&buffer, b"\r\n\r\n") {
            break index + 4;
        }
    };

    let header_text = String::from_utf8_lossy(&buffer[..header_end]).into_owned();
    let mut lines = header_text.split("\r\n");
    let request_line = lines.next().ok_or_else(|| {
        std::io::Error::new(std::io::ErrorKind::InvalidData, "missing request line")
    })?;
    let mut request_parts = request_line.split_whitespace();
    let method = request_parts
        .next()
        .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::InvalidData, "missing method"))?
        .to_owned();
    let path = request_parts
        .next()
        .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::InvalidData, "missing path"))?
        .to_owned();

    let mut headers = BTreeMap::new();
    let mut content_length = 0usize;
    for line in lines {
        if line.is_empty() {
            continue;
        }
        let Some((name, value)) = line.split_once(':') else {
            continue;
        };
        let normalized_name = name.trim().to_ascii_lowercase();
        let trimmed_value = value.trim().to_owned();
        if normalized_name == "content-length" {
            content_length = trimmed_value.parse::<usize>().unwrap_or_default();
        }
        headers.insert(normalized_name, trimmed_value);
    }

    while buffer.len() < header_end + content_length {
        let read = stream.read(&mut chunk)?;
        if read == 0 {
            break;
        }
        buffer.extend_from_slice(&chunk[..read]);
    }

    let body = if content_length == 0 {
        Vec::new()
    } else {
        buffer[header_end..header_end + content_length].to_vec()
    };

    Ok(CapturedRequest {
        method,
        path,
        headers,
        body,
    })
}

fn write_response(stream: &mut std::net::TcpStream, response: TestResponse) -> std::io::Result<()> {
    let mut header_text = format!("HTTP/1.1 {} OK\r\n", response.status);
    let mut has_content_length = false;
    for (name, value) in &response.headers {
        if name.eq_ignore_ascii_case("content-length") {
            has_content_length = true;
        }
        header_text.push_str(&format!("{name}: {value}\r\n"));
    }
    if !has_content_length {
        header_text.push_str(&format!("Content-Length: {}\r\n", response.body.len()));
    }
    header_text.push_str("Connection: close\r\n\r\n");
    stream.write_all(header_text.as_bytes())?;
    stream.write_all(&response.body)?;
    stream.flush()
}

fn find_bytes(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    haystack
        .windows(needle.len())
        .position(|window| window == needle)
}

fn escape_xml(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}
