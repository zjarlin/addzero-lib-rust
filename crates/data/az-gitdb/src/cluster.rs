use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use gitdb::db::{Connection, ConnectionPool, DatabaseConfig, DatabaseError};
use gitdb::executor::QueryResult;

use crate::classify::{GitDbQueryKind, classify_gitdb_query};
use crate::config::{GitDbClusterConfig, GitDbLoadBalanceStrategy, GitDbNodeConfig, GitDbNodeRole};
use crate::error::{GitDbClusterError, GitDbClusterResult};

/// Multi-repository GitDB pool with read/write routing and load balancing.
pub struct GitDbCluster {
    nodes: Vec<Arc<GitDbNodePool>>,
    strategy: GitDbLoadBalanceStrategy,
    next_read: AtomicUsize,
    next_write: AtomicUsize,
}

impl GitDbCluster {
    /// Build a cluster from validated configuration.
    pub fn new(config: GitDbClusterConfig) -> GitDbClusterResult<Self> {
        config.validate()?;

        Ok(Self {
            nodes: config
                .nodes
                .into_iter()
                .map(GitDbNodePool::new)
                .map(|node| node.map(Arc::new))
                .collect::<GitDbClusterResult<Vec<_>>>()?,
            strategy: config.strategy,
            next_read: AtomicUsize::new(0),
            next_write: AtomicUsize::new(0),
        })
    }

    /// Execute SQL through the cluster router.
    ///
    /// Transaction control statements are rejected here because a transaction
    /// requires a stable checked-out connection. Use [`Self::checkout_write`]
    /// for `BEGIN`/`COMMIT`/`ROLLBACK` flows.
    pub fn execute(&self, sql: &str) -> GitDbClusterResult<GitDbRoutedResult> {
        match classify_gitdb_query(sql)? {
            GitDbQueryKind::Read => self.execute_classified(sql, GitDbQueryKind::Read),
            GitDbQueryKind::Write => self.execute_classified(sql, GitDbQueryKind::Write),
            GitDbQueryKind::TransactionControl => {
                Err(GitDbClusterError::TransactionRequiresConnection)
            }
        }
    }

    /// Execute a read SQL statement on a read-capable node.
    pub fn execute_read(&self, sql: &str) -> GitDbClusterResult<GitDbRoutedResult> {
        self.execute_expected(sql, GitDbQueryKind::Read)
    }

    /// Execute a write SQL statement on a write-capable node.
    pub fn execute_write(&self, sql: &str) -> GitDbClusterResult<GitDbRoutedResult> {
        self.execute_expected(sql, GitDbQueryKind::Write)
    }

    fn execute_expected(
        &self,
        sql: &str,
        expected: GitDbQueryKind,
    ) -> GitDbClusterResult<GitDbRoutedResult> {
        let actual = classify_gitdb_query(sql)?;
        if actual == GitDbQueryKind::TransactionControl {
            return Err(GitDbClusterError::TransactionRequiresConnection);
        }
        if actual != expected {
            return Err(GitDbClusterError::UnexpectedQueryKind { expected, actual });
        }

        self.execute_classified(sql, actual)
    }

    fn execute_classified(
        &self,
        sql: &str,
        kind: GitDbQueryKind,
    ) -> GitDbClusterResult<GitDbRoutedResult> {
        let mut connection = self.checkout_for(kind)?;
        let result = connection.execute(sql)?;
        Ok(GitDbRoutedResult {
            node_id: connection.node_id().to_owned(),
            result,
        })
    }

    /// Execute SQL against every configured node.
    ///
    /// This is useful for DDL setup across independent Git repositories.
    pub fn broadcast_execute(&self, sql: &str) -> GitDbClusterResult<Vec<GitDbRoutedResult>> {
        let mut results = Vec::with_capacity(self.nodes.len());
        for node in &self.nodes {
            let mut connection = node.checkout()?;
            let result = connection.execute(sql)?;
            results.push(GitDbRoutedResult {
                node_id: connection.node_id().to_owned(),
                result,
            });
        }
        Ok(results)
    }

