//! 系统后台持久化服务。
//!
//! 该服务把页面契约快照和操作执行记录写入 PostgreSQL，保证后台管理面
//! 不是纯静态展示。具体业务表由对应系统模块继续按 `sys_*` 表边界承载。

use anyhow::{Context, anyhow, bail};
use sha2::{Digest, Sha256};
use serde_json::Value;
use toasty::stmt::{List, Query};
use uuid::Uuid;

use crate::{
    core::db,
    system::{
        catalog::{SystemOperation, SystemPage, system_pages},
        model::{
            CreatedSystemApiKey, SystemApiKeyRecord, SystemApiKeySummary, SystemDataRecord,
            SystemOperationRecord, SystemOperationRecordSummary, SystemPageDataPagination,
            SystemPageDataResponse, SystemPageRecord, SystemPageRecordSummary, SystemStoreStatus,
            TABLE_NAME_PREFIX,
        },
    },
};

#[derive(Clone)]
pub struct SystemAdminStore {
    pub(crate) db: db::Db,
}

impl SystemAdminStore {
    pub fn from_shared(db: db::Db) -> Self {
        Self { db }
    }

    pub async fn sync_catalog_snapshot(&self) -> anyhow::Result<Vec<SystemPageRecordSummary>> {
        let mut summaries = Vec::new();

        for page in system_pages().iter().copied() {
            summaries.push(self.upsert_page_snapshot(page).await?);
            self.sync_page_data_records(page).await?;
        }

        Ok(summaries)
    }

    pub async fn list_page_snapshots(&self) -> anyhow::Result<Vec<SystemPageRecordSummary>> {
        let mut db = self.db.lock().await;
        let records = Query::<List<SystemPageRecord>>::all()
            .exec(&mut *db)
            .await
            .context("读取系统后台页面快照失败")?;
        Ok(records.into_iter().map(Into::into).collect())
    }

    pub async fn list_operation_records(
        &self,
        limit: usize,
    ) -> anyhow::Result<Vec<SystemOperationRecordSummary>> {
        let limit = limit.clamp(1, 100);
        let mut db = self.db.lock().await;
        let records = Query::<List<SystemOperationRecord>>::all()
            .exec(&mut *db)
            .await
            .context("读取系统后台操作记录失败")?;
        Ok(records.into_iter().rev().take(limit).map(Into::into).collect())
    }

    pub async fn list_page_data_records(
        &self,
        page_id: Option<&str>,
        offset: usize,
        size: usize,
    ) -> anyhow::Result<SystemPageDataResponse> {
        let size = size.clamp(1, 100);
        let mut db = self.db.lock().await;
        let records = Query::<List<SystemDataRecord>>::all()
            .exec(&mut *db)
            .await
            .context("读取系统后台数据快照失败")?;
        let filtered = records
            .into_iter()
            .filter(|record| page_id.is_none_or(|id| record.page_id == id))
            .map(Into::into)
            .collect::<Vec<_>>();
        let total = filtered.len();
        let data = filtered.into_iter().skip(offset).take(size).collect();

        Ok(SystemPageDataResponse {
            d: data,
            t: total,
            p: SystemPageDataPagination { o: offset, s: size },
        })
    }

    pub async fn list_api_keys(&self) -> anyhow::Result<Vec<SystemApiKeySummary>> {
        let mut db = self.db.lock().await;
        let records = Query::<List<SystemApiKeyRecord>>::all()
            .exec(&mut *db)
            .await
            .context("读取系统 API 密钥失败")?;
        Ok(records.into_iter().map(Into::into).collect())
    }

    pub async fn create_api_key(
        &self,
        input: CreateSystemApiKeyInput,
    ) -> anyhow::Result<CreatedSystemApiKey> {
        let name = normalized_api_key_name(&input.name)?;
        let scope = input
            .scope
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty())
            .unwrap_or_else(|| "all-services".to_string());
        if scope != "all-services" {
            bail!("api_key scope must be all-services");
        }

