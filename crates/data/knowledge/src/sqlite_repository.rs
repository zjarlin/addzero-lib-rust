use std::{fs, path::PathBuf, str::FromStr, time::Duration};

use anyhow::Context;
use chrono::{DateTime, Utc};
use sqlx::{
    QueryBuilder, Row, Sqlite, SqlitePool,
    sqlite::{SqliteConnectOptions, SqlitePoolOptions},
};
use uuid::Uuid;

use crate::types::{KnowledgeDocument, KnowledgeSourceSpec};

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

#[derive(Clone)]
pub(crate) struct SqliteKnowledgeRepository {
    pool: SqlitePool,
}

impl SqliteKnowledgeRepository {
    pub(crate) async fn connect(database_url: &str) -> anyhow::Result<Self> {
        ensure_sqlite_parent_dir(database_url)?;

        let options = SqliteConnectOptions::from_str(database_url)
            .with_context(|| format!("parse sqlite database url `{database_url}`"))?
            .create_if_missing(true)
            .foreign_keys(true);
        let pool = SqlitePoolOptions::new()
            .max_connections(4)
            .min_connections(1)
            .acquire_timeout(Duration::from_secs(5))
            .connect_with(options)
            .await
            .with_context(|| format!("connect sqlite database `{database_url}`"))?;

        let repository = Self { pool };
        repository.ensure_schema().await?;
        Ok(repository)
    }

    async fn ensure_schema(&self) -> anyhow::Result<()> {
        for statement in SQLITE_SCHEMA {
            sqlx::query(statement)
                .execute(&self.pool)
                .await
                .context("ensure sqlite knowledge schema")?;
        }
        Ok(())
    }

    pub(crate) async fn list_documents(&self) -> anyhow::Result<Vec<KnowledgeDocument>> {
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
        .context("query active sqlite knowledge documents")?;

        rows.into_iter().map(row_to_document).collect()
    }

    pub(crate) async fn upsert_source(
        &self,
        source: &KnowledgeSourceSpec,
    ) -> anyhow::Result<String> {
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
        .with_context(|| format!("upsert sqlite knowledge source `{}`", source.slug))?;

        let row = sqlx::query("SELECT id FROM knowledge_sources WHERE slug = ?1")
            .bind(&source.slug)
            .fetch_one(&self.pool)
            .await
            .with_context(|| format!("load sqlite knowledge source `{}`", source.slug))?;
        row.try_get("id")
            .with_context(|| format!("load sqlite source id for `{}`", source.slug))
    }

    pub(crate) async fn source_root_by_slug(
        &self,
        slug: &str,
    ) -> anyhow::Result<Option<String>> {
        let row = sqlx::query("SELECT root_path FROM knowledge_sources WHERE slug = ?1")
            .bind(slug)
            .fetch_optional(&self.pool)
            .await
            .with_context(|| format!("load sqlite knowledge source root `{slug}`"))?;
        row.map(|row| {
            row.try_get("root_path")
                .with_context(|| format!("decode sqlite knowledge source root `{slug}`"))
        })
        .transpose()
    }

    pub(crate) async fn upsert_document(
        &self,
        source_id: &str,
        doc: &KnowledgeDocument,
    ) -> anyhow::Result<()> {
        let now = doc.updated_at.to_rfc3339();
        let headings_json = serde_json::to_string(&doc.headings)
            .with_context(|| format!("encode sqlite headings for `{}`", doc.source_path))?;
        let tags_json = serde_json::to_string(&doc.tags)
            .with_context(|| format!("encode sqlite tags for `{}`", doc.source_path))?;

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
        .with_context(|| format!("upsert sqlite knowledge document `{}`", doc.source_path))?;

        Ok(())
    }

    pub(crate) async fn deactivate_missing_documents(
        &self,
        source_id: &str,
        active_paths: &[String],
    ) -> anyhow::Result<()> {
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
            .context("deactivate missing sqlite knowledge documents")?;
        Ok(())
    }

    pub(crate) async fn deactivate_document_by_source_path(
        &self,
        source_path: &str,
    ) -> anyhow::Result<()> {
        sqlx::query(
            "UPDATE knowledge_documents SET is_active = 0, updated_at = ?1 WHERE source_path = ?2",
        )
        .bind(Utc::now().to_rfc3339())
        .bind(source_path)
        .execute(&self.pool)
        .await
        .with_context(|| format!("deactivate sqlite knowledge document `{source_path}`"))?;
        Ok(())
    }
}

fn row_to_document(row: sqlx::sqlite::SqliteRow) -> anyhow::Result<KnowledgeDocument> {
    let headings_json: String = row
        .try_get("headings_json")
        .context("load sqlite headings json")?;
    let tags_json: String = row
        .try_get("tags_json")
        .context("load sqlite tags json")?;
    let updated_at_raw: String = row
        .try_get("updated_at")
        .context("load sqlite updated_at")?;
    let updated_at = parse_timestamp(&updated_at_raw)?;

    Ok(KnowledgeDocument {
        source_slug: row.try_get("source_slug").context("load sqlite source_slug")?,
        source_name: row.try_get("source_name").context("load sqlite source_name")?,
        source_root: row.try_get("source_root").context("load sqlite source_root")?,
        slug: row.try_get("slug").context("load sqlite slug")?,
        title: row.try_get("title").context("load sqlite title")?,
        filename: row.try_get("filename").context("load sqlite filename")?,
        source_path: row.try_get("source_path").context("load sqlite source_path")?,
        relative_path: row
            .try_get("relative_path")
            .context("load sqlite relative_path")?,
        bytes: usize::try_from(row.get::<i64, _>("bytes")).unwrap_or_default(),
        section_count: usize::try_from(row.get::<i32, _>("section_count")).unwrap_or_default(),
        preview: row.try_get("preview").context("load sqlite preview")?,
        excerpt: row.try_get("excerpt").context("load sqlite excerpt")?,
        headings: serde_json::from_str(&headings_json)
            .context("decode sqlite headings")?,
        tags: serde_json::from_str(&tags_json)
            .context("decode sqlite tags")?,
        body: row.try_get("body").context("load sqlite body")?,
        content_hash: row
            .try_get("content_hash")
            .context("load sqlite content_hash")?,
        updated_at,
    })
}

fn parse_timestamp(value: &str) -> anyhow::Result<DateTime<Utc>> {
    DateTime::parse_from_rfc3339(value)
        .map(|timestamp| timestamp.with_timezone(&Utc))
        .with_context(|| format!("parse sqlite timestamp `{value}`"))
}

fn ensure_sqlite_parent_dir(database_url: &str) -> anyhow::Result<()> {
    let Some(path) = sqlite_file_path(database_url) else {
        return Ok(());
    };
    let Some(parent) = path.parent() else {
        return Ok(());
    };
    fs::create_dir_all(parent)
        .with_context(|| format!("create sqlite directory `{}`", parent.display()))
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
