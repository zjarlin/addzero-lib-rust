//! Transaction isolation levels.
//!
//! GitDB supports two isolation levels:
//! - ReadCommitted: Reads see the latest committed state of main
//! - RepeatableRead: Reads see the state at transaction start (snapshot isolation)
use az_derive_aliases::{apply, plain_default_copy_eq_display};

/// Transaction isolation level.
#[apply(plain_default_copy_eq_display)]
pub enum IsolationLevel {
    /// Read Committed isolation.
    ///
    /// Each read sees the most recently committed data at the time of the read.
    /// This means different reads within the same transaction might see different
    /// data if another transaction commits in between.
    ///
    /// Pros:
    /// - Always see fresh data
    /// - No phantom read issues for single-row reads
    ///
    /// Cons:
    /// - Non-repeatable reads possible
    /// - May see partial effects of other transactions
    #[default]
    #[display("READ COMMITTED")]
    ReadCommitted,

    /// Repeatable Read isolation (Snapshot Isolation).
    ///
    /// All reads within a transaction see a consistent snapshot of the database
    /// as of the transaction's start time. The transaction operates on this
    /// snapshot and is unaware of concurrent modifications.
    ///
    /// Pros:
    /// - Consistent view throughout transaction
    /// - Repeatable reads guaranteed
    ///
    /// Cons:
    /// - May operate on stale data
    /// - Higher chance of conflict at commit time
    #[display("REPEATABLE READ")]
    RepeatableRead,
}

impl IsolationLevel {
    /// Check if this isolation level uses snapshot reads.
    pub fn uses_snapshot(&self) -> bool {
        matches!(self, IsolationLevel::RepeatableRead)
    }

    /// Get a human-readable description of this isolation level.
    pub fn description(&self) -> &'static str {
        match self {
            IsolationLevel::ReadCommitted => "Each read sees the latest committed data",
            IsolationLevel::RepeatableRead => {
                "All reads see a consistent snapshot from transaction start"
            }
        }
    }
}

/// Parse isolation level from string (SQL syntax).
impl std::str::FromStr for IsolationLevel {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().as_str() {
            "READ COMMITTED" | "READ_COMMITTED" | "READCOMMITTED" => {
                Ok(IsolationLevel::ReadCommitted)
            }
            "REPEATABLE READ" | "REPEATABLE_READ" | "REPEATABLEREAD" | "SNAPSHOT" => {
                Ok(IsolationLevel::RepeatableRead)
            }
            _ => Err(format!("unknown isolation level: {}", s)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_isolation() {
        assert_eq!(IsolationLevel::default(), IsolationLevel::ReadCommitted);
    }

    #[test]
    fn test_uses_snapshot() {
        assert!(!IsolationLevel::ReadCommitted.uses_snapshot());
        assert!(IsolationLevel::RepeatableRead.uses_snapshot());
    }

    #[test]
    fn test_parse_isolation() {
        assert_eq!(
            "READ COMMITTED".parse::<IsolationLevel>().unwrap(),
            IsolationLevel::ReadCommitted
        );
        assert_eq!(
            "REPEATABLE READ".parse::<IsolationLevel>().unwrap(),
            IsolationLevel::RepeatableRead
        );
        assert_eq!(
            "snapshot".parse::<IsolationLevel>().unwrap(),
            IsolationLevel::RepeatableRead
        );
    }
}
