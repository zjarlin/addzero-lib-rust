//! Multi-repository pooling for [`gitdb`].
//!
//! This crate wraps the upstream Git-backed SQL database with explicit node
//! configuration, bounded per-node connection pools, and load-balancing
//! strategies across multiple Git repositories.
//!
//! # Consistency boundary
//!
//! `az-gitdb` does not replicate data between Git repositories. Load balancing
//! assumes the configured repositories are equivalent replicas, or that the
//! caller deliberately treats them as independent shards. Use
//! [`GitDbCluster::broadcast_execute`] for schema setup or other statements
//! that must be applied to every node.

#![forbid(unsafe_code)]

mod classify;
mod cluster;
mod config;
mod error;

pub use classify::{GitDbQueryKind, classify_gitdb_query};
pub use cluster::{GitDbCluster, GitDbConnection, GitDbNodeStats, GitDbRoutedResult, GitDbStats};
pub use config::{GitDbClusterConfig, GitDbLoadBalanceStrategy, GitDbNodeConfig, GitDbNodeRole};
pub use error::{GitDbClusterError, GitDbClusterResult};
pub use gitdb::db::{Database, DatabaseConfig};
pub use gitdb::executor::{QueryResult, ResultSet};
