#!/usr/bin/env python3
"""Run PaddleOCR PP-StructureV3 and normalize document parsing artifacts."""

from __future__ import annotations

import argparse
import json
import shutil
import sys
from pathlib import Path
from typing import Any


def parse_bool(value: str) -> bool:
    normalized = value.strip().lower()
    if normalized in {"1", "true", "yes", "y", "on"}:
        return True
    if normalized in {"0", "false", "no", "n", "off"}:
        return False
    raise argparse.ArgumentTypeError(f"invalid bool value: {value!r}")


def jsonable(value: Any) -> Any:
    if value is None or isinstance(value, (bool, int, float, str)):
        return value
    if isinstance(value, Path):
        return str(value)
    if isinstance(value, dict):
        return {str(key): jsonable(item) for key, item in value.items()}
    if isinstance(value, (list, tuple, set)):
        return [jsonable(item) for item in value]
    if hasattr(value, "tolist"):
        return jsonable(value.tolist())
    try:
        json.dumps(value)
        return value
    except TypeError:
        return str(value)


def find_files(root: Path, suffix: str) -> list[Path]:
    return sorted(path for path in root.rglob(f"*{suffix}") if path.is_file())


def normalize_markdown(output_dir: Path) -> tuple[Path, list[str]]:
    markdown_file = output_dir / "document.md"
    native_markdown_files = [
        path for path in find_files(output_dir, ".md") if path.resolve() != markdown_file.resolve()
    ]
    parts = []
    for path in native_markdown_files:
        text = path.read_text(encoding="utf-8").strip()
        if text:
            parts.append(text)
    markdown_file.write_text("\n\n".join(parts).strip() + "\n", encoding="utf-8")
    return markdown_file, [str(path.relative_to(output_dir)) for path in native_markdown_files]


def normalize_json(
    input_path: Path,
    output_dir: Path,
    result_json_values: list[Any],
    native_json_files: list[Path],
) -> Path:
    structured_file = output_dir / "structured.json"
    native_json_values = []
    for path in native_json_files:
        try:
            native_json_values.append(json.loads(path.read_text(encoding="utf-8")))
        except json.JSONDecodeError:
            native_json_values.append({"path": str(path), "raw": path.read_text(encoding="utf-8")})
    payload = {
        "engine": "paddleocr-pp-structure-v3",
        "input_path": str(input_path),
        "pages": result_json_values,
        "native_json_files": [str(path.relative_to(output_dir)) for path in native_json_files],
        "native_json": native_json_values,
    }
    structured_file.write_text(
        json.dumps(jsonable(payload), ensure_ascii=False, indent=2),
        encoding="utf-8",
    )
    return structured_file


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--input", required=True)
    parser.add_argument("--output-dir", required=True)
    parser.add_argument("--use-doc-orientation-classify", type=parse_bool, default=False)
    parser.add_argument("--use-doc-unwarping", type=parse_bool, default=False)
    parser.add_argument("--use-textline-orientation", type=parse_bool, default=False)
    parser.add_argument("--use-table-recognition", type=parse_bool, default=True)
    parser.add_argument("--use-formula-recognition", type=parse_bool, default=True)
    parser.add_argument("--use-chart-recognition", type=parse_bool, default=False)
    parser.add_argument("--use-region-detection", type=parse_bool, default=True)
    args, extra_args = parser.parse_known_args()

    input_path = Path(args.input).resolve()
    output_dir = Path(args.output_dir).resolve()
    output_dir.mkdir(parents=True, exist_ok=True)

    try:
        from paddleocr import PPStructureV3
    except Exception as exc:  # noqa: BLE001 - bridge should report import failures verbosely.
        print(
            "PaddleOCR PPStructureV3 is unavailable. Install PaddleOCR with the doc-parser "
            "dependencies in a compatible Python environment before running this bridge.",
            file=sys.stderr,
        )
        print(f"import error: {exc}", file=sys.stderr)
        return 2

    if extra_args:
        print(f"unsupported extra arguments: {extra_args}", file=sys.stderr)
        return 2

    pipeline = PPStructureV3(
        use_doc_orientation_classify=args.use_doc_orientation_classify,
        use_doc_unwarping=args.use_doc_unwarping,
        use_textline_orientation=args.use_textline_orientation,
        use_table_recognition=args.use_table_recognition,
        use_formula_recognition=args.use_formula_recognition,
        use_chart_recognition=args.use_chart_recognition,
        use_region_detection=args.use_region_detection,
    )
    results = list(pipeline.predict(input=str(input_path)))
    result_json_values = []
    for result in results:
        if hasattr(result, "save_to_json"):
            result.save_to_json(save_path=str(output_dir))
        if hasattr(result, "save_to_markdown"):
            result.save_to_markdown(save_path=str(output_dir))
        result_json_values.append(jsonable(getattr(result, "json", None)))

    native_json_files = [
        path
        for path in find_files(output_dir, ".json")
        if path.name not in {"structured.json", "manifest.json"}
    ]
    markdown_file, native_markdown_files = normalize_markdown(output_dir)
    structured_file = normalize_json(input_path, output_dir, result_json_values, native_json_files)
    manifest_file = output_dir / "manifest.json"
    artifact_files = sorted(
        str(path.relative_to(output_dir))
        for path in output_dir.rglob("*")
        if path.is_file()
    )
    manifest_file.write_text(
        json.dumps(
            {
                "engine": "paddleocr-pp-structure-v3",
                "input_path": str(input_path),
                "markdown_file": str(markdown_file),
                "structured_json_file": str(structured_file),
                "native_markdown_files": native_markdown_files,
                "native_json_files": [str(path.relative_to(output_dir)) for path in native_json_files],
                "artifact_files": artifact_files,
            },
            ensure_ascii=False,
            indent=2,
        ),
        encoding="utf-8",
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
