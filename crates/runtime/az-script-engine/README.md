# az-script-engine

Common contracts for embeddable script engines.

This crate defines the request, response, language enum, and traits that
concrete engines implement.

## What It Provides

- `ScriptLang`
- `ScriptInput`
- `ScriptOutput`
- `ScriptEngine`

## Example

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

## Scope

This crate is contract-only. It does not ship a concrete engine.
