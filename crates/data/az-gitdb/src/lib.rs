#![doc = include_str!("../README.md")]
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
