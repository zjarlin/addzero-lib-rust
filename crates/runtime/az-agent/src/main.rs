use az_agent::complete::chat_completions;
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
    let result = chat_completions(&model, Some("You are a helpful assistant."), "hi").await?;
    println!("1111111111111{result}");
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
