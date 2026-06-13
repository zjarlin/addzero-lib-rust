# az-sql

带类型安全的流式 SQL 查询构建器，支持 SELECT、INSERT、UPDATE 和 DELETE，使用参数化值防止 SQL 注入。

## 一句话说明

通过链式调用以类型安全的方式构建参数化 SQL 语句，自动引用标识符防止注入。

## 功能

- **SELECT 查询构建** — 支持 `select`、`distinct`、`from`、`where`、`inner_join`、`left_join`、`right_join`、`full_outer_join`、`cross_join`、`group_by`、`having`、`order_by`、`limit`、`offset`
- **INSERT 查询构建** — 支持 `into`、`columns`、`values`，可批量插入多行
- **UPDATE 查询构建** — 支持 `table`、`set`、`where`、`limit`
- **DELETE 查询构建** — 支持 `from`、`where`、`limit`
- **参数化查询** — `where` 条件与 `values` 使用占位符 `?`，返回独立的参数列表，防止 SQL 注入
- **标识符自动引用** — 表名和列名自动以 ANSI SQL 双引号包裹，内部双引号自动转义（`"` → `""`）
- **构建前校验** — `try_build()` 方法在缺少表名、列名或参数数量不匹配时返回明确的 `anyhow::Error`
- **统一的 `Query` trait** — 所有查询构建器实现 `Query` trait，提供 `build()` 和 `to_sql()` 方法

## 安装

### 通过工作空间路径引入

在你的 `Cargo.toml` 中添加：

```toml
[dependencies]
az-sql = { path = "../crates/data/az-sql" }
```

### 通过 crates.io 引入

```toml
[dependencies]
az-sql = "0.1"
```

## 用法

### SELECT 查询

```rust
use az_sql::query::Query;
use az_sql::select::SelectQuery;

fn main() -> anyhow::Result<()> {
    let query = SelectQuery::new()
        .select(&["id", "name", "email"])
        .from("users")
        .r#where("active = ?", vec!["true"])
        .order_by("name", true)
        .limit(10);

    let (sql, params) = query.build()?;
    // sql:  SELECT "id", "name", "email" FROM "users" WHERE active = ? ORDER BY "name" ASC LIMIT 10
    // params: ["true"]
    Ok(())
}
```

### INSERT 查询

```rust
use az_sql::insert::InsertQuery;
use az_sql::query::Query;

let query = InsertQuery::new()
    .into("users")
    .columns(&["name", "email"])
    .values(vec!["Alice", "alice@example.com"])
    .values(vec!["Bob", "bob@example.com"]);

let (sql, params) = query.build()?;
// sql:    INSERT INTO "users" ("name", "email") VALUES (?, ?), (?, ?);
// params: ["Alice", "alice@example.com", "Bob", "bob@example.com"]
```

### UPDATE 查询

```rust
use az_sql::query::Query;
use az_sql::update::UpdateQuery;

let query = UpdateQuery::new()
    .table("users")
    .set("name", "Alice")
    .set("email", "alice@new.com")
    .r#where("id = ?", vec!["1"]);

let (sql, params) = query.build()?;
// sql:    UPDATE "users" SET "name" = ?, "email" = ? WHERE id = ?;
// params: ["Alice", "alice@new.com", "1"]
```

### DELETE 查询

```rust
use az_sql::delete::DeleteQuery;
use az_sql::query::Query;

let query = DeleteQuery::new()
    .from("users")
    .r#where("id = ?", vec!["42"]);

let (sql, params) = query.build()?;
// sql:    DELETE FROM "users" WHERE id = ?;
// params: ["42"]
```

### 使用 `try_build()` 进行校验

```rust
use az_sql::select::SelectQuery;

let result = SelectQuery::new().select(&["id"]).try_build();
assert!(result.unwrap_err().to_string().contains("no table"));
```

## 依赖的 crates

| crate | 说明 |
|-------|------|
| [`anyhow`](https://crates.io/crates/anyhow) | 错误上下文与统一 `Result` |
| [`serde`](https://crates.io/crates/serde) | 序列化/反序列化支持 |
