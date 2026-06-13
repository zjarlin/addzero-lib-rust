# az-ddl-generator

支持多种数据库方言的 DDL 语句生成器，提供类型安全的 API 构建建表、改表、建索引等 SQL 语句。

## 功能

- 支持 MySQL、PostgreSQL、SQLite 三种主流数据库方言
- 生成 `CREATE TABLE`、`ALTER TABLE`、`CREATE INDEX` 等 DDL 语句
- 类型安全的 `Table`、`Column`、`ColumnType` 构建器
- 列级别的主键、非空、唯一等约束支持
- 标识符自动转义（方言感知的引号处理）

## 安装

在 `Cargo.toml` 中添加：

```toml
[dependencies]
az-ddl-generator = { path = "../az-ddl-generator" }       # workspace 内部引用
# 或发布后：
# az-ddl-generator = "0.1"                                 # crates.io 引用
```

## 用法

```rust
use az_ddl_generator::column::{Column, ColumnType};
use az_ddl_generator::dialect::Dialect;
use az_ddl_generator::generator::DdlGenerator;
use az_ddl_generator::table::Table;

let table = Table::new("users")
    .column(Column::new("id", ColumnType::BigInt).primary_key().not_null())
    .column(Column::new("name", ColumnType::Varchar(255)).not_null())
    .column(Column::new("email", ColumnType::Varchar(255)).unique());

let ddl = DdlGenerator::new(Dialect::PostgreSQL)
    .generate_create_table(&table)
    .unwrap();
// => "CREATE TABLE ..."
```

## 依赖的 crates

- `anyhow` - 错误上下文与统一 `Result`
- `serde` / `serde_json` - 序列化支持
