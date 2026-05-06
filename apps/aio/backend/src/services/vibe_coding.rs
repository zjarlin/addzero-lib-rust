use serde::{Deserialize, Serialize};

use super::terminal_sessions::{
    TerminalProfileDto, TerminalSessionCreateDto, TerminalSessionInputDto,
    TerminalSessionSnapshotDto, create_terminal_session_on_server, send_terminal_input_on_server,
};

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct StartVibeCodingRequestDto {
    #[serde(default)]
    pub profile: Option<TerminalProfileDto>,
    pub cwd: String,
    pub goal: String,
    #[serde(default)]
    pub skill_path: Option<String>,
    #[serde(default)]
    pub window_context: Option<String>,
    #[serde(default)]
    pub title: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct StartVibeCodingResponseDto {
    pub session: TerminalSessionSnapshotDto,
    pub bootstrap_prompt: String,
}

pub fn start_vibe_coding_on_server(
    input: StartVibeCodingRequestDto,
) -> Result<StartVibeCodingResponseDto, String> {
    if input.cwd.trim().is_empty() {
        return Err("cwd is required".to_string());
    }
    if input.goal.trim().is_empty() {
        return Err("goal is required".to_string());
    }

    let profile = input.profile.unwrap_or(TerminalProfileDto::Codex);
    let title = input
        .title
        .clone()
        .filter(|value| !value.trim().is_empty())
        .or_else(|| Some("Vibe Coding".to_string()));

    let session = create_terminal_session_on_server(TerminalSessionCreateDto {
        profile,
        cwd: Some(input.cwd.clone()),
        title,
        rows: None,
        cols: None,
    })?;

    let bootstrap_prompt = build_bootstrap_prompt(&input);
    let command = format!("{bootstrap_prompt}\n");
    let session = send_terminal_input_on_server(
        &session.summary.id.to_string(),
        TerminalSessionInputDto { data: command },
    )?;

    Ok(StartVibeCodingResponseDto {
        session,
        bootstrap_prompt,
    })
}

fn build_bootstrap_prompt(input: &StartVibeCodingRequestDto) -> String {
    let mut sections = Vec::new();
    sections.push(
        "You are running inside the AIO workbench as the current plugin-development coding agent."
            .to_string(),
    );
    sections.push(format!(
        "Set the working focus to this directory and work in place: {}",
        input.cwd.trim()
    ));
    if let Some(skill_path) = input
        .skill_path
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty())
    {
        sections.push(format!(
            "Before changing code, read this skill/instruction file and follow it: {}",
            skill_path
        ));
    }
    if let Some(window_context) = input
        .window_context
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        sections.push(format!(
            "Current window context from the host UI:\n{}",
            window_context
        ));
    }
    sections.push(format!("Primary goal:\n{}", input.goal.trim()));
    sections.push(
        "Stay inside the current workspace, inspect the repo before changing code, and continue the implementation directly in this terminal session."
            .to_string(),
    );

    sections.join("\n\n")
}
