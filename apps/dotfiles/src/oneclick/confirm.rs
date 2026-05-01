use crate::error::Result;
use crate::platform::{prompt_yes_no, run_interactive_shell};

pub(crate) fn confirm_and_run(
    assume_yes: bool,
    dry_run: bool,
    description: &str,
    command: &str,
) -> Result<()> {
    if dry_run {
        println!("[dry-run] {description}: {command}");
        return Ok(());
    }
    if !prompt_yes_no(&format!("是否执行: {description}?"), false, assume_yes)? {
        println!("已跳过: {description}");
        return Ok(());
    }
    run_interactive_shell(command)
}

pub(crate) fn confirm_and_run_action<F>(
    assume_yes: bool,
    dry_run: bool,
    description: &str,
    action: F,
) -> Result<()>
where
    F: FnOnce() -> Result<()>,
{
    if dry_run {
        println!("[dry-run] {description}");
        return Ok(());
    }
    if !prompt_yes_no(&format!("是否执行: {description}?"), false, assume_yes)? {
        println!("已跳过: {description}");
        return Ok(());
    }
    action()
}

pub(crate) fn run_or_print<F>(dry_run: bool, description: &str, action: F) -> Result<()>
where
    F: FnOnce() -> Result<()>,
{
    if dry_run {
        println!("[dry-run] {description}");
        Ok(())
    } else {
        action()
    }
}
