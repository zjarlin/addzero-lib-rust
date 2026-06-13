//! 知识文档管理与同步服务。
//!
//! 负责扫描本地 Markdown/文本文件目录，将文档元数据持久化到 PostgreSQL 或 SQLite，
//! 并提供目录生成（catalog）与全文检索能力。
//!
//! # 主要模块
//!
//! - [`catalog`] — 将文档元数据渲染为 Rust 常量数组，供编译期嵌入
//! - [`config`] — 数据库连接 URL 与知识源目录的环境变量解析
//! - [`discovery`] — 递归扫描目录，按扩展名与过滤规则发现文档
//! - [`service`] — `KnowledgeService`：同步、查询、手动录入的核心入口
//! - [`types`] — 文档、扫描结果、同步报告等共享数据结构
//!
//! # 快速开始
//!
//! ```rust,no_run
//! use az_knowledge::{KnowledgeService, KnowledgeSourceSpec, source_specs};
//!
//! # async fn example() -> Result<(), az_knowledge::KnowledgeError> {
//! let url = az_knowledge::database_url().expect("DATABASE_URL 未设置");
//! let service = KnowledgeService::connect(&url).await?;
//! let docs = service.list_documents().await?;
//! println!("已索引 {} 篇文档", docs.len());
//! # Ok(())
//! # }
//! ```

automod::dir!(pub "src");

pub use catalog::render_catalog;
pub use config::{database_url, local_env_path, source_specs};
pub use discovery::{discover_documents, discover_source_documents};
pub use service::KnowledgeService;
pub use types::{
    KnowledgeDocument, KnowledgeError, KnowledgeScan, KnowledgeSourceSpec, KnowledgeSyncReport,
    ManualKnowledgeDocumentInput,
};
