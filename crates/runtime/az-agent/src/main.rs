use az_agent::{
    config::OpenAiRuntimeConfig, di::create_agent_context, responses::ResponsesRunRequest,
    spi::resolve_agent_responses_spi, structured::time_answer_schema, tool::current_time,
};
use std::path::Path;
use tracing::Level;
use tracing_subscriber::FmtSubscriber;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    load_env();

    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;
    let model = std::env::var("OPENAI_MODEL").unwrap_or_else(|_| "gpt-5.5".to_string());
    let prompt = std::env::var("AZ_AGENT_PROMPT").unwrap_or_else(|_| "现在几点?".to_string());
    let config = OpenAiRuntimeConfig::from_env()?;
    let mut cx = create_agent_context(config);
    let request = ResponsesRunRequest {
        model,
        instructions: Some(
            "You are a helpful assistant. When answering current time questions, use the available time tool before answering."
                .to_string(),
        ),
        prompt,
        images: Vec::new(),
        structured_output: Some(time_answer_schema()),
        tool_choice: Some(current_time::TOOL_NAME.to_string()),
    };
    let runner = resolve_agent_responses_spi(&mut cx)?;
    let result = runner.run_responses(request).await?;
    println!("requested_model: {}", result.requested_model);
    println!("response_model: {}", result.response_model);
    println!("response_id: {}", result.response_id);
    println!("status: {}", result.status);
    if let Some(warning) = result.warning {
        println!("warning: {warning}");
    }
    println!("{}", result.output_text);
    Ok(())
}

fn load_env() {
    let crate_env = Path::new(env!("CARGO_MANIFEST_DIR")).join(".env");
    if crate_env.exists() {
        let _ = dotenvy::from_path(crate_env);
    } else {
        let _ = dotenvy::dotenv();
    }
}
