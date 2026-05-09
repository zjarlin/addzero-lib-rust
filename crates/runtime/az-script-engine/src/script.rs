//! Common contracts for embeddable script engines.

use az_sandbox::sandbox::SandboxPolicy;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::future::Future;
use std::pin::Pin;

// ─── Script Types ───────────────────────────────────────────────────

/// Supported script languages.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ScriptLang {
    Rhai,
    Python,
    TypeScript,
    Bash,
}

/// Input to a script execution.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ScriptInput {
    /// The script source code.
    pub source: String,
    /// Script language.
    pub lang: ScriptLang,
    /// Variables passed into the script scope.
    pub vars: BTreeMap<String, serde_json::Value>,
    /// Sandbox policy for this execution.
    pub policy: SandboxPolicy,
    /// Timeout in seconds (0 = no timeout).
    pub timeout_secs: u64,
}

/// Output from a script execution.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ScriptOutput {
    /// Exit code (0 = success).
    pub exit_code: i32,
    /// Captured stdout.
    pub stdout: String,
    /// Captured stderr.
    pub stderr: String,
    /// Variables exported from the script scope.
    pub vars: BTreeMap<String, serde_json::Value>,
    /// Execution duration in milliseconds.
    pub duration_ms: u64,
}

// ─── Engine Trait ───────────────────────────────────────────────────

/// Future returned by script execution methods.
pub type ScriptFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

/// Unified script engine trait.
///
/// Implementations: RhaiEngine, PythonEngine, TypeScriptEngine, BashEngine.
pub trait ScriptEngine: Send + Sync {
    /// The language this engine handles.
    fn lang(&self) -> ScriptLang;

    /// Execute a script synchronously (blocking the calling thread).
    /// For async execution, use `run_async`.
    fn run(&self, input: ScriptInput) -> ScriptOutput;

    /// Execute a script asynchronously.
    fn run_async<'a>(&'a self, input: ScriptInput) -> ScriptFuture<'a, ScriptOutput>;
}

// ─── Engine Registry ────────────────────────────────────────────────

/// A registry of all available script engines.
pub trait ScriptEngineRegistry: Send + Sync {
    /// Register a script engine.
    fn register(&mut self, engine: Box<dyn ScriptEngine>);

    /// Get an engine for the given language.
    fn get(&self, lang: ScriptLang) -> Option<&dyn ScriptEngine>;

    /// List all registered languages.
    fn languages(&self) -> Vec<ScriptLang>;
}

// ─── Error ──────────────────────────────────────────────────────────

#[derive(Clone, Debug, thiserror::Error)]
pub enum ScriptError {
    #[error("unsupported language: {0:?}")]
    UnsupportedLanguage(ScriptLang),

    #[error("script execution timed out after {0}s")]
    Timeout(u64),

    #[error("sandbox policy violation: {0}")]
    SandboxViolation(String),

    #[error("engine error: {0}")]
    Engine(String),
}
