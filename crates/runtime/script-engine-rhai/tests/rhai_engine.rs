use std::collections::BTreeMap;

use az_sandbox::sandbox::SandboxPolicy;
use az_script_engine::script::{
    ScriptEngine, ScriptEngineFactory, ScriptEngineRegistry, ScriptInput, ScriptLang,
};
use az_script_engine_rhai::engine::{RhaiEngine, RhaiEngineFactory, rhai_engine_registry};
use rhai::Engine;

fn rhai_input(source: impl Into<String>, vars: BTreeMap<String, serde_json::Value>) -> ScriptInput {
    ScriptInput {
        source: source.into(),
        lang: ScriptLang::Rhai,
        vars,
        policy: SandboxPolicy::permissive(),
        timeout_secs: 0,
    }
}

#[test]
fn rhai_engine_evaluates_expression_result() {
    let engine = RhaiEngine::new();
    let output = engine.run(rhai_input("let x = 40 + 2; x", BTreeMap::new()));

    // A successful script must expose both a zero exit code and the `_result` value.
    assert_eq!(output.exit_code, 0);
    assert_eq!(
        output.vars.get("_result").and_then(|v| v.as_i64()),
        Some(42)
    );
}

#[test]
fn rhai_engine_injects_variables() {
    let engine = RhaiEngine::new();
    let mut vars = BTreeMap::new();
    vars.insert("name".into(), serde_json::json!("AIO"));

    let output = engine.run(rhai_input(r#""Hello, " + name + "!""#, vars));

    // Variable injection is part of the public script engine contract.
    assert_eq!(output.exit_code, 0);
    assert_eq!(
        output.vars.get("_result").and_then(|v| v.as_str()),
        Some("Hello, AIO!")
    );
}

#[test]
fn rhai_engine_captures_print_output() {
    let engine = RhaiEngine::new();
    let output = engine.run(rhai_input(
        r#"print("hello from rhai"); 1"#,
        BTreeMap::new(),
    ));

    // Host callers need print output surfaced as script stdout.
    assert_eq!(output.exit_code, 0);
    assert!(output.stdout.contains("hello from rhai"));
}

#[test]
fn with_engine_uses_injected_rhai_runtime() {
    let mut rhai = Engine::new();
    rhai.register_fn("answer", || 42_i64);
    let engine = RhaiEngine::with_engine(rhai);

    let output = engine.run(rhai_input("answer()", BTreeMap::new()));

    // Custom host functions prove this instance uses the injected Rhai runtime.
    assert_eq!(output.exit_code, 0);
    assert_eq!(
        output.vars.get("_result").and_then(|v| v.as_i64()),
        Some(42)
    );
}

#[test]
fn rhai_engine_registry_registers_usable_default_engine() {
    let registry = rhai_engine_registry();

    assert_eq!(registry.languages(), vec![ScriptLang::Rhai]);
    let output = registry
        .get(ScriptLang::Rhai)
        .expect("rhai engine should be registered")
        .run(rhai_input("21 * 2", BTreeMap::new()));

    // The helper must expose a usable engine, not just a language marker.
    assert_eq!(output.exit_code, 0);
    assert_eq!(
        output.vars.get("_result").and_then(|v| v.as_i64()),
        Some(42)
    );
}

#[test]
fn rhai_engine_factory_builds_rhai_engine_instances() {
    let factory = RhaiEngineFactory;
    assert_eq!(factory.lang(), ScriptLang::Rhai);

    let engine = factory.build();
    let output = engine.run(rhai_input("6 * 7", BTreeMap::new()));

    assert_eq!(output.exit_code, 0);
    assert_eq!(
        output.vars.get("_result").and_then(|v| v.as_i64()),
        Some(42)
    );
}
