//! Internal AST types for GitDB SQL.
//!
//! These types are simplified representations of SQL statements
//! that the query executor understands.

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// A parsed SQL statement.
#[derive(Debug, Clone, PartialEq)]
pub enum Statement {
    /// CREATE TABLE statement.
    CreateTable(CreateTable),
    /// DROP TABLE statement.
    DropTable(DropTable),
    /// SELECT statement.
    Select(Select),
    /// INSERT statement.
    Insert(Insert),
    /// UPDATE statement.
    Update(Update),
    /// DELETE statement.
    Delete(Delete),
    /// BEGIN TRANSACTION.
    Begin,
    /// COMMIT.
    Commit,
    /// ROLLBACK.
    Rollback,
    /// SHOW TABLES.
    ShowTables,
    /// DESCRIBE table.
    Describe(String),
}

/// CREATE TABLE statement.
#[derive(Debug, Clone, PartialEq)]
pub struct CreateTable {
    pub name: String,
    pub columns: Vec<ColumnDef>,
    pub if_not_exists: bool,
}

/// Column definition in CREATE TABLE.
#[derive(Debug, Clone, PartialEq)]
pub struct ColumnDef {
    pub name: String,
    pub data_type: SqlDataType,
    pub constraints: Vec<ColumnConstraint>,
}

/// SQL data types.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SqlDataType {
    Text,
    Integer,
    Float,
    Boolean,
    Json,
    Timestamp,
    Uuid,
}

/// Column constraints.
#[derive(Debug, Clone, PartialEq)]
pub enum ColumnConstraint {
    NotNull,
    Unique,
    PrimaryKey,
    Default(Expr),
}

/// DROP TABLE statement.
#[derive(Debug, Clone, PartialEq)]
pub struct DropTable {
    pub name: String,
    pub if_exists: bool,
}

/// SELECT statement.
#[derive(Debug, Clone, PartialEq)]
pub struct Select {
    pub columns: Vec<SelectColumn>,
    pub from: String,
    pub where_clause: Option<Expr>,
    pub order_by: Vec<OrderBy>,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
}

/// A column in SELECT clause.
#[derive(Debug, Clone, PartialEq)]
pub enum SelectColumn {
    /// SELECT *
    Wildcard,
    /// SELECT column_name
    Column(String),
    /// SELECT expr AS alias
    Expr { expr: Expr, alias: Option<String> },
}

/// ORDER BY clause item.
#[derive(Debug, Clone, PartialEq)]
pub struct OrderBy {
    pub column: String,
    pub ascending: bool,
}

/// INSERT statement.
#[derive(Debug, Clone, PartialEq)]
pub struct Insert {
    pub table: String,
    pub columns: Option<Vec<String>>,
    pub values: Vec<Vec<Expr>>,
}

/// UPDATE statement.
#[derive(Debug, Clone, PartialEq)]
pub struct Update {
    pub table: String,
    pub assignments: Vec<Assignment>,
    pub where_clause: Option<Expr>,
}

/// SET clause assignment.
#[derive(Debug, Clone, PartialEq)]
pub struct Assignment {
    pub column: String,
    pub value: Expr,
}

/// DELETE statement.
#[derive(Debug, Clone, PartialEq)]
pub struct Delete {
    pub table: String,
    pub where_clause: Option<Expr>,
}

/// SQL expression.
#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    /// Column reference.
    Column(String),
    /// Literal value.
    Literal(LiteralValue),
    /// Binary operation (e.g., a = b, a AND b).
    BinaryOp {
        left: Box<Expr>,
        op: BinaryOperator,
        right: Box<Expr>,
    },
    /// Unary operation (e.g., NOT a, -x).
    UnaryOp { op: UnaryOperator, expr: Box<Expr> },
    /// IS NULL / IS NOT NULL.
    IsNull { expr: Box<Expr>, negated: bool },
    /// IN list.
    InList {
        expr: Box<Expr>,
        list: Vec<Expr>,
        negated: bool,
    },
    /// BETWEEN a AND b.
    Between {
        expr: Box<Expr>,
        low: Box<Expr>,
        high: Box<Expr>,
        negated: bool,
    },
    /// LIKE pattern.
    Like {
        expr: Box<Expr>,
        pattern: String,
        negated: bool,
    },
    /// Function call.
    Function { name: String, args: Vec<Expr> },
    /// Nested expression in parentheses.
    Nested(Box<Expr>),
}

/// Literal value.
#[derive(Debug, Clone, PartialEq)]
pub enum LiteralValue {
    Null,
    Boolean(bool),
    Integer(i64),
    Float(f64),
    String(String),
    Json(Value),
}

impl LiteralValue {
    /// Convert to JSON value for storage.
    pub fn to_json(&self) -> Value {
        match self {
            LiteralValue::Null => Value::Null,
            LiteralValue::Boolean(b) => Value::Bool(*b),
            LiteralValue::Integer(n) => Value::Number((*n).into()),
            LiteralValue::Float(f) => serde_json::Number::from_f64(*f)
                .map(Value::Number)
                .unwrap_or(Value::Null),
            LiteralValue::String(s) => Value::String(s.clone()),
            LiteralValue::Json(v) => v.clone(),
        }
    }
}

/// Binary operators.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinaryOperator {
    // Comparison
    Eq,
    NotEq,
    Lt,
    LtEq,
    Gt,
    GtEq,
    // Logical
    And,
    Or,
    // Arithmetic
    Plus,
    Minus,
    Multiply,
    Divide,
    Modulo,
    // String
    Concat,
}

impl BinaryOperator {
    /// Check if this is a comparison operator.
    pub fn is_comparison(&self) -> bool {
        matches!(
            self,
            BinaryOperator::Eq
                | BinaryOperator::NotEq
                | BinaryOperator::Lt
                | BinaryOperator::LtEq
                | BinaryOperator::Gt
                | BinaryOperator::GtEq
        )
    }

    /// Check if this is a logical operator.
    pub fn is_logical(&self) -> bool {
        matches!(self, BinaryOperator::And | BinaryOperator::Or)
    }
}

/// Unary operators.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnaryOperator {
    Not,
    Minus,
    Plus,
}
