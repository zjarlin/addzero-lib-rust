#!/usr/bin/env python3
"""Generate typed Rust OpenAI REST contracts from openai/openai-openapi."""

from __future__ import annotations

import argparse
import json
import keyword
import os
import re
import subprocess
import sys
import textwrap
import urllib.error
import urllib.request
from dataclasses import dataclass
from pathlib import Path
from typing import Any

import yaml


SOURCE_REPOSITORY = "https://github.com/openai/openai-openapi"
DEFAULT_SOURCE_SPEC_URL = "https://raw.githubusercontent.com/openai/openai-openapi/master/openapi.yaml"
BASE_URL = "https://api.openai.com/v1/"
HEADER = "// Generated from OpenAPI spec. Do not edit by hand."
SPEC_URL_ENV = "AZ_OPENAI_OPENAPI_SPEC_URL"

HTTP_METHODS = ("get", "post", "put", "delete", "patch")
RUST_KEYWORDS = {
    "as",
    "break",
    "const",
    "continue",
    "crate",
    "else",
    "enum",
    "extern",
    "false",
    "fn",
    "for",
    "if",
    "impl",
    "in",
    "let",
    "loop",
    "match",
    "mod",
    "move",
    "mut",
    "pub",
    "ref",
    "return",
    "self",
    "Self",
    "static",
    "struct",
    "super",
    "trait",
    "true",
    "type",
    "unsafe",
    "use",
    "where",
    "while",
    "async",
    "await",
    "dyn",
    "abstract",
    "become",
    "box",
    "do",
    "final",
    "macro",
    "override",
    "priv",
    "typeof",
    "unsized",
    "virtual",
    "yield",
    "try",
}

PRIMITIVE_TYPES = {
    "String",
    "i32",
    "i64",
    "f32",
    "f64",
    "bool",
    "()",
    "OpenAiBinaryBody",
    "OpenAiTextBody",
    "OpenAiJsonValue",
    "OpenAiJsonObject",
}
BODY_TYPES = {
    "OpenAiBinaryBody",
    "OpenAiTextBody",
    "OpenAiJsonValue",
    "OpenAiJsonObject",
}


@dataclass(frozen=True)
class RustType:
    code: str
    optional: bool = False

    def rendered(self, force_optional: bool = False) -> str:
        optional = self.optional or force_optional
        if optional and not self.code.startswith("Option<"):
            return f"Option<{self.code}>"
        return self.code

    def plain(self) -> str:
        if self.code.startswith("Option<") and self.code.endswith(">"):
            return self.code.removeprefix("Option<")[:-1]
        return self.code


@dataclass
class PropertyDef:
    original_name: str
    name: str
    type_ref: RustType
    required: bool
    description: str | None


@dataclass
class VariantDef:
    name: str
    type_ref: RustType


@dataclass
class ModelDef:
    name: str
    kind: str
    properties: list[PropertyDef]
    alias: RustType | None
    variants: list[VariantDef]
    description: str | None


@dataclass
class ParameterDef:
    original_name: str
    name: str
    location: str
    type_ref: RustType
    required: bool


@dataclass
class OperationDef:
    function_name: str
    http_method: str
    path: str
    path_const: str
    summary: str | None
    parameters: list[ParameterDef]
    request_body: RustType | None
    request_body_required: bool
    response_type: RustType


@dataclass
class InterfaceDef:
    name: str
    module_name: str
    operations: list[OperationDef]


@dataclass(frozen=True)
class LoadedSpec:
    spec: dict[str, Any]
    source_spec_location: str


def load_spec(spec_file: str | None, spec_url: str) -> LoadedSpec:
    if spec_file:
        with open(spec_file, "r", encoding="utf-8") as handle:
            text = handle.read()
        return LoadedSpec(
            spec=parse_spec_text(text, spec_file),
            source_spec_location=str(Path(spec_file)),
        )

    try:
        with urllib.request.urlopen(spec_url) as response:
            text = response.read().decode("utf-8")
    except urllib.error.HTTPError as exc:
        raise SystemExit(f"failed to fetch OpenAPI spec from {spec_url}: HTTP {exc.code}") from exc
    except urllib.error.URLError as exc:
        raise SystemExit(f"failed to fetch OpenAPI spec from {spec_url}: {exc.reason}") from exc
    return LoadedSpec(spec=parse_spec_text(text, spec_url), source_spec_location=spec_url)


def parse_spec_text(text: str, source_name: str) -> dict[str, Any]:
    suffix = Path(source_name).suffix.lower()
    if suffix == ".json":
        return json.loads(text)
    return yaml.safe_load(text)


