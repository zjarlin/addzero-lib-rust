//! 行数据的 blob 编解码与读写。
//!
//! GitDB 把每一行保存为独立 JSON 文件，文件内容同时包含 `_pk`、
//! `_version`、时间戳等元数据和用户列值。这个模块负责保持文件名、
//! 主键和 JSON 内容之间的一致性。

use std::collections::BTreeMap;

use az_derive_aliases::{apply, plain_clone_debug, plain_eq, serde_partial_eq};
use serde_json::Value;

use crate::storage::error;
pub(crate) use crate::storage::types::{BlobId, RowKey};

/// 带元数据的数据库行。
///
/// Git 中存储的内部 JSON 格式如下：
/// ```json
/// {
///   "_pk": "abc123",
///   "_version": 1,
///   "_created_at": "xxxx-xx-xxT00:00:00Z",
///   "_updated_at": "xxxx-xx-xxT00:00:00Z",
///   "name": "abc",
///   "email": "abc@example.com"
/// }
/// ```
#[apply(plain_eq)]
pub struct Row {
    /// 主键，必须与去掉 `.json` 后缀的文件名一致。
    pub key: RowKey,
    /// 用于乐观并发控制的版本号。
    pub version: u64,
    /// 创建时间戳。
    pub created_at: String,
    /// 最近更新时间戳。
    pub updated_at: String,
    /// 用户列值。
    pub data: BTreeMap<String, Value>,
}

impl Row {
    /// 使用行键和列值创建新行。
    ///
    /// 新行版本从 1 开始，创建和更新时间都设为当前时间。
    pub fn new(key: RowKey, data: BTreeMap<String, Value>) -> Self {
        let now = chrono::Utc::now().to_rfc3339();
        Self {
            key,
            version: 1,
            created_at: now.clone(),
            updated_at: now,
            data,
        }
    }

    /// 从 JSON 对象创建新行，通常用于 `INSERT`。
    pub fn from_value(key: RowKey, value: Value) -> anyhow::Result<Self> {
        let data = match value {
            Value::Object(map) => map.into_iter().collect(),
            _ => {
                let message = "row data must be a JSON object".to_string();
                let error = error::schema_violation(message);

                return Err(error);
            }
        };
        Ok(Self::new(key, data))
    }

    /// 使用新列值生成此行的更新版本。
    ///
    /// 该操作会递增版本号并刷新更新时间。
    pub fn with_update(self, new_data: BTreeMap<String, Value>) -> Self {
        Self {
            key: self.key,
            version: self.version + 1,
            created_at: self.created_at,
            updated_at: chrono::Utc::now().to_rfc3339(),
            data: new_data,
        }
    }

    /// 将部分列值合并到当前行，用于局部更新。
    pub fn merge_data(&mut self, updates: BTreeMap<String, Value>) {
        for (k, v) in updates {
            self.data.insert(k, v);
        }
        self.version += 1;
        self.updated_at = chrono::Utc::now().to_rfc3339();
    }

    /// 按列名读取值。
    pub fn get(&self, column: &str) -> Option<&Value> {
        self.data.get(column)
    }

    /// 判断行是否包含指定列。
    pub fn has_column(&self, column: &str) -> bool {
        self.data.contains_key(column)
    }
}

/// JSON 序列化使用的内部结构。
///
/// 元数据字段统一使用 `_` 前缀，避免与用户列名冲突。
#[apply(serde_partial_eq)]
struct RowJson {
    #[serde(rename = "_pk")]
    pk: String,
    #[serde(rename = "_version")]
    version: u64,
    #[serde(rename = "_created_at")]
    created_at: String,
    #[serde(rename = "_updated_at")]
    updated_at: String,
    #[serde(flatten)]
    data: BTreeMap<String, Value>,
}

/// 将行序列化为 JSON 字节。
///
/// 用户列使用 `BTreeMap` 保持稳定排序，便于 Git 复用相同内容。
pub fn serialize_row(row: &Row) -> anyhow::Result<Vec<u8>> {
    let json = RowJson {
        pk: row.key.to_string(),
        version: row.version,
        created_at: row.created_at.clone(),
        updated_at: row.updated_at.clone(),
        data: row.data.clone(),
    };

    let bytes = serde_json::to_vec_pretty(&json)?;
    Ok(bytes)
}

