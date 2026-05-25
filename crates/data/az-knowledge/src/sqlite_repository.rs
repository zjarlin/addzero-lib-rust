use std::{fs, path::PathBuf, str::FromStr, time::Duration};

use az_derive_aliases::{apply, plain_clone};
use chrono::{DateTime, Utc};
use sqlx::{
    QueryBuilder, Row, Sqlite, SqlitePool,
    sqlite::{SqliteConnectOptions, SqlitePoolOptions},
};
use uuid::Uuid;

use crate::types::{KnowledgeDocument, KnowledgeError, KnowledgeSourceSpec};

const SQLITE_SCHEMA: &[&str] = &[
    r#"
    CREATE TABLE IF NOT EXISTS knowledge_sources (
        id TEXT PRIMARY KEY,
        slug TEXT NOT NULL UNIQUE,
        name TEXT NOT NULL,
        root_path TEXT NOT NULL UNIQUE,
        last_synced_at TEXT,
        created_at TEXT NOT NULL,
        updated_at TEXT NOT NULL
    )
    "#,
    r#"
    CREATE TABLE IF NOT EXISTS knowledge_documents (
        id TEXT PRIMARY KEY,
        source_id TEXT NOT NULL,
        slug TEXT NOT NULL UNIQUE,
        title TEXT NOT NULL,
        filename TEXT NOT NULL,
        source_path TEXT NOT NULL UNIQUE,
        relative_path TEXT NOT NULL,
        bytes INTEGER NOT NULL,
        section_count INTEGER NOT NULL,
        preview TEXT NOT NULL,
        excerpt TEXT NOT NULL,
        headings_json TEXT NOT NULL DEFAULT '[]',
        tags_json TEXT NOT NULL DEFAULT '[]',
        body TEXT NOT NULL,
        content_hash TEXT NOT NULL,
        is_active INTEGER NOT NULL DEFAULT 1,
        created_at TEXT NOT NULL,
        updated_at TEXT NOT NULL,
        FOREIGN KEY(source_id) REFERENCES knowledge_sources(id) ON DELETE CASCADE
    )
    "#,
    "CREATE INDEX IF NOT EXISTS knowledge_documents_source_idx ON knowledge_documents (source_id, is_active)",
    "CREATE INDEX IF NOT EXISTS knowledge_documents_path_idx ON knowledge_documents (source_path)",
    "CREATE INDEX IF NOT EXISTS knowledge_documents_hash_idx ON knowledge_documents (content_hash)",
];

#[apply(plain_clone)]
pub(crate) struct SqliteKnowledgeRepository {
    pool: SqlitePool,
}

impl SqliteKnowledgeRepository {
    pub(crate) async fn connect(database_url: &str) -> Result<Self, KnowledgeError> {
        ensure_sqlite_parent_dir(database_url)?;

        let options = SqliteConnectOptions::from_str(database_url)
            .map_err(|err| KnowledgeError::Message(format!("parse sqlite url: {err}")))?
            .create_if_missing(true)
            .foreign_keys(true);
        let pool = SqlitePoolOptions::new()
            .max_connections(4)
            .min_connections(1)
            .acquire_timeout(Duration::from_secs(5))
            .connect_with(options)
            .await
            .map_err(KnowledgeError::Sqlite)?;

        let repository = Self { pool };
        repository.ensure_schema().await?;
        Ok(repository)
    }

    async fn ensure_schema(&self) -> Result<(), KnowledgeError> {
        for statement in SQLITE_SCHEMA {
            sqlx::query(statement)
                .execute(&self.pool)
                .await
                .map_err(KnowledgeError::Sqlite)?;
        }
        Ok(())
    }

