//! 系统字典的 PostgreSQL 模型与 API 契约。

use serde::{Deserialize, Serialize};

use crate::core::db::ToastyModelContribution;

/// 字典类型 PostgreSQL 记录。
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, toasty::Model)]
#[table = "sys_dict_type"]
pub struct DictionaryTypeRecord {
    #[key]
    pub id: String,
    #[index]
    pub code: String,
    pub name: String,
    pub description: String,
    pub scope: String,
    pub raw_value_kind: String,
    pub open_enum: bool,
    pub sort_index: i64,
    pub status: String,
    pub created_at_ms: i64,
    pub updated_at_ms: i64,
}

/// 字典项 PostgreSQL 记录。
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, toasty::Model)]
#[table = "sys_dict_data"]
pub struct DictionaryItemRecord {
    #[key]
    pub id: String,
    #[index]
    pub dictionary_type_id: String,
    pub code: String,
    pub label: String,
    pub description: String,
    pub raw_value: String,
    pub sort_index: i64,
    pub status: String,
    pub meta_json: String,
    pub created_at_ms: i64,
    pub updated_at_ms: i64,
}

/// 注册系统字典 Toasty 模型。
#[rudi::Singleton(name = "system-dictionary-toasty-models")]
pub fn dictionary_model_contribution() -> ToastyModelContribution {
    ToastyModelContribution::new(toasty::models!(
        DictionaryTypeRecord,
        DictionaryItemRecord
    ))
}

/// 字典类型列表项。
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DictionaryTypeSummary {
    pub id: String,
    pub code: String,
    pub name: String,
    pub description: String,
    pub scope: String,
    pub raw_value_kind: String,
    pub open_enum: bool,
    pub sort_index: i64,
    pub status: String,
    pub item_count: usize,
    pub created_at_ms: i64,
    pub updated_at_ms: i64,
}

impl DictionaryTypeRecord {
    /// 转换为包含条目数量的管理端投影。
    pub fn summary(self, item_count: usize) -> DictionaryTypeSummary {
        DictionaryTypeSummary {
            id: self.id,
            code: self.code,
            name: self.name,
            description: self.description,
            scope: self.scope,
            raw_value_kind: self.raw_value_kind,
            open_enum: self.open_enum,
            sort_index: self.sort_index,
            status: self.status,
            item_count,
            created_at_ms: self.created_at_ms,
            updated_at_ms: self.updated_at_ms,
        }
    }
}

/// 字典项列表投影。
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DictionaryItemSummary {
    pub id: String,
    pub dictionary_type_id: String,
    pub code: String,
    pub label: String,
    pub description: String,
    pub raw_value: String,
    pub sort_index: i64,
    pub status: String,
    pub meta_json: String,
    pub created_at_ms: i64,
    pub updated_at_ms: i64,
}

impl From<DictionaryItemRecord> for DictionaryItemSummary {
    fn from(record: DictionaryItemRecord) -> Self {
        Self {
            id: record.id,
            dictionary_type_id: record.dictionary_type_id,
            code: record.code,
            label: record.label,
            description: record.description,
            raw_value: record.raw_value,
            sort_index: record.sort_index,
            status: record.status,
            meta_json: record.meta_json,
            created_at_ms: record.created_at_ms,
            updated_at_ms: record.updated_at_ms,
        }
    }
}

/// 新建或更新字典类型的请求。
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DictionaryTypeInput {
    pub code: String,
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub scope: String,
    #[serde(default)]
    pub raw_value_kind: String,
    #[serde(default)]
    pub open_enum: bool,
    #[serde(default)]
    pub sort_index: i64,
    #[serde(default)]
    pub status: String,
}

/// 新建或更新字典项的请求。
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DictionaryItemInput {
    pub dictionary_type_id: String,
    pub code: String,
    pub label: String,
    #[serde(default)]
    pub description: String,
    pub raw_value: String,
    #[serde(default)]
    pub sort_index: i64,
    #[serde(default)]
    pub status: String,
    #[serde(default)]
    pub meta_json: String,
}

/// 字典项分页查询。
#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DictionaryItemQuery {
    pub dictionary_type_id: String,
    pub q: Option<String>,
    pub o: Option<usize>,
    pub s: Option<usize>,
}

/// 字典项分页信息。
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct DictionaryPagination {
    pub o: usize,
    pub s: usize,
}

/// 字典项分页响应。
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct DictionaryItemPage {
    pub d: Vec<DictionaryItemSummary>,
    pub t: usize,
    pub p: DictionaryPagination,
}

/// 系统字典 PostgreSQL 建表语句。
pub const DICTIONARY_BOOTSTRAP_SQL: &[&str] = &[
    "CREATE TABLE IF NOT EXISTS sys_dict_type (id TEXT PRIMARY KEY, code TEXT NOT NULL, name TEXT NOT NULL, description TEXT NOT NULL, scope TEXT NOT NULL, raw_value_kind TEXT NOT NULL, open_enum BOOLEAN NOT NULL, sort_index BIGINT NOT NULL, status TEXT NOT NULL, created_at_ms BIGINT NOT NULL, updated_at_ms BIGINT NOT NULL)",
    "CREATE UNIQUE INDEX IF NOT EXISTS sys_dict_type_code_uq ON sys_dict_type (code)",
    "CREATE INDEX IF NOT EXISTS sys_dict_type_scope_idx ON sys_dict_type (scope)",
    "CREATE TABLE IF NOT EXISTS sys_dict_data (id TEXT PRIMARY KEY, dictionary_type_id TEXT NOT NULL, code TEXT NOT NULL, label TEXT NOT NULL, description TEXT NOT NULL, raw_value TEXT NOT NULL, sort_index BIGINT NOT NULL, status TEXT NOT NULL, meta_json TEXT NOT NULL, created_at_ms BIGINT NOT NULL, updated_at_ms BIGINT NOT NULL)",
    "CREATE INDEX IF NOT EXISTS sys_dict_data_dictionary_type_id_idx ON sys_dict_data (dictionary_type_id)",
    "CREATE UNIQUE INDEX IF NOT EXISTS sys_dict_data_type_code_uq ON sys_dict_data (dictionary_type_id, code)",
    "CREATE UNIQUE INDEX IF NOT EXISTS sys_dict_data_type_raw_value_uq ON sys_dict_data (dictionary_type_id, raw_value)",
];
