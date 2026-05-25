#![forbid(unsafe_code)]

use anyhow::Result;
use study_toasty::{database_url, run_toasty_crud};

#[tokio::main]
async fn main() -> Result<()> {
    let database_url = database_url()?;
    run_toasty_crud(&database_url).await
}
