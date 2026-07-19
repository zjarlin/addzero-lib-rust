use std::path::PathBuf;

use anyhow::{Result, bail};

/// The role a GitDB repository node can serve inside a cluster.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, strum::EnumString, strum::IntoStaticStr, strum::VariantArray)]
#[strum(serialize_all = "snake_case")]
pub enum GitDbNodeRole {
    /// Accept both read and write SQL statements.
    ReadWrite,
    /// Accept read SQL statements only.
    ReadOnly,
    /// Accept write SQL statements only.
    WriteOnly,
}

impl GitDbNodeRole {
    #[allow(dead_code)]
    pub const ALL: &'static [Self] = <Self as strum::VariantArray>::VARIANTS;

    #[must_use]
    pub fn as_str(self) -> &'static str {
        self.into()
    }

    #[must_use]
    pub fn code(self) -> &'static str {
        self.as_str()
    }

    pub fn from_code(value: &str) -> Option<Self> {
        value.parse().ok()
    }
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
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Default, strum::EnumString, strum::IntoStaticStr, strum::VariantArray)]
#[strum(serialize_all = "snake_case")]
pub enum GitDbLoadBalanceStrategy {
    /// Rotate eligible nodes in a stable sequence.
    #[default]
    RoundRobin,
    /// Rotate by node weight. Higher weights receive proportionally more starts.
    WeightedRoundRobin,
    /// Prefer the eligible node with the lowest currently checked-out count.
    LeastInFlight,
}

impl GitDbLoadBalanceStrategy {
    #[allow(dead_code)]
    pub const ALL: &'static [Self] = <Self as strum::VariantArray>::VARIANTS;

    #[must_use]
    pub fn as_str(self) -> &'static str {
        self.into()
    }

    #[must_use]
    pub fn code(self) -> &'static str {
        self.as_str()
    }

    pub fn from_code(value: &str) -> Option<Self> {
        value.parse().ok()
    }

    #[must_use]
    pub fn from_code_or_default(value: &str) -> Self {
        Self::from_code(value).unwrap_or_default()
    }
}

/// Configuration for one Git-backed database node.
#[derive(Clone, Debug)]
pub struct GitDbNodeConfig {
    /// Stable node identifier returned with routed results and errors.
    pub id: String,
    /// Clone-capable remote URL for the GitDB repository.
    pub remote_url: String,
    /// Local checkout path used by upstream GitDB.
    pub checkout_path: PathBuf,
    /// Maximum number of checked-out connections for this node.
    pub max_connections: usize,
    /// Node role used for read/write routing.
    pub role: GitDbNodeRole,
    /// Weight used by [`GitDbLoadBalanceStrategy::WeightedRoundRobin`].
    pub weight: usize,
    /// Clone the remote repository when the local checkout is missing.
    pub clone_if_missing: bool,
    /// Enable upstream GitDB query planning.
    pub enable_planner: bool,
    /// Enable upstream GitDB verbose SQL logging.
    pub verbose: bool,
    /// Enable upstream GitDB auto-commit behavior.
    pub auto_commit: bool,
}

impl GitDbNodeConfig {
    /// Create a read-write node backed by an existing remote GitDB repository.
    pub fn new(
        id: impl Into<String>,
        remote_url: impl Into<String>,
        checkout_path: impl Into<PathBuf>,
    ) -> Self {
        Self {
            id: id.into(),
            remote_url: remote_url.into(),
            checkout_path: checkout_path.into(),
            max_connections: 4,
            role: GitDbNodeRole::ReadWrite,
            weight: 1,
            clone_if_missing: true,
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

    /// Set whether the remote repository is cloned when the checkout is missing.
    pub fn clone_if_missing(mut self, value: bool) -> Self {
        self.clone_if_missing = value;
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

    pub(crate) fn validate(&self) -> Result<()> {
        if self.id.trim().is_empty() {
            bail!("invalid GitDB cluster configuration: node id must not be empty");
        }

        if self.remote_url.trim().is_empty() {
            bail!(
                "invalid GitDB cluster configuration: node '{}' remote URL must not be empty",
                self.id
            );
        }

        if self.checkout_path.as_os_str().is_empty() {
            bail!(
                "invalid GitDB cluster configuration: node '{}' checkout path must not be empty",
                self.id
            );
        }

        if self.max_connections == 0 {
            bail!(
                "invalid GitDB cluster configuration: node '{}' must allow at least one connection",
                self.id
            );
        }

        if self.weight == 0 {
            bail!(
                "invalid GitDB cluster configuration: node '{}' weight must be greater than zero",
                self.id
            );
        }

        Ok(())
    }
}

/// Configuration for a multi-repository GitDB cluster.
#[derive(Clone, Debug)]
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

    pub(crate) fn validate(&self) -> Result<()> {
        if self.nodes.is_empty() {
            bail!("invalid GitDB cluster configuration: at least one node is required");
        }

        let mut ids = std::collections::BTreeSet::new();
        for node in &self.nodes {
            node.validate()?;
            if !ids.insert(node.id.as_str()) {
                bail!(
                    "invalid GitDB cluster configuration: duplicate node id '{}'",
                    node.id
                );
            }
        }

        Ok(())
    }
}
