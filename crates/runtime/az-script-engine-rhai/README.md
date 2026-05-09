# az-script-engine-rhai

Rhai implementation of the `az-script-engine` contract.

## What It Provides

- `RhaiEngine`

## Example

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

## Scope

Use this crate when you want an in-process Rhai engine behind a stable
cross-engine contract.
