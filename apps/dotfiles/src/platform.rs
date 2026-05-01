use std::ffi::OsStr;
use std::process::{Command, Output, Stdio};

use crate::config::PlatformType;
use crate::error::{DotfilesError, Result, message};

#[derive(Clone, Debug)]
pub struct CommandOutput {
    pub code: Option<i32>,
    pub stdout: String,
    pub stderr: String,
}

impl CommandOutput {
    pub fn success(&self) -> bool {
        self.code == Some(0)
    }
}

pub fn run_shell(command: &str) -> Result<CommandOutput> {
    let output = if cfg!(windows) {
        Command::new("powershell")
            .args(["-NoProfile", "-Command", command])
            .output()
    } else {
        Command::new("bash").args(["-lc", command]).output()
    }
    .map_err(|err| message(format!("无法执行命令 `{command}`: {err}")))?;

    Ok(command_output(output))
}

pub fn run_shell_checked(command: &str) -> Result<CommandOutput> {
    let output = run_shell(command)?;
    if output.success() {
        Ok(output)
    } else {
        Err(DotfilesError::Command {
            command: command.to_string(),
            code: output.code,
            stdout: output.stdout,
            stderr: output.stderr,
        })
    }
}

pub fn run_checked<I, S>(program: &str, args: I) -> Result<CommandOutput>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let args = args.into_iter().collect::<Vec<_>>();
    let output = Command::new(program)
        .args(args.iter().map(AsRef::as_ref))
        .output()
        .map_err(|err| message(format!("无法执行命令 `{program}`: {err}")))?;
    let output = command_output(output);

    if output.success() {
        Ok(output)
    } else {
        let args = args
            .iter()
            .map(|arg| arg.as_ref().to_string_lossy())
            .collect::<Vec<_>>()
            .join(" ");
        Err(DotfilesError::Command {
            command: format!("{program} {args}"),
            code: output.code,
            stdout: output.stdout,
            stderr: output.stderr,
        })
    }
}

pub fn run_interactive_shell(command: &str) -> Result<()> {
    let status = if cfg!(windows) {
        Command::new("powershell")
            .args(["-NoProfile", "-Command", command])
            .stdin(Stdio::inherit())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .status()
    } else {
        Command::new("bash")
            .args(["-lc", command])
            .stdin(Stdio::inherit())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .status()
    }
    .map_err(|err| message(format!("无法执行命令 `{command}`: {err}")))?;

    if status.success() {
        Ok(())
    } else {
        Err(DotfilesError::Command {
            command: command.to_string(),
            code: status.code(),
            stdout: String::new(),
            stderr: String::new(),
        })
    }
}

pub fn command_exists(program: &str) -> bool {
    if cfg!(windows) {
        run_shell(&format!(
            "Get-Command {} -ErrorAction SilentlyContinue",
            quote_ps(program)
        ))
        .is_ok_and(|output| output.success())
    } else {
        run_shell(&format!("command -v {}", quote_sh(program))).is_ok_and(|output| output.success())
    }
}

pub fn show_port(port: u16) -> Result<String> {
    let command = match PlatformType::current() {
        PlatformType::Windows => format!(
            "Get-NetTCPConnection -LocalPort {port} -ErrorAction SilentlyContinue | ForEach-Object {{ $proc = Get-Process -Id $_.OwningProcess -ErrorAction SilentlyContinue; if ($proc) {{ \"Port {port} used by $($proc.ProcessName) (PID: $($_.OwningProcess))\" }} else {{ \"Port {port} used by PID $($_.OwningProcess)\" }} }}"
        ),
        PlatformType::Macos => format!("lsof -i :{port} -P -n"),
        PlatformType::Linux | PlatformType::Unknown => {
            format!("(ss -tulpn 2>/dev/null || netstat -tulpn 2>/dev/null) | grep ':{port} '")
        }
    };

    let output = run_shell(&command)?;
    if output.success() && !output.stdout.trim().is_empty() {
        Ok(format!(
            "端口 {port} 被以下进程占用:\n{}",
            output.stdout.trim()
        ))
    } else {
        Ok(format!("端口 {port} 未被占用"))
    }
}

pub fn kill_port(port: u16) -> Result<()> {
    let command = match PlatformType::current() {
        PlatformType::Windows => format!(
            "Get-NetTCPConnection -LocalPort {port} -ErrorAction SilentlyContinue | ForEach-Object {{ Stop-Process -Id $_.OwningProcess -Force }}"
        ),
        PlatformType::Macos | PlatformType::Linux | PlatformType::Unknown => {
            format!("lsof -ti :{port} | xargs kill -9")
        }
    };

    let output = run_shell(&command)?;
    if output.success() || (output.stderr.trim().is_empty() && output.stdout.trim().is_empty()) {
        Ok(())
    } else {
        Err(DotfilesError::Command {
            command,
            code: output.code,
            stdout: output.stdout,
            stderr: output.stderr,
        })
    }
}

pub fn prompt_yes_no(prompt: &str, default_yes: bool, assume_yes: bool) -> Result<bool> {
    if assume_yes {
        return Ok(true);
    }

    let default = if default_yes { "Y/n" } else { "y/N" };
    println!("{prompt} ({default})");

    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;
    let input = input.trim().to_ascii_lowercase();

    if input.is_empty() {
        return Ok(default_yes);
    }

    Ok(matches!(input.as_str(), "y" | "yes"))
}

pub fn quote_sh(value: &str) -> String {
    if value.is_empty() {
        return "''".to_string();
    }
    format!("'{}'", value.replace('\'', "'\\''"))
}

fn quote_ps(value: &str) -> String {
    format!("'{}'", value.replace('\'', "''"))
}

fn command_output(output: Output) -> CommandOutput {
    CommandOutput {
        code: output.status.code(),
        stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
        stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shell_quote_handles_apostrophe() {
        assert_eq!(quote_sh("a'b"), "'a'\\''b'");
    }
}
