# az-log

基于类型的日志目标（target）生成工具，为 `log` crate 提供按 Rust 类型自动映射日志目标的能力。

## 功能

- 按泛型类型 `T` 生成并缓存日志目标字符串，避免重复的 `type_name` 调用
- 为任意值动态获取类型名称作为日志目标
- 提供 `trace_for!`、`debug_for!`、`info_for!`、`warn_for!`、`error_for!` 便捷宏，以类型/值为中心发起日志调用

## 安装

在 `Cargo.toml` 中添加：

```toml
[dependencies]
az-log = { path = "../az-log" }       # workspace 内部引用
# 或发布后：
# az-log = "0.1"                      # crates.io 引用
```

## 用法

```rust
use az_log::api::{logger_target, value_logger_target};

// 为泛型类型获取日志目标
struct MyService;
let target = logger_target::<MyService>(); // "my_crate::MyService"

// 为具体值获取日志目标
let value = 42u32;
let target = value_logger_target(&value); // "u32"

// 使用便捷宏
az_log::info_for!(MyService, "服务已启动");
```

## 依赖的 crates

- `log` - Rust 标准日志门面