    /// Check out a connection from a read-capable node.
    pub fn checkout_read(&self) -> GitDbClusterResult<GitDbConnection> {
        self.checkout_for(GitDbQueryKind::Read)
    }

    /// Check out a connection from a write-capable node.
    pub fn checkout_write(&self) -> GitDbClusterResult<GitDbConnection> {
        self.checkout_for(GitDbQueryKind::Write)
    }

    /// Check out a connection from a specific node.
    pub fn checkout_node(&self, node_id: &str) -> GitDbClusterResult<GitDbConnection> {
        let node = self
            .nodes
            .iter()
            .find(|node| node.config.id == node_id)
            .ok_or_else(|| GitDbClusterError::NodeNotFound {
                node_id: node_id.to_owned(),
            })?;

        node.checkout()
    }

    /// Return current per-node pool statistics.
    pub fn stats(&self) -> GitDbStats {
        GitDbStats {
            nodes: self.nodes.iter().map(|node| node.stats()).collect(),
        }
    }

    fn checkout_for(&self, kind: GitDbQueryKind) -> GitDbClusterResult<GitDbConnection> {
        let ordered = self.ordered_candidates(kind)?;
        let mut exhausted = Vec::new();

        for node in ordered {
            match node.checkout() {
                Ok(connection) => return Ok(connection),
                Err(GitDbClusterError::PoolExhausted { node_id, .. }) => {
                    exhausted.push(node_id);
                }
                Err(error) => return Err(error),
            }
        }

        Err(GitDbClusterError::PoolsExhausted {
            kind,
            node_ids: exhausted,
        })
    }

    fn ordered_candidates(
        &self,
        kind: GitDbQueryKind,
    ) -> GitDbClusterResult<Vec<Arc<GitDbNodePool>>> {
        let mut eligible: Vec<_> = self
            .nodes
            .iter()
            .filter(|node| node.supports(kind))
            .cloned()
            .collect();

        if eligible.is_empty() {
            return Err(GitDbClusterError::NoEligibleNode { kind });
        }

        match self.strategy {
            GitDbLoadBalanceStrategy::LeastInFlight => {
                eligible.sort_by_key(|node| node.in_flight.load(Ordering::Relaxed));
                Ok(eligible)
            }
            GitDbLoadBalanceStrategy::RoundRobin => {
                let start = self.next_counter(kind).fetch_add(1, Ordering::Relaxed);
                Ok(rotate_candidates(eligible, start))
            }
            GitDbLoadBalanceStrategy::WeightedRoundRobin => {
                let start = self.next_counter(kind).fetch_add(1, Ordering::Relaxed);
                let selected = weighted_index(&eligible, start);
                Ok(rotate_candidates(eligible, selected))
            }
        }
    }

    fn next_counter(&self, kind: GitDbQueryKind) -> &AtomicUsize {
        match kind {
            GitDbQueryKind::Read => &self.next_read,
            GitDbQueryKind::Write | GitDbQueryKind::TransactionControl => &self.next_write,
        }
    }
}

/// A checked-out GitDB connection that returns to its node pool on drop.
pub struct GitDbConnection {
    node: Arc<GitDbNodePool>,
    connection: Option<Connection>,
}

impl GitDbConnection {
    /// Node id backing this connection.
    pub fn node_id(&self) -> &str {
        &self.node.config.id
    }

    /// Execute SQL on this checked-out connection.
    pub fn execute(&mut self, sql: &str) -> GitDbClusterResult<QueryResult> {
        let node_id = self.node_id().to_owned();
        self.connection_mut()?
            .execute(sql)
            .map_err(|source| GitDbClusterError::NodeDatabase { node_id, source })
    }

    /// Execute a semicolon-separated SQL batch on this checked-out connection.
    pub fn execute_batch(&mut self, sql: &str) -> GitDbClusterResult<Vec<QueryResult>> {
        sql.split(';')
            .map(str::trim)
            .filter(|statement| !statement.is_empty())
            .map(|statement| self.execute(statement))
            .collect()
    }