def current_openapi_commit(repository: str, ref: str) -> str:
    result = subprocess.run(
        ["git", "ls-remote", git_remote_url(repository), ref],
        check=True,
        text=True,
        capture_output=True,
    )
    return result.stdout.split()[0]


def git_remote_url(repository: str) -> str:
    if repository.endswith(".git"):
        return repository
    return f"{repository}.git"


def split_words(value: str) -> list[str]:
    value = re.sub(r"([a-z0-9])([A-Z])", r"\1 \2", value)
    value = re.sub(r"[^A-Za-z0-9]+", " ", value)
    return [part for part in value.strip().split() if part]


def pascal_case(value: str) -> str:
    words = split_words(value)
    if not words:
        return "Value"
    return "".join(word[:1].upper() + word[1:] for word in words)


def snake_case(value: str) -> str:
    words = split_words(value)
    if not words:
        return "value"
    return sanitize_identifier("_".join(word.lower() for word in words))


def sanitize_identifier(value: str) -> str:
    if not value:
        return "value"
    value = re.sub(r"[^A-Za-z0-9_]", "_", value)
    if re.match(r"^[0-9]", value):
        value = f"value_{value}"
    if value in RUST_KEYWORDS or keyword.iskeyword(value):
        return f"{value}_value"
    return value


def path_const_name(path: str) -> str:
    segments = [segment for segment in path.strip("/").split("/") if segment]
    names = []
    for segment in segments:
        clean = segment.strip("{}")
        words = split_words(clean)
        names.append("_".join(word.upper() for word in words))
    return "_BY_".join(names) if names else "ROOT"


def rust_string(value: str) -> str:
    return '"' + value.replace("\\", "\\\\").replace('"', '\\"').replace("\n", "\\n") + '"'


def doc_lines(text: str | None, indent: str = "") -> list[str]:
    if not text:
        return []
    cleaned = re.sub(r"\s+", " ", text.replace("*/", "* /")).strip()
    if not cleaned:
        return []
    return [f"{indent}/// {line}" for line in textwrap.wrap(cleaned, width=100)]


def file_stem_for_model(name: str) -> str:
    return snake_case(name)


