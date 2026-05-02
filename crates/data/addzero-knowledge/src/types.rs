use std::path::PathBuf;

use addzero_persistence::PersistenceError;
use chrono::{DateTime, Utc};
use sea_orm::DbErr;
use serde::{Deserialize, Serialize};
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

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ManualKnowledgeDocumentInput {
    pub source_slug: String,
    pub source_name: String,
    pub source_root: String,
    pub source_path: String,
    pub relative_path: String,
    pub title: String,
    pub source_label: String,
    pub body: String,
    pub tags: Vec<String>,
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
    #[error("connect knowledge persistence: {0}")]
    Persistence(#[from] PersistenceError),
    #[error("query knowledge rows: {0}")]
    Query(#[source] DbErr),
    #[error("{0}")]
    Message(String),
}
