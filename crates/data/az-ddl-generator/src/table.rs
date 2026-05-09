use serde::{Deserialize, Serialize};

use crate::column::Column;

/// Represents a database table definition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Table {
    /// Table name.
    pub name: String,
    /// Columns in the table.
    pub columns: Vec<Column>,
    /// Optional table comment.
    pub comment: Option<String>,
}

impl Table {
    /// Create a new table with the given name.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            columns: Vec::new(),
            comment: None,
        }
    }

    /// Add a column to this table.
    pub fn column(mut self, col: Column) -> Self {
        self.columns.push(col);
        self
    }

    /// Add a table comment.
    pub fn comment(mut self, text: impl Into<String>) -> Self {
        self.comment = Some(text.into());
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::column::ColumnType;

    #[test]
    fn table_builder() {
        let table = Table::new("users")
            .column(Column::new("id", ColumnType::BigInt).primary_key())
            .column(Column::new("name", ColumnType::Varchar(100)))
            .comment("User accounts table");

        assert_eq!(table.name, "users");
        assert_eq!(table.columns.len(), 2);
        assert!(table.columns[0].primary_key);
        assert_eq!(table.comment.as_deref(), Some("User accounts table"));
    }

    #[test]
    fn table_serialization_roundtrip() {
        let table = Table::new("orders")
            .column(Column::new("id", ColumnType::BigInt).primary_key())
            .column(Column::new("total", ColumnType::Float).not_null());

        let json = serde_json::to_string(&table).unwrap();
        let deserialized: Table = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.name, "orders");
        assert_eq!(deserialized.columns.len(), 2);
    }
}
