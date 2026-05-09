use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use sqlx::Row;
use sqlx::postgres::{PgPool, PgPoolOptions};
use std::time::Duration;
use uuid::Uuid;

use crate::types::{Skill, SkillSource, SkillUpsert};

const SCHEMA_SQL: &str = include_str!("../migrations/0001_init.sql");

/// Postgres repository for the `skills` table.
#[derive(Clone)]
pub struct PgRepo {
    pool: PgPool,
}

impl PgRepo {
    /// Connect to PG with a short timeout. Failure is normal (offline mode);
    /// callers should treat it as "fall back to fs-only" instead of panicking.
    pub async fn connect(database_url: &str) -> Result<Self> {
        let pool = PgPoolOptions::new()
            .max_connections(4)
            .acquire_timeout(Duration::from_secs(3))
            .connect(database_url)
            .await
            .context("connect to postgres")?;
        Ok(Self { pool })
    }

    pub fn pool(&self) -> &PgPool {
        &self.pool
    }

    /// Idempotent schema bootstrap. Safe to call on every startup.
    pub async fn ensure_schema(&self) -> Result<()> {
        sqlx::query(SCHEMA_SQL)
            .execute(&self.pool)
            .await
            .context("apply skills schema")?;
        Ok(())
    }

    pub async fn list(&self) -> Result<Vec<Skill>> {
        let rows = sqlx::query(
            r#"SELECT id, name, keywords, description, body, content_hash, updated_at
               FROM skills
               ORDER BY name"#,
        )
        .fetch_all(&self.pool)
        .await
        .context("list skills")?;
        Ok(rows.into_iter().map(row_to_skill).collect())
    }

    pub async fn get(&self, name: &str) -> Result<Option<Skill>> {
        let row = sqlx::query(
            r#"SELECT id, name, keywords, description, body, content_hash, updated_at
               FROM skills WHERE name = $1"#,
        )
        .bind(name)
        .fetch_optional(&self.pool)
        .await
        .context("get skill")?;
        Ok(row.map(row_to_skill))
    }

    pub async fn delete(&self, name: &str) -> Result<()> {
        sqlx::query("DELETE FROM skills WHERE name = $1")
            .bind(name)
            .execute(&self.pool)
            .await
            .context("delete skill")?;
        Ok(())
    }

    /// Insert or replace by name. The caller decides `updated_at`/`content_hash`
    /// so we don't double-stamp during sync flows.
    pub async fn upsert(
        &self,
        input: &SkillUpsert,
        updated_at: DateTime<Utc>,
        content_hash: &str,
    ) -> Result<Skill> {
        let row = sqlx::query(
            r#"INSERT INTO skills (name, keywords, description, body, content_hash, updated_at)
               VALUES ($1, $2, $3, $4, $5, $6)
               ON CONFLICT (name) DO UPDATE SET
                   keywords = EXCLUDED.keywords,
                   description = EXCLUDED.description,
                   body = EXCLUDED.body,
                   content_hash = EXCLUDED.content_hash,
                   updated_at = EXCLUDED.updated_at
               RETURNING id, name, keywords, description, body, content_hash, updated_at"#,
        )
        .bind(&input.name)
        .bind(&input.keywords)
        .bind(&input.description)
        .bind(&input.body)
        .bind(content_hash)
        .bind(updated_at)
        .fetch_one(&self.pool)
        .await
        .context("upsert skill")?;
        Ok(row_to_skill(row))
    }
}

fn row_to_skill(row: sqlx::postgres::PgRow) -> Skill {
    let id: Uuid = row.try_get("id").unwrap_or_else(|_| Uuid::new_v4());
    let name: String = row.try_get("name").unwrap_or_default();
    let keywords: Vec<String> = row.try_get("keywords").unwrap_or_default();
    let description: String = row.try_get("description").unwrap_or_default();
    let body: String = row.try_get("body").unwrap_or_default();
    let content_hash: String = row.try_get("content_hash").unwrap_or_default();
    let updated_at: DateTime<Utc> = row.try_get("updated_at").unwrap_or_else(|_| Utc::now());
    Skill {
        id,
        name,
        keywords,
        description,
        body,
        content_hash,
        updated_at,
        source: SkillSource::Postgres,
    }
}