        let api_key = generate_api_key();
        let now = db::timestamp_secs();
        let mut db = self.db.lock().await;
        let record = SystemApiKeyRecord::create()
            .id(db::new_uuid_id())
            .key_hash(system_api_key_hash(&api_key))
            .name(name)
            .prefix(api_key_prefix(&api_key))
            .scope(scope)
            .status("active".to_string())
            .created_at(now)
            .last_used_at(String::new())
            .exec(&mut *db)
            .await
            .context("创建系统 API 密钥失败")?;

        Ok(CreatedSystemApiKey {
            api_key,
            summary: record.into(),
        })
    }

    pub async fn revoke_api_key(&self, id: &str) -> anyhow::Result<SystemApiKeySummary> {
        let id = id.trim();
        if id.is_empty() {
            bail!("api_key id is required");
        }

        let mut db = self.db.lock().await;
        let existing = Query::<List<SystemApiKeyRecord>>::filter(
            SystemApiKeyRecord::fields().id().eq(id),
        )
        .first()
        .exec(&mut *db)
        .await
        .context("读取系统 API 密钥失败")?;
        if existing.is_none() {
            bail!("not found api_key: {id}");
        }

        SystemApiKeyRecord::filter(SystemApiKeyRecord::fields().id().eq(id))
            .update()
            .status("revoked".to_string())
            .exec(&mut *db)
            .await
            .context("撤销系统 API 密钥失败")?;
        let record =
            Query::<List<SystemApiKeyRecord>>::filter(SystemApiKeyRecord::fields().id().eq(id))
                .one()
                .exec(&mut *db)
                .await
                .context("读取已撤销系统 API 密钥失败")?;

        Ok(record.into())
    }

    pub async fn authorize_api_key(
        &self,
        cleartext_api_key: &str,
    ) -> anyhow::Result<Option<SystemApiKeySummary>> {
        if cleartext_api_key.trim().is_empty() {
            return Ok(None);
        }

        let hash = system_api_key_hash(cleartext_api_key);
        let mut db = self.db.lock().await;
        let Some(record) = Query::<List<SystemApiKeyRecord>>::filter(
            SystemApiKeyRecord::fields().key_hash().eq(&hash),
        )
        .first()
        .exec(&mut *db)
        .await
        .context("读取系统 API 密钥失败")?
        else {
            return Ok(None);
        };
        let mut summary = SystemApiKeySummary::from(record);
        if summary.status != "active" {
            return Ok(None);
        }

        let now = db::timestamp_secs();
        SystemApiKeyRecord::filter(SystemApiKeyRecord::fields().id().eq(&summary.id))
            .update()
            .last_used_at(now.clone())
            .exec(&mut *db)
            .await
            .context("更新系统 API 密钥最近使用时间失败")?;
        summary.last_used_at = now;
        Ok(Some(summary))
    }

    pub async fn execute_operation(
        &self,
        input: SystemOperationInput,
    ) -> anyhow::Result<SystemOperationRecordSummary> {
        let page = system_pages()
            .iter()
            .copied()
            .find(|page| page.id == input.page_id)
            .ok_or_else(|| anyhow!("system page not found: {}", input.page_id))?;
        let operation = page
            .operations
            .iter()
            .copied()
            .find(|operation| operation.id == input.operation_id)
            .ok_or_else(|| anyhow!("system operation not found: {}", input.operation_id))?;
        validate_operation_input(page, operation, &input)?;

        let payload_json = serde_json::to_string(&input.payload).context("序列化操作载荷失败")?;
        let now = db::timestamp_secs();
        let mut db = self.db.lock().await;
        let record = SystemOperationRecord::create()
            .id(db::new_uuid_id())
            .operation_id(operation.id.to_string())
            .page_id(page.id.to_string())
            .method(operation.method.to_string())
            .api_path(operation.path.to_string())
            .cli(operation.cli.to_string())
            .payload_json(payload_json)
            .status("accepted".to_string())
            .created_at(now)
            .exec(&mut *db)
            .await
            .context("写入系统后台操作记录失败")?;
        Ok(record.into())
    }

    async fn sync_page_data_records(&self, page: SystemPage) -> anyhow::Result<()> {
        for (index, row) in page.rows.iter().enumerate() {
            self.upsert_page_data_record(page, index, row.cells).await?;
        }

        Ok(())
    }

    async fn upsert_page_data_record(
        &self,
        page: SystemPage,
        index: usize,
        cells: &[crate::system::catalog::SystemTableCell],
    ) -> anyhow::Result<()> {
        let id = format!("{}-{index}", page.id);
        let now = db::timestamp_secs();
        let cells_json =
            serde_json::to_string(cells).context("序列化系统后台数据快照失败")?;
        let mut db = self.db.lock().await;
        let existing =
            Query::<List<SystemDataRecord>>::filter(SystemDataRecord::fields().id().eq(&id))
                .first()
                .exec(&mut *db)
                .await
                .context("读取系统后台数据快照失败")?;

        match existing {
            Some(_) => {
                SystemDataRecord::filter(SystemDataRecord::fields().id().eq(&id))
                    .update()
                    .page_id(page.id.to_string())
                    .row_key(index.to_string())
                    .cells_json(cells_json)
                    .updated_at(now)
                    .exec(&mut *db)
                    .await
                    .context("更新系统后台数据快照失败")?;
            }
            None => {
                SystemDataRecord::create()
                    .id(id)
                    .page_id(page.id.to_string())
                    .row_key(index.to_string())
                    .cells_json(cells_json)
                    .updated_at(now)
                    .exec(&mut *db)
                    .await
                    .context("创建系统后台数据快照失败")?;
            }
        }

        Ok(())
    }

    async fn upsert_page_snapshot(
        &self,
        page: SystemPage,
    ) -> anyhow::Result<SystemPageRecordSummary> {
        let now = db::timestamp_secs();
        let operations = page
            .operations
            .iter()
            .map(|operation| operation.id)
            .collect::<Vec<_>>()
            .join("\n");
        let pg_tables = page.pg_tables.join("\n");
        let status = format!("{:?}", page.status);
        let mut db = self.db.lock().await;
        let existing = Query::<List<SystemPageRecord>>::filter(
            SystemPageRecord::fields().id().eq(page.id),
        )
        .first()
        .exec(&mut *db)
        .await
        .context("读取系统后台页面快照失败")?;
        let record = match existing {
            Some(_) => {
                SystemPageRecord::filter(SystemPageRecord::fields().id().eq(page.id))
                    .update()
                    .route(page.route.to_string())
                    .label(page.label.to_string())
                    .status(status)
                    .pg_tables(pg_tables)
                    .operations(operations)
                    .updated_at(now)
                    .exec(&mut *db)
                    .await
                    .context("更新系统后台页面快照失败")?;
                Query::<List<SystemPageRecord>>::filter(
                    SystemPageRecord::fields().id().eq(page.id),
                )
                .one()
                .exec(&mut *db)
                .await
                .context("读取已更新系统后台页面快照失败")?
            }
            None => SystemPageRecord::create()
                .id(page.id.to_string())
                .route(page.route.to_string())
                .label(page.label.to_string())
                .status(status)
                .pg_tables(pg_tables)
                .operations(operations)
                .updated_at(now)
                .exec(&mut *db)
                .await
                .context("创建系统后台页面快照失败")?,
        };

        Ok(record.into())
    }
}

