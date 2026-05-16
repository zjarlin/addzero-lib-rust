//!  Blob operations for row storage.
//!
//! This module handles reading & writing raw data as a JSON blob,
//! each row is stored as a separate JSON file, with a consistent format
//! that includes metadata for version tracking and conflict detection

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::storage::error::{StorageError, StorageResult};
pub(crate) use crate::storage::types::{BlobId, RowKey};

/// a db row with metadata and user data
///
/// The internal format stored in Git:
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
#[derive(Debug, Clone, PartialEq)]
pub struct Row {
    /// primary key (must match filename without . json extension)
    pub key: RowKey,
    /// version number for optimistic concurrency control
    pub version: u64,
    /// creation timestamp
    pub created_at: String,
    /// last update timestamp
    pub updated_at: String,
    /// data (column values)
    pub data: BTreeMap<String, Value>,
}

impl Row {
    /// creates a new row with key & data
    ///
    /// sets v1 and current time
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

    /// create a new row from a JSON value (typically from INSERT)
    pub fn from_value(key: RowKey, value: Value) -> StorageResult<Self> {
        let data = match value {
            Value::Object(map) => map.into_iter().collect(),
            _ => {
                return Err(StorageError::SchemaViolation(
                    "row data must be a JSON object".to_string(),
                ))
            }
        };
        Ok(Self::new(key, data))
    }

    /// create an updated version of this row
    ///
    /// increments version and updates the timestamp
    pub fn with_update(self, new_data: BTreeMap<String, Value>) -> Self {
        Self {
            key: self.key,
            version: self.version + 1,
            created_at: self.created_at,
            updated_at: chrono::Utc::now().to_rfc3339(),
            data: new_data,
        }
    }

    /// merge new data into existing data (for partial updates)
    pub fn merge_data(&mut self, updates: BTreeMap<String, Value>) {
        for (k, v) in updates {
            self.data.insert(k, v);
        }
        self.version += 1;
        self.updated_at = chrono::Utc::now().to_rfc3339();
    }

    /// get a column value by name
    pub fn get(&self, column: &str) -> Option<&Value> {
        self.data.get(column)
    }

    /// check if the row has a column
    pub fn has_column(&self, column: &str) -> bool {
        self.data.contains_key(column)
    }
}

/// internal format for JSON serialization
///
/// uses `_` prefix for metadata fields to avoid conflicts with user columns
#[derive(Serialize, Deserialize)]
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

/// serialize a row to JSON bytes
///
/// uses BTreeMap for consistent key ordering (important for git deduplication)
pub fn serialize_row(row: &Row) -> StorageResult<Vec<u8>> {
    let json = RowJson {
        pk: row.key.as_str().to_string(),
        version: row.version,
        created_at: row.created_at.clone(),
        updated_at: row.updated_at.clone(),
        data: row.data.clone(),
    };

    let bytes = serde_json::to_vec_pretty(&json)?;
    Ok(bytes)
}

/// deserialize a row from JSON bytes
///
/// validates that the primary key in the JSON matches the expected key
pub fn deserialize_row(bytes: &[u8], expected_key: &RowKey) -> StorageResult<Row> {
    let json: RowJson = serde_json::from_slice(bytes)?;

    // Validate primary key consistency
    if json.pk != expected_key.as_str() {
        return Err(StorageError::CorruptedData {
            path: format!("{}.json", expected_key).into(),
            reason: format!(
                "primary key mismatch: file name suggests '{}' but content has '{}'",
                expected_key, json.pk
            ),
        });
    }

    Ok(Row {
        key: expected_key.clone(),
        version: json.version,
        created_at: json.created_at,
        updated_at: json.updated_at,
        data: json.data,
    })
}

/// write a row as a blob to the repository
///
/// returns the blob ID (SHA-1 hash of the content)
pub fn write_blob(repo: &git2::Repository, row: &Row) -> StorageResult<BlobId> {
    let bytes = serialize_row(row)?;
    let oid = repo.blob(&bytes)?;
    Ok(BlobId::new(oid))
}

/// read a blob's content from the repository
pub fn read_blob(repo: &git2::Repository, blob_id: BlobId) -> StorageResult<Vec<u8>> {
    let blob = repo.find_blob(blob_id.raw())?;
    Ok(blob.content().to_vec())
}

/// metadata about a blob without reading its full content
#[derive(Debug, Clone)]
pub struct BlobMetadata {
    pub id: BlobId,
    pub size: usize,
}

impl BlobMetadata {
    /// get the metadata for a bolb
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
        assert!(matches!(result, Err(StorageError::CorruptedData { .. })));
    }
}
