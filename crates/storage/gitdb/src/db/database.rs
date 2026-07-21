//! Database API - high-level interface for GitDB.

use std::path::{Path, PathBuf};
use std::rc::Rc;

use anyhow::bail;
use parking_lot::RwLock;

use crate::catalog::Catalog;
use crate::executor::{QueryExecutor, QueryResult};
use crate::planner::QueryPlanner;
use crate::sql::{Parser, Statement};
use crate::storage::{GitRepository, is_not_found};
use crate::transaction::{Transaction, TransactionManager, TxActive};

/// Database configuration options.
#[derive(Clone, Debug)]
pub struct DatabaseConfig {
    /// Path to the database directory.
    pub path: PathBuf,
    /// Create if doesn't exist.
    pub create_if_missing: bool,
    /// Enable query planning/optimization.
    pub enable_planner: bool,
    /// Enable verbose logging.
    pub verbose: bool,
    /// Auto-commit mode (commit after each statement).
    pub auto_commit: bool,
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        DatabaseConfig {
            path: PathBuf::from(".gitdb"),
            create_if_missing: true,
            enable_planner: true,
            verbose: false,
            auto_commit: true,
        }
    }
}

impl DatabaseConfig {
    /// Create a new configuration with the given path.
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self {
            path: path.into(),
            ..Default::default()
        }
    }

    /// Set create_if_missing flag.
    pub fn create_if_missing(mut self, value: bool) -> Self {
        self.create_if_missing = value;
        self
    }

    /// Set verbose flag.
    pub fn verbose(mut self, value: bool) -> Self {
        self.verbose = value;
        self
    }

    /// Set auto_commit flag.
    pub fn auto_commit(mut self, value: bool) -> Self {
        self.auto_commit = value;
        self
    }
}

/// The main database handle.
pub struct Database {
    config: DatabaseConfig,
    repo: Rc<RwLock<GitRepository>>,
    executor: QueryExecutor,
    planner: Option<QueryPlanner>,
    catalog: Catalog,
    tx_manager: TransactionManager,
}

impl Database {
    /// Open or create a database at the given path.
    pub fn open(path: impl AsRef<Path>) -> anyhow::Result<Self> {
        Self::open_with_config(DatabaseConfig::new(path.as_ref()))
    }

    /// Open or create a database with custom configuration.
    pub fn open_with_config(config: DatabaseConfig) -> anyhow::Result<Self> {
        let repo = if config.create_if_missing {
            GitRepository::open_or_init(&config.path)?
        } else if config.path.exists() {
            GitRepository::open(&config.path)?
        } else {
            bail!("database not found: {}", config.path.display());
        };

        let shared_repo = Rc::new(RwLock::new(repo.clone()));
        let executor = QueryExecutor::new(repo.clone());
        let catalog = Catalog::new(shared_repo.clone());
        let tx_manager = TransactionManager::new(repo);

        let planner = if config.enable_planner {
            Some(QueryPlanner::new(shared_repo.clone()))
        } else {
            None
        };

        Ok(Self {
            config,
            repo: shared_repo,
            executor,
            planner,
            catalog,
            tx_manager,
        })
    }

    /// Create a new in-memory database (for testing).
    pub fn in_memory() -> anyhow::Result<Self> {
        let dir = tempfile::TempDir::new()?;
        Self::open(dir.path())
    }

    /// Execute a SQL query string.
    pub fn execute(&mut self, sql: &str) -> anyhow::Result<QueryResult> {
        if self.config.verbose {
            eprintln!("[SQL] {}", sql);
        }

        // Parse and execute.
        let result = self.executor.execute(sql)?;

        if self.config.verbose {
            eprintln!("[Result] {:?}", result);
        }

        Ok(result)
    }

    /// Execute multiple SQL statements separated by semicolons.
    pub fn execute_batch(&mut self, sql: &str) -> anyhow::Result<Vec<QueryResult>> {
        let mut results = Vec::new();

        for stmt in sql.split(';') {
            let stmt = stmt.trim();
            if stmt.is_empty() {
                continue;
            }
            results.push(self.execute(stmt)?);
        }

        Ok(results)
    }

    /// Parse a SQL statement without executing.
    pub fn parse(&self, sql: &str) -> anyhow::Result<Statement> {
        Parser::parse(sql)
    }

    /// Explain a query (show the execution plan).
    pub fn explain(&self, sql: &str) -> anyhow::Result<String> {
        let stmt = Parser::parse(sql)?;

        if let Some(ref planner) = self.planner {
            Ok(planner.explain(&stmt)?)
        } else {
            Ok(format!("Statement: {:?}", stmt))
        }
    }

    /// Get database statistics.
    pub fn stats(&self) -> DatabaseStats {
        let repo = self.repo.read();
        let tables = self.catalog.list_tables().unwrap_or_default();

        let total_rows = if let Ok(head) = repo.head() {
            repo.stats(head).map(|s| s.total_rows).unwrap_or(0)
        } else {
            0
        };

        DatabaseStats {
            tables: tables.len(),
            total_rows,
            total_size_bytes: 0, // Not tracked currently.
            active_transactions: self.tx_manager.active_count(),
        }
    }

    /// List all tables.
    pub fn tables(&self) -> anyhow::Result<Vec<String>> {
        self.catalog.list_tables()
    }