#[derive(Clone, Debug)]
pub struct SystemOperationInput {
    pub page_id: String,
    pub operation_id: String,
    pub method: String,
    pub path: String,
    pub payload: Value,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CreateSystemApiKeyInput {
    pub name: String,
    pub scope: Option<String>,
}

pub fn system_store_status(database_url: &Option<String>, store: &Option<SystemAdminStore>) -> SystemStoreStatus {
    SystemStoreStatus {
        database_configured: database_url.as_ref().is_some_and(|value| !value.trim().is_empty()),
        store_connected: store.is_some(),
        table_prefix: TABLE_NAME_PREFIX.to_string(),
    }
}

fn validate_operation_input(
    page: SystemPage,
    operation: SystemOperation,
    input: &SystemOperationInput,
) -> anyhow::Result<()> {
    if input.method != operation.method {
        bail!("system operation method mismatch: {}", input.operation_id);
    }
    if input.path != operation.path {
        bail!("system operation path mismatch: {}", input.operation_id);
    }
    if page.pg_tables.is_empty() {
        bail!("system page has no pg boundary: {}", page.id);
    }
    Ok(())
}

pub fn system_admin_models() -> toasty::ModelSet {
    toasty::models!(
        SystemPageRecord,
        SystemOperationRecord,
        SystemDataRecord,
        SystemApiKeyRecord
    )
}

pub const SYSTEM_ADMIN_BOOTSTRAP_SQL: &[&str] = &[
    "CREATE TABLE IF NOT EXISTS biz_system_admin_system_page_records (id TEXT PRIMARY KEY, route TEXT NOT NULL, label TEXT NOT NULL, status TEXT NOT NULL, pg_tables TEXT NOT NULL, operations TEXT NOT NULL, updated_at TEXT NOT NULL)",
    "CREATE INDEX IF NOT EXISTS biz_system_admin_system_page_records_route_idx ON biz_system_admin_system_page_records (route)",
    "CREATE TABLE IF NOT EXISTS biz_system_admin_system_operation_records (id TEXT PRIMARY KEY, operation_id TEXT NOT NULL, page_id TEXT NOT NULL, method TEXT NOT NULL, api_path TEXT NOT NULL, cli TEXT NOT NULL, payload_json TEXT NOT NULL, status TEXT NOT NULL, created_at TEXT NOT NULL)",
    "CREATE INDEX IF NOT EXISTS biz_system_admin_system_operation_records_operation_id_idx ON biz_system_admin_system_operation_records (operation_id)",
    "CREATE TABLE IF NOT EXISTS biz_system_admin_system_data_records (id TEXT PRIMARY KEY, page_id TEXT NOT NULL, row_key TEXT NOT NULL, cells_json TEXT NOT NULL, updated_at TEXT NOT NULL)",
    "CREATE INDEX IF NOT EXISTS biz_system_admin_system_data_records_page_id_idx ON biz_system_admin_system_data_records (page_id)",
    "CREATE TABLE IF NOT EXISTS biz_system_admin_system_api_key_records (id TEXT PRIMARY KEY, key_hash TEXT NOT NULL, name TEXT NOT NULL, prefix TEXT NOT NULL, scope TEXT NOT NULL, status TEXT NOT NULL, created_at TEXT NOT NULL, last_used_at TEXT NOT NULL)",
    "CREATE INDEX IF NOT EXISTS biz_system_admin_system_api_key_records_key_hash_idx ON biz_system_admin_system_api_key_records (key_hash)",
];

fn normalized_api_key_name(name: &str) -> anyhow::Result<String> {
    let name = name.trim();
    if name.is_empty() {
        bail!("api_key name is required");
    }
    if name.chars().count() > 80 {
        bail!("api_key name must not be longer than 80 characters");
    }
    Ok(name.to_string())
}

fn generate_api_key() -> String {
    format!(
        "az_live_{}{}",
        Uuid::new_v4().simple(),
        Uuid::new_v4().simple()
    )
}

fn api_key_prefix(api_key: &str) -> String {
    let head = api_key.chars().take(16).collect::<String>();
    format!("{head}…")
}

pub fn system_api_key_hash(cleartext_api_key: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(cleartext_api_key.trim().as_bytes());
    format!("{:x}", hasher.finalize())
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn status_reports_degraded_without_store() {
        let status = system_store_status(&None, &None);

        assert!(!status.database_configured);
        assert!(!status.store_connected);
        assert_eq!(status.table_prefix, TABLE_NAME_PREFIX);
    }

    #[test]
    fn operation_input_rejects_wrong_path() {
        let page = system_pages()
            .iter()
            .copied()
            .find(|page| !page.operations.is_empty())
            .unwrap();
        let operation = page.operations[0];
        let input = SystemOperationInput {
            page_id: page.id.to_string(),
            operation_id: operation.id.to_string(),
            method: operation.method.to_string(),
            path: "/api/system/wrong".to_string(),
            payload: json!({}),
        };

        let error = validate_operation_input(page, operation, &input).unwrap_err();

        // 关键断言：统一执行器必须校验路径，避免 API/CLI contract 漂移。
        assert!(error.to_string().contains("path mismatch"));
    }
}
