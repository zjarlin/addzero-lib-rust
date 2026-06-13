//! 用于代码生成的数据库模型和 Schema 定义。
//!
//! 以方言无关的方式提供数据结构，用于表示数据库 Schema、表、列、
//! 关系和索引。
//!
//! # 快速开始
//!
//! ```no_run
//! use az_database_model::column::{Column, DataType};
//! use az_database_model::schema::Schema;
//! use az_database_model::table::Table;
//!
//! let users = Table::new("users")
//!     .column(Column::new("id", DataType::BigInt).primary_key().auto_increment())
//!     .column(Column::new("name", DataType::Varchar(255)).not_null())
//!     .column(Column::new("email", DataType::Varchar(255)).unique());
//!
//! let schema = Schema::new("myapp").table(users);
//!
//! assert_eq!(schema.tables.len(), 1);
//! assert_eq!(schema.tables[0].name, "users");
//! ```

automod::dir!(pub "src");
