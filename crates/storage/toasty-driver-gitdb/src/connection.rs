use crate::capability::CAPABILITY_GITDB;
use crate::error::GitDbDriverError;
use crate::sql::inline_indexed_params;
use crate::value::from_json_value;
use async_trait::async_trait;
use gitdb::db::{Database, DatabaseConfig};
use gitdb::executor::QueryResult;
use std::borrow::Cow;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::mpsc::{self, Sender};
use std::thread;
use toasty_core::driver::operation::{Insert, IsolationLevel, Operation, QuerySql, Transaction};
use toasty_core::driver::{Capability, Driver, ExecResponse};
use toasty_core::schema::db::{self, AppliedMigration, Migration, SchemaDiff};
use toasty_core::{Connection, Result, Schema, stmt};
use toasty_sql as sql;

const MIGRATIONS_TABLE: &str = "__toasty_migrations";

/// Toasty driver for a filesystem-backed `gitdb` database.
#[derive(Debug, Clone)]
pub struct GitDb {
    path: PathBuf,
    create_if_missing: bool,
}

impl GitDb {
    /// Opens or creates a gitdb-backed Toasty driver rooted at `path`.
    pub fn open(path: impl AsRef<Path>) -> Self {
        Self {
            path: path.as_ref().to_path_buf(),
            create_if_missing: true,
        }
    }

    /// Creates a driver from an existing `gitdb:` URL or local path-like URL.
    pub fn new(url: impl Into<String>) -> Result<Self> {
        let url = url.into();
        let path = if let Some(path) = url.strip_prefix("gitdb://") {
            PathBuf::from(path)
        } else if let Some(path) = url.strip_prefix("gitdb:") {
            PathBuf::from(path)
        } else {
            return Err(toasty_core::Error::invalid_connection_url(format!(
                "connection URL does not have a `gitdb:` scheme; url={url}"
            )));
        };

        Ok(Self::open(path))
    }

    fn database_config(&self) -> DatabaseConfig {
        DatabaseConfig::new(&self.path).create_if_missing(self.create_if_missing)
    }
}

#[async_trait]
impl Driver for GitDb {
    fn url(&self) -> Cow<'_, str> {
        Cow::Owned(format!("gitdb:{}", self.path.display()))
    }

    fn capability(&self) -> &'static Capability {
        &CAPABILITY_GITDB
    }

    async fn connect(&self) -> Result<Box<dyn Connection>> {
        Ok(Box::new(GitDbConnection::spawn(self.database_config())?))
    }

    fn max_connections(&self) -> Option<usize> {
        None
    }

    fn generate_migration(&self, schema_diff: &SchemaDiff<'_>) -> Migration {
        let statements = sql::MigrationStatement::from_diff(schema_diff, &CAPABILITY_GITDB);
        let sql_strings: Vec<String> = statements
            .iter()
            .map(|stmt| sql::Serializer::sqlite(stmt.schema()).serialize(stmt.statement()))
            .collect();
        Migration::new_sql_with_breakpoints(&sql_strings)
    }

    async fn reset_db(&self) -> Result<()> {
        if self.path.exists() {
            std::fs::remove_dir_all(&self.path)
                .map_err(toasty_core::Error::driver_operation_failed)?;
        }
        Ok(())
    }
}

#[derive(Debug)]
pub struct GitDbConnection {
    request_tx: Sender<WorkerRequest>,
}

impl GitDbConnection {
    fn spawn(config: DatabaseConfig) -> Result<Self> {
        let (request_tx, request_rx) = mpsc::channel::<WorkerRequest>();
        let (ready_tx, ready_rx) = mpsc::channel();

        thread::Builder::new()
            .name("toasty-gitdb-connection".into())
            .spawn(move || {
                let mut database = match Database::open_with_config(config) {
                    Ok(database) => {
                        let _ = ready_tx.send(Ok(()));
                        database
                    }
                    Err(err) => {
                        let _ = ready_tx.send(Err(map_driver_error(err)));
                        return;
                    }
                };

                while let Ok(request) = request_rx.recv() {
                    match request {
                        WorkerRequest::Execute { sql, response_tx } => {
                            let response = database.execute(&sql).map(convert_query_result);
                            let _ = response_tx.send(response.map_err(map_driver_error));
                        }
                        WorkerRequest::Shutdown => break,
                    }
                }
            })
            .map_err(toasty_core::Error::driver_operation_failed)?;

        ready_rx
            .recv()
            .map_err(toasty_core::Error::driver_operation_failed)??;

        Ok(Self { request_tx })
    }