    fn connection_mut(&mut self) -> GitDbClusterResult<&mut Connection> {
        self.connection.as_mut().ok_or_else(|| {
            GitDbClusterError::Internal("checked-out connection has no upstream connection".into())
        })
    }
}

impl Drop for GitDbConnection {
    fn drop(&mut self) {
        if let Some(connection) = self.connection.take() {
            self.node.return_connection(connection);
        }
    }
}

/// Query result plus the node that served it.
#[derive(Debug)]
pub struct GitDbRoutedResult {
    /// Node id selected by the router.
    pub node_id: String,
    /// Upstream GitDB query result.
    pub result: QueryResult,
}

/// Cluster-level pool statistics.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GitDbStats {
    /// Per-node statistics.
    pub nodes: Vec<GitDbNodeStats>,
}

/// Pool statistics for one node.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GitDbNodeStats {
    /// Node id.
    pub id: String,
    /// Repository path.
    pub path: PathBuf,
    /// Node role.
    pub role: GitDbNodeRole,
    /// Node weight.
    pub weight: usize,
    /// Maximum checked-out connections.
    pub max_connections: usize,
    /// Total opened upstream database handles.
    pub opened: usize,
    /// Idle database handles currently retained by the pool.
    pub idle: usize,
    /// Currently checked-out connection count.
    pub in_flight: usize,
}

struct GitDbNodePool {
    config: GitDbNodeConfig,
    pool: ConnectionPool,
    in_flight: AtomicUsize,
}

impl GitDbNodePool {
    fn new(config: GitDbNodeConfig) -> GitDbClusterResult<Self> {
        let pool_config = DatabaseConfig {
            path: config.path.clone(),
            create_if_missing: config.create_if_missing,
            enable_planner: config.enable_planner,
            verbose: config.verbose,
            auto_commit: config.auto_commit,
        };
        let pool = ConnectionPool::new(pool_config, config.max_connections).map_err(|source| {
            GitDbClusterError::NodeDatabase {
                node_id: config.id.clone(),
                source,
            }
        })?;

        Ok(Self {
            config,
            pool,
            in_flight: AtomicUsize::new(0),
        })
    }

    fn supports(&self, kind: GitDbQueryKind) -> bool {
        match kind {
            GitDbQueryKind::Read => self.config.role.supports_read(),
            GitDbQueryKind::Write | GitDbQueryKind::TransactionControl => {
                self.config.role.supports_write()
            }
        }
    }

    fn checkout(self: &Arc<Self>) -> GitDbClusterResult<GitDbConnection> {
        match self.pool.get() {
            Ok(connection) => {
                self.in_flight.fetch_add(1, Ordering::Relaxed);
                Ok(GitDbConnection {
                    node: Arc::clone(self),
                    connection: Some(connection),
                })
            }
            Err(DatabaseError::InvalidConfig(message))
                if message == "connection pool exhausted" =>
            {
                Err(GitDbClusterError::PoolExhausted {
                    node_id: self.config.id.clone(),
                    max_connections: self.config.max_connections,
                })
            }
            Err(source) => Err(GitDbClusterError::NodeDatabase {
                node_id: self.config.id.clone(),
                source,
            }),
        }
    }

    fn stats(&self) -> GitDbNodeStats {
        GitDbNodeStats {
            id: self.config.id.clone(),
            path: self.config.path.clone(),
            role: self.config.role,
            weight: self.config.weight,
            max_connections: self.config.max_connections,
            opened: self.pool.created(),
            idle: self.pool.available(),
            in_flight: self.in_flight.load(Ordering::Relaxed),
        }
    }

    fn return_connection(&self, connection: Connection) {
        drop(connection);
        self.in_flight.fetch_sub(1, Ordering::Relaxed);
    }
}

fn rotate_candidates<T>(mut values: Vec<T>, start: usize) -> Vec<T> {
    let len = values.len();
    if len > 1 {
        values.rotate_left(start % len);
    }
    values
}

