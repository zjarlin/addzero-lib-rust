# az-script-engine-rhai

`az-script-engine` 契约的 Rhai 实现。

## 提供内容

- `RhaiEngine`
- `RhaiEngineFactory`
- `register_rhai_engine`
- `rhai_engine_registry`

## 示例

```rust
use std::collections::BTreeMap;

use az_sandbox::sandbox::SandboxPolicy;
use az_script_engine::script::{ScriptEngine, ScriptInput, ScriptLang};
use az_script_engine_rhai::RhaiEngine;

let engine = RhaiEngine::new();
let output = engine.run(ScriptInput {
    source: "let x = 40 + 2; x".to_string(),
    lang: ScriptLang::Rhai,
    vars: BTreeMap::new(),
    policy: SandboxPolicy::permissive(),
    timeout_secs: 0,
});

assert_eq!(output.exit_code, 0);
```

## Registry / factory

```rust
use az_script_engine::script::{ScriptEngineFactory, ScriptLang};
use az_script_engine_rhai::RhaiEngineFactory;

let factory = RhaiEngineFactory;
assert_eq!(factory.lang(), ScriptLang::Rhai);
let engine = factory.build();
```

## 适用范围

当需要一个基于稳定跨引擎契约的进程内 Rhai 引擎时使用此 crate。
