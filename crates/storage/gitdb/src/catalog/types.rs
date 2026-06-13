//! GitDB schema 中的数据类型、列约束和列定义。

use std::fmt;

use az_derive_aliases::{
    apply, serde_code_display_props_enum, serde_code_partial_eq, serde_partial_eq,
};
use serde_json::Value;

/// GitDB 支持的 SQL 风格列类型。
///
/// `code()` 和 serde wire value 保持小写 snake_case，`Display`/`sql_name()` 保持 SQL 大写名称。
#[apply(serde_code_display_props_enum)]
pub enum DataType {
    /// 文本/字符串数据，对应 SQL 的 `TEXT`。
    #[display("TEXT")]
    #[strum(props(sql = "TEXT"))]
    Text,
    /// 整数数据，对应 SQL 的 `INTEGER`。
    #[display("INTEGER")]
    #[strum(props(sql = "INTEGER"))]
    Integer,
    /// 浮点数数据，对应 SQL 的 `REAL`。
    #[display("REAL")]
    #[strum(props(sql = "REAL"))]
    Float,
    /// 布尔值，对应 SQL 的 `BOOLEAN`。
    #[display("BOOLEAN")]
    #[strum(props(sql = "BOOLEAN"))]
    Boolean,
    /// JSON 对象或数组，对应 SQL 的 `JSON`。
    #[display("JSON")]
    #[strum(props(sql = "JSON"))]
    Json,
    /// 时间戳，当前以 ISO 8601 字符串存储，对应 SQL 的 `TIMESTAMP`。
    #[display("TIMESTAMP")]
    #[strum(props(sql = "TIMESTAMP"))]
    Timestamp,
    /// UUID，当前以字符串存储，对应 SQL 的 `UUID`。
    #[display("UUID")]
    #[strum(props(sql = "UUID"))]
    Uuid,
}

impl DataType {
    /// 判断 JSON 值是否符合当前列类型。
    pub fn matches(&self, value: &Value) -> bool {
        match (self, value) {
            (DataType::Text, Value::String(_)) => true,
            (DataType::Integer, Value::Number(n)) => n.is_i64() || n.is_u64(),
            (DataType::Float, Value::Number(_)) => true,
            (DataType::Boolean, Value::Bool(_)) => true,
            (DataType::Json, Value::Object(_) | Value::Array(_)) => true,
            (DataType::Timestamp, Value::String(s)) => {
                // 只做轻量格式守卫；真正的时区和精度语义留给上层字段约束。
                chrono::DateTime::parse_from_rfc3339(s).is_ok()
                    || chrono::NaiveDateTime::parse_from_str(s, "%Y-%m-%dT%H:%M:%S").is_ok()
            }
            (DataType::Uuid, Value::String(s)) => {
                // 这里守住 8-4-4-4-12 的基础形状，不在 catalog 层做完整 UUID 解析。
                s.len() == 36 && s.chars().filter(|c| *c == '-').count() == 4
            }
            _ => false,
        }
    }

    /// 返回用于 SQL 展示和 DDL 拼装的大写类型名。
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

/// 列约束。
#[apply(serde_code_partial_eq)]
#[derive(derive_more::Display)]
pub enum Constraint {
    /// 列值不能为空。
    #[display("NOT NULL")]
    NotNull,
    /// 列值在所有行中必须唯一。
    #[display("UNIQUE")]
    Unique,
    /// 主键约束，语义上同时包含非空和唯一。
    #[display("PRIMARY KEY")]
    PrimaryKey,
    /// 列默认值。
    #[display("DEFAULT {_0}")]
    Default(Value),
    /// CHECK 约束，当前表达式以字符串形式保存。
    #[display("CHECK ({_0})")]
    Check(String),
}

impl Constraint {
    /// 判断约束是否要求非空。
    pub fn is_not_null(&self) -> bool {
        matches!(self, Constraint::NotNull | Constraint::PrimaryKey)
    }

    /// 判断约束是否要求唯一。
    pub fn is_unique(&self) -> bool {
        matches!(self, Constraint::Unique | Constraint::PrimaryKey)
    }

    /// 返回约束的 SQL 片段展示。
    pub fn sql_name(&self) -> String {
        self.to_string()
    }
}

/// 完整列定义，包含列名、类型、约束和可选说明。
#[apply(serde_partial_eq)]
pub struct ColumnDef {
    /// 列名。
    pub name: String,
    /// 列数据类型。
    pub data_type: DataType,
    /// 作用在该列上的约束列表。
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub constraints: Vec<Constraint>,
    /// 可选列说明。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

impl ColumnDef {
    /// 创建不带约束的列定义。
    pub fn new(name: impl Into<String>, data_type: DataType) -> Self {
        Self {
            name: name.into(),
            data_type,
            constraints: Vec::new(),
            description: None,
        }
    }

    /// 以 builder 风格追加一个列约束。
    pub fn with_constraint(mut self, constraint: Constraint) -> Self {
        self.constraints.push(constraint);
        self
    }

    /// 设置列说明。
    pub fn with_description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }

    /// 判断该列是否允许为空。
    pub fn is_nullable(&self) -> bool {
        !self.constraints.iter().any(|c| c.is_not_null())
    }

    /// 判断该列是否必须唯一。
    pub fn is_unique(&self) -> bool {
        self.constraints.iter().any(|c| c.is_unique())
    }

    /// 返回列默认值；没有 `DEFAULT` 约束时返回 `None`。
    pub fn default_value(&self) -> Option<&Value> {
        self.constraints.iter().find_map(|c| {
            if let Constraint::Default(v) = c {
                Some(v)
            } else {
                None
            }
        })
    }

    /// 按列类型、非空约束和默认值约束校验输入值。
    pub fn validate(&self, value: Option<&Value>) -> Result<(), String> {
        match value {
            Some(v) => {
                if !self.data_type.matches(v) {
                    let message = format!(
                        "column '{}' expects type {}, got {:?}",
                        self.name, self.data_type, v
                    );

                    return Err(message);
                }
                Ok(())
            }
            None => {
                if !self.is_nullable() && self.default_value().is_none() {
                    let message = format!("column '{}' cannot be null", self.name);

                    return Err(message);
                }
                Ok(())
            }
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
        assert!(!DataType::Integer.matches(&json!(3.5)));

        assert!(DataType::Float.matches(&json!(3.5)));
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
    fn constraint_display_matches_sql_name() {
        let cases = [
            (Constraint::NotNull, "NOT NULL"),
            (Constraint::Unique, "UNIQUE"),
            (Constraint::PrimaryKey, "PRIMARY KEY"),
            (Constraint::Default(json!(1)), "DEFAULT 1"),
            (Constraint::Check("x > 0".to_owned()), "CHECK (x > 0)"),
        ];

        for (constraint, expected) in cases {
            assert_eq!(constraint.sql_name(), expected);
            assert_eq!(constraint.to_string(), expected);
        }
    }

    #[test]
    fn constraint_serde_uses_snake_case_wire_values() {
        assert_eq!(
            serde_json::to_value(Constraint::NotNull).unwrap(),
            json!("not_null")
        );
        assert_eq!(
            serde_json::from_str::<Constraint>("\"primary_key\"").unwrap(),
            Constraint::PrimaryKey
        );
        assert_eq!(
            serde_json::to_value(Constraint::Default(json!(1))).unwrap(),
            json!({ "default": 1 })
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
