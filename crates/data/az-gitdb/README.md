# az-gitdb

`az-gitdb` 是对 `gitdb` 的多 repository 连接池和路由封装。它不改写 GitDB 的 SQL 执行能力，而是在多个 GitDB repo 之上增加节点配置、读写角色、负载均衡、连接池上限和带节点信息的执行结果。

> 注意：当前 `az-gitdb` 的 `gitdb.workspace = true` 指向根 `Cargo.toml` 中固定 rev 的 `qeqqe/gitdb` 依赖；本仓库的 `crates/storage/gitdb` 是另一个同名 workspace member。需要改成使用本地 `crates/storage/gitdb` 时，应先调整依赖来源，而不是只改 README。

## 功能

- **多节点配置**：每个 GitDB repo 有独立 `id`、路径、角色、权重和连接池上限
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

```rust
use az_gitdb::{
    GitDbCluster, GitDbClusterConfig, GitDbClusterError, GitDbLoadBalanceStrategy,
    GitDbNodeConfig, QueryResult,
};

fn main() -> Result<(), GitDbClusterError> {
    let cluster = GitDbCluster::new(
        GitDbClusterConfig::new(vec![
            GitDbNodeConfig::new("git-a", "./data/git-a").max_connections(4),
            GitDbNodeConfig::new("git-b", "./data/git-b").max_connections(4),
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
use az_gitdb::{GitDbNodeConfig, GitDbNodeRole};

let node = GitDbNodeConfig::new("primary", "./data/primary")
    .max_connections(8)
    .role(GitDbNodeRole::ReadWrite)
    .weight(2)
    .create_if_missing(true)
    .enable_planner(true)
    .verbose(false)
    .auto_commit(true);
```

`GitDbNodeConfig` 字段：

| 字段 | 默认值 | 说明 |
|---|---:|---|
| `id` | 必填 | 稳定节点 ID，会出现在结果和错误中 |
| `path` | 必填 | GitDB repository 路径 |
| `max_connections` | `4` | 最大同时 checkout 连接数 |
| `role` | `ReadWrite` | 节点读写能力 |
| `weight` | `1` | `WeightedRoundRobin` 的权重 |
| `create_if_missing` | `true` | 缺失时创建 repository |
| `enable_planner` | `true` | 启用上游 planner |
| `verbose` | `false` | 启用上游 SQL 日志 |
| `auto_commit` | `true` | 启用上游 auto commit |

配置校验会拒绝空节点列表、重复节点 ID、空节点 ID、`max_connections = 0` 和 `weight = 0`。

### 集群

```rust
use az_gitdb::{GitDbClusterConfig, GitDbLoadBalanceStrategy, GitDbNodeConfig};

let config = GitDbClusterConfig::new(vec![
    GitDbNodeConfig::new("a", "./data/a").weight(2),
    GitDbNodeConfig::new("b", "./data/b").weight(1),
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
use az_gitdb::{GitDbCluster, GitDbClusterError};

fn route(cluster: &GitDbCluster) -> Result<(), GitDbClusterError> {
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
use az_gitdb::{GitDbCluster, GitDbClusterError};

fn route(cluster: &GitDbCluster) -> Result<(), GitDbClusterError> {
    cluster.execute_read("SELECT * FROM users")?;
    cluster.execute_write("UPDATE users SET name = 'Alicia' WHERE id = '1'")?;
    Ok(())
}
```

`execute_read()` 收到写 SQL，或 `execute_write()` 收到读 SQL，会返回 `UnexpectedQueryKind`。这个约束可以尽早暴露调用方把 SQL 放错通道的问题。

### 广播执行

```rust
use az_gitdb::{GitDbCluster, GitDbClusterError};

fn initialize(cluster: &GitDbCluster) -> Result<(), GitDbClusterError> {
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
use az_gitdb::{GitDbCluster, GitDbClusterError};

fn write_in_transaction(cluster: &GitDbCluster) -> Result<(), GitDbClusterError> {
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
use az_gitdb::{GitDbCluster, GitDbClusterError};

fn write_with_rollback(cluster: &GitDbCluster) -> Result<(), GitDbClusterError> {
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
use az_gitdb::{GitDbCluster, GitDbClusterError};

fn inspect_primary(cluster: &GitDbCluster) -> Result<(), GitDbClusterError> {
    let mut primary = cluster.checkout_node("primary")?;
    primary.execute("SHOW TABLES")?;
    Ok(())
}
```

## 统计与观测

```rust
use az_gitdb::GitDbCluster;

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

`GitDbNodeStats` 包含节点 ID、路径、角色、权重、连接上限、已打开连接数、空闲连接数和当前 checkout 数。

## 一致性边界

`az-gitdb` 不提供：

- repository 间数据复制
- 分布式事务
- 自动 failover 后的数据补偿
- read-after-write 跨节点一致性保证

多节点只在以下情况下使用：

- 外部系统已经保证这些 Git repo 是等价副本
- 调用方明确把节点当作独立 shard
- 只需要把 DDL/初始化语句广播到所有节点

如果需要正式业务数据的一致持久化，仍应优先使用 PostgreSQL；本仓库约定正式 admin 业务数据遵循 `all in pg`。

## 错误类型

常见错误：

| 错误 | 含义 |
|---|---|
| `InvalidConfig` | 集群或节点配置非法 |
| `NoEligibleNode` | 没有节点能服务该类 SQL |
| `NodeNotFound` | 指定节点 ID 不存在 |
| `PoolExhausted` | 单节点连接池耗尽 |
| `PoolsExhausted` | 所有可服务节点连接池都耗尽 |
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
