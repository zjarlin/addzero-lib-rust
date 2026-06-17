use std::sync::Arc;

use rudi::{Context, DynProvider, Module, modules, providers, singleton};

use crate::{
    chat::OpenAiChatBackend,
    chat_responses::ChatResponsesAgentRunner,
    config::OpenAiRuntimeConfig,
    responses::ResponsesRunner,
    tool::{AgentTool, ToolRegistry, current_time::CurrentTimeTool, workspace::WorkspaceTool},
};

/// Rudi module that wires az-agent runtime services.
pub struct AgentModule;

impl Module for AgentModule {
    fn providers() -> Vec<DynProvider> {
        providers![
            singleton(|_| {
                ToolRegistry::new(vec![
                    Arc::new(CurrentTimeTool) as Arc<dyn AgentTool + Send + Sync>,
                    Arc::new(WorkspaceTool) as Arc<dyn AgentTool + Send + Sync>,
                ])
            }),
            singleton(|cx| {
                let config = cx.resolve::<OpenAiRuntimeConfig>();
                let tools = cx.resolve::<ToolRegistry>();
                ResponsesRunner::new(config.client(), tools)
            }),
            singleton(|cx| {
                let config = cx.resolve::<OpenAiRuntimeConfig>();
                let tools = cx.resolve::<ToolRegistry>();
                ChatResponsesAgentRunner::new(OpenAiChatBackend::new(config.client()), tools)
            }),
        ]
    }
}

/// Creates a rudi context for az-agent with runtime configuration inserted as a singleton.
pub fn create_agent_context(config: OpenAiRuntimeConfig) -> Context {
    Context::options()
        .singleton(config)
        .create(modules![AgentModule])
}