    fn execute_sql(&self, sql: String) -> Result<WorkerQueryResult> {
        let (response_tx, response_rx) = mpsc::channel();
        self.request_tx
            .send(WorkerRequest::Execute { sql, response_tx })
            .map_err(toasty_core::Error::connection_lost)?;
        response_rx
            .recv()
            .map_err(toasty_core::Error::connection_lost)?
    }

    fn exec_query_sql(&self, schema: &db::Schema, op: QuerySql) -> Result<ExecResponse> {
        if op.last_insert_id_hack.is_some() {
            return Err(toasty_core::Error::unsupported_feature(
                "gitdb does not support MySQL last_insert_id_hack semantics",
            ));
        }

        let sql = lower_sql_statement(schema, op.stmt, op.params)?;
        let result = self.execute_sql(sql)?;
        map_worker_result(result, op.ret.as_deref())
    }

    fn exec_insert(&self, schema: &db::Schema, op: Insert) -> Result<ExecResponse> {
        if op.ret.is_some() {
            return Err(toasty_core::Error::unsupported_feature(
                "gitdb does not support RETURNING on insert mutations",
            ));
        }

        let sql = lower_sql_statement(schema, op.stmt, op.params)?;
        let result = self.execute_sql(sql)?;
        map_worker_result(result, None)
    }

    fn exec_transaction(&self, schema: &db::Schema, op: Transaction) -> Result<ExecResponse> {
        if let Transaction::Start {
            isolation,
            read_only,
        } = &op
        {
            if *read_only {
                return Err(toasty_core::Error::unsupported_feature(
                    "gitdb transactions do not support read_only mode",
                ));
            }
            if !matches!(isolation, Some(IsolationLevel::Serializable) | None) {
                return Err(toasty_core::Error::unsupported_feature(
                    "gitdb only supports default/serializable transaction semantics",
                ));
            }
        }

        let sql = sql::Serializer::sqlite(schema).serialize_transaction(&op);
        let result = self.execute_sql(sql.trim_end_matches(';').to_string())?;
        map_worker_result(result, None)
    }

    fn ensure_migrations_table(&self) -> Result<()> {
        self.execute_sql(format!(
            "CREATE TABLE IF NOT EXISTS {MIGRATIONS_TABLE} (id INTEGER PRIMARY KEY, name TEXT NOT NULL, applied_at TEXT NOT NULL)"
        ))?;
        Ok(())
    }

    fn create_table_if_missing(&self, table: &db::Table) -> Result<()> {
        self.execute_sql(create_table_sql(table)?)?;
        Ok(())
    }
}

impl Drop for GitDbConnection {
    fn drop(&mut self) {
        let _ = self.request_tx.send(WorkerRequest::Shutdown);
    }
}

#[async_trait]
impl Connection for GitDbConnection {
    async fn exec(&mut self, schema: &Arc<Schema>, op: Operation) -> Result<ExecResponse> {
        match op {
            Operation::QuerySql(op) => self.exec_query_sql(&schema.db, op),
            Operation::Insert(op) => self.exec_insert(&schema.db, op),
            Operation::Transaction(op) => self.exec_transaction(&schema.db, op),
            unsupported => Err(toasty_core::Error::unsupported_feature(format!(
                "gitdb driver does not support operation `{}` yet",
                unsupported.name()
            ))),
        }
    }

    async fn push_schema(&mut self, schema: &Schema) -> Result<()> {
        for table in &schema.db.tables {
            self.create_table_if_missing(table)?;
        }
        Ok(())
    }

    async fn applied_migrations(&mut self) -> Result<Vec<AppliedMigration>> {
        self.ensure_migrations_table()?;
        let result = self.execute_sql(format!(
            "SELECT id FROM {MIGRATIONS_TABLE} ORDER BY id ASC"
        ))?;

        let WorkerQueryResult::Rows { rows, .. } = result else {
            return Err(toasty_core::Error::invalid_result(
                "expected migration rows from gitdb",
            ));
        };

        rows.into_iter()
            .map(|row| {
                let id = row
                    .get("id")
                    .and_then(serde_json::Value::as_u64)
                    .or_else(|| {
                        row.get("id")
                            .and_then(serde_json::Value::as_i64)
                            .map(|v| v as u64)
                    })
                    .ok_or_else(|| {
                        toasty_core::Error::invalid_result(
                            "expected numeric migration id from gitdb",
                        )
                    })?;
                Ok(AppliedMigration::new(id))
            })
            .collect()
    }

