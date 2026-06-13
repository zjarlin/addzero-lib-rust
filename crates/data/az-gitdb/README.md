# az-gitdb

`az-gitdb` 是对 `gitdb` 的多 repository 连接池和路由封装。每个节点以一个已有远程 Git repository 作为 GitDB 数据源，并在运行时准备本地 checkout 供上游 `gitdb` 打开；它不负责创建新的正式数据仓库，也不改写 GitDB 的 SQL 执行能力。

> 注意：当前 `az-gitdb` 的 `gitdb.workspace = true` 指向根 `Cargo.toml` 中固定 rev 的 `qeqqe/gitdb` 依赖；本仓库的 `crates/storage/gitdb` 是另一个同名 workspace member。需要改成使用本地 `crates/storage/gitdb` 时，应先调整依赖来源，而不是只改 README。

## 功能

- **多节点配置**：每个 GitDB repo 有独立 `id`、远程 URL、本地 checkout、角色、权重和连接池上限
- **远程数据源**：节点从已有远程仓库 clone 到本地 checkout；已有 checkout 会校验 `origin` 是否匹配配置
- **读写路由**：`ReadWrite`、`ReadOnly`、`WriteOnly` 三类节点角色
- **负载均衡**：支持 `RoundRobin`、`WeightedRoundRobin`、`LeastInFlight`
- **连接池**：每个节点使用有界连接池，连接 drop 后自动归还
- **SQL 分类**：通过上游 GitDB parser 将 SQL 分类为 read、write 或 transaction control
- **广播执行**：`broadcast_execute()` 可把 DDL 或初始化语句发到全部节点
- **路由结果**：返回 `GitDbRoutedResult { node_id, result }`，调用方可以记录命中节点

## 安装

在 workspace 内部使用：

```toml
[dependencies]
az-gitdb = { path = "../crates/data/az-gitdb" }
```

在本仓库其他 crate 中使用 workspace 依赖时：

```toml
[dependencies]
az-gitdb.workspace = true
```

如果根 `Cargo.toml` 尚未声明 `az-gitdb` workspace dependency，先在那里统一加，不要在 leaf crate 里散落版本或路径。

## 快速开始

```rust,no_run
use az_gitdb::{
    cluster::GitDbCluster,
    config::{GitDbClusterConfig, GitDbLoadBalanceStrategy, GitDbNodeConfig},
};
use gitdb::executor::QueryResult;

fn main() -> anyhow::Result<()> {
    let cluster = GitDbCluster::new(
        GitDbClusterConfig::new(vec![
            GitDbNodeConfig::new(
                "git-a",
                "git@github.com:example/gitdb-a.git",
                "/var/lib/az-gitdb/checkouts/git-a",
            )
            .max_connections(4),
            GitDbNodeConfig::new(
                "git-b",
                "git@github.com:example/gitdb-b.git",
                "/var/lib/az-gitdb/checkouts/git-b",
            )
            .max_connections(4),
        ])
        .strategy(GitDbLoadBalanceStrategy::WeightedRoundRobin),
    )?;

    cluster.broadcast_execute("CREATE TABLE IF NOT EXISTS users (id TEXT PRIMARY KEY, name TEXT)")?;

    let inserted = cluster.execute_write("INSERT INTO users (id, name) VALUES ('1', 'Alice')")?;
    println!("write served by {}", inserted.node_id);

    let selected = cluster.execute_read("SELECT * FROM users")?;
    println!("read served by {}", selected.node_id);

    if let QueryResult::Select(rows) = selected.result {
        println!("rows={}", rows.len());
    }

    Ok(())
}
```

## 配置模型

### 节点

```rust
use az_gitdb::config::{GitDbNodeConfig, GitDbNodeRole};

let node = GitDbNodeConfig::new(
    "primary",
    "git@github.com:example/gitdb-primary.git",
    "/var/lib/az-gitdb/checkouts/primary",
)
    .max_connections(8)
    .role(GitDbNodeRole::ReadWrite)
    .weight(2)
    .clone_if_missing(true)
    .enable_planner(true)
    .verbose(false)
    .auto_commit(true);
```

