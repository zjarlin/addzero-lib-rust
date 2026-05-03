/// PG repository for lowcode layouts (skeleton — to be fleshed out in subsequent issues).

use uuid::Uuid;

use crate::schema::Layout;

/// Errors from layout repository operations.
#[derive(Debug, thiserror::Error)]
pub enum RepoError {
    #[error("layout not found: {0}")]
    NotFound(Uuid),
    #[error("database error: {0}")]
    Database(#[from] sqlx::Error),
}

/// Layout repository backed by PostgreSQL (skeleton).
///
/// The actual query implementations will be added alongside issues #75–#81.
pub struct LayoutRepo;

impl LayoutRepo {
    pub fn new() -> Self {
        Self
    }

    pub async fn create(&self, _layout: &Layout) -> Result<Layout, RepoError> {
        todo!("layout create — will be implemented in #75")
    }

    pub async fn get(&self, _id: Uuid) -> Result<Layout, RepoError> {
        todo!("layout get — will be implemented in #75")
    }

    pub async fn list(&self) -> Result<Vec<Layout>, RepoError> {
        todo!("layout list — will be implemented in #75")
    }

    pub async fn update(&self, _layout: &Layout) -> Result<Layout, RepoError> {
        todo!("layout update — will be implemented in #75")
    }

    pub async fn delete(&self, _id: Uuid) -> Result<(), RepoError> {
        todo!("layout delete — will be implemented in #75")
    }
}

impl Default for LayoutRepo {
    fn default() -> Self {
        Self::new()
    }
}
