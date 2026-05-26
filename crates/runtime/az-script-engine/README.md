# az-script-engine

可嵌入脚本引擎的通用契约。

此 crate 定义了请求、响应、语言枚举以及具体引擎实现的 trait。

## 提供内容

- `ScriptLang`
- `ScriptInput`
- `ScriptOutput`
- `ScriptEngine`
- `ScriptEngineFactory`
- `InMemoryScriptEngineRegistry`

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

## 插件化接入

具体引擎实现 crate 可以实现 `ScriptEngineFactory`，宿主再用 registry 统一装配：

```rust
use az_script_engine::script::{
    InMemoryScriptEngineRegistry, ScriptEngineFactory,
};

fn registry_from_plugins(factories: &[&dyn ScriptEngineFactory]) -> InMemoryScriptEngineRegistry {
    InMemoryScriptEngineRegistry::with_factories(factories)
}
```

单个插件也可以显式注册，registry 会按 `ScriptLang` 替换同语言旧实现：

```rust
use az_script_engine::script::{InMemoryScriptEngineRegistry, ScriptEngineFactory};

fn add_plugin(registry: &mut InMemoryScriptEngineRegistry, factory: &dyn ScriptEngineFactory) {
    registry.register_from_factory(factory);
}
```

面向 trait object registry 时可直接复用同一层契约检查：

```rust
use az_script_engine::script::{
    ScriptEngineFactory, ScriptEngineRegistry, register_engine_factory,
};

fn add_plugin(registry: &mut dyn ScriptEngineRegistry, factory: &dyn ScriptEngineFactory) {
    register_engine_factory(registry, factory);
}
```

## 适用范围

此 crate 仅包含契约定义，不附带具体引擎实现。
