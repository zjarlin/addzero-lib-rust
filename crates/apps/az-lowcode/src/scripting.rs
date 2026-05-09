//! Rhai scripting engine for the lowcode platform.
//!
//! Embeds a [Rhai](https://rhai.rs) engine with sandboxed execution limits
//! and platform API stubs.  Scripts can be validated (syntax-only) or
//! executed against an [`EventContext`] to produce [`SideEffect`]s.

use std::sync::Arc;

use rhai::{Dynamic, Engine, Scope};
use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Errors
// ---------------------------------------------------------------------------

/// Errors produced by the scripting engine.
#[derive(Debug, Clone, thiserror::Error)]
pub enum ScriptError {
    /// Syntax / parse error with line and column information.
    #[error("syntax error at line {line}, column {column}: {message}")]
    SyntaxError {
        line: usize,
        column: usize,
        message: String,
    },
    /// Runtime evaluation error.
    #[error("runtime error: {0}")]
    RuntimeError(String),
    /// Execution exceeded a configured security limit.
    #[error("limit exceeded: {0}")]
    LimitExceeded(String),
    /// Script execution did not finish within the deadline.
    #[error("timeout")]
    Timeout,
}

// ---------------------------------------------------------------------------
// ScriptEngine
// ---------------------------------------------------------------------------

/// Sandboxed Rhai scripting engine.
///
/// Pre-configured with resource limits and platform API stubs.  Use
/// [`execute`](Self::execute) for full evaluation or
/// [`validate`](Self::validate) for syntax-only checking.
#[derive(Clone)]
pub struct ScriptEngine {
    /// The configured Rhai engine (wrapped in Arc because `Engine` is not
    /// `Clone`).
    engine: Arc<Engine>,
    /// Stored for introspection / tests.
    pub max_operations: u64,
    /// Maximum string size the engine may allocate.
    pub max_string_size: usize,
    /// Maximum array (Dynamic array) length.
    pub max_array_size: usize,
}

impl ScriptEngine {
    /// Creates a new engine with default limits and all platform API stubs
    /// registered.
    pub fn new() -> Self {
        let mut engine = Engine::new();

        // --- security limits ---------------------------------------------------
        engine.set_max_operations(10_000);
        engine.set_max_string_size(100_000);
        engine.set_max_array_size(1_000);

        // --- platform API stubs -------------------------------------------------
        engine.register_fn("get_prop", |_path: &str, _key: &str| -> Dynamic {
            Dynamic::UNIT
        });

        engine.register_fn("set_prop", |_path: &str, _key: &str, _value: Dynamic| {
            // no-op placeholder
        });

        engine.register_fn("http_get", |_url: &str| -> String {
            "[http_get placeholder]".to_string()
        });

        engine.register_fn("show_toast", |_msg: &str, _level: &str| {
            // no-op placeholder
        });

        engine.register_fn("navigate", |_url: &str| {
            // no-op placeholder
        });

        engine.register_fn("log", |msg: &str| {
            eprintln!("[lowcode:script] {msg}");
        });

        Self {
            engine: Arc::new(engine),
            max_operations: 10_000,
            max_string_size: 100_000,
            max_array_size: 1_000,
        }
    }

    /// Creates an engine with custom limits (useful for tests).
    pub fn with_limits(max_operations: u64, max_string_size: usize, max_array_size: usize) -> Self {
        let mut base = Self::new();
        let mut engine = Engine::new();

        engine.set_max_operations(max_operations);
        engine.set_max_string_size(max_string_size);
        engine.set_max_array_size(max_array_size);

        // Re-register all platform stubs on the custom engine.
        engine.register_fn("get_prop", |_path: &str, _key: &str| -> Dynamic {
            Dynamic::UNIT
        });
        engine.register_fn("set_prop", |_path: &str, _key: &str, _value: Dynamic| {});
        engine.register_fn("http_get", |_url: &str| -> String {
            "[http_get placeholder]".to_string()
        });
        engine.register_fn("show_toast", |_msg: &str, _level: &str| {});
        engine.register_fn("navigate", |_url: &str| {});
        engine.register_fn("log", |msg: &str| {
            eprintln!("[lowcode:script] {msg}");
        });

        base.engine = Arc::new(engine);
        base.max_operations = max_operations;
        base.max_string_size = max_string_size;
        base.max_array_size = max_array_size;
        base
    }

    /// Returns a reference to the inner Rhai engine.
    pub fn engine(&self) -> &Engine {
        &self.engine
    }

    // -----------------------------------------------------------------------
    // Public API
    // -----------------------------------------------------------------------

    /// Validates a script for syntax correctness **without** executing it.
    ///
    /// Returns `Ok(())` if the script parses cleanly, or a
    /// [`ScriptError::SyntaxError`] with line / column details.
    pub fn validate(&self, script: &str) -> Result<(), ScriptError> {
        self.engine
            .compile(script)
            .map(|_| ())
            .map_err(|err| parse_error_to_script_error(&err))
    }

