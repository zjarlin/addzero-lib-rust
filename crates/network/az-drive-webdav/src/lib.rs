#![forbid(unsafe_code)]

//! 面向独立网盘的极简 WebDAV 适配层。
//!
//! 本 crate 通过 Axum 暴露 WebDAV 方法，同时将身份管理、版本控制和对象存储
//! 委托给共享的网盘 crate。它在设计上独立于 `apps/aio`。

use axum::Router;
use axum::body::Bytes;
use axum::extract::{Path, State};
use axum::http::{HeaderMap, HeaderName, HeaderValue, Method, StatusCode, header};
use axum::response::{IntoResponse, Response};
use axum::routing::any;
use az_drive_core::{EntryKey, RelativePath, RootAlias, content_hash, object_key_for_hash};
use az_drive_store::{
    DriveEntry, DriveEntryKind, DriveLock, DriveMetadataStore, DriveObjectStore, DriveStoreError,
    DriveVersion,
};
use chrono::{Duration, Utc};
use std::sync::Arc;
use thiserror::Error;
use uuid::Uuid;

const DEVICE_HEADER: &str = "x-aio-device-id";
const OWNER_HEADER: &str = "x-aio-owner";
const LOCK_TOKEN_HEADER: &str = "lock-token";

/// Result alias for WebDAV adapter operations.
pub type DriveWebdavResult<T> = Result<T, DriveWebdavError>;

