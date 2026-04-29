use anyhow::{Context, Result, bail};

use addzero_knowledge::{KnowledgeService, database_url, source_specs};

#[tokio::main]
async fn main() -> Result<()> {
    let database_url =
        database_url().context("missing MSC_AIO_DATABASE_URL / DATABASE_URL / local env file")?;
    let sources = source_specs();
    if sources.is_empty() {
        bail!("no knowledge roots were discovered on this machine");
    }

    let service = KnowledgeService::connect(&database_url).await?;
    let report = service.sync_sources(&sources).await?;

    println!("synced sources: {}", report.synced_sources.join(", "));
    println!("upserted documents: {}", report.upserted_documents);
    println!("skipped paths: {}", report.skipped_paths.len());

    Ok(())
}
