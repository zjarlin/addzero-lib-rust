# az-cli-repl

可扩展的 CLI REPL（交互式命令行）引擎框架，支持参数解析、帮助系统和命令注册。

## 功能

- 提供 [`Command`] trait，通过实现该 trait 快速注册自定义命令
- 支持按命令名称或数字索引执行命令
- 内置帮助系统（`h`）和退出命令（`q`）
- 自动参数解析，支持 String、Int、Float、Bool 四种类型
- 参数支持默认值与必填校验
- 可自定义提示符、退出命令和帮助命令

## 安装

在 `Cargo.toml` 中添加：
```toml
[dependencies]
az-cli-repl = { path = "../az-cli-repl" }       # workspace 内部引用
# 或发布后：
# az-cli-repl = "0.1"                             # crates.io 引用
```

## 用法

```rust
use az_cli_repl::{Command, ParamDef, ParamType, ParsedParams, ReplEngine, ReplError};

struct GreetCommand;

impl Command for GreetCommand {
    fn command(&self) -> &str { "greet" }
    fn description(&self) -> &str { "打招呼" }
    fn param_defs(&self) -> &[ParamDef] {
        &[ParamDef::new("name", ParamType::String, "你的名字")]
    }
    fn eval(&self, params: ParsedParams) -> Result<String, ReplError> {
        let name = params.get_string(0).unwrap_or("世界");
        Ok(format!("你好，{name}！"))
    }
}

let engine = ReplEngine::new(vec![Box::new(GreetCommand)]);
let outcome = engine.run_line("greet Rust");
```

## 依赖的 crates

- `thiserror` - 错误类型派生宏
