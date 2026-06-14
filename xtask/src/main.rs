#![forbid(unsafe_code)]

use std::process::ExitCode;

fn main() -> ExitCode {
    eprintln!("xtask is currently disabled — awaiting plugin API refactor");
    ExitCode::from(0)
}