`GitDbNodeConfig` 字段：

| 字段 | 默认值 | 说明 |
|---|---:|---|
| `id` | 必填 | 稳定节点 ID，会出现在结果和错误中 |
| `remote_url` | 必填 | 已有 GitDB 远程 repository URL |
| `checkout_path` | 必填 | 本地 checkout/cache 路径，供上游 `gitdb` 打开 |
| `max_connections` | `4` | 最大同时 checkout 连接数 |
| `role` | `ReadWrite` | 节点读写能力 |
| `weight` | `1` | `WeightedRoundRobin` 的权重 |
| `clone_if_missing` | `true` | 本地 checkout 缺失时从 `remote_url` clone |
| `enable_planner` | `true` | 启用上游 planner |
| `verbose` | `false` | 启用上游 SQL 日志 |
| `auto_commit` | `true` | 启用上游 auto commit |

配置校验会拒绝空节点列表、重复节点 ID、空节点 ID、空远程 URL、空 checkout 路径、`max_connections = 0` 和 `weight = 0`。如果 `checkout_path` 已存在，必须是一个 Git repository，且 `origin` 必须匹配 `remote_url`。

### 集群

```rust
use az_gitdb::config::{GitDbClusterConfig, GitDbLoadBalanceStrategy, GitDbNodeConfig};

let config = GitDbClusterConfig::new(vec![
    GitDbNodeConfig::new(
        "a",
        "git@github.com:example/gitdb-a.git",
        "/var/lib/az-gitdb/checkouts/a",
    )
    .weight(2),
    GitDbNodeConfig::new(
        "b",
        "git@github.com:example/gitdb-b.git",
        "/var/lib/az-gitdb/checkouts/b",
    )
    .weight(1),
])
.strategy(GitDbLoadBalanceStrategy::WeightedRoundRobin);
```

负载策略：

| 策略 | 说明 |
|---|---|
| `RoundRobin` | 在可服务节点中轮转，默认策略 |
| `WeightedRoundRobin` | 按节点 `weight` 轮转，权重越大越常被选中 |
| `LeastInFlight` | 优先选择当前 checkout 数最少的节点 |

## 执行 SQL

### 自动分类路由

```rust
use az_gitdb::cluster::GitDbCluster;

fn route(cluster: &GitDbCluster) -> anyhow::Result<()> {
    let result = cluster.execute("SELECT * FROM users")?;
    println!("served by {}", result.node_id);
    Ok(())
}
```

`execute()` 会先解析 SQL：

- `SELECT`、`SHOW TABLES`、`DESCRIBE` 路由到读节点
- `CREATE TABLE`、`DROP TABLE`、`INSERT`、`UPDATE`、`DELETE` 路由到写节点
- `BEGIN`、`COMMIT`、`ROLLBACK` 会被拒绝，因为事务需要固定连接

### 显式读写 API

```rust
use az_gitdb::cluster::GitDbCluster;

fn route(cluster: &GitDbCluster) -> anyhow::Result<()> {
    cluster.execute_read("SELECT * FROM users")?;
    cluster.execute_write("UPDATE users SET name = 'Alicia' WHERE id = '1'")?;
    Ok(())
}
```

`execute_read()` 收到写 SQL，或 `execute_write()` 收到读 SQL，会返回带上下文的 `anyhow::Error`。这个约束可以尽早暴露调用方把 SQL 放错通道的问题。

### 广播执行

```rust
use az_gitdb::cluster::GitDbCluster;

fn initialize(cluster: &GitDbCluster) -> anyhow::Result<()> {
    let results = cluster.broadcast_execute(
        "CREATE TABLE IF NOT EXISTS users (id TEXT PRIMARY KEY, name TEXT)",
    )?;

    for routed in results {
        println!("initialized {}", routed.node_id);
    }

    Ok(())
}
```

`broadcast_execute()` 会对所有节点执行 SQL，适合建表、迁移或初始化。它不是分布式事务；某个节点失败后，已经成功的节点不会自动回滚。

## 事务

集群级 `execute()` 不接受事务控制语句。事务必须 checkout 一个写连接，保证 `BEGIN`、后续写入和 `COMMIT` 都打到同一个节点。

