#![cfg(not(target_arch = "wasm32"))]

use std::collections::BTreeMap;

use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};
use sqlx::{Executor, PgPool, Row};

use crate::{
    contracts::{SyncFileListItem, SyncFilesResponse},
    sync_index::SyncIndexedFileKind,
    sync_model::{
        SyncCrdtEnvelope, SyncDeviceInfo, SyncDocumentRecord, SyncFileStatus, SyncRoot,
        normalize_home_relative_path,
    },
};

pub const SYNC_SERVER_SCHEMA_SQL: &str = include_str!("../migrations/0001_sync_server.sql");
pub const DEFAULT_OBJECT_CHUNK_BYTES: u64 = 8 * 1024 * 1024;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct SyncServerDeviceRecord {
    pub device_name: String,
    pub home_dir: String,
    pub os: String,
    pub arch: String,
}

impl From<&SyncDeviceInfo> for SyncServerDeviceRecord {
    fn from(device: &SyncDeviceInfo) -> Self {
        Self {
            device_name: device.device_name.clone(),
            home_dir: device.home_dir.to_string_lossy().to_string(),
            os: device.os.clone(),
            arch: device.arch.clone(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct SyncServerRootRecord {
    pub device_name: String,
    pub alias: String,
    pub relative_path: String,
    pub space_id: String,
}

impl SyncServerRootRecord {
    pub fn from_root(device_name: impl Into<String>, root: &SyncRoot) -> Self {
        Self {
            device_name: device_name.into(),
            alias: root.alias.clone(),
            relative_path: root.relative_path.clone(),
            space_id: root.space_id.clone(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct SyncServerFileRecord {
    pub space_id: String,
    pub relative_path: String,
    pub file_kind: SyncIndexedFileKind,
    pub content_hash: String,
    pub crdt_version: Vec<u8>,
    pub status: SyncFileStatus,
    pub size_bytes: Option<u64>,
    pub updated_by_device: String,
}

impl SyncServerFileRecord {
    pub fn from_document(space_id: impl Into<String>, document: &SyncDocumentRecord) -> Self {
        Self {
            space_id: space_id.into(),
            relative_path: document.relative_path.clone(),
            file_kind: SyncIndexedFileKind::Text,
            content_hash: document.content_hash.clone(),
            crdt_version: document.crdt_version.clone(),
            status: document.status,
            size_bytes: None,
            updated_by_device: document.device_name.clone(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct SyncServerUpdateRecord {
    pub space_id: String,
    pub relative_path: String,
    pub source_device: String,
    pub base_version: Option<Vec<u8>>,
    pub version: Vec<u8>,
    pub blob: Vec<u8>,
}

impl SyncServerUpdateRecord {
    pub fn from_envelope(
        space_id: impl Into<String>,
        envelope: SyncCrdtEnvelope,
    ) -> Result<Self> {
        Ok(Self {
            space_id: space_id.into(),
            relative_path: normalize_home_relative_path(&envelope.relative_path)?,
            source_device: envelope.source_device,
            base_version: envelope.base_version,
            version: envelope.version,
            blob: envelope.blob,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct SyncObjectManifest {
    pub space_id: String,
    pub relative_path: String,
    pub content_hash: String,
    pub size_bytes: u64,
    pub chunk_size_bytes: u64,
    pub chunks: Vec<SyncObjectChunk>,
}

impl SyncObjectManifest {
    pub fn plan(
        space_id: impl Into<String>,
        relative_path: &str,
        content_hash: impl Into<String>,
        size_bytes: u64,
        chunk_size_bytes: u64,
    ) -> Result<Self> {
        let relative_path = normalize_home_relative_path(relative_path)?;
        let chunk_size_bytes = chunk_size_bytes.max(1);
        let chunk_count = size_bytes.div_ceil(chunk_size_bytes).max(1);
        let mut chunks = Vec::new();
        for chunk_index in 0..chunk_count {
            let offset = chunk_index * chunk_size_bytes;
            let remaining = size_bytes.saturating_sub(offset);
            chunks.push(SyncObjectChunk {
                chunk_index,
                offset,
                size_bytes: remaining.min(chunk_size_bytes),
                object_key: object_key_for_chunk(&relative_path, chunk_index),
            });
        }
        Ok(Self {
            space_id: space_id.into(),
            relative_path,
            content_hash: content_hash.into(),
            size_bytes,
            chunk_size_bytes,
            chunks,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct SyncObjectChunk {
    pub chunk_index: u64,
    pub offset: u64,
    pub size_bytes: u64,
    pub object_key: String,
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct SyncServerSnapshot {
    pub devices: Vec<SyncServerDeviceRecord>,
    pub roots: Vec<SyncServerRootRecord>,
    pub files: Vec<SyncServerFileRecord>,
    pub update_count: usize,
    pub object_count: usize,
    pub session_count: usize,
}

#[derive(Clone)]
pub struct SyncPgRepository {
    pool: PgPool,
}

impl SyncPgRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn migrate(&self) -> Result<()> {
        self.pool
            .execute(SYNC_SERVER_SCHEMA_SQL)
            .await
            .context("PostgreSQL sync repository failed")?;
        Ok(())
    }

    pub async fn register_device(&self, device: &SyncDeviceInfo) -> Result<()> {
        let record = SyncServerDeviceRecord::from(device);
        sqlx::query(
            r#"
            INSERT INTO sys_sync_device (device_name, home_dir, os, arch, last_seen_at)
            VALUES ($1, $2, $3, $4, now())
            ON CONFLICT (device_name) DO UPDATE SET
                home_dir = EXCLUDED.home_dir,
                os = EXCLUDED.os,
                arch = EXCLUDED.arch,
                last_seen_at = now()
            "#,
        )
        .bind(&record.device_name)
        .bind(&record.home_dir)
        .bind(&record.os)
        .bind(&record.arch)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn upsert_root(&self, record: &SyncServerRootRecord) -> Result<()> {
        normalize_home_relative_path(&record.relative_path)?;
        sqlx::query(
            r#"
            INSERT INTO biz_sync_root (device_name, alias, relative_path, space_id, updated_at)
            VALUES ($1, $2, $3, $4, now())
            ON CONFLICT (device_name, alias) DO UPDATE SET
                relative_path = EXCLUDED.relative_path,
                space_id = EXCLUDED.space_id,
                updated_at = now()
            "#,
        )
        .bind(&record.device_name)
        .bind(&record.alias)
        .bind(&record.relative_path)
        .bind(&record.space_id)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn upsert_file(&self, record: &SyncServerFileRecord) -> Result<()> {
        normalize_home_relative_path(&record.relative_path)?;
        let file_kind = file_kind_to_db(record.file_kind);
        let status = file_status_to_db(record.status);
        sqlx::query(
            r#"
            INSERT INTO biz_sync_file_record (
                space_id,
                relative_path,
                file_kind,
                content_hash,
                crdt_version,
                status,
                size_bytes,
                updated_by_device,
                updated_at,
                deleted_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, now(), CASE WHEN $6 = 'deleted' THEN now() ELSE NULL END)
            ON CONFLICT (space_id, relative_path) DO UPDATE SET
                file_kind = EXCLUDED.file_kind,
                content_hash = EXCLUDED.content_hash,
                crdt_version = EXCLUDED.crdt_version,
                status = EXCLUDED.status,
                size_bytes = EXCLUDED.size_bytes,
                updated_by_device = EXCLUDED.updated_by_device,
                updated_at = now(),
                deleted_at = CASE WHEN EXCLUDED.status = 'deleted' THEN now() ELSE NULL END
            "#,
        )
        .bind(&record.space_id)
        .bind(&record.relative_path)
        .bind(file_kind)
        .bind(&record.content_hash)
        .bind(&record.crdt_version)
        .bind(status)
        .bind(record.size_bytes.and_then(|value| i64::try_from(value).ok()))
        .bind(&record.updated_by_device)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn list_files_page(
        &self,
        space_id: &str,
        cursor: Option<&str>,
        limit: usize,
    ) -> Result<SyncFilesResponse> {
        if let Some(cursor) = cursor {
            normalize_home_relative_path(cursor)?;
        }
        let effective_limit = limit.max(1);
        let query_limit = i64::try_from(effective_limit.saturating_add(1)).unwrap_or(i64::MAX);
        let rows = sqlx::query(
            r#"
            SELECT
                space_id,
                relative_path,
                file_kind,
                content_hash,
                crdt_version,
                status,
                size_bytes,
                updated_by_device
            FROM biz_sync_file_record
            WHERE space_id = $1
              AND ($2::text IS NULL OR relative_path > $2)
            ORDER BY relative_path ASC
            LIMIT $3
            "#,
        )
        .bind(space_id)
        .bind(cursor)
        .bind(query_limit)
        .fetch_all(&self.pool)
        .await?;
        Ok(files_response_from_items(
            space_id.to_string(),
            rows.into_iter()
                .map(sync_file_list_item_from_row)
                .collect::<Result<Vec<_>>>()?,
            effective_limit,
        ))
    }

    pub async fn append_update(&self, record: &SyncServerUpdateRecord) -> Result<()> {
        normalize_home_relative_path(&record.relative_path)?;
        sqlx::query(
            r#"
            INSERT INTO biz_sync_crdt_update_log (
                space_id,
                relative_path,
                source_device,
                base_version,
                version,
                blob
            )
            VALUES ($1, $2, $3, $4, $5, $6)
            "#,
        )
        .bind(&record.space_id)
        .bind(&record.relative_path)
        .bind(&record.source_device)
        .bind(&record.base_version)
        .bind(&record.version)
        .bind(&record.blob)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn upsert_object_manifest(&self, manifest: &SyncObjectManifest) -> Result<()> {
        normalize_home_relative_path(&manifest.relative_path)?;
        let size_bytes = i64::try_from(manifest.size_bytes).unwrap_or(i64::MAX);
        let chunk_size_bytes = i64::try_from(manifest.chunk_size_bytes).unwrap_or(i64::MAX);
        let mut tx = self.pool.begin().await?;
        sqlx::query(
            r#"
            INSERT INTO biz_sync_object_metadata (
                space_id,
                relative_path,
                content_hash,
                size_bytes,
                chunk_size_bytes,
                updated_at
            )
            VALUES ($1, $2, $3, $4, $5, now())
            ON CONFLICT (space_id, relative_path, content_hash) DO UPDATE SET
                size_bytes = EXCLUDED.size_bytes,
                chunk_size_bytes = EXCLUDED.chunk_size_bytes,
                updated_at = now()
            "#,
        )
        .bind(&manifest.space_id)
        .bind(&manifest.relative_path)
        .bind(&manifest.content_hash)
        .bind(size_bytes)
        .bind(chunk_size_bytes)
        .execute(&mut *tx)
        .await?;
        sqlx::query(
            r#"
            DELETE FROM biz_sync_object_chunk
            WHERE space_id = $1 AND relative_path = $2 AND content_hash = $3
            "#,
        )
        .bind(&manifest.space_id)
        .bind(&manifest.relative_path)
        .bind(&manifest.content_hash)
        .execute(&mut *tx)
        .await?;
        for chunk in &manifest.chunks {
            sqlx::query(
                r#"
                INSERT INTO biz_sync_object_chunk (
                    space_id,
                    relative_path,
                    content_hash,
                    chunk_index,
                    offset_bytes,
                    size_bytes,
                    object_key
                )
                VALUES ($1, $2, $3, $4, $5, $6, $7)
                "#,
            )
            .bind(&manifest.space_id)
            .bind(&manifest.relative_path)
            .bind(&manifest.content_hash)
            .bind(i64::try_from(chunk.chunk_index).unwrap_or(i64::MAX))
            .bind(i64::try_from(chunk.offset).unwrap_or(i64::MAX))
            .bind(i64::try_from(chunk.size_bytes).unwrap_or(i64::MAX))
            .bind(&chunk.object_key)
            .execute(&mut *tx)
            .await?;
        }
        tx.commit().await?;
        Ok(())
    }
}

#[derive(Default)]
pub struct InMemorySyncServerRepository {
    devices: BTreeMap<String, SyncServerDeviceRecord>,
    roots: BTreeMap<(String, String), SyncServerRootRecord>,
    files: BTreeMap<(String, String), SyncServerFileRecord>,
    updates: Vec<SyncServerUpdateRecord>,
    objects: BTreeMap<(String, String), SyncObjectManifest>,
    sessions: BTreeMap<String, String>,
}

impl InMemorySyncServerRepository {
    pub fn register_device(&mut self, device: &SyncDeviceInfo) {
        let record = SyncServerDeviceRecord::from(device);
        self.devices.insert(record.device_name.clone(), record);
    }

    pub fn upsert_root(&mut self, record: SyncServerRootRecord) -> Result<()> {
        normalize_home_relative_path(&record.relative_path)?;
        self.roots
            .insert((record.device_name.clone(), record.alias.clone()), record);
        Ok(())
    }

    pub fn upsert_file(&mut self, record: SyncServerFileRecord) -> Result<()> {
        normalize_home_relative_path(&record.relative_path)?;
        self.files.insert(
            (record.space_id.clone(), record.relative_path.clone()),
            record,
        );
        Ok(())
    }

    pub fn list_files_page(
        &self,
        space_id: &str,
        cursor: Option<&str>,
        limit: usize,
    ) -> Result<SyncFilesResponse> {
        if let Some(cursor) = cursor {
            normalize_home_relative_path(cursor)?;
        }
        let effective_limit = limit.max(1);
        let items = self
            .files
            .values()
            .filter(|record| record.space_id == space_id)
            .filter(|record| {
                cursor
                    .map(|cursor| record.relative_path.as_str() > cursor)
                    .unwrap_or(true)
            })
            .take(effective_limit.saturating_add(1))
            .cloned()
            .map(SyncFileListItem::from)
            .collect::<Vec<_>>();
        Ok(files_response_from_items(
            space_id.to_string(),
            items,
            effective_limit,
        ))
    }

    pub fn append_update(&mut self, record: SyncServerUpdateRecord) -> Result<()> {
        normalize_home_relative_path(&record.relative_path)?;
        self.updates.push(record);
        Ok(())
    }

    pub fn upsert_object_manifest(&mut self, manifest: SyncObjectManifest) -> Result<()> {
        normalize_home_relative_path(&manifest.relative_path)?;
        self.objects.insert(
            (manifest.space_id.clone(), manifest.relative_path.clone()),
            manifest,
        );
        Ok(())
    }

    pub fn open_session(&mut self, session_id: impl Into<String>, device_name: impl Into<String>) {
        self.sessions.insert(session_id.into(), device_name.into());
    }

    pub fn snapshot(&self) -> SyncServerSnapshot {
        SyncServerSnapshot {
            devices: self.devices.values().cloned().collect(),
            roots: self.roots.values().cloned().collect(),
            files: self.files.values().cloned().collect(),
            update_count: self.updates.len(),
            object_count: self.objects.len(),
            session_count: self.sessions.len(),
        }
    }
}

fn object_key_for_chunk(relative_path: &str, chunk_index: u64) -> String {
    format!("sync/{relative_path}/{chunk_index:016x}.chunk")
}

fn files_response_from_items(
    space_id: String,
    mut files: Vec<SyncFileListItem>,
    limit: usize,
) -> SyncFilesResponse {
    let next_cursor = if files.len() > limit {
        files.pop();
        files.last().map(|file| file.relative_path.clone())
    } else {
        None
    };
    SyncFilesResponse {
        space_id,
        files,
        next_cursor,
        limit,
    }
}

fn file_kind_to_db(value: SyncIndexedFileKind) -> &'static str {
    match value {
        SyncIndexedFileKind::Text => "text",
        SyncIndexedFileKind::Binary => "binary",
        SyncIndexedFileKind::Directory => "directory",
        SyncIndexedFileKind::Missing => "missing",
    }
}

fn file_status_to_db(value: SyncFileStatus) -> &'static str {
    match value {
        SyncFileStatus::Synced => "synced",
        SyncFileStatus::Syncing => "syncing",
        SyncFileStatus::Error => "error",
        SyncFileStatus::Shared => "shared",
        SyncFileStatus::Deleted => "deleted",
    }
}

fn file_kind_from_db(value: &str) -> Result<SyncIndexedFileKind> {
    match value {
        "text" => Ok(SyncIndexedFileKind::Text),
        "binary" => Ok(SyncIndexedFileKind::Binary),
        "directory" => Ok(SyncIndexedFileKind::Directory),
        "missing" => Ok(SyncIndexedFileKind::Missing),
        _ => bail!("invalid sync file kind `{value}`"),
    }
}

fn file_status_from_db(value: &str) -> Result<SyncFileStatus> {
    match value {
        "synced" => Ok(SyncFileStatus::Synced),
        "syncing" => Ok(SyncFileStatus::Syncing),
        "error" => Ok(SyncFileStatus::Error),
        "shared" => Ok(SyncFileStatus::Shared),
        "deleted" => Ok(SyncFileStatus::Deleted),
        _ => bail!("invalid sync file status `{value}`"),
    }
}

fn sync_file_list_item_from_row(row: sqlx::postgres::PgRow) -> Result<SyncFileListItem> {
    let relative_path: String = row.try_get("relative_path")?;
    let file_kind: String = row.try_get("file_kind")?;
    let status: String = row.try_get("status")?;
    let size_bytes: Option<i64> = row.try_get("size_bytes")?;
    Ok(SyncFileListItem {
        space_id: row.try_get("space_id")?,
        relative_path: normalize_home_relative_path(&relative_path)?,
        file_kind: file_kind_from_db(&file_kind)?,
        content_hash: row.try_get("content_hash")?,
        crdt_version: row.try_get("crdt_version")?,
        status: file_status_from_db(&status)?,
        size_bytes: size_bytes.and_then(|size| u64::try_from(size).ok()),
        updated_by_device: row.try_get("updated_by_device")?,
    })
}

impl From<SyncServerFileRecord> for SyncFileListItem {
    fn from(value: SyncServerFileRecord) -> Self {
        Self {
            space_id: value.space_id,
            relative_path: value.relative_path,
            file_kind: value.file_kind,
            content_hash: value.content_hash,
            crdt_version: value.crdt_version,
            status: value.status,
            size_bytes: value.size_bytes,
            updated_by_device: value.updated_by_device,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use crate::{
        sync_index::SyncIndexedFileKind,
        sync_model::{SyncBlobKind, SyncCrdtEnvelope, SyncDeviceInfo, SyncFileStatus, SyncRoot},
        sync_server::{
            DEFAULT_OBJECT_CHUNK_BYTES, InMemorySyncServerRepository, SYNC_SERVER_SCHEMA_SQL,
            SyncObjectManifest, SyncServerFileRecord, SyncServerRootRecord, SyncServerUpdateRecord,
        },
    };

    #[test]
    fn schema_declares_required_pg_tables() {
        for table in [
            "sys_sync_device",
            "biz_sync_root",
            "biz_sync_file_record",
            "biz_sync_crdt_update_log",
            "biz_sync_object_metadata",
            "biz_sync_object_chunk",
            "sys_sync_session",
        ] {
            assert!(SYNC_SERVER_SCHEMA_SQL.contains(table), "missing {table}");
        }
    }

    #[test]
    fn pg_repository_sql_declares_upsert_and_append_boundaries() {
        let source = include_str!("sync_server.rs");
        for sql_boundary in [
            "ON CONFLICT (device_name)",
            "ON CONFLICT (device_name, alias)",
            "ON CONFLICT (space_id, relative_path)",
            "AND ($2::text IS NULL OR relative_path > $2)",
            "ORDER BY relative_path ASC",
            "LIMIT $3",
            "INSERT INTO biz_sync_crdt_update_log",
            "INSERT INTO biz_sync_object_metadata",
            "INSERT INTO biz_sync_object_chunk",
        ] {
            assert!(
                source.contains(sql_boundary),
                "missing SQL boundary {sql_boundary}"
            );
        }
    }

    #[test]
    fn object_manifest_chunks_large_files_without_single_blob()
    -> Result<(), Box<dyn std::error::Error>> {
        let manifest = SyncObjectManifest::plan(
            "main",
            "az-sync/video.bin",
            "sha256:demo",
            DEFAULT_OBJECT_CHUNK_BYTES * 2 + 7,
            DEFAULT_OBJECT_CHUNK_BYTES,
        )?;

        assert_eq!(manifest.chunks.len(), 3);
        assert_eq!(manifest.chunks[2].size_bytes, 7);
        Ok(())
    }

    #[test]
    fn server_repository_tracks_devices_roots_files_updates_objects_and_sessions()
    -> Result<(), Box<dyn std::error::Error>> {
        let device = SyncDeviceInfo::new("mac-a", PathBuf::from("/Users/a"));
        let root = SyncRoot::default_for_device(&device);
        let mut repo = InMemorySyncServerRepository::default();

        repo.register_device(&device);
        repo.upsert_root(SyncServerRootRecord::from_root(&device.device_name, &root))?;
        repo.upsert_file(SyncServerFileRecord {
            space_id: "main".to_string(),
            relative_path: "az-sync/a.txt".to_string(),
            file_kind: SyncIndexedFileKind::Text,
            content_hash: "fnv1a64:1".to_string(),
            crdt_version: vec![1],
            status: SyncFileStatus::Synced,
            size_bytes: Some(5),
            updated_by_device: device.device_name.clone(),
        })?;
        repo.append_update(SyncServerUpdateRecord::from_envelope(
            "main",
            SyncCrdtEnvelope {
                relative_path: "az-sync/a.txt".to_string(),
                source_device: device.device_name.clone(),
                base_version: None,
                version: vec![1],
                kind: SyncBlobKind::Update,
                blob: vec![1, 2, 3],
                content_hash: "fnv1a64:1".to_string(),
            },
        )?)?;
        repo.upsert_object_manifest(SyncObjectManifest::plan(
            "main",
            "az-sync/a.bin",
            "sha256:bin",
            9,
            4,
        )?)?;
        repo.open_session("session-a", &device.device_name);

        let snapshot = repo.snapshot();
        assert_eq!(snapshot.devices.len(), 1);
        assert_eq!(snapshot.roots.len(), 1);
        assert_eq!(snapshot.files.len(), 1);
        assert_eq!(snapshot.update_count, 1);
        assert_eq!(snapshot.object_count, 1);
        assert_eq!(snapshot.session_count, 1);
        Ok(())
    }

    #[test]
    fn server_repository_lists_files_by_cursor_page() -> Result<(), Box<dyn std::error::Error>> {
        let mut repo = InMemorySyncServerRepository::default();
        for relative_path in ["az-sync/a.txt", "az-sync/b.txt", "az-sync/c.txt"] {
            repo.upsert_file(SyncServerFileRecord {
                space_id: "main".to_string(),
                relative_path: relative_path.to_string(),
                file_kind: SyncIndexedFileKind::Text,
                content_hash: format!("hash:{relative_path}"),
                crdt_version: vec![1],
                status: SyncFileStatus::Synced,
                size_bytes: Some(5),
                updated_by_device: "mac-a".to_string(),
            })?;
        }

        let first_page = repo.list_files_page("main", None, 2)?;
        assert_eq!(
            first_page
                .files
                .iter()
                .map(|file| file.relative_path.as_str())
                .collect::<Vec<_>>(),
            vec!["az-sync/a.txt", "az-sync/b.txt"]
        );
        assert_eq!(first_page.next_cursor.as_deref(), Some("az-sync/b.txt"));

        let second_page = repo.list_files_page("main", first_page.next_cursor.as_deref(), 2)?;
        assert_eq!(
            second_page
                .files
                .iter()
                .map(|file| file.relative_path.as_str())
                .collect::<Vec<_>>(),
            vec!["az-sync/c.txt"]
        );
        assert_eq!(second_page.next_cursor, None);
        Ok(())
    }

    #[test]
    fn server_repository_rejects_home_escape_paths() {
        let mut repo = InMemorySyncServerRepository::default();

        let error = repo
            .upsert_file(SyncServerFileRecord {
                space_id: "main".to_string(),
                relative_path: "../secret".to_string(),
                file_kind: SyncIndexedFileKind::Text,
                content_hash: "hash".to_string(),
                crdt_version: Vec::new(),
                status: SyncFileStatus::Synced,
                size_bytes: None,
                updated_by_device: "mac-a".to_string(),
            })
            .unwrap_err();
        assert!(error.to_string().contains("invalid sync relative path"));
    }
}