fn weighted_index(nodes: &[Arc<GitDbNodePool>], counter: usize) -> usize {
    let total_weight = nodes.iter().map(|node| node.config.weight).sum::<usize>();
    let mut slot = counter % total_weight;

    for (index, node) in nodes.iter().enumerate() {
        if slot < node.config.weight {
            return index;
        }
        slot -= node.config.weight;
    }

    0
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{GitDbClusterConfig, GitDbLoadBalanceStrategy};
    use std::path::Path;

    #[test]
    fn round_robin_should_spread_write_queries() {
        let dir = tempfile::tempdir().unwrap();
        let cluster = test_cluster(
            dir.path(),
            GitDbLoadBalanceStrategy::RoundRobin,
            &[("a", 2), ("b", 2)],
        );

        cluster
            .broadcast_execute("CREATE TABLE users (id TEXT PRIMARY KEY, name TEXT)")
            .unwrap();

        let first = cluster
            .execute_write("INSERT INTO users (id, name) VALUES ('1', 'Alice')")
            .unwrap();
        let second = cluster
            .execute_write("INSERT INTO users (id, name) VALUES ('2', 'Bob')")
            .unwrap();

        assert_ne!(first.node_id, second.node_id);
    }

    #[test]
    fn checked_out_connection_should_return_to_pool_on_drop() {
        let dir = tempfile::tempdir().unwrap();
        let cluster = GitDbCluster::new(GitDbClusterConfig::new(vec![
            GitDbNodeConfig::new("primary", dir.path().join("primary")).max_connections(1),
        ]))
        .unwrap();

        let connection = cluster.checkout_write().unwrap();
        let error = match cluster.checkout_write() {
            Ok(_) => panic!("pool should be exhausted"),
            Err(error) => error,
        };
        assert!(matches!(error, GitDbClusterError::PoolsExhausted { .. }));

        drop(connection);

        let connection = cluster.checkout_write().unwrap();
        assert_eq!(connection.node_id(), "primary");
    }

    #[test]
    fn cluster_execute_should_reject_transaction_control() {
        let dir = tempfile::tempdir().unwrap();
        let cluster = test_cluster(
            dir.path(),
            GitDbLoadBalanceStrategy::RoundRobin,
            &[("primary", 1)],
        );

        let error = cluster.execute("BEGIN").unwrap_err();

        assert!(matches!(
            error,
            GitDbClusterError::TransactionRequiresConnection
        ));
    }

    #[test]
    fn execute_read_should_reject_write_sql() {
        let dir = tempfile::tempdir().unwrap();
        let cluster = test_cluster(
            dir.path(),
            GitDbLoadBalanceStrategy::RoundRobin,
            &[("primary", 1)],
        );

        let error = cluster
            .execute_read("CREATE TABLE users (id TEXT PRIMARY KEY)")
            .unwrap_err();

        assert!(matches!(
            error,
            GitDbClusterError::UnexpectedQueryKind {
                expected: GitDbQueryKind::Read,
                actual: GitDbQueryKind::Write
            }
        ));
    }

    #[test]
    fn weighted_round_robin_should_honor_weights() {
        let dir = tempfile::tempdir().unwrap();
        let cluster = test_cluster(
            dir.path(),
            GitDbLoadBalanceStrategy::WeightedRoundRobin,
            &[("a", 2), ("b", 1)],
        );

        let mut selected = Vec::new();
        for _ in 0..3 {
            let connection = cluster.checkout_write().unwrap();
            selected.push(connection.node_id().to_owned());
        }

        assert_eq!(selected, vec!["a", "a", "b"]);
    }

    fn test_cluster(
        root: &Path,
        strategy: GitDbLoadBalanceStrategy,
        nodes: &[(&str, usize)],
    ) -> GitDbCluster {
        let configs = nodes
            .iter()
            .map(|(id, weight)| {
                GitDbNodeConfig::new(*id, root.join(id))
                    .max_connections(2)
                    .weight(*weight)
            })
            .collect();

        GitDbCluster::new(GitDbClusterConfig::new(configs).strategy(strategy)).unwrap()
    }
}
