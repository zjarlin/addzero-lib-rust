/// Template management — save/load reusable layout templates.
///
/// The `TemplateRepo` provides a PG-backed CRUD interface.  Implementations
/// will be completed alongside #81.
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::schema::LayoutSchema;

/// A reusable layout template.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Template {
    /// Unique identifier.
    pub id: Uuid,
    /// Human-readable name.
    pub name: String,
    /// The layout schema this template wraps.
    pub layout: LayoutSchema,
    /// ISO-8601 creation timestamp.
    pub created_at: String,
}

/// Errors from template operations.
#[derive(Debug, thiserror::Error)]
pub enum TemplateError {
    /// The requested template does not exist.
    #[error("template not found: {0}")]
    NotFound(Uuid),
    /// The template failed validation before save.
    #[error("template validation failed: {0}")]
    ValidationFailed(String),
    /// Underlying database error.
    #[error("database error: {0}")]
    Database(#[from] sqlx::Error),
}

/// Template repository backed by PostgreSQL (skeleton).
///
/// The actual query implementations will be added alongside #81.
pub struct TemplateRepo;

impl TemplateRepo {
    /// Creates a new (stub) repo.
    pub fn new() -> Self {
        Self
    }

    /// Inserts a template into the database.
    ///
    /// Currently returns a not-implemented error; will be wired to PG in #81.
    pub async fn create(&self, _tpl: &Template) -> Result<Template, TemplateError> {
        Err(TemplateError::ValidationFailed(
            "template create is not yet implemented (pending #81)".into(),
        ))
    }

    /// Retrieves a template by id.
    pub async fn get(&self, id: Uuid) -> Result<Template, TemplateError> {
        Err(TemplateError::NotFound(id))
    }

    /// Lists all templates.
    pub async fn list(&self) -> Result<Vec<Template>, TemplateError> {
        Ok(vec![])
    }

    /// Updates an existing template.
    pub async fn update(&self, _tpl: &Template) -> Result<Template, TemplateError> {
        Err(TemplateError::ValidationFailed(
            "template update is not yet implemented (pending #81)".into(),
        ))
    }

    /// Deletes a template by id.
    pub async fn delete(&self, id: Uuid) -> Result<(), TemplateError> {
        Err(TemplateError::NotFound(id))
    }
}

impl Default for TemplateRepo {
    fn default() -> Self {
        Self::new()
    }
}
