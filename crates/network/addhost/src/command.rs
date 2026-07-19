use std::{
    io::Write,
    process::{Command, Stdio},
};

use anyhow::{Context, Result, bail};

pub(crate) struct CommandRequest<'a> {
    pub program: &'a str,
    pub args: &'a [String],
    pub stdin: Option<&'a str>,
}

pub(crate) struct CommandOutput {
    pub status: i32,
    pub stdout: String,
    pub stderr: String,
}

impl CommandOutput {
    pub fn ensure_success(&self, action: &str) -> Result<()> {
        if self.status == 0 {
            return Ok(());
        }

        let detail = if self.stderr.trim().is_empty() {
            self.stdout.trim()
        } else {
            self.stderr.trim()
        };
        bail!("{action}失败，退出码 {}：{detail}", self.status)
    }
}

pub(crate) trait CommandRunner {
    fn run(&self, request: CommandRequest<'_>) -> Result<CommandOutput>;
}

pub(crate) struct SystemCommandRunner;

impl CommandRunner for SystemCommandRunner {
    fn run(&self, request: CommandRequest<'_>) -> Result<CommandOutput> {
        let mut command = Command::new(request.program);
        command.args(request.args);
        command.stdout(Stdio::piped());
        command.stderr(Stdio::piped());

        if request.stdin.is_some() {
            command.stdin(Stdio::piped());
        }

        let mut child = command
            .spawn()
            .with_context(|| format!("无法启动命令 {}", request.program))?;

        if let Some(stdin) = request.stdin {
            let mut child_stdin = child.stdin.take().context("无法打开命令标准输入")?;
            child_stdin
                .write_all(stdin.as_bytes())
                .context("无法写入命令标准输入")?;
        }

        let output = child
            .wait_with_output()
            .with_context(|| format!("等待命令 {} 完成时失败", request.program))?;

        Ok(CommandOutput {
            status: output.status.code().map_or(-1, |status| status),
            stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
            stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
        })
    }
}
