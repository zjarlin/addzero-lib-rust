use std::sync::Arc;

use rudi::{Context, DynProvider, Module, modules, providers, singleton};

use crate::{
    chat::OpenAiChatBackend,
    chat_responses::ChatResponsesAgentRunner,
    config::OpenAiRuntimeConfig,
    responses::ResponsesRunner,
    spi::AgentResponsesSpiRef,
    tool::{AgentTool, ToolRegistry, current_time::CurrentTimeTool, workspace::WorkspaceTool},
};

/// 负责装配 az-agent 运行时服务的 Rudi 模块。
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
            singleton(|cx| {
                if std::env::var("AZ_AGENT_TOOL_MODE").as_deref() == Ok("chat") {
                    Arc::new(cx.resolve::<ChatResponsesAgentRunner<OpenAiChatBackend>>())
                        as AgentResponsesSpiRef
                } else {
                    Arc::new(cx.resolve::<ResponsesRunner>()) as AgentResponsesSpiRef
                }
            }),
        ]
    }
}

/// 创建 az-agent 的 Rudi 上下文，并把运行时配置作为 singleton 注入。
pub fn create_agent_context(config: OpenAiRuntimeConfig) -> Context {
    Context::options()
        .singleton(config)
        .create(modules![AgentModule])
}