class OpenAiRustGenerator:
    def __init__(
        self,
        spec: dict[str, Any],
        source_repository: str,
        source_spec_location: str,
        source_commit: str,
    ) -> None:
        self.spec = spec
        self.source_repository = source_repository
        self.source_spec_location = source_spec_location
        self.source_commit = source_commit
        self.schemas: dict[str, dict[str, Any]] = spec.get("components", {}).get("schemas", {})
        self.models: dict[str, ModelDef] = {}
        self.model_names: set[str] = set()
        self.nullable_aliases: dict[str, bool] = {}
        self.processing: set[str] = set()

    def generate(self, crate_dir: Path) -> None:
        src_dir = crate_dir / "src"
        api_dir = src_dir / "api"
        models_dir = src_dir / "models"
        api_dir.mkdir(parents=True, exist_ok=True)
        models_dir.mkdir(parents=True, exist_ok=True)

        for rs_file in api_dir.glob("*.rs"):
            rs_file.unlink()
        for rs_file in models_dir.glob("*.rs"):
            rs_file.unlink()

        for name, schema in self.schemas.items():
            self.ensure_component_model(name, schema)

        interfaces = self.parse_interfaces()

        self.write_bodies(src_dir / "bodies.rs")
        self.write_paths(src_dir / "paths.rs")
        self.write_api_entry(src_dir / "api.rs", interfaces)
        self.write_models_entry(src_dir / "models.rs")
        for interface in interfaces:
            self.write_interface(api_dir / f"{interface.module_name}.rs", interface)
        for model in sorted(self.models.values(), key=lambda item: item.name):
            self.write_model(models_dir / f"{file_stem_for_model(model.name)}.rs", model)

    def ensure_component_model(self, name: str, schema: dict[str, Any]) -> RustType:
        model_name = pascal_case(name)
        if model_name in self.models:
            return RustType(model_name)
        if model_name in self.processing:
            return RustType(model_name)

        self.processing.add(model_name)
        self.model_names.add(model_name)
        model = self.build_model(model_name, schema)
        self.models[model_name] = model
        self.processing.remove(model_name)
        return RustType(model_name)

    def build_model(self, name: str, schema: dict[str, Any]) -> ModelDef:
        union_schema = self.union_schema(schema)
        object_schema = self.object_schema(schema)
        if union_schema is not None and object_schema is None:
            variants = self.collect_union_variants(union_schema, name)
            if variants:
                return ModelDef(
                    name=name,
                    kind="enum",
                    properties=[],
                    alias=None,
                    variants=variants,
                    description=schema.get("description") or schema.get("title"),
                )

        if object_schema is not None:
            properties = self.collect_properties(object_schema, name)
            return ModelDef(
                name=name,
                kind="data",
                properties=properties,
                alias=None,
                variants=[],
                description=schema.get("description") or schema.get("title"),
            )

        alias = self.map_schema(schema, name)
        self.nullable_aliases[name] = alias.optional
        return ModelDef(
            name=name,
            kind="alias",
            properties=[],
            alias=RustType(alias.code, optional=False),
            variants=[],
            description=schema.get("description") or schema.get("title"),
        )

    def object_schema(self, schema: dict[str, Any]) -> dict[str, Any] | None:
        if schema.get("properties") or schema.get("allOf"):
            return schema
        schema_type = schema.get("type")
        if schema_type == "object" and not schema.get("additionalProperties"):
            return schema
        return None

    def union_schema(self, schema: dict[str, Any]) -> list[dict[str, Any]] | None:
        alternatives = self.non_null_alternatives(schema.get("oneOf") or schema.get("anyOf"))
        if len(alternatives) > 1:
            return alternatives
        return None

    def collect_union_variants(self, alternatives: list[dict[str, Any]], model_name: str) -> list[VariantDef]:
        variants: list[VariantDef] = []
        used: set[str] = set()
        for index, item in enumerate(alternatives, start=1):
            variant_name = self.variant_name(item, index)
            while variant_name in used:
                variant_name = f"{variant_name}{index}"
            used.add(variant_name)
            mapped = self.map_schema(item, f"{model_name}{variant_name}")
            variants.append(VariantDef(name=variant_name, type_ref=RustType(mapped.code)))
        return variants

    def variant_name(self, schema: dict[str, Any], index: int) -> str:
        if "$ref" in schema:
            return pascal_case(schema["$ref"].split("/")[-1])
        title = schema.get("title")
        if isinstance(title, str) and title.strip():
            return pascal_case(title)
        const_value = schema.get("const")
        if isinstance(const_value, str) and const_value.strip():
            return pascal_case(const_value)
        enum_values = schema.get("enum")
        if isinstance(enum_values, list) and len(enum_values) == 1 and isinstance(enum_values[0], str):
            return pascal_case(enum_values[0])
        schema_type = schema.get("type")
        if schema_type == "string":
            return "String"
        if schema_type == "integer":
            return "Integer"
        if schema_type == "number":
            return "Number"
        if schema_type == "boolean":
            return "Boolean"
        if schema_type == "array":
            return "Array"
        if schema_type == "object" or schema.get("properties"):
            return "Object"
        return f"Variant{index}"

    def collect_properties(self, schema: dict[str, Any], model_name: str) -> list[PropertyDef]:
        merged: dict[str, PropertyDef] = {}
        for required_props, prop_map in self.iter_object_properties(schema):
            for original_name, prop_schema in prop_map.items():
                if original_name in merged:
                    continue
                property_name = snake_case(original_name)
                property_type = self.map_schema(prop_schema, f"{model_name}{pascal_case(original_name)}")
                merged[original_name] = PropertyDef(
                    original_name=original_name,
                    name=property_name,
                    type_ref=property_type,
                    required=original_name in required_props,
                    description=prop_schema.get("description") or prop_schema.get("title"),
                )
        return list(merged.values())

    def iter_object_properties(
        self,
        schema: dict[str, Any],
    ) -> list[tuple[set[str], dict[str, dict[str, Any]]]]:
        if "$ref" in schema:
            ref_schema = self.resolve_ref(schema["$ref"])
            return self.iter_object_properties(ref_schema)

        if schema.get("allOf"):
            result: list[tuple[set[str], dict[str, dict[str, Any]]]] = []
            parent_required = set(schema.get("required") or [])
            for item in schema["allOf"]:
                for required, props in self.iter_object_properties(item):
                    result.append((parent_required | required, props))
            return result

        alternatives = self.non_null_alternatives(schema.get("oneOf") or schema.get("anyOf"))
        if len(alternatives) == 1:
            return self.iter_object_properties(alternatives[0])

        props = schema.get("properties") or {}
        required = set(schema.get("required") or [])
        return [(required, props)]

    def map_schema(self, schema: dict[str, Any] | None, suggested_name: str) -> RustType:
        if not schema:
            return RustType("OpenAiJsonValue")

        schema = dict(schema)
        optional = bool(schema.pop("nullable", False))

        schema_type = schema.get("type")
        if isinstance(schema_type, list):
            optional = optional or "null" in schema_type
            non_null_types = [item for item in schema_type if item != "null"]
            schema["type"] = non_null_types[0] if len(non_null_types) == 1 else None

        if "$ref" in schema:
            ref_raw = schema["$ref"].split("/")[-1]
            ref_name = pascal_case(ref_raw)
            ref_schema = self.schemas.get(ref_raw)
            if ref_schema is not None:
                self.ensure_component_model(ref_raw, ref_schema)
            return RustType(ref_name, optional or self.nullable_aliases.get(ref_name, False))

        alternatives = self.non_null_alternatives(schema.get("oneOf") or schema.get("anyOf"))
        if alternatives:
            optional = optional or self.has_null_alternative(schema.get("oneOf") or schema.get("anyOf"))
            if len(alternatives) == 1:
                mapped = self.map_schema(alternatives[0], suggested_name)
                return RustType(mapped.code, mapped.optional or optional)
            primitive_types = [self.map_schema(item, suggested_name).plain() for item in alternatives]
            if len(set(primitive_types)) == 1 and primitive_types[0] in {"String", "i32", "i64", "f32", "f64", "bool"}:
                return RustType(primitive_types[0], optional)
            inline_name = self.unique_inline_name(suggested_name)
            self.model_names.add(inline_name)
            self.models[inline_name] = ModelDef(
                name=inline_name,
                kind="enum",
                properties=[],
                alias=None,
                variants=self.collect_union_variants(alternatives, inline_name),
                description=schema.get("description") or schema.get("title"),
            )
            return RustType(inline_name, optional)

        if schema.get("allOf"):
            inline_name = self.unique_inline_name(suggested_name)
            self.models[inline_name] = self.build_model(inline_name, schema)
            self.model_names.add(inline_name)
            return RustType(inline_name, optional)

        if schema.get("enum") and not schema.get("type"):
            return RustType("String", optional)

        schema_type = schema.get("type")
        if schema_type == "string":
            if schema.get("format") == "binary":
                return RustType("OpenAiBinaryBody", optional)
            return RustType("String", optional)
        if schema_type == "integer":
            if schema.get("format") in {"int64", "unixtime"}:
                return RustType("i64", optional)
            return RustType("i32", optional)
        if schema_type == "number":
            if schema.get("format") == "float":
                return RustType("f32", optional)
            return RustType("f64", optional)
        if schema_type == "boolean":
            return RustType("bool", optional)
        if schema_type == "array":
            item_name = self.item_model_name(suggested_name)
            item_type = self.map_schema(schema.get("items"), item_name)
            return RustType(f"Vec<{item_type.plain()}>", optional)
        if schema_type == "object" or schema.get("properties"):
            if schema.get("properties"):
                inline_name = self.unique_inline_name(suggested_name)
                self.models[inline_name] = self.build_model(inline_name, schema)
                self.model_names.add(inline_name)
                return RustType(inline_name, optional)

            additional = schema.get("additionalProperties")
            if isinstance(additional, dict):
                value_type = self.map_schema(additional, f"{suggested_name}Value")
                return RustType(f"std::collections::BTreeMap<String, {value_type.plain()}>", optional)
            return RustType("OpenAiJsonObject", optional)

        return RustType("OpenAiJsonValue", optional)

    def non_null_alternatives(self, alternatives: list[dict[str, Any]] | None) -> list[dict[str, Any]]:
        if not alternatives:
            return []
        return [item for item in alternatives if item.get("type") != "null"]

    def has_null_alternative(self, alternatives: list[dict[str, Any]] | None) -> bool:
        return any(item.get("type") == "null" for item in alternatives or [])

    def item_model_name(self, suggested_name: str) -> str:
        if suggested_name.endswith("ies"):
            return suggested_name[:-3] + "y"
        if suggested_name.endswith("s") and len(suggested_name) > 1:
            return suggested_name[:-1]
        return f"{suggested_name}Item"

    def unique_inline_name(self, suggested_name: str) -> str:
        base = pascal_case(suggested_name)
        if base not in self.model_names:
            return base
        index = 2
        while f"{base}{index}" in self.model_names:
            index += 1
        return f"{base}{index}"

    def resolve_ref(self, ref: str) -> dict[str, Any]:
        name = ref.split("/")[-1]
        return self.schemas[name]

    def parse_interfaces(self) -> list[InterfaceDef]:
        grouped: dict[str, list[OperationDef]] = {}
        for path, path_item in (self.spec.get("paths") or {}).items():
            for method in HTTP_METHODS:
                operation = path_item.get(method)
                if not isinstance(operation, dict):
                    continue
                tag = (operation.get("tags") or [self.infer_tag_from_path(path)])[0]
                interface_name = f"OpenAi{pascal_case(tag)}Api"
                operation_def = self.parse_operation(method, path, operation)
                grouped.setdefault(interface_name, []).append(operation_def)

        interfaces = []
        for name, ops in sorted(grouped.items()):
            module_base = name.removeprefix("OpenAi").removesuffix("Api")
            interfaces.append(InterfaceDef(name=name, module_name=snake_case(module_base), operations=ops))
        return interfaces

    def infer_tag_from_path(self, path: str) -> str:
        clean_path = path.strip("/")
        if clean_path.startswith("organization/admin_api_keys"):
            return "AdminApiKeys"
        if clean_path.startswith("chatkit/"):
            return "Chatkit"
        if clean_path.startswith("containers"):
            return "Containers"
        if clean_path.startswith("responses/"):
            return "Responses"
        return "Default"

    def parse_operation(self, method: str, path: str, operation: dict[str, Any]) -> OperationDef:
        function_name = self.operation_function_name(method, path, operation)
        path_const = path_const_name(path)
        parameters = self.parse_parameters(operation)
        request_body, request_body_required = self.parse_request_body(function_name, operation)
        response_type = self.parse_response_type(function_name, operation)
        return OperationDef(
            function_name=function_name,
            http_method=method.upper(),
            path=path.strip("/"),
            path_const=path_const,
            summary=operation.get("summary") or operation.get("description"),
            parameters=parameters,
            request_body=request_body,
            request_body_required=request_body_required,
            response_type=response_type,
        )

    def operation_function_name(self, method: str, path: str, operation: dict[str, Any]) -> str:
        overrides = {
            ("post", "/responses/input_tokens"): "get_input_token_counts",
            ("post", "/responses/compact"): "compact_conversation",
        }
        override = overrides.get((method, path))
        if override:
            return override
        return snake_case(operation.get("operationId") or f"{method}_{path}")

    def parse_parameters(self, operation: dict[str, Any]) -> list[ParameterDef]:
        result: list[ParameterDef] = []
        used: set[str] = set()
        for param in operation.get("parameters") or []:
            location = param.get("in")
            if location not in {"path", "query", "header"}:
                continue
            original_name = param["name"]
            name = snake_case(original_name)
            while name in used:
                name = f"{name}_value"
            used.add(name)
            schema = param.get("schema") or {}
            type_ref = self.map_schema(schema, f"{pascal_case(name)}Parameter")
            result.append(
                ParameterDef(
                    original_name=original_name,
                    name=name,
                    location=location,
                    type_ref=type_ref,
                    required=bool(param.get("required")) or location == "path",
                )
            )
        return result

    def parse_request_body(self, function_name: str, operation: dict[str, Any]) -> tuple[RustType | None, bool]:
        request_body = operation.get("requestBody")
        if not request_body:
            return None, False
        content = request_body.get("content") or {}
        selected = self.select_request_content(content)
        if selected is None:
            return None, False
        content_type, schema = selected
        if content_type == "application/sdp":
            return RustType("OpenAiTextBody"), bool(request_body.get("required"))
        body_type = self.map_schema(schema, f"{pascal_case(function_name)}Request")
        return self.promote_operation_type(body_type, f"{pascal_case(function_name)}Request"), bool(
            request_body.get("required")
        )

    def parse_response_type(self, function_name: str, operation: dict[str, Any]) -> RustType:
        responses = operation.get("responses") or {}
        response = None
        for status in ("200", "201", "202", "204"):
            if status in responses:
                response = responses[status]
                break
        if response is None:
            response = next((value for key, value in responses.items() if str(key).startswith("2")), None)
        if response is None:
            return RustType("()")

        content = response.get("content") or {}
        selected = self.select_response_content(content)
        if selected is None:
            return RustType("()")

        content_type, schema = selected
        if content_type in {"application/octet-stream", "application/zip", "video/mp4", "image/webp"}:
            return RustType("OpenAiBinaryBody")
        if content_type in {"text/event-stream", "application/sdp"}:
            return RustType("OpenAiTextBody")
        response_type = self.map_schema(schema, f"{pascal_case(function_name)}Response")
        return self.promote_operation_type(response_type, f"{pascal_case(function_name)}Response")

    def promote_operation_type(self, type_ref: RustType, suggested_name: str) -> RustType:
        if type_ref.plain() != "OpenAiJsonValue":
            return type_ref
        model_name = self.unique_inline_name(suggested_name)
        self.model_names.add(model_name)
        self.models[model_name] = ModelDef(
            name=model_name,
            kind="alias",
            properties=[],
            alias=RustType("OpenAiJsonValue"),
            variants=[],
            description="Untyped JSON payload retained because the OpenAPI schema does not expose a fixed object shape.",
        )
        return RustType(model_name, type_ref.optional)

    def select_request_content(self, content: dict[str, Any]) -> tuple[str, dict[str, Any]] | None:
        for content_type in ("application/json", "multipart/form-data", "application/x-www-form-urlencoded", "application/sdp"):
            if content_type in content:
                return content_type, content[content_type].get("schema") or {}
        if content:
            content_type, media = next(iter(content.items()))
            return content_type, media.get("schema") or {}
        return None

    def select_response_content(self, content: dict[str, Any]) -> tuple[str, dict[str, Any]] | None:
        for content_type in (
            "application/json",
            "application/octet-stream",
            "application/zip",
            "video/mp4",
            "image/webp",
            "text/event-stream",
            "application/sdp",
        ):
            if content_type in content:
                return content_type, content[content_type].get("schema") or {}
        if content:
            content_type, media = next(iter(content.items()))
            return content_type, media.get("schema") or {}
        return None

    def write_bodies(self, path: Path) -> None:
        content = "\n".join(
            [
                HEADER,
                "//! Shared body aliases used by generated OpenAI REST traits.",
                "",
                "/// JSON value used only for schema fields that are intentionally open-ended.",
                "pub type OpenAiJsonValue = serde_json::Value;",
                "/// JSON object used only for schema fields that are intentionally open-ended.",
                "pub type OpenAiJsonObject = std::collections::BTreeMap<String, serde_json::Value>;",
                "/// Binary response body for content download endpoints.",
                "pub type OpenAiBinaryBody = Vec<u8>;",
                "/// Text body used by non-JSON endpoints such as SDP and event streams.",
                "pub type OpenAiTextBody = String;",
                "",
            ]
        )
        path.write_text(content, encoding="utf-8")

    def write_paths(self, path: Path) -> None:
        spec_version = self.spec.get("info", {}).get("version", "")
        lines = [
            HEADER,
            "//! OpenAI REST API source metadata and fixed path constants.",
            "",
            "/// Source metadata for the generated OpenAI OpenAPI contract.",
            "pub struct OpenAiApiSpec;",
            "",
            "impl OpenAiApiSpec {",
            "    /// OpenAI REST API base URL.",
            f"    pub const BASE_URL: &'static str = {rust_string(BASE_URL)};",
            "    /// Source OpenAPI repository.",
            f"    pub const SOURCE_REPOSITORY: &'static str = {rust_string(self.source_repository)};",
            "    /// Source OpenAPI spec URL or local file path used during generation.",
            f"    pub const SOURCE_SPEC_URL: &'static str = {rust_string(self.source_spec_location)};",
            "    /// Source OpenAPI spec version observed during generation.",
            f"    pub const SOURCE_SPEC_VERSION: &'static str = {rust_string(str(spec_version))};",
            "    /// Source git commit observed during generation.",
            f"    pub const SOURCE_COMMIT: &'static str = {rust_string(self.source_commit)};",
            "}",
            "",
            "/// Fixed relative REST paths from the OpenAI OpenAPI spec.",
            "pub struct OpenAiApiPath;",
            "",
            "impl OpenAiApiPath {",
        ]
        for raw_path in sorted((self.spec.get("paths") or {}).keys()):
            clean_path = raw_path.strip("/")
            lines.append(f"    /// `{raw_path}`")
            lines.append(f"    pub const {path_const_name(raw_path)}: &'static str = {rust_string(clean_path)};")
        lines.extend(["}", ""])
        path.write_text("\n".join(lines), encoding="utf-8")

    def write_api_entry(self, path: Path, interfaces: list[InterfaceDef]) -> None:
        lines = [
            HEADER,
            "//! Generated OpenAI REST API traits.",
            "",
            'automod::dir!(pub "src/api");',
            "",
        ]
        for interface in interfaces:
            lines.append(f"pub use {interface.module_name}::{interface.name};")
        lines.append("")
        path.write_text("\n".join(lines), encoding="utf-8")

    def write_models_entry(self, path: Path) -> None:
        lines = [
            HEADER,
            "//! Generated OpenAI REST DTOs.",
            "",
            'automod::dir!(pub "src/models");',
            "",
        ]
        for model in sorted(self.models.values(), key=lambda item: item.name):
            module_name = file_stem_for_model(model.name)
            lines.append(f"pub use {module_name}::{model.name};")
        lines.append("")
        path.write_text("\n".join(lines), encoding="utf-8")

    def write_interface(self, path: Path, interface: InterfaceDef) -> None:
        imports = {"async_trait::async_trait"}
        used_model_names: set[str] = set()
        used_body_names: set[str] = set()
        for operation in interface.operations:
            self.collect_model_names(operation.response_type, used_model_names)
            self.collect_body_names(operation.response_type, used_body_names)
            if operation.request_body is not None:
                self.collect_model_names(operation.request_body, used_model_names)
                self.collect_body_names(operation.request_body, used_body_names)
            for param in operation.parameters:
                self.collect_model_names(param.type_ref, used_model_names)
                self.collect_body_names(param.type_ref, used_body_names)

        lines = [HEADER, f"//! {interface.name.removeprefix('OpenAi').removesuffix('Api')} REST endpoint contract.", ""]
        for item in sorted(imports):
            lines.append(f"use {item};")
        if used_body_names:
            lines.append("")
            lines.append("use crate::bodies::{")
            for name in sorted(used_body_names):
                lines.append(f"    {name},")
            lines.append("};")
        if used_model_names:
            lines.append("")
            lines.append("use crate::models::{")
            for name in sorted(used_model_names):
                lines.append(f"    {name},")
            lines.append("};")
        lines.extend(["", *doc_lines(f"{interface.name.removeprefix('OpenAi').removesuffix('Api')} REST endpoints."), "#[async_trait]"])
        lines.append(f"pub trait {interface.name}: Send + Sync {{")
        lines.append("    /// Error type returned by the application-layer implementation.")
        lines.append("    type Error: std::error::Error + Send + Sync + 'static;")
        used_names: set[str] = set()
        for operation in interface.operations:
            function_name = operation.function_name
            while function_name in used_names:
                function_name = f"{function_name}_value"
            used_names.add(function_name)
            lines.append("")
            lines.extend(self.render_operation(operation, function_name))
        lines.append("}")
        lines.append("")
        path.write_text("\n".join(lines), encoding="utf-8")

    def render_operation(self, operation: OperationDef, function_name: str) -> list[str]:
        result: list[str] = []
        doc = operation.summary or f"REST: {operation.http_method} /{operation.path}"
        result.extend(doc_lines(doc, "    "))
        result.append("    ///")
        result.append(f"    /// REST: `{operation.http_method} /{operation.path}`.")
        result.append(
            f"    /// Path constant: [`OpenAiApiPath::{operation.path_const}`](crate::paths::OpenAiApiPath::{operation.path_const})."
        )

        params: list[str] = ["&self"]
        for param in operation.parameters:
            force_optional = not param.required
            params.append(f"{param.name}: {param.type_ref.rendered(force_optional)}")
        if operation.request_body is not None:
            force_optional = not operation.request_body_required
            params.append(f"body: {operation.request_body.rendered(force_optional)}")

        return_type = operation.response_type.rendered()
        result.extend(self.render_signature(function_name, params, return_type, "    "))
        return result

    def render_signature(
        self,
        function_name: str,
        params: list[str],
        return_type: str,
        indent: str,
    ) -> list[str]:
        signature = f"async fn {function_name}({', '.join(params)}) -> Result<{return_type}, Self::Error>;"
        if len(indent) + len(signature) <= 100:
            return [f"{indent}{signature}"]

        result = [f"{indent}async fn {function_name}("]
        for param in params:
            result.append(f"{indent}    {param},")
        result.append(f"{indent}) -> Result<{return_type}, Self::Error>;")
        return result

    def collect_model_names(self, type_ref: RustType, names: set[str]) -> None:
        for name in re.findall(r"\b[A-Z][A-Za-z0-9]*\b", type_ref.code):
            if name in self.model_names and name not in PRIMITIVE_TYPES:
                names.add(name)

    def collect_body_names(self, type_ref: RustType, names: set[str]) -> None:
        for name in BODY_TYPES:
            if name in type_ref.code:
                names.add(name)

    def write_model(self, path: Path, model: ModelDef) -> None:
        imports = set()
        if model.kind in {"data", "enum"}:
            imports.add("serde::{Deserialize, Serialize}")
        used_model_names: set[str] = set()
        used_body_names: set[str] = set()
        if model.kind == "data":
            if not model.properties:
                used_body_names.add("OpenAiJsonObject")
            for prop in model.properties:
                self.collect_model_names(prop.type_ref, used_model_names)
                self.collect_body_names(prop.type_ref, used_body_names)
        elif model.kind == "alias" and model.alias is not None:
            self.collect_model_names(model.alias, used_model_names)
            self.collect_body_names(model.alias, used_body_names)
        elif model.kind == "enum":
            for variant in model.variants:
                self.collect_model_names(variant.type_ref, used_model_names)
                self.collect_body_names(variant.type_ref, used_body_names)

        used_model_names.discard(model.name)
        lines = [HEADER, f"//! `{model.name}` DTO.", ""]
        for item in sorted(imports):
            lines.append(f"use {item};")
        if used_body_names:
            if imports:
                lines.append("")
            lines.append("use crate::bodies::{")
            for name in sorted(used_body_names):
                lines.append(f"    {name},")
            lines.append("};")
        if used_model_names:
            if imports or used_body_names:
                lines.append("")
            lines.append("use crate::models::{")
            for name in sorted(used_model_names):
                lines.append(f"    {name},")
            lines.append("};")
        lines.append("")
        if model.description:
            lines.extend(doc_lines(model.description))

        if model.kind == "alias":
            alias = model.alias or RustType("OpenAiJsonValue")
            lines.append(f"pub type {model.name} = {alias.rendered()};")
            lines.append("")
            path.write_text("\n".join(lines), encoding="utf-8")
            return

        if model.kind == "enum":
            lines.extend(
                [
                    "#[derive(Debug, Clone, Serialize, Deserialize)]",
                    "#[serde(untagged)]",
                    f"pub enum {model.name} {{",
                ]
            )
            for variant in model.variants:
                lines.append(f"    {variant.name}({variant.type_ref.plain()}),")
            lines.append("}")
            lines.append("")
            path.write_text("\n".join(lines), encoding="utf-8")
            return

        lines.append("#[derive(Debug, Clone, Serialize, Deserialize)]")
        lines.append(f"pub struct {model.name} {{")
        if not model.properties:
            lines.append("    #[serde(flatten)]")
            lines.append("    pub value: OpenAiJsonObject,")
        else:
            for prop in model.properties:
                lines.extend(self.render_property(prop))
        lines.append("}")
        lines.append("")
        path.write_text("\n".join(lines), encoding="utf-8")

    def render_property(self, prop: PropertyDef) -> list[str]:
        result: list[str] = []
        result.extend(doc_lines(prop.description, "    "))
        attrs: list[str] = []
        if prop.original_name != prop.name:
            attrs.append(f'rename = "{prop.original_name}"')
        if not prop.required or prop.type_ref.optional:
            attrs.append("default")
            attrs.append("skip_serializing_if = \"Option::is_none\"")
        if attrs:
            result.append(f"    #[serde({', '.join(attrs)})]")
        result.append(f"    pub {prop.name}: {prop.type_ref.rendered(not prop.required)},")
        return result


