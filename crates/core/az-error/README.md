# az-error

addzero 生态系统的错误处理辅助入口。内部失败默认使用 `anyhow::Result`，HTTP、CLI、插件等外部边界再按各自协议映射响应。

## 功能

- `status_code_for_error` / `status_code_for_message`：边界层按错误文本推断 HTTP 状态码
- `error_type_for_error` / `error_type_for_message`：边界层按错误文本推断机器可读错误类型

## 用法

```rust
use anyhow::bail;
use az_error::status_code_for_message;

fn find_user(id: u64) -> anyhow::Result<String> {
    if id == 0 {
        bail!("not found: 用户 {id} 不存在");
    }
    Ok(format!("用户 {id}"))
}

let result = find_user(0);
assert!(result.is_err());
assert_eq!(status_code_for_message(&result.unwrap_err().to_string()), 404);
```

## 依赖的 crates

- `anyhow` — 错误返回与上下文
