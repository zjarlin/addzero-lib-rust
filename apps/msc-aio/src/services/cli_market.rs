use std::{future::Future, pin::Pin, rc::Rc};

#[cfg(not(target_arch = "wasm32"))]
use std::{
    collections::{BTreeMap, BTreeSet},
    io::{Cursor, Read, Write},
    time::Duration,
};

use addzero_cli_market_contract::{
    CliDocRef, CliEntryKind, CliImportFormat, CliImportMode, CliInstallMethod, CliInstallerKind,
    CliLocale, CliLocaleText, CliMarketCatalog, CliMarketEntry, CliMarketEntryUpsert,
    CliMarketExportArtifact, CliMarketExportRequest, CliMarketImportJob, CliMarketImportJobDetail,
    CliMarketImportReport, CliMarketImportRequest, CliMarketImportRowReport,
    CliMarketInstallHistoryItem, CliMarketInstallRequest, CliMarketInstallResult,
    CliMarketSourceType, CliMarketStatus, CliMarketSummary, CliPlatform, CliRegistryCompatEntry,
};
#[cfg(not(target_arch = "wasm32"))]
use chrono::{DateTime, Utc};
#[cfg(not(target_arch = "wasm32"))]
use quick_xml::{Reader, events::Event};
use serde::{Deserialize, Serialize};
#[cfg(not(target_arch = "wasm32"))]
use sqlx::{
    Row,
    postgres::{PgPool, PgPoolOptions},
};
use thiserror::Error;
#[cfg(not(target_arch = "wasm32"))]
use tokio::process::Command;
#[cfg(not(target_arch = "wasm32"))]
use uuid::Uuid;
#[cfg(not(target_arch = "wasm32"))]
use zip::{ZipArchive, ZipWriter, write::SimpleFileOptions};

pub type LocalBoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + 'a>>;

#[derive(Clone, Debug, Error, PartialEq, Eq)]
pub enum CliMarketError {
    #[error("{0}")]
    Message(String),
}

pub type CliMarketResult<T> = Result<T, CliMarketError>;

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct CliMarketPublicRegistry {
    pub schema_version: String,
    pub generated_at: String,
    pub entries: Vec<CliRegistryCompatEntry>,
}

pub trait CliMarketApi: 'static {
    fn catalog(&self) -> LocalBoxFuture<'_, CliMarketResult<CliMarketCatalog>>;
    fn get_entry(&self, id: String) -> LocalBoxFuture<'_, CliMarketResult<Option<CliMarketEntry>>>;
    fn upsert_entry(
        &self,
        input: CliMarketEntryUpsert,
    ) -> LocalBoxFuture<'_, CliMarketResult<CliMarketEntry>>;
    fn import_entries(
        &self,
        input: CliMarketImportRequest,
    ) -> LocalBoxFuture<'_, CliMarketResult<CliMarketImportReport>>;
    fn import_jobs(&self) -> LocalBoxFuture<'_, CliMarketResult<Vec<CliMarketImportJob>>>;
    fn import_job_detail(
        &self,
        id: String,
    ) -> LocalBoxFuture<'_, CliMarketResult<Option<CliMarketImportJobDetail>>>;
    fn export_json(
        &self,
        input: CliMarketExportRequest,
    ) -> LocalBoxFuture<'_, CliMarketResult<CliMarketExportArtifact>>;
    fn export_xlsx(
        &self,
        input: CliMarketExportRequest,
    ) -> LocalBoxFuture<'_, CliMarketResult<CliMarketExportArtifact>>;
    fn install_history(
        &self,
        id: String,
    ) -> LocalBoxFuture<'_, CliMarketResult<Vec<CliMarketInstallHistoryItem>>>;
    fn install_entry(
        &self,
        id: String,
        input: CliMarketInstallRequest,
    ) -> LocalBoxFuture<'_, CliMarketResult<CliMarketInstallResult>>;
    fn publish_entry(&self, id: String) -> LocalBoxFuture<'_, CliMarketResult<CliMarketEntry>>;
    fn archive_entry(&self, id: String) -> LocalBoxFuture<'_, CliMarketResult<CliMarketEntry>>;
}

pub type SharedCliMarketApi = Rc<dyn CliMarketApi>;

pub fn default_cli_market_api() -> SharedCliMarketApi {
    #[cfg(target_arch = "wasm32")]
    {
        Rc::new(BrowserCliMarketApi)
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        Rc::new(EmbeddedCliMarketApi)
    }
}

#[cfg(target_arch = "wasm32")]
struct BrowserCliMarketApi;

#[cfg(target_arch = "wasm32")]
impl CliMarketApi for BrowserCliMarketApi {
    fn catalog(&self) -> LocalBoxFuture<'_, CliMarketResult<CliMarketCatalog>> {
        Box::pin(async move {
            super::browser_http::get_json("/api/cli-market")
                .await
                .map_err(CliMarketError::Message)
        })
    }

    fn get_entry(&self, id: String) -> LocalBoxFuture<'_, CliMarketResult<Option<CliMarketEntry>>> {
        Box::pin(async move {
            super::browser_http::get_json(&format!("/api/cli-market/{id}"))
                .await
                .map_err(CliMarketError::Message)
        })
    }

    fn upsert_entry(
        &self,
        input: CliMarketEntryUpsert,
    ) -> LocalBoxFuture<'_, CliMarketResult<CliMarketEntry>> {
        Box::pin(async move {
            super::browser_http::post_json("/api/cli-market/upsert", &input)
                .await
                .map_err(CliMarketError::Message)
        })
    }

    fn import_entries(
        &self,
        input: CliMarketImportRequest,
    ) -> LocalBoxFuture<'_, CliMarketResult<CliMarketImportReport>> {
        Box::pin(async move {
            super::browser_http::post_json("/api/cli-market/import", &input)
                .await
                .map_err(CliMarketError::Message)
        })
    }

    fn import_jobs(&self) -> LocalBoxFuture<'_, CliMarketResult<Vec<CliMarketImportJob>>> {
        Box::pin(async move {
            super::browser_http::get_json("/api/cli-market/import-jobs")
                .await
                .map_err(CliMarketError::Message)
        })
    }

    fn import_job_detail(
        &self,
        id: String,
    ) -> LocalBoxFuture<'_, CliMarketResult<Option<CliMarketImportJobDetail>>> {
        Box::pin(async move {
            super::browser_http::get_json(&format!("/api/cli-market/import-jobs/{id}"))
                .await
                .map_err(CliMarketError::Message)
        })
    }

    fn export_json(
        &self,
        input: CliMarketExportRequest,
    ) -> LocalBoxFuture<'_, CliMarketResult<CliMarketExportArtifact>> {
        Box::pin(async move {
            super::browser_http::post_json("/api/cli-market/export/json", &input)
                .await
                .map_err(CliMarketError::Message)
        })
    }

    fn export_xlsx(
        &self,
        input: CliMarketExportRequest,
    ) -> LocalBoxFuture<'_, CliMarketResult<CliMarketExportArtifact>> {
        Box::pin(async move {
            super::browser_http::post_json("/api/cli-market/export/xlsx", &input)
                .await
                .map_err(CliMarketError::Message)
        })
    }

    fn install_history(
        &self,
        id: String,
    ) -> LocalBoxFuture<'_, CliMarketResult<Vec<CliMarketInstallHistoryItem>>> {
        Box::pin(async move {
            super::browser_http::get_json(&format!("/api/cli-market/{id}/install-history"))
                .await
                .map_err(CliMarketError::Message)
        })
    }

    fn install_entry(
        &self,
        id: String,
        input: CliMarketInstallRequest,
    ) -> LocalBoxFuture<'_, CliMarketResult<CliMarketInstallResult>> {
        Box::pin(async move {
            super::browser_http::post_json(&format!("/api/cli-market/{id}/install"), &input)
                .await
                .map_err(CliMarketError::Message)
        })
    }

    fn publish_entry(&self, id: String) -> LocalBoxFuture<'_, CliMarketResult<CliMarketEntry>> {
        Box::pin(async move {
            super::browser_http::post_json(
                &format!("/api/cli-market/{id}/publish"),
                &serde_json::json!({}),
            )
            .await
            .map_err(CliMarketError::Message)
        })
    }

    fn archive_entry(&self, id: String) -> LocalBoxFuture<'_, CliMarketResult<CliMarketEntry>> {
        Box::pin(async move {
            super::browser_http::post_json(
                &format!("/api/cli-market/{id}/archive"),
                &serde_json::json!({}),
            )
            .await
            .map_err(CliMarketError::Message)
        })
    }
}

#[cfg(not(target_arch = "wasm32"))]
#[derive(Clone)]
struct EmbeddedCliMarketApi;

#[cfg(not(target_arch = "wasm32"))]
impl CliMarketApi for EmbeddedCliMarketApi {
    fn catalog(&self) -> LocalBoxFuture<'_, CliMarketResult<CliMarketCatalog>> {
        Box::pin(async move { catalog_on_server().await })
    }

    fn get_entry(&self, id: String) -> LocalBoxFuture<'_, CliMarketResult<Option<CliMarketEntry>>> {
        Box::pin(async move { get_entry_on_server(&id).await })
    }

    fn upsert_entry(
        &self,
        input: CliMarketEntryUpsert,
    ) -> LocalBoxFuture<'_, CliMarketResult<CliMarketEntry>> {
        Box::pin(async move { upsert_entry_on_server(input).await })
    }

    fn import_entries(
        &self,
        input: CliMarketImportRequest,
    ) -> LocalBoxFuture<'_, CliMarketResult<CliMarketImportReport>> {
        Box::pin(async move { import_entries_on_server(input).await })
    }

    fn import_jobs(&self) -> LocalBoxFuture<'_, CliMarketResult<Vec<CliMarketImportJob>>> {
        Box::pin(async move { import_jobs_on_server().await })
    }

    fn import_job_detail(
        &self,
        id: String,
    ) -> LocalBoxFuture<'_, CliMarketResult<Option<CliMarketImportJobDetail>>> {
        Box::pin(async move { import_job_detail_on_server(&id).await })
    }

    fn export_json(
        &self,
        input: CliMarketExportRequest,
    ) -> LocalBoxFuture<'_, CliMarketResult<CliMarketExportArtifact>> {
        Box::pin(async move { export_json_on_server(input).await })
    }

    fn export_xlsx(
        &self,
        input: CliMarketExportRequest,
    ) -> LocalBoxFuture<'_, CliMarketResult<CliMarketExportArtifact>> {
        Box::pin(async move { export_xlsx_on_server(input).await })
    }

    fn install_history(
        &self,
        id: String,
    ) -> LocalBoxFuture<'_, CliMarketResult<Vec<CliMarketInstallHistoryItem>>> {
        Box::pin(async move { install_history_on_server(&id).await })
    }

    fn install_entry(
        &self,
        id: String,
        input: CliMarketInstallRequest,
    ) -> LocalBoxFuture<'_, CliMarketResult<CliMarketInstallResult>> {
        Box::pin(async move { install_entry_on_server(&id, input).await })
    }

    fn publish_entry(&self, id: String) -> LocalBoxFuture<'_, CliMarketResult<CliMarketEntry>> {
        Box::pin(async move { change_status_on_server(&id, CliMarketStatus::Published).await })
    }

    fn archive_entry(&self, id: String) -> LocalBoxFuture<'_, CliMarketResult<CliMarketEntry>> {
        Box::pin(async move { change_status_on_server(&id, CliMarketStatus::Archived).await })
    }
}