def main() -> int:
    parser = argparse.ArgumentParser(description="Generate typed Rust OpenAI REST API contracts.")
    parser.add_argument(
        "--spec-file",
        help="Optional local OpenAPI JSON or YAML file. Takes precedence over --spec-url.",
    )
    parser.add_argument(
        "--spec",
        dest="legacy_spec_file",
        help="Deprecated alias for --spec-file.",
    )
    parser.add_argument(
        "--spec-url",
        default=os.environ.get(SPEC_URL_ENV, DEFAULT_SOURCE_SPEC_URL),
        help=f"Remote OpenAPI JSON or YAML URL. Defaults to ${SPEC_URL_ENV} or the official YAML spec.",
    )
    parser.add_argument(
        "--source-repository",
        default=SOURCE_REPOSITORY,
        help="Git repository used to record SOURCE_COMMIT metadata.",
    )
    parser.add_argument(
        "--source-ref",
        default="refs/heads/master",
        help="Git ref used to record SOURCE_COMMIT metadata.",
    )
    parser.add_argument(
        "--crate-dir",
        default="crates/api/az-openai-openapi",
        help="Rust crate directory to write into.",
    )
    args = parser.parse_args()

    spec_file = args.spec_file or args.legacy_spec_file
    loaded = load_spec(spec_file, args.spec_url)
    source_commit = current_openapi_commit(args.source_repository, args.source_ref)
    generator = OpenAiRustGenerator(
        loaded.spec,
        args.source_repository,
        loaded.source_spec_location,
        source_commit,
    )
    generator.generate(Path(args.crate_dir))
    return 0


if __name__ == "__main__":
    sys.exit(main())
