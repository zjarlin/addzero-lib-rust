use async_openai::types::chat::{
    ChatCompletionRequestMessageContentPartRefusal, ChatCompletionRequestSystemMessageArgs,
    ChatCompletionRequestUserMessageArgs, CreateChatCompletionRequest,
    CreateChatCompletionRequestArgs,
};

/// Sends a single chat completion request with an optional system prompt.
pub async fn chat_completions(
    model: &str,
    system: Option<&str>,
    prompt: &str,
) -> anyhow::Result<(String)> {
    let client = async_openai::Client::new();

    let mut messages = vec![];
    if let Some(system) = system {
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
        .max_tokens(2048u32)
        .build()?;
    let response = client.chat().create(request).await?;
    tracing::info!("Response: {:#?}", response);
    let content = response
        .choices
        .into_iter()
        .next()
        .and_then(|c| c.message.content)
        .ok_or_else(|| anyhow::anyhow!("No content in reposense"))?;

    Ok(content)
}