#[cfg(not(target_arch = "wasm32"))]
const CLI_MARKET_SCHEMA_SQL: &str =
    include_str!("../server/migrations/0002_clianything_market.sql");

#[cfg(not(target_arch = "wasm32"))]
#[derive(Clone)]
pub struct CliMarketService {
    pg: Option<CliMarketRepo>,
}

#[cfg(not(target_arch = "wasm32"))]
impl CliMarketService {
    pub async fn try_attach(database_url: Option<&str>) -> Self {
        let Some(url) = database_url.filter(|url| !url.is_empty()) else {
            return Self { pg: None };
        };
        match CliMarketRepo::connect(url).await {
            Ok(repo) => match repo.ensure_schema().await {
                Ok(()) => Self { pg: Some(repo) },
                Err(err) => {
                    log::warn!("cli market schema bootstrap failed: {err}");
                    Self { pg: None }
                }
            },
            Err(err) => {
                log::warn!("cli market postgres connect failed: {err}");
                Self { pg: None }
            }
        }
    }

    pub fn is_pg_online(&self) -> bool {
        self.pg.is_some()
    }

    fn repo(&self) -> CliMarketResult<&CliMarketRepo> {
        self.pg.as_ref().ok_or_else(|| {
            CliMarketError::Message(
                "CLI 市场 PostgreSQL 未就绪：请设置 MSC_AIO_DATABASE_URL 或 DATABASE_URL"
                    .to_string(),
            )
        })
    }

    pub async fn catalog(&self) -> CliMarketResult<CliMarketCatalog> {
        self.repo()?.catalog().await
    }

    pub async fn get_entry(&self, id: &str) -> CliMarketResult<Option<CliMarketEntry>> {
        self.repo()?.get_entry(id).await
    }

    pub async fn upsert_entry(
        &self,
        input: CliMarketEntryUpsert,
    ) -> CliMarketResult<CliMarketEntry> {
        self.repo()?.upsert_entry(input).await
    }

    pub async fn import_entries(
        &self,
        input: CliMarketImportRequest,
    ) -> CliMarketResult<CliMarketImportReport> {
        self.repo()?.import_entries(input).await
    }

    pub async fn list_jobs(&self) -> CliMarketResult<Vec<CliMarketImportJob>> {
        self.repo()?.list_jobs().await
    }

    pub async fn job_detail(&self, id: &str) -> CliMarketResult<Option<CliMarketImportJobDetail>> {
        self.repo()?.job_detail(id).await
    }

    pub async fn export_json(
        &self,
        input: CliMarketExportRequest,
    ) -> CliMarketResult<CliMarketExportArtifact> {
        self.repo()?.export_json(input).await
    }

    pub async fn export_xlsx(
        &self,
        input: CliMarketExportRequest,
    ) -> CliMarketResult<CliMarketExportArtifact> {
        self.repo()?.export_xlsx(input).await
    }

    pub async fn install_history(
        &self,
        id: &str,
    ) -> CliMarketResult<Vec<CliMarketInstallHistoryItem>> {
        self.repo()?.install_history(id).await
    }

    pub async fn install_entry(
        &self,
        id: &str,
        input: CliMarketInstallRequest,
    ) -> CliMarketResult<CliMarketInstallResult> {
        self.repo()?.install_entry(id, input).await
    }

    pub async fn public_registry_json_bytes(&self) -> CliMarketResult<Vec<u8>> {
        let artifact = self
            .repo()?
            .export_json(CliMarketExportRequest {
                only_published: true,
                locale: None,
            })
            .await?;
        artifact
            .decode()
            .map_err(|err| CliMarketError::Message(err.to_string()))
    }

    pub async fn public_registry_xlsx_bytes(&self) -> CliMarketResult<Vec<u8>> {
        let artifact = self
            .repo()?
            .export_xlsx(CliMarketExportRequest {
                only_published: true,
                locale: None,
            })
            .await?;
        artifact
            .decode()
            .map_err(|err| CliMarketError::Message(err.to_string()))
    }

    pub async fn change_status(
        &self,
        id: &str,
        status: CliMarketStatus,
    ) -> CliMarketResult<CliMarketEntry> {
        self.repo()?.change_status(id, status).await
    }
}

#[cfg(not(target_arch = "wasm32"))]
#[derive(Clone)]
struct CliMarketRepo {
    pool: PgPool,
}

#[cfg(not(target_arch = "wasm32"))]
impl CliMarketRepo {
    async fn connect(database_url: &str) -> CliMarketResult<Self> {
        let pool = PgPoolOptions::new()
            .max_connections(4)
            .acquire_timeout(Duration::from_secs(3))
            .connect(database_url)
            .await
            .map_err(|err| CliMarketError::Message(format!("连接 PostgreSQL 失败：{err}")))?;
        Ok(Self { pool })
    }

    async fn ensure_schema(&self) -> CliMarketResult<()> {
        sqlx::query(CLI_MARKET_SCHEMA_SQL)
            .execute(&self.pool)
            .await
            .map_err(|err| CliMarketError::Message(format!("初始化 CLI 市场表失败：{err}")))?;
        Ok(())
    }

    async fn catalog(&self) -> CliMarketResult<CliMarketCatalog> {
        let entries = self.list_entries(None).await?;
        let jobs = self.list_jobs().await?;
        let categories: i64 =
            sqlx::query_scalar("SELECT COUNT(*)::BIGINT FROM clianything_market_category")
                .fetch_one(&self.pool)
                .await
                .unwrap_or_default();
        let published_entries = entries
            .iter()
            .filter(|entry| entry.status == CliMarketStatus::Published)
            .count();
        Ok(CliMarketCatalog {
            schema_version: "1".to_string(),
            summary: CliMarketSummary {
                total_entries: entries.len(),
                published_entries,
                import_jobs: jobs.len(),
                categories: usize::try_from(categories).unwrap_or_default(),
            },
            entries,
        })
    }

    async fn list_entries(
        &self,
        status: Option<CliMarketStatus>,
    ) -> CliMarketResult<Vec<CliMarketEntry>> {
        let rows = if let Some(status) = status {
            sqlx::query(
                r#"SELECT id, slug, status, source_type, entry_kind, vendor_name, latest_version,
                          homepage_url, repo_url, docs_url, entry_point, category_code, raw,
                          created_at, updated_at
                   FROM clianything_market
                   WHERE status = $1
                   ORDER BY slug"#,
            )
            .bind(status.code())
            .fetch_all(&self.pool)
            .await
        } else {
            sqlx::query(
                r#"SELECT id, slug, status, source_type, entry_kind, vendor_name, latest_version,
                          homepage_url, repo_url, docs_url, entry_point, category_code, raw,
                          created_at, updated_at
                   FROM clianything_market
                   ORDER BY slug"#,
            )
            .fetch_all(&self.pool)
            .await
        }
        .map_err(|err| CliMarketError::Message(format!("加载 CLI 市场失败：{err}")))?;

        let mut entries = Vec::with_capacity(rows.len());
        for row in rows {
            entries.push(self.inflate_entry(row).await?);
        }
        Ok(entries)
    }

    async fn get_entry(&self, id: &str) -> CliMarketResult<Option<CliMarketEntry>> {
        let parsed = parse_uuid(id)?;
        let row = sqlx::query(
            r#"SELECT id, slug, status, source_type, entry_kind, vendor_name, latest_version,
                      homepage_url, repo_url, docs_url, entry_point, category_code, raw,
                      created_at, updated_at
               FROM clianything_market
               WHERE id = $1"#,
        )
        .bind(parsed)
        .fetch_optional(&self.pool)
        .await
        .map_err(|err| CliMarketError::Message(format!("查询 CLI 条目失败：{err}")))?;

        match row {
            Some(row) => Ok(Some(self.inflate_entry(row).await?)),
            None => Ok(None),
        }
    }

    async fn upsert_entry(&self, input: CliMarketEntryUpsert) -> CliMarketResult<CliMarketEntry> {
        validate_upsert(&input)?;
        let id = input
            .id
            .as_deref()
            .map(parse_uuid)
            .transpose()?
            .unwrap_or_else(Uuid::new_v4);
        let slug = input.slug.trim().to_string();
        let row = sqlx::query(
            r#"INSERT INTO clianything_market (
                    id, slug, status, source_type, entry_kind, vendor_name, latest_version,
                    homepage_url, repo_url, docs_url, entry_point, category_code, raw, updated_at
               ) VALUES (
                    $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, NOW()
               )
               ON CONFLICT (slug) DO UPDATE SET
                    status = EXCLUDED.status,
                    source_type = EXCLUDED.source_type,
                    entry_kind = EXCLUDED.entry_kind,
                    vendor_name = EXCLUDED.vendor_name,
                    latest_version = EXCLUDED.latest_version,
                    homepage_url = EXCLUDED.homepage_url,
                    repo_url = EXCLUDED.repo_url,
                    docs_url = EXCLUDED.docs_url,
                    entry_point = EXCLUDED.entry_point,
                    category_code = EXCLUDED.category_code,
                    raw = EXCLUDED.raw,
                    updated_at = NOW()
               RETURNING id"#,
        )
        .bind(id)
        .bind(&slug)
        .bind(input.status.code())
        .bind(source_type_code(input.source_type))
        .bind(entry_kind_code(input.entry_kind))
        .bind(input.vendor_name.trim())
        .bind(input.latest_version.trim())
        .bind(input.homepage_url.trim())
        .bind(input.repo_url.trim())
        .bind(input.docs_url.trim())
        .bind(input.entry_point.trim())
        .bind(input.category_code.trim())
        .bind(input.raw.clone())
        .fetch_one(&self.pool)
        .await
        .map_err(|err| CliMarketError::Message(format!("保存 CLI 条目失败：{err}")))?;
        let market_id: Uuid = row.try_get("id").unwrap_or(id);

