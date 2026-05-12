//! Table schema definitions and validation.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::types::{ColumnDef, Constraint, DataType};

/// Schema version for tracking migrations.
pub type SchemaVersion = u32;

/// Table schema definition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableSchema {
    /// Table name.
    pub name: String,
    /// Schema version (incremented on each modification).
    pub version: SchemaVersion,
    /// Column definitions.
    pub columns: Vec<ColumnDef>,
    /// Primary key column name (optional, defaults to auto-generated row key).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub primary_key: Option<String>,
    /// Table description/comment.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Creation timestamp.
    pub created_at: DateTime<Utc>,
    /// Last modification timestamp.
    pub updated_at: DateTime<Utc>,
}

impl TableSchema {
    /// Create a new table schema.
    pub fn new(name: impl Into<String>, columns: Vec<ColumnDef>) -> Self {
        let now = Utc::now();
        Self {
            name: name.into(),
            version: 1,
            columns,
            primary_key: None,
            description: None,
            created_at: now,
            updated_at: now,
        }
    }

    /// Set the primary key column.
    pub fn with_primary_key(mut self, column_name: impl Into<String>) -> Self {
        self.primary_key = Some(column_name.into());
        self
    }

    /// Set the table description.
    pub fn with_description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }

    /// Get a column definition by name.
    pub fn get_column(&self, name: &str) -> Option<&ColumnDef> {
        self.columns.iter().find(|c| c.name == name)
    }

    /// Get column names.
    pub fn column_names(&self) -> Vec<&str> {
        self.columns.iter().map(|c| c.name.as_str()).collect()
    }

    /// Validate the schema itself (e.g., primary key exists).
    pub fn validate(&self) -> Result<(), SchemaError> {
        // Check for duplicate column names
        let mut seen = std::collections::HashSet::new();
        for col in &self.columns {
            if !seen.insert(&col.name) {
                return Err(SchemaError::DuplicateColumn(col.name.clone()));
            }
        }

        // Check primary key references valid column
        if let Some(pk) = &self.primary_key {
            if !self.columns.iter().any(|c| &c.name == pk) {
                return Err(SchemaError::InvalidPrimaryKey(pk.clone()));
            }
        }

        Ok(())
    }

    /// Validate a row against this schema.
    pub fn validate_row(&self, row: &Value) -> Result<(), SchemaError> {
        let obj = row
            .as_object()
            .ok_or_else(|| SchemaError::InvalidRow("row must be a JSON object".into()))?;

        // Check all columns
        for col in &self.columns {
            let value = obj.get(&col.name);
            col.validate(value)
                .map_err(|e| SchemaError::InvalidRow(e))?;
        }

        Ok(())
    }

    /// Apply defaults to a row, returning a new row with defaults filled in.
    pub fn apply_defaults(&self, row: &Value) -> Result<Value, SchemaError> {
        let mut obj = row
            .as_object()
            .cloned()
            .ok_or_else(|| SchemaError::InvalidRow("row must be a JSON object".into()))?;

        for col in &self.columns {
            if !obj.contains_key(&col.name) {
                if let Some(default) = col.default_value() {
                    obj.insert(col.name.clone(), default.clone());
                }
            }
        }

        Ok(Value::Object(obj))
    }

    /// Increment the version and update timestamp.
    pub fn bump_version(&mut self) {
        self.version += 1;
        self.updated_at = Utc::now();
    }

    /// Add a new column (schema migration).
    pub fn add_column(&mut self, column: ColumnDef) -> Result<(), SchemaError> {
        if self.columns.iter().any(|c| c.name == column.name) {
            return Err(SchemaError::DuplicateColumn(column.name.clone()));
        }
        self.columns.push(column);
        self.bump_version();
        Ok(())
    }

    /// Remove a column (schema migration).
    pub fn remove_column(&mut self, name: &str) -> Result<ColumnDef, SchemaError> {
        let pos = self
            .columns
            .iter()
            .position(|c| c.name == name)
            .ok_or_else(|| SchemaError::ColumnNotFound(name.to_string()))?;

        // Don't allow removing primary key
        if self.primary_key.as_deref() == Some(name) {
            return Err(SchemaError::CannotRemovePrimaryKey(name.to_string()));
        }

        let col = self.columns.remove(pos);
        self.bump_version();
        Ok(col)
    }
}

/// Schema-related errors.
#[derive(Debug, Clone, thiserror::Error)]
pub enum SchemaError {
    #[error("duplicate column: {0}")]
    DuplicateColumn(String),

    #[error("invalid primary key reference: {0}")]
    InvalidPrimaryKey(String),

    #[error("column not found: {0}")]
    ColumnNotFound(String),

    #[error("cannot remove primary key column: {0}")]
    CannotRemovePrimaryKey(String),

    #[error("invalid row: {0}")]
    InvalidRow(String),

    #[error("table already exists: {0}")]
    TableExists(String),

    #[error("table not found: {0}")]
    TableNotFound(String),

    #[error("schema version mismatch: expected {expected}, found {found}")]
    VersionMismatch {
        expected: SchemaVersion,
        found: SchemaVersion,
    },

