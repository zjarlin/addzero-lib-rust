//! Main query executor.

use std::collections::BTreeMap;
use std::sync::Arc;

use parking_lot::RwLock;
use serde_json::Value;

use super::error::{ExecuteError, ExecuteResult};
use super::eval::evaluate;
use super::operators::{
    FilterOperator, LimitOperator, Operator, ProjectOperator, Row, ScanOperator, SortOperator,
};
use super::result::{QueryResult, ResultSet};
use crate::catalog::{Catalog, ColumnDef, Constraint, DataType, SchemaBuilder};
use crate::sql::{
    Assignment, CreateTable, Delete, DropTable, Insert, Parser, Select, SelectColumn, SqlDataType,
    Statement, Update,
};
use crate::storage::{GitRepository, Row as StorageRow, RowKey, TableName};
use crate::transaction::{Transaction, TransactionManager, TxActive};

/// The query executor.
pub struct QueryExecutor {
    repo: Arc<RwLock<GitRepository>>,
    catalog: Catalog,
    tx_manager: TransactionManager,
    current_tx: Option<Transaction<TxActive>>,
}

impl QueryExecutor {
    /// Create a new executor.
    pub fn new(repo: GitRepository) -> Self {
        let shared_repo = Arc::new(RwLock::new(repo.clone()));
        let catalog = Catalog::new(shared_repo.clone());
        let tx_manager = TransactionManager::new(repo);
        Self {
            repo: shared_repo,
            catalog,
            tx_manager,
            current_tx: None,
        }
    }

    /// Execute a SQL string.
    pub fn execute(&mut self, sql: &str) -> ExecuteResult<QueryResult> {
        let stmt = Parser::parse(sql)?;
        self.execute_statement(stmt)
    }

    /// Execute a parsed statement.
    pub fn execute_statement(&mut self, stmt: Statement) -> ExecuteResult<QueryResult> {
        match stmt {
            Statement::CreateTable(ct) => self.execute_create_table(ct),
            Statement::DropTable(dt) => self.execute_drop_table(dt),
            Statement::Select(s) => self.execute_select(s),
            Statement::Insert(i) => self.execute_insert(i),
            Statement::Update(u) => self.execute_update(u),
            Statement::Delete(d) => self.execute_delete(d),
            Statement::Begin => self.execute_begin(),
            Statement::Commit => self.execute_commit(),
            Statement::Rollback => self.execute_rollback(),
            Statement::ShowTables => self.execute_show_tables(),
            Statement::Describe(table) => self.execute_describe(&table),
        }
    }

    fn execute_create_table(&mut self, ct: CreateTable) -> ExecuteResult<QueryResult> {
        // Check if already exists
        if self.catalog.table_exists(&ct.name) {
            if ct.if_not_exists {
                return Ok(QueryResult::success(format!(
                    "Table '{}' already exists",
                    ct.name
                )));
            }
            return Err(ExecuteError::TableNotFound(format!(
                "Table '{}' already exists",
                ct.name
            )));
        }

        // Convert SQL column defs to catalog column defs
        let mut builder = SchemaBuilder::new(&ct.name);
        for col in ct.columns {
            let data_type = convert_sql_type(&col.data_type);
            let mut col_def = ColumnDef::new(&col.name, data_type);

            for constraint in col.constraints {
                let c = match constraint {
                    crate::sql::ColumnConstraint::NotNull => Constraint::NotNull,
                    crate::sql::ColumnConstraint::Unique => Constraint::Unique,
                    crate::sql::ColumnConstraint::PrimaryKey => Constraint::PrimaryKey,
                    crate::sql::ColumnConstraint::Default(expr) => {
                        // Evaluate default expression
                        let empty_row = serde_json::Map::new();
                        let value = evaluate(&expr, &empty_row)?;
                        Constraint::Default(value)
                    }
                };
                col_def = col_def.with_constraint(c);
            }
            builder = builder.column(col_def);
        }

        let schema = builder.build().map_err(ExecuteError::Schema)?;
        self.catalog.create_table(schema)?;

        // Also create the actual table in storage
        let repo = self.repo.write();
        let head = repo.head()?;
        let table_name = TableName::new(&ct.name)?;
        let new_head = repo.create_table(&table_name, head, None)?;
        repo.update_branch(&crate::storage::BranchName::main(), new_head)?;

        Ok(QueryResult::success(format!("Created table '{}'", ct.name)))
    }

