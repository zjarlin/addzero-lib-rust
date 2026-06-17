use az_agent::complete::responses_with_demo_tool;
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
    let result = responses_with_demo_tool(
        &model,
        Some("You are a helpful assistant. When answering current time questions, use the available time tool before answering."),
        &prompt,
    )
    .await?;
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
