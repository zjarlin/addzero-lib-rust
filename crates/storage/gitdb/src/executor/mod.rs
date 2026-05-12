//! Query execution engine for GitDB.
//!
//! Uses the Volcano/Iterator model where each operator produces
//! rows one at a time.

mod error;
mod eval;
mod executor;
mod operators;
mod result;

pub use error::{ExecuteError, ExecuteResult};
pub use executor::QueryExecutor;
pub use result::{QueryResult, ResultSet, RowIter};
