# addzero-sandbox

Serializable sandbox policy types for script and plugin execution.

This crate does not execute anything by itself. It only defines a small,
portable policy object that higher-level runtimes can embed in requests.

## What It Provides

- `SandboxPolicy`
- `SandboxPolicy::permissive()`
- `SandboxPolicy::deny_all()`

## Example

```rust
use addzero_sandbox::sandbox::SandboxPolicy;

let policy = SandboxPolicy {
    fs_allow: vec!["/tmp".to_string()],
    net_allow: vec!["api.example.com:443".to_string()],
    cmd_allow: vec!["git".to_string()],
    max_memory_mb: 256,
    max_time_secs: 30,
};

assert_eq!(policy.max_time_secs, 30);
```

## Scope

Use this crate when you need to pass sandbox intent across layers.
Do not use it as a process runner, filesystem wrapper, or network client.
