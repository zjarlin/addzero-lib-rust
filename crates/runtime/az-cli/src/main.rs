automod::dir!(pub "src");

use anyhow::Result;

fn main() -> Result<()> {
    let args = cli::Cli::parse_args();
    cli::run(args)
}
