use async_openai::types::chat::{
    ChatCompletionRequestMessage, ChatCompletionRequestSystemMessageArgs,
    ChatCompletionRequestUserMessageArgs, CreateChatCompletionRequestArgs,
};

use crate::{
    config::OpenAiRuntimeConfig,
    di::create_agent_context,
    responses::{ResponsesResult, ResponsesRunRequest},
    spi::resolve_agent_responses_spi,
    structured::time_answer_schema,
};

/// 发送单次 chat completion 请求，可选系统提示词。
///
/// 这里保留为小型兼容门面。新的 agent 流程应优先走
/// [`crate::spi::AgentResponsesSpi`]，让工具调用和结构化输出共用同一路径。
pub async fn chat_completions(
    model: &str,
    system: Option<&str>,
    prompt: &str,
) -> anyhow::Result<String> {
    let config = OpenAiRuntimeConfig::from_env()?;
    let client = config.client();

    let mut messages: Vec<ChatCompletionRequestMessage> = Vec::new();
    if let Some(system) = system.filter(|value| !value.trim().is_empty()) {
        messages.push(
            ChatCompletionRequestSystemMessageArgs::default()
                .content(system)
                .build()?
                .into(),
        );
    }
    messages.push(
        ChatCompletionRequestUserMessageArgs::default()
            .content(prompt)
            .build()?
            .into(),
    );

    let request = CreateChatCompletionRequestArgs::default()
        .model(model)
        .messages(messages)
        .max_completion_tokens(2048u32)
        .build()?;

    let response = client.chat().create(request).await?;
    tracing::info!("chat completion response received");
    let content = response
        .choices
        .into_iter()
        .next()
        .and_then(|choice| choice.message.content)
        .ok_or_else(|| anyhow::anyhow!("No content in response"))?;

    Ok(content)
}

/// 使用当前时间工具和结构化输出 schema 运行 Responses API 示例。
pub async fn responses_with_demo_tool(
    model: &str,
    instructions: Option<&str>,
    prompt: &str,
) -> anyhow::Result<ResponsesResult> {
    let config = OpenAiRuntimeConfig::from_env()?;
    let mut cx = create_agent_context(config);
    let runner = resolve_agent_responses_spi(&mut cx)?;
    runner
        .run_responses(ResponsesRunRequest {
            model: model.to_string(),
            instructions: instructions.map(ToString::to_string),
            prompt: prompt.to_string(),
            images: Vec::new(),
            structured_output: Some(time_answer_schema()),
            tool_choice: Some(crate::tool::current_time::TOOL_NAME.to_string()),
        })
        .await
}