/// WebDAV adapter error.
#[derive(Debug, Error)]
pub enum DriveWebdavError {
    /// Root alias or relative path was invalid.
    #[error("invalid drive path: {0}")]
    Core(#[from] az_drive_core::DriveCoreError),
    /// Metadata or object store operation failed.
    #[error("drive store error: {0}")]
    Store(#[from] DriveStoreError),
    /// Required HTTP header is missing or invalid.
    #[error("invalid header: {0}")]
    InvalidHeader(String),
    /// Requested resource was not found.
    #[error("not found: {0}")]
    NotFound(String),
}

/// Shared state for WebDAV routes.
#[derive(Clone)]
pub struct DriveWebdavState {
    metadata: Arc<dyn DriveMetadataStore>,
    objects: Arc<dyn DriveObjectStore>,
}

impl DriveWebdavState {
    /// Creates WebDAV state from shared stores.
    #[must_use]
    pub fn new(metadata: Arc<dyn DriveMetadataStore>, objects: Arc<dyn DriveObjectStore>) -> Self {
        Self { metadata, objects }
    }
}

/// Builds the WebDAV router.
pub fn drive_webdav_router(state: DriveWebdavState) -> Router {
    Router::new()
        .route("/dav/{space}/{root}", any(handle_dav_root))
        .route("/dav/{space}/{root}/{*relative}", any(handle_dav_path))
        .with_state(state)
}

async fn handle_dav_root(
    State(state): State<DriveWebdavState>,
    method: Method,
    headers: HeaderMap,
    Path((space, root)): Path<(String, String)>,
    body: Bytes,
) -> Response {
    handle_dav(state, method, headers, space, root, String::new(), body).await
}

async fn handle_dav_path(
    State(state): State<DriveWebdavState>,
    method: Method,
    headers: HeaderMap,
    Path((space, root, relative)): Path<(String, String, String)>,
    body: Bytes,
) -> Response {
    handle_dav(state, method, headers, space, root, relative, body).await
}

async fn handle_dav(
    state: DriveWebdavState,
    method: Method,
    headers: HeaderMap,
    space: String,
    root: String,
    relative: String,
    body: Bytes,
) -> Response {
    let result = match method.as_str() {
        "PROPFIND" => propfind(&state, &space, &root, &relative).await,
        "GET" => get_object(&state, &space, &root, &relative).await,
        "HEAD" => head_object(&state, &space, &root, &relative).await,
        "PUT" => put_object(&state, &headers, &space, &root, &relative, &body).await,
        "DELETE" => delete_object(&state, &space, &root, &relative).await,
        "COPY" => copy_object(&state, &headers, &space, &root, &relative, false).await,
        "MOVE" => copy_object(&state, &headers, &space, &root, &relative, true).await,
        "LOCK" => lock_object(&state, &headers, &space, &root, &relative).await,
        "UNLOCK" => unlock_object(&state, &headers, &space, &root, &relative).await,
        _ => Ok(status_response(
            StatusCode::METHOD_NOT_ALLOWED,
            "unsupported WebDAV method",
        )),
    };
    result.unwrap_or_else(error_response)
}

async fn propfind(
    state: &DriveWebdavState,
    space: &str,
    root: &str,
    relative: &str,
) -> DriveWebdavResult<Response> {
    let root_alias = RootAlias::parse(root)?;
    let prefix = RelativePath::parse(relative)?;
    let entries = state
        .metadata
        .list_entries(space, &root_alias, &prefix)
        .await?;
    let xml = multistatus_xml(space, root, &entries);
    Ok(xml_response(StatusCode::MULTI_STATUS, xml))
}

async fn get_object(
    state: &DriveWebdavState,
    space: &str,
    root: &str,
    relative: &str,
) -> DriveWebdavResult<Response> {
    let entry = require_entry(state, space, root, relative).await?;
    let version = state
        .metadata
        .latest_version(entry.id)
        .await?
        .ok_or_else(|| DriveWebdavError::NotFound(entry.key.remote_path()))?;
    let bytes = state.objects.get_object(&version.object_key).await?;
    let mut headers = HeaderMap::new();
    headers.insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("application/octet-stream"),
    );
    headers.insert(
        header::ETAG,
        HeaderValue::from_str(&format!("\"{}\"", version.content_hash))
            .map_err(|err| DriveWebdavError::InvalidHeader(err.to_string()))?,
    );
    Ok((StatusCode::OK, headers, bytes).into_response())
}

async fn head_object(
    state: &DriveWebdavState,
    space: &str,
    root: &str,
    relative: &str,
) -> DriveWebdavResult<Response> {
    let entry = require_entry(state, space, root, relative).await?;
    let version = state
        .metadata
        .latest_version(entry.id)
        .await?
        .ok_or_else(|| DriveWebdavError::NotFound(entry.key.remote_path()))?;
    let mut headers = HeaderMap::new();
    headers.insert(
        header::ETAG,
        HeaderValue::from_str(&format!("\"{}\"", version.content_hash))
            .map_err(|err| DriveWebdavError::InvalidHeader(err.to_string()))?,
    );
    Ok((StatusCode::OK, headers).into_response())
}

async fn put_object(
    state: &DriveWebdavState,
    headers: &HeaderMap,
    space: &str,
    root: &str,
    relative: &str,
    body: &[u8],
) -> DriveWebdavResult<Response> {
    let key = entry_key(space, root, relative)?;
    let hash = content_hash(body);
    let object_key = object_key_for_hash(&hash);
    if !state.objects.object_exists(&object_key).await? {
        state.objects.put_object(&object_key, body).await?;
    }
    let entry = state
        .metadata
        .upsert_entry(&key, DriveEntryKind::File)
        .await?;
    let latest = state.metadata.latest_version(entry.id).await?;
    let version = latest
        .map(|version| version.version.saturating_add(1))
        .unwrap_or(1);
    let device_id = header_or(headers, DEVICE_HEADER, "webdav")?;
    state
        .metadata
        .insert_version(DriveVersion {
            id: Uuid::new_v4(),
            entry_id: entry.id,
            version,
            content_hash: hash,
            object_key,
            size_bytes: body.len() as u64,
            device_id,
            modified_at: Utc::now(),
        })
        .await?;
    Ok(status_response(StatusCode::CREATED, "created"))
}

async fn delete_object(
    state: &DriveWebdavState,
    space: &str,
    root: &str,
    relative: &str,
) -> DriveWebdavResult<Response> {
    let key = entry_key(space, root, relative)?;
    state.metadata.delete_entry(&key).await?;
    Ok(status_response(StatusCode::NO_CONTENT, ""))
}

async fn copy_object(
    state: &DriveWebdavState,
    headers: &HeaderMap,
    space: &str,
    root: &str,
    relative: &str,
    move_source: bool,
) -> DriveWebdavResult<Response> {
    let source = entry_key(space, root, relative)?;
    let destination = destination_key(headers)?;
    let source_entry = state
        .metadata
        .get_entry(&source)
        .await?
        .ok_or_else(|| DriveWebdavError::NotFound(source.remote_path()))?;
    let source_version = state
        .metadata
        .latest_version(source_entry.id)
        .await?
        .ok_or_else(|| DriveWebdavError::NotFound(source.remote_path()))?;
    let bytes = state.objects.get_object(&source_version.object_key).await?;
    let response = put_object(
        state,
        headers,
        &destination.space_id,
        destination.root_alias.as_str(),
        destination.relative_path.as_str(),
        &bytes,
    )
    .await?;
    if move_source {
        state.metadata.delete_entry(&source).await?;
    }
    Ok(response)
}

async fn lock_object(
    state: &DriveWebdavState,
    headers: &HeaderMap,
    space: &str,
    root: &str,
    relative: &str,
) -> DriveWebdavResult<Response> {
    let key = entry_key(space, root, relative)?;
    let entry = state
        .metadata
        .upsert_entry(&key, DriveEntryKind::File)
        .await?;
    let token = format!("opaquelocktoken:{}", Uuid::new_v4());
    let owner_device_id = header_or(headers, DEVICE_HEADER, "webdav")?;
    let owner_name = header_or(headers, OWNER_HEADER, &owner_device_id)?;
    let lock = state
        .metadata
        .acquire_lock(DriveLock {
            entry_id: entry.id,
            owner_device_id,
            owner_name,
            token: token.clone(),
            expires_at: Utc::now() + Duration::minutes(30),
        })
        .await?;

    let mut headers = HeaderMap::new();
    headers.insert(
        HeaderName::from_static(LOCK_TOKEN_HEADER),
        HeaderValue::from_str(&format!("<{}>", lock.token))
            .map_err(|err| DriveWebdavError::InvalidHeader(err.to_string()))?,
    );
    Ok((
        StatusCode::OK,
        headers,
        lockdiscovery_xml(&lock.owner_device_id, &lock.token),
    )
        .into_response())
}

async fn unlock_object(
    state: &DriveWebdavState,
    headers: &HeaderMap,
    space: &str,
    root: &str,
    relative: &str,
) -> DriveWebdavResult<Response> {
    let entry = require_entry(state, space, root, relative).await?;
    let token = header_or(headers, LOCK_TOKEN_HEADER, "")?
        .trim()
        .trim_start_matches('<')
        .trim_end_matches('>')
        .to_owned();
    if token.is_empty() {
        return Err(DriveWebdavError::InvalidHeader(
            LOCK_TOKEN_HEADER.to_owned(),
        ));
    }
    let released = state.metadata.release_lock(entry.id, &token).await?;
    if released {
        Ok(status_response(StatusCode::NO_CONTENT, ""))
    } else {
        Ok(status_response(
            StatusCode::PRECONDITION_FAILED,
            "lock token did not match",
        ))
    }
}

async fn require_entry(
    state: &DriveWebdavState,
    space: &str,
    root: &str,
    relative: &str,
) -> DriveWebdavResult<DriveEntry> {
    let key = entry_key(space, root, relative)?;
    state
        .metadata
        .get_entry(&key)
        .await?
        .filter(|entry| !entry.deleted)
        .ok_or_else(|| DriveWebdavError::NotFound(key.remote_path()))
}

fn entry_key(space: &str, root: &str, relative: &str) -> DriveWebdavResult<EntryKey> {
    Ok(EntryKey::new(
        space.to_owned(),
        RootAlias::parse(root)?,
        RelativePath::parse(relative)?,
    ))
}

fn destination_key(headers: &HeaderMap) -> DriveWebdavResult<EntryKey> {
    let destination = header_or(headers, "destination", "")?;
    let path = destination
        .split_once("://")
        .and_then(|(_, rest)| rest.split_once('/').map(|(_, path)| format!("/{path}")))
        .unwrap_or(destination);
    let path = path.trim_start_matches('/');
    let parts = path.split('/').collect::<Vec<_>>();
    if parts.len() < 3 || parts[0] != "dav" {
        return Err(DriveWebdavError::InvalidHeader("destination".to_owned()));
    }
    let relative = if parts.len() > 3 {
        parts[3..].join("/")
    } else {
        String::new()
    };
    entry_key(parts[1], parts[2], &relative)
}

fn header_or(headers: &HeaderMap, name: &str, default: &str) -> DriveWebdavResult<String> {
    let Some(value) = headers.get(name) else {
        return Ok(default.to_owned());
    };
    value
        .to_str()
        .map(ToOwned::to_owned)
        .map_err(|err| DriveWebdavError::InvalidHeader(err.to_string()))
}

fn multistatus_xml(space: &str, root: &str, entries: &[DriveEntry]) -> String {
    let mut xml =
        String::from(r#"<?xml version="1.0" encoding="utf-8"?><D:multistatus xmlns:D="DAV:">"#);
    for entry in entries {
        xml.push_str("<D:response><D:href>");
        xml.push_str(&escape_xml(&format!(
            "/dav/{}/{}/{}",
            space,
            root,
            entry.key.relative_path.as_str()
        )));
        xml.push_str("</D:href><D:propstat><D:prop>");
        xml.push_str("<D:resourcetype>");
        if entry.kind == DriveEntryKind::Directory {
            xml.push_str("<D:collection/>");
        }
        xml.push_str("</D:resourcetype>");
        if let Some(hash) = &entry.latest_hash {
            xml.push_str("<D:getetag>");
            xml.push_str(&escape_xml(hash));
            xml.push_str("</D:getetag>");
        }
        xml.push_str("</D:prop><D:status>HTTP/1.1 200 OK</D:status></D:propstat></D:response>");
    }
    xml.push_str("</D:multistatus>");
    xml
}

fn lockdiscovery_xml(owner: &str, token: &str) -> String {
    format!(
        r#"<?xml version="1.0" encoding="utf-8"?><D:prop xmlns:D="DAV:"><D:lockdiscovery><D:activelock><D:locktype><D:write/></D:locktype><D:lockscope><D:exclusive/></D:lockscope><D:owner>{}</D:owner><D:locktoken><D:href>{}</D:href></D:locktoken></D:activelock></D:lockdiscovery></D:prop>"#,
        escape_xml(owner),
        escape_xml(token)
    )
}

fn escape_xml(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

fn xml_response(status: StatusCode, body: String) -> Response {
    let mut headers = HeaderMap::new();
    headers.insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("application/xml; charset=utf-8"),
    );
    (status, headers, body).into_response()
}

fn status_response(status: StatusCode, message: &str) -> Response {
    if message.is_empty() {
        status.into_response()
    } else {
        (status, message.to_owned()).into_response()
    }
}

fn error_response(error: DriveWebdavError) -> Response {
    let status = match &error {
        DriveWebdavError::NotFound(_)
        | DriveWebdavError::Store(DriveStoreError::EntryNotFound(_)) => StatusCode::NOT_FOUND,
        DriveWebdavError::Store(DriveStoreError::LockedByOther { .. }) => StatusCode::LOCKED,
        DriveWebdavError::Core(_) | DriveWebdavError::InvalidHeader(_) => StatusCode::BAD_REQUEST,
        DriveWebdavError::Store(_) => StatusCode::INTERNAL_SERVER_ERROR,
    };
    (status, error.to_string()).into_response()
}

#[cfg(test)]
mod tests {
    use super::destination_key;
    use axum::http::{HeaderMap, HeaderValue};

    #[test]
    fn destination_key_parses_webdav_absolute_url() {
        let mut headers = HeaderMap::new();
        headers.insert(
            "destination",
            HeaderValue::from_static("http://localhost:8788/dav/main/workspace/a.txt"),
        );

        let key = destination_key(&headers).expect("destination should parse");

        assert_eq!(key.remote_path(), "main/workspace/a.txt");
    }
}
