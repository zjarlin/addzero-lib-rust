//! CRUD repository trait + PG implementation for lowcode layouts.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::schema::LayoutSchema;

// ---------------------------------------------------------------------------
// Errors
// ---------------------------------------------------------------------------

/// Errors from layout repository operations.
#[derive(Debug, thiserror::Error)]
pub enum RepoError {
    #[error("layout not found: {0}")]
    NotFound(Uuid),
    #[error("database error: {0}")]
    Database(#[from] sqlx::Error),
}

// ---------------------------------------------------------------------------
// Persisted record — the row-level shape stored in PG `lc_layout`
// ---------------------------------------------------------------------------

/// A layout row as stored in / read from PostgreSQL.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LayoutRecord {
    pub id: Uuid,
    pub name: String,
    pub schema: LayoutSchema,
    pub version: i32,
    pub created_at: String,
    pub updated_at: String,
}

// ---------------------------------------------------------------------------
// Repository trait
// ---------------------------------------------------------------------------

/// CRUD operations for lowcode layouts.
#[async_trait]
pub trait LayoutRepository: Send + Sync {
    /// Insert a new layout and return the created record.
    async fn create(&self, name: &str, schema: &LayoutSchema) -> Result<LayoutRecord, RepoError>;

    /// Get a layout by id.
    async fn get(&self, id: Uuid) -> Result<LayoutRecord, RepoError>;

    /// List all layouts.
    async fn list(&self) -> Result<Vec<LayoutRecord>, RepoError>;

    /// Update layout name and/or schema. Returns the updated record.
    async fn update(
        &self,
        id: Uuid,
        name: &str,
        schema: &LayoutSchema,
    ) -> Result<LayoutRecord, RepoError>;

    /// Delete a layout by id.
    async fn delete(&self, id: Uuid) -> Result<(), RepoError>;
}

// ---------------------------------------------------------------------------
// PG implementation (queries will be wired up once PG is connected)
// ---------------------------------------------------------------------------

/// PostgreSQL-backed layout repository.
///
/// Query bodies are `todo!()` stubs — they will be filled in when the PG
/// connection is wired up. The trait signatures are final.
pub struct PgLayoutRepo {
    _pool: sqlx::PgPool,
}

impl PgLayoutRepo {
    pub fn new(pool: sqlx::PgPool) -> Self {
        Self { _pool: pool }
    }
}

#[async_trait]
impl LayoutRepository for PgLayoutRepo {
    async fn create(&self, _name: &str, _schema: &LayoutSchema) -> Result<LayoutRecord, RepoError> {
        todo!("PG create layout — will be wired to real queries")
    }

    async fn get(&self, _id: Uuid) -> Result<LayoutRecord, RepoError> {
        todo!("PG get layout — will be wired to real queries")
    }

    async fn list(&self) -> Result<Vec<LayoutRecord>, RepoError> {
        todo!("PG list layouts — will be wired to real queries")
    }

    async fn update(
        &self,
        _id: Uuid,
        _name: &str,
        _schema: &LayoutSchema,
    ) -> Result<LayoutRecord, RepoError> {
        todo!("PG update layout — will be wired to real queries")
    }

    async fn delete(&self, _id: Uuid) -> Result<(), RepoError> {
        todo!("PG delete layout — will be wired to real queries")
    }
}