    /// Evaluates a script and returns its result value.
    ///
    /// Performs a compile-time syntax check first, then evaluates.  Security
    /// limits (max operations, max string size, etc.) are enforced by the
    /// engine during evaluation.
    pub fn execute(&self, script: &str) -> Result<Dynamic, ScriptError> {
        // Pre-check syntax so we get nice line/column info.
        self.validate(script)?;

        let mut scope = Scope::new();
        self.engine
            .eval_with_scope::<Dynamic>(&mut scope, script)
            .map_err(|err| match *err {
                rhai::EvalAltResult::ErrorTooManyOperations(_) => ScriptError::LimitExceeded(
                    format!("exceeded max_operations ({})", self.max_operations),
                ),
                rhai::EvalAltResult::ErrorDataTooLarge(..) => {
                    ScriptError::LimitExceeded("data too large".into())
                }
                _ => ScriptError::RuntimeError(err.to_string()),
            })
    }
}

impl Default for ScriptEngine {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Extract a [`ScriptError`] from a boxed `ParseError`.
fn parse_error_to_script_error(err: &rhai::ParseError) -> ScriptError {
    let pos = err.position();
    let line = pos.line().unwrap_or(1);
    let col = pos.position().unwrap_or(0);
    ScriptError::SyntaxError {
        line,
        column: col,
        message: err.to_string(),
    }
}

/// Validation response body returned by the `/api/lowcode/script/validate`
/// endpoint.
#[derive(Debug, Serialize, Deserialize)]
pub struct ValidateResponse {
    /// Whether the script is syntactically valid.
    pub valid: bool,
    /// Human-readable error description when `valid` is `false`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    /// Line number of the syntax error (1-indexed).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line: Option<usize>,
    /// Column number of the syntax error.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub col: Option<usize>,
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_engine_basic_evaluation() {
        let engine = ScriptEngine::new();
        let result = engine.execute("40 + 2").unwrap();
        assert_eq!(result.as_int().unwrap(), 42);
    }

    #[test]
    fn test_engine_platform_functions_registered() {
        let engine = ScriptEngine::new();
        // log() should not panic or error
        engine.execute(r#"log("hello")"#).unwrap();
    }

    #[test]
    fn test_validate_valid_script() {
        let engine = ScriptEngine::new();
        assert!(engine.validate("40 + 2").is_ok());
    }

    #[test]
    fn test_validate_syntax_error() {
        let engine = ScriptEngine::new();
        let err = engine.validate("fn(").unwrap_err();
        assert!(
            matches!(err, ScriptError::SyntaxError { .. }),
            "expected SyntaxError, got {err:?}"
        );
    }

    #[test]
    fn test_syntax_error_has_line_and_column() {
        let engine = ScriptEngine::new();
        match engine.validate("fn(") {
            Err(ScriptError::SyntaxError {
                line,
                column,
                message,
            }) => {
                assert_eq!(line, 1, "expected line 1, got {line}");
                assert!(column > 0, "expected column > 0, got {column}");
                assert!(!message.is_empty());
            }
            other => panic!("expected SyntaxError, got {other:?}"),
        }
    }

    #[test]
    fn test_execute_runtime_error() {
        let engine = ScriptEngine::new();
        // Undefined variable → runtime error
        let err = engine.execute("let x = nonexistent_var").unwrap_err();
        assert!(
            matches!(err, ScriptError::RuntimeError(_)),
            "expected RuntimeError, got {err:?}"
        );
    }

    #[test]
    fn test_execute_limit_exceeded() {
        // Tiny operation budget so the script is guaranteed to exceed it.
        let engine = ScriptEngine::with_limits(10, 100_000, 1_000);
        // A simple loop will quickly exceed 10 operations.
        let err = engine
            .execute("let x = 0; while x < 1000 { x += 1; }")
            .unwrap_err();
        assert!(
            matches!(err, ScriptError::LimitExceeded(_)),
            "expected LimitExceeded, got {err:?}"
        );
    }

    #[test]
    fn test_get_prop_returns_unit() {
        let engine = ScriptEngine::new();
        let result = engine.execute(r#"get_prop("root/0", "label")"#).unwrap();
        assert!(result.is_unit());
    }

    #[test]
    fn test_set_prop_does_not_error() {
        let engine = ScriptEngine::new();
        engine
            .execute(r#"set_prop("root/0", "label", "hello")"#)
            .unwrap();
    }

    #[test]
    fn test_http_get_returns_placeholder() {
        let engine = ScriptEngine::new();
        let result = engine
            .execute(r#"http_get("https://example.com")"#)
            .unwrap();
        assert_eq!(result.into_string().unwrap(), "[http_get placeholder]");
    }

    #[test]
    fn test_show_toast_does_not_error() {
        let engine = ScriptEngine::new();
        engine.execute(r#"show_toast("hello", "info")"#).unwrap();
    }

    #[test]
    fn test_navigate_does_not_error() {
        let engine = ScriptEngine::new();
        engine.execute(r#"navigate("/dashboard")"#).unwrap();
    }

    #[test]
    fn test_validate_syntax_error_has_details() {
        let engine = ScriptEngine::new();
        match engine.validate("let x = (1 +") {
            Err(ScriptError::SyntaxError { line, message, .. }) => {
                assert_eq!(line, 1);
                assert!(!message.is_empty());
            }
            other => panic!("expected SyntaxError, got {other:?}"),
        }
    }

    #[test]
    fn test_script_engine_is_clone() {
        let engine = ScriptEngine::new();
        let engine2 = engine.clone();
        let result = engine2.execute("40 + 2").unwrap();
        assert_eq!(result.as_int().unwrap(), 42);
    }
}
