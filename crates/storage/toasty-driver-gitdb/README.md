# toasty-driver-gitdb

Toasty ORM 的 `gitdb` 数据库驱动适配器。将 Toasty 的 schema 操作、查询和迁移映射到本地文件系统上的 [gitdb](../gitdb/) SQL 数据库。

## 功能

- **SQL 操作**：支持 SELECT、INSERT、UPDATE、DELETE 以及事务
- **Schema 迁移**：自动将 Toasty schema diff 转换为 gitdb DDL，并追踪已应用的迁移
- **连接管理**：每个连接对应一个后台线程持有的 gitdb `Database` 实例
- **类型映射**：将 gitdb 的 JSON 结果行转换为 Toasty 的类型化值

## 安装

在 `Cargo.toml` 中添加：

```toml
[dependencies]
toasty-driver-gitdb = { path = "../toasty-driver-gitdb" }   # workspace 内部引用
# 或发布后：
# toasty-driver-gitdb = "0.1"                                # crates.io 引用
```

## 用法

```rust
use toasty_core::driver::Driver;
use toasty_driver_gitdb::GitDb;

let driver = GitDb::open("./my-gitdb-data");

// 注册 driver 到 Toasty 的 driver 注册表
// （具体取决于你使用的 Toasty API 版本）
```

通过连接 URL 创建：

```rust
use toasty_driver_gitdb::GitDb;

let driver = GitDb::new("gitdb://./my-gitdb-data")?;
```

## SQL 支持范围

本驱动基于 gitdb 的 SQL 子集：

- **DDL**：`CREATE TABLE`（含 `IF NOT EXISTS`）、列约束 `PRIMARY KEY` / `NOT NULL`
- **DML**：`INSERT`、`SELECT`（单表）、`UPDATE`、`DELETE`
- **事务**：`BEGIN` / `COMMIT` / `ROLLBACK`
- **不支持**：JOIN、子查询、GROUP BY、聚合、AUTO_INCREMENT、RETURNING、read_only 事务

## 依赖的 crates

- `gitdb` — 本地文件系统 SQL 数据库引擎
- `toasty-core` — Toasty ORM 核心抽象（Driver / Connection trait）
- `toasty-sql` — Toasty SQL 生成与序列化
- `chrono` — 迁移时间戳
- `uuid` — UUID 值支持

## 验证

```bash
cargo test -p toasty-driver-gitdb
```