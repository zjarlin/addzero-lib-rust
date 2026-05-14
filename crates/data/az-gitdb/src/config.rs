use std::path::PathBuf;

use crate::error::{GitDbClusterError, GitDbClusterResult};

/// The role a GitDB repository node can serve inside a cluster.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GitDbNodeRole {
    /// Accept both read and write SQL statements.
    ReadWrite,
    /// Accept read SQL statements only.
    ReadOnly,
    /// Accept write SQL statements only.
    WriteOnly,
}

impl GitDbNodeRole {
    pub(crate) fn supports_read(self) -> bool {
        matches!(self, Self::ReadWrite | Self::ReadOnly)
    }

    pub(crate) fn supports_write(self) -> bool {
        matches!(self, Self::ReadWrite | Self::WriteOnly)
    }
}

/// Load-balancing strategy used when more than one node can serve a request.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum GitDbLoadBalanceStrategy {
    /// Rotate eligible nodes in a stable sequence.
    #[default]
    RoundRobin,
    /// Rotate by node weight. Higher weights receive proportionally more starts.
    WeightedRoundRobin,
    /// Prefer the eligible node with the lowest currently checked-out count.
    LeastInFlight,
}

/// Configuration for one Git-backed database node.
#[derive(Debug, Clone)]
pub struct GitDbNodeConfig {
    /// Stable node identifier returned with routed results and errors.
    pub id: String,
    /// Filesystem path of the GitDB repository.
    pub path: PathBuf,
    /// Maximum number of checked-out connections for this node.
    pub max_connections: usize,
    /// Node role used for read/write routing.
    pub role: GitDbNodeRole,
    /// Weight used by [`GitDbLoadBalanceStrategy::WeightedRoundRobin`].
    pub weight: usize,
    /// Create the Git repository when it does not exist.
    pub create_if_missing: bool,
    /// Enable upstream GitDB query planning.
    pub enable_planner: bool,
    /// Enable upstream GitDB verbose SQL logging.
    pub verbose: bool,
    /// Enable upstream GitDB auto-commit behavior.
    pub auto_commit: bool,
}

impl GitDbNodeConfig {
    /// Create a read-write node with conservative defaults.
    pub fn new(id: impl Into<String>, path: impl Into<PathBuf>) -> Self {
        Self {
            id: id.into(),
            path: path.into(),
            max_connections: 4,
            role: GitDbNodeRole::ReadWrite,
            weight: 1,
            create_if_missing: true,
            enable_planner: true,
            verbose: false,
            auto_commit: true,
        }
    }

    /// Set the maximum checked-out connections for this node.
    pub fn max_connections(mut self, value: usize) -> Self {
        self.max_connections = value;
        self
    }

    /// Set the node role.
    pub fn role(mut self, value: GitDbNodeRole) -> Self {
        self.role = value;
        self
    }

    /// Set the weighted round-robin weight.
    pub fn weight(mut self, value: usize) -> Self {
        self.weight = value;
        self
    }

    /// Set whether the repository is created when missing.
    pub fn create_if_missing(mut self, value: bool) -> Self {
        self.create_if_missing = value;
        self
    }

    /// Set whether the upstream planner is enabled.
    pub fn enable_planner(mut self, value: bool) -> Self {
        self.enable_planner = value;
        self
    }

    /// Set upstream verbose logging.
    pub fn verbose(mut self, value: bool) -> Self {
        self.verbose = value;
        self
    }

    /// Set upstream auto-commit behavior.
    pub fn auto_commit(mut self, value: bool) -> Self {
        self.auto_commit = value;
        self
    }

    pub(crate) fn validate(&self) -> GitDbClusterResult<()> {
        if self.id.trim().is_empty() {
            return Err(GitDbClusterError::InvalidConfig(
                "node id must not be empty".into(),
            ));
        }

        if self.max_connections == 0 {
            return Err(GitDbClusterError::InvalidConfig(format!(
                "node '{}' must allow at least one connection",
                self.id
            )));
        }

        if self.weight == 0 {
            return Err(GitDbClusterError::InvalidConfig(format!(
                "node '{}' weight must be greater than zero",
                self.id
            )));
        }

        Ok(())
    }
}

/// Configuration for a multi-repository GitDB cluster.
#[derive(Debug, Clone)]
pub struct GitDbClusterConfig {
    /// GitDB repository nodes.
    pub nodes: Vec<GitDbNodeConfig>,
    /// Load-balancing strategy.
    pub strategy: GitDbLoadBalanceStrategy,
}

impl GitDbClusterConfig {
    /// Create a cluster config with the given nodes.
    pub fn new(nodes: Vec<GitDbNodeConfig>) -> Self {
        Self {
            nodes,
            strategy: GitDbLoadBalanceStrategy::default(),
        }
    }

    /// Set the load-balancing strategy.
    pub fn strategy(mut self, value: GitDbLoadBalanceStrategy) -> Self {
        self.strategy = value;
        self
    }

    pub(crate) fn validate(&self) -> GitDbClusterResult<()> {
        if self.nodes.is_empty() {
            return Err(GitDbClusterError::InvalidConfig(
                "at least one node is required".into(),
            ));
        }

        let mut ids = std::collections::BTreeSet::new();
        for node in &self.nodes {
            node.validate()?;
            if !ids.insert(node.id.as_str()) {
                return Err(GitDbClusterError::InvalidConfig(format!(
                    "duplicate node id '{}'",
                    node.id
                )));
            }
        }

        Ok(())
    }
}
