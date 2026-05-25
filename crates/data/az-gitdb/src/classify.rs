use az_derive_aliases::{apply, plain_copy_eq};
use gitdb::sql::{Parser, Statement};

use crate::error::{GitDbClusterError, GitDbClusterResult};

/// High-level query kind used for cluster routing.
#[apply(plain_copy_eq)]
pub enum GitDbQueryKind {
    /// Read-only query.
    Read,
    /// Mutating query.
    Write,
    /// Transaction control statement.
    TransactionControl,
}

/// Classify SQL by parsing it with upstream GitDB's parser.
pub fn classify_gitdb_query(sql: &str) -> GitDbClusterResult<GitDbQueryKind> {
    let statement = Parser::parse(sql).map_err(GitDbClusterError::Parse)?;

    Ok(match statement {
        Statement::Select(_) | Statement::ShowTables | Statement::Describe(_) => {
            GitDbQueryKind::Read
        }
        Statement::CreateTable(_)
        | Statement::DropTable(_)
        | Statement::Insert(_)
        | Statement::Update(_)
        | Statement::Delete(_) => GitDbQueryKind::Write,
        Statement::Begin | Statement::Commit | Statement::Rollback => {
            GitDbQueryKind::TransactionControl
        }
    })
}