    fn execute_drop_table(&mut self, dt: DropTable) -> ExecuteResult<QueryResult> {
        if !self.catalog.table_exists(&dt.name) {
            if dt.if_exists {
                return Ok(QueryResult::success(format!(
                    "Table '{}' does not exist",
                    dt.name
                )));
            }
            return Err(ExecuteError::TableNotFound(dt.name));
        }

        self.catalog.drop_table(&dt.name)?;

        // Also drop from storage
        let repo = self.repo.write();
        let head = repo.head()?;
        let table_name = TableName::new(&dt.name)?;
        let new_head = repo.drop_table(&table_name, head, None)?;
        repo.update_branch(&crate::storage::BranchName::main(), new_head)?;

        Ok(QueryResult::success(format!("Dropped table '{}'", dt.name)))
    }

    fn execute_select(&self, select: Select) -> ExecuteResult<QueryResult> {
        // Get table rows
        let rows = self.scan_table(&select.from)?;

        // Build operator tree
        let mut op: Box<dyn Operator> = Box::new(ScanOperator::new(rows));

        // Apply WHERE
        if let Some(where_clause) = select.where_clause {
            op = Box::new(FilterOperator::new(op, where_clause));
        }

        // Apply ORDER BY
        if !select.order_by.is_empty() {
            op = Box::new(SortOperator::new(op, select.order_by));
        }

        // Apply LIMIT/OFFSET
        if select.limit.is_some() || select.offset.is_some() {
            let limit = select.limit.unwrap_or(usize::MAX);
            let offset = select.offset.unwrap_or(0);
            op = Box::new(LimitOperator::new(op, limit, offset));
        }

        // Apply projection
        if !select
            .columns
            .iter()
            .any(|c| matches!(c, SelectColumn::Wildcard))
        {
            op = Box::new(ProjectOperator::new(op, select.columns.clone()));
        }

        // Collect results
        let mut result_rows = Vec::new();
        while let Some(row) = op.next_row()? {
            result_rows.push(row);
        }

        // Determine columns
        let columns = if select
            .columns
            .iter()
            .any(|c| matches!(c, SelectColumn::Wildcard))
        {
            result_rows
                .first()
                .map(|r| r.keys().cloned().collect())
                .unwrap_or_default()
        } else {
            select
                .columns
                .iter()
                .filter_map(|c| match c {
                    SelectColumn::Column(name) => Some(name.clone()),
                    SelectColumn::Expr { alias, .. } => alias.clone(),
                    SelectColumn::Wildcard => None,
                })
                .collect()
        };

        Ok(QueryResult::Select(ResultSet {
            columns,
            rows: result_rows,
        }))
    }

