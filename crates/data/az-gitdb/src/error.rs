use az_derive_aliases::{apply, error};

use crate::classify::GitDbQueryKind;

/// Result type for multi-node GitDB operations.
pub type GitDbClusterResult<T> = Result<T, GitDbClusterError>;

/// Errors returned by the multi-node GitDB pool.
#[apply(error)]
pub enum GitDbClusterError {
    /// Cluster or node configuration is invalid.
    #[error("invalid GitDB cluster configuration: {0}")]
    InvalidConfig(String),

    /// No configured node can serve the requested query kind.
    #[error("no eligible GitDB node for {kind} query")]
    NoEligibleNode { kind: GitDbQueryKind },

    /// A specific node id does not exist.
    #[error("GitDB node not found: {node_id}")]
    NodeNotFound { node_id: String },

    /// All eligible pools are at capacity.
    #[error("all eligible GitDB pools are exhausted for {kind} query: {node_ids:?}")]
    PoolsExhausted {
        kind: GitDbQueryKind,
        node_ids: Vec<String>,
    },

    /// One node pool is at capacity.
    #[error("GitDB pool exhausted: node={node_id}, max_connections={max_connections}")]
    PoolExhausted {
        node_id: String,
        max_connections: usize,
    },

    /// Cluster-level execution rejected transaction control because it needs a stable connection.
    #[error("transaction control requires an explicitly checked-out connection")]
    TransactionRequiresConnection,

    /// The SQL kind does not match the explicit execution API that was called.
    #[error("expected {expected} query, got {actual} query")]
    UnexpectedQueryKind {
        expected: GitDbQueryKind,
        actual: GitDbQueryKind,
    },

    /// SQL could not be parsed for cluster routing.
    #[error("failed to classify GitDB SQL: {0}")]
    Parse(#[from] gitdb::sql::ParseError),

    /// Upstream GitDB returned an error for a specific node.
    #[error("GitDB node '{node_id}' failed: {source}")]
    NodeDatabase {
        node_id: String,
        #[source]
        source: gitdb::db::DatabaseError,
    },

    /// Internal synchronization failed.
    #[error("internal GitDB cluster error: {0}")]
    Internal(String),
}