    #[error("storage error: {0}")]
    Storage(String),
}

/// Builder for creating table schemas.
pub struct SchemaBuilder {
    name: String,
    columns: Vec<ColumnDef>,
    primary_key: Option<String>,
    description: Option<String>,
}

impl SchemaBuilder {
    /// Start building a new schema.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            columns: Vec::new(),
            primary_key: None,
            description: None,
        }
    }

    /// Add a column.
    pub fn column(mut self, column: ColumnDef) -> Self {
        self.columns.push(column);
        self
    }

    /// Add a simple column with just name and type.
    pub fn add_column(mut self, name: impl Into<String>, data_type: DataType) -> Self {
        self.columns.push(ColumnDef::new(name, data_type));
        self
    }

    /// Add a non-nullable column.
    pub fn add_required_column(mut self, name: impl Into<String>, data_type: DataType) -> Self {
        self.columns
            .push(ColumnDef::new(name, data_type).with_constraint(Constraint::NotNull));
        self
    }

    /// Set the primary key.
    pub fn primary_key(mut self, column_name: impl Into<String>) -> Self {
        self.primary_key = Some(column_name.into());
        self
    }

    /// Set the description.
    pub fn description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }

    /// Build the schema.
    pub fn build(self) -> Result<TableSchema, SchemaError> {
        let mut schema = TableSchema::new(self.name, self.columns);
        if let Some(pk) = self.primary_key {
            schema = schema.with_primary_key(pk);
        }
        if let Some(desc) = self.description {
            schema = schema.with_description(desc);
        }
        schema.validate()?;
        Ok(schema)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn sample_schema() -> TableSchema {
        SchemaBuilder::new("users")
            .add_required_column("id", DataType::Uuid)
            .add_required_column("name", DataType::Text)
            .add_column("email", DataType::Text)
            .add_column("age", DataType::Integer)
            .primary_key("id")
            .description("User accounts")
            .build()
            .unwrap()
    }

    #[test]
    fn test_schema_validation() {
        let schema = sample_schema();
        assert!(schema.validate().is_ok());
        assert_eq!(schema.primary_key, Some("id".to_string()));
    }

    #[test]
    fn test_schema_duplicate_column() {
        let result = SchemaBuilder::new("bad")
            .add_column("name", DataType::Text)
            .add_column("name", DataType::Integer) // duplicate!
            .build();

        assert!(matches!(result, Err(SchemaError::DuplicateColumn(_))));
    }

    #[test]
    fn test_schema_invalid_primary_key() {
        let result = SchemaBuilder::new("bad")
            .add_column("name", DataType::Text)
            .primary_key("id") // doesn't exist!
            .build();

        assert!(matches!(result, Err(SchemaError::InvalidPrimaryKey(_))));
    }

    #[test]
    fn test_row_validation() {
        let schema = sample_schema();

        let valid_row = json!({
            "id": "550e8400-e29b-41d4-a716-446655440000",
            "name": "Alice",
            "email": "alice@example.com",
            "age": 30
        });
        assert!(schema.validate_row(&valid_row).is_ok());

        // Missing required field
        let invalid_row = json!({
            "id": "550e8400-e29b-41d4-a716-446655440000"
            // missing name!
        });
        assert!(schema.validate_row(&invalid_row).is_err());

        // Wrong type
        let wrong_type = json!({
            "id": "550e8400-e29b-41d4-a716-446655440000",
            "name": "Alice",
            "age": "thirty" // should be integer
        });
        assert!(schema.validate_row(&wrong_type).is_err());
    }

    #[test]
    fn test_apply_defaults() {
        let schema = SchemaBuilder::new("items")
            .add_required_column("id", DataType::Text)
            .column(
                ColumnDef::new("status", DataType::Text)
                    .with_constraint(Constraint::Default(json!("pending"))),
            )
            .build()
            .unwrap();

        let row = json!({"id": "123"});
        let with_defaults = schema.apply_defaults(&row).unwrap();

        assert_eq!(with_defaults["status"], "pending");
    }

    #[test]
    fn test_schema_migration() {
        let mut schema = sample_schema();
        let initial_version = schema.version;

        // Add column
        schema
            .add_column(ColumnDef::new("bio", DataType::Text))
            .unwrap();
        assert_eq!(schema.version, initial_version + 1);
        assert!(schema.get_column("bio").is_some());

        // Remove column
        let removed = schema.remove_column("bio").unwrap();
        assert_eq!(removed.name, "bio");
        assert_eq!(schema.version, initial_version + 2);

        // Cannot remove primary key
        let result = schema.remove_column("id");
        assert!(matches!(
            result,
            Err(SchemaError::CannotRemovePrimaryKey(_))
        ));
    }

    #[test]
    fn test_schema_serialization() {
        let schema = sample_schema();
        let json = serde_json::to_string_pretty(&schema).unwrap();
        let deserialized: TableSchema = serde_json::from_str(&json).unwrap();

        assert_eq!(schema.name, deserialized.name);
        assert_eq!(schema.columns.len(), deserialized.columns.len());
        assert_eq!(schema.primary_key, deserialized.primary_key);
    }
}
