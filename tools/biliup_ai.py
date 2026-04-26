#!/usr/bin/env python3
"""
AI-friendly wrapper for `biliup` CLI.

Design goals:
- stable JSON output for LLM/tooling integration
- explicit timeout/error surface
- no shell interpolation (argv only)
"""

from __future__ import annotations

import argparse
import json
import shutil
import subprocess
import sys
from dataclasses import dataclass
from typing import Any


@dataclass
class CommandResult:
    ok: bool
    command: list[str]
    exit_code: int | None
    stdout: str
    stderr: str
    error: str | None = None

    def to_dict(self) -> dict[str, Any]:
        return {
            "ok": self.ok,
            "command": self.command,
            "exit_code": self.exit_code,
            "stdout": self.stdout,
            "stderr": self.stderr,
            "error": self.error,
        }


def run_biliup(
    command: list[str],
    timeout: int,
    cwd: str | None,
) -> CommandResult:
    if shutil.which("biliup") is None:
        return CommandResult(
            ok=False,
            command=command,
            exit_code=None,
            stdout="",
            stderr="",
            error="`biliup` not found in PATH",
        )

    try:
        proc = subprocess.run(
            command,
            cwd=cwd,
            capture_output=True,
            text=True,
            timeout=timeout,
            check=False,
        )
        return CommandResult(
            ok=proc.returncode == 0,
            command=command,
            exit_code=proc.returncode,
            stdout=proc.stdout,
            stderr=proc.stderr,
        )
    except subprocess.TimeoutExpired as exc:
        return CommandResult(
            ok=False,
            command=command,
            exit_code=None,
            stdout=exc.stdout or "",
            stderr=exc.stderr or "",
            error=f"timeout after {timeout}s",
        )
    except Exception as exc:  # pragma: no cover - defensive
        return CommandResult(
            ok=False,
            command=command,
            exit_code=None,
            stdout="",
            stderr="",
            error=f"unexpected error: {exc}",
        )


def add_common_args(parser: argparse.ArgumentParser) -> None:
    parser.add_argument("--proxy", default=None, help="biliup --proxy")
    parser.add_argument(
        "--user-cookie",
        default=None,
        help="biliup --user-cookie path",
    )
    parser.add_argument(
        "--timeout",
        type=int,
        default=120,
        help="subprocess timeout in seconds",
    )
    parser.add_argument(
        "--cwd",
        default=None,
        help="working directory for command execution",
    )


def inject_common_flags(args: argparse.Namespace, cmd: list[str]) -> list[str]:
    if args.proxy:
        cmd.extend(["--proxy", args.proxy])
    if args.user_cookie:
        cmd.extend(["--user-cookie", args.user_cookie])
    return cmd


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(
        description="AI-friendly wrapper for biliup CLI (JSON output)"
    )
    subparsers = parser.add_subparsers(dest="action", required=True)

    login = subparsers.add_parser("login", help="biliup login")
    add_common_args(login)

    renew = subparsers.add_parser("renew", help="biliup renew")
    add_common_args(renew)

    show = subparsers.add_parser("show", help="biliup show <vid>")
    add_common_args(show)
    show.add_argument("--vid", required=True, help="av or bv id")

    upload = subparsers.add_parser("upload", help="biliup upload")
    add_common_args(upload)
    upload.add_argument("--video-path", action="append", default=[], help="video path")
    upload.add_argument("--config", default=None, help="config file path")
    upload.add_argument("--line", default=None, help="upload line")
    upload.add_argument("--limit", type=int, default=None, help="single file concurrency")
    upload.add_argument("--submit", default=None, help="submit option")

    list_parser = subparsers.add_parser("list", help="biliup list")
    add_common_args(list_parser)
    list_parser.add_argument("--is-pubing", action="store_true")
    list_parser.add_argument("--pubed", action="store_true")
    list_parser.add_argument("--not-pubed", action="store_true")
    list_parser.add_argument("--from-page", type=int, default=None)
    list_parser.add_argument("--max-pages", type=int, default=None)

    download = subparsers.add_parser("download", help="biliup download")
    add_common_args(download)
    download.add_argument("--url", required=True)
    download.add_argument("--output", default=None)
    download.add_argument("--split-size", default=None)
    download.add_argument("--split-time", default=None)

    server = subparsers.add_parser("server", help="biliup server")
    add_common_args(server)
    server.add_argument("--bind", default=None)
    server.add_argument("--port", type=int, default=None)
    server.add_argument("--auth", action="store_true")

    raw = subparsers.add_parser(
        "raw", help="direct pass-through after `--` (e.g. raw -- upload a.mp4)"
    )
    raw.add_argument("--timeout", type=int, default=120)
    raw.add_argument("--cwd", default=None)
    raw.add_argument("args", nargs=argparse.REMAINDER, help="raw args after --")
    return parser


def command_from_args(args: argparse.Namespace) -> list[str]:
    cmd = ["biliup"]
    action = args.action

    if action == "raw":
        raw_args = args.args
        if raw_args and raw_args[0] == "--":
            raw_args = raw_args[1:]
        return cmd + raw_args

    cmd = inject_common_flags(args, cmd)

    if action == "login":
        cmd.append("login")
    elif action == "renew":
        cmd.append("renew")
    elif action == "show":
        cmd.extend(["show", args.vid])
    elif action == "upload":
        cmd.append("upload")
        if args.submit:
            cmd.extend(["--submit", args.submit])
        if args.config:
            cmd.extend(["--config", args.config])
        if args.line:
            cmd.extend(["--line", args.line])
        if args.limit is not None:
            cmd.extend(["--limit", str(args.limit)])
        cmd.extend(args.video_path)
    elif action == "list":
        cmd.append("list")
        if args.is_pubing:
            cmd.append("--is-pubing")
        if args.pubed:
            cmd.append("--pubed")
        if args.not_pubed:
            cmd.append("--not-pubed")
        if args.from_page is not None:
            cmd.extend(["--from-page", str(args.from_page)])
        if args.max_pages is not None:
            cmd.extend(["--max-pages", str(args.max_pages)])
    elif action == "download":
        cmd.extend(["download", args.url])
        if args.output:
            cmd.extend(["--output", args.output])
        if args.split_size:
            cmd.extend(["--split-size", args.split_size])
        if args.split_time:
            cmd.extend(["--split-time", args.split_time])
    elif action == "server":
        cmd.append("server")
        if args.bind:
            cmd.extend(["--bind", args.bind])
        if args.port is not None:
            cmd.extend(["--port", str(args.port)])
        if args.auth:
            cmd.append("--auth")
    else:
        raise ValueError(f"unsupported action: {action}")

    return cmd


def main() -> int:
    parser = build_parser()
    args = parser.parse_args()
    command = command_from_args(args)
    timeout = getattr(args, "timeout", 120)
    cwd = getattr(args, "cwd", None)
    result = run_biliup(command=command, timeout=timeout, cwd=cwd)
    print(json.dumps(result.to_dict(), ensure_ascii=False))
    return 0 if result.ok else 1


if __name__ == "__main__":
    raise SystemExit(main())
