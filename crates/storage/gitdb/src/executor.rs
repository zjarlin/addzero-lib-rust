//! Query execution engine for GitDB.
//!
//! Uses the Volcano/Iterator model where each operator produces
//! rows one at a time.

automod::dir!("src/executor");

pub use executor::QueryExecutor;
pub use result::{QueryResult, ResultSet, RowIter};
