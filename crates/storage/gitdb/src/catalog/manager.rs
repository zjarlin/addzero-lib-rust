//! Catalog manager for schema persistence and retrieval.

use std::collections::BTreeMap;
use std::sync::Arc;

use anyhow::{Context, bail};
use parking_lot::RwLock;
use serde_json::Value;

use super::schema::TableSchema;
use crate::storage::{GitRepository, Row, RowKey, TableName, is_not_found};

/// Directory where schemas are stored.
const SCHEMA_DIR: &str = "_schemas";

/// The catalog manages table schemas, storing them in the repository.
pub struct Catalog {
    repo: Arc<RwLock<GitRepository>>,
}

impl Catalog {
    /// Create a new catalog backed by the given repository.
    pub fn new(repo: Arc<RwLock<GitRepository>>) -> Self {
        Self { repo }
    }

    /// Create a new table schema.
    pub fn create_table(&self, schema: TableSchema) -> anyhow::Result<()> {
        // Validate schema
        schema.validate()?;

        let repo = self.repo.write();
        let head = repo.head().context("failed to read catalog head")?;

        let table_name = TableName::new(SCHEMA_DIR)?;

        // Ensure _schemas table exists
        let head = if !repo
            .table_exists(&table_name, head)
            .context("failed to check catalog schema table")?
        {
            repo.create_table(&table_name, head, None)
                .context("failed to create catalog schema table")?
        } else {
            head
        };

        // Check if table schema already exists
        let row_key = RowKey::new(&schema.name)?;
        if repo
            .read_row(&table_name, &row_key, head)
            .with_context(|| format!("failed to read schema row for table `{}`", schema.name))?
            .is_some()
        {
            bail!("table already exists: {}", schema.name);
        }

        // Serialize schema to row data
        let schema_json = serde_json::to_value(&schema)
            .with_context(|| format!("failed to serialize schema `{}`", schema.name))?;
        let mut data = BTreeMap::new();
        data.insert("schema".to_string(), schema_json);
        let row = Row::new(row_key, data);

        let new_head = repo
            .upsert_row(&table_name, row, head, None)
            .with_context(|| format!("failed to write schema row for table `{}`", schema.name))?;

        // Update main branch
        repo.update_branch(&crate::storage::BranchName::main(), new_head)
            .context("failed to update main branch after catalog create")?;

        Ok(())
    }

    /// Get a table schema by name.
    pub fn get_table(&self, name: &str) -> anyhow::Result<TableSchema> {
        let repo = self.repo.read();
        let head = repo.head().context("failed to read catalog head")?;

        let table_name = TableName::new(SCHEMA_DIR)?;

        let row_key = RowKey::new(name)?;

        let row = match repo.read_row(&table_name, &row_key, head) {
            Ok(Some(row)) => row,
            Ok(None) => bail!("table not found: {name}"),
            Err(error) if is_not_found(&error) => bail!("table not found: {name}"),
            Err(error) => {
                return Err(error)
                    .with_context(|| format!("failed to read schema row for table `{name}`"));
            }
        };

        let Some(schema_value) = row.get("schema") else {
            bail!("missing schema field");
        };

        serde_json::from_value(schema_value.clone())
            .with_context(|| format!("failed to deserialize schema `{name}`"))
    }

    /// Check if a table exists.
    pub fn table_exists(&self, name: &str) -> bool {
        let repo = self.repo.read();
        let head = match repo.head() {
            Ok(h) => h,
            Err(_) => return false,
        };

        let table_name = match TableName::new(SCHEMA_DIR) {
            Ok(t) => t,
            Err(_) => return false,
        };

        let row_key = match RowKey::new(name) {
            Ok(k) => k,
            Err(_) => return false,
        };

        repo.read_row(&table_name, &row_key, head)
            .map(|opt| opt.is_some())
            .unwrap_or(false)
    }

    /// Update a table schema (for migrations).
    pub fn update_table(&self, schema: TableSchema) -> anyhow::Result<()> {
        schema.validate()?;

        let repo = self.repo.write();
        let head = repo.head().context("failed to read catalog head")?;

        // Check table exists and verify version
        let table_name = TableName::new(SCHEMA_DIR)?;

        let row_key = RowKey::new(&schema.name)?;

        let existing_row = repo
            .read_row(&table_name, &row_key, head)
            .with_context(|| format!("failed to read schema row for table `{}`", schema.name))?;
        let Some(existing_row) = existing_row else {
            bail!("table not found: {}", schema.name);
        };

        let Some(existing_schema_value) = existing_row.get("schema") else {
            bail!("corrupted schema");
        };
        let existing_schema: TableSchema = serde_json::from_value(existing_schema_value.clone())
            .with_context(|| format!("failed to deserialize schema `{}`", schema.name))?;

        // Verify version is incrementing
        if schema.version <= existing_schema.version {
            bail!(
                "schema version mismatch: expected {}, found {}",
                existing_schema.version + 1,
                schema.version
            );
        }

        // Store updated schema
        let schema_json = serde_json::to_value(&schema)
            .with_context(|| format!("failed to serialize schema `{}`", schema.name))?;
        let mut data = BTreeMap::new();
        data.insert("schema".to_string(), schema_json);
        let row = Row::new(row_key, data);

        let new_head = repo
            .upsert_row(&table_name, row, head, None)
            .with_context(|| format!("failed to write schema row for table `{}`", schema.name))?;

        repo.update_branch(&crate::storage::BranchName::main(), new_head)
            .context("failed to update main branch after catalog update")?;

        Ok(())
    }