    pub(crate) async fn list_documents(&self) -> Result<Vec<KnowledgeDocument>, KnowledgeError> {
        let rows = sqlx::query(
            r#"
            SELECT
                doc.slug,
                doc.title,
                doc.filename,
                doc.source_path,
                doc.relative_path,
                doc.bytes,
                doc.section_count,
                doc.preview,
                doc.excerpt,
                doc.headings_json,
                doc.tags_json,
                doc.body,
                doc.content_hash,
                doc.updated_at,
                source.slug AS source_slug,
                source.name AS source_name,
                source.root_path AS source_root
            FROM knowledge_documents doc
            JOIN knowledge_sources source ON source.id = doc.source_id
            WHERE doc.is_active = 1
            ORDER BY doc.source_id ASC, doc.relative_path ASC
            "#,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(KnowledgeError::Sqlite)?;

        rows.into_iter().map(row_to_document).collect()
    }

    pub(crate) async fn upsert_source(
        &self,
        source: &KnowledgeSourceSpec,
    ) -> Result<String, KnowledgeError> {
        let now = Utc::now().to_rfc3339();
        let id = Uuid::new_v4().to_string();
        let root_path = source.root_path.display().to_string();

        sqlx::query(
            r#"
            INSERT INTO knowledge_sources (
                id, slug, name, root_path, last_synced_at, created_at, updated_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
            ON CONFLICT(slug) DO UPDATE SET
                name = excluded.name,
                root_path = excluded.root_path,
                last_synced_at = excluded.last_synced_at,
                updated_at = excluded.updated_at
            "#,
        )
        .bind(id)
        .bind(&source.slug)
        .bind(&source.name)
        .bind(root_path)
        .bind(Some(now.clone()))
        .bind(now.clone())
        .bind(now)
        .execute(&self.pool)
        .await
        .map_err(KnowledgeError::Sqlite)?;

        let row = sqlx::query("SELECT id FROM knowledge_sources WHERE slug = ?1")
            .bind(&source.slug)
            .fetch_one(&self.pool)
            .await
            .map_err(KnowledgeError::Sqlite)?;
        row.try_get("id")
            .map_err(|err| KnowledgeError::Message(format!("load sqlite source id: {err}")))
    }

    pub(crate) async fn source_root_by_slug(
        &self,
        slug: &str,
    ) -> Result<Option<String>, KnowledgeError> {
        let row = sqlx::query("SELECT root_path FROM knowledge_sources WHERE slug = ?1")
            .bind(slug)
            .fetch_optional(&self.pool)
            .await
            .map_err(KnowledgeError::Sqlite)?;
        row.map(|row| {
            row.try_get("root_path")
                .map_err(|err| KnowledgeError::Message(format!("load sqlite source root: {err}")))
        })
        .transpose()
    }

    pub(crate) async fn upsert_document(
        &self,
        source_id: &str,
        doc: &KnowledgeDocument,
    ) -> Result<(), KnowledgeError> {
        let now = doc.updated_at.to_rfc3339();
        let headings_json = serde_json::to_string(&doc.headings)
            .map_err(|err| KnowledgeError::Message(format!("encode sqlite headings: {err}")))?;
        let tags_json = serde_json::to_string(&doc.tags)
            .map_err(|err| KnowledgeError::Message(format!("encode sqlite tags: {err}")))?;

        sqlx::query(
            r#"
            INSERT INTO knowledge_documents (
                id, source_id, slug, title, filename, source_path, relative_path,
                bytes, section_count, preview, excerpt, headings_json, tags_json,
                body, content_hash, is_active, created_at, updated_at
            ) VALUES (
                ?1, ?2, ?3, ?4, ?5, ?6, ?7,
                ?8, ?9, ?10, ?11, ?12, ?13,
                ?14, ?15, 1, ?16, ?17
            )
            ON CONFLICT(source_path) DO UPDATE SET
                source_id = excluded.source_id,
                slug = excluded.slug,
                title = excluded.title,
                filename = excluded.filename,
                relative_path = excluded.relative_path,
                bytes = excluded.bytes,
                section_count = excluded.section_count,
                preview = excluded.preview,
                excerpt = excluded.excerpt,
                headings_json = excluded.headings_json,
                tags_json = excluded.tags_json,
                body = excluded.body,
                content_hash = excluded.content_hash,
                is_active = 1,
                updated_at = excluded.updated_at
            "#,
        )
        .bind(Uuid::new_v4().to_string())
        .bind(source_id)
        .bind(&doc.slug)
        .bind(&doc.title)
        .bind(&doc.filename)
        .bind(&doc.source_path)
        .bind(&doc.relative_path)
        .bind(i64::try_from(doc.bytes).unwrap_or_default())
        .bind(i32::try_from(doc.section_count).unwrap_or_default())
        .bind(&doc.preview)
        .bind(&doc.excerpt)
        .bind(headings_json)
        .bind(tags_json)
        .bind(&doc.body)
        .bind(&doc.content_hash)
        .bind(now.clone())
        .bind(now)
        .execute(&self.pool)
        .await
        .map_err(KnowledgeError::Sqlite)?;

        Ok(())
    }

    pub(crate) async fn deactivate_missing_documents(
        &self,
        source_id: &str,
        active_paths: &[String],
    ) -> Result<(), KnowledgeError> {
        let mut builder = QueryBuilder::<Sqlite>::new(
            "UPDATE knowledge_documents SET is_active = 0, updated_at = ",
        );
        builder.push_bind(Utc::now().to_rfc3339());
        builder.push(" WHERE source_id = ");
        builder.push_bind(source_id);

        if !active_paths.is_empty() {
            builder.push(" AND source_path NOT IN (");
            let mut separated = builder.separated(", ");
            for path in active_paths {
                separated.push_bind(path);
            }
            separated.push_unseparated(")");
        }

        builder
            .build()
            .execute(&self.pool)
            .await
            .map_err(KnowledgeError::Sqlite)?;
        Ok(())
    }

    pub(crate) async fn deactivate_document_by_source_path(
        &self,
        source_path: &str,
    ) -> Result<(), KnowledgeError> {
        sqlx::query(
            "UPDATE knowledge_documents SET is_active = 0, updated_at = ?1 WHERE source_path = ?2",
        )
        .bind(Utc::now().to_rfc3339())
        .bind(source_path)
        .execute(&self.pool)
        .await
        .map_err(KnowledgeError::Sqlite)?;
        Ok(())
    }
}

fn row_to_document(row: sqlx::sqlite::SqliteRow) -> Result<KnowledgeDocument, KnowledgeError> {
    let headings_json: String = row
        .try_get("headings_json")
        .map_err(|err| KnowledgeError::Message(format!("load sqlite headings json: {err}")))?;
    let tags_json: String = row
        .try_get("tags_json")
        .map_err(|err| KnowledgeError::Message(format!("load sqlite tags json: {err}")))?;
    let updated_at_raw: String = row
        .try_get("updated_at")
        .map_err(|err| KnowledgeError::Message(format!("load sqlite updated_at: {err}")))?;
    let updated_at = parse_timestamp(&updated_at_raw)?;

    Ok(KnowledgeDocument {
        source_slug: row
            .try_get("source_slug")
            .map_err(|err| KnowledgeError::Message(format!("load sqlite source_slug: {err}")))?,
        source_name: row
            .try_get("source_name")
            .map_err(|err| KnowledgeError::Message(format!("load sqlite source_name: {err}")))?,
        source_root: row
            .try_get("source_root")
            .map_err(|err| KnowledgeError::Message(format!("load sqlite source_root: {err}")))?,
        slug: row
            .try_get("slug")
            .map_err(|err| KnowledgeError::Message(format!("load sqlite slug: {err}")))?,
        title: row
            .try_get("title")
            .map_err(|err| KnowledgeError::Message(format!("load sqlite title: {err}")))?,
        filename: row
            .try_get("filename")
            .map_err(|err| KnowledgeError::Message(format!("load sqlite filename: {err}")))?,
        source_path: row
            .try_get("source_path")
            .map_err(|err| KnowledgeError::Message(format!("load sqlite source_path: {err}")))?,
        relative_path: row
            .try_get("relative_path")
            .map_err(|err| KnowledgeError::Message(format!("load sqlite relative_path: {err}")))?,
        bytes: usize::try_from(row.get::<i64, _>("bytes")).unwrap_or_default(),
        section_count: usize::try_from(row.get::<i32, _>("section_count")).unwrap_or_default(),
        preview: row
            .try_get("preview")
            .map_err(|err| KnowledgeError::Message(format!("load sqlite preview: {err}")))?,
        excerpt: row
            .try_get("excerpt")
            .map_err(|err| KnowledgeError::Message(format!("load sqlite excerpt: {err}")))?,
        headings: serde_json::from_str(&headings_json)
            .map_err(|err| KnowledgeError::Message(format!("decode sqlite headings: {err}")))?,
        tags: serde_json::from_str(&tags_json)
            .map_err(|err| KnowledgeError::Message(format!("decode sqlite tags: {err}")))?,
        body: row
            .try_get("body")
            .map_err(|err| KnowledgeError::Message(format!("load sqlite body: {err}")))?,
        content_hash: row
            .try_get("content_hash")
            .map_err(|err| KnowledgeError::Message(format!("load sqlite content_hash: {err}")))?,
        updated_at,
    })
}

fn parse_timestamp(value: &str) -> Result<DateTime<Utc>, KnowledgeError> {
    DateTime::parse_from_rfc3339(value)
        .map(|timestamp| timestamp.with_timezone(&Utc))
        .map_err(|err| KnowledgeError::Message(format!("parse sqlite timestamp: {err}")))
}

fn ensure_sqlite_parent_dir(database_url: &str) -> Result<(), KnowledgeError> {
    let Some(path) = sqlite_file_path(database_url) else {
        return Ok(());
    };
    let Some(parent) = path.parent() else {
        return Ok(());
    };
    fs::create_dir_all(parent).map_err(|err| {
        KnowledgeError::Message(format!(
            "create sqlite directory {}: {err}",
            parent.display()
        ))
    })
}

fn sqlite_file_path(database_url: &str) -> Option<PathBuf> {
    if database_url == "sqlite::memory:" {
        return None;
    }

    database_url
        .strip_prefix("sqlite://")
        .or_else(|| database_url.strip_prefix("sqlite:"))
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
}