    fn execute_insert(&mut self, insert: Insert) -> ExecuteResult<QueryResult> {
        let schema = self.catalog.get_table(&insert.table)?;
        let repo = self.repo.write();
        let mut head = repo.head()?;
        let table_name = TableName::new(&insert.table)?;

        let column_names = insert
            .columns
            .as_ref()
            .map(|c| c.clone())
            .unwrap_or_else(|| {
                schema
                    .column_names()
                    .into_iter()
                    .map(|s| s.to_string())
                    .collect()
            });

        let mut inserted = 0;
        for row_values in &insert.values {
            // Build row data
            let mut data = BTreeMap::new();
            let empty_row = serde_json::Map::new();

            for (i, expr) in row_values.iter().enumerate() {
                if i < column_names.len() {
                    let value = evaluate(expr, &empty_row)?;
                    data.insert(column_names[i].clone(), value);
                }
            }

            // Apply defaults
            let row_value =
                Value::Object(data.iter().map(|(k, v)| (k.clone(), v.clone())).collect());
            let with_defaults = schema.apply_defaults(&row_value)?;
            let data: BTreeMap<String, Value> = with_defaults
                .as_object()
                .map(|o| o.iter().map(|(k, v)| (k.clone(), v.clone())).collect())
                .unwrap_or_default();

            // Validate
            schema.validate_row(&Value::Object(
                data.iter().map(|(k, v)| (k.clone(), v.clone())).collect(),
            ))?;

            // Generate row key
            let key = if let Some(pk) = &schema.primary_key {
                let pk_value = data
                    .get(pk)
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| ExecuteError::MissingColumn(pk.clone()))?;
                RowKey::new(pk_value)?
            } else {
                RowKey::generate()
            };

            let storage_row = StorageRow::new(key, data);
            head = repo.insert_row(&table_name, storage_row, head, None)?;
            inserted += 1;
        }

        repo.update_branch(&crate::storage::BranchName::main(), head)?;
        Ok(QueryResult::modified(inserted))
    }

    fn execute_update(&mut self, update: Update) -> ExecuteResult<QueryResult> {
        let _schema = self.catalog.get_table(&update.table)?;
        let repo = self.repo.write();
        let mut head = repo.head()?;
        let table_name = TableName::new(&update.table)?;

        // Get all rows
        let rows = repo.scan_table(&table_name, head)?;
        let mut updated = 0;

        for storage_row in rows {
            // Check WHERE clause
            let row_map: serde_json::Map<String, Value> = storage_row
                .data
                .iter()
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect();

            let matches = if let Some(ref where_clause) = update.where_clause {
                super::eval::matches_where(where_clause, &row_map)?
            } else {
                true
            };

            if matches {
                // Apply updates
                let mut new_data = storage_row.data.clone();
                for Assignment { column, value } in &update.assignments {
                    let new_value = evaluate(value, &row_map)?;
                    new_data.insert(column.clone(), new_value);
                }

                let updated_row = storage_row.with_update(new_data);
                head = repo.update_row(&table_name, updated_row, head, None)?;
                updated += 1;
            }
        }

        repo.update_branch(&crate::storage::BranchName::main(), head)?;
        Ok(QueryResult::modified(updated))
    }

    fn execute_delete(&mut self, delete: Delete) -> ExecuteResult<QueryResult> {
        let repo = self.repo.write();
        let mut head = repo.head()?;
        let table_name = TableName::new(&delete.table)?;

        // Get all rows
        let rows = repo.scan_table(&table_name, head)?;
        let mut deleted = 0;

        for storage_row in rows {
            // Check WHERE clause
            let row_map: serde_json::Map<String, Value> = storage_row
                .data
                .iter()
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect();

            let matches = if let Some(ref where_clause) = delete.where_clause {
                super::eval::matches_where(where_clause, &row_map)?
            } else {
                true
            };

            if matches {
                head = repo.delete_row(&table_name, &storage_row.key, head, None)?;
                deleted += 1;
            }
        }

        repo.update_branch(&crate::storage::BranchName::main(), head)?;
        Ok(QueryResult::modified(deleted))
    }

    fn execute_begin(&mut self) -> ExecuteResult<QueryResult> {
        if self.current_tx.is_some() {
            return Err(ExecuteError::Internal("transaction already active".into()));
        }
        let tx = self.tx_manager.begin()?;
        self.current_tx = Some(tx);
        Ok(QueryResult::transaction("BEGIN"))
    }

    fn execute_commit(&mut self) -> ExecuteResult<QueryResult> {
        let tx = self.current_tx.take().ok_or(ExecuteError::NoTransaction)?;
        tx.commit()?;
        Ok(QueryResult::transaction("COMMIT"))
    }

    fn execute_rollback(&mut self) -> ExecuteResult<QueryResult> {
        let tx = self.current_tx.take().ok_or(ExecuteError::NoTransaction)?;
        tx.rollback()?;
        Ok(QueryResult::transaction("ROLLBACK"))
    }

    fn execute_show_tables(&self) -> ExecuteResult<QueryResult> {
        let tables = self.catalog.list_tables()?;
        let rows: Vec<Row> = tables
            .into_iter()
            .map(|name| {
                let mut row = Row::new();
                row.insert("table_name".into(), Value::String(name));
                row
            })
            .collect();

        Ok(QueryResult::Select(ResultSet {
            columns: vec!["table_name".into()],
            rows,
        }))
    }

    fn execute_describe(&self, table: &str) -> ExecuteResult<QueryResult> {
        let schema = self.catalog.get_table(table)?;
        let rows: Vec<Row> = schema
            .columns
            .iter()
            .map(|col| {
                let mut row = Row::new();
                row.insert("column".into(), Value::String(col.name.clone()));
                row.insert(
                    "type".into(),
                    Value::String(col.data_type.sql_name().to_string()),
                );
                row.insert("nullable".into(), Value::Bool(col.is_nullable()));
                row.insert(
                    "primary_key".into(),
                    Value::Bool(schema.primary_key.as_deref() == Some(&col.name)),
                );
                row
            })
            .collect();

        Ok(QueryResult::Select(ResultSet {
            columns: vec![
                "column".into(),
                "type".into(),
                "nullable".into(),
                "primary_key".into(),
            ],
            rows,
        }))
    }

    fn scan_table(&self, table: &str) -> ExecuteResult<Vec<Row>> {
        let repo = self.repo.read();
        let head = repo.head()?;
        let table_name = TableName::new(table)?;

        let storage_rows = repo.scan_table(&table_name, head)?;
        let rows: Vec<Row> = storage_rows.into_iter().map(|sr| sr.data).collect();

        Ok(rows)
    }

    /// Get the catalog.
    pub fn catalog(&self) -> &Catalog {
        &self.catalog
    }

    /// Check if in transaction.
    pub fn in_transaction(&self) -> bool {
        self.current_tx.is_some()
    }
}

