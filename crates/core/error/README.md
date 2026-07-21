# az-error

addzero 生态系统的错误处理基础库。内部致命错误继续使用 `anyhow::Result` 透明传播；允许安全降级的阶段可使用 `Diagnostics` 收集全部非致命错误，并由调用方在阶段边界决定接受部分结果还是聚合失败。

可恢复诊断设计参考 [A Novel Look at Error Handling in Rust](https://jtjlehi.github.io/2026/06/25/novel-rust-error-handling.html)，但实现继续遵循本仓库以 `anyhow` 为默认运行时错误类型的约定。

## 能力

- `diagnostics::Diagnostics`：按发生顺序收集带上下文的 `anyhow::Error`
- `Diagnostics::capture`：失败时记录诊断并返回 `None`
- `Diagnostics::recover`：失败时记录诊断并执行调用方显式提供的降级闭包
- `Diagnostics::finish` / `Diagnostics::into_result`：在阶段边界把全部诊断聚合成一个 `anyhow::Error`
- `classification::status_code_for_error` / `classification::status_code_for_message`：在协议边界推断 HTTP 状态码
- `classification::error_type_for_error` / `classification::error_type_for_message`：在协议边界推断机器可读错误类型

## 选择错误路径

| 场景 | 处理方式 |
| --- | --- |
| 结果已经不可信、关键不变量被破坏、后续操作无意义 | 返回 `anyhow::Result<T>` 并使用 `?` 立即早退 |
| 当前操作失败，但存在明确且安全的领域降级值 | 使用 `Diagnostics::recover` |
| 当前操作失败，调用方需要自行选择是否提供降级值 | 使用 `Diagnostics::capture` |
| 阶段结束后允许交付部分结果 | 读取 `Diagnostics::iter` 或消费 `Diagnostics::into_errors` |
| 阶段结束后任何诊断都应使整体失败 | 使用 `Diagnostics::finish` 或 `Diagnostics::into_result` |

`Diagnostics` 只应覆盖一个清晰阶段或批处理范围。不要把它作为全局错误通道，也不要用它替代正常的 `Result` 传播。

## Workspace 依赖

```toml
[dependencies]
anyhow.workspace = true
az-error.workspace = true
```

## 显式降级

致命输入继续通过 `?` 早退；只有可选输入使用诊断收集器降级。

```rust
use anyhow::{Context, Result};
use az_error::diagnostics::Diagnostics;

fn calculate(required: &str, optional: &str) -> Result<(u32, Diagnostics)> {
    let required = required.parse::<u32>().context("解析必填值")?;
    let mut diagnostics = Diagnostics::default();
    let optional = diagnostics.recover(
        optional.parse::<u32>().context("解析可选值"),
        || 0,
    );

    Ok((required + optional, diagnostics))
}

let (value, diagnostics) = calculate("10", "无效值")?;
assert_eq!(value, 10);
assert_eq!(diagnostics.len(), 1);
# Ok::<(), anyhow::Error>(())
```

降级闭包只在失败路径执行。闭包返回值必须具有明确领域语义，不能为了让流程继续而随意使用默认值。

## 延迟决定降级值

当深层函数只负责记录错误、不应决定最终降级值时，使用 `capture` 返回 `Option<T>`。

```rust
use anyhow::Context;
use az_error::diagnostics::Diagnostics;

let mut diagnostics = Diagnostics::default();
let parsed = diagnostics.capture("无效端口".parse::<u16>().context("解析服务端口"));
let port = match parsed {
    Some(port) => port,
    None => 8080,
};

assert_eq!(port, 8080);
assert_eq!(diagnostics.len(), 1);
```

## 阶段边界

接受部分结果时，调用方可展示、记录或序列化自己的诊断模型。

```rust
use anyhow::anyhow;
use az_error::diagnostics::Diagnostics;

let mut diagnostics = Diagnostics::default();
diagnostics.record(anyhow!("读取备用配置失败"));

for error in diagnostics.iter() {
    eprintln!("{error:#}");
}

let errors = diagnostics.into_errors();
assert_eq!(errors.len(), 1);
```

拒绝部分结果时，使用 `finish` 保留全部错误及其顺序。返回的 `anyhow::Error` 可向下转型为 `Diagnostics`，继续检查单项诊断。

```rust
use anyhow::anyhow;
use az_error::diagnostics::Diagnostics;

let mut diagnostics = Diagnostics::default();
diagnostics.record(anyhow!("第一项失败"));
diagnostics.record(anyhow!("第二项失败"));

let result = diagnostics.finish("部分结果");
let Err(error) = result else {
    panic!("存在诊断时必须聚合失败");
};
let Some(collected) = error.downcast_ref::<Diagnostics>() else {
    panic!("聚合错误必须保留 Diagnostics");
};

assert_eq!(collected.len(), 2);
```

## 协议边界映射

HTTP、CLI、插件等外部边界可以继续使用 `classification` 模块完成最终协议映射。业务内部不要依赖字符串分类做控制流。

```rust
use anyhow::bail;
use az_error::classification::status_code_for_message;

fn find_user(id: u64) -> anyhow::Result<String> {
    if id == 0 {
        bail!("not found: 用户 {id} 不存在");
    }

    Ok(format!("用户 {id}"))
}

let result = find_user(0);
let status = result
    .as_ref()
    .err()
    .map(|error| status_code_for_message(&error.to_string()));

assert_eq!(status, Some(404));
```

## 测试要求

使用 `Diagnostics` 的业务阶段应覆盖以下语义：

- 成功路径不产生诊断
- 降级结果符合领域约定
- 全部错误及其上下文按稳定顺序保留
- 阶段边界能够接受部分结果或聚合失败
- 致命错误仍通过 `?` 立即终止，且不会执行后续可恢复步骤
