use serde::{Deserialize, Serialize};

/// Database-agnostic data types for column definitions.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum DataType {
    /// Boolean value.
    Boolean,
    /// 8-bit signed integer.
    TinyInt,
    /// 16-bit signed integer.
    SmallInt,
    /// 32-bit signed integer.
    Integer,
    /// 64-bit signed integer.
    BigInt,
    /// Single-precision float.
    Float,
    /// Double-precision float.
    Double,
    /// Decimal with precision and scale.
    Decimal { precision: u32, scale: u32 },
    /// Fixed-length string.
    Char(u32),
    /// Variable-length string.
    Varchar(u32),
    /// Long text.
    Text,
    /// Binary data.
    Blob,
    /// Date.
    Date,
    /// Time.
    Time,
    /// Date and time.
    DateTime,
    /// Timestamp.
    Timestamp,
    /// JSON document.
    Json,
    /// UUID.
    Uuid,
    /// Enum with allowed values.
    Enum(Vec<String>),
    /// Auto-detected or custom type.
    Custom(String),
}

/// Represents a column definition in a table.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Column {
    /// Column name.
    pub name: String,
    /// Data type.
    pub data_type: DataType,
    /// Whether this column is a primary key.
    pub primary_key: bool,
    /// Whether this column should auto-increment.
    pub auto_increment: bool,
    /// Whether this column is NOT NULL.
    pub not_null: bool,
    /// Whether this column has a UNIQUE constraint.
    pub unique: bool,
    /// Optional default value expression.
    pub default: Option<String>,
    /// Optional comment.
    pub comment: Option<String>,
    /// Optional foreign key reference as `"table.column"`.
    pub foreign_key: Option<String>,
}

impl Column {
    /// Create a new column with the given name and data type.
    pub fn new(name: impl Into<String>, data_type: DataType) -> Self {
        Self {
            name: name.into(),
            data_type,
            primary_key: false,
            auto_increment: false,
            not_null: false,
            unique: false,
            default: None,
            comment: None,
            foreign_key: None,
        }
    }

    /// Mark as primary key.
    pub fn primary_key(mut self) -> Self {
        self.primary_key = true;
        self
    }

    /// Mark as auto-increment.
    pub fn auto_increment(mut self) -> Self {
        self.auto_increment = true;
        self
    }

    /// Mark as NOT NULL.
    pub fn not_null(mut self) -> Self {
        self.not_null = true;
        self
    }

    /// Mark as UNIQUE.
    pub fn unique(mut self) -> Self {
        self.unique = true;
        self
    }

    /// Set default value expression.
    pub fn default(mut self, value: impl Into<String>) -> Self {
        self.default = Some(value.into());
        self
    }

    /// Add a comment.
    pub fn comment(mut self, text: impl Into<String>) -> Self {
        self.comment = Some(text.into());
        self
    }

    /// Set a foreign key reference as `"table.column"`.
    pub fn foreign_key(mut self, reference: impl Into<String>) -> Self {
        self.foreign_key = Some(reference.into());
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn column_builder_chain() {
        let col = Column::new("user_id", DataType::BigInt)
            .primary_key()
            .auto_increment()
            .not_null()
            .comment("User primary key");

        assert_eq!(col.name, "user_id");
        assert_eq!(col.data_type, DataType::BigInt);
        assert!(col.primary_key);
        assert!(col.auto_increment);
        assert!(col.not_null);
        assert_eq!(col.comment.as_deref(), Some("User primary key"));
    }

    #[test]
    fn decimal_data_type() {
        let dt = DataType::Decimal {
            precision: 10,
            scale: 2,
        };
        let json = serde_json::to_string(&dt).unwrap();
        let deserialized: DataType = serde_json::from_str(&json).unwrap();
        assert_eq!(dt, deserialized);
    }

    #[test]
    fn enum_data_type() {
        let dt = DataType::Enum(vec![
            "active".into(),
            "inactive".into(),
            "banned".into(),
        ]);
        let json = serde_json::to_string(&dt).unwrap();
        let deserialized: DataType = serde_json::from_str(&json).unwrap();
        assert_eq!(dt, deserialized);
    }
}
