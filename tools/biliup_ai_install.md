# biliup AI Wrapper Setup

This document explains how to prepare and use `tools/biliup_ai.py` in this repository.

## 1) Install `biliup`

`biliup_ai.py` only wraps the CLI. You still need `biliup` installed in your shell PATH.

Recommended:

```bash
uv tool install biliup
```

Alternative:

```bash
pipx install biliup
```

## 2) Verify PATH

Run:

```bash
biliup --version
```

If this fails, ensure the install bin path is in PATH.

Typical zsh additions (`~/.zshrc`):

```bash
export PATH="$HOME/.local/bin:$PATH"
export PATH="$HOME/.cargo/bin:$PATH"
```

Then reload:

```bash
source ~/.zshrc
```

## 3) Wrapper quick check

From repository root:

```bash
python3 tools/biliup_ai.py --help
python3 tools/biliup_ai.py raw -- --version
```

Expected: JSON output with `ok=true` for the version command.

## 4) Cookie file convention

Use one stable cookie file path, for example:

- `./cookies.json` (repo-local)
- or `~/.config/biliup/cookies.json` (global)

When running wrapper commands, pass:

```bash
--user-cookie /absolute/or/relative/path/to/cookies.json
```

## 5) Common commands

Login:

```bash
python3 tools/biliup_ai.py login --user-cookie cookies.json
```

Upload:

```bash
python3 tools/biliup_ai.py upload \
  --video-path /path/to/video.mp4 \
  --limit 3 \
  --user-cookie cookies.json
```

List videos:

```bash
python3 tools/biliup_ai.py list --from-page 1 --max-pages 2 --user-cookie cookies.json
```

Pass-through mode (for new CLI params):

```bash
python3 tools/biliup_ai.py raw -- upload /path/to/video.mp4 --limit 3
```

## 6) AI prompt template

Use this stable instruction when asking AI to operate biliup:

```text
请使用 `python3 tools/biliup_ai.py` 调用 biliup，返回完整 JSON（ok/command/exit_code/stdout/stderr/error），不要省略 stderr。
```

## 7) Troubleshooting

- `error: "biliup not found in PATH"`  
Install `biliup` and fix PATH, then re-open terminal.
- timeout errors  
Increase wrapper timeout with `--timeout`, for example `--timeout 300`.
- login issues  
Try running a direct command once: `biliup login`.

