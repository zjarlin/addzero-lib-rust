use serde::{Deserialize, Serialize};

/// Represents a database index on a table.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Index {
    /// Index name.
    pub name: String,
    /// The table this index belongs to.
    pub table: String,
    /// Column names included in this index.
    pub columns: Vec<String>,
    /// Whether this is a unique index.
    pub unique: bool,
}

impl Index {
    /// Create a new non-unique index.
    pub fn new(
        name: impl Into<String>,
        table: impl Into<String>,
        columns: Vec<String>,
    ) -> Self {
        Self {
            name: name.into(),
            table: table.into(),
            columns,
            unique: false,
        }
    }

    /// Create a new unique index.
    pub fn unique(
        name: impl Into<String>,
        table: impl Into<String>,
        columns: Vec<String>,
    ) -> Self {
        Self {
            name: name.into(),
            table: table.into(),
            columns,
            unique: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn index_builder() {
        let idx = Index::new("idx_user_email", "users", vec!["email".into()]);
        assert_eq!(idx.name, "idx_user_email");
        assert_eq!(idx.table, "users");
        assert_eq!(idx.columns, vec!["email"]);
        assert!(!idx.unique);
    }

    #[test]
    fn unique_index() {
        let idx = Index::unique("uq_user_email", "users", vec!["email".into()]);
        assert!(idx.unique);
    }

    #[test]
    fn composite_index() {
        let idx = Index::new(
            "idx_order_user_date",
            "orders",
            vec!["user_id".into(), "created_at".into()],
        );
        assert_eq!(idx.columns.len(), 2);
    }

    #[test]
    fn index_serialization_roundtrip() {
        let idx = Index::unique("uq_name", "tags", vec!["name".into()]);
        let json = serde_json::to_string(&idx).unwrap();
        let deserialized: Index = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.name, "uq_name");
        assert!(deserialized.unique);
    }
}
