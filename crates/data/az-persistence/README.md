# az-persistence

Workspace 级 PostgreSQL 持久化基础设施，提供统一的数据库连接建立、URL 发现和 SeaORM schema 迁移管理。

## 功能

- **数据库 URL 发现**：按优先级从环境变量（`MSC_AIO_DATABASE_URL` > `DATABASE_URL`）或 `~/.config/aio/aio.env` 配置文件读取连接地址
- **连接管理**：通过 `PersistenceContext` 封装 SeaORM `DatabaseConnection`，自动配置连接池参数
- **Workspace 迁移**：首次连接时自动执行所有 workspace 级 SQL 迁移，支持并发安全
- **并发迁移冲突处理**：自动检测并忽略并发执行迁移时的主键冲突
- **无效迁移记录清理**：连接时自动清除 `lib` 级别的无效 `seaql_migrations` 记录

## 安装

在 `Cargo.toml` 中添加：

```toml
[dependencies]
az-persistence = { path = "../az-persistence" }       # workspace 内部引用
# 或发布后：
# az-persistence = "0.1"                              # crates.io 引用
```

## 用法

```rust
use az_persistence::PersistenceContext;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 自动从环境变量或配置文件发现数据库 URL 并建立连接
    let ctx = PersistenceContext::connect().await?;

    // 获取 SeaORM DatabaseConnection
    let db = ctx.db();
    // ... 使用 db 进行数据库操作

    Ok(())
}
```

也可以指定连接地址：

```rust
let ctx = PersistenceContext::connect_with_url("postgresql://user:pass@localhost/mydb").await?;
```

## 依赖的 crates

- `sea-orm` / `sea-orm-migration` — 数据库 ORM 和迁移框架
- `async-trait` — async trait 支持
- `thiserror` — 错误类型派生
- `tokio` — 异步运行时（用于迁移锁）
