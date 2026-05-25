//! Data types and constraints for schema definitions.

use std::fmt;

use az_derive_aliases::{apply, serde_code_display_enum, serde_partial_eq};
use serde_json::Value;

/// SQL-like data types supported by GitDB.
#[apply(serde_code_display_enum)]
pub enum DataType {
    /// Text/string data (VARCHAR in SQL).
    #[display("TEXT")]
    Text,
    /// Integer numbers (BIGINT in SQL).
    #[display("INTEGER")]
    Integer,
    /// Floating point numbers (DOUBLE in SQL).
    #[display("REAL")]
    Float,
    /// Boolean values.
    #[display("BOOLEAN")]
    Boolean,
    /// JSON objects or arrays.
    #[display("JSON")]
    Json,
    /// Timestamps (stored as ISO 8601 strings).
    #[display("TIMESTAMP")]
    Timestamp,
    /// UUIDs (stored as strings).
    #[display("UUID")]
    Uuid,
}

impl DataType {
    /// Check if a JSON value matches this data type.
    pub fn matches(&self, value: &Value) -> bool {
        match (self, value) {
            (DataType::Text, Value::String(_)) => true,
            (DataType::Integer, Value::Number(n)) => n.is_i64() || n.is_u64(),
            (DataType::Float, Value::Number(_)) => true,
            (DataType::Boolean, Value::Bool(_)) => true,
            (DataType::Json, Value::Object(_) | Value::Array(_)) => true,
            (DataType::Timestamp, Value::String(s)) => {
                // Basic ISO 8601 check
                chrono::DateTime::parse_from_rfc3339(s).is_ok()
                    || chrono::NaiveDateTime::parse_from_str(s, "%Y-%m-%dT%H:%M:%S").is_ok()
            }
            (DataType::Uuid, Value::String(s)) => {
                // Basic UUID format check (8-4-4-4-12)
                s.len() == 36 && s.chars().filter(|c| *c == '-').count() == 4
            }
            _ => false,
        }
    }

    /// Get the SQL name for this type.
    pub fn sql_name(&self) -> &'static str {
        match self {
            DataType::Text => "TEXT",
            DataType::Integer => "INTEGER",
            DataType::Float => "REAL",
            DataType::Boolean => "BOOLEAN",
            DataType::Json => "JSON",
            DataType::Timestamp => "TIMESTAMP",
            DataType::Uuid => "UUID",
        }
    }
}

/// Column constraints.
#[apply(serde_partial_eq)]
#[serde(rename_all = "snake_case")]
pub enum Constraint {
    /// Column cannot be null.
    NotNull,
    /// Column values must be unique across all rows.
    Unique,
    /// Column is the primary key (implies NotNull + Unique).
    PrimaryKey,
    /// Default value for the column.
    Default(Value),
    /// Check constraint (expression stored as string for now).
    Check(String),
}

impl Constraint {
    /// Check if this is a NOT NULL constraint.
    pub fn is_not_null(&self) -> bool {
        matches!(self, Constraint::NotNull | Constraint::PrimaryKey)
    }

    /// Check if this is a UNIQUE constraint.
    pub fn is_unique(&self) -> bool {
        matches!(self, Constraint::Unique | Constraint::PrimaryKey)
    }

    /// Get the SQL representation of this constraint.
    pub fn sql_name(&self) -> String {
        match self {
            Constraint::NotNull => "NOT NULL".to_string(),
            Constraint::Unique => "UNIQUE".to_string(),
            Constraint::PrimaryKey => "PRIMARY KEY".to_string(),
            Constraint::Default(v) => format!("DEFAULT {}", v),
            Constraint::Check(expr) => format!("CHECK ({})", expr),
        }
    }
}

/// Full column definition including name, type, and constraints.
#[apply(serde_partial_eq)]
pub struct ColumnDef {
    /// Column name.
    pub name: String,
    /// Data type.
    pub data_type: DataType,
    /// Constraints on this column.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub constraints: Vec<Constraint>,
    /// Optional column description/comment.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

impl ColumnDef {
    /// Create a new column definition.
    pub fn new(name: impl Into<String>, data_type: DataType) -> Self {
        Self {
            name: name.into(),
            data_type,
            constraints: Vec::new(),
            description: None,
        }
    }

