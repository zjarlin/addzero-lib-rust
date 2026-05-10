use clap::Parser;

use crate::cli::{Cli, Command};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InternalCommand {
    Refresh,
    System(String),
}

pub fn parse_internal_command(input: &str) -> Result<InternalCommand, String> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Err("命令不能为空。".to_string());
    }
    if trimmed == "refresh" {
        return Ok(InternalCommand::Refresh);
    }
    if trimmed.starts_with("system ") {
        return Ok(InternalCommand::System(trimmed.to_string()));
    }
    Err("命令栏当前只支持 `refresh` 或 `system ...`。".to_string())
}

pub async fn execute_system_command(input: &str) -> Result<String, String> {
    let argv = shlex::split(&format!("aio {input}"))
        .ok_or_else(|| "命令包含未闭合的引号。".to_string())?;
    let cli = Cli::try_parse_from(argv).map_err(|err| err.to_string())?;
    match cli.command {
        Some(Command::System(system_cli)) => {
            crate::system_cli::run_system_cli_to_string(system_cli)
                .await
                .map_err(|err| err.to_string())
        }
        _ => Err("命令栏当前只支持 `system ...`。".to_string()),
    }
}
