use az_derive_aliases::{apply, serde_eq};

use anyhow::{Result, bail};

use crate::index::Index;
use crate::relation::Relation;
use crate::table::Table;

/// Represents a complete database schema.
#[apply(serde_eq)]
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
    pub fn validate(&self) -> Result<()> {
        let mut errors = Vec::new();

        if self.name.is_empty() {
            errors.push("empty schema name".to_owned());
        }

        let mut seen_tables = std::collections::HashSet::new();
        for table in &self.tables {
            if table.name.is_empty() {
                errors.push(format!("empty table name in schema '{}'", self.name));
            }
            if !seen_tables.insert(&table.name) {
                errors.push(format!("duplicate table name: '{}'", table.name));
            }

            let mut seen_cols = std::collections::HashSet::new();
            for col in &table.columns {
                if col.name.is_empty() {
                    errors.push(format!("empty column name in table '{}'", table.name));
                }
                if !seen_cols.insert(&col.name) {
                    errors.push(format!(
                        "duplicate column '{}' in table '{}'",
                        col.name, table.name
                    ));
                }
            }
        }

        for rel in &self.relations {
            let from_table = self.get_table(&rel.from_table);
            let to_table = self.get_table(&rel.to_table);

            match from_table {
                Some(table) => {
                    if table.get_column(&rel.from_column).is_none() {
                        errors.push(format!(
                            "relation references unknown column '{}' in table '{}'",
                            rel.from_column, rel.from_table
                        ));
                    }
                }
                None => {
                    errors.push(format!("relation references unknown table '{}'", rel.from_table));
                }
            }

            match to_table {
                Some(table) => {
                    if table.get_column(&rel.to_column).is_none() {
                        errors.push(format!(
                            "relation references unknown column '{}' in table '{}'",
                            rel.to_column, rel.to_table
                        ));
                    }
                }
                None => {
                    errors.push(format!(
                        "relation references unknown table '{}'",
                        rel.to_table
                    ));
                }
            }
        }

        for idx in &self.indexes {
            if let Some(table) = self.get_table(&idx.table) {
                for col_name in &idx.columns {
                    if table.get_column(col_name).is_none() {
                        errors.push(format!(
                            "index '{}' references unknown column '{}' in table '{}'",
                            idx.name, col_name, idx.table
                        ));
                    }
                }
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            bail!("schema validation failed: {}", errors.join("; "))
        }
    }

    /// Serialize the schema to JSON.
    pub fn to_json(&self) -> Result<String> {
        Ok(serde_json::to_string_pretty(self)?)
    }

    /// Deserialize a schema from JSON.
    pub fn from_json(json: &str) -> Result<Self> {
        Ok(serde_json::from_str(json)?)
    }
}

#[cfg(test)]
mod tests {
    use crate::column::{Column, DataType};
    use crate::index::Index;
    use crate::relation::{Relation, RelationKind};
    use crate::schema::Schema;
    use crate::table::Table;

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
            .index(Index::new(
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
        let error = schema.validate().unwrap_err();
        assert!(error.to_string().contains("empty schema name"));
    }

    #[test]
    fn duplicate_table_fails_validation() {
        let schema = Schema::new("test")
            .table(Table::new("users").column(Column::new("id", DataType::Integer)))
            .table(Table::new("users").column(Column::new("id", DataType::Integer)));
        let error = schema.validate().unwrap_err();
        assert!(error.to_string().contains("duplicate table name"));
    }

    #[test]
    fn duplicate_column_fails_validation() {
        let schema = Schema::new("test").table(
            Table::new("t")
                .column(Column::new("id", DataType::Integer))
                .column(Column::new("id", DataType::Text)),
        );
        let error = schema.validate().unwrap_err();
        assert!(error.to_string().contains("duplicate column"));
    }

    #[test]
    fn relation_to_unknown_table_fails() {
        let schema = Schema::new("test")
            .table(Table::new("users").column(Column::new("id", DataType::BigInt)))
            .relation(Relation::new(
                "fk_bad",
                "users",
                "id",
                "nonexistent",
                "id",
                RelationKind::ManyToOne,
            ));
        let error = schema.validate().unwrap_err();
        assert!(error.to_string().contains("unknown table 'nonexistent'"));
    }

    #[test]
    fn relation_to_unknown_column_fails() {
        let schema = Schema::new("test")
            .table(Table::new("users").column(Column::new("id", DataType::BigInt)))
            .table(Table::new("orders").column(Column::new("id", DataType::BigInt)))
            .relation(Relation::new(
                "fk_bad",
                "orders",
                "nonexistent_col",
                "users",
                "id",
                RelationKind::ManyToOne,
            ));
        let error = schema.validate().unwrap_err();
        assert!(error.to_string().contains("unknown column 'nonexistent_col'"));
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
            .table(Table::new("users").column(Column::new("id", DataType::BigInt)))
            .index(Index::new(
                "idx_bad",
                "users",
                vec!["nonexistent".into()],
            ));
        let error = schema.validate().unwrap_err();
        assert!(error.to_string().contains("idx_bad"));
        assert!(error.to_string().contains("nonexistent"));
    }
}
