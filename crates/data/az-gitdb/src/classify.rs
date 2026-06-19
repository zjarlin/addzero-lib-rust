use anyhow::{Context, Result};
use gitdb::sql::{Parser, Statement};

/// High-level query kind used for cluster routing.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, derive_more::Display, strum::EnumString, strum::IntoStaticStr, strum::VariantArray)]
#[strum(serialize_all = "snake_case")]
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

impl GitDbQueryKind {
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