/// 从 JSON 字节反序列化行。
///
/// 反序列化时会校验 JSON 内 `_pk` 是否与文件名推导出的行键一致。
pub fn deserialize_row(bytes: &[u8], expected_key: &RowKey) -> anyhow::Result<Row> {
    let json: RowJson = serde_json::from_slice(bytes)?;

    // Validate primary key consistency
    if json.pk != expected_key.as_str() {
        let path = format!("{}.json", expected_key);
        let reason = format!(
            "primary key mismatch: file name suggests '{}' but content has '{}'",
            expected_key, json.pk
        );
        let error = error::corrupted_data(path.as_ref(), reason);

        return Err(error);
    }

    Ok(Row {
        key: expected_key.clone(),
        version: json.version,
        created_at: json.created_at,
        updated_at: json.updated_at,
        data: json.data,
    })
}

/// 将行写入 Git blob。
///
/// 返回值是内容对应的 Git blob ID。
pub fn write_blob(repo: &git2::Repository, row: &Row) -> anyhow::Result<BlobId> {
    let bytes = serialize_row(row)?;
    let oid = repo.blob(&bytes)?;
    Ok(BlobId::new(oid))
}

/// 从仓库读取 blob 内容。
pub fn read_blob(repo: &git2::Repository, blob_id: BlobId) -> anyhow::Result<Vec<u8>> {
    let blob = repo.find_blob(blob_id.raw())?;
    Ok(blob.content().to_vec())
}

/// 不读取完整内容时可获得的 blob 元数据。
#[apply(plain_clone_debug)]
pub struct BlobMetadata {
    pub id: BlobId,
    pub size: usize,
}

impl BlobMetadata {
    /// 从 `git2::Blob` 提取元数据。
    pub fn from_blob(blob: &git2::Blob) -> Self {
        Self {
            id: BlobId::new(blob.id()),
            size: blob.size(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_row_creation() {
        let key = RowKey::new("test123").unwrap();
        let mut data = BTreeMap::new();
        data.insert("name".to_string(), Value::String("Alice".to_string()));
        data.insert("age".to_string(), Value::Number(30.into()));

        let row = Row::new(key.clone(), data);

        assert_eq!(row.key, key);
        assert_eq!(row.version, 1);
        assert_eq!(row.get("name"), Some(&Value::String("Alice".to_string())));
    }

    #[test]
    fn test_serialization_roundtrip() {
        let key = RowKey::new("test123").unwrap();
        let mut data = BTreeMap::new();
        data.insert("name".to_string(), Value::String("Alice".to_string()));
        data.insert("count".to_string(), Value::Number(42.into()));

        let row = Row::new(key.clone(), data);
        let bytes = serialize_row(&row).unwrap();
        let restored = deserialize_row(&bytes, &key).unwrap();

        assert_eq!(row.key, restored.key);
        assert_eq!(row.version, restored.version);
        assert_eq!(row.data, restored.data);
    }

    #[test]
    fn test_serialization_format() {
        let key = RowKey::new("abc").unwrap();
        let mut data = BTreeMap::new();
        data.insert("b_field".to_string(), Value::Number(2.into()));
        data.insert("a_field".to_string(), Value::Number(1.into()));

        let row = Row::new(key, data);
        let bytes = serialize_row(&row).unwrap();
        let json_str = String::from_utf8(bytes).unwrap();

        // verify its valid json
        let parsed: serde_json::Value = serde_json::from_str(&json_str).unwrap();
        assert!(parsed.is_object());

        // check if metadata exist
        assert!(parsed.get("_pk").is_some());
        assert!(parsed.get("_version").is_some());
    }

    #[test]
    fn test_version_increment() {
        let key = RowKey::new("test").unwrap();
        let data = BTreeMap::new();
        let row = Row::new(key, data);

        assert_eq!(row.version, 1);

        let updated = row.with_update(BTreeMap::new());
        assert_eq!(updated.version, 2);
    }

    #[test]
    fn test_key_mismatch_detection() {
        let key = RowKey::new("correct").unwrap();
        let wrong_key = RowKey::new("wrong").unwrap();

        let row = Row::new(key, BTreeMap::new());
        let bytes = serialize_row(&row).unwrap();

        let result = deserialize_row(&bytes, &wrong_key);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .starts_with("corrupted data at"));
    }
}
