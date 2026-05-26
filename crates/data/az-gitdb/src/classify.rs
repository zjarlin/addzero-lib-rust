use az_derive_aliases::{apply, plain_copy_eq_display};
use gitdb::sql::{Parser, Statement};

use crate::error::{GitDbClusterError, GitDbClusterResult};

/// High-level query kind used for cluster routing.
#[apply(plain_copy_eq_display)]
pub enum GitDbQueryKind {
    /// Read-only query.
    #[display("read")]
    Read,
    /// Mutating query.
    #[display("write")]
    Write,
    /// Transaction control statement.
    #[display("transaction-control")]
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
