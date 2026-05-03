//! Event system — component events decoupled from handler logic.
//!
//! Designers bind events to handler types + config; no code writing required.
//! Each handler type implements [`EventHandler`] and is registered in
//! [`HandlerRegistry`].  Dispatch is async with configurable timeout.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

// ---------------------------------------------------------------------------
// Data types
// ---------------------------------------------------------------------------

/// Runtime context passed to every handler when an event fires.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventContext {
    pub component_path: String,
    pub event_type: String,
    pub component_props: serde_json::Value,
    #[serde(default)]
    pub form_data: Option<serde_json::Value>,
    #[serde(default)]
    pub trigger_value: Option<serde_json::Value>,
}

/// A single side-effect produced by a handler.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum SideEffect {
    UpdateProps {
        path: String,
        props_patch: serde_json::Value,
    },
    Navigate {
        url: String,
    },
    ShowMessage {
        level: String,
        message: String,
    },
    TriggerEvent {
        target_path: String,
        event_type: String,
    },
    RefreshDataSource {
        source_id: String,
    },
    SetState {
        key: String,
        value: serde_json::Value,
    },
}

/// Outcome of a successful handler invocation.
#[derive(Debug, Clone, Default)]
pub struct EventResult {
    pub side_effects: Vec<SideEffect>,
}

// ---------------------------------------------------------------------------
// Errors
// ---------------------------------------------------------------------------

/// Errors that may occur during event dispatch.
#[derive(Debug, thiserror::Error)]
pub enum EventError {
    #[error("handler not found: {0}")]
    HandlerNotFound(String),
    #[error("handler timeout after {0}ms")]
    Timeout(u64),
    #[error("handler execution failed: {0}")]
    ExecutionFailed(String),
    #[error("invalid config: {0}")]
    InvalidConfig(String),
}

// ---------------------------------------------------------------------------
// Trait
// ---------------------------------------------------------------------------

/// A pluggable event handler.
///
/// Implementors receive the event [`EventContext`] together with a
/// handler-specific `config` JSON value and return an [`EventResult`]
/// containing zero or more [`SideEffect`]s.
#[async_trait::async_trait]
pub trait EventHandler: Send + Sync {
    async fn handle(
        &self,
        ctx: &EventContext,
        config: &serde_json::Value,
    ) -> Result<EventResult, EventError>;
}

// ---------------------------------------------------------------------------
// HandlerRegistry (Arc-wrapped for Clone)
// ---------------------------------------------------------------------------

/// Thread-safe, cloneable registry of event handlers.
///
/// Pre-populated with all built-in handlers via [`HandlerRegistry::new`].
#[derive(Clone)]
pub struct HandlerRegistry {
    handlers: Arc<RwLock<HashMap<String, Box<dyn EventHandler>>>>,
}

impl HandlerRegistry {
    /// Creates a new registry with all 7 built-in handlers pre-registered.
    pub fn new() -> Self {
        let mut map: HashMap<String, Box<dyn EventHandler>> = HashMap::new();
        map.insert("noop".into(), Box::new(NoopHandler));
        map.insert("navigate".into(), Box::new(NavigateHandler));
        map.insert("show_message".into(), Box::new(ShowMessageHandler));
        map.insert("set_state".into(), Box::new(SetStateHandler));
        map.insert("emit_event".into(), Box::new(EmitEventHandler));
        map.insert("http_call".into(), Box::new(HttpCallHandler));
        map.insert("rhai_script".into(), Box::new(RhaiScriptHandler));
        Self {
            handlers: Arc::new(RwLock::new(map)),
        }
    }

    /// Register (or replace) a handler under the given type name.
    pub async fn register(&self, name: impl Into<String>, handler: Box<dyn EventHandler>) {
        self.handlers.write().await.insert(name.into(), handler);
    }

