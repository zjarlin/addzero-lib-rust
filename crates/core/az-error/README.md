# az-error

addzero 生态系统的统一错误类型，为后端服务提供一致的错误建模与 HTTP 状态码映射。

## 功能

- **AppError** — 统一应用错误枚举，覆盖常见错误场景：
  - `NotFound` (404) — 资源未找到
  - `Validation` (422) — 输入校验失败
  - `Unauthorized` (401) — 未认证
  - `Forbidden` (403) — 无权限
  - `Conflict` (409) — 资源冲突
  - `BadRequest` (400) — 请求格式错误
  - `Timeout` (504) — 操作超时
  - `Internal` (500) — 内部错误
  - `Io` (500) — I/O 错误（自动从 `std::io::Error` 转换）
  - `Json` (500) — JSON 序列化/反序列化错误（自动从 `serde_json::Error` 转换）
- **AppResult\<T\>** — 便捷的 `Result<T, AppError>` 类型别名
- **status_code** — 获取每个错误变体对应的 HTTP 状态码
- **error_type** — 获取机器可读的错误类型标识符

## 安装

在 `Cargo.toml` 中添加：

```toml
[dependencies]
az-error = { path = "../az-error" }        # workspace 内部引用
# 或发布后：
# az-error = "0.1"                         # crates.io 引用
```

## 用法

```rust
use az_error::{AppError, AppResult};

fn find_user(id: u64) -> AppResult<String> {
    if id == 0 {
        return Err(AppError::NotFound(format!("用户 {id} 不存在")));
    }
    Ok(format!("用户 {id}"))
}

let result = find_user(0);
assert!(result.is_err());
assert_eq!(result.unwrap_err().status_code(), 404);

// std::io::Error 自动转换
let io_result: AppResult<()> = Err(std::io::Error::new(
    std::io::ErrorKind::NotFound,
    "file missing",
).into());
assert_eq!(io_result.unwrap_err().error_type(), "io");
```

## 依赖的 crates

- `thiserror` — 错误类型派生宏
- `serde` — 序列化支持
- `serde_json` — JSON 处理（提供 `Json` 变体的自动转换）
