use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use anyhow::{Context, Result, bail};
use az_derive_aliases::{apply, plain_debug, plain_eq};
use git2::Repository;
use gitdb::db::{Connection, ConnectionPool, DatabaseConfig};
use gitdb::executor::QueryResult;

use crate::classify::{GitDbQueryKind, classify_gitdb_query};
use crate::config::{GitDbClusterConfig, GitDbLoadBalanceStrategy, GitDbNodeConfig, GitDbNodeRole};

/// Multi-repository GitDB pool with read/write routing and load balancing.
pub struct GitDbCluster {
    nodes: Vec<Arc<GitDbNodePool>>,
    strategy: GitDbLoadBalanceStrategy,
    next_read: AtomicUsize,
    next_write: AtomicUsize,
}

impl GitDbCluster {
    /// Build a cluster from validated configuration.
    pub fn new(config: GitDbClusterConfig) -> Result<Self> {
        config.validate()?;

        Ok(Self {
            nodes: config
                .nodes
                .into_iter()
                .map(GitDbNodePool::new)
                .map(|node| node.map(Arc::new))
                .collect::<Result<Vec<_>>>()?,
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
    pub fn execute(&self, sql: &str) -> Result<GitDbRoutedResult> {
        match classify_gitdb_query(sql)? {
            GitDbQueryKind::Read => self.execute_classified(sql, GitDbQueryKind::Read),
            GitDbQueryKind::Write => self.execute_classified(sql, GitDbQueryKind::Write),
            GitDbQueryKind::TransactionControl => {
                bail!("transaction control requires an explicitly checked-out connection");
            }
        }
    }

    /// Execute a read SQL statement on a read-capable node.
    pub fn execute_read(&self, sql: &str) -> Result<GitDbRoutedResult> {
        self.execute_expected(sql, GitDbQueryKind::Read)
    }

    /// Execute a write SQL statement on a write-capable node.
    pub fn execute_write(&self, sql: &str) -> Result<GitDbRoutedResult> {
        self.execute_expected(sql, GitDbQueryKind::Write)
    }

    fn execute_expected(&self, sql: &str, expected: GitDbQueryKind) -> Result<GitDbRoutedResult> {
        let actual = classify_gitdb_query(sql)?;
        if actual == GitDbQueryKind::TransactionControl {
            bail!("transaction control requires an explicitly checked-out connection");
        }
        if actual != expected {
            bail!("expected {expected} query, got {actual} query");
        }

        self.execute_classified(sql, actual)
    }

    fn execute_classified(&self, sql: &str, kind: GitDbQueryKind) -> Result<GitDbRoutedResult> {
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
    pub fn broadcast_execute(&self, sql: &str) -> Result<Vec<GitDbRoutedResult>> {
        let mut results = Vec::with_capacity(self.nodes.len());
        for node in &self.nodes {
            let Some(mut connection) = node.try_checkout()? else {
                bail!(
                    "GitDB pool exhausted: node={}, max_connections={}",
                    node.config.id,
                    node.config.max_connections
                );
            };
            let result = connection.execute(sql)?;
            results.push(GitDbRoutedResult {
                node_id: connection.node_id().to_owned(),
                result,
            });
        }
        Ok(results)
    }

    /// Check out a connection from a read-capable node.
    pub fn checkout_read(&self) -> Result<GitDbConnection> {
        self.checkout_for(GitDbQueryKind::Read)
    }

    /// Check out a connection from a write-capable node.
    pub fn checkout_write(&self) -> Result<GitDbConnection> {
        self.checkout_for(GitDbQueryKind::Write)
    }

    /// Check out a connection from a specific node.
    pub fn checkout_node(&self, node_id: &str) -> Result<GitDbConnection> {
        let node = self
            .nodes
            .iter()
            .find(|node| node.config.id == node_id)
            .with_context(|| format!("GitDB node not found: {node_id}"))?;

        node.try_checkout()?.with_context(|| {
            format!(
                "GitDB pool exhausted: node={}, max_connections={}",
                node.config.id, node.config.max_connections
            )
        })
    }

    /// Return current per-node pool statistics.
    pub fn stats(&self) -> GitDbStats {
        GitDbStats {
            nodes: self.nodes.iter().map(|node| node.stats()).collect(),
        }
    }

    fn checkout_for(&self, kind: GitDbQueryKind) -> Result<GitDbConnection> {
        let ordered = self.ordered_candidates(kind)?;
        let mut exhausted = Vec::new();

        for node in ordered {
            match node.try_checkout()? {
                Some(connection) => return Ok(connection),
                None => {
                    exhausted.push(node.config.id.clone());
                }
            }
        }

        bail!("all eligible GitDB pools are exhausted for {kind} query: {exhausted:?}");
    }

    fn ordered_candidates(&self, kind: GitDbQueryKind) -> Result<Vec<Arc<GitDbNodePool>>> {
        let mut eligible: Vec<_> = self
            .nodes
            .iter()
            .filter(|node| node.supports(kind))
            .cloned()
            .collect();

        if eligible.is_empty() {
            bail!("no eligible GitDB node for {kind} query");
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
    pub fn execute(&mut self, sql: &str) -> Result<QueryResult> {
        let node_id = self.node_id().to_owned();
        self.connection_mut()?
            .execute(sql)
            .with_context(|| format!("GitDB node '{node_id}' failed"))
    }

    /// Execute a semicolon-separated SQL batch on this checked-out connection.
    pub fn execute_batch(&mut self, sql: &str) -> Result<Vec<QueryResult>> {
        sql.split(';')
            .map(str::trim)
            .filter(|statement| !statement.is_empty())
            .map(|statement| self.execute(statement))
            .collect()
    }

    fn connection_mut(&mut self) -> Result<&mut Connection> {
        self.connection
            .as_mut()
            .context("internal GitDB cluster error: checked-out connection has no upstream connection")
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
#[apply(plain_debug)]
pub struct GitDbRoutedResult {
    /// Node id selected by the router.
    pub node_id: String,
    /// Upstream GitDB query result.
    pub result: QueryResult,
}

/// Cluster-level pool statistics.
#[apply(plain_eq)]
pub struct GitDbStats {
    /// Per-node statistics.
    pub nodes: Vec<GitDbNodeStats>,
}

/// Pool statistics for one node.
#[apply(plain_eq)]
pub struct GitDbNodeStats {
    /// Node id.
    pub id: String,
    /// Clone-capable remote URL for the GitDB repository.
    pub remote_url: String,
    /// Local checkout path used by upstream GitDB.
    pub checkout_path: PathBuf,
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
    fn new(config: GitDbNodeConfig) -> Result<Self> {
        prepare_checkout(&config)?;
        let pool_config = DatabaseConfig {
            path: config.checkout_path.clone(),
            create_if_missing: false,
            enable_planner: config.enable_planner,
            verbose: config.verbose,
            auto_commit: config.auto_commit,
        };
        let pool = ConnectionPool::new(pool_config, config.max_connections)
            .with_context(|| format!("GitDB node '{}' failed", config.id))?;

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

    fn try_checkout(self: &Arc<Self>) -> Result<Option<GitDbConnection>> {
        match self.pool.get() {
            Ok(connection) => {
                self.in_flight.fetch_add(1, Ordering::Relaxed);
                Ok(Some(GitDbConnection {
                    node: Arc::clone(self),
                    connection: Some(connection),
                }))
            }
            Err(error) if is_pool_exhausted(&error) => Ok(None),
            Err(source) => {
                Err(source).with_context(|| format!("GitDB node '{}' failed", self.config.id))
            }
        }
    }

    fn stats(&self) -> GitDbNodeStats {
        GitDbNodeStats {
            id: self.config.id.clone(),
            remote_url: self.config.remote_url.clone(),
            checkout_path: self.config.checkout_path.clone(),
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

fn prepare_checkout(config: &GitDbNodeConfig) -> Result<()> {
    if config.checkout_path.join(".git").exists() {
        let repository = Repository::open(&config.checkout_path).with_context(|| {
            format!(
                "GitDB node '{}' remote checkout failed at '{}'",
                config.id,
                config.checkout_path.display()
            )
        })?;
        validate_origin(config, &repository)?;
        return Ok(());
    }

    if config.checkout_path.exists() || !config.clone_if_missing {
        bail!(
            "node '{}' checkout '{}' is not a Git repository cloned from '{}'",
            config.id,
            config.checkout_path.display(),
            config.remote_url
        );
    }

    if let Some(parent) = config.checkout_path.parent() {
        std::fs::create_dir_all(parent).with_context(|| {
            format!(
                "GitDB node '{}' checkout directory failed at '{}'",
                config.id,
                config.checkout_path.display()
            )
        })?;
    }

    Repository::clone(&config.remote_url, &config.checkout_path).with_context(|| {
        format!(
            "GitDB node '{}' remote checkout failed at '{}'",
            config.id,
            config.checkout_path.display()
        )
    })?;
    Ok(())
}

fn is_pool_exhausted(error: &anyhow::Error) -> bool {
    error.to_string() == "invalid configuration: connection pool exhausted"
}

fn validate_origin(config: &GitDbNodeConfig, repository: &Repository) -> Result<()> {
    let origin = repository.find_remote("origin").with_context(|| {
        format!(
            "GitDB node '{}' remote checkout failed at '{}'",
            config.id,
            config.checkout_path.display()
        )
    })?;
    let Some(url) = origin.url() else {
        bail!(
            "node '{}' checkout '{}' has no UTF-8 origin URL",
            config.id,
            config.checkout_path.display()
        );
    };

    if normalize_remote_url(url) != normalize_remote_url(&config.remote_url) {
        bail!(
            "node '{}' checkout '{}' origin '{}' does not match configured remote '{}'",
            config.id,
            config.checkout_path.display(),
            url,
            config.remote_url
        );
    }

    Ok(())
}

fn normalize_remote_url(url: &str) -> &str {
    url.trim().trim_end_matches('/')
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
    use super::GitDbCluster;
    use crate::config::{GitDbClusterConfig, GitDbLoadBalanceStrategy, GitDbNodeConfig};
    use git2::Repository;
    use gitdb::db::{Database, DatabaseConfig};
    use std::path::Path;
    use std::path::PathBuf;

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
        let remote = create_remote_gitdb(dir.path(), "primary");
        let cluster = GitDbCluster::new(GitDbClusterConfig::new(vec![
            GitDbNodeConfig::new(
                "primary",
                remote.to_string_lossy(),
                dir.path().join("checkout-primary"),
            )
            .max_connections(1),
        ]))
        .unwrap();

        let connection = cluster.checkout_write().unwrap();
        let error = match cluster.checkout_write() {
            Ok(_) => panic!("pool should be exhausted"),
            Err(error) => error,
        };
        assert!(
            error
                .to_string()
                .contains("all eligible GitDB pools are exhausted")
        );

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

        assert_eq!(
            error.to_string(),
            "transaction control requires an explicitly checked-out connection"
        );
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

        assert_eq!(error.to_string(), "expected read query, got write query");
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

    #[test]
    fn cluster_should_reject_existing_checkout_with_wrong_origin() {
        let dir = tempfile::tempdir().unwrap();
        let configured_remote = create_remote_gitdb(dir.path(), "configured");
        let other_remote = create_remote_gitdb(dir.path(), "other");
        let checkout = dir.path().join("checkout");
        Repository::clone(other_remote.to_str().unwrap(), &checkout).unwrap();

        let error = match GitDbCluster::new(GitDbClusterConfig::new(vec![GitDbNodeConfig::new(
            "primary",
            configured_remote.to_string_lossy(),
            checkout,
        )])) {
            Ok(_) => panic!("checkout with mismatched origin should be rejected"),
            Err(error) => error,
        };

        assert!(
            error
                .to_string()
                .contains("does not match configured remote")
        );
    }

    fn test_cluster(
        root: &Path,
        strategy: GitDbLoadBalanceStrategy,
        nodes: &[(&str, usize)],
    ) -> GitDbCluster {
        let configs = nodes
            .iter()
            .map(|(id, weight)| {
                let remote = create_remote_gitdb(root, id);
                GitDbNodeConfig::new(
                    *id,
                    remote.to_string_lossy(),
                    root.join(format!("checkout-{id}")),
                )
                .max_connections(2)
                .weight(*weight)
            })
            .collect();

        GitDbCluster::new(GitDbClusterConfig::new(configs).strategy(strategy)).unwrap()
    }

    fn create_remote_gitdb(root: &Path, id: &str) -> PathBuf {
        let source = root.join(format!("source-{id}"));
        Database::open_with_config(DatabaseConfig::new(&source).create_if_missing(true)).unwrap();
        let remote = root.join(format!("{id}.git"));
        Repository::clone(source.to_str().unwrap(), &remote).unwrap();
        remote
    }
}