fn convert_sql_type(sql_type: &SqlDataType) -> DataType {
    match sql_type {
        SqlDataType::Text => DataType::Text,
        SqlDataType::Integer => DataType::Integer,
        SqlDataType::Float => DataType::Float,
        SqlDataType::Boolean => DataType::Boolean,
        SqlDataType::Json => DataType::Json,
        SqlDataType::Timestamp => DataType::Timestamp,
        SqlDataType::Uuid => DataType::Uuid,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn setup() -> (QueryExecutor, TempDir) {
        let dir = TempDir::new().unwrap();
        let repo = GitRepository::open_or_init(dir.path()).unwrap();
        let executor = QueryExecutor::new(repo);
        (executor, dir)
    }

    #[test]
    fn test_create_table() {
        let (mut exec, _dir) = setup();

        let result = exec
            .execute("CREATE TABLE users (id TEXT PRIMARY KEY, name TEXT NOT NULL)")
            .unwrap();
        assert!(matches!(result, QueryResult::Success { .. }));

        // Verify in catalog
        assert!(exec.catalog().table_exists("users"));
    }

    #[test]
    fn test_insert_and_select() {
        let (mut exec, _dir) = setup();

        exec.execute("CREATE TABLE users (id TEXT PRIMARY KEY, name TEXT NOT NULL)")
            .unwrap();
        exec.execute("INSERT INTO users (id, name) VALUES ('1', 'Alice')")
            .unwrap();
        exec.execute("INSERT INTO users (id, name) VALUES ('2', 'Bob')")
            .unwrap();

        let result = exec.execute("SELECT * FROM users").unwrap();
        if let QueryResult::Select(rs) = result {
            assert_eq!(rs.len(), 2);
        } else {
            panic!("Expected Select result");
        }
    }

    #[test]
    fn test_select_where() {
        let (mut exec, _dir) = setup();

        exec.execute("CREATE TABLE users (id TEXT PRIMARY KEY, name TEXT, age INTEGER)")
            .unwrap();
        exec.execute("INSERT INTO users (id, name, age) VALUES ('1', 'Alice', 30)")
            .unwrap();
        exec.execute("INSERT INTO users (id, name, age) VALUES ('2', 'Bob', 25)")
            .unwrap();
        exec.execute("INSERT INTO users (id, name, age) VALUES ('3', 'Charlie', 35)")
            .unwrap();

        let result = exec.execute("SELECT * FROM users WHERE age > 28").unwrap();
        if let QueryResult::Select(rs) = result {
            assert_eq!(rs.len(), 2); // Alice and Charlie
        } else {
            panic!("Expected Select result");
        }
    }

    #[test]
    fn test_update() {
        let (mut exec, _dir) = setup();

        exec.execute("CREATE TABLE users (id TEXT PRIMARY KEY, name TEXT)")
            .unwrap();
        exec.execute("INSERT INTO users (id, name) VALUES ('1', 'Alice')")
            .unwrap();

        let result = exec
            .execute("UPDATE users SET name = 'Alicia' WHERE id = '1'")
            .unwrap();
        assert!(matches!(result, QueryResult::Modified { rows_affected: 1 }));

        let result = exec.execute("SELECT * FROM users WHERE id = '1'").unwrap();
        if let QueryResult::Select(rs) = result {
            assert_eq!(
                rs.rows[0].get("name").unwrap(),
                &Value::String("Alicia".into())
            );
        }
    }

    #[test]
    fn test_delete() {
        let (mut exec, _dir) = setup();

        exec.execute("CREATE TABLE users (id TEXT PRIMARY KEY, name TEXT)")
            .unwrap();
        exec.execute("INSERT INTO users (id, name) VALUES ('1', 'Alice')")
            .unwrap();
        exec.execute("INSERT INTO users (id, name) VALUES ('2', 'Bob')")
            .unwrap();

        let result = exec.execute("DELETE FROM users WHERE id = '1'").unwrap();
        assert!(matches!(result, QueryResult::Modified { rows_affected: 1 }));

        let result = exec.execute("SELECT * FROM users").unwrap();
        if let QueryResult::Select(rs) = result {
            assert_eq!(rs.len(), 1);
        }
    }

    #[test]
    fn test_show_tables() {
        let (mut exec, _dir) = setup();

        exec.execute("CREATE TABLE users (id TEXT)").unwrap();
        exec.execute("CREATE TABLE orders (id TEXT)").unwrap();

        let result = exec.execute("SHOW TABLES").unwrap();
        if let QueryResult::Select(rs) = result {
            assert_eq!(rs.len(), 2);
        }
    }

    #[test]
    fn test_order_by_limit() {
        let (mut exec, _dir) = setup();

        exec.execute("CREATE TABLE users (id TEXT PRIMARY KEY, name TEXT, age INTEGER)")
            .unwrap();
        exec.execute("INSERT INTO users (id, name, age) VALUES ('1', 'Alice', 30)")
            .unwrap();
        exec.execute("INSERT INTO users (id, name, age) VALUES ('2', 'Bob', 25)")
            .unwrap();
        exec.execute("INSERT INTO users (id, name, age) VALUES ('3', 'Charlie', 35)")
            .unwrap();

        let result = exec
            .execute("SELECT * FROM users ORDER BY age DESC LIMIT 2")
            .unwrap();
        if let QueryResult::Select(rs) = result {
            assert_eq!(rs.len(), 2);
            // Should be Charlie (35) then Alice (30)
            assert_eq!(
                rs.rows[0].get("name").unwrap(),
                &Value::String("Charlie".into())
            );
            assert_eq!(
                rs.rows[1].get("name").unwrap(),
                &Value::String("Alice".into())
            );
        }
    }
}