    /// Dispatch an event to the named handler type.
    ///
    /// The handler call is wrapped in a tokio timeout.  `timeout_ms` of `0`
    /// falls back to the default 5 000 ms.
    pub async fn dispatch(
        &self,
        ctx: &EventContext,
        handler_type: &str,
        config: &serde_json::Value,
        timeout_ms: u64,
    ) -> Result<EventResult, EventError> {
        let guard = self.handlers.read().await;
        let handler = guard
            .get(handler_type)
            .ok_or_else(|| EventError::HandlerNotFound(handler_type.to_string()))?;

        // To satisfy the borrow across the await, we need the handler behind the
        // lock.  For the timeout wrapper we clone the dispatch future (not the
        // handler).  Since `EventHandler` is object-safe and behind `Box`, we
        // call `handle` while still holding the read guard and wrap the whole
        // future in timeout.
        let effective_timeout = if timeout_ms == 0 {
            5_000
        } else {
            timeout_ms
        };
        let deadline = Duration::from_millis(effective_timeout);

        match tokio::time::timeout(deadline, handler.handle(ctx, config)).await {
            Ok(result) => result,
            Err(_elapsed) => Err(EventError::Timeout(effective_timeout)),
        }
    }
}

impl Default for HandlerRegistry {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Built-in handlers
// ---------------------------------------------------------------------------

/// No-op handler — returns an empty result.
#[derive(Debug, Clone)]
struct NoopHandler;

#[async_trait::async_trait]
impl EventHandler for NoopHandler {
    async fn handle(
        &self,
        _ctx: &EventContext,
        _config: &serde_json::Value,
    ) -> Result<EventResult, EventError> {
        Ok(EventResult::default())
    }
}

/// Navigate handler — reads `url` from config.
#[derive(Debug, Clone)]
struct NavigateHandler;

#[async_trait::async_trait]
impl EventHandler for NavigateHandler {
    async fn handle(
        &self,
        _ctx: &EventContext,
        config: &serde_json::Value,
    ) -> Result<EventResult, EventError> {
        let url = config
            .get("url")
            .and_then(|v| v.as_str())
            .ok_or_else(|| EventError::InvalidConfig("missing `url`".into()))?
            .to_string();
        Ok(EventResult {
            side_effects: vec![SideEffect::Navigate { url }],
        })
    }
}

/// Show-message handler — reads `level` and `message` from config.
#[derive(Debug, Clone)]
struct ShowMessageHandler;

#[async_trait::async_trait]
impl EventHandler for ShowMessageHandler {
    async fn handle(
        &self,
        _ctx: &EventContext,
        config: &serde_json::Value,
    ) -> Result<EventResult, EventError> {
        let level = config
            .get("level")
            .and_then(|v| v.as_str())
            .unwrap_or("info")
            .to_string();
        let message = config
            .get("message")
            .and_then(|v| v.as_str())
            .ok_or_else(|| EventError::InvalidConfig("missing `message`".into()))?
            .to_string();
        Ok(EventResult {
            side_effects: vec![SideEffect::ShowMessage { level, message }],
        })
    }
}

/// Set-state handler — reads `key` and `value` from config.
#[derive(Debug, Clone)]
struct SetStateHandler;

#[async_trait::async_trait]
impl EventHandler for SetStateHandler {
    async fn handle(
        &self,
        _ctx: &EventContext,
        config: &serde_json::Value,
    ) -> Result<EventResult, EventError> {
        let key = config
            .get("key")
            .and_then(|v| v.as_str())
            .ok_or_else(|| EventError::InvalidConfig("missing `key`".into()))?
            .to_string();
        let value = config
            .get("value")
            .cloned()
            .unwrap_or(serde_json::Value::Null);
        Ok(EventResult {
            side_effects: vec![SideEffect::SetState { key, value }],
        })
    }
}

/// Emit-event handler — reads `target_path` and `event_type` from config.
#[derive(Debug, Clone)]
struct EmitEventHandler;

#[async_trait::async_trait]
impl EventHandler for EmitEventHandler {
    async fn handle(
        &self,
        _ctx: &EventContext,
        config: &serde_json::Value,
    ) -> Result<EventResult, EventError> {
        let target_path = config
            .get("target_path")
            .and_then(|v| v.as_str())
            .ok_or_else(|| EventError::InvalidConfig("missing `target_path`".into()))?
            .to_string();
        let event_type = config
            .get("event_type")
            .and_then(|v| v.as_str())
            .ok_or_else(|| EventError::InvalidConfig("missing `event_type`".into()))?
            .to_string();
        Ok(EventResult {
            side_effects: vec![SideEffect::TriggerEvent {
                target_path,
                event_type,
            }],
        })
    }
}

/// HTTP-call handler — fires a request and returns a ShowMessage with the
/// response status.
#[derive(Debug, Clone)]
struct HttpCallHandler;

#[async_trait::async_trait]
impl EventHandler for HttpCallHandler {
    async fn handle(
        &self,
        _ctx: &EventContext,
        config: &serde_json::Value,
    ) -> Result<EventResult, EventError> {
        let url = config
            .get("url")
            .and_then(|v| v.as_str())
            .ok_or_else(|| EventError::InvalidConfig("missing `url`".into()))?;
        let method = config
            .get("method")
            .and_then(|v| v.as_str())
            .unwrap_or("GET");

        let client = reqwest::Client::new();
        let builder = match method.to_uppercase().as_str() {
            "POST" => client.post(url),
            "PUT" => client.put(url),
            "DELETE" => client.delete(url),
            "PATCH" => client.patch(url),
            _ => client.get(url),
        };

        let resp = builder
            .send()
            .await
            .map_err(|e| EventError::ExecutionFailed(e.to_string()))?;

        let status = resp.status().as_u16();
        Ok(EventResult {
            side_effects: vec![SideEffect::ShowMessage {
                level: if status < 400 { "info" } else { "error" }.to_string(),
                message: format!("HTTP {status}"),
            }],
        })
    }
}

/// Rhai-script handler — executes an inline Rhai script.
///
/// The script has access to variables `component_path`, `event_type`,
/// `form_data`, and `trigger_value` extracted from the context.
#[derive(Debug, Clone)]
struct RhaiScriptHandler;

#[async_trait::async_trait]
impl EventHandler for RhaiScriptHandler {
    async fn handle(
        &self,
        ctx: &EventContext,
        config: &serde_json::Value,
    ) -> Result<EventResult, EventError> {
        let script = config
            .get("script")
            .and_then(|v| v.as_str())
            .ok_or_else(|| EventError::InvalidConfig("missing `script`".into()))?;

        // Run Rhai in a blocking task to avoid starving the tokio runtime.
        let ctx_clone = ctx.clone();
        let script_owned = script.to_string();

        tokio::task::spawn_blocking(move || {
            let engine = rhai::Engine::new();
            let mut scope = rhai::Scope::new();
            scope.push("component_path", ctx_clone.component_path.clone());
            scope.push("event_type", ctx_clone.event_type.clone());
            scope.push(
                "form_data",
                ctx_clone
                    .form_data
                    .as_ref()
                    .map(|v| v.to_string())
                    .unwrap_or_default(),
            );
            scope.push(
                "trigger_value",
                ctx_clone
                    .trigger_value
                    .as_ref()
                    .map(|v| v.to_string())
                    .unwrap_or_default(),
            );

            let result = engine
                .eval_with_scope::<rhai::Dynamic>(&mut scope, &script_owned)
                .map_err(|e| EventError::ExecutionFailed(e.to_string()))?;

            Ok(EventResult {
                side_effects: vec![SideEffect::ShowMessage {
                    level: "info".to_string(),
                    message: result.to_string(),
                }],
            })
        })
        .await
        .map_err(|e| EventError::ExecutionFailed(e.to_string()))?
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn ctx_stub() -> EventContext {
        EventContext {
            component_path: "root.button1".into(),
            event_type: "click".into(),
            component_props: serde_json::json!({}),
            form_data: None,
            trigger_value: None,
        }
    }

    #[tokio::test]
    async fn dispatch_noop_returns_empty_side_effects() {
        let reg = HandlerRegistry::new();
        let ctx = ctx_stub();
        let result = reg
            .dispatch(&ctx, "noop", &serde_json::json!({}), 5_000)
            .await
            .unwrap();
        assert!(result.side_effects.is_empty());
    }

    #[tokio::test]
    async fn dispatch_navigate_returns_correct_side_effect() {
        let reg = HandlerRegistry::new();
        let ctx = ctx_stub();
        let config = serde_json::json!({ "url": "/dashboard" });
        let result = reg
            .dispatch(&ctx, "navigate", &config, 5_000)
            .await
            .unwrap();
        assert_eq!(result.side_effects.len(), 1);
        match &result.side_effects[0] {
            SideEffect::Navigate { url } => assert_eq!(url, "/dashboard"),
            other => panic!("expected Navigate, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn dispatch_show_message_returns_side_effect() {
        let reg = HandlerRegistry::new();
        let ctx = ctx_stub();
        let config = serde_json::json!({ "level": "warn", "message": "hello" });
        let result = reg
            .dispatch(&ctx, "show_message", &config, 5_000)
            .await
            .unwrap();
        assert_eq!(result.side_effects.len(), 1);
        match &result.side_effects[0] {
            SideEffect::ShowMessage { level, message } => {
                assert_eq!(level, "warn");
                assert_eq!(message, "hello");
            }
            other => panic!("expected ShowMessage, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn dispatch_unknown_handler_returns_not_found() {
        let reg = HandlerRegistry::new();
        let ctx = ctx_stub();
        let err = reg
            .dispatch(&ctx, "nonexistent", &serde_json::json!({}), 5_000)
            .await
            .unwrap_err();
        assert!(
            matches!(err, EventError::HandlerNotFound(_)),
            "expected HandlerNotFound, got {err:?}"
        );
    }

    #[tokio::test]
    async fn dispatch_timeout_on_slow_handler() {
        // Register a deliberately slow handler.
        #[derive(Debug, Clone)]
        struct SlowHandler;
        #[async_trait::async_trait]
        impl EventHandler for SlowHandler {
            async fn handle(
                &self,
                _ctx: &EventContext,
                _config: &serde_json::Value,
            ) -> Result<EventResult, EventError> {
                tokio::time::sleep(Duration::from_secs(60)).await;
                Ok(EventResult::default())
            }
        }

        let reg = HandlerRegistry::new();
        reg.register("slow", Box::new(SlowHandler)).await;

        let ctx = ctx_stub();
        let err = reg
            .dispatch(&ctx, "slow", &serde_json::json!({}), 100)
            .await
            .unwrap_err();
        assert!(
            matches!(err, EventError::Timeout(100)),
            "expected Timeout(100), got {err:?}"
        );
    }

    #[tokio::test]
    async fn dispatch_rhai_script_basic_execution() {
        let reg = HandlerRegistry::new();
        let ctx = ctx_stub();
        let config = serde_json::json!({ "script": "40 + 2" });
        let result = reg
            .dispatch(&ctx, "rhai_script", &config, 5_000)
            .await
            .unwrap();
        assert_eq!(result.side_effects.len(), 1);
        match &result.side_effects[0] {
            SideEffect::ShowMessage { message, .. } => {
                assert_eq!(message, "42");
            }
            other => panic!("expected ShowMessage, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn side_effect_serde_roundtrip() {
        let se = SideEffect::Navigate {
            url: "/home".into(),
        };
        let json = serde_json::to_string(&se).unwrap();
        assert!(json.contains("\"type\":\"Navigate\""));
        let back: SideEffect = serde_json::from_str(&json).unwrap();
        match back {
            SideEffect::Navigate { url } => assert_eq!(url, "/home"),
            _ => panic!("roundtrip failed"),
        }
    }
}
