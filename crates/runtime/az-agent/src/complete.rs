use async_openai::types::chat::{
    ChatCompletionRequestMessage, ChatCompletionRequestSystemMessageArgs,
    ChatCompletionRequestUserMessageArgs, CreateChatCompletionRequestArgs,
};

use crate::{
    config::OpenAiRuntimeConfig,
    di::create_agent_context,
    responses::{ResponsesResult, ResponsesRunRequest, ResponsesRunner},
    structured::time_answer_schema,
};

/// Sends a single chat completion request with an optional system prompt.
///
/// This remains as a small compatibility facade. New agent flows should prefer
/// [`ResponsesRunner`] so tools and structured output share the same path.
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

/// Runs the Responses API demo with the current-time tool and structured output schema.
pub async fn responses_with_demo_tool(
    model: &str,
    instructions: Option<&str>,
    prompt: &str,
) -> anyhow::Result<ResponsesResult> {
    let config = OpenAiRuntimeConfig::from_env()?;
    let mut cx = create_agent_context(config);
    let runner = cx.resolve::<ResponsesRunner>();
    runner
        .run(ResponsesRunRequest {
            model: model.to_string(),
            instructions: instructions.map(ToString::to_string),
            prompt: prompt.to_string(),
            images: Vec::new(),
            structured_output: Some(time_answer_schema()),
            tool_choice: Some(crate::tool::current_time::TOOL_NAME.to_string()),
        })
        .await
}
