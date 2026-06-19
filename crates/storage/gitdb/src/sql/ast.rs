//! GitDB SQL 的内部 AST 类型。
//!
//! 本模块只表达执行器当前能理解的 SQL 子集，不承诺覆盖完整 SQL 标准。
//! 解析器负责把外部 SQL 文本转换为这些结构，后续 planner/executor
//! 只依赖这里的稳定形状。

use serde_json::Value;

/// 已解析的 SQL 语句。
#[derive(Clone, Debug, PartialEq)]
pub enum Statement {
    /// `CREATE TABLE` 建表语句。
    CreateTable(CreateTable),
    /// `DROP TABLE` 删表语句。
    DropTable(DropTable),
    /// `SELECT` 查询语句。
    Select(Select),
    /// `INSERT` 插入语句。
    Insert(Insert),
    /// `UPDATE` 更新语句。
    Update(Update),
    /// `DELETE` 删除语句。
    Delete(Delete),
    /// `BEGIN`，开启事务。
    Begin,
    /// `COMMIT`，提交当前事务。
    Commit,
    /// `ROLLBACK`，回滚当前事务。
    Rollback,
    /// `SHOW TABLES`，列出表。
    ShowTables,
    /// `DESCRIBE table`，查看表结构。
    Describe(String),
}

/// `CREATE TABLE` 的内部表示。
#[derive(Clone, Debug, PartialEq)]
pub struct CreateTable {
    pub name: String,
    pub columns: Vec<ColumnDef>,
    pub if_not_exists: bool,
}

/// `CREATE TABLE` 中的一列定义。
#[derive(Clone, Debug, PartialEq)]
pub struct ColumnDef {
    pub name: String,
    pub data_type: SqlDataType,
    pub constraints: Vec<ColumnConstraint>,
}

/// GitDB 当前支持的 SQL 数据类型。
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SqlDataType {
    Text,
    Integer,
    Float,
    Boolean,
    Json,
    Timestamp,
    Uuid,
}

/// 列约束定义。
#[derive(Clone, Debug, PartialEq)]
pub enum ColumnConstraint {
    NotNull,
    Unique,
    PrimaryKey,
    Default(Expr),
}

/// `DROP TABLE` 的内部表示。
#[derive(Clone, Debug, PartialEq)]
pub struct DropTable {
    pub name: String,
    pub if_exists: bool,
}

/// `SELECT` 查询的内部表示。
#[derive(Clone, Debug, PartialEq)]
pub struct Select {
    pub columns: Vec<SelectColumn>,
    pub from: String,
    pub where_clause: Option<Expr>,
    pub order_by: Vec<OrderBy>,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
}

/// `SELECT` 子句中的投影列。
#[derive(Clone, Debug, PartialEq)]
pub enum SelectColumn {
    /// SELECT *
    Wildcard,
    /// SELECT column_name
    Column(String),
    /// SELECT expr AS alias
    Expr { expr: Expr, alias: Option<String> },
}

/// `ORDER BY` 子句中的一个排序项。
#[derive(Clone, Debug, PartialEq)]
pub struct OrderBy {
    pub column: String,
    pub ascending: bool,
}

/// `INSERT` 语句的内部表示。
#[derive(Clone, Debug, PartialEq)]
pub struct Insert {
    pub table: String,
    pub columns: Option<Vec<String>>,
    pub values: Vec<Vec<Expr>>,
}

/// `UPDATE` 语句的内部表示。
#[derive(Clone, Debug, PartialEq)]
pub struct Update {
    pub table: String,
    pub assignments: Vec<Assignment>,
    pub where_clause: Option<Expr>,
}

/// `SET` 子句中的单列赋值。
#[derive(Clone, Debug, PartialEq)]
pub struct Assignment {
    pub column: String,
    pub value: Expr,
}

/// `DELETE` 语句的内部表示。
#[derive(Clone, Debug, PartialEq)]
pub struct Delete {
    pub table: String,
    pub where_clause: Option<Expr>,
}

/// SQL 表达式树。
#[derive(Clone, Debug, PartialEq)]
pub enum Expr {
    /// 列引用。
    Column(String),
    /// 字面量值。
    Literal(LiteralValue),
    /// 二元运算，例如 `a = b` 或 `a AND b`。
    BinaryOp {
        left: Box<Expr>,
        op: BinaryOperator,
        right: Box<Expr>,
    },
    /// 一元运算，例如 `NOT a` 或 `-x`。
    UnaryOp { op: UnaryOperator, expr: Box<Expr> },
    /// `IS NULL` / `IS NOT NULL`。
    IsNull { expr: Box<Expr>, negated: bool },
    /// `IN (...)` 列表判断。
    InList {
        expr: Box<Expr>,
        list: Vec<Expr>,
        negated: bool,
    },
    /// `BETWEEN a AND b` 范围判断。
    Between {
        expr: Box<Expr>,
        low: Box<Expr>,
        high: Box<Expr>,
        negated: bool,
    },
    /// `LIKE` 模式匹配。
    Like {
        expr: Box<Expr>,
        pattern: String,
        negated: bool,
    },
    /// 函数调用。
    Function { name: String, args: Vec<Expr> },
    /// 括号中的嵌套表达式。
    Nested(Box<Expr>),
}

/// 字面量值。
#[derive(Clone, Debug, PartialEq)]
pub enum LiteralValue {
    Null,
    Boolean(bool),
    Integer(i64),
    Float(f64),
    String(String),
    Json(Value),
}

impl LiteralValue {
    /// 转换为存储层使用的 JSON 值。
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

/// SQL 二元运算符。
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
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
    /// 判断是否为比较运算符。
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

    /// 判断是否为逻辑运算符。
    pub fn is_logical(&self) -> bool {
        matches!(self, BinaryOperator::And | BinaryOperator::Or)
    }
}

/// SQL 一元运算符。
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UnaryOperator {
    Not,
    Minus,
    Plus,
}
