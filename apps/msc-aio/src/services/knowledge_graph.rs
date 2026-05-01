use std::{future::Future, pin::Pin, rc::Rc};

#[cfg(not(target_arch = "wasm32"))]
use std::time::Duration;

#[cfg(not(target_arch = "wasm32"))]
use chrono::{DateTime, Utc};
#[cfg(not(target_arch = "wasm32"))]
use sqlx::Row;
use serde::{Deserialize, Serialize};
use thiserror::Error;

pub type LocalBoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + 'a>>;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum KnowledgeSourceTypeDto {
    Note,
    Chat,
    Web,
    File,
    Import,
    Ocr,
    Ai,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum KnowledgeRawTypeDto {
    RawNote,
    RawChat,
    RawWebExcerpt,
    RawFile,
    RawOcr,
    RawAiDraft,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum KnowledgeRawStatusDto {
    Active,
    Covered,
    Archived,
    Discarded,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum KnowledgeNodeTypeDto {
    Note,
    Topic,
    Entity,
    Decision,
    Fact,
    Summary,
    Project,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum KnowledgeNodeStatusDto {
    Active,
    Superseded,
    Conflicted,
    Archived,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum KnowledgeVisibilityDto {
    Default,
    Hidden,
    System,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum KnowledgeEdgeTypeDto {
    SameAs,
    SimilarTo,
    BelongsTo,
    Mentions,
    About,
    ConflictsWith,
    Supersedes,
    RelatedTo,
    DependsOn,
    DerivedFromRaw,
    SupportedByRaw,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum KnowledgeEdgeStatusDto {
    Active,
    Suggested,
    Rejected,
    Hidden,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum KnowledgeEvidenceTypeDto {
    DirectQuote,
    SummarySource,
    EntityExtraction,
    ConflictSource,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum KnowledgeSuggestionTypeDto {
    MergeNodes,
    ArchiveRaw,
    LinkNode,
    ReplaceSummary,
    CreateEdge,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum KnowledgeSuggestionStatusDto {
    Pending,
    Applied,
    Discarded,
    PromotedToException,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum KnowledgeExceptionTypeDto {
    Conflict,
    UncertainMerge,
    HighRiskArchive,
    BehaviorAffectingChange,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum KnowledgeExceptionSeverityDto {
    Low,
    Medium,
    High,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum KnowledgeExceptionStatusDto {
    Open,
    Resolved,
    Ignored,
    AutoResolved,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum KnowledgeResolutionDto {
    AcceptA,
    AcceptB,
    KeepBoth,
    Merge,
    Ignore,
    ArchiveRaw,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct KnowledgeNodeSummaryDto {
    pub id: String,
    pub node_type: String,
    pub title: String,
    pub summary: String,
    pub status: String,
    pub confidence: f64,
    pub source_count: usize,
    pub has_conflict: bool,
    pub updated_at: Option<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct KnowledgeNodeDetailDto {
    pub id: String,
    pub node_type: String,
    pub title: String,
    pub body: String,
    pub summary: String,
    pub status: String,
    pub visibility: String,
    pub confidence: f64,
    pub source_count: usize,
    pub relation_count: usize,
    pub has_conflict: bool,
    pub updated_at: Option<String>,
    pub metadata: serde_json::Value,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct KnowledgeSourceRefDto {
    pub raw_item_id: String,
    pub source_type: String,
    pub raw_type: String,
    pub title: String,
    pub excerpt: String,
    pub status: String,
    pub captured_at: Option<String>,
    pub updated_at: Option<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct KnowledgeExceptionCardDto {
    pub id: String,
    pub exception_type: String,
    pub severity: String,
    pub subject_title: String,
    pub related_title: Option<String>,
    pub ai_recommendation: String,
    pub reason: String,
    pub status: String,
    pub created_at: Option<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct KnowledgeFeedDto {
    pub items: Vec<KnowledgeNodeSummaryDto>,
    pub total: usize,
    pub open_exception_count: usize,
    pub warnings: Vec<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct IngestKnowledgeRawInput {
    pub source_type: String,
    pub source_ref: Option<String>,
    pub title: String,
    pub content: String,
    pub locale: Option<String>,
    pub metadata: serde_json::Value,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct ResolveKnowledgeExceptionInput {
    pub resolution: String,
    pub resolved_by: Option<String>,
    pub note: Option<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct KnowledgeMaintenanceReportDto {
    pub raw_items_ingested: usize,
    pub nodes_created: usize,
    pub nodes_updated: usize,
    pub suggestions_applied: usize,
    pub exceptions_opened: usize,
    pub warnings: Vec<String>,
}

#[derive(Clone, Debug, Error, PartialEq, Eq)]
pub enum KnowledgeGraphError {
    #[error("{0}")]
    Message(String),
}

impl KnowledgeGraphError {
    fn new(message: impl Into<String>) -> Self {
        Self::Message(message.into())
    }
}

pub type KnowledgeGraphResult<T> = Result<T, KnowledgeGraphError>;

pub trait KnowledgeGraphApi: 'static {
    fn feed(&self) -> LocalBoxFuture<'_, KnowledgeGraphResult<KnowledgeFeedDto>>;
    fn node_detail(
        &self,
        node_id: String,
    ) -> LocalBoxFuture<'_, KnowledgeGraphResult<KnowledgeNodeDetailDto>>;
    fn node_sources(
        &self,
        node_id: String,
    ) -> LocalBoxFuture<'_, KnowledgeGraphResult<Vec<KnowledgeSourceRefDto>>>;
    fn exceptions(
        &self,
    ) -> LocalBoxFuture<'_, KnowledgeGraphResult<Vec<KnowledgeExceptionCardDto>>>;
    fn ingest_raw(
        &self,
        input: IngestKnowledgeRawInput,
    ) -> LocalBoxFuture<'_, KnowledgeGraphResult<KnowledgeNodeSummaryDto>>;
    fn resolve_exception(
        &self,
        exception_id: String,
        input: ResolveKnowledgeExceptionInput,
    ) -> LocalBoxFuture<'_, KnowledgeGraphResult<KnowledgeExceptionCardDto>>;
    fn run_maintenance(
        &self,
    ) -> LocalBoxFuture<'_, KnowledgeGraphResult<KnowledgeMaintenanceReportDto>>;
}

pub type SharedKnowledgeGraphApi = Rc<dyn KnowledgeGraphApi>;

pub fn default_knowledge_graph_api() -> SharedKnowledgeGraphApi {
    #[cfg(target_arch = "wasm32")]
    {
        Rc::new(BrowserKnowledgeGraphApi)
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        Rc::new(EmbeddedKnowledgeGraphApi)
    }
}

#[cfg(target_arch = "wasm32")]
struct BrowserKnowledgeGraphApi;

#[cfg(target_arch = "wasm32")]
impl KnowledgeGraphApi for BrowserKnowledgeGraphApi {
    fn feed(&self) -> LocalBoxFuture<'_, KnowledgeGraphResult<KnowledgeFeedDto>> {
        Box::pin(async move {
            super::browser_http::get_json("/api/admin/knowledge/feed")
                .await
                .map_err(KnowledgeGraphError::new)
        })
    }

    fn node_detail(
        &self,
        node_id: String,
    ) -> LocalBoxFuture<'_, KnowledgeGraphResult<KnowledgeNodeDetailDto>> {
        Box::pin(async move {
            super::browser_http::get_json(&format!("/api/admin/knowledge/nodes/{node_id}"))
                .await
                .map_err(KnowledgeGraphError::new)
        })
    }

    fn node_sources(
        &self,
        node_id: String,
    ) -> LocalBoxFuture<'_, KnowledgeGraphResult<Vec<KnowledgeSourceRefDto>>> {
        Box::pin(async move {
            super::browser_http::get_json(&format!(
                "/api/admin/knowledge/nodes/{node_id}/sources"
            ))
            .await
            .map_err(KnowledgeGraphError::new)
        })
    }

    fn exceptions(
        &self,
    ) -> LocalBoxFuture<'_, KnowledgeGraphResult<Vec<KnowledgeExceptionCardDto>>> {
        Box::pin(async move {
            super::browser_http::get_json("/api/admin/knowledge/exceptions")
                .await
                .map_err(KnowledgeGraphError::new)
        })
    }

    fn ingest_raw(
        &self,
        input: IngestKnowledgeRawInput,
    ) -> LocalBoxFuture<'_, KnowledgeGraphResult<KnowledgeNodeSummaryDto>> {
        Box::pin(async move {
            super::browser_http::post_json("/api/admin/knowledge/raw-items", &input)
                .await
                .map_err(KnowledgeGraphError::new)
        })
    }

    fn resolve_exception(
        &self,
        exception_id: String,
        input: ResolveKnowledgeExceptionInput,
    ) -> LocalBoxFuture<'_, KnowledgeGraphResult<KnowledgeExceptionCardDto>> {
        Box::pin(async move {
            super::browser_http::post_json(
                &format!("/api/admin/knowledge/exceptions/{exception_id}/resolve"),
                &input,
            )
            .await
            .map_err(KnowledgeGraphError::new)
        })
    }

    fn run_maintenance(
        &self,
    ) -> LocalBoxFuture<'_, KnowledgeGraphResult<KnowledgeMaintenanceReportDto>> {
        Box::pin(async move {
            super::browser_http::post_json(
                "/api/admin/knowledge/maintenance/run",
                &serde_json::json!({}),
            )
            .await
            .map_err(KnowledgeGraphError::new)
        })
    }
}

#[cfg(not(target_arch = "wasm32"))]
struct EmbeddedKnowledgeGraphApi;

#[cfg(not(target_arch = "wasm32"))]
impl KnowledgeGraphApi for EmbeddedKnowledgeGraphApi {
    fn feed(&self) -> LocalBoxFuture<'_, KnowledgeGraphResult<KnowledgeFeedDto>> {
        Box::pin(async move { load_knowledge_feed_on_server().await })
    }

    fn node_detail(
        &self,
        node_id: String,
    ) -> LocalBoxFuture<'_, KnowledgeGraphResult<KnowledgeNodeDetailDto>> {
        Box::pin(async move { load_knowledge_node_detail_on_server(&node_id).await })
    }

    fn node_sources(
        &self,
        node_id: String,
    ) -> LocalBoxFuture<'_, KnowledgeGraphResult<Vec<KnowledgeSourceRefDto>>> {
        Box::pin(async move { load_knowledge_node_sources_on_server(&node_id).await })
    }

    fn exceptions(
        &self,
    ) -> LocalBoxFuture<'_, KnowledgeGraphResult<Vec<KnowledgeExceptionCardDto>>> {
        Box::pin(async move { load_knowledge_exceptions_on_server().await })
    }

    fn ingest_raw(
        &self,
        input: IngestKnowledgeRawInput,
    ) -> LocalBoxFuture<'_, KnowledgeGraphResult<KnowledgeNodeSummaryDto>> {
        Box::pin(async move { ingest_knowledge_raw_on_server(input).await })
    }

    fn resolve_exception(
        &self,
        exception_id: String,
        input: ResolveKnowledgeExceptionInput,
    ) -> LocalBoxFuture<'_, KnowledgeGraphResult<KnowledgeExceptionCardDto>> {
        Box::pin(async move { resolve_knowledge_exception_on_server(&exception_id, input).await })
    }

    fn run_maintenance(
        &self,
    ) -> LocalBoxFuture<'_, KnowledgeGraphResult<KnowledgeMaintenanceReportDto>> {
        Box::pin(async move { run_knowledge_maintenance_on_server().await })
    }
}

#[cfg(not(target_arch = "wasm32"))]
const KNOWLEDGE_SCHEMA_SQL: &str =
    include_str!("../server/migrations/0005_admin_knowledge_graph.sql");

#[cfg(not(target_arch = "wasm32"))]
pub async fn load_knowledge_feed_on_server() -> KnowledgeGraphResult<KnowledgeFeedDto> {
    let pool = connect_pool().await?;
    ensure_knowledge_schema(&pool).await?;
    read_knowledge_feed(&pool).await
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn load_knowledge_node_detail_on_server(
    node_id: &str,
) -> KnowledgeGraphResult<KnowledgeNodeDetailDto> {
    let pool = connect_pool().await?;
    ensure_knowledge_schema(&pool).await?;
    read_knowledge_node_detail(&pool, node_id).await
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn load_knowledge_node_sources_on_server(
    node_id: &str,
) -> KnowledgeGraphResult<Vec<KnowledgeSourceRefDto>> {
    let pool = connect_pool().await?;
    ensure_knowledge_schema(&pool).await?;
    read_knowledge_node_sources(&pool, node_id).await
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn load_knowledge_exceptions_on_server(
) -> KnowledgeGraphResult<Vec<KnowledgeExceptionCardDto>> {
    let pool = connect_pool().await?;
    ensure_knowledge_schema(&pool).await?;
    read_knowledge_exceptions(&pool).await
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn ingest_knowledge_raw_on_server(
    input: IngestKnowledgeRawInput,
) -> KnowledgeGraphResult<KnowledgeNodeSummaryDto> {
    let pool = connect_pool().await?;
    ensure_knowledge_schema(&pool).await?;

    let raw_id = format!("raw:{}", uuid::Uuid::new_v4());
    let source_id = format!("source:{}", uuid::Uuid::new_v4());
    let node_id = format!("node:{}", uuid::Uuid::new_v4());
    let now = Utc::now();
    let locale = input.locale.unwrap_or_else(|| "zh-CN".to_string());
    let title = if input.title.trim().is_empty() {
        "未命名知识".to_string()
    } else {
        input.title.trim().to_string()
    };
    let summary = summarize(&input.content);
    let content_hash = blake3::hash(input.content.as_bytes()).to_hex().to_string();
    let metadata = input.metadata;
    let source_ref = input.source_ref.unwrap_or_default();

    let mut tx = pool.begin().await.map_err(query_error)?;

    sqlx::query(
        r#"
        INSERT INTO admin_knowledge_sources (
            id, source_type, source_ref, title, locale, metadata, captured_at, created_at, updated_at
        ) VALUES ($1, $2, $3, $4, $5, $6, $7, NOW(), NOW())
        "#,
    )
    .bind(&source_id)
    .bind(input.source_type)
    .bind(source_ref)
    .bind(&title)
    .bind(&locale)
    .bind(metadata.clone())
    .bind(now)
    .execute(&mut *tx)
    .await
    .map_err(query_error)?;

    sqlx::query(
        r#"
        INSERT INTO admin_knowledge_raw_items (
            id, source_id, raw_type, title, content, content_hash, hash_algorithm, locale,
            status, importance_score, quality_score, token_estimate, metadata, captured_at, created_at, updated_at
        ) VALUES ($1, $2, 'raw_note', $3, $4, $5, 'blake3', $6, 'active', 0.5, 0.5, $7, $8, $9, NOW(), NOW())
        "#,
    )
    .bind(&raw_id)
    .bind(&source_id)
    .bind(&title)
    .bind(&input.content)
    .bind(&content_hash)
    .bind(&locale)
    .bind(token_estimate(&input.content) as i32)
    .bind(metadata.clone())
    .bind(now)
    .execute(&mut *tx)
    .await
    .map_err(query_error)?;

    sqlx::query(
        r#"
        INSERT INTO admin_knowledge_nodes (
            id, node_type, title, body, summary, status, visibility, locale,
            confidence, importance_score, last_ai_refresh_at, metadata, created_at, updated_at
        ) VALUES ($1, 'note', $2, $3, $4, 'active', 'default', $5, 0.65, 0.5, NOW(), $6, NOW(), NOW())
        "#,
    )
    .bind(&node_id)
    .bind(&title)
    .bind(&input.content)
    .bind(&summary)
    .bind(&locale)
    .bind(metadata.clone())
    .execute(&mut *tx)
    .await
    .map_err(query_error)?;

    sqlx::query(
        r#"
        INSERT INTO admin_knowledge_evidence (
            id, node_id, raw_item_id, evidence_type, excerpt, confidence, metadata, created_at
        ) VALUES ($1, $2, $3, 'summary_source', $4, 0.7, '{}'::jsonb, NOW())
        "#,
    )
    .bind(format!("evidence:{}", uuid::Uuid::new_v4()))
    .bind(&node_id)
    .bind(&raw_id)
    .bind(summary.clone())
    .execute(&mut *tx)
    .await
    .map_err(query_error)?;

    tx.commit().await.map_err(query_error)?;

    Ok(KnowledgeNodeSummaryDto {
        id: node_id,
        node_type: "note".to_string(),
        title,
        summary,
        status: "active".to_string(),
        confidence: 0.65,
        source_count: 1,
        has_conflict: false,
        updated_at: Some(now.to_rfc3339()),
    })
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn resolve_knowledge_exception_on_server(
    exception_id: &str,
    input: ResolveKnowledgeExceptionInput,
) -> KnowledgeGraphResult<KnowledgeExceptionCardDto> {
    let pool = connect_pool().await?;
    ensure_knowledge_schema(&pool).await?;

    let row = sqlx::query(
        r#"
        UPDATE admin_knowledge_exceptions
        SET status = 'resolved',
            resolution = $2,
            resolved_by = $3,
            resolved_at = NOW(),
            updated_at = NOW(),
            metadata = jsonb_strip_nulls(COALESCE(metadata, '{}'::jsonb) || jsonb_build_object('note', $4))
        WHERE id = $1
        RETURNING id, exception_type, severity, ai_recommendation, reason, status, created_at,
                  subject_node_id, related_node_id
        "#,
    )
    .bind(exception_id)
    .bind(input.resolution)
    .bind(input.resolved_by)
    .bind(input.note)
    .fetch_optional(&pool)
    .await
    .map_err(query_error)?
    .ok_or_else(|| KnowledgeGraphError::new(format!("未找到异常：{exception_id}")))?;

    Ok(KnowledgeExceptionCardDto {
        id: row.try_get("id").map_err(query_error)?,
        exception_type: row.try_get("exception_type").map_err(query_error)?,
        severity: row.try_get("severity").map_err(query_error)?,
        subject_title: node_title_by_id(&pool, row.try_get("subject_node_id").ok().flatten()).await?,
        related_title: match row.try_get::<Option<String>, _>("related_node_id") {
            Ok(value) => node_title_by_id(&pool, value).await.ok(),
            Err(_) => None,
        },
        ai_recommendation: row.try_get("ai_recommendation").map_err(query_error)?,
        reason: row.try_get("reason").map_err(query_error)?,
        status: row.try_get("status").map_err(query_error)?,
        created_at: row
            .try_get::<Option<DateTime<Utc>>, _>("created_at")
            .map_err(query_error)?
            .map(|value| value.to_rfc3339()),
    })
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn run_knowledge_maintenance_on_server(
) -> KnowledgeGraphResult<KnowledgeMaintenanceReportDto> {
    let pool = connect_pool().await?;
    ensure_knowledge_schema(&pool).await?;

    let raw_items_ingested = scalar_count(
        &pool,
        "SELECT COUNT(*)::BIGINT FROM admin_knowledge_raw_items",
    )
    .await?;
    let nodes_created = scalar_count(
        &pool,
        "SELECT COUNT(*)::BIGINT FROM admin_knowledge_nodes",
    )
    .await?;
    let exceptions_opened = scalar_count(
        &pool,
        "SELECT COUNT(*)::BIGINT FROM admin_knowledge_exceptions WHERE status = 'open'",
    )
    .await?;

    Ok(KnowledgeMaintenanceReportDto {
        raw_items_ingested,
        nodes_created,
        nodes_updated: 0,
        suggestions_applied: 0,
        exceptions_opened,
        warnings: vec!["当前为骨架实现：已完成 schema ensure、feed/detail/sources/exceptions 查询与基础 ingest。".to_string()],
    })
}

#[cfg(not(target_arch = "wasm32"))]
async fn connect_pool() -> KnowledgeGraphResult<sqlx::postgres::PgPool> {
    let database_url = addzero_knowledge::database_url().ok_or_else(|| {
        KnowledgeGraphError::new("缺少 PostgreSQL 连接：请设置 MSC_AIO_DATABASE_URL 或 DATABASE_URL")
    })?;

    sqlx::postgres::PgPoolOptions::new()
        .max_connections(4)
        .acquire_timeout(Duration::from_secs(5))
        .connect(&database_url)
        .await
        .map_err(|err| KnowledgeGraphError::new(format!("连接 PostgreSQL 失败：{err}")))
}

#[cfg(not(target_arch = "wasm32"))]
async fn ensure_knowledge_schema(pool: &sqlx::postgres::PgPool) -> KnowledgeGraphResult<()> {
    for statement in KNOWLEDGE_SCHEMA_SQL.split(';') {
        let trimmed = statement.trim();
        if trimmed.is_empty() {
            continue;
        }
        sqlx::query(trimmed)
            .execute(pool)
            .await
            .map_err(query_error)?;
    }
    Ok(())
}

#[cfg(not(target_arch = "wasm32"))]
async fn read_knowledge_feed(pool: &sqlx::postgres::PgPool) -> KnowledgeGraphResult<KnowledgeFeedDto> {
    let rows = sqlx::query(
        r#"
        SELECT
            n.id,
            n.node_type,
            n.title,
            n.summary,
            n.status,
            n.confidence,
            n.updated_at,
            COUNT(DISTINCT e.raw_item_id)::BIGINT AS source_count,
            EXISTS (
                SELECT 1
                FROM admin_knowledge_exceptions ex
                WHERE ex.subject_node_id = n.id
                  AND ex.status = 'open'
            ) AS has_conflict
        FROM admin_knowledge_nodes n
        LEFT JOIN admin_knowledge_evidence e ON e.node_id = n.id
        WHERE n.visibility <> 'hidden'
        GROUP BY n.id
        ORDER BY n.updated_at DESC
        LIMIT 200
        "#,
    )
    .fetch_all(pool)
    .await
    .map_err(query_error)?;

    let items = rows
        .into_iter()
        .map(|row| KnowledgeNodeSummaryDto {
            id: row.try_get("id").unwrap_or_default(),
            node_type: row.try_get("node_type").unwrap_or_default(),
            title: row.try_get("title").unwrap_or_default(),
            summary: row.try_get("summary").unwrap_or_default(),
            status: row.try_get("status").unwrap_or_default(),
            confidence: row.try_get("confidence").unwrap_or(0.0),
            source_count: row.try_get::<i64, _>("source_count").unwrap_or(0).max(0) as usize,
            has_conflict: row.try_get("has_conflict").unwrap_or(false),
            updated_at: row
                .try_get::<Option<DateTime<Utc>>, _>("updated_at")
                .ok()
                .flatten()
                .map(|value| value.to_rfc3339()),
        })
        .collect::<Vec<_>>();

    let open_exception_count = scalar_count(
        pool,
        "SELECT COUNT(*)::BIGINT FROM admin_knowledge_exceptions WHERE status = 'open'",
    )
    .await?;

    Ok(KnowledgeFeedDto {
        total: items.len(),
        items,
        open_exception_count,
        warnings: Vec::new(),
    })
}

#[cfg(not(target_arch = "wasm32"))]
async fn read_knowledge_node_detail(
    pool: &sqlx::postgres::PgPool,
    node_id: &str,
) -> KnowledgeGraphResult<KnowledgeNodeDetailDto> {
    let row = sqlx::query(
        r#"
        SELECT
            n.id,
            n.node_type,
            n.title,
            n.body,
            n.summary,
            n.status,
            n.visibility,
            n.confidence,
            n.updated_at,
            n.metadata,
            COUNT(DISTINCT e.raw_item_id)::BIGINT AS source_count,
            COUNT(DISTINCT ed.id)::BIGINT AS relation_count,
            EXISTS (
                SELECT 1
                FROM admin_knowledge_exceptions ex
                WHERE ex.subject_node_id = n.id
                  AND ex.status = 'open'
            ) AS has_conflict
        FROM admin_knowledge_nodes n
        LEFT JOIN admin_knowledge_evidence e ON e.node_id = n.id
        LEFT JOIN admin_knowledge_edges ed ON ed.from_node_id = n.id AND ed.status = 'active'
        WHERE n.id = $1
        GROUP BY n.id
        "#,
    )
    .bind(node_id)
    .fetch_optional(pool)
    .await
    .map_err(query_error)?
    .ok_or_else(|| KnowledgeGraphError::new(format!("未找到知识节点：{node_id}")))?;

    Ok(KnowledgeNodeDetailDto {
        id: row.try_get("id").map_err(query_error)?,
        node_type: row.try_get("node_type").map_err(query_error)?,
        title: row.try_get("title").map_err(query_error)?,
        body: row.try_get("body").map_err(query_error)?,
        summary: row.try_get("summary").map_err(query_error)?,
        status: row.try_get("status").map_err(query_error)?,
        visibility: row.try_get("visibility").map_err(query_error)?,
        confidence: row.try_get("confidence").map_err(query_error)?,
        source_count: row.try_get::<i64, _>("source_count").map_err(query_error)?.max(0) as usize,
        relation_count: row.try_get::<i64, _>("relation_count").map_err(query_error)?.max(0) as usize,
        has_conflict: row.try_get("has_conflict").map_err(query_error)?,
        updated_at: row
            .try_get::<Option<DateTime<Utc>>, _>("updated_at")
            .map_err(query_error)?
            .map(|value| value.to_rfc3339()),
        metadata: row.try_get("metadata").map_err(query_error)?,
    })
}

#[cfg(not(target_arch = "wasm32"))]
async fn read_knowledge_node_sources(
    pool: &sqlx::postgres::PgPool,
    node_id: &str,
) -> KnowledgeGraphResult<Vec<KnowledgeSourceRefDto>> {
    let rows = sqlx::query(
        r#"
        SELECT
            r.id AS raw_item_id,
            COALESCE(s.source_type, 'note') AS source_type,
            r.raw_type,
            r.title,
            COALESCE(NULLIF(ev.excerpt, ''), LEFT(r.content, 160)) AS excerpt,
            r.status,
            r.captured_at,
            r.updated_at
        FROM admin_knowledge_evidence ev
        JOIN admin_knowledge_raw_items r ON r.id = ev.raw_item_id
        LEFT JOIN admin_knowledge_sources s ON s.id = r.source_id
        WHERE ev.node_id = $1
        ORDER BY r.updated_at DESC, ev.created_at DESC
        "#,
    )
    .bind(node_id)
    .fetch_all(pool)
    .await
    .map_err(query_error)?;

    Ok(rows
        .into_iter()
        .map(|row| KnowledgeSourceRefDto {
            raw_item_id: row.try_get("raw_item_id").unwrap_or_default(),
            source_type: row.try_get("source_type").unwrap_or_else(|_| "note".to_string()),
            raw_type: row.try_get("raw_type").unwrap_or_default(),
            title: row.try_get("title").unwrap_or_default(),
            excerpt: row.try_get("excerpt").unwrap_or_default(),
            status: row.try_get("status").unwrap_or_default(),
            captured_at: row
                .try_get::<Option<DateTime<Utc>>, _>("captured_at")
                .ok()
                .flatten()
                .map(|value| value.to_rfc3339()),
            updated_at: row
                .try_get::<Option<DateTime<Utc>>, _>("updated_at")
                .ok()
                .flatten()
                .map(|value| value.to_rfc3339()),
        })
        .collect())
}

#[cfg(not(target_arch = "wasm32"))]
async fn read_knowledge_exceptions(
    pool: &sqlx::postgres::PgPool,
) -> KnowledgeGraphResult<Vec<KnowledgeExceptionCardDto>> {
    let rows = sqlx::query(
        r#"
        SELECT
            ex.id,
            ex.exception_type,
            ex.severity,
            ex.ai_recommendation,
            ex.reason,
            ex.status,
            ex.created_at,
            sn.title AS subject_title,
            rn.title AS related_title
        FROM admin_knowledge_exceptions ex
        LEFT JOIN admin_knowledge_nodes sn ON sn.id = ex.subject_node_id
        LEFT JOIN admin_knowledge_nodes rn ON rn.id = ex.related_node_id
        ORDER BY
            CASE ex.severity WHEN 'high' THEN 3 WHEN 'medium' THEN 2 ELSE 1 END DESC,
            ex.updated_at DESC
        LIMIT 100
        "#,
    )
    .fetch_all(pool)
    .await
    .map_err(query_error)?;

    Ok(rows
        .into_iter()
        .map(|row| KnowledgeExceptionCardDto {
            id: row.try_get("id").unwrap_or_default(),
            exception_type: row.try_get("exception_type").unwrap_or_default(),
            severity: row.try_get("severity").unwrap_or_default(),
            subject_title: row.try_get("subject_title").unwrap_or_default(),
            related_title: row.try_get("related_title").ok(),
            ai_recommendation: row.try_get("ai_recommendation").unwrap_or_default(),
            reason: row.try_get("reason").unwrap_or_default(),
            status: row.try_get("status").unwrap_or_default(),
            created_at: row
                .try_get::<Option<DateTime<Utc>>, _>("created_at")
                .ok()
                .flatten()
                .map(|value| value.to_rfc3339()),
        })
        .collect())
}

#[cfg(not(target_arch = "wasm32"))]
async fn node_title_by_id(
    pool: &sqlx::postgres::PgPool,
    node_id: Option<String>,
) -> KnowledgeGraphResult<String> {
    let Some(node_id) = node_id else {
        return Ok(String::new());
    };
    let title = sqlx::query_scalar::<_, String>(
        "SELECT title FROM admin_knowledge_nodes WHERE id = $1 LIMIT 1",
    )
    .bind(node_id)
    .fetch_optional(pool)
    .await
    .map_err(query_error)?
    .unwrap_or_default();
    Ok(title)
}

#[cfg(not(target_arch = "wasm32"))]
async fn scalar_count(pool: &sqlx::postgres::PgPool, sql: &str) -> KnowledgeGraphResult<usize> {
    let value = sqlx::query_scalar::<_, i64>(sql)
        .fetch_one(pool)
        .await
        .map_err(query_error)?;
    Ok(value.max(0) as usize)
}

#[cfg(not(target_arch = "wasm32"))]
fn summarize(content: &str) -> String {
    let trimmed = content.trim();
    if trimmed.is_empty() {
        return "空内容".to_string();
    }
    let mut summary = trimmed.chars().take(120).collect::<String>();
    if trimmed.chars().count() > 120 {
        summary.push('…');
    }
    summary
}

#[cfg(not(target_arch = "wasm32"))]
fn token_estimate(content: &str) -> usize {
    content.chars().count().div_ceil(4)
}

#[cfg(not(target_arch = "wasm32"))]
fn query_error(err: impl std::fmt::Display) -> KnowledgeGraphError {
    KnowledgeGraphError::new(err.to_string())
}
