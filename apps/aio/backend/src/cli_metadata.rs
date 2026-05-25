use std::fmt::Write;

use az_derive_aliases::{apply, serialize_copy_eq};

#[apply(serialize_copy_eq)]
pub struct AioCommandMetadata {
    pub name: &'static str,
    pub summary: &'static str,
    pub example: &'static str,
}

macro_rules! aio_command_metadata {
    ($(($name:literal, $summary:literal, $example:literal)),+ $(,)?) => {
        pub const AIO_COMMANDS: &[AioCommandMetadata] = &[
            $(
                AioCommandMetadata {
                    name: $name,
                    summary: $summary,
                    example: $example,
                },
            )+
        ];
    };
}

aio_command_metadata! {
    ("reg", "注册 AIO 用户", "aio reg --username zjarlin --password-stdin"),
    ("login", "登录并准备当前用户的 Drive API key", "aio login --username zjarlin --password-stdin"),
    ("logout", "清除本机登录态", "aio logout"),
    ("whoami", "查看当前登录用户", "aio whoami"),
    ("key", "管理 API key 和融合授权", "aio key list"),
    ("drive", "托管、同步和查看 Drive 文件", "aio drive ls --format json"),
    ("cli", "管理 CLI 元数据、skill.sh、shell 组件和外部 CLI", "aio cli component list"),
    ("serve", "启动 AIO API 服务", "aio serve"),
    ("migrate", "运行数据库迁移", "aio migrate"),
    ("status", "打印当前架构状态", "aio status"),
    ("system", "面向 agent 的系统治理命令", "aio system docs"),
}

pub fn render_metadata_table() -> String {
    let mut out = String::from("COMMAND      SUMMARY\n");
    for command in AIO_COMMANDS {
        let _ = writeln!(out, "{:<12} {}", command.name, command.summary);
    }
    out
}

pub fn render_skill_sh() -> String {
    let mut help = String::from("AIO CLI Skill\n\n内置命令元数据：\n");
    for command in AIO_COMMANDS {
        let _ = writeln!(
            help,
            "  {:<10} {}\n             示例: {}",
            command.name, command.summary, command.example
        );
    }
    help.push_str(
        "\nShell 组件：\n  aio cli component list\n  aio cli component upsert NAME --kind export --value VALUE\n  aio cli component set NAME --enabled false\n  aio cli component build\n\n外部 CLI：\n  aio cli add NAME --command \"program [args]\" [--arg value] [--env KEY=VALUE]\n  aio cli list\n  aio cli run NAME -- ARGS...\n  aio cli remove NAME\n",
    );

    format!(
        r#"#!/usr/bin/env bash
set -euo pipefail

AIO_BIN="${{AIO_BIN:-aio}}"

aio_skill_help() {{
  cat <<'AIO_SKILL_HELP'
{help}AIO_SKILL_HELP
}}

case "${{1:-help}}" in
  help|-h|--help)
    aio_skill_help
    ;;
  metadata)
    shift
    "$AIO_BIN" cli metadata "${{@}}"
    ;;
  add)
    shift
    "$AIO_BIN" cli add "${{@}}"
    ;;
  list)
    shift
    "$AIO_BIN" cli list "${{@}}"
    ;;
  run)
    shift
    "$AIO_BIN" cli run "${{@}}"
    ;;
  remove)
    shift
    "$AIO_BIN" cli remove "${{@}}"
    ;;
  *)
    "$AIO_BIN" "${{@}}"
    ;;
esac
"#
    )
}

pub fn render_skill_md() -> String {
    let mut body = String::from(
        r#"---
name: aio-cli
description: Use when extending or operating the local AIO CLI, including Drive, API key auth, generated command metadata, and locally registered external CLIs.
allowed-tools: Bash(aio:*), Bash(skill.sh:*)
---

# AIO CLI

Use `skill.sh` in this directory as the stable shell entrypoint for agent-side AIO CLI operations.

Generated command metadata:

"#,
    );
    for command in AIO_COMMANDS {
        let _ = writeln!(
            body,
            "- `{}`: {}. Example: `{}`",
            command.name, command.summary, command.example
        );
    }
    body.push_str(
        "\nShell components are managed with `aio cli component list`, `upsert`, `set`, and `build`. External CLIs are managed with `aio cli add`, `aio cli list`, `aio cli run`, and `aio cli remove`.\n",
    );
    body
}

#[cfg(test)]
mod tests {
    use super::{AIO_COMMANDS, render_skill_sh};

    #[test]
    fn metadata_should_not_include_tui() {
        let names: Vec<_> = AIO_COMMANDS.iter().map(|command| command.name).collect();
        assert!(!names.contains(&"tui"));
        assert!(names.contains(&"cli"));
    }

    #[test]
    fn skill_script_should_expose_external_cli_commands() {
        let script = render_skill_sh();
        assert!(script.contains("aio cli add NAME"));
        assert!(script.contains("\"$AIO_BIN\" cli run"));
    }
}