    /// Add a constraint to this column.
    pub fn with_constraint(mut self, constraint: Constraint) -> Self {
        self.constraints.push(constraint);
        self
    }

    /// Set the description.
    pub fn with_description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }

    /// Check if this column is nullable.
    pub fn is_nullable(&self) -> bool {
        !self.constraints.iter().any(|c| c.is_not_null())
    }

    /// Check if this column must be unique.
    pub fn is_unique(&self) -> bool {
        self.constraints.iter().any(|c| c.is_unique())
    }

    /// Get the default value, if any.
    pub fn default_value(&self) -> Option<&Value> {
        self.constraints.iter().find_map(|c| {
            if let Constraint::Default(v) = c {
                Some(v)
            } else {
                None
            }
        })
    }

    /// Validate a value against this column definition.
    pub fn validate(&self, value: Option<&Value>) -> Result<(), String> {
        match value {
            Some(v) => {
                if !self.data_type.matches(v) {
                    return Err(format!(
                        "column '{}' expects type {}, got {:?}",
                        self.name, self.data_type, v
                    ));
                }
                Ok(())
            }
            None => {
                if !self.is_nullable() && self.default_value().is_none() {
                    return Err(format!("column '{}' cannot be null", self.name));
                }
                Ok(())
            }
        }
    }
}

impl fmt::Display for Constraint {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Constraint::NotNull => write!(f, "NOT NULL"),
            Constraint::Unique => write!(f, "UNIQUE"),
            Constraint::PrimaryKey => write!(f, "PRIMARY KEY"),
            Constraint::Default(v) => write!(f, "DEFAULT {}", v),
            Constraint::Check(expr) => write!(f, "CHECK ({})", expr),
        }
    }
}

impl fmt::Display for ColumnDef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {}", self.name, self.data_type)?;
        for constraint in &self.constraints {
            write!(f, " {}", constraint)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_data_type_matches() {
        assert!(DataType::Text.matches(&json!("hello")));
        assert!(!DataType::Text.matches(&json!(123)));

        assert!(DataType::Integer.matches(&json!(42)));
        assert!(DataType::Integer.matches(&json!(-17)));
        assert!(!DataType::Integer.matches(&json!(3.14)));

        assert!(DataType::Float.matches(&json!(3.14)));
        assert!(DataType::Float.matches(&json!(42)));

        assert!(DataType::Boolean.matches(&json!(true)));
        assert!(!DataType::Boolean.matches(&json!("true")));

        assert!(DataType::Json.matches(&json!({"key": "value"})));
        assert!(DataType::Json.matches(&json!([1, 2, 3])));
    }

    #[test]
    fn data_type_code_and_sql_display_are_separate() {
        assert_eq!(DataType::Text.code(), "text");
        assert_eq!(DataType::Text.to_string(), "TEXT");
        assert_eq!(
            serde_json::to_string(&DataType::Timestamp).expect("serialize"),
            "\"timestamp\""
        );
    }

    #[test]
    fn test_column_validation() {
        let col = ColumnDef::new("name", DataType::Text).with_constraint(Constraint::NotNull);

        assert!(col.validate(Some(&json!("Alice"))).is_ok());
        assert!(col.validate(Some(&json!(123))).is_err());
        assert!(col.validate(None).is_err());

        let nullable_col = ColumnDef::new("nickname", DataType::Text);
        assert!(nullable_col.validate(None).is_ok());
    }

    #[test]
    fn test_column_with_default() {
        let col = ColumnDef::new("status", DataType::Text)
            .with_constraint(Constraint::NotNull)
            .with_constraint(Constraint::Default(json!("active")));

        assert!(col.validate(None).is_ok()); // Has default
        assert_eq!(col.default_value(), Some(&json!("active")));
    }
}
