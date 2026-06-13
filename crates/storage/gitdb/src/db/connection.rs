//! Connection pooling for database access.

use std::collections::VecDeque;
use std::sync::Arc;

use anyhow::bail;
use parking_lot::{Mutex, RwLock};

use super::api::DatabaseConfig;
use crate::executor::QueryExecutor;
use crate::storage::GitRepository;

/// A database connection from the pool.
pub struct Connection {
    id: usize,
    executor: QueryExecutor,
    pool: Option<Arc<ConnectionPoolInner>>,
}

impl Connection {
    /// Create a standalone connection (not from a pool).
    pub fn new(repo: GitRepository) -> Self {
        Self {
            id: 0,
            executor: QueryExecutor::new(repo),
            pool: None,
        }
    }

    /// Execute a SQL query.
    pub fn execute(&mut self, sql: &str) -> anyhow::Result<crate::executor::QueryResult> {
        Ok(self.executor.execute(sql)?)
    }
}

impl Drop for Connection {
    fn drop(&mut self) {
        // Return connection to pool if pooled.
        if let Some(ref pool) = self.pool {
            let mut available = pool.available.lock();
            available.push_back(self.id);
        }
    }
}

struct ConnectionPoolInner {
    config: DatabaseConfig,
    repo: Arc<RwLock<GitRepository>>,
    available: Mutex<VecDeque<usize>>,
    max_connections: usize,
    created: Mutex<usize>,
}

/// Connection pool for database access.
pub struct ConnectionPool {
    inner: Arc<ConnectionPoolInner>,
}

impl ConnectionPool {
    /// Create a new connection pool.
    pub fn new(config: DatabaseConfig, max_connections: usize) -> anyhow::Result<Self> {
        let repo = if config.create_if_missing {
            GitRepository::open_or_init(&config.path)?
        } else {
            GitRepository::open(&config.path)?
        };

        let inner = Arc::new(ConnectionPoolInner {
            config,
            repo: Arc::new(RwLock::new(repo)),
            available: Mutex::new(VecDeque::new()),
            max_connections,
            created: Mutex::new(0),
        });

        Ok(Self { inner })
    }

    /// Get a connection from the pool.
    pub fn get(&self) -> anyhow::Result<Connection> {
        // Try to get an available connection.
        {
            let mut available = self.inner.available.lock();
            if let Some(id) = available.pop_front() {
                let repo = self.inner.repo.read().clone();
                return Ok(Connection {
                    id,
                    executor: QueryExecutor::new(repo),
                    pool: Some(self.inner.clone()),
                });
            }
        }

        // Create a new connection if under limit.
        {
            let mut created = self.inner.created.lock();
            if *created < self.inner.max_connections {
                *created += 1;
                let id = *created;
                let repo = self.inner.repo.read().clone();
                return Ok(Connection {
                    id,
                    executor: QueryExecutor::new(repo),
                    pool: Some(self.inner.clone()),
                });
            }
        }

        // Pool exhausted - in production we'd wait, but for now error.
        bail!("invalid configuration: connection pool exhausted")
    }

    /// Get the number of available connections.
    pub fn available(&self) -> usize {
        self.inner.available.lock().len()
    }

    /// Get the total number of connections created.
    pub fn created(&self) -> usize {
        *self.inner.created.lock()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_connection_pool() {
        let dir = tempfile::TempDir::new().unwrap();
        let config = DatabaseConfig::new(dir.path());
        let pool = ConnectionPool::new(config, 5).unwrap();

        let mut conn = pool.get().unwrap();
        conn.execute("CREATE TABLE test (id TEXT)").unwrap();

        assert_eq!(pool.created(), 1);
    }

    #[test]
    fn test_pool_reuse() {
        let dir = tempfile::TempDir::new().unwrap();
        let config = DatabaseConfig::new(dir.path());
        let pool = ConnectionPool::new(config, 5).unwrap();

        {
            let _conn1 = pool.get().unwrap();
            let _conn2 = pool.get().unwrap();
            assert_eq!(pool.created(), 2);
        }

        // Connections returned to pool.
        assert_eq!(pool.available(), 2);

        // Reuse existing connection.
        let _conn3 = pool.get().unwrap();
        assert_eq!(pool.created(), 2); // No new connection created.
    }
}
