# az-gitdb

Multi-repository pooling and load balancing for [`qeqqe/gitdb`](https://github.com/qeqqe/gitdb).

`az-gitdb` keeps the upstream Git-backed SQL database implementation intact and adds:

- bounded per-node GitDB connection pools,
- read/write node roles,
- round-robin, weighted round-robin, and least-in-flight routing,
- routed execution results that include the selected node id,
- broadcast execution for schema setup across all configured Git repositories.

## Consistency Boundary

This crate does not replicate data between Git repositories and does not provide distributed transactions. Configure multiple nodes only when they are externally replicated, or when the caller intentionally treats them as independent shards.

Use `broadcast_execute` for DDL or setup statements that must exist on every node.

```rust
use az_gitdb::{
    GitDbCluster, GitDbClusterConfig, GitDbClusterError, GitDbLoadBalanceStrategy,
    GitDbNodeConfig,
};

fn main() -> Result<(), GitDbClusterError> {
    let cluster = GitDbCluster::new(
        GitDbClusterConfig::new(vec![
            GitDbNodeConfig::new("git-a", "./data/git-a").max_connections(4),
            GitDbNodeConfig::new("git-b", "./data/git-b").max_connections(4),
        ])
        .strategy(GitDbLoadBalanceStrategy::WeightedRoundRobin),
    )?;

    cluster.broadcast_execute("CREATE TABLE users (id TEXT PRIMARY KEY, name TEXT)")?;
    let routed = cluster.execute_write("INSERT INTO users (id, name) VALUES ('1', 'Alice')")?;

    println!("served by {}", routed.node_id);
    Ok(())
}
```
