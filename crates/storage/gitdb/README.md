# gitdb

`gitdb` 是一个 Git-backed SQL 文档数据库 crate，同时提供库 API、`gitdb` CLI、连接池和一个多 Git repo 分片对象存储。

它适合需要可审计、本地优先、低写入量的数据或对象存储场景：每次 SQL 写入会落成 Git commit，对象存储的每次 put/delete 也会在对应 shard repo 中提交。

## 功能

- **SQL 数据库**：支持 `CREATE TABLE`、`DROP TABLE`、`INSERT`、`SELECT`、`UPDATE`、`DELETE`、`BEGIN`、`COMMIT`、`ROLLBACK`、`SHOW TABLES`、`DESCRIBE`
- **Git 存储**：表是 Git tree，行是 JSON blob，写入会产生 commit
- **库 API**：通过 `gitdb::db::Database` 直接执行 SQL
- **CLI/REPL**：`cargo run -p gitdb -- ...` 可直接打开交互式 SQL 终端
- **连接池**：`ConnectionPool` 提供有界连接复用
- **分片对象存储**：`ShardedBlobStore` 将对象写入多个 Git repo shard，超过软容量后自动创建新 shard

## 作为 workspace crate 使用

在本仓库内直接依赖本地 crate。路径按调用方 crate 的位置调整，例如 `crates/storage/az-drive-store` 当前使用：

```toml
[dependencies]
gitdb = { path = "../gitdb" }
```

如果从 workspace 根目录运行 CLI：

```bash
cargo run -p gitdb -- --help
cargo run -p gitdb -- ./tmp/my-gitdb
cargo run -p gitdb -- -d ./tmp/my-gitdb -e "SHOW TABLES"
```

## CLI 用法

```bash
# 使用默认数据库目录 .gitdb 打开 REPL
cargo run -p gitdb

# 指定数据库目录
cargo run -p gitdb -- ./data/gitdb
cargo run -p gitdb -- -d ./data/gitdb

# 执行单条 SQL 后退出
cargo run -p gitdb -- -d ./data/gitdb -e "CREATE TABLE users (id TEXT PRIMARY KEY, name TEXT)"
cargo run -p gitdb -- -d ./data/gitdb -e "INSERT INTO users (id, name) VALUES ('1', 'Alice')"
cargo run -p gitdb -- -d ./data/gitdb -e "SELECT * FROM users"

# 打印 SQL 和执行结果日志
cargo run -p gitdb -- -d ./data/gitdb -v -e "SHOW TABLES"
```

CLI 参数：

| 参数 | 说明 |
|---|---|
| `-d, --database PATH` | 数据库目录，默认 `.gitdb` |
| `-e, --execute SQL` | 执行单条 SQL 后退出 |
| `-v, --verbose` | 输出 SQL 和结果调试日志 |
| `-h, --help` | 显示帮助 |
| `--version` | 显示版本 |

REPL 内置命令：

| 命令 | 说明 |
|---|---|
| `.help`, `.h`, `.?` | 显示帮助 |
| `.quit`, `.exit`, `.q` | 退出 |
| `.tables`, `.dt` | 列出表 |
| `.schema <table>`, `.describe <table>`, `.d <table>` | 查看表结构 |
| `.stats` | 查看统计信息 |
| `.history` | 查看当前 REPL 输入历史 |
| `.explain <sql>` | 输出查询计划 |
| `.timing` | 开关耗时显示 |
| `.clear` | 清屏 |

## 库 API

### 打开数据库并执行 SQL

```rust
use gitdb::db::{Database, DatabaseError};
use gitdb::executor::QueryResult;

fn main() -> Result<(), DatabaseError> {
    let mut db = Database::open("./data/gitdb")?;

    db.execute("CREATE TABLE IF NOT EXISTS users (id TEXT PRIMARY KEY, name TEXT, age INTEGER)")?;
    db.execute("INSERT INTO users (id, name, age) VALUES ('1', 'Alice', 30)")?;

    let result = db.execute("SELECT id, name FROM users WHERE age >= 18 ORDER BY name ASC LIMIT 10")?;
    if let QueryResult::Select(rows) = result {
        for row in rows.iter() {
            println!("{row:?}");
        }
    }

    Ok(())
}
```

### 使用配置

```rust
use gitdb::db::{Database, DatabaseConfig};

fn main() -> Result<(), gitdb::db::DatabaseError> {
    let config = DatabaseConfig::new("./data/gitdb")
        .create_if_missing(true)
        .verbose(false)
        .auto_commit(true);

    let mut db = Database::open_with_config(config)?;
    db.execute("SHOW TABLES")?;

    Ok(())
}
```

`DatabaseConfig` 主要字段：

| 字段 | 默认值 | 说明 |
|---|---:|---|
| `path` | `.gitdb` | GitDB repository 路径 |
| `create_if_missing` | `true` | 目录不存在时初始化 Git repo |
| `enable_planner` | `true` | 启用查询计划/explain |
| `verbose` | `false` | 输出调试日志 |
| `auto_commit` | `true` | 自动提交写入 |

### 批量执行

```rust
use gitdb::db::Database;

fn main() -> Result<(), gitdb::db::DatabaseError> {
    let mut db = Database::open("./data/gitdb")?;
    let results = db.execute_batch(
        r#"
        CREATE TABLE IF NOT EXISTS users (id TEXT PRIMARY KEY, name TEXT);
        INSERT INTO users (id, name) VALUES ('1', 'Alice');
        SELECT * FROM users;
        "#,
    )?;

    println!("{} statements executed", results.len());
    Ok(())
}
```

### 事务

