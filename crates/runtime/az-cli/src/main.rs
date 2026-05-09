mod cli;
mod novel;
mod pipeline;
mod web;
mod web_text;

use anyhow::Result;

fn main() -> Result<()> {
    let args = cli::Cli::parse_args();
    cli::run(args)
}
