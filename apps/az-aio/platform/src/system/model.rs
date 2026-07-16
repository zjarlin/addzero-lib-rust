//! 系统后台 PostgreSQL 模型。
//!
//! 这里保存的是系统后台自身的管理契约快照和操作审计，不替代各业务表。
//! 业务表仍按 catalog 声明的 `sys_*` 边界在正式模块中落库。

use serde::{Deserialize, Serialize};

use crate::core::db::ToastyModelContribution;

pub const TABLE_NAME_PREFIX: &str = "biz_system_admin_";

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, toasty::Model)]
#[table = "biz_system_admin_system_page_records"]
pub struct SystemPageRecord {
    #[key]
    pub id: String,
    #[index]
    pub route: String,
    pub label: String,
    pub status: String,
    pub pg_tables: String,
    pub operations: String,
    pub updated_at: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, toasty::Model)]
#[table = "biz_system_admin_system_operation_records"]
pub struct SystemOperationRecord {
    #[key]
    pub id: String,
    #[index]
    pub operation_id: String,
    pub page_id: String,
    pub method: String,
    pub api_path: String,
    pub cli: String,
    pub payload_json: String,
    pub status: String,
    pub created_at: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, toasty::Model)]
#[table = "biz_system_admin_system_data_records"]
pub struct SystemDataRecord {
    #[key]
    pub id: String,
    #[index]
    pub page_id: String,
    pub row_key: String,
    pub cells_json: String,
    pub updated_at: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, toasty::Model)]
#[table = "biz_system_admin_system_api_key_records"]
pub struct SystemApiKeyRecord {
    #[key]
    pub id: String,
    #[index]
    pub key_hash: String,
    pub name: String,
    pub prefix: String,
    pub scope: String,
    pub status: String,
    pub created_at: String,
    pub last_used_at: String,
}

#[rudi::Singleton(name = "system-admin-toasty-models")]
pub fn system_admin_model_contribution() -> ToastyModelContribution {
    ToastyModelContribution::new(toasty::models!(
        SystemPageRecord,
        SystemOperationRecord,
        SystemDataRecord,
        SystemApiKeyRecord
    ))
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct SystemStoreStatus {
    pub database_configured: bool,
    pub store_connected: bool,
    pub table_prefix: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct SystemPageRecordSummary {
    pub id: String,
    pub route: String,
    pub label: String,
    pub status: String,
    pub pg_tables: Vec<String>,
    pub operations: Vec<String>,
    pub updated_at: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct SystemOperationRecordSummary {
    pub id: String,
    pub operation_id: String,
    pub page_id: String,
    pub method: String,
    pub path: String,
    pub cli: String,
    pub payload_json: String,
    pub status: String,
    pub created_at: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct SystemDataRecordSummary {
    pub id: String,
    pub page_id: String,
    pub row_key: String,
    pub cells_json: String,
    pub updated_at: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SystemApiKeySummary {
    pub id: String,
    pub name: String,
    pub prefix: String,
    pub scope: String,
    pub status: String,
    pub created_at: String,
    pub last_used_at: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreatedSystemApiKey {
    pub api_key: String,
    pub summary: SystemApiKeySummary,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct SystemPageDataResponse {
    pub d: Vec<SystemDataRecordSummary>,
    pub t: usize,
    pub p: SystemPageDataPagination,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct SystemPageDataPagination {
    pub o: usize,
    pub s: usize,
}

impl From<SystemPageRecord> for SystemPageRecordSummary {
    fn from(record: SystemPageRecord) -> Self {
        Self {
            id: record.id,
            route: record.route,
            label: record.label,
            status: record.status,
            pg_tables: split_lines(&record.pg_tables),
            operations: split_lines(&record.operations),
            updated_at: record.updated_at,
        }
    }
}

impl From<SystemOperationRecord> for SystemOperationRecordSummary {
    fn from(record: SystemOperationRecord) -> Self {
        Self {
            id: record.id,
            operation_id: record.operation_id,
            page_id: record.page_id,
            method: record.method,
            path: record.api_path,
            cli: record.cli,
            payload_json: record.payload_json,
            status: record.status,
            created_at: record.created_at,
        }
    }
}

impl From<SystemDataRecord> for SystemDataRecordSummary {
    fn from(record: SystemDataRecord) -> Self {
        Self {
            id: record.id,
            page_id: record.page_id,
            row_key: record.row_key,
            cells_json: record.cells_json,
            updated_at: record.updated_at,
        }
    }
}

impl From<SystemApiKeyRecord> for SystemApiKeySummary {
    fn from(record: SystemApiKeyRecord) -> Self {
        Self {
            id: record.id,
            name: record.name,
            prefix: record.prefix,
            scope: record.scope,
            status: record.status,
            created_at: record.created_at,
            last_used_at: record.last_used_at,
        }
    }
}

fn split_lines(value: &str) -> Vec<String> {
    value
        .lines()
        .map(str::trim)
        .filter(|item| !item.is_empty())
        .map(ToOwned::to_owned)
        .collect()
}
