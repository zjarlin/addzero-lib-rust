use serde::{Deserialize, Serialize};

use crate::{Index, ModelError, Relation, Table};

/// Represents a complete database schema.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Schema {
    /// Schema or database name.
    pub name: String,
    /// Tables in this schema.
    pub tables: Vec<Table>,
    /// Foreign key relations between tables.
    pub relations: Vec<Relation>,
    /// Indexes across tables.
    pub indexes: Vec<Index>,
}

impl Schema {
    /// Create a new empty schema.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            tables: Vec::new(),
            relations: Vec::new(),
            indexes: Vec::new(),
        }
    }

    /// Add a table to the schema.
    pub fn table(mut self, table: Table) -> Self {
        self.tables.push(table);
        self
    }

    /// Add a relation to the schema.
    pub fn relation(mut self, relation: Relation) -> Self {
        self.relations.push(relation);
        self
    }

    /// Add an index to the schema.
    pub fn index(mut self, index: Index) -> Self {
        self.indexes.push(index);
        self
    }

    /// Get a table by name.
    pub fn get_table(&self, name: &str) -> Option<&Table> {
        self.tables.iter().find(|t| t.name == name)
    }

    /// Validate the schema for consistency.
    #[allow(clippy::collapsible_if)]
    pub fn validate(&self) -> Result<(), Vec<ModelError>> {
        let mut errors = Vec::new();

        if self.name.is_empty() {
            errors.push(ModelError::EmptySchemaName);
        }

        // Check for duplicate table names
        let mut seen_tables = std::collections::HashSet::new();
        for table in &self.tables {
            if table.name.is_empty() {
                errors.push(ModelError::EmptyTableName {
                    schema: self.name.clone(),
                });
            }
            if !seen_tables.insert(&table.name) {
                errors.push(ModelError::DuplicateTable(table.name.clone()));
            }

            // Check for duplicate columns
            let mut seen_cols = std::collections::HashSet::new();
            for col in &table.columns {
                if col.name.is_empty() {
                    errors.push(ModelError::EmptyColumnName {
                        table: table.name.clone(),
                    });
                }
                if !seen_cols.insert(&col.name) {
                    errors.push(ModelError::DuplicateColumn {
                        table: table.name.clone(),
                        column: col.name.clone(),
                    });
                }
            }
        }

        // Validate relations reference existing tables/columns
        for rel in &self.relations {
            let from_table = self.get_table(&rel.from_table);
            let to_table = self.get_table(&rel.to_table);

            if from_table.is_none() {
                errors.push(ModelError::UnknownTable(rel.from_table.clone()));
            } else if let Some(t) = from_table {
                if t.get_column(&rel.from_column).is_none() {
                    errors.push(ModelError::UnknownColumn {
                        table: rel.from_table.clone(),
                        column: rel.from_column.clone(),
                    });
                }
            }

            if to_table.is_none() {
                errors.push(ModelError::UnknownTable(rel.to_table.clone()));
            } else if let Some(t) = to_table {
                if t.get_column(&rel.to_column).is_none() {
                    errors.push(ModelError::UnknownColumn {
                        table: rel.to_table.clone(),
                        column: rel.to_column.clone(),
                    });
                }
            }
        }

        // Validate indexes reference existing columns
        for idx in &self.indexes {
            if let Some(table) = self.get_table(&idx.table) {
                for col_name in &idx.columns {
                    if table.get_column(col_name).is_none() {
                        errors.push(ModelError::UnknownIndexColumn {
                            index: idx.name.clone(),
                            table: idx.table.clone(),
                            column: col_name.clone(),
                        });
                    }
                }
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    /// Serialize the schema to JSON.
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    /// Deserialize a schema from JSON.
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::column::{Column, DataType};
    use crate::relation::RelationKind;

    fn sample_schema() -> Schema {
        Schema::new("myapp")
            .table(
                Table::new("users")
                    .column(
                        Column::new("id", DataType::BigInt)
                            .primary_key()
                            .auto_increment(),
                    )
                    .column(Column::new("name", DataType::Varchar(255)).not_null())
                    .column(Column::new("email", DataType::Varchar(255)).unique()),
            )
            .table(
                Table::new("orders")
                    .column(Column::new("id", DataType::BigInt).primary_key())
                    .column(Column::new("user_id", DataType::BigInt).not_null())
                    .column(Column::new(
                        "total",
                        DataType::Decimal {
                            precision: 10,
                            scale: 2,
                        },
                    )),
            )
            .relation(
                Relation::new(
                    "fk_order_user",
                    "orders",
                    "user_id",
                    "users",
                    "id",
                    RelationKind::ManyToOne,
                )
                .on_delete_cascade(),
            )
            .index(crate::Index::new(
                "idx_order_user",
                "orders",
                vec!["user_id".into()],
            ))
    }

    #[test]
    fn schema_builder() {
        let schema = sample_schema();
        assert_eq!(schema.name, "myapp");
        assert_eq!(schema.tables.len(), 2);
        assert_eq!(schema.relations.len(), 1);
        assert_eq!(schema.indexes.len(), 1);
    }

    #[test]
    fn get_table_by_name() {
        let schema = sample_schema();
        assert!(schema.get_table("users").is_some());
        assert!(schema.get_table("orders").is_some());
        assert!(schema.get_table("nonexistent").is_none());
    }

    #[test]
    fn valid_schema_passes_validation() {
        let schema = sample_schema();
        assert!(schema.validate().is_ok());
    }

    #[test]
    fn empty_schema_name_fails_validation() {
        let schema = Schema::new("");
        let errors = schema.validate().unwrap_err();
        assert!(errors.contains(&ModelError::EmptySchemaName));
    }

    #[test]
    fn duplicate_table_fails_validation() {
        let schema = Schema::new("test")
            .table(
                Table::new("users")
                    .column(Column::new("id", DataType::Integer)),
            )
            .table(
                Table::new("users")
                    .column(Column::new("id", DataType::Integer)),
            );
        let errors = schema.validate().unwrap_err();
        assert!(errors.contains(&ModelError::DuplicateTable("users".to_string())));
    }

    #[test]
    fn duplicate_column_fails_validation() {
        let schema = Schema::new("test").table(
            Table::new("t")
                .column(Column::new("id", DataType::Integer))
                .column(Column::new("id", DataType::Text)),
        );
        let errors = schema.validate().unwrap_err();
        assert!(errors.contains(&ModelError::DuplicateColumn {
            table: "t".to_string(),
            column: "id".to_string(),
        }));
    }

    #[test]
    fn relation_to_unknown_table_fails() {
        let schema = Schema::new("test")
            .table(
                Table::new("users")
                    .column(Column::new("id", DataType::BigInt)),
            )
            .relation(Relation::new(
                "fk_bad",
                "users",
                "id",
                "nonexistent",
                "id",
                RelationKind::ManyToOne,
            ));
        let errors = schema.validate().unwrap_err();
        assert!(errors.contains(&ModelError::UnknownTable("nonexistent".to_string())));
    }

    #[test]
    fn relation_to_unknown_column_fails() {
        let schema = Schema::new("test")
            .table(
                Table::new("users")
                    .column(Column::new("id", DataType::BigInt)),
            )
            .table(
                Table::new("orders")
                    .column(Column::new("id", DataType::BigInt)),
            )
            .relation(Relation::new(
                "fk_bad",
                "orders",
                "nonexistent_col",
                "users",
                "id",
                RelationKind::ManyToOne,
            ));
        let errors = schema.validate().unwrap_err();
        assert!(errors.contains(&ModelError::UnknownColumn {
            table: "orders".to_string(),
            column: "nonexistent_col".to_string(),
        }));
    }

    #[test]
    fn schema_json_roundtrip() {
        let schema = sample_schema();
        let json = schema.to_json().unwrap();
        let deserialized = Schema::from_json(&json).unwrap();
        assert_eq!(deserialized.name, "myapp");
        assert_eq!(deserialized.tables.len(), 2);
        assert_eq!(deserialized.relations.len(), 1);
    }

    #[test]
    fn index_references_unknown_column() {
        let schema = Schema::new("test")
            .table(
                Table::new("users")
                    .column(Column::new("id", DataType::BigInt)),
            )
            .index(crate::Index::new(
                "idx_bad",
                "users",
                vec!["nonexistent".into()],
            ));
        let errors = schema.validate().unwrap_err();
        assert!(errors.contains(&ModelError::UnknownIndexColumn {
            index: "idx_bad".to_string(),
            table: "users".to_string(),
            column: "nonexistent".to_string(),
        }));
    }
}