```rust
use gitdb::db::Database;

fn main() -> Result<(), gitdb::db::DatabaseError> {
    let mut db = Database::open("./data/gitdb")?;

    db.execute("BEGIN")?;
    db.execute("INSERT INTO users (id, name) VALUES ('2', 'Bob')")?;
    db.execute("COMMIT")?;

    db.transaction(|tx_db| {
        tx_db.execute("INSERT INTO users (id, name) VALUES ('3', 'Carol')")?;
        Ok(())
    })?;

    Ok(())
}
```

### 元数据和历史

```rust
use gitdb::db::Database;

fn main() -> Result<(), gitdb::db::DatabaseError> {
    let db = Database::open("./data/gitdb")?;

    let tables = db.tables()?;
    let stats = db.stats();
    let history = db.history(Some(20))?;
    let current_snapshot = db.snapshot("manual checkpoint")?;

    println!("tables={tables:?}, rows={}, commits={}", stats.total_rows, history.len());
    println!("current commit: {current_snapshot}");
    Ok(())
}
```

### 连接池

```rust
use gitdb::db::{ConnectionPool, DatabaseConfig};

fn main() -> Result<(), gitdb::db::DatabaseError> {
    let config = DatabaseConfig::new("./data/gitdb");
    let pool = ConnectionPool::new(config, 4)?;

    let mut connection = pool.get()?;
    connection.execute("SHOW TABLES")?;

    println!("opened={}, idle={}", pool.created(), pool.available());
    Ok(())
}
```

`ConnectionPool::get()` 在达到 `max_connections` 且没有空闲连接时会返回错误；连接在 drop 时归还到池。

## SQL 支持范围

### DDL

```sql
CREATE TABLE IF NOT EXISTS users (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    age INTEGER,
    active BOOLEAN
);

DROP TABLE users;
DROP TABLE IF EXISTS users;
```

支持的数据类型：

| SQL 类型 | 内部类型 |
|---|---|
| `TEXT`, `VARCHAR`, `CHAR`, `STRING` | Text |
| `INT`, `INTEGER`, `BIGINT`, `SMALLINT`, `TINYINT` | Integer |
| `FLOAT`, `REAL`, `DOUBLE`, `DECIMAL`, `NUMERIC` | Float |
| `BOOLEAN`, `BOOL` | Boolean |
| `JSON`, `JSONB` | Json |
| `TIMESTAMP`, `DATETIME`, `DATE` | Timestamp |
| `UUID` | Uuid |

支持的列约束：`PRIMARY KEY`、`NOT NULL`、`UNIQUE`、`DEFAULT <expr>`。

### DML

```sql
INSERT INTO users (id, name, age) VALUES ('1', 'Alice', 30);
INSERT INTO users (id, name, age) VALUES ('2', 'Bob', 25), ('3', 'Carol', 28);

SELECT * FROM users;
SELECT id, name FROM users WHERE age > 18 ORDER BY name ASC LIMIT 10 OFFSET 5;

UPDATE users SET name = 'Alicia' WHERE id = '1';
DELETE FROM users WHERE id = '1';
```

表达式支持比较、`AND`/`OR`、一元 `NOT`/`+`/`-`、`IS NULL`、`IS NOT NULL`、`IN`、`BETWEEN`、`LIKE`、基础算术和函数表达式解析。

当前限制：

- `SELECT` 只支持单表查询，不支持 join、subquery、group by、聚合执行
- `INSERT ... SELECT` 不支持
- 一次 `Database::execute()` 只接受一条语句；多语句使用 `execute_batch()`
- 主键行 key 当前按字符串读取，主键值建议使用 `TEXT`

## 分片对象存储

`ShardedBlobStore` 是独立于 SQL 路径的对象存储能力，用于把较大的二进制对象按 key 放进多个 Git repo shard。

```rust
use gitdb::blob_store::{BlobStoreConfig, ShardedBlobStore};

fn main() -> Result<(), gitdb::storage::StorageError> {
    let mut config = BlobStoreConfig::new("./data/gitdb-objects".into());
    config.max_shard_size_bytes = 8 * 1024 * 1024 * 1024;
    config.shard_prefix = "shard".to_owned();

    let store = ShardedBlobStore::open(config)?;

    store.put("objects/sha256/ab/cdef", b"hello")?;
    assert!(store.exists("objects/sha256/ab/cdef")?);
    assert_eq!(store.get("objects/sha256/ab/cdef")?, b"hello");
    store.delete("objects/sha256/ab/cdef")?;

    let shards = store.list_shards()?;
    println!("{shards:?}");
    Ok(())
}
```

对象 key 必须是相对路径，不能是空字符串、绝对路径或包含 `..`。目录结构大致如下：

```text
root/
  control/
    index/
    shards/
  shards/
    shard-0001/.git/
    shard-0002/.git/
```

`max_shard_size_bytes` 是软限制；当当前 shard 容量不足时，新对象会写入新 shard。

## 适用边界

适合：

- 本地可审计的小型数据集
- 低并发、低写入量的元数据存储
- 需要 Git commit 历史追踪的数据变更
- AIO Drive 这类需要 Git repo shard 保存对象字节的场景

不适合：

- 高频写入或大规模 OLTP
- 复杂 SQL 分析查询
- 多进程强一致写入
- 替代 PostgreSQL 作为正式业务数据源

## 验证

```bash
cargo test -p gitdb
cargo run -p gitdb -- -d ./tmp/gitdb-readme -e "CREATE TABLE IF NOT EXISTS users (id TEXT PRIMARY KEY, name TEXT)"
cargo run -p gitdb -- -d ./tmp/gitdb-readme -e "SHOW TABLES"
```
