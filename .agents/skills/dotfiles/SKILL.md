---
name: dotfiles
description: "Use when managing this user's dotfiles CLI, syncing dotfiles, installing configured packages, checking ports, or running one-click setup tasks migrated from /Users/zjarlin/IdeaProjects/dotfiles-cli-graalvm. The AI chooses and executes the appropriate dotfiles CLI command, with confirmation for destructive or privileged operations."
---

# Dotfiles CLI

Use the Rust app in `/Users/zjarlin/IdeaProjects/zjarlin/addzero-lib-rust/apps/dotfiles`.

Prefer the installed `dotfiles` binary if available. Otherwise run from the repo:

```bash
cargo run -q -p dotfiles -- <args>
```

## Core Commands

- Print config: `dotfiles config cat` or legacy `dotfiles cat-config`
- Sync dotfiles: `dotfiles sync`
- Sync and replace local files with links only after confirmation: `dotfiles sync --force-links`
- Push sync dir: `dotfiles push -m "Update dotfiles"`
- Add links: `dotfiles add-dotfiles <path...> [--abs]`
- Remove links: `dotfiles remove-dotfiles <path...>` or `dotfiles rm-dotfiles <path...>`
- Add packages: `dotfiles add-pkg <pkg...>`
- Remove packages: `dotfiles rm-pkg <pkg...>`
- Install configured packages: `dotfiles package install`
- Port inspection: `dotfiles show-port <port>`
- Kill port process only after confirmation: `dotfiles kill-port <port>`
- Init baseline: `dotfiles init`

## One-Click Tasks

Use `dotfiles oneclick <task>` for migrated GraalVM CLI init tasks. `task` and `auto-init` are aliases.

List tasks first when unsure:

```bash
dotfiles oneclick list
```

Main tasks:

- `all`: run the migrated init task set
- `env-scripts`: install shell/PowerShell `setenv` helper
- `git`, `node`, `jdk`, `pnpm`
- `pkg`: install the current package manager
- `pkg-manager`: install configured default packages
- `graalvm`, `final-shell`, `idea`, `zulu-jdk`
- `powershell`, `powershell-env`
- `quark`, `docker`, `lazyvim`, `homebrew`, `ohmyzsh`, `macos`
- `enable-all-sources`, `keji`

Use `--dry-run` before broad or privileged setup:

```bash
dotfiles oneclick --dry-run all
```

Use `--yes` only after the user explicitly approves the exact task or workflow:

```bash
dotfiles oneclick --yes pnpm
```

## AI Execution Rules

- The AI owns sequencing: inspect status/config first, choose the smallest command that satisfies the request, run it, then report the result.
- Confirm before commands that install software, run `curl | shell`, change system defaults, edit shell profiles, kill processes, use sudo, overwrite links, or push to git.
- Prefer focused one-click tasks over `oneclick all` unless the user asks for full machine bootstrap.
- Use `--dry-run` when translating vague setup requests into an execution plan.
- If a task is platform-gated and skipped, report that it was skipped rather than treating it as a failure.

## Validation

After changing this CLI, run:

```bash
cargo fmt -p dotfiles
cargo check -p dotfiles
cargo test -p dotfiles
cargo run -q -p dotfiles -- --help
cargo run -q -p dotfiles -- oneclick --help
```
