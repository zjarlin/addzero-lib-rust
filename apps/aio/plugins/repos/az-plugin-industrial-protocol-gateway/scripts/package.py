#!/usr/bin/env python3
"""Build an AIO `.azplugin` package for the industrial gateway plugin."""

from __future__ import annotations

import argparse
import hashlib
import shutil
import subprocess
import sys
import zipfile
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
DEFAULT_CATALOG = ROOT.parents[1] / "catalog"


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument(
        "--out",
        type=Path,
        default=DEFAULT_CATALOG / "industrial-protocol-gateway.azplugin",
        help="Output .azplugin path.",
    )
    parser.add_argument(
        "--skip-cargo",
        action="store_true",
        help="Use the embedded lifecycle-only wasm module instead of cargo build.",
    )
    args = parser.parse_args()

    stage = ROOT / "target" / "azplugin"
    if stage.exists():
        shutil.rmtree(stage)
    (stage / "backend").mkdir(parents=True)
    (stage / "assets").mkdir(parents=True)

    shutil.copy2(ROOT / "plugin.toml", stage / "plugin.toml")
    shutil.copy2(ROOT / "assets" / "gateway-profile.json", stage / "assets" / "gateway-profile.json")
    write_wasm(stage / "backend" / "plugin.wasm", args.skip_cargo)
    write_checksums(stage)
    write_package(stage, args.out)
    print(args.out)
    return 0


def write_wasm(target: Path, skip_cargo: bool) -> None:
    if skip_cargo or not wasm_target_installed():
        target.write_bytes(compile_fallback_wasm())
        return

    subprocess.run(
        [
            "cargo",
            "build",
            "--manifest-path",
            str(ROOT / "Cargo.toml"),
            "--release",
            "--target",
            "wasm32-unknown-unknown",
        ],
        check=True,
    )
    built = (
        ROOT
        / "target"
        / "wasm32-unknown-unknown"
        / "release"
        / "az_plugin_industrial_protocol_gateway.wasm"
    )
    shutil.copy2(built, target)


def compile_fallback_wasm() -> bytes:
    exports = [
        "aio_on_load",
        "aio_on_enable",
        "aio_on_disable",
        "aio_on_unload",
    ]
    header = b"\0asm" + bytes([1, 0, 0, 0])
    type_section = wasm_section(1, wasm_vec([bytes([0x60]) + wasm_vec([]) + wasm_vec([0x7F])]))
    function_section = wasm_section(3, wasm_vec([bytes([0]) for _ in exports]))
    export_entries = [
        wasm_name(name) + bytes([0x00]) + encode_u32(index)
        for index, name in enumerate(exports)
    ]
    export_section = wasm_section(7, wasm_vec(export_entries))
    body = bytes([0x00, 0x41, 0x00, 0x0B])
    code_section = wasm_section(
        10,
        wasm_vec([encode_u32(len(body)) + body for _ in exports]),
    )
    return header + type_section + function_section + export_section + code_section


def wasm_section(section_id: int, payload: bytes) -> bytes:
    return bytes([section_id]) + encode_u32(len(payload)) + payload


def wasm_vec(items: list[bytes | int]) -> bytes:
    payload = bytearray(encode_u32(len(items)))
    for item in items:
        if isinstance(item, int):
            payload.extend(encode_u32(item))
        else:
            payload.extend(item)
    return bytes(payload)


def wasm_name(value: str) -> bytes:
    encoded = value.encode("utf-8")
    return encode_u32(len(encoded)) + encoded


def encode_u32(value: int) -> bytes:
    encoded = bytearray()
    while True:
        byte = value & 0x7F
        value >>= 7
        if value:
            encoded.append(byte | 0x80)
        else:
            encoded.append(byte)
            return bytes(encoded)


def wasm_target_installed() -> bool:
    try:
        result = subprocess.run(
            ["rustup", "target", "list", "--installed"],
            check=True,
            capture_output=True,
            text=True,
        )
    except (FileNotFoundError, subprocess.CalledProcessError):
        return False
    return "wasm32-unknown-unknown" in result.stdout.splitlines()


def write_checksums(stage: Path) -> None:
    entries = [
        path
        for path in sorted(stage.rglob("*"))
        if path.is_file() and path.name != "checksums.sha256"
    ]
    lines = []
    for path in entries:
        relative = path.relative_to(stage).as_posix()
        digest = hashlib.sha256(path.read_bytes()).hexdigest()
        lines.append(f"{digest}  {relative}\n")
    (stage / "checksums.sha256").write_text("".join(lines), encoding="utf-8")


def write_package(stage: Path, output: Path) -> None:
    output.parent.mkdir(parents=True, exist_ok=True)
    if output.exists():
        output.unlink()
    entries = [path for path in sorted(stage.rglob("*")) if path.is_file()]
    with zipfile.ZipFile(output, "w", compression=zipfile.ZIP_DEFLATED) as archive:
        for path in entries:
            archive.write(path, path.relative_to(stage).as_posix())


if __name__ == "__main__":
    sys.exit(main())
