# az-persistence

Workspace 级 PostgreSQL 持久化基础设施，提供统一的 Toasty 连接、模型注册和 SQL 迁移管理。

## 功能

- **数据库 URL 发现**：按优先级从环境变量（`MSC_AIO_DATABASE_URL` > `DATABASE_URL`）或 `~/.config/aio/aio.env` 配置文件读取连接地址
- **连接管理**：通过 `PersistenceContext` 封装共享 `toasty::Db`
- **模型注册**：连接时显式传入当前服务使用的 `toasty::ModelSet`
- **Workspace 迁移**：连接时按稳定顺序执行幂等 SQL 迁移

## 安装

在 `Cargo.toml` 中添加：

```toml
[dependencies]
az-persistence = { path = "../persistence" }       # workspace 内部引用
# 或发布后：
# az-persistence = "0.1"                              # crates.io 引用
```

## 用法

```rust,no_run
use az_persistence::context::PersistenceContext;

#[derive(toasty::Model)]
struct ExampleRecord {
    #[key]
    id: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let models = toasty::models!(ExampleRecord);
    let ctx = PersistenceContext::connect(models).await?;
    let mut db = ctx.db().lock().await;
    let _records = toasty::stmt::Query::<toasty::stmt::List<ExampleRecord>>::all()
        .exec(&mut *db)
        .await?;

    Ok(())
}
```

也可以指定连接地址：

```rust,no_run
use az_persistence::context::PersistenceContext;

async fn connect_explicitly() -> anyhow::Result<()> {
let ctx = PersistenceContext::connect_with_url(
    "postgresql://user:pass@localhost/mydb",
    toasty::ModelSet::new(),
).await?;
let _db = ctx.db();

Ok(())
}
```

## 依赖的 crates

- `toasty` — PostgreSQL 模型、查询 DSL 和数据库执行器
- `anyhow` — 错误返回与上下文
- `tokio` — 异步运行时与共享数据库锁