    async fn apply_migration(&mut self, id: u64, name: &str, migration: &Migration) -> Result<()> {
        self.ensure_migrations_table()?;

        for statement in migration.statements() {
            if let Err(err) = self.execute_sql(statement.to_string()) {
                return Err(err);
            }
        }

        let applied_at = chrono::Utc::now().to_rfc3339();
        let insert = format!(
            "INSERT INTO {MIGRATIONS_TABLE} (id, name, applied_at) VALUES ({id}, {}, {})",
            quote_sql(name),
            quote_sql(&applied_at)
        );
        self.execute_sql(insert)?;
        Ok(())
    }
}

#[derive(Debug)]
enum WorkerRequest {
    Execute {
        sql: String,
        response_tx: Sender<Result<WorkerQueryResult>>,
    },
    Shutdown,
}

#[derive(Debug)]
enum WorkerQueryResult {
    Count(u64),
    Rows {
        columns: Vec<String>,
        rows: Vec<BTreeMap<String, serde_json::Value>>,
    },
}

fn convert_query_result(result: QueryResult) -> WorkerQueryResult {
    match result {
        QueryResult::Modified { rows_affected } => WorkerQueryResult::Count(rows_affected as u64),
        QueryResult::Success { .. } | QueryResult::Transaction { .. } => WorkerQueryResult::Count(0),
        QueryResult::Select(result_set) => WorkerQueryResult::Rows {
            columns: result_set.columns,
            rows: result_set.rows,
        },
    }
}

fn lower_sql_statement(
    schema: &db::Schema,
    statement: stmt::Statement,
    params: Vec<toasty_core::driver::operation::TypedValue>,
) -> Result<String> {
    let sql = sql::Serializer::sqlite(schema).serialize(&sql::Statement::from(statement));
    inline_indexed_params(&sql, &params).map_err(map_local_error)
}

fn map_worker_result(result: WorkerQueryResult, ret: Option<&[stmt::Type]>) -> Result<ExecResponse> {
    match result {
        WorkerQueryResult::Count(count) => Ok(ExecResponse::count(count)),
        WorkerQueryResult::Rows { columns, rows } => {
            let Some(ret_tys) = ret else {
                return Err(toasty_core::Error::invalid_result(
                    "gitdb returned rows for a Toasty statement without a return type",
                ));
            };

            if columns.len() != ret_tys.len() {
                return Err(toasty_core::Error::invalid_result(format!(
                    "gitdb result width {} does not match Toasty return width {}",
                    columns.len(),
                    ret_tys.len()
                )));
            }

            let values = rows
                .into_iter()
                .map(|row| row_to_value_record(row, &columns, ret_tys))
                .collect::<Result<Vec<_>>>()?;

            Ok(ExecResponse::value_stream(stmt::ValueStream::from_vec(values)))
        }
    }
}

fn create_table_sql(table: &db::Table) -> Result<String> {
    let mut parts = Vec::with_capacity(table.columns.len());
    for column in &table.columns {
        parts.push(column_definition_sql(column)?);
    }

    Ok(format!(
        "CREATE TABLE IF NOT EXISTS {} ({})",
        table.name,
        parts.join(", ")
    ))
}

fn column_definition_sql(column: &db::Column) -> Result<String> {
    let mut sql = format!("{} {}", column.name, storage_type_sql(&column.storage_ty)?);
    if column.primary_key {
        sql.push_str(" PRIMARY KEY");
    }
    if !column.nullable {
        sql.push_str(" NOT NULL");
    }
    if column.auto_increment {
        return Err(toasty_core::Error::unsupported_feature(format!(
            "gitdb does not support auto_increment column `{}`",
            column.name
        )));
    }
    Ok(sql)
}