```rust
use az_gitdb::cluster::GitDbCluster;

fn write_in_transaction(cluster: &GitDbCluster) -> anyhow::Result<()> {
    let mut connection = cluster.checkout_write()?;
    println!("transaction node={}", connection.node_id());

    connection.execute("BEGIN")?;
    connection.execute("INSERT INTO users (id, name) VALUES ('2', 'Bob')")?;
    connection.execute("COMMIT")?;

    Ok(())
}
```

如果中途出错，调用方负责在同一个连接上执行 `ROLLBACK`：

```rust
use az_gitdb::cluster::GitDbCluster;

fn write_with_rollback(cluster: &GitDbCluster) -> anyhow::Result<()> {
    let mut connection = cluster.checkout_write()?;
    connection.execute("BEGIN")?;

    let result = connection.execute("INSERT INTO users (id, name) VALUES ('3', 'Carol')");
    if result.is_err() {
        let _ = connection.execute("ROLLBACK");
        result?;
    }

    connection.execute("COMMIT")?;
    Ok(())
}
```

也可以直接 checkout 指定节点：

```rust
use az_gitdb::cluster::GitDbCluster;

fn inspect_primary(cluster: &GitDbCluster) -> anyhow::Result<()> {
    let mut primary = cluster.checkout_node("primary")?;
    primary.execute("SHOW TABLES")?;
    Ok(())
}
```

## 统计与观测

```rust
use az_gitdb::cluster::GitDbCluster;

fn print_stats(cluster: &GitDbCluster) {
    let stats = cluster.stats();
    for node in stats.nodes {
        println!(
            "{} role={:?} opened={} idle={} in_flight={}",
            node.id, node.role, node.opened, node.idle, node.in_flight
        );
    }
}
```

`GitDbNodeStats` 包含节点 ID、远程 URL、本地 checkout 路径、角色、权重、连接上限、已打开连接数、空闲连接数和当前 checkout 数。

## 一致性边界

`az-gitdb` 不提供：

- repository 间数据复制
- 自动创建新的正式远程数据仓库
- 写入后的自动 push 或读取前的自动 pull
- 分布式事务
- 自动 failover 后的数据补偿
- read-after-write 跨节点一致性保证

多节点只在以下情况下使用：

- 外部系统已经保证这些远程 Git repo 是等价副本
- 调用方明确把节点当作独立 shard
- 只需要把 DDL/初始化语句广播到所有节点

本地 `checkout_path` 只是上游 `gitdb` 的运行时工作副本。正式数据源应是 `remote_url` 指向的已有 repository；如果需要把写入同步回远程，当前版本由调用方在连接使用边界之外执行 Git push/pull。

如果需要正式业务数据的一致持久化，仍应优先使用 PostgreSQL；本仓库约定正式 admin 业务数据遵循 `all in pg`。

## 错误上下文

`az-gitdb` 使用 `anyhow::Result` 暴露失败。常见错误消息会保留操作对象和节点信息：

| 消息片段 | 含义 |
|---|---|
| `invalid GitDB cluster configuration` | 集群或节点配置非法 |
| `no eligible GitDB node` | 没有节点能服务该类 SQL |
| `GitDB node not found` | 指定节点 ID 不存在 |
| `GitDB pool exhausted` | 单节点连接池耗尽 |
| `PoolsExhausted` | 所有可服务节点连接池都耗尽 |
| `NodeCheckout` | 节点远程仓库 clone/open 校验失败 |
| `NodeCheckoutIo` | 节点本地 checkout 目录准备失败 |
| `TransactionRequiresConnection` | 事务控制语句需要显式 checkout 连接 |
| `UnexpectedQueryKind` | 显式读写 API 收到错误 SQL 类型 |
| `Parse` | SQL 解析失败，无法分类 |
| `NodeDatabase` | 某个节点的上游 GitDB 执行失败 |

## 验证

```bash
cargo test -p az-gitdb
```

如果改了底层 `gitdb` 或依赖来源，同时跑：

```bash
cargo test -p gitdb
cargo test -p az-gitdb
```
