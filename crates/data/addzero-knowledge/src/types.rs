use std::path::PathBuf;

use chrono::{DateTime, Utc};
use thiserror::Error;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct KnowledgeSourceSpec {
    pub slug: String,
    pub name: String,
    pub root_path: PathBuf,
}

impl KnowledgeSourceSpec {
    pub fn new(
        slug: impl Into<String>,
        name: impl Into<String>,
        root_path: impl Into<PathBuf>,
    ) -> Self {
        Self {
            slug: slug.into(),
            name: name.into(),
            root_path: root_path.into(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct KnowledgeDocument {
    pub source_slug: String,
    pub source_name: String,
    pub source_root: String,
    pub slug: String,
    pub title: String,
    pub filename: String,
    pub source_path: String,
    pub relative_path: String,
    pub bytes: usize,
    pub section_count: usize,
    pub preview: String,
    pub excerpt: String,
    pub headings: Vec<String>,
    pub body: String,
    pub content_hash: String,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct KnowledgeScan {
    pub documents: Vec<KnowledgeDocument>,
    pub skipped_paths: Vec<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct KnowledgeSyncReport {
    pub synced_sources: Vec<String>,
    pub upserted_documents: usize,
    pub skipped_paths: Vec<String>,
    pub finished_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Error)]
pub enum KnowledgeError {
    #[error("could not resolve home directory")]
    MissingHomeDir,
    #[error("connect to postgres: {0}")]
    ConnectPostgres(#[source] sqlx::Error),
    #[error("apply knowledge schema: {0}")]
    ApplySchema(#[source] sqlx::Error),
    #[error("query knowledge rows: {0}")]
    Query(#[source] sqlx::Error),
}
