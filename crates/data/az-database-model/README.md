# az-database-model

方言无关的数据库模型与 Schema 定义库，用于代码生成场景。

## 功能

- `Schema` / `Table` / `Column`：以构建器风格描述数据库 Schema 结构
- `DataType`：方言无关的列数据类型枚举（BigInt、Varchar、Text 等）
- `Relation` / `RelationKind`：表间关系定义（一对一、一对多、多对多）
- `Index`：索引定义
- `ModelError`：Schema 校验错误（空名称、重复表/列、未知引用等）

## 安装

在 `Cargo.toml` 中添加：
```toml
[dependencies]
az-database-model = { path = "../az-database-model" }       # workspace 内部引用
# 或发布后：
# az-database-model = "0.1"                      # crates.io 引用
```

## 用法

```rust
use az_database_model::{Schema, Table, Column, DataType, Relation, RelationKind};

// 定义表结构
let users = Table::new("users")
    .column(Column::new("id", DataType::BigInt).primary_key().auto_increment())
    .column(Column::new("name", DataType::Varchar(255)).not_null())
    .column(Column::new("email", DataType::Varchar(255)).unique());

let posts = Table::new("posts")
    .column(Column::new("id", DataType::BigInt).primary_key().auto_increment())
    .column(Column::new("user_id", DataType::BigInt).not_null())
    .column(Column::new("title", DataType::Varchar(255)).not_null());

// 构建 Schema
let schema = Schema::new("myapp")
    .table(users)
    .table(posts);

assert_eq!(schema.tables.len(), 2);
```

## 依赖的 crates

- `thiserror` - 错误类型派生
- `serde` / `serde_json` - 序列化支持
