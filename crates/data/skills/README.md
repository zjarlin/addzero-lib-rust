# az-skills

针对 `~/.agents/skills/<name>/SKILL.md` 以及可选 Postgres 镜像的技能管理服务，是 msc-aio 服务端函数使用的唯一入口。

## 功能

- **双后端存储**：同时支持文件系统（`FsRepo`）和 PostgreSQL（`PgRepo`）两种存储后端，PG 不可用时自动降级为纯文件模式
- **CRUD 操作**：提供技能的创建、读取、更新、删除，自动同步文件系统与 PG
- **内容哈希比对**：通过 SHA-256 计算内容哈希，合并时判断文件系统与 PG 是否一致
- **批量同步**：`sync_now()` 执行全量双向对账，生成 `SyncReport` 报告
- **来源追踪**：`SkillSource` 枚举标识技能来源（文件系统 / PG / Both）

## 安装

在 `Cargo.toml` 中添加：

```toml
[dependencies]
az-skills = { path = "../skills" }       # workspace 内部引用
# 或发布后：
# az-skills = "0.1"                         # crates.io 引用
```

## 用法

```rust
use az_skills::{FsRepo, SkillService};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let fs = FsRepo::new("~/.agents/skills");

    // 尝试连接 PG，失败时自动降级为纯文件模式
    let service = SkillService::try_attach(Some("postgresql://localhost/msc"), fs).await;

    // 列出所有技能
    let skills = service.list().await?;
    for skill in &skills {
        println!("{} [{:?}]", skill.name, skill.source);
    }

    // 手动触发同步
    let report = service.sync_now().await?;
    println!("同步完成：已创建 {} 条，已更新 {} 条", report.created, report.updated);

    Ok(())
}
```

## 依赖的 crates

- `sqlx` — PostgreSQL 异步驱动
- `tokio` — 异步运行时
- `sha2` — 内容哈希计算
- `serde` / `serde_yaml` — 技能元数据序列化
- `chrono` — 时间戳管理
- `regex` — 技能名称验证
- `uuid` — 唯一标识生成
- `anyhow` — 错误返回与上下文