    /// Drop a table schema.
    pub fn drop_table(&self, name: &str) -> anyhow::Result<()> {
        let repo = self.repo.write();
        let head = repo.head().context("failed to read catalog head")?;

        let table_name = TableName::new(SCHEMA_DIR)?;

        // Check if _schemas table exists
        if !repo
            .table_exists(&table_name, head)
            .context("failed to check catalog schema table")?
        {
            bail!("table not found: {name}");
        }

        let row_key = RowKey::new(name)?;

        // Check exists
        if repo
            .read_row(&table_name, &row_key, head)
            .with_context(|| format!("failed to read schema row for table `{name}`"))?
            .is_none()
        {
            bail!("table not found: {name}");
        }

        let new_head = repo
            .delete_row(&table_name, &row_key, head, None)
            .with_context(|| format!("failed to delete schema row for table `{name}`"))?;

        repo.update_branch(&crate::storage::BranchName::main(), new_head)
            .context("failed to update main branch after catalog drop")?;

        Ok(())
    }

    /// List all table names.
    pub fn list_tables(&self) -> anyhow::Result<Vec<String>> {
        let repo = self.repo.read();
        let head = repo.head().context("failed to read catalog head")?;

        let table_name = TableName::new(SCHEMA_DIR)?;

        // Table might not exist yet
        if !repo
            .table_exists(&table_name, head)
            .context("failed to check catalog schema table")?
        {
            return Ok(Vec::new());
        }

        let rows = repo
            .list_rows(&table_name, head)
            .context("failed to list catalog schemas")?;

        Ok(rows.into_iter().map(|key| key.to_string()).collect())
    }

    /// Validate a row against a table's schema.
    pub fn validate_row(&self, table_name: &str, row: &Value) -> anyhow::Result<()> {
        let schema = self.get_table(table_name)?;
        schema.validate_row(row)
    }

    /// Apply defaults to a row based on table schema.
    pub fn apply_defaults(&self, table_name: &str, row: &Value) -> anyhow::Result<Value> {
        let schema = self.get_table(table_name)?;
        schema.apply_defaults(row)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::catalog::schema::SchemaBuilder;
    use crate::catalog::types::DataType;
    use tempfile::TempDir;

    fn setup_catalog() -> (Catalog, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let repo = GitRepository::open_or_init(temp_dir.path()).unwrap();
        let catalog = Catalog::new(Arc::new(RwLock::new(repo)));
        (catalog, temp_dir)
    }

    #[test]
    fn test_create_and_get_table() {
        let (catalog, _dir) = setup_catalog();

        let schema = SchemaBuilder::new("users")
            .add_required_column("id", DataType::Text)
            .add_required_column("name", DataType::Text)
            .primary_key("id")
            .build()
            .unwrap();

        catalog.create_table(schema).unwrap();

        let retrieved = catalog.get_table("users").unwrap();
        assert_eq!(retrieved.name, "users");
        assert_eq!(retrieved.columns.len(), 2);
        assert_eq!(retrieved.primary_key, Some("id".to_string()));
    }

    #[test]
    fn test_table_exists() {
        let (catalog, _dir) = setup_catalog();

        assert!(!catalog.table_exists("users"));

        let schema = SchemaBuilder::new("users")
            .add_column("id", DataType::Text)
            .build()
            .unwrap();

        catalog.create_table(schema).unwrap();
        assert!(catalog.table_exists("users"));
    }

    #[test]
    fn test_create_duplicate_table() {
        let (catalog, _dir) = setup_catalog();

        let schema = SchemaBuilder::new("users")
            .add_column("id", DataType::Text)
            .build()
            .unwrap();

        catalog.create_table(schema.clone()).unwrap();

        let result = catalog.create_table(schema);
        assert_eq!(
            result.unwrap_err().to_string(),
            "table already exists: users"
        );
    }

    #[test]
    fn test_drop_table() {
        let (catalog, _dir) = setup_catalog();

        let schema = SchemaBuilder::new("users")
            .add_column("id", DataType::Text)
            .build()
            .unwrap();

        catalog.create_table(schema).unwrap();
        assert!(catalog.table_exists("users"));

        catalog.drop_table("users").unwrap();
        assert!(!catalog.table_exists("users"));
    }

    #[test]
    fn test_drop_nonexistent_table() {
        let (catalog, _dir) = setup_catalog();

        let result = catalog.drop_table("nonexistent");
        assert_eq!(
            result.unwrap_err().to_string(),
            "table not found: nonexistent"
        );
    }

    #[test]
    fn test_list_tables() {
        let (catalog, _dir) = setup_catalog();

        // Initially empty
        let tables = catalog.list_tables().unwrap();
        assert!(tables.is_empty());

        // Add some tables
        for name in ["users", "orders", "products"] {
            let schema = SchemaBuilder::new(name)
                .add_column("id", DataType::Text)
                .build()
                .unwrap();
            catalog.create_table(schema).unwrap();
        }

        let tables = catalog.list_tables().unwrap();
        assert_eq!(tables, vec!["orders", "products", "users"]);
    }

    #[test]
    fn test_update_table() {
        let (catalog, _dir) = setup_catalog();

        let schema = SchemaBuilder::new("users")
            .add_column("id", DataType::Text)
            .build()
            .unwrap();

        catalog.create_table(schema).unwrap();

        // Get and modify
        let mut schema = catalog.get_table("users").unwrap();
        schema
            .add_column(crate::catalog::types::ColumnDef::new(
                "email",
                DataType::Text,
            ))
            .unwrap();

        catalog.update_table(schema).unwrap();

        // Verify
        let updated = catalog.get_table("users").unwrap();
        assert_eq!(updated.version, 2);
        assert_eq!(updated.columns.len(), 2);
    }
}
