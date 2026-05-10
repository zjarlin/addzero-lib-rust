# az-script-engine

可嵌入脚本引擎的通用契约。

此 crate 定义了请求、响应、语言枚举以及具体引擎实现的 trait。

## 提供内容

- `ScriptLang`
- `ScriptInput`
- `ScriptOutput`
- `ScriptEngine`

## 示例

```rust
use std::collections::BTreeMap;

use az_sandbox::sandbox::SandboxPolicy;
use az_script_engine::script::{ScriptInput, ScriptLang};

let input = ScriptInput {
    source: "40 + 2".to_string(),
    lang: ScriptLang::Rhai,
    vars: BTreeMap::new(),
    policy: SandboxPolicy::permissive(),
    timeout_secs: 5,
};

assert_eq!(input.timeout_secs, 5);
```

## 适用范围

此 crate 仅包含契约定义，不附带具体引擎实现。
