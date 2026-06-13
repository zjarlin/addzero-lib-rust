# az-knowledge

知识文档管理与同步服务——扫描本地 Markdown/文本目录，将文档元数据持久化到 PostgreSQL 或 SQLite，并生成编译期嵌入的文档目录。

## 功能

- **文档发现**：递归扫描指定目录，自动识别 `.md`、`.txt`、`.org`、`.rst` 文件
- **智能过滤**：自动跳过 `target/`、`node_modules/`、`vendor/` 等构建目录，以及 license、changelog 等元文件
- **双存储后端**：支持 PostgreSQL（正式）和 SQLite（本地开发）
- **同步管理**：按源（source）粒度同步，自动计算内容哈希，仅更新变化文档
- **目录生成**：`render_catalog` 将文档列表渲染为 Rust 常量数组，供编译期嵌入
- **手动录入**：通过 `ManualKnowledgeDocumentInput` 手动添加文档
- **CLI 工具**：内置 `knowledge-sync` 二进制，用于命令行执行同步

## 安装

在 `Cargo.toml` 中添加：

```toml
[dependencies]
az-knowledge = { path = "../az-knowledge" }       # workspace 内部引用
# 或发布后：
# az-knowledge = "0.1"                            # crates.io 引用
```

## 用法

```rust,ignore
use az_knowledge::{
    KnowledgeService, KnowledgeSourceSpec,
    source_specs, database_url,
};

#[tokio::main]
async fn main() -> Result<(), az_knowledge::KnowledgeError> {
    let url = database_url().expect("DATABASE_URL 未设置");
    let service = KnowledgeService::connect(&url).await?;

    // 同步所有配置的知识源
    let sources = source_specs();
    let report = service.sync_sources(&sources).await?;
    println!("已同步 {} 篇文档", report.upserted_documents);

    // 列出已索引文档
    let docs = service.list_documents().await?;
    for doc in &docs {
        println!("{}: {}", doc.slug, doc.title);
    }

    Ok(())
}
```

## 环境变量

| 变量 | 说明 |
|------|------|
| `DATABASE_URL` | 数据库连接 URL（支持 `postgres://` 和 `sqlite://`） |
| `MSC_AIO_KB_SOURCE_DIR` | 额外的知识源根目录 |
| `MSC_AIO_KNOWLEDGE_EXTRA_ROOTS` | 额外知识源目录列表（换行或分号分隔） |

## 依赖的 crates

- `az-persistence` - 数据库连接与持久化上下文
- `sea-orm` - PostgreSQL ORM
- `sqlx` - SQLite 驱动
- `chrono` - 时间处理
- `sha2` - 内容哈希计算
- `deunicode` - Unicode 转 ASCII slug 生成
- `serde` / `serde_json` - 序列化
- `anyhow` - 错误返回与上下文
- `tokio` - 异步运行时
- `uuid` - UUID 生成
