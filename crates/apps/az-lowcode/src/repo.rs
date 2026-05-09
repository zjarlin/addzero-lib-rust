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
    /// The referenced layout does not exist.
    #[error("layout not found: {0}")]
    NotFound(Uuid),
    /// Underlying database error.
    #[error("database error: {0}")]
    Database(#[from] sqlx::Error),
}

// ---------------------------------------------------------------------------
// Persisted record — the row-level shape stored in PG `lc_layout`
// ---------------------------------------------------------------------------

/// A layout row as stored in / read from PostgreSQL.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LayoutRecord {
    /// Primary key.
    pub id: Uuid,
    /// Human-readable layout name.
    pub name: String,
    /// The deserialized layout schema.
    pub schema: LayoutSchema,
    /// Optimistic locking version.
    pub version: i32,
    /// ISO-8601 creation timestamp.
    pub created_at: String,
    /// ISO-8601 last-update timestamp.
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
/// The trait signatures are final.  Query bodies return a placeholder
/// `RepoError::Database` until real SQL queries are wired up.
pub struct PgLayoutRepo {
    _pool: sqlx::PgPool,
}

impl PgLayoutRepo {
    /// Creates a new repository wrapping the given PG connection pool.
    pub fn new(pool: sqlx::PgPool) -> Self {
        Self { _pool: pool }
    }
}

#[async_trait]
impl LayoutRepository for PgLayoutRepo {
    async fn create(&self, _name: &str, _schema: &LayoutSchema) -> Result<LayoutRecord, RepoError> {
        Err(RepoError::Database(sqlx::Error::RowNotFound))
    }

    async fn get(&self, id: Uuid) -> Result<LayoutRecord, RepoError> {
        Err(RepoError::NotFound(id))
    }

    async fn list(&self) -> Result<Vec<LayoutRecord>, RepoError> {
        Ok(vec![])
    }

    async fn update(
        &self,
        id: Uuid,
        _name: &str,
        _schema: &LayoutSchema,
    ) -> Result<LayoutRecord, RepoError> {
        Err(RepoError::NotFound(id))
    }

    async fn delete(&self, id: Uuid) -> Result<(), RepoError> {
        Err(RepoError::NotFound(id))
    }
}
