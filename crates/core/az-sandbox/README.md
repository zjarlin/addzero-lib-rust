# az-sandbox

用于脚本和插件执行的可序列化沙箱策略类型。

此 crate 本身不执行任何操作。它仅定义一个小型、可移植的策略对象，供高级运行时嵌入到请求中。

## 提供内容

- `SandboxPolicy`
- `SandboxPolicy::permissive()`
- `SandboxPolicy::deny_all()`

## 示例

```rust
use az_sandbox::sandbox::SandboxPolicy;

let policy = SandboxPolicy {
    fs_allow: vec!["/tmp".to_string()],
    net_allow: vec!["api.example.com:443".to_string()],
    cmd_allow: vec!["git".to_string()],
    max_memory_mb: 256,
    max_time_secs: 30,
};

assert_eq!(policy.max_time_secs, 30);
```

## 适用范围

当需要跨层传递沙箱意图时使用此 crate。
不要将其用作进程运行器、文件系统包装器或网络客户端。