        self.upsert_category(input.category_code.trim()).await?;
        self.replace_locales(market_id, &input.locales).await?;
        self.replace_tags(market_id, &input.tags).await?;
        self.replace_install_methods(market_id, &input.install_methods)
            .await?;
        self.replace_doc_refs(market_id, &input.doc_refs).await?;

        self.get_entry(&market_id.to_string())
            .await?
            .ok_or_else(|| CliMarketError::Message("保存后未找到 CLI 条目".to_string()))
    }

    async fn change_status(
        &self,
        id: &str,
        status: CliMarketStatus,
    ) -> CliMarketResult<CliMarketEntry> {
        let parsed = parse_uuid(id)?;
        sqlx::query("UPDATE clianything_market SET status = $2, updated_at = NOW() WHERE id = $1")
            .bind(parsed)
            .bind(status.code())
            .execute(&self.pool)
            .await
            .map_err(|err| CliMarketError::Message(format!("更新条目状态失败：{err}")))?;

        if status == CliMarketStatus::Published {
            let published = self.list_entries(Some(CliMarketStatus::Published)).await?;
            sqlx::query(
                "INSERT INTO clianything_market_release (schema_version, entry_count, payload) VALUES ($1, $2, $3)",
            )
            .bind("1")
            .bind(i32::try_from(published.len()).unwrap_or_default())
            .bind(serde_json::to_value(&published).unwrap_or_else(|_| serde_json::json!([])))
            .execute(&self.pool)
            .await
            .map_err(|err| CliMarketError::Message(format!("记录发布快照失败：{err}")))?;
        }

        self.get_entry(id)
            .await?
            .ok_or_else(|| CliMarketError::Message("未找到要更新的 CLI 条目".to_string()))
    }

    async fn list_jobs(&self) -> CliMarketResult<Vec<CliMarketImportJob>> {
        let rows = sqlx::query(
            r#"SELECT id, file_name, format, mode, submitted_by, total_rows, success_rows,
                      failed_rows, status, created_at
               FROM clianything_market_import_job
               ORDER BY created_at DESC"#,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|err| CliMarketError::Message(format!("加载导入任务失败：{err}")))?;
        Ok(rows.into_iter().map(row_to_import_job).collect())
    }

    async fn job_detail(&self, id: &str) -> CliMarketResult<Option<CliMarketImportJobDetail>> {
        let parsed = parse_uuid(id)?;
        let row = sqlx::query(
            r#"SELECT id, file_name, format, mode, submitted_by, total_rows, success_rows,
                      failed_rows, status, created_at
               FROM clianything_market_import_job
               WHERE id = $1"#,
        )
        .bind(parsed)
        .fetch_optional(&self.pool)
        .await
        .map_err(|err| CliMarketError::Message(format!("加载导入任务详情失败：{err}")))?;
        let Some(row) = row else {
            return Ok(None);
        };

        let rows = sqlx::query(
            r#"SELECT row_index, slug, success, error_message, market_id
               FROM clianything_market_import_row
               WHERE job_id = $1
               ORDER BY row_index"#,
        )
        .bind(parsed)
        .fetch_all(&self.pool)
        .await
        .map_err(|err| CliMarketError::Message(format!("加载导入行详情失败：{err}")))?;

        Ok(Some(CliMarketImportJobDetail {
            job: row_to_import_job(row),
            rows: rows.into_iter().map(row_to_import_row_report).collect(),
        }))
    }

    async fn import_entries(
        &self,
        input: CliMarketImportRequest,
    ) -> CliMarketResult<CliMarketImportReport> {
        let bytes = input
            .decode_payload()
            .map_err(|err| CliMarketError::Message(err.to_string()))?;
        let rows = match input.format {
            CliImportFormat::Json => parse_json_rows(&bytes, input.mode)?,
            CliImportFormat::Xlsx => parse_xlsx_rows(&bytes)?,
        };
        let job_id = Uuid::new_v4();
        let mut reports = Vec::with_capacity(rows.len());
        let mut success_rows = 0usize;
        for (index, row) in rows.into_iter().enumerate() {
            let row_slug = row.slug.clone();
            match self.upsert_entry(row).await {
                Ok(saved) => {
                    success_rows += 1;
                    reports.push(CliMarketImportRowReport {
                        row_index: index + 1,
                        slug: saved.slug.clone(),
                        success: true,
                        error: None,
                        market_id: Some(saved.id.clone()),
                    });
                }
                Err(err) => {
                    reports.push(CliMarketImportRowReport {
                        row_index: index + 1,
                        slug: row_slug,
                        success: false,
                        error: Some(err.to_string()),
                        market_id: None,
                    });
                }
            }
        }
        let failed_rows = reports.len().saturating_sub(success_rows);

        sqlx::query(
            r#"INSERT INTO clianything_market_import_job
               (id, file_name, format, mode, submitted_by, total_rows, success_rows, failed_rows, status)
               VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)"#,
        )
        .bind(job_id)
        .bind(input.file_name.trim())
        .bind(import_format_code(input.format))
        .bind(import_mode_code(input.mode))
        .bind(input.submitted_by.trim())
        .bind(i32::try_from(reports.len()).unwrap_or_default())
        .bind(i32::try_from(success_rows).unwrap_or_default())
        .bind(i32::try_from(failed_rows).unwrap_or_default())
        .bind("completed")
        .execute(&self.pool)
        .await
        .map_err(|err| CliMarketError::Message(format!("保存导入任务失败：{err}")))?;

        for report in &reports {
            sqlx::query(
                r#"INSERT INTO clianything_market_import_row
                   (job_id, row_index, slug, success, error_message, market_id)
                   VALUES ($1, $2, $3, $4, $5, $6)"#,
            )
            .bind(job_id)
            .bind(i32::try_from(report.row_index).unwrap_or_default())
            .bind(&report.slug)
            .bind(report.success)
            .bind(report.error.clone())
            .bind(
                report
                    .market_id
                    .as_deref()
                    .map(parse_uuid)
                    .transpose()
                    .map_err(|err| CliMarketError::Message(err.to_string()))?,
            )
            .execute(&self.pool)
            .await
            .map_err(|err| CliMarketError::Message(format!("保存导入行失败：{err}")))?;
        }

        Ok(CliMarketImportReport {
            job_id: job_id.to_string(),
            format: input.format,
            mode: input.mode,
            total_rows: reports.len(),
            success_rows,
            failed_rows,
            rows: reports,
        })
    }

    async fn export_json(
        &self,
        input: CliMarketExportRequest,
    ) -> CliMarketResult<CliMarketExportArtifact> {
        let registry = self.public_registry(input.only_published).await?;
        let bytes = serde_json::to_vec_pretty(&registry)
            .map_err(|err| CliMarketError::Message(format!("导出 JSON 失败：{err}")))?;
        Ok(CliMarketExportArtifact::encode(
            "cli-market-registry.json",
            "application/json",
            bytes,
        ))
    }

    async fn export_xlsx(
        &self,
        input: CliMarketExportRequest,
    ) -> CliMarketResult<CliMarketExportArtifact> {
        let entries = if input.only_published {
            self.list_entries(Some(CliMarketStatus::Published)).await?
        } else {
            self.list_entries(None).await?
        };
        let bytes = write_xlsx(&entries)?;
        Ok(CliMarketExportArtifact::encode(
            "cli-market-registry.xlsx",
            "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
            bytes,
        ))
    }

    async fn public_registry(
        &self,
        only_published: bool,
    ) -> CliMarketResult<CliMarketPublicRegistry> {
        let entries = if only_published {
            self.list_entries(Some(CliMarketStatus::Published)).await?
        } else {
            self.list_entries(None).await?
        };
        Ok(CliMarketPublicRegistry {
            schema_version: "1".to_string(),
            generated_at: Utc::now().to_rfc3339(),
            entries: entries
                .iter()
                .filter_map(entry_to_registry_compat)
                .collect(),
        })
    }

    async fn install_entry(
        &self,
        id: &str,
        input: CliMarketInstallRequest,
    ) -> CliMarketResult<CliMarketInstallResult> {
        let entry = self
            .get_entry(id)
            .await?
            .ok_or_else(|| CliMarketError::Message("未找到要安装的 CLI 条目".to_string()))?;
        let host_platform = runtime_host_platform();
        let method = select_install_method(&entry, input.method_id.as_deref(), host_platform)?;
        if method.command_template.trim().is_empty() {
            return Err(CliMarketError::Message(
                "当前条目没有可执行的安装命令".to_string(),
            ));
        }
        validate_install_method(&method)?;

        let started_at = Utc::now();
        let output = run_install_command(&method.command_template).await?;
        let finished_at = Utc::now();
        let method_uuid = method.id.as_deref().map(parse_uuid).transpose()?;
        let result = CliMarketInstallResult {
            entry_id: entry.id.clone(),
            slug: entry.slug.clone(),
            method_id: method.id.clone().unwrap_or_default(),
            platform: method.platform,
            installer_kind: method.installer_kind,
            command: method.command_template.clone(),
            success: output.status.success(),
            exit_code: output.status.code(),
            stdout: String::from_utf8_lossy(&output.stdout).to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
            started_at: started_at.to_rfc3339(),
            finished_at: finished_at.to_rfc3339(),
        };
        self.append_install_history(parse_uuid(id)?, method_uuid, &result)
            .await?;

        Ok(result)
    }

    async fn install_history(&self, id: &str) -> CliMarketResult<Vec<CliMarketInstallHistoryItem>> {
        let market_id = parse_uuid(id)?;
        let slug = self
            .get_entry(id)
            .await?
            .map(|entry| entry.slug)
            .unwrap_or_default();
        let rows = sqlx::query(
            r#"SELECT id, market_id, method_id, platform, installer_kind, command, success,
                      exit_code, started_at, finished_at, created_at
               FROM clianything_market_install_history
               WHERE market_id = $1
               ORDER BY created_at DESC"#,
        )
        .bind(market_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|err| CliMarketError::Message(format!("读取安装历史失败：{err}")))?;
        Ok(rows
            .into_iter()
            .map(|row| row_to_install_history_item(row, id, &slug))
            .collect())
    }

    async fn inflate_entry(&self, row: sqlx::postgres::PgRow) -> CliMarketResult<CliMarketEntry> {
        let market_id: Uuid = row
            .try_get("id")
            .map_err(|err| CliMarketError::Message(format!("读取 CLI 条目主键失败：{err}")))?;
        let locales = sqlx::query(
            r#"SELECT locale, display_name, summary, description_md, install_guide_md,
                      docs_summary, requires_text, install_command
               FROM clianything_market_i18n
               WHERE market_id = $1
               ORDER BY locale"#,
        )
        .bind(market_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|err| CliMarketError::Message(format!("读取多语言字段失败：{err}")))?;
        let tags = sqlx::query(
            "SELECT tag_code FROM clianything_market_tag_rel WHERE market_id = $1 ORDER BY tag_code",
        )
        .bind(market_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|err| CliMarketError::Message(format!("读取标签失败：{err}")))?;
        let methods = sqlx::query(
            r#"SELECT id, platform, installer_kind, package_id, command_template,
                      validation_note, priority
               FROM clianything_market_install_method
               WHERE market_id = $1
               ORDER BY priority DESC, installer_kind"#,
        )
        .bind(market_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|err| CliMarketError::Message(format!("读取安装方式失败：{err}")))?;
        let docs = sqlx::query(
            r#"SELECT id, locale, title, url, version, source_label, summary
               FROM clianything_market_doc_ref
               WHERE market_id = $1
               ORDER BY locale, title"#,
        )
        .bind(market_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|err| CliMarketError::Message(format!("读取文档引用失败：{err}")))?;

        Ok(CliMarketEntry {
            id: market_id.to_string(),
            slug: row.try_get("slug").unwrap_or_default(),
            status: parse_status(&row.try_get::<String, _>("status").unwrap_or_default())?,
            source_type: parse_source_type(
                &row.try_get::<String, _>("source_type").unwrap_or_default(),
            )?,
            entry_kind: parse_entry_kind(
                &row.try_get::<String, _>("entry_kind").unwrap_or_default(),
            )?,
            vendor_name: row.try_get("vendor_name").unwrap_or_default(),
            latest_version: row.try_get("latest_version").unwrap_or_default(),
            homepage_url: row.try_get("homepage_url").unwrap_or_default(),
            repo_url: row.try_get("repo_url").unwrap_or_default(),
            docs_url: row.try_get("docs_url").unwrap_or_default(),
            entry_point: row.try_get("entry_point").unwrap_or_default(),
            category_code: row.try_get("category_code").unwrap_or_default(),
            tags: tags
                .into_iter()
                .map(|tag| tag.try_get("tag_code").unwrap_or_default())
                .collect(),
            locales: locales
                .into_iter()
                .map(|locale| CliLocaleText {
                    locale: parse_locale(
                        &locale.try_get::<String, _>("locale").unwrap_or_default(),
                    )
                    .unwrap_or(CliLocale::ZhCn),
                    display_name: locale.try_get("display_name").unwrap_or_default(),
                    summary: locale.try_get("summary").unwrap_or_default(),
                    description_md: locale.try_get("description_md").unwrap_or_default(),
                    install_guide_md: locale.try_get("install_guide_md").unwrap_or_default(),
                    docs_summary: locale.try_get("docs_summary").unwrap_or_default(),
                    requires_text: locale.try_get("requires_text").unwrap_or_default(),
                    install_command: locale.try_get("install_command").unwrap_or_default(),
                })
                .collect(),
            install_methods: methods
                .into_iter()
                .map(|method| CliInstallMethod {
                    id: Some(
                        method
                            .try_get::<Uuid, _>("id")
                            .unwrap_or_else(|_| Uuid::new_v4())
                            .to_string(),
                    ),
                    platform: parse_platform(
                        &method.try_get::<String, _>("platform").unwrap_or_default(),
                    )
                    .unwrap_or(CliPlatform::CrossPlatform),
                    installer_kind: parse_installer_kind(
                        &method
                            .try_get::<String, _>("installer_kind")
                            .unwrap_or_default(),
                    )
                    .unwrap_or(CliInstallerKind::Custom),
                    package_id: method.try_get("package_id").unwrap_or_default(),
                    command_template: method.try_get("command_template").unwrap_or_default(),
                    validation_note: method.try_get("validation_note").unwrap_or_default(),
                    priority: method.try_get("priority").unwrap_or_default(),
                })
                .collect(),
            doc_refs: docs
                .into_iter()
                .map(|doc| CliDocRef {
                    id: Some(
                        doc.try_get::<Uuid, _>("id")
                            .unwrap_or_else(|_| Uuid::new_v4())
                            .to_string(),
                    ),
                    locale: parse_locale(&doc.try_get::<String, _>("locale").unwrap_or_default())
                        .unwrap_or(CliLocale::ZhCn),
                    title: doc.try_get("title").unwrap_or_default(),
                    url: doc.try_get("url").unwrap_or_default(),
                    version: doc.try_get("version").unwrap_or_default(),
                    source_label: doc.try_get("source_label").unwrap_or_default(),
                    summary: doc.try_get("summary").unwrap_or_default(),
                })
                .collect(),
            raw: row.try_get("raw").unwrap_or_else(|_| serde_json::json!({})),
            created_at: row
                .try_get::<DateTime<Utc>, _>("created_at")
                .ok()
                .map(|value| value.to_rfc3339()),
            updated_at: row
                .try_get::<DateTime<Utc>, _>("updated_at")
                .ok()
                .map(|value| value.to_rfc3339()),
        })
    }

    async fn upsert_category(&self, code: &str) -> CliMarketResult<()> {
        if code.trim().is_empty() {
            return Ok(());
        }
        sqlx::query(
            r#"INSERT INTO clianything_market_category (code, label_zh, label_en, updated_at)
               VALUES ($1, $1, $1, NOW())
               ON CONFLICT (code) DO UPDATE SET updated_at = NOW()"#,
        )
        .bind(code)
        .execute(&self.pool)
        .await
        .map_err(|err| CliMarketError::Message(format!("保存分类失败：{err}")))?;
        Ok(())
    }

    async fn replace_locales(
        &self,
        market_id: Uuid,
        locales: &[CliLocaleText],
    ) -> CliMarketResult<()> {
        sqlx::query("DELETE FROM clianything_market_i18n WHERE market_id = $1")
            .bind(market_id)
            .execute(&self.pool)
            .await
            .map_err(|err| CliMarketError::Message(format!("清理多语言字段失败：{err}")))?;
        for locale in locales {
            sqlx::query(
                r#"INSERT INTO clianything_market_i18n
                   (market_id, locale, display_name, summary, description_md, install_guide_md,
                    docs_summary, requires_text, install_command)
                   VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)"#,
            )
            .bind(market_id)
            .bind(locale.locale.code())
            .bind(locale.display_name.trim())
            .bind(locale.summary.trim())
            .bind(locale.description_md.trim())
            .bind(locale.install_guide_md.trim())
            .bind(locale.docs_summary.trim())
            .bind(locale.requires_text.trim())
            .bind(locale.install_command.trim())
            .execute(&self.pool)
            .await
            .map_err(|err| CliMarketError::Message(format!("保存多语言字段失败：{err}")))?;
        }
        Ok(())
    }

    async fn replace_tags(&self, market_id: Uuid, tags: &[String]) -> CliMarketResult<()> {
        sqlx::query("DELETE FROM clianything_market_tag_rel WHERE market_id = $1")
            .bind(market_id)
            .execute(&self.pool)
            .await
            .map_err(|err| CliMarketError::Message(format!("清理标签关联失败：{err}")))?;
        for tag in clean_tags(tags) {
            sqlx::query(
                "INSERT INTO clianything_market_tag (code) VALUES ($1) ON CONFLICT (code) DO NOTHING",
            )
            .bind(&tag)
            .execute(&self.pool)
            .await
            .map_err(|err| CliMarketError::Message(format!("保存标签失败：{err}")))?;
            sqlx::query(
                "INSERT INTO clianything_market_tag_rel (market_id, tag_code) VALUES ($1, $2)",
            )
            .bind(market_id)
            .bind(&tag)
            .execute(&self.pool)
            .await
            .map_err(|err| CliMarketError::Message(format!("保存标签关联失败：{err}")))?;
        }
        Ok(())
    }

    async fn replace_install_methods(
        &self,
        market_id: Uuid,
        methods: &[CliInstallMethod],
    ) -> CliMarketResult<()> {
        sqlx::query("DELETE FROM clianything_market_install_method WHERE market_id = $1")
            .bind(market_id)
            .execute(&self.pool)
            .await
            .map_err(|err| CliMarketError::Message(format!("清理安装方式失败：{err}")))?;
        for method in methods
            .iter()
            .filter(|method| !method.command_template.trim().is_empty())
        {
            sqlx::query(
                r#"INSERT INTO clianything_market_install_method
                   (id, market_id, platform, installer_kind, package_id, command_template, validation_note, priority)
                   VALUES ($1, $2, $3, $4, $5, $6, $7, $8)"#,
            )
            .bind(
                method
                    .id
                    .as_deref()
                    .map(parse_uuid)
                    .transpose()?
                    .unwrap_or_else(Uuid::new_v4),
            )
            .bind(market_id)
            .bind(method.platform.code())
            .bind(method.installer_kind.code())
            .bind(method.package_id.trim())
            .bind(method.command_template.trim())
            .bind(method.validation_note.trim())
            .bind(method.priority)
            .execute(&self.pool)
            .await
            .map_err(|err| CliMarketError::Message(format!("保存安装方式失败：{err}")))?;
        }
        Ok(())
    }

    async fn replace_doc_refs(&self, market_id: Uuid, docs: &[CliDocRef]) -> CliMarketResult<()> {
        sqlx::query("DELETE FROM clianything_market_doc_ref WHERE market_id = $1")
            .bind(market_id)
            .execute(&self.pool)
            .await
            .map_err(|err| CliMarketError::Message(format!("清理文档引用失败：{err}")))?;
        for doc in docs.iter().filter(|doc| !doc.url.trim().is_empty()) {
            sqlx::query(
                r#"INSERT INTO clianything_market_doc_ref
                   (id, market_id, locale, title, url, version, source_label, summary)
                   VALUES ($1, $2, $3, $4, $5, $6, $7, $8)"#,
            )
            .bind(
                doc.id
                    .as_deref()
                    .map(parse_uuid)
                    .transpose()?
                    .unwrap_or_else(Uuid::new_v4),
            )
            .bind(market_id)
            .bind(doc.locale.code())
            .bind(doc.title.trim())
            .bind(doc.url.trim())
            .bind(doc.version.trim())
            .bind(doc.source_label.trim())
            .bind(doc.summary.trim())
            .execute(&self.pool)
            .await
            .map_err(|err| CliMarketError::Message(format!("保存文档引用失败：{err}")))?;
        }
        Ok(())
    }

    async fn append_install_history(
        &self,
        market_id: Uuid,
        method_id: Option<Uuid>,
        result: &CliMarketInstallResult,
    ) -> CliMarketResult<()> {
        sqlx::query(
            r#"INSERT INTO clianything_market_install_history
               (market_id, method_id, platform, installer_kind, command, success, exit_code, stdout, stderr, started_at, finished_at)
               VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)"#,
        )
        .bind(market_id)
        .bind(method_id)
        .bind(result.platform.code())
        .bind(result.installer_kind.code())
        .bind(&result.command)
        .bind(result.success)
        .bind(result.exit_code)
        .bind(&result.stdout)
        .bind(&result.stderr)
        .bind(parse_rfc3339_utc(&result.started_at)?)
        .bind(parse_rfc3339_utc(&result.finished_at)?)
        .execute(&self.pool)
        .await
        .map_err(|err| CliMarketError::Message(format!("写入安装历史失败：{err}")))?;
        Ok(())
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn catalog_on_server() -> CliMarketResult<CliMarketCatalog> {
    crate::server::services().await.cli_market.catalog().await
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn get_entry_on_server(id: &str) -> CliMarketResult<Option<CliMarketEntry>> {
    crate::server::services()
        .await
        .cli_market
        .get_entry(id)
        .await
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn upsert_entry_on_server(
    input: CliMarketEntryUpsert,
) -> CliMarketResult<CliMarketEntry> {
    crate::server::services()
        .await
        .cli_market
        .upsert_entry(input)
        .await
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn import_entries_on_server(
    input: CliMarketImportRequest,
) -> CliMarketResult<CliMarketImportReport> {
    crate::server::services()
        .await
        .cli_market
        .import_entries(input)
        .await
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn import_jobs_on_server() -> CliMarketResult<Vec<CliMarketImportJob>> {
    crate::server::services().await.cli_market.list_jobs().await
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn import_job_detail_on_server(
    id: &str,
) -> CliMarketResult<Option<CliMarketImportJobDetail>> {
    crate::server::services()
        .await
        .cli_market
        .job_detail(id)
        .await
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn export_json_on_server(
    input: CliMarketExportRequest,
) -> CliMarketResult<CliMarketExportArtifact> {
    crate::server::services()
        .await
        .cli_market
        .export_json(input)
        .await
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn export_xlsx_on_server(
    input: CliMarketExportRequest,
) -> CliMarketResult<CliMarketExportArtifact> {
    crate::server::services()
        .await
        .cli_market
        .export_xlsx(input)
        .await
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn public_registry_json_on_server() -> CliMarketResult<Vec<u8>> {
    crate::server::services()
        .await
        .cli_market
        .public_registry_json_bytes()
        .await
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn public_registry_xlsx_on_server() -> CliMarketResult<Vec<u8>> {
    crate::server::services()
        .await
        .cli_market
        .public_registry_xlsx_bytes()
        .await
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn change_status_on_server(
    id: &str,
    status: CliMarketStatus,
) -> CliMarketResult<CliMarketEntry> {
    crate::server::services()
        .await
        .cli_market
        .change_status(id, status)
        .await
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn install_history_on_server(
    id: &str,
) -> CliMarketResult<Vec<CliMarketInstallHistoryItem>> {
    crate::server::services()
        .await
        .cli_market
        .install_history(id)
        .await
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn install_entry_on_server(
    id: &str,
    input: CliMarketInstallRequest,
) -> CliMarketResult<CliMarketInstallResult> {
    crate::server::services()
        .await
        .cli_market
        .install_entry(id, input)
        .await
}

#[cfg(not(target_arch = "wasm32"))]
async fn run_install_command(command: &str) -> CliMarketResult<std::process::Output> {
    let argv = split_install_command(command)?;
    let Some((program, args)) = argv.split_first() else {
        return Err(CliMarketError::Message("安装命令不能为空".to_string()));
    };

    let mut process = Command::new(program);
    process.args(args);

    process
        .output()
        .await
        .map_err(|err| CliMarketError::Message(format!("执行安装命令失败：{err}")))
}

#[cfg(not(target_arch = "wasm32"))]
fn select_install_method(
    entry: &CliMarketEntry,
    requested_method_id: Option<&str>,
    host_platform: CliPlatform,
) -> CliMarketResult<CliInstallMethod> {
    if let Some(method_id) = requested_method_id {
        return entry
            .install_methods
            .iter()
            .find(|method| method.id.as_deref() == Some(method_id))
            .cloned()
            .ok_or_else(|| CliMarketError::Message("未找到指定的安装方式".to_string()));
    }

    entry
        .install_methods
        .iter()
        .find(|method| method.platform == host_platform)
        .or_else(|| {
            entry
                .install_methods
                .iter()
                .find(|method| method.platform == CliPlatform::CrossPlatform)
        })
        .or_else(|| entry.install_methods.first())
        .cloned()
        .ok_or_else(|| CliMarketError::Message("当前条目没有安装方式".to_string()))
}

#[cfg(not(target_arch = "wasm32"))]
fn validate_install_method(method: &CliInstallMethod) -> CliMarketResult<()> {
    match method.installer_kind {
        CliInstallerKind::Brew
        | CliInstallerKind::Cargo
        | CliInstallerKind::Npm
        | CliInstallerKind::Pipx
        | CliInstallerKind::Bun
        | CliInstallerKind::Winget
        | CliInstallerKind::Scoop => {}
        CliInstallerKind::Curl | CliInstallerKind::Custom => {
            return Err(CliMarketError::Message(
                "当前一键安装只允许受控包管理器命令，暂不允许 curl/custom 直接执行".to_string(),
            ));
        }
    }

    let command = method.command_template.trim();
    if command.is_empty() {
        return Err(CliMarketError::Message("安装命令不能为空".to_string()));
    }
    if command.contains("&&")
        || command.contains("||")
        || command.contains(';')
        || command.contains('\n')
        || command.contains('|')
        || command.contains('>')
        || command.contains('<')
    {
        return Err(CliMarketError::Message(
            "安装命令包含不允许的 shell 组合符，请拆成单条受控安装命令".to_string(),
        ));
    }

    let allowed_program = match method.installer_kind {
        CliInstallerKind::Brew => "brew",
        CliInstallerKind::Cargo => "cargo",
        CliInstallerKind::Npm => "npm",
        CliInstallerKind::Pipx => "pipx",
        CliInstallerKind::Bun => "bun",
        CliInstallerKind::Winget => "winget",
        CliInstallerKind::Scoop => "scoop",
        CliInstallerKind::Curl | CliInstallerKind::Custom => unreachable!(),
    };

    let program = split_install_command(command)?
        .into_iter()
        .next()
        .unwrap_or_default();

    if program != allowed_program {
        return Err(CliMarketError::Message(format!(
            "安装命令必须以 `{allowed_program}` 开头"
        )));
    }
    Ok(())
}

#[cfg(not(target_arch = "wasm32"))]
fn split_install_command(command: &str) -> CliMarketResult<Vec<String>> {
    shlex::split(command)
        .ok_or_else(|| CliMarketError::Message("安装命令解析失败，请检查引号和空格".to_string()))
}

#[cfg(not(target_arch = "wasm32"))]
fn runtime_host_platform() -> CliPlatform {
    #[cfg(target_os = "windows")]
    {
        return CliPlatform::Windows;
    }

    #[cfg(target_os = "linux")]
    {
        return CliPlatform::Linux;
    }

    CliPlatform::Macos
}

#[cfg(not(target_arch = "wasm32"))]
fn parse_rfc3339_utc(value: &str) -> CliMarketResult<DateTime<Utc>> {
    chrono::DateTime::parse_from_rfc3339(value)
        .map(|dt| dt.with_timezone(&Utc))
        .map_err(|err| CliMarketError::Message(format!("解析时间失败：{err}")))
}

#[cfg(not(target_arch = "wasm32"))]
fn validate_upsert(input: &CliMarketEntryUpsert) -> CliMarketResult<()> {
    if input.slug.trim().is_empty() {
        return Err(CliMarketError::Message("slug 不能为空".to_string()));
    }
    for locale in CliLocale::ALL {
        let Some(text) = input.locales.iter().find(|text| text.locale == locale) else {
            return Err(CliMarketError::Message(format!(
                "缺少 {} 多语言字段",
                locale.code()
            )));
        };
        if text.display_name.trim().is_empty() {
            return Err(CliMarketError::Message(format!(
                "{} 的显示名称不能为空",
                locale.code()
            )));
        }
    }
    if input
        .install_methods
        .iter()
        .all(|method| method.command_template.trim().is_empty())
    {
        return Err(CliMarketError::Message("至少需要一个安装方式".to_string()));
    }
    Ok(())
}

#[cfg(not(target_arch = "wasm32"))]
fn parse_uuid(value: &str) -> CliMarketResult<Uuid> {
    Uuid::parse_str(value).map_err(|err| CliMarketError::Message(format!("无效 UUID：{err}")))
}

#[cfg(not(target_arch = "wasm32"))]
fn source_type_code(value: CliMarketSourceType) -> &'static str {
    match value {
        CliMarketSourceType::Manual => "manual",
        CliMarketSourceType::ImportJson => "import_json",
        CliMarketSourceType::ImportExcel => "import_excel",
        CliMarketSourceType::SyncExternal => "sync_external",
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn entry_kind_code(value: CliEntryKind) -> &'static str {
    match value {
        CliEntryKind::Cli => "cli",
        CliEntryKind::Wrapper => "wrapper",
        CliEntryKind::Installer => "installer",
        CliEntryKind::Bundle => "bundle",
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn import_format_code(value: CliImportFormat) -> &'static str {
    match value {
        CliImportFormat::Json => "json",
        CliImportFormat::Xlsx => "xlsx",
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn import_mode_code(value: CliImportMode) -> &'static str {
    match value {
        CliImportMode::Native => "native",
        CliImportMode::RegistryCompat => "registry_compat",
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn parse_status(value: &str) -> CliMarketResult<CliMarketStatus> {
    match value {
        "draft" => Ok(CliMarketStatus::Draft),
        "reviewing" => Ok(CliMarketStatus::Reviewing),
        "published" => Ok(CliMarketStatus::Published),
        "archived" => Ok(CliMarketStatus::Archived),
        _ => Err(CliMarketError::Message(format!("未知状态：{value}"))),
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn parse_source_type(value: &str) -> CliMarketResult<CliMarketSourceType> {
    match value {
        "manual" => Ok(CliMarketSourceType::Manual),
        "import_json" => Ok(CliMarketSourceType::ImportJson),
        "import_excel" => Ok(CliMarketSourceType::ImportExcel),
        "sync_external" => Ok(CliMarketSourceType::SyncExternal),
        _ => Err(CliMarketError::Message(format!("未知来源类型：{value}"))),
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn parse_entry_kind(value: &str) -> CliMarketResult<CliEntryKind> {
    match value {
        "cli" => Ok(CliEntryKind::Cli),
        "wrapper" => Ok(CliEntryKind::Wrapper),
        "installer" => Ok(CliEntryKind::Installer),
        "bundle" => Ok(CliEntryKind::Bundle),
        _ => Err(CliMarketError::Message(format!("未知条目类型：{value}"))),
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn parse_locale(value: &str) -> CliMarketResult<CliLocale> {
    match value {
        "zh-CN" => Ok(CliLocale::ZhCn),
        "en-US" => Ok(CliLocale::EnUs),
        _ => Err(CliMarketError::Message(format!("未知 locale：{value}"))),
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn parse_platform(value: &str) -> CliMarketResult<CliPlatform> {
    match value {
        "macos" => Ok(CliPlatform::Macos),
        "windows" => Ok(CliPlatform::Windows),
        "linux" => Ok(CliPlatform::Linux),
        "cross_platform" => Ok(CliPlatform::CrossPlatform),
        _ => Err(CliMarketError::Message(format!("未知平台：{value}"))),
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn parse_installer_kind(value: &str) -> CliMarketResult<CliInstallerKind> {
    match value {
        "brew" => Ok(CliInstallerKind::Brew),
        "bun" => Ok(CliInstallerKind::Bun),
        "npm" => Ok(CliInstallerKind::Npm),
        "cargo" => Ok(CliInstallerKind::Cargo),
        "pipx" => Ok(CliInstallerKind::Pipx),
        "winget" => Ok(CliInstallerKind::Winget),
        "scoop" => Ok(CliInstallerKind::Scoop),
        "curl" => Ok(CliInstallerKind::Curl),
        "custom" => Ok(CliInstallerKind::Custom),
        _ => Err(CliMarketError::Message(format!("未知安装器类型：{value}"))),
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn row_to_import_job(row: sqlx::postgres::PgRow) -> CliMarketImportJob {
    CliMarketImportJob {
        id: row
            .try_get::<Uuid, _>("id")
            .unwrap_or_else(|_| Uuid::new_v4())
            .to_string(),
        file_name: row.try_get("file_name").unwrap_or_default(),
        format: match row
            .try_get::<String, _>("format")
            .unwrap_or_default()
            .as_str()
        {
            "xlsx" => CliImportFormat::Xlsx,
            _ => CliImportFormat::Json,
        },
        mode: match row
            .try_get::<String, _>("mode")
            .unwrap_or_default()
            .as_str()
        {
            "registry_compat" => CliImportMode::RegistryCompat,
            _ => CliImportMode::Native,
        },
        submitted_by: row.try_get("submitted_by").unwrap_or_default(),
        total_rows: usize::try_from(row.try_get::<i32, _>("total_rows").unwrap_or_default())
            .unwrap_or_default(),
        success_rows: usize::try_from(row.try_get::<i32, _>("success_rows").unwrap_or_default())
            .unwrap_or_default(),
        failed_rows: usize::try_from(row.try_get::<i32, _>("failed_rows").unwrap_or_default())
            .unwrap_or_default(),
        status: row.try_get("status").unwrap_or_default(),
        created_at: row
            .try_get::<DateTime<Utc>, _>("created_at")
            .ok()
            .map(|value| value.to_rfc3339()),
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn row_to_import_row_report(row: sqlx::postgres::PgRow) -> CliMarketImportRowReport {
    CliMarketImportRowReport {
        row_index: usize::try_from(row.try_get::<i32, _>("row_index").unwrap_or_default())
            .unwrap_or_default(),
        slug: row.try_get("slug").unwrap_or_default(),
        success: row.try_get("success").unwrap_or_default(),
        error: row.try_get("error_message").ok(),
        market_id: row
            .try_get::<Option<Uuid>, _>("market_id")
            .ok()
            .flatten()
            .map(|id| id.to_string()),
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn row_to_install_history_item(
    row: sqlx::postgres::PgRow,
    entry_id: &str,
    slug: &str,
) -> CliMarketInstallHistoryItem {
    CliMarketInstallHistoryItem {
        id: row
            .try_get::<Uuid, _>("id")
            .unwrap_or_else(|_| Uuid::new_v4())
            .to_string(),
        entry_id: entry_id.to_string(),
        slug: slug.to_string(),
        method_id: row
            .try_get::<Option<Uuid>, _>("method_id")
            .ok()
            .flatten()
            .map(|value| value.to_string()),
        platform: parse_platform(&row.try_get::<String, _>("platform").unwrap_or_default())
            .unwrap_or(CliPlatform::CrossPlatform),
        installer_kind: parse_installer_kind(
            &row.try_get::<String, _>("installer_kind")
                .unwrap_or_default(),
        )
        .unwrap_or(CliInstallerKind::Custom),
        command: row.try_get("command").unwrap_or_default(),
        success: row.try_get("success").unwrap_or_default(),
        exit_code: row.try_get("exit_code").ok(),
        started_at: row
            .try_get::<DateTime<Utc>, _>("started_at")
            .ok()
            .map(|value| value.to_rfc3339())
            .unwrap_or_default(),
        finished_at: row
            .try_get::<DateTime<Utc>, _>("finished_at")
            .ok()
            .map(|value| value.to_rfc3339())
            .unwrap_or_default(),
        created_at: row
            .try_get::<DateTime<Utc>, _>("created_at")
            .ok()
            .map(|value| value.to_rfc3339())
            .unwrap_or_default(),
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn clean_tags(tags: &[String]) -> Vec<String> {
    let mut set = BTreeSet::new();
    for tag in tags {
        let trimmed = tag.trim();
        if !trimmed.is_empty() {
            set.insert(trimmed.to_string());
        }
    }
    set.into_iter().collect()
}

#[cfg(not(target_arch = "wasm32"))]
fn entry_to_registry_compat(entry: &CliMarketEntry) -> Option<CliRegistryCompatEntry> {
    let zh = entry
        .locales
        .iter()
        .find(|locale| locale.locale == CliLocale::ZhCn)?;
    let first_method = entry.install_methods.first()?;
    Some(CliRegistryCompatEntry {
        name: entry.slug.clone(),
        display_name: zh.display_name.clone(),
        version: entry.latest_version.clone(),
        description: zh.summary.clone(),
        requires: zh.requires_text.clone(),
        install_cmd: first_method.command_template.clone(),
        entry_point: entry.entry_point.clone(),
        category: entry.category_code.clone(),
    })
}

#[cfg(not(target_arch = "wasm32"))]
fn parse_json_rows(
    bytes: &[u8],
    mode: CliImportMode,
) -> CliMarketResult<Vec<CliMarketEntryUpsert>> {
    match mode {
        CliImportMode::Native => serde_json::from_slice(bytes)
            .map_err(|err| CliMarketError::Message(format!("解析原生 JSON 失败：{err}"))),
        CliImportMode::RegistryCompat => {
            let entries: Vec<CliRegistryCompatEntry> = serde_json::from_slice(bytes)
                .map_err(|err| CliMarketError::Message(format!("解析兼容 JSON 失败：{err}")))?;
            Ok(entries.into_iter().map(registry_entry_to_upsert).collect())
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn registry_entry_to_upsert(entry: CliRegistryCompatEntry) -> CliMarketEntryUpsert {
    CliMarketEntryUpsert {
        id: None,
        slug: entry.name,
        status: CliMarketStatus::Draft,
        source_type: CliMarketSourceType::ImportJson,
        entry_kind: CliEntryKind::Cli,
        vendor_name: String::new(),
        latest_version: entry.version,
        homepage_url: String::new(),
        repo_url: String::new(),
        docs_url: String::new(),
        entry_point: entry.entry_point,
        category_code: entry.category,
        tags: Vec::new(),
        locales: vec![
            CliLocaleText {
                locale: CliLocale::ZhCn,
                display_name: entry.display_name.clone(),
                summary: entry.description.clone(),
                description_md: String::new(),
                install_guide_md: String::new(),
                docs_summary: String::new(),
                requires_text: entry.requires.clone(),
                install_command: entry.install_cmd.clone(),
            },
            CliLocaleText {
                locale: CliLocale::EnUs,
                display_name: entry.display_name,
                summary: entry.description,
                description_md: String::new(),
                install_guide_md: String::new(),
                docs_summary: String::new(),
                requires_text: entry.requires,
                install_command: entry.install_cmd.clone(),
            },
        ],
        install_methods: vec![CliInstallMethod {
            id: None,
            platform: CliPlatform::CrossPlatform,
            installer_kind: CliInstallerKind::Custom,
            package_id: String::new(),
            command_template: entry.install_cmd,
            validation_note: String::new(),
            priority: 100,
        }],
        doc_refs: Vec::new(),
        raw: serde_json::json!({}),
    }
}

const XLSX_COLUMNS: [&str; 22] = [
    "slug",
    "display_name_zh",
    "display_name_en",
    "summary_zh",
    "summary_en",
    "category_code",
    "tags",
    "vendor_name",
    "homepage_url",
    "repo_url",
    "docs_url",
    "requires_text_zh",
    "requires_text_en",
    "entry_point",
    "latest_version",
    "platforms",
    "installer_kind",
    "package_id",
    "install_command_zh",
    "install_command_en",
    "doc_summary_zh",
    "doc_summary_en",
];

#[cfg(not(target_arch = "wasm32"))]
fn parse_xlsx_rows(bytes: &[u8]) -> CliMarketResult<Vec<CliMarketEntryUpsert>> {
    let mut archive = ZipArchive::new(Cursor::new(bytes))
        .map_err(|err| CliMarketError::Message(format!("打开 xlsx 失败：{err}")))?;
    let shared_strings = read_zip_file(&mut archive, "xl/sharedStrings.xml").ok();
    let sheet = read_zip_file(&mut archive, "xl/worksheets/sheet1.xml")
        .map_err(|err| CliMarketError::Message(format!("读取工作表失败：{err}")))?;
    let strings = shared_strings
        .map(|xml| parse_shared_strings(&xml))
        .transpose()?
        .unwrap_or_default();
    let rows = parse_sheet_rows(&sheet, &strings)?;
    if rows.is_empty() {
        return Ok(Vec::new());
    }
    let header = rows[0].clone();
    let mut entries = Vec::new();
    for row in rows.into_iter().skip(1) {
        let map = header
            .iter()
            .cloned()
            .zip(row.into_iter().chain(std::iter::repeat(String::new())))
            .collect::<BTreeMap<_, _>>();
        if map
            .get("slug")
            .map(|value| value.trim().is_empty())
            .unwrap_or(true)
        {
            continue;
        }
        entries.push(CliMarketEntryUpsert {
            id: None,
            slug: map.get("slug").cloned().unwrap_or_default(),
            status: CliMarketStatus::Draft,
            source_type: CliMarketSourceType::ImportExcel,
            entry_kind: CliEntryKind::Cli,
            vendor_name: map.get("vendor_name").cloned().unwrap_or_default(),
            latest_version: map.get("latest_version").cloned().unwrap_or_default(),
            homepage_url: map.get("homepage_url").cloned().unwrap_or_default(),
            repo_url: map.get("repo_url").cloned().unwrap_or_default(),
            docs_url: map.get("docs_url").cloned().unwrap_or_default(),
            entry_point: map.get("entry_point").cloned().unwrap_or_default(),
            category_code: map.get("category_code").cloned().unwrap_or_default(),
            tags: split_csv_like(map.get("tags").cloned().unwrap_or_default().as_str()),
            locales: vec![
                CliLocaleText {
                    locale: CliLocale::ZhCn,
                    display_name: map.get("display_name_zh").cloned().unwrap_or_default(),
                    summary: map.get("summary_zh").cloned().unwrap_or_default(),
                    description_md: String::new(),
                    install_guide_md: String::new(),
                    docs_summary: map.get("doc_summary_zh").cloned().unwrap_or_default(),
                    requires_text: map.get("requires_text_zh").cloned().unwrap_or_default(),
                    install_command: map.get("install_command_zh").cloned().unwrap_or_default(),
                },
                CliLocaleText {
                    locale: CliLocale::EnUs,
                    display_name: map.get("display_name_en").cloned().unwrap_or_default(),
                    summary: map.get("summary_en").cloned().unwrap_or_default(),
                    description_md: String::new(),
                    install_guide_md: String::new(),
                    docs_summary: map.get("doc_summary_en").cloned().unwrap_or_default(),
                    requires_text: map.get("requires_text_en").cloned().unwrap_or_default(),
                    install_command: map.get("install_command_en").cloned().unwrap_or_default(),
                },
            ],
            install_methods: vec![CliInstallMethod {
                id: None,
                platform: parse_first_platform(
                    map.get("platforms").cloned().unwrap_or_default().as_str(),
                )?,
                installer_kind: parse_first_installer_kind(
                    map.get("installer_kind")
                        .cloned()
                        .unwrap_or_default()
                        .as_str(),
                )?,
                package_id: map.get("package_id").cloned().unwrap_or_default(),
                command_template: map.get("install_command_zh").cloned().unwrap_or_default(),
                validation_note: String::new(),
                priority: 100,
            }],
            doc_refs: Vec::new(),
            raw: serde_json::json!({ "source_file": "xlsx" }),
        });
    }
    Ok(entries)
}

#[cfg(not(target_arch = "wasm32"))]
fn parse_first_platform(value: &str) -> CliMarketResult<CliPlatform> {
    let first = split_csv_like(value)
        .into_iter()
        .next()
        .unwrap_or_else(|| "cross_platform".to_string());
    parse_platform(first.trim()).or(Ok(CliPlatform::CrossPlatform))
}

#[cfg(not(target_arch = "wasm32"))]
fn parse_first_installer_kind(value: &str) -> CliMarketResult<CliInstallerKind> {
    let first = split_csv_like(value)
        .into_iter()
        .next()
        .unwrap_or_else(|| "custom".to_string());
    parse_installer_kind(first.trim()).or(Ok(CliInstallerKind::Custom))
}

#[cfg(not(target_arch = "wasm32"))]
fn split_csv_like(value: &str) -> Vec<String> {
    value
        .split([',', ';', '，', '|'])
        .map(|item| item.trim().to_string())
        .filter(|item| !item.is_empty())
        .collect()
}

#[cfg(not(target_arch = "wasm32"))]
fn read_zip_file<R: Read + std::io::Seek>(
    archive: &mut ZipArchive<R>,
    name: &str,
) -> Result<String, zip::result::ZipError> {
    let mut file = archive.by_name(name)?;
    let mut text = String::new();
    file.read_to_string(&mut text)
        .map_err(zip::result::ZipError::Io)?;
    Ok(text)
}

#[cfg(not(target_arch = "wasm32"))]
fn parse_shared_strings(xml: &str) -> CliMarketResult<Vec<String>> {
    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(true);
    let mut values = Vec::new();
    let mut capture = false;
    loop {
        match reader.read_event() {
            Ok(Event::Start(ref event)) if event.name().as_ref() == b"t" => {
                capture = true;
            }
            Ok(Event::Text(text)) if capture => {
                values.push(
                    text.decode()
                        .map_err(|err| {
                            CliMarketError::Message(format!("解析 shared strings 失败：{err}"))
                        })?
                        .into_owned(),
                );
                capture = false;
            }
            Ok(Event::Eof) => break,
            Err(err) => {
                return Err(CliMarketError::Message(format!(
                    "解析 shared strings 失败：{err}"
                )));
            }
            _ => {}
        }
    }
    Ok(values)
}

#[cfg(not(target_arch = "wasm32"))]
fn parse_sheet_rows(xml: &str, shared_strings: &[String]) -> CliMarketResult<Vec<Vec<String>>> {
    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(true);
    let mut rows = Vec::new();
    let mut current_row: Vec<String> = Vec::new();
    let mut current_cell_type = String::new();
    let mut in_value = false;
    loop {
        match reader.read_event() {
            Ok(Event::Start(ref event)) if event.name().as_ref() == b"row" => {
                current_row.clear();
            }
            Ok(Event::Start(ref event)) if event.name().as_ref() == b"c" => {
                current_cell_type.clear();
                for attr in event.attributes().flatten() {
                    if attr.key.as_ref() == b"t" {
                        current_cell_type =
                            String::from_utf8_lossy(attr.value.as_ref()).to_string();
                    }
                }
            }
            Ok(Event::Start(ref event))
                if event.name().as_ref() == b"v" || event.name().as_ref() == b"t" =>
            {
                in_value = true;
            }
            Ok(Event::Text(text)) if in_value => {
                let raw = text
                    .decode()
                    .map_err(|err| CliMarketError::Message(format!("解析单元格失败：{err}")))?
                    .into_owned();
                let value = if current_cell_type == "s" {
                    raw.parse::<usize>()
                        .ok()
                        .and_then(|index| shared_strings.get(index).cloned())
                        .unwrap_or_default()
                } else {
                    raw
                };
                current_row.push(value);
                in_value = false;
            }
            Ok(Event::End(ref event)) if event.name().as_ref() == b"row" => {
                rows.push(std::mem::take(&mut current_row));
            }
            Ok(Event::Eof) => break,
            Err(err) => return Err(CliMarketError::Message(format!("解析工作表失败：{err}"))),
            _ => {}
        }
    }
    Ok(rows)
}

#[cfg(not(target_arch = "wasm32"))]
fn write_xlsx(entries: &[CliMarketEntry]) -> CliMarketResult<Vec<u8>> {
    let mut rows = Vec::with_capacity(entries.len() + 1);
    rows.push(
        XLSX_COLUMNS
            .iter()
            .map(|value| value.to_string())
            .collect::<Vec<_>>(),
    );
    for entry in entries {
        let zh = entry
            .locales
            .iter()
            .find(|locale| locale.locale == CliLocale::ZhCn)
            .cloned()
            .unwrap_or_default();
        let en = entry
            .locales
            .iter()
            .find(|locale| locale.locale == CliLocale::EnUs)
            .cloned()
            .unwrap_or_default();
        let first_method = entry.install_methods.first().cloned().unwrap_or_default();
        rows.push(vec![
            entry.slug.clone(),
            zh.display_name,
            en.display_name,
            zh.summary,
            en.summary,
            entry.category_code.clone(),
            entry.tags.join(","),
            entry.vendor_name.clone(),
            entry.homepage_url.clone(),
            entry.repo_url.clone(),
            entry.docs_url.clone(),
            zh.requires_text,
            en.requires_text,
            entry.entry_point.clone(),
            entry.latest_version.clone(),
            first_method.platform.code().to_string(),
            first_method.installer_kind.code().to_string(),
            first_method.package_id,
            zh.install_command,
            en.install_command,
            zh.docs_summary,
            en.docs_summary,
        ]);
    }

    let (shared_string_index, shared_string_values) = build_shared_strings(&rows);
    let worksheet = build_sheet_xml(&rows, &shared_string_index);
    let shared_xml = build_shared_strings_xml(shared_string_values);

    let mut cursor = Cursor::new(Vec::new());
    let mut zip = ZipWriter::new(&mut cursor);
    let options = SimpleFileOptions::default();

    zip.start_file("[Content_Types].xml", options)
        .map_err(|err| CliMarketError::Message(format!("写入 xlsx 失败：{err}")))?;
    zip.write_all(CONTENT_TYPES_XML.as_bytes())
        .map_err(|err| CliMarketError::Message(format!("写入 xlsx 失败：{err}")))?;
    zip.start_file("_rels/.rels", options)
        .map_err(|err| CliMarketError::Message(format!("写入 xlsx 失败：{err}")))?;
    zip.write_all(ROOT_RELS_XML.as_bytes())
        .map_err(|err| CliMarketError::Message(format!("写入 xlsx 失败：{err}")))?;
    zip.start_file("xl/workbook.xml", options)
        .map_err(|err| CliMarketError::Message(format!("写入 xlsx 失败：{err}")))?;
    zip.write_all(WORKBOOK_XML.as_bytes())
        .map_err(|err| CliMarketError::Message(format!("写入 xlsx 失败：{err}")))?;
    zip.start_file("xl/_rels/workbook.xml.rels", options)
        .map_err(|err| CliMarketError::Message(format!("写入 xlsx 失败：{err}")))?;
    zip.write_all(WORKBOOK_RELS_XML.as_bytes())
        .map_err(|err| CliMarketError::Message(format!("写入 xlsx 失败：{err}")))?;
    zip.start_file("xl/worksheets/sheet1.xml", options)
        .map_err(|err| CliMarketError::Message(format!("写入 xlsx 失败：{err}")))?;
    zip.write_all(worksheet.as_bytes())
        .map_err(|err| CliMarketError::Message(format!("写入 xlsx 失败：{err}")))?;
    zip.start_file("xl/sharedStrings.xml", options)
        .map_err(|err| CliMarketError::Message(format!("写入 xlsx 失败：{err}")))?;
    zip.write_all(shared_xml.as_bytes())
        .map_err(|err| CliMarketError::Message(format!("写入 xlsx 失败：{err}")))?;
    zip.finish()
        .map_err(|err| CliMarketError::Message(format!("完成 xlsx 失败：{err}")))?;
    Ok(cursor.into_inner())
}

#[cfg(not(target_arch = "wasm32"))]
fn build_shared_strings(rows: &[Vec<String>]) -> (BTreeMap<String, usize>, Vec<String>) {
    let mut map = BTreeMap::new();
    let mut values = Vec::new();
    for row in rows {
        for value in row {
            if !map.contains_key(value) {
                let index = map.len();
                map.insert(value.clone(), index);
                values.push(value.clone());
            }
        }
    }
    (map, values)
}

#[cfg(not(target_arch = "wasm32"))]
fn build_shared_strings_xml(values: Vec<String>) -> String {
    let mut xml = format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?><sst xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main" count="{0}" uniqueCount="{0}">"#,
        values.len()
    );
    for value in values {
        xml.push_str("<si><t>");
        xml.push_str(&escape_xml(&value));
        xml.push_str("</t></si>");
    }
    xml.push_str("</sst>");
    xml
}

#[cfg(not(target_arch = "wasm32"))]
fn build_sheet_xml(rows: &[Vec<String>], shared_strings: &BTreeMap<String, usize>) -> String {
    let mut xml = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?><worksheet xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main"><sheetData>"#.to_string();
    for (row_index, row) in rows.iter().enumerate() {
        xml.push_str(&format!(r#"<row r="{}">"#, row_index + 1));
        for (col_index, value) in row.iter().enumerate() {
            let cell_ref = format!("{}{}", excel_col(col_index), row_index + 1);
            let shared_index = shared_strings.get(value).copied().unwrap_or_default();
            xml.push_str(&format!(
                r#"<c r="{cell_ref}" t="s"><v>{shared_index}</v></c>"#
            ));
        }
        xml.push_str("</row>");
    }
    xml.push_str("</sheetData></worksheet>");
    xml
}

#[cfg(not(target_arch = "wasm32"))]
fn excel_col(index: usize) -> String {
    let mut n = index + 1;
    let mut label = String::new();
    while n > 0 {
        let rem = (n - 1) % 26;
        label.insert(0, char::from(b'A' + u8::try_from(rem).unwrap_or_default()));
        n = (n - 1) / 26;
    }
    label
}

#[cfg(not(target_arch = "wasm32"))]
fn escape_xml(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

const CONTENT_TYPES_XML: &str = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types">
  <Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/>
  <Default Extension="xml" ContentType="application/xml"/>
  <Override PartName="/xl/workbook.xml" ContentType="application/vnd.openxmlformats-officedocument.spreadsheetml.sheet.main+xml"/>
  <Override PartName="/xl/worksheets/sheet1.xml" ContentType="application/vnd.openxmlformats-officedocument.spreadsheetml.worksheet+xml"/>
  <Override PartName="/xl/sharedStrings.xml" ContentType="application/vnd.openxmlformats-officedocument.spreadsheetml.sharedStrings+xml"/>
</Types>"#;

const ROOT_RELS_XML: &str = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
  <Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="xl/workbook.xml"/>
</Relationships>"#;

const WORKBOOK_XML: &str = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<workbook xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main" xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships">
  <sheets>
    <sheet name="CLI Market" sheetId="1" r:id="rId1"/>
  </sheets>
</workbook>"#;

const WORKBOOK_RELS_XML: &str = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
  <Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/worksheet" Target="worksheets/sheet1.xml"/>
  <Relationship Id="rId2" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/sharedStrings" Target="sharedStrings.xml"/>
</Relationships>"#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn registry_compat_json_can_be_converted() {
        let bytes = serde_json::to_vec(&vec![CliRegistryCompatEntry {
            name: "gimp".to_string(),
            display_name: "GIMP".to_string(),
            version: "1.0.0".to_string(),
            description: "Raster tool".to_string(),
            requires: "brew".to_string(),
            install_cmd: "brew install gimp".to_string(),
            entry_point: "gimp".to_string(),
            category: "image".to_string(),
        }])
        .expect("serialize compat json");

        let rows = parse_json_rows(&bytes, CliImportMode::RegistryCompat).expect("parse compat");
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].slug, "gimp");
        assert_eq!(rows[0].locales.len(), 2);
        assert_eq!(
            rows[0].install_methods[0].command_template,
            "brew install gimp"
        );
    }

    #[test]
    fn xlsx_round_trip_preserves_slug() {
        let entry = CliMarketEntry {
            id: Uuid::new_v4().to_string(),
            slug: "uv".to_string(),
            status: CliMarketStatus::Draft,
            source_type: CliMarketSourceType::Manual,
            entry_kind: CliEntryKind::Cli,
            vendor_name: "Astral".to_string(),
            latest_version: "0.7.0".to_string(),
            homepage_url: String::new(),
            repo_url: String::new(),
            docs_url: String::new(),
            entry_point: "uv".to_string(),
            category_code: "python".to_string(),
            tags: vec!["python".to_string()],
            locales: vec![
                CliLocaleText {
                    locale: CliLocale::ZhCn,
                    display_name: "uv".to_string(),
                    summary: "Python 包管理".to_string(),
                    description_md: String::new(),
                    install_guide_md: String::new(),
                    docs_summary: String::new(),
                    requires_text: String::new(),
                    install_command: "brew install uv".to_string(),
                },
                CliLocaleText {
                    locale: CliLocale::EnUs,
                    display_name: "uv".to_string(),
                    summary: "Python package manager".to_string(),
                    description_md: String::new(),
                    install_guide_md: String::new(),
                    docs_summary: String::new(),
                    requires_text: String::new(),
                    install_command: "brew install uv".to_string(),
                },
            ],
            install_methods: vec![CliInstallMethod {
                id: None,
                platform: CliPlatform::Macos,
                installer_kind: CliInstallerKind::Brew,
                package_id: "uv".to_string(),
                command_template: "brew install uv".to_string(),
                validation_note: String::new(),
                priority: 100,
            }],
            doc_refs: Vec::new(),
            raw: serde_json::json!({}),
            created_at: None,
            updated_at: None,
        };

        let bytes = write_xlsx(&[entry]).expect("write xlsx");
        let rows = parse_xlsx_rows(&bytes).expect("parse xlsx");
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].slug, "uv");
        assert_eq!(rows[0].category_code, "python");
    }

    #[test]
    fn select_install_method_prefers_host_platform() {
        let entry = CliMarketEntry {
            id: Uuid::new_v4().to_string(),
            slug: "demo".to_string(),
            status: CliMarketStatus::Draft,
            source_type: CliMarketSourceType::Manual,
            entry_kind: CliEntryKind::Cli,
            vendor_name: String::new(),
            latest_version: String::new(),
            homepage_url: String::new(),
            repo_url: String::new(),
            docs_url: String::new(),
            entry_point: String::new(),
            category_code: String::new(),
            tags: Vec::new(),
            locales: Vec::new(),
            install_methods: vec![
                CliInstallMethod {
                    id: Some("cross".to_string()),
                    platform: CliPlatform::CrossPlatform,
                    installer_kind: CliInstallerKind::Custom,
                    package_id: String::new(),
                    command_template: "generic".to_string(),
                    validation_note: String::new(),
                    priority: 1,
                },
                CliInstallMethod {
                    id: Some("mac".to_string()),
                    platform: CliPlatform::Macos,
                    installer_kind: CliInstallerKind::Brew,
                    package_id: String::new(),
                    command_template: "brew install demo".to_string(),
                    validation_note: String::new(),
                    priority: 100,
                },
            ],
            doc_refs: Vec::new(),
            raw: serde_json::json!({}),
            created_at: None,
            updated_at: None,
        };

        let selected = select_install_method(&entry, None, CliPlatform::Macos)
            .expect("select host platform method");
        assert_eq!(selected.id.as_deref(), Some("mac"));
    }

    #[test]
    fn validate_install_method_accepts_single_brew_command() {
        let method = CliInstallMethod {
            id: None,
            platform: CliPlatform::Macos,
            installer_kind: CliInstallerKind::Brew,
            package_id: "uv".to_string(),
            command_template: "brew install uv".to_string(),
            validation_note: String::new(),
            priority: 100,
        };

        assert!(validate_install_method(&method).is_ok());
    }

    #[test]
    fn validate_install_method_rejects_shell_chain() {
        let method = CliInstallMethod {
            id: None,
            platform: CliPlatform::Macos,
            installer_kind: CliInstallerKind::Brew,
            package_id: "uv".to_string(),
            command_template: "brew install uv && echo ok".to_string(),
            validation_note: String::new(),
            priority: 100,
        };

        assert!(validate_install_method(&method).is_err());
    }

    #[test]
    fn validate_install_method_rejects_custom_installer() {
        let method = CliInstallMethod {
            id: None,
            platform: CliPlatform::CrossPlatform,
            installer_kind: CliInstallerKind::Custom,
            package_id: "demo".to_string(),
            command_template: "custom install demo".to_string(),
            validation_note: String::new(),
            priority: 100,
        };

        assert!(validate_install_method(&method).is_err());
    }
}
