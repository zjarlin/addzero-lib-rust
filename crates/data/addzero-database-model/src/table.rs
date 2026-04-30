use serde::{Deserialize, Serialize};

/// Represents a table definition in the database schema.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Table {
    /// Table name.
    pub name: String,
    /// Columns in this table.
    pub columns: Vec<super::Column>,
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

    /// Add a column.
    pub fn column(mut self, col: super::Column) -> Self {
        self.columns.push(col);
        self
    }

    /// Set a table comment.
    pub fn comment(mut self, text: impl Into<String>) -> Self {
        self.comment = Some(text.into());
        self
    }

    /// Get a column by name.
    pub fn get_column(&self, name: &str) -> Option<&super::Column> {
        self.columns.iter().find(|c| c.name == name)
    }

    /// Get the primary key column, if any.
    pub fn primary_key(&self) -> Option<&super::Column> {
        self.columns.iter().find(|c| c.primary_key)
    }

    /// Get all columns marked as NOT NULL.
    pub fn required_columns(&self) -> Vec<&super::Column> {
        self.columns.iter().filter(|c| c.not_null).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::column::DataType;

    #[test]
    fn table_builder() {
        let table = Table::new("users")
            .column(
                super::super::Column::new("id", DataType::BigInt)
                    .primary_key(),
            )
            .column(
                super::super::Column::new("name", DataType::Varchar(100))
                    .not_null(),
            )
            .comment("User accounts");

        assert_eq!(table.name, "users");
        assert_eq!(table.columns.len(), 2);
        assert!(table.primary_key().is_some());
        assert_eq!(table.primary_key().unwrap().name, "id");
    }

    #[test]
    fn get_column_by_name() {
        let table = Table::new("orders")
            .column(super::super::Column::new("id", DataType::BigInt))
            .column(super::super::Column::new(
                "total",
                DataType::Decimal {
                    precision: 10,
                    scale: 2,
                },
            ));

        let col = table.get_column("total");
        assert!(col.is_some());
        assert_eq!(
            col.unwrap().data_type,
            DataType::Decimal {
                precision: 10,
                scale: 2,
            }
        );

        assert!(table.get_column("nonexistent").is_none());
    }

    #[test]
    fn required_columns() {
        let table = Table::new("items")
            .column(super::super::Column::new("id", DataType::BigInt).not_null())
            .column(
                super::super::Column::new("name", DataType::Varchar(255))
                    .not_null(),
            )
            .column(super::super::Column::new("description", DataType::Text));

        let required = table.required_columns();
        assert_eq!(required.len(), 2);
    }
}
