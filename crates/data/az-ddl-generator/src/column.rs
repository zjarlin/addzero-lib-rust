use serde::{Deserialize, Serialize};

/// Supported SQL column types.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ColumnType {
    /// Boolean type.
    Boolean,
    /// Signed 32-bit integer.
    Integer,
    /// Signed 64-bit integer.
    BigInt,
    /// Floating-point number.
    Float,
    /// Double-precision floating-point.
    Double,
    /// Fixed-length character string.
    Char(u32),
    /// Variable-length character string with max length.
    Varchar(u32),
    /// Unlimited-length text.
    Text,
    /// Binary large object.
    Blob,
    /// Date without time.
    Date,
    /// Date and time.
    DateTime,
    /// Timestamp with timezone.
    Timestamp,
    /// JSON document.
    Json,
    /// UUID value.
    Uuid,
}

/// Represents a single column in a table.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Column {
    /// Column name.
    pub name: String,
    /// Column data type.
    pub column_type: ColumnType,
    /// Whether the column is a primary key.
    pub primary_key: bool,
    /// Whether the column is NOT NULL.
    pub not_null: bool,
    /// Whether the column has a UNIQUE constraint.
    pub unique: bool,
    /// Optional default value expression.
    pub default: Option<String>,
    /// Optional comment.
    pub comment: Option<String>,
}

impl Column {
    /// Create a new column with the given name and type.
    pub fn new(name: impl Into<String>, column_type: ColumnType) -> Self {
        Self {
            name: name.into(),
            column_type,
            primary_key: false,
            not_null: false,
            unique: false,
            default: None,
            comment: None,
        }
    }

    /// Mark this column as a primary key.
    pub fn primary_key(mut self) -> Self {
        self.primary_key = true;
        self
    }

    /// Mark this column as NOT NULL.
    pub fn not_null(mut self) -> Self {
        self.not_null = true;
        self
    }

    /// Mark this column as UNIQUE.
    pub fn unique(mut self) -> Self {
        self.unique = true;
        self
    }

    /// Set a default value expression.
    pub fn default(mut self, value: impl Into<String>) -> Self {
        self.default = Some(value.into());
        self
    }

    /// Add a comment to this column.
    pub fn comment(mut self, text: impl Into<String>) -> Self {
        self.comment = Some(text.into());
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn column_builder_chain() {
        let col = Column::new("id", ColumnType::BigInt)
            .primary_key()
            .not_null()
            .default("0")
            .comment("primary key");

        assert_eq!(col.name, "id");
        assert_eq!(col.column_type, ColumnType::BigInt);
        assert!(col.primary_key);
        assert!(col.not_null);
        assert_eq!(col.default.as_deref(), Some("0"));
        assert_eq!(col.comment.as_deref(), Some("primary key"));
    }

    #[test]
    fn column_type_serialization_roundtrip() {
        let col_type = ColumnType::Varchar(255);
        let json = serde_json::to_string(&col_type).unwrap();
        let deserialized: ColumnType = serde_json::from_str(&json).unwrap();
        assert_eq!(col_type, deserialized);
    }

    #[test]
    fn column_serialization_roundtrip() {
        let col = Column::new("email", ColumnType::Varchar(255))
            .unique()
            .not_null();
        let json = serde_json::to_string(&col).unwrap();
        let deserialized: Column = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.name, "email");
        assert!(deserialized.unique);
        assert!(deserialized.not_null);
    }
}