    /// Check if a table exists.
    pub fn table_exists(&self, name: &str) -> bool {
        self.catalog.table_exists(name)
    }

    /// Get the schema for a table.
    pub fn table_schema(&self, name: &str) -> anyhow::Result<Option<crate::catalog::TableSchema>> {
        match self.catalog.get_table(name) {
            Ok(schema) => Ok(Some(schema)),
            Err(error) if is_not_found(&error) => Ok(None),
            Err(error) => Err(error),
        }
    }

    /// Begin a new transaction.
    pub fn begin(&mut self) -> anyhow::Result<Transaction<TxActive>> {
        self.tx_manager.begin()
    }

    /// Execute within a transaction.
    pub fn transaction<F, T>(&mut self, f: F) -> anyhow::Result<T>
    where
        F: FnOnce(&mut Self) -> anyhow::Result<T>,
    {
        self.execute("BEGIN")?;
        match f(self) {
            Ok(result) => {
                self.execute("COMMIT")?;
                Ok(result)
            }
            Err(e) => {
                self.execute("ROLLBACK")?;
                Err(e)
            }
        }
    }

    /// Get the database path.
    pub fn path(&self) -> &Path {
        &self.config.path
    }

    /// Get the configuration.
    pub fn config(&self) -> &DatabaseConfig {
        &self.config
    }

    /// Get the version/commit history.
    pub fn history(&self, limit: Option<usize>) -> anyhow::Result<Vec<CommitInfo>> {
        let repo = self.repo.read();
        let head = repo.head()?;
        let commits_result = repo.history(head, limit);

        match commits_result {
            Ok(commits) => Ok(commits
                .into_iter()
                .map(|c| CommitInfo {
                    id: c.id.to_string(),
                    message: c.message,
                    timestamp: c.timestamp.timestamp(),
                })
                .collect()),
            Err(e) => Err(e),
        }
    }

    /// Create a backup/snapshot at current state.
    pub fn snapshot(&self, _message: &str) -> anyhow::Result<String> {
        let repo = self.repo.read();
        let head = repo.head()?;
        Ok(head.to_string())
    }
}

/// Database statistics.
#[derive(Clone, Debug)]
pub struct DatabaseStats {
    /// Number of tables.
    pub tables: usize,
    /// Total number of rows across all tables.
    pub total_rows: usize,
    /// Total size in bytes.
    pub total_size_bytes: usize,
    /// Number of active transactions.
    pub active_transactions: usize,
}

/// Information about a commit in history.
#[derive(Clone, Debug)]
pub struct CommitInfo {
    /// Commit ID (SHA).
    pub id: String,
    /// Commit message.
    pub message: String,
    /// Unix timestamp.
    pub timestamp: i64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_open_database() {
        let dir = tempfile::TempDir::new().unwrap();
        let db = Database::open(dir.path()).unwrap();
        assert!(db.tables().unwrap().is_empty());
    }

    #[test]
    fn test_create_table() {
        let dir = tempfile::TempDir::new().unwrap();
        let mut db = Database::open(dir.path()).unwrap();

        db.execute("CREATE TABLE users (id TEXT PRIMARY KEY, name TEXT)")
            .unwrap();
        assert!(db.table_exists("users"));
        assert_eq!(db.tables().unwrap().len(), 1);
    }

    #[test]
    fn test_insert_and_select() {
        let dir = tempfile::TempDir::new().unwrap();
        let mut db = Database::open(dir.path()).unwrap();

        db.execute("CREATE TABLE users (id TEXT PRIMARY KEY, name TEXT)")
            .unwrap();
        db.execute("INSERT INTO users (id, name) VALUES ('1', 'Alice')")
            .unwrap();

        let result = db.execute("SELECT * FROM users").unwrap();
        if let QueryResult::Select(rs) = result {
            assert_eq!(rs.len(), 1);
        } else {
            panic!("Expected Select result");
        }
    }

    #[test]
    fn test_execute_batch() {
        let dir = tempfile::TempDir::new().unwrap();
        let mut db = Database::open(dir.path()).unwrap();

        let results = db
            .execute_batch(
                r#"
            CREATE TABLE users (id TEXT PRIMARY KEY, name TEXT);
            INSERT INTO users (id, name) VALUES ('1', 'Alice');
            INSERT INTO users (id, name) VALUES ('2', 'Bob');
            SELECT * FROM users
        "#,
            )
            .unwrap();

        assert_eq!(results.len(), 4);
    }

    #[test]
    fn test_stats() {
        let dir = tempfile::TempDir::new().unwrap();
        let mut db = Database::open(dir.path()).unwrap();

        db.execute("CREATE TABLE users (id TEXT PRIMARY KEY)")
            .unwrap();
        db.execute("INSERT INTO users (id) VALUES ('1')").unwrap();

        let stats = db.stats();
        assert_eq!(stats.tables, 1);
    }

    #[test]
    fn test_explain() {
        let dir = tempfile::TempDir::new().unwrap();
        let mut db = Database::open(dir.path()).unwrap();

        db.execute("CREATE TABLE users (id TEXT PRIMARY KEY, name TEXT)")
            .unwrap();

        let plan = db.explain("SELECT * FROM users WHERE id = '1'").unwrap();
        assert!(plan.contains("Plan"));
    }
}
