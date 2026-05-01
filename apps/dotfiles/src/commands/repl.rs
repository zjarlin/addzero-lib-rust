use std::io::{self, Write};

use clap::Parser;

use crate::cli::{Cli, Command};
use crate::error::Result;
use crate::settings::Settings;

use super::run_with_cli;

pub(crate) fn run(settings: Settings) -> Result<()> {
    println!("dotfiles Rust REPL，输入 help 查看命令，输入 exit 退出。");
    loop {
        print!("dotfiles> ");
        io::stdout().flush()?;

        let mut line = String::new();
        if io::stdin().read_line(&mut line)? == 0 {
            break;
        }

        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        if matches!(line, "exit" | "quit" | "q") {
            break;
        }
        if line == "help" {
            println!(
                "常用命令: status, sync, add-dotfiles, remove-dotfiles, add-pkg, rm-pkg, config cat"
            );
            println!("也可以直接输入任意 dotfiles 子命令参数，例如: show-port 3000");
            continue;
        }

        let Some(words) = shlex::split(line) else {
            eprintln!("无法解析命令行");
            continue;
        };

        let args = std::iter::once("dotfiles".to_string())
            .chain(std::iter::once("--config".to_string()))
            .chain(std::iter::once(
                settings.config_file.to_string_lossy().into_owned(),
            ))
            .chain(words)
            .collect::<Vec<_>>();

        match Cli::try_parse_from(args) {
            Ok(cli) => {
                if matches!(cli.command, Some(Command::Repl)) {
                    eprintln!("当前已经在 REPL 中");
                    continue;
                }
                if let Err(err) = run_with_cli(cli) {
                    eprintln!("error: {err}");
                }
            }
            Err(err) => eprintln!("{err}"),
        }
    }

    Ok(())
}
