use std::time::Duration;

use chrono::Utc;
use sqlx::{
    Row,
    postgres::{PgPool, PgPoolOptions},
};
use uuid::Uuid;

use crate::types::{KnowledgeDocument, KnowledgeError, KnowledgeSourceSpec};

const SCHEMA_SQL: &str = include_str!("../migrations/0001_init.sql");

#[derive(Clone)]
pub struct PgRepo {
    pool: PgPool,
}

impl PgRepo {
    pub async fn connect(database_url: &str) -> Result<Self, KnowledgeError> {
        let pool = PgPoolOptions::new()
            .max_connections(4)
            .acquire_timeout(Duration::from_secs(5))
            .connect(database_url)
            .await
            .map_err(KnowledgeError::ConnectPostgres)?;
        Ok(Self { pool })
    }

    pub async fn ensure_schema(&self) -> Result<(), KnowledgeError> {
        for statement in SCHEMA_SQL.split(';') {
            let trimmed = statement.trim();
            if trimmed.is_empty() {
                continue;
            }
            sqlx::query(trimmed)
                .execute(&self.pool)
                .await
                .map_err(KnowledgeError::ApplySchema)?;
        }
        Ok(())
    }

    pub async fn list_documents(&self) -> Result<Vec<KnowledgeDocument>, KnowledgeError> {
        let rows = sqlx::query(
            r#"
            SELECT
                s.slug AS source_slug,
                s.name AS source_name,
                s.root_path AS source_root,
                d.slug,
                d.title,
                d.filename,
                d.source_path,
                d.relative_path,
                d.bytes,
                d.section_count,
                d.preview,
                d.excerpt,
                d.headings,
                d.body,
                d.content_hash
            FROM knowledge_documents d
            INNER JOIN knowledge_sources s ON s.id = d.source_id
            WHERE d.is_active = TRUE
            ORDER BY s.name, d.relative_path
            "#,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(KnowledgeError::Query)?;

        Ok(rows.into_iter().map(row_to_document).collect())
    }

    pub async fn upsert_source(
        &self,
        source: &KnowledgeSourceSpec,
    ) -> Result<Uuid, KnowledgeError> {
        let row = sqlx::query(
            r#"
            INSERT INTO knowledge_sources (slug, name, root_path, last_synced_at, updated_at)
            VALUES ($1, $2, $3, $4, $4)
            ON CONFLICT (slug) DO UPDATE SET
                name = EXCLUDED.name,
                root_path = EXCLUDED.root_path,
                last_synced_at = EXCLUDED.last_synced_at,
                updated_at = EXCLUDED.updated_at
            RETURNING id
            "#,
        )
        .bind(&source.slug)
        .bind(&source.name)
        .bind(source.root_path.display().to_string())
        .bind(Utc::now())
        .fetch_one(&self.pool)
        .await
        .map_err(KnowledgeError::Query)?;

        row.try_get("id").map_err(KnowledgeError::Query)
    }

    pub async fn upsert_document(
        &self,
        source_id: Uuid,
        doc: &KnowledgeDocument,
    ) -> Result<(), KnowledgeError> {
        sqlx::query(
            r#"
            INSERT INTO knowledge_documents (
                source_id,
                slug,
                title,
                filename,
                source_path,
                relative_path,
                bytes,
                section_count,
                preview,
                excerpt,
                headings,
                body,
                content_hash,
                is_active,
                updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, TRUE, $14)
            ON CONFLICT (source_path) DO UPDATE SET
                source_id = EXCLUDED.source_id,
                slug = EXCLUDED.slug,
                title = EXCLUDED.title,
                filename = EXCLUDED.filename,
                relative_path = EXCLUDED.relative_path,
                bytes = EXCLUDED.bytes,
                section_count = EXCLUDED.section_count,
                preview = EXCLUDED.preview,
                excerpt = EXCLUDED.excerpt,
                headings = EXCLUDED.headings,
                body = EXCLUDED.body,
                content_hash = EXCLUDED.content_hash,
                is_active = TRUE,
                updated_at = EXCLUDED.updated_at
            "#,
        )
        .bind(source_id)
        .bind(&doc.slug)
        .bind(&doc.title)
        .bind(&doc.filename)
        .bind(&doc.source_path)
        .bind(&doc.relative_path)
        .bind(doc.bytes as i64)
        .bind(doc.section_count as i32)
        .bind(&doc.preview)
        .bind(&doc.excerpt)
        .bind(&doc.headings)
        .bind(&doc.body)
        .bind(&doc.content_hash)
        .bind(Utc::now())
        .execute(&self.pool)
        .await
        .map_err(KnowledgeError::Query)?;
        Ok(())
    }

    pub async fn deactivate_missing_documents(
        &self,
        source_id: Uuid,
        active_paths: &[String],
    ) -> Result<(), KnowledgeError> {
        if active_paths.is_empty() {
            sqlx::query(
                "UPDATE knowledge_documents SET is_active = FALSE, updated_at = $2 WHERE source_id = $1",
            )
            .bind(source_id)
            .bind(Utc::now())
            .execute(&self.pool)
            .await
            .map_err(KnowledgeError::Query)?;
            return Ok(());
        }

        sqlx::query(
            r#"
            UPDATE knowledge_documents
            SET is_active = FALSE, updated_at = $3
            WHERE source_id = $1
              AND source_path <> ALL($2)
            "#,
        )
        .bind(source_id)
        .bind(active_paths)
        .bind(Utc::now())
        .execute(&self.pool)
        .await
        .map_err(KnowledgeError::Query)?;
        Ok(())
    }
}

fn row_to_document(row: sqlx::postgres::PgRow) -> KnowledgeDocument {
    let bytes = row.try_get::<i64, _>("bytes").unwrap_or_default();
    let section_count = row.try_get::<i32, _>("section_count").unwrap_or_default();

    KnowledgeDocument {
        source_slug: row.try_get("source_slug").unwrap_or_default(),
        source_name: row.try_get("source_name").unwrap_or_default(),
        source_root: row.try_get("source_root").unwrap_or_default(),
        slug: row.try_get("slug").unwrap_or_default(),
        title: row.try_get("title").unwrap_or_default(),
        filename: row.try_get("filename").unwrap_or_default(),
        source_path: row.try_get("source_path").unwrap_or_default(),
        relative_path: row.try_get("relative_path").unwrap_or_default(),
        bytes: usize::try_from(bytes).unwrap_or_default(),
        section_count: usize::try_from(section_count).unwrap_or_default(),
        preview: row.try_get("preview").unwrap_or_default(),
        excerpt: row.try_get("excerpt").unwrap_or_default(),
        headings: row.try_get("headings").unwrap_or_default(),
        body: row.try_get("body").unwrap_or_default(),
        content_hash: row.try_get("content_hash").unwrap_or_default(),
    }
}
