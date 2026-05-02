pub mod catalog;
pub mod config;
pub mod discovery;
mod entity;
mod repository;
pub mod service;
pub mod types;

pub use catalog::render_catalog;
pub use config::{database_url, local_env_path, source_specs};
pub use discovery::{discover_documents, discover_source_documents};
pub use service::KnowledgeService;
pub use types::{
    KnowledgeDocument, KnowledgeError, KnowledgeScan, KnowledgeSourceSpec, KnowledgeSyncReport,
    ManualKnowledgeDocumentInput,
};
