//! 带类型安全 SQL 构建的 SQL 查询构建器。
//!
//! 提供流式 API，用于构建 SELECT、INSERT、UPDATE 和 DELETE 查询，
//! 并使用参数化值来防止 SQL 注入。
//!
//! # 快速开始
//!
//! ```
//! use az_sql::query::Query;
//! use az_sql::select::SelectQuery;
//!
//! fn main() -> anyhow::Result<()> {
//!
//! let query = SelectQuery::new()
//!     .select(&["id", "name", "email"])
//!     .from("users")
//!     .r#where("active = ?", vec!["true"])
//!     .order_by("name", true)
//!     .limit(10);
//!
//! let (sql, params) = query.build()?;
//! assert!(sql.contains(r#"SELECT "id", "name", "email""#));
//! assert!(sql.contains(r#"FROM "users""#));
//! # let _ = params;
//! # Ok(())
//! # }
//! ```

automod::dir!(pub "src");
