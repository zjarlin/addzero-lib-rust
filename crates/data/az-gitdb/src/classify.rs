use az_derive_aliases::{apply, plain_code_display_no_default_enum};
use anyhow::{Context, Result};
use gitdb::sql::{Parser, Statement};

/// High-level query kind used for cluster routing.
#[apply(plain_code_display_no_default_enum)]
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

#[cfg(test)]
mod tests {
    use super::GitDbQueryKind;

    #[test]
    fn query_kind_code_is_snake_case() {
        assert_eq!(GitDbQueryKind::Read.code(), "read");
        assert_eq!(
            GitDbQueryKind::from_code("transaction_control"),
            Some(GitDbQueryKind::TransactionControl)
        );
    }

    #[test]
    fn query_kind_display_keeps_human_readable_labels() {
        assert_eq!(GitDbQueryKind::Read.to_string(), "read");
        assert_eq!(
            GitDbQueryKind::TransactionControl.to_string(),
            "transaction-control"
        );
    }
}

/// Classify SQL by parsing it with upstream GitDB's parser.
pub fn classify_gitdb_query(sql: &str) -> Result<GitDbQueryKind> {
    let statement = Parser::parse(sql).context("failed to classify GitDB SQL")?;

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