fn storage_type_sql(ty: &db::Type) -> Result<&'static str> {
    match ty {
        db::Type::Boolean => Ok("BOOLEAN"),
        db::Type::Integer(_) | db::Type::UnsignedInteger(_) => Ok("INTEGER"),
        db::Type::Float(_) | db::Type::Numeric(_) => Ok("REAL"),
        db::Type::Text | db::Type::VarChar(_) | db::Type::Enum(_) => Ok("TEXT"),
        db::Type::Uuid => Ok("UUID"),
        db::Type::Timestamp(_) | db::Type::Date | db::Type::Time(_) | db::Type::DateTime(_) => {
            Ok("TIMESTAMP")
        }
        db::Type::List(_) => Ok("JSON"),
        db::Type::Blob | db::Type::Binary(_) | db::Type::Custom(_) => Err(
            toasty_core::Error::unsupported_feature(format!(
                "gitdb schema push does not support storage type `{ty:?}`"
            )),
        ),
    }
}

fn row_to_value_record(
    row: BTreeMap<String, serde_json::Value>,
    columns: &[String],
    ret_tys: &[stmt::Type],
) -> Result<stmt::Value> {
    let mut fields = Vec::with_capacity(columns.len());
    for (column_name, ty) in columns.iter().zip(ret_tys.iter()) {
        let value = row
            .get(column_name)
            .cloned()
            .unwrap_or(serde_json::Value::Null);
        fields.push(from_json_value(value, ty).map_err(map_local_error)?);
    }

    Ok(stmt::ValueRecord::from_vec(fields).into())
}

fn map_driver_error(error: impl std::error::Error + Send + Sync + 'static) -> toasty_core::Error {
    toasty_core::Error::driver_operation_failed(error)
}

fn map_local_error(error: GitDbDriverError) -> toasty_core::Error {
    match error {
        GitDbDriverError::UnsupportedValue(message) => toasty_core::Error::unsupported_feature(message),
        GitDbDriverError::InvalidResult(message) => toasty_core::Error::invalid_result(message),
    }
}

fn quote_sql(value: &str) -> String {
    format!("'{}'", value.replace('\'', "''"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use toasty_core::schema::db::{Column, ColumnId, IndexId, PrimaryKey, Table, TableId, Type};

    #[tokio::test]
    async fn apply_and_list_migrations() {
        let dir = TempDir::new().unwrap();
        let driver = GitDb::open(dir.path());
        let mut conn = driver.connect().await.unwrap();

        conn.apply_migration(
            1,
            "create_users",
            &Migration::new_sql("CREATE TABLE IF NOT EXISTS users (id TEXT PRIMARY KEY)".into()),
        )
        .await
        .unwrap();

        let migrations = conn.applied_migrations().await.unwrap();
        assert_eq!(migrations.len(), 1);
        assert_eq!(migrations[0].id(), 1);
    }

    #[test]
    fn create_table_sql_maps_basic_types() {
        let table = Table {
            id: TableId(0),
            name: "users".into(),
            columns: vec![
                Column {
                    id: ColumnId {
                        table: TableId(0),
                        index: 0,
                    },
                    name: "id".into(),
                    ty: stmt::Type::String,
                    storage_ty: Type::Text,
                    nullable: false,
                    primary_key: true,
                    auto_increment: false,
                    versionable: false,
                },
                Column {
                    id: ColumnId {
                        table: TableId(0),
                        index: 1,
                    },
                    name: "age".into(),
                    ty: stmt::Type::I64,
                    storage_ty: Type::Integer(8),
                    nullable: true,
                    primary_key: false,
                    auto_increment: false,
                    versionable: false,
                },
            ],
            primary_key: PrimaryKey {
                columns: vec![ColumnId {
                    table: TableId(0),
                    index: 0,
                }],
                index: IndexId {
                    table: TableId(0),
                    index: 0,
                },
            },
            indices: vec![],
        };

        assert_eq!(
            create_table_sql(&table).unwrap(),
            "CREATE TABLE IF NOT EXISTS users (id TEXT PRIMARY KEY NOT NULL, age INTEGER)"
        );
    }

    #[test]
    fn worker_result_maps_rows() {
        let result = WorkerQueryResult::Rows {
            columns: vec!["id".into(), "age".into()],
            rows: vec![BTreeMap::from([
                ("id".into(), serde_json::Value::String("u1".into())),
                ("age".into(), serde_json::Value::from(7)),
            ])],
        };

        let response =
            map_worker_result(result, Some(&[stmt::Type::String, stmt::Type::I64])).unwrap();
        let mut stream = response.values.into_value_stream();
        let row = tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(stream.next())
            .unwrap()
            .unwrap();
        let record = row.as_record_unwrap();
        assert_eq!(record[0], stmt::Value::String("u1".into()));
        assert_eq!(record[1], stmt::Value::I64(7));
    }
}
