/// Template management — save/load reusable layout templates (skeleton — to be fleshed out in #81).

use uuid::Uuid;

use crate::schema::Template;

/// Errors from template operations.
#[derive(Debug, thiserror::Error)]
pub enum TemplateError {
    #[error("template not found: {0}")]
    NotFound(Uuid),
    #[error("template validation failed: {0}")]
    ValidationFailed(String),
    #[error("database error: {0}")]
    Database(#[from] sqlx::Error),
}

/// Template repository backed by PostgreSQL (skeleton).
///
/// The actual query implementations will be added alongside #81.
pub struct TemplateRepo;

impl TemplateRepo {
    pub fn new() -> Self {
        Self
    }

    pub async fn create(&self, _tpl: &Template) -> Result<Template, TemplateError> {
        todo!("template create — will be implemented in #81")
    }

    pub async fn get(&self, _id: Uuid) -> Result<Template, TemplateError> {
        todo!("template get — will be implemented in #81")
    }

    pub async fn list(&self) -> Result<Vec<Template>, TemplateError> {
        todo!("template list — will be implemented in #81")
    }

    pub async fn update(&self, _tpl: &Template) -> Result<Template, TemplateError> {
        todo!("template update — will be implemented in #81")
    }

    pub async fn delete(&self, _id: Uuid) -> Result<(), TemplateError> {
        todo!("template delete — will be implemented in #81")
    }
}

impl Default for TemplateRepo {
    fn default() -> Self {
        Self::new()
    }
}
