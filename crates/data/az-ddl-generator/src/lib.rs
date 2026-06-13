//! 支持多种数据库方言的 DDL 语句生成器。
//!
//! 提供类型安全的 API，用于在不同 SQL 方言（MySQL、PostgreSQL、SQLite）
//! 之间生成 `CREATE TABLE`、`ALTER TABLE`、`CREATE INDEX` 及其他 DDL 语句。
//!
//! # 快速开始
//!
//! ```no_run
//! use az_ddl_generator::column::{Column, ColumnType};
//! use az_ddl_generator::dialect::Dialect;
//! use az_ddl_generator::generator::DdlGenerator;
//! use az_ddl_generator::table::Table;
//!
//! # fn main() -> anyhow::Result<()> {
//! let table = Table::new("users")
//!     .column(Column::new("id", ColumnType::BigInt).primary_key().not_null())
//!     .column(Column::new("name", ColumnType::Varchar(255)).not_null())
//!     .column(Column::new("email", ColumnType::Varchar(255)).unique());
//!
//! let ddl = DdlGenerator::new(Dialect::PostgreSQL).generate_create_table(&table)?;
//! assert!(ddl.contains("CREATE TABLE"));
//! assert!(ddl.contains("users"));
//! # Ok(())
//! # }
//! ```

automod::dir!(pub "src");
