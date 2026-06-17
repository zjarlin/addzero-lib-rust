use az_agent::complete::chat_completions;
use tracing::Level;
use tracing_subscriber::FmtSubscriber;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let _ = dotenvy::dotenv();

    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;
    let result = chat_completions("gpt-5.4", Some("You are a helpful assistant."), "hi").await?;
    print!("Result: {result}");
    Ok(())
}
