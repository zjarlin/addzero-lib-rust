//! Query result types.

use serde_json::Value;
use std::collections::BTreeMap;

/// Result of a query execution.
#[derive(Debug)]
pub enum QueryResult {
    /// Rows returned from SELECT.
    Select(ResultSet),
    /// Number of rows affected by INSERT/UPDATE/DELETE.
    Modified { rows_affected: usize },
    /// DDL statement executed.
    Success { message: String },
    /// Transaction control result.
    Transaction { message: String },
}

impl QueryResult {
    /// Create a success result.
    pub fn success(message: impl Into<String>) -> Self {
        QueryResult::Success {
            message: message.into(),
        }
    }

    /// Create a transaction result.
    pub fn transaction(message: impl Into<String>) -> Self {
        QueryResult::Transaction {
            message: message.into(),
        }
    }

    /// Create a modified result.
    pub fn modified(rows: usize) -> Self {
        QueryResult::Modified {
            rows_affected: rows,
        }
    }
}

/// A set of rows from a SELECT query.
#[derive(Debug, Clone)]
pub struct ResultSet {
    /// Column names in order.
    pub columns: Vec<String>,
    /// Rows as maps of column name to value.
    pub rows: Vec<BTreeMap<String, Value>>,
}

impl ResultSet {
    /// Create a new empty result set.
    pub fn new(columns: Vec<String>) -> Self {
        Self {
            columns,
            rows: Vec::new(),
        }
    }

    /// Create from rows, inferring columns from first row.
    pub fn from_rows(rows: Vec<BTreeMap<String, Value>>) -> Self {
        let columns = rows
            .first()
            .map(|r| r.keys().cloned().collect())
            .unwrap_or_default();
        Self { columns, rows }
    }

    /// Add a row.
    pub fn push(&mut self, row: BTreeMap<String, Value>) {
        self.rows.push(row);
    }

    /// Number of rows.
    pub fn len(&self) -> usize {
        self.rows.len()
    }

    /// Check if empty.
    pub fn is_empty(&self) -> bool {
        self.rows.is_empty()
    }

    /// Get a row by index.
    pub fn get(&self, index: usize) -> Option<&BTreeMap<String, Value>> {
        self.rows.get(index)
    }

    /// Iterate over rows.
    pub fn iter(&self) -> impl Iterator<Item = &BTreeMap<String, Value>> {
        self.rows.iter()
    }
}

impl IntoIterator for ResultSet {
    type Item = BTreeMap<String, Value>;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.rows.into_iter()
    }
}

/// Iterator over query result rows.
pub struct RowIter {
    rows: std::vec::IntoIter<BTreeMap<String, Value>>,
}

impl RowIter {
    pub fn new(rows: Vec<BTreeMap<String, Value>>) -> Self {
        Self {
            rows: rows.into_iter(),
        }
    }
}

impl Iterator for RowIter {
    type Item = BTreeMap<String, Value>;

    fn next(&mut self) -> Option<Self::Item> {
        self.rows.next()
    }
}

impl ExactSizeIterator for RowIter {
    fn len(&self) -> usize {
        self.rows.len()
    }
}
