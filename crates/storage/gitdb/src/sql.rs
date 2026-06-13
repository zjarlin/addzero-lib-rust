//! SQL parsing and AST types for GitDB.
//!
//! Uses `sqlparser` crate for parsing, then converts to our internal AST
//! representation for execution.

automod::dir!("src/sql");

pub use ast::{
    Assignment, BinaryOperator, ColumnConstraint, ColumnDef, CreateTable, Delete, DropTable, Expr,
    Insert, LiteralValue, OrderBy, Select, SelectColumn, SqlDataType, Statement, UnaryOperator,
    Update,
};
pub use parser::Parser;
