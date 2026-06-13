use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::Path,
};

use convert_case::{Case, Casing};
use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote};
use serde_json::{Map, Value};

const DEFAULT_SOURCE_REPOSITORY: &str = "https://github.com/openai/openai-openapi";
const DEFAULT_SOURCE_SPEC_URL: &str =
    "https://raw.githubusercontent.com/openai/openai-openapi/master/openapi.yaml";
const BASE_URL: &str = "https://api.openai.com/v1/";
const SPEC_URL_ENV: &str = "AZ_OPENAI_OPENAPI_SPEC_URL";
const HTTP_METHODS: [&str; 5] = ["get", "post", "put", "delete", "patch"];

#[derive(Clone, Debug)]
pub struct OpenApiContractConfig {
    pub spec_url: String,
    pub source_repository: String,
    pub source_commit: String,
}

impl Default for OpenApiContractConfig {
    fn default() -> Self {
        Self {
            spec_url: std::env::var(SPEC_URL_ENV).unwrap_or_else(|_| DEFAULT_SOURCE_SPEC_URL.to_string()),
            source_repository: DEFAULT_SOURCE_REPOSITORY.to_string(),
            source_commit: "runtime".to_string(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct GeneratedOpenApiContract {
    pub files: Vec<GeneratedOpenApiFile>,
}

#[derive(Clone, Debug)]
pub struct GeneratedOpenApiFile {
    pub relative_path: String,
    pub source: String,
}

pub fn generate_openai_contract(
    config: OpenApiContractConfig,
) -> Result<GeneratedOpenApiContract, String> {
    let mut generator = OpenApiContractGenerator::from_config(config)?;
    generator.generate_source()
}

pub fn write_openai_contract(
    config: OpenApiContractConfig,
    output_dir: impl AsRef<Path>,
) -> Result<(), String> {
    let output_dir = output_dir.as_ref();
    let generated = generate_openai_contract(config)?;
    fs::create_dir_all(output_dir)
        .map_err(|error| format!("failed to create {}: {error}", output_dir.display()))?;
    let generated_dir = output_dir.join("generated");
    if generated_dir.exists() {
        fs::remove_dir_all(&generated_dir)
            .map_err(|error| format!("failed to remove {}: {error}", generated_dir.display()))?;
    }
    for file in generated.files {
        let output_file = output_dir.join(&file.relative_path);
        if let Some(parent) = output_file.parent() {
            fs::create_dir_all(parent)
                .map_err(|error| format!("failed to create {}: {error}", parent.display()))?;
        }
        fs::write(&output_file, file.source)
            .map_err(|error| format!("failed to write {}: {error}", output_file.display()))?;
    }
    Ok(())
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
struct RustType {
    code: String,
    optional: bool,
}

impl RustType {
    fn new(code: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            optional: false,
        }
    }

    fn optional(code: impl Into<String>, optional: bool) -> Self {
        Self {
            code: code.into(),
            optional,
        }
    }

    fn rendered(&self, force_optional: bool) -> TokenStream2 {
        let ty = parse_type(&self.code);
        if (self.optional || force_optional) && !self.code.starts_with("Option<") {
            quote! { ::std::option::Option<#ty> }
        } else {
            quote! { #ty }
        }
    }

    fn plain_code(&self) -> String {
        self.code
            .strip_prefix("Option<")
            .and_then(|value| value.strip_suffix('>'))
            .unwrap_or(&self.code)
            .to_string()
    }
}

#[derive(Clone, Debug)]
struct PropertyDef {
    original_name: String,
    name: String,
    type_ref: RustType,
    required: bool,
    description: Option<String>,
}

#[derive(Clone, Debug)]
struct VariantDef {
    name: String,
    type_ref: RustType,
}

#[derive(Clone, Debug)]
enum ModelKind {
    Data(Vec<PropertyDef>),
    Alias(RustType),
    Enum(Vec<VariantDef>),
}

#[derive(Clone, Debug)]
struct ModelDef {
    name: String,
    kind: ModelKind,
    description: Option<String>,
}

#[derive(Clone, Debug)]
struct ParameterDef {
    name: String,
    type_ref: RustType,
    required: bool,
}

#[derive(Clone, Debug)]
struct OperationDef {
    function_name: String,
    http_method: String,
    path: String,
    path_const: String,
    summary: Option<String>,
    parameters: Vec<ParameterDef>,
    request_body: Option<RustType>,
    request_body_required: bool,
    response_type: RustType,
}

#[derive(Clone, Debug)]
struct InterfaceDef {
    name: String,
    operations: Vec<OperationDef>,
}

struct OpenApiContractGenerator {
    spec: Value,
    source_repository: String,
    source_spec_url: String,
    source_commit: String,
    schemas: Map<String, Value>,
    models: BTreeMap<String, ModelDef>,
    model_names: BTreeSet<String>,
    nullable_aliases: BTreeMap<String, bool>,
    processing: BTreeSet<String>,
}

impl OpenApiContractGenerator {
    fn from_config(config: OpenApiContractConfig) -> Result<Self, String> {
        let source_spec_url = config.spec_url;
        let source_repository = config.source_repository;
        let source_commit = config.source_commit;
        let spec = fetch_spec(&source_spec_url)?;
        let schemas = spec
            .get("components")
            .and_then(|components| components.get("schemas"))
            .and_then(Value::as_object)
            .cloned()
            .unwrap_or_default();

        Ok(Self {
            spec,
            source_repository,
            source_spec_url,
            source_commit,
            schemas,
            models: BTreeMap::new(),
            model_names: BTreeSet::new(),
            nullable_aliases: BTreeMap::new(),
            processing: BTreeSet::new(),
        })
    }

    fn generate_source(&mut self) -> Result<GeneratedOpenApiContract, String> {
        let schema_entries = self
            .schemas
            .clone()
            .into_iter()
            .collect::<Vec<(String, Value)>>();
        for (name, schema) in schema_entries {
            self.ensure_component_model(&name, &schema)?;
        }

        let interfaces = self.parse_interfaces()?;
        let bodies = self.render_bodies_content();
        let paths = self.render_paths_content();
        let models = self.render_models_content();
        let api = self.render_api_content(&interfaces);

        Ok(GeneratedOpenApiContract {
            files: vec![
                GeneratedOpenApiFile {
                    relative_path: "generated.rs".to_string(),
                    source: generated_entry_source(),
                },
                GeneratedOpenApiFile {
                    relative_path: "generated/bodies.rs".to_string(),
                    source: normalize_rust_source(bodies),
                },
                GeneratedOpenApiFile {
                    relative_path: "generated/paths.rs".to_string(),
                    source: normalize_rust_source(paths),
                },
                GeneratedOpenApiFile {
                    relative_path: "generated/models.rs".to_string(),
                    source: normalize_rust_source(models),
                },
                GeneratedOpenApiFile {
                    relative_path: "generated/api.rs".to_string(),
                    source: normalize_rust_source(api),
                },
            ],
        })
    }

    fn ensure_component_model(&mut self, name: &str, schema: &Value) -> Result<RustType, String> {
        let model_name = pascal_case(name);
        if self.models.contains_key(&model_name) || self.processing.contains(&model_name) {
            return Ok(RustType::new(model_name));
        }

        self.processing.insert(model_name.clone());
        self.model_names.insert(model_name.clone());
        let model = self.build_model(&model_name, schema)?;
        self.models.insert(model_name.clone(), model);
        self.processing.remove(&model_name);
        Ok(RustType::new(model_name))
    }

    fn build_model(&mut self, name: &str, schema: &Value) -> Result<ModelDef, String> {
        let union_schema = self.union_schema(schema);
        let object_schema = self.object_schema(schema);
        let description = description(schema);

        if let (Some(alternatives), None) = (union_schema, object_schema) {
            let variants = self.collect_union_variants(&alternatives, name)?;
            if !variants.is_empty() {
                return Ok(ModelDef {
                    name: name.to_string(),
                    kind: ModelKind::Enum(variants),
                    description,
                });
            }
        }

        if object_schema.is_some() {
            return Ok(ModelDef {
                name: name.to_string(),
                kind: ModelKind::Data(self.collect_properties(schema, name)?),
                description,
            });
        }

        let alias = self.map_schema(schema, name)?;
        self.nullable_aliases.insert(name.to_string(), alias.optional);
        Ok(ModelDef {
            name: name.to_string(),
            kind: ModelKind::Alias(RustType::new(alias.code)),
            description,
        })
    }

    fn object_schema<'a>(&self, schema: &'a Value) -> Option<&'a Value> {
        if schema.get("properties").is_some() || schema.get("allOf").is_some() {
            return Some(schema);
        }
        if schema.get("type").and_then(Value::as_str) == Some("object")
            && schema.get("additionalProperties").is_none()
        {
            return Some(schema);
        }
        None
    }

    fn union_schema(&self, schema: &Value) -> Option<Vec<Value>> {
        let alternatives = non_null_alternatives(schema.get("oneOf").or_else(|| schema.get("anyOf")));
        if alternatives.len() > 1 {
            Some(alternatives)
        } else {
            None
        }
    }

    fn collect_union_variants(
        &mut self,
        alternatives: &[Value],
        model_name: &str,
    ) -> Result<Vec<VariantDef>, String> {
        let mut variants = Vec::new();
        let mut used = BTreeSet::new();
        for (index, item) in alternatives.iter().enumerate() {
            let mut variant_name = variant_name(item, index + 1);
            while used.contains(&variant_name) {
                variant_name = format!("{}{}", variant_name, index + 1);
            }
            used.insert(variant_name.clone());
            let mapped = self.map_schema(item, &format!("{model_name}{variant_name}"))?;
            variants.push(VariantDef {
                name: variant_name,
                type_ref: RustType::new(mapped.code),
            });
        }
        Ok(variants)
    }

    fn collect_properties(
        &mut self,
        schema: &Value,
        model_name: &str,
    ) -> Result<Vec<PropertyDef>, String> {
        let mut merged = BTreeMap::new();
        for (required_props, props) in self.iter_object_properties(schema)? {
            for (original_name, prop_schema) in props {
                if merged.contains_key(&original_name) {
                    continue;
                }
                let name = snake_case(&original_name);
                let required = required_props.contains(&original_name);
                let type_ref =
                    self.map_schema(&prop_schema, &format!("{model_name}{}", pascal_case(&original_name)))?;
                merged.insert(
                    original_name.clone(),
                    PropertyDef {
                        original_name,
                        name,
                        type_ref,
                        required,
                        description: description(&prop_schema),
                    },
                );
            }
        }
        Ok(merged.into_values().collect())
    }

    fn iter_object_properties(
        &mut self,
        schema: &Value,
    ) -> Result<Vec<(BTreeSet<String>, Map<String, Value>)>, String> {
        if let Some(reference) = schema.get("$ref").and_then(Value::as_str) {
            let ref_schema = self.resolve_ref(reference)?;
            return self.iter_object_properties(&ref_schema);
        }

        if let Some(all_of) = schema.get("allOf").and_then(Value::as_array) {
            let mut result = Vec::new();
            let parent_required = required_set(schema);
            for item in all_of {
                for (required, props) in self.iter_object_properties(item)? {
                    let mut merged_required = parent_required.clone();
                    merged_required.extend(required);
                    result.push((merged_required, props));
                }
            }
            return Ok(result);
        }

        let alternatives = non_null_alternatives(schema.get("oneOf").or_else(|| schema.get("anyOf")));
        if alternatives.len() == 1 {
            return self.iter_object_properties(&alternatives[0]);
        }

        Ok(vec![(
            required_set(schema),
            schema
                .get("properties")
                .and_then(Value::as_object)
                .cloned()
                .unwrap_or_default(),
        )])
    }

    fn map_schema(&mut self, schema: &Value, suggested_name: &str) -> Result<RustType, String> {
        if schema.is_null() {
            return Ok(RustType::new("OpenAiJsonValue"));
        }

        let mut schema = schema.clone();
        let mut optional = take_bool(&mut schema, "nullable");
        if let Some(types) = schema.get("type").and_then(Value::as_array) {
            optional = optional || types.iter().any(|item| item.as_str() == Some("null"));
            let non_null = types
                .iter()
                .filter_map(Value::as_str)
                .filter(|value| *value != "null")
                .collect::<Vec<_>>();
            if non_null.len() == 1 {
                schema["type"] = Value::String(non_null[0].to_string());
            } else {
                schema.as_object_mut().expect("schema object").remove("type");
            }
        }

        if let Some(reference) = schema.get("$ref").and_then(Value::as_str) {
            let ref_raw = reference
                .rsplit('/')
                .next()
                .ok_or_else(|| format!("invalid OpenAPI ref `{reference}`"))?;
            let ref_name = pascal_case(ref_raw);
            if let Some(ref_schema) = self.schemas.get(ref_raw).cloned() {
                self.ensure_component_model(ref_raw, &ref_schema)?;
            }
            return Ok(RustType::optional(
                ref_name.clone(),
                optional || self.nullable_aliases.get(&ref_name).copied().unwrap_or(false),
            ));
        }

        let alternatives = non_null_alternatives(schema.get("oneOf").or_else(|| schema.get("anyOf")));
        if !alternatives.is_empty() {
            optional = optional || has_null_alternative(schema.get("oneOf").or_else(|| schema.get("anyOf")));
            if alternatives.len() == 1 {
                let mapped = self.map_schema(&alternatives[0], suggested_name)?;
                return Ok(RustType::optional(mapped.code, mapped.optional || optional));
            }
            let primitive_types = alternatives
                .iter()
                .map(|item| self.map_schema(item, suggested_name).map(|mapped| mapped.plain_code()))
                .collect::<Result<Vec<_>, _>>()?;
            if primitive_types.iter().collect::<BTreeSet<_>>().len() == 1
                && matches!(
                    primitive_types[0].as_str(),
                    "String" | "i32" | "i64" | "f32" | "f64" | "bool"
                )
            {
                return Ok(RustType::optional(primitive_types[0].clone(), optional));
            }
            let inline_name = self.unique_inline_name(suggested_name);
            self.model_names.insert(inline_name.clone());
            let variants = self.collect_union_variants(&alternatives, &inline_name)?;
            self.models.insert(
                inline_name.clone(),
                ModelDef {
                    name: inline_name.clone(),
                    kind: ModelKind::Enum(variants),
                    description: description(&schema),
                },
            );
            return Ok(RustType::optional(inline_name, optional));
        }

        if schema.get("allOf").is_some() {
            let inline_name = self.unique_inline_name(suggested_name);
            let model = self.build_model(&inline_name, &schema)?;
            self.model_names.insert(inline_name.clone());
            self.models.insert(inline_name.clone(), model);
            return Ok(RustType::optional(inline_name, optional));
        }

        if schema.get("enum").is_some() && schema.get("type").is_none() {
            return Ok(RustType::optional("String", optional));
        }

        match schema.get("type").and_then(Value::as_str) {
            Some("string") => {
                if schema.get("format").and_then(Value::as_str) == Some("binary") {
                    Ok(RustType::optional("OpenAiBinaryBody", optional))
                } else {
                    Ok(RustType::optional("String", optional))
                }
            }
            Some("integer") => {
                if matches!(
                    schema.get("format").and_then(Value::as_str),
                    Some("int64" | "unixtime")
                ) {
                    Ok(RustType::optional("i64", optional))
                } else {
                    Ok(RustType::optional("i32", optional))
                }
            }
            Some("number") => {
                if schema.get("format").and_then(Value::as_str) == Some("float") {
                    Ok(RustType::optional("f32", optional))
                } else {
                    Ok(RustType::optional("f64", optional))
                }
            }
            Some("boolean") => Ok(RustType::optional("bool", optional)),
            Some("array") => {
                let item_name = item_model_name(suggested_name);
                let item_type = self.map_schema(schema.get("items").unwrap_or(&Value::Null), &item_name)?;
                Ok(RustType::optional(format!("Vec<{}>", item_type.plain_code()), optional))
            }
            Some("object") | None if schema.get("properties").is_some() => {
                if schema.get("properties").is_some() {
                    let inline_name = self.unique_inline_name(suggested_name);
                    let model = self.build_model(&inline_name, &schema)?;
                    self.model_names.insert(inline_name.clone());
                    self.models.insert(inline_name.clone(), model);
                    return Ok(RustType::optional(inline_name, optional));
                }
                let additional = schema.get("additionalProperties");
                if let Some(additional) = additional.filter(|value| value.is_object()) {
                    let value_type = self.map_schema(additional, &format!("{suggested_name}Value"))?;
                    Ok(RustType::optional(
                        format!(
                            "::std::collections::BTreeMap<String, {}>",
                            value_type.plain_code()
                        ),
                        optional,
                    ))
                } else {
                    Ok(RustType::optional("OpenAiJsonObject", optional))
                }
            }
            _ => Ok(RustType::optional("OpenAiJsonValue", optional)),
        }
    }

    fn unique_inline_name(&self, suggested_name: &str) -> String {
        let base = pascal_case(suggested_name);
        if !self.model_names.contains(&base) {
            return base;
        }
        let mut index = 2;
        loop {
            let candidate = format!("{base}{index}");
            if !self.model_names.contains(&candidate) {
                return candidate;
            }
            index += 1;
        }
    }

    fn resolve_ref(&self, reference: &str) -> Result<Value, String> {
        let name = reference
            .rsplit('/')
            .next()
            .ok_or_else(|| format!("invalid OpenAPI ref `{reference}`"))?;
        self.schemas
            .get(name)
            .cloned()
            .ok_or_else(|| format!("missing OpenAPI schema ref `{name}`"))
    }

    fn parse_interfaces(&mut self) -> Result<Vec<InterfaceDef>, String> {
        let paths = self
            .spec
            .get("paths")
            .and_then(Value::as_object)
            .cloned()
            .unwrap_or_default();
        let mut grouped: BTreeMap<String, Vec<OperationDef>> = BTreeMap::new();

        for (path, path_item) in paths {
            let Some(path_item) = path_item.as_object() else {
                continue;
            };
            for method in HTTP_METHODS {
                let Some(operation) = path_item.get(method).filter(|value| value.is_object()) else {
                    continue;
                };
                let tag = operation
                    .get("tags")
                    .and_then(Value::as_array)
                    .and_then(|tags| tags.first())
                    .and_then(Value::as_str)
                    .map(str::to_string)
                    .unwrap_or_else(|| infer_tag_from_path(&path));
                let interface_name = format!("OpenAi{}Api", pascal_case(&tag));
                let operation_def = self.parse_operation(method, &path, operation)?;
                grouped.entry(interface_name).or_default().push(operation_def);
            }
        }

        Ok(grouped
            .into_iter()
            .map(|(name, operations)| InterfaceDef { name, operations })
            .collect())
    }

    fn parse_operation(
        &mut self,
        method: &str,
        path: &str,
        operation: &Value,
    ) -> Result<OperationDef, String> {
        let function_name = operation_function_name(method, path, operation);
        let parameters = self.parse_parameters(operation)?;
        let (request_body, request_body_required) = self.parse_request_body(&function_name, operation)?;
        let response_type = self.parse_response_type(&function_name, operation)?;
        Ok(OperationDef {
            function_name,
            http_method: method.to_uppercase(),
            path: path.trim_start_matches('/').to_string(),
            path_const: path_const_name(path),
            summary: operation
                .get("summary")
                .or_else(|| operation.get("description"))
                .and_then(Value::as_str)
                .map(str::to_string),
            parameters,
            request_body,
            request_body_required,
            response_type,
        })
    }

    fn parse_parameters(&mut self, operation: &Value) -> Result<Vec<ParameterDef>, String> {
        let mut result = Vec::new();
        let mut used = BTreeSet::new();
        for param in operation
            .get("parameters")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
        {
            let location = param.get("in").and_then(Value::as_str).unwrap_or_default();
            if !matches!(location, "path" | "query" | "header") {
                continue;
            }
            let Some(original_name) = param.get("name").and_then(Value::as_str) else {
                continue;
            };
            let mut name = snake_case(original_name);
            while used.contains(&name) {
                name = format!("{name}_value");
            }
            used.insert(name.clone());
            let schema = param.get("schema").unwrap_or(&Value::Null);
            let type_ref = self.map_schema(schema, &format!("{}Parameter", pascal_case(&name)))?;
            result.push(ParameterDef {
                name,
                type_ref,
                required: param.get("required").and_then(Value::as_bool).unwrap_or(false)
                    || location == "path",
            });
        }
        Ok(result)
    }

    fn parse_request_body(
        &mut self,
        function_name: &str,
        operation: &Value,
    ) -> Result<(Option<RustType>, bool), String> {
        let Some(request_body) = operation.get("requestBody") else {
            return Ok((None, false));
        };
        let Some((content_type, schema)) =
            select_content(request_body.get("content"), &["application/json", "multipart/form-data", "application/x-www-form-urlencoded", "application/sdp"])
        else {
            return Ok((None, false));
        };
        if content_type == "application/sdp" {
            return Ok((
                Some(RustType::new("OpenAiTextBody")),
                request_body
                    .get("required")
                    .and_then(Value::as_bool)
                    .unwrap_or(false),
            ));
        }
        let body_type = self.map_schema(&schema, &format!("{}Request", pascal_case(function_name)))?;
        Ok((
            Some(self.promote_operation_type(body_type, &format!("{}Request", pascal_case(function_name)))?),
            request_body
                .get("required")
                .and_then(Value::as_bool)
                .unwrap_or(false),
        ))
    }

    fn parse_response_type(
        &mut self,
        function_name: &str,
        operation: &Value,
    ) -> Result<RustType, String> {
        let Some(responses) = operation.get("responses").and_then(Value::as_object) else {
            return Ok(RustType::new("()"));
        };
        let response = ["200", "201", "202", "204"]
            .iter()
            .find_map(|status| responses.get(*status))
            .or_else(|| responses.iter().find(|(key, _)| key.starts_with('2')).map(|(_, value)| value));
        let Some(response) = response else {
            return Ok(RustType::new("()"));
        };
        let Some((content_type, schema)) = select_content(
            response.get("content"),
            &[
                "application/json",
                "application/octet-stream",
                "application/zip",
                "video/mp4",
                "image/webp",
                "text/event-stream",
                "application/sdp",
            ],
        ) else {
            return Ok(RustType::new("()"));
        };
        if matches!(
            content_type.as_str(),
            "application/octet-stream" | "application/zip" | "video/mp4" | "image/webp"
        ) {
            return Ok(RustType::new("OpenAiBinaryBody"));
        }
        if matches!(content_type.as_str(), "text/event-stream" | "application/sdp") {
            return Ok(RustType::new("OpenAiTextBody"));
        }
        let response_type = self.map_schema(&schema, &format!("{}Response", pascal_case(function_name)))?;
        self.promote_operation_type(response_type, &format!("{}Response", pascal_case(function_name)))
    }

    fn promote_operation_type(
        &mut self,
        type_ref: RustType,
        suggested_name: &str,
    ) -> Result<RustType, String> {
        if type_ref.plain_code() != "OpenAiJsonValue" {
            return Ok(type_ref);
        }
        let model_name = self.unique_inline_name(suggested_name);
        self.model_names.insert(model_name.clone());
        self.models.insert(
            model_name.clone(),
            ModelDef {
                name: model_name.clone(),
                kind: ModelKind::Alias(RustType::new("OpenAiJsonValue")),
                description: Some(
                    "Untyped JSON payload retained because the OpenAPI schema does not expose a fixed object shape."
                        .to_string(),
                ),
            },
        );
        Ok(RustType::optional(model_name, type_ref.optional))
    }

    fn render_bodies_content(&self) -> TokenStream2 {
        quote! {
            pub type OpenAiJsonValue = ::serde_json::Value;
            pub type OpenAiJsonObject = ::std::collections::BTreeMap<String, ::serde_json::Value>;
            pub type OpenAiBinaryBody = Vec<u8>;
            pub type OpenAiTextBody = String;
        }
    }

    fn render_paths_content(&self) -> TokenStream2 {
        let spec_version = self
            .spec
            .get("info")
            .and_then(|info| info.get("version"))
            .and_then(Value::as_str)
            .unwrap_or_default();
        let source_repository = &self.source_repository;
        let source_spec_url = &self.source_spec_url;
        let source_commit = &self.source_commit;
        let path_consts = self
            .spec
            .get("paths")
            .and_then(Value::as_object)
            .into_iter()
            .flat_map(|paths| paths.keys())
            .map(|raw_path| {
                let name = format_ident!("{}", path_const_name(raw_path));
                let value = raw_path.trim_start_matches('/');
                quote! { pub const #name: &'static str = #value; }
            })
            .collect::<Vec<_>>();

        quote! {
            pub struct OpenAiApiSpec;

            impl OpenAiApiSpec {
                pub const BASE_URL: &'static str = #BASE_URL;
                pub const SOURCE_REPOSITORY: &'static str = #source_repository;
                pub const SOURCE_SPEC_URL: &'static str = #source_spec_url;
                pub const SOURCE_SPEC_VERSION: &'static str = #spec_version;
                pub const SOURCE_COMMIT: &'static str = #source_commit;
            }

            pub struct OpenAiApiPath;

            impl OpenAiApiPath {
                #(#path_consts)*
            }
        }
    }

    fn render_models_content(&self) -> TokenStream2 {
        let items = self
            .models
            .values()
            .map(|model| self.render_model(model))
            .collect::<Vec<_>>();
        quote! {
            use super::bodies::*;
            #(#items)*
        }
    }

    fn render_model(&self, model: &ModelDef) -> TokenStream2 {
        let ident = format_ident!("{}", model.name);
        let docs = doc_attrs(model.description.as_deref());
        match &model.kind {
            ModelKind::Alias(alias) => {
                let ty = alias.rendered(false);
                quote! {
                    #(#docs)*
                    pub type #ident = #ty;
                }
            }
            ModelKind::Enum(variants) => {
                let variants = variants
                    .iter()
                    .map(|variant| {
                        let variant_ident = format_ident!("{}", sanitize_variant_ident(&variant.name));
                        let ty = variant.type_ref.rendered(false);
                        quote! { #variant_ident(#ty) }
                    })
                    .collect::<Vec<_>>();
                quote! {
                    #(#docs)*
                    #[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
                    #[serde(untagged)]
                    pub enum #ident {
                        #(#variants,)*
                    }
                }
            }
            ModelKind::Data(properties) => {
                let fields = if properties.is_empty() {
                    vec![quote! {
                        #[serde(flatten)]
                        pub value: OpenAiJsonObject
                    }]
                } else {
                    properties
                        .iter()
                        .map(|prop| render_property(prop))
                        .collect::<Vec<_>>()
                };
                quote! {
                    #(#docs)*
                    #[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
                    pub struct #ident {
                        #(#fields,)*
                    }
                }
            }
        }
    }

    fn render_api_content(&self, interfaces: &[InterfaceDef]) -> TokenStream2 {
        let traits = interfaces
            .iter()
            .map(|interface| self.render_interface(interface))
            .collect::<Vec<_>>();
        quote! {
            use super::bodies::*;
            use super::models::*;

            #(#traits)*
        }
    }

    fn render_interface(&self, interface: &InterfaceDef) -> TokenStream2 {
        let ident = format_ident!("{}", interface.name);
        let docs = doc_attrs(Some(&format!(
            "{} REST endpoints.",
            interface
                .name
                .strip_prefix("OpenAi")
                .and_then(|value| value.strip_suffix("Api"))
                .unwrap_or(&interface.name)
        )));
        let methods = interface
            .operations
            .iter()
            .map(|operation| render_operation(operation))
            .collect::<Vec<_>>();
        quote! {
            #(#docs)*
            #[::async_trait::async_trait]
            pub trait #ident: Send + Sync {
                type Error: ::std::error::Error + Send + Sync + 'static;

                #(#methods)*
            }
        }
    }
}

fn fetch_spec(spec_url: &str) -> Result<Value, String> {
    let response = reqwest::blocking::get(spec_url)
        .map_err(|error| format!("failed to fetch OpenAPI spec from {spec_url}: {error}"))?;
    let text = response
        .error_for_status()
        .map_err(|error| format!("failed to fetch OpenAPI spec from {spec_url}: {error}"))?
        .text()
        .map_err(|error| format!("failed to read OpenAPI spec from {spec_url}: {error}"))?;
    if spec_url.ends_with(".json") {
        serde_json::from_str(&text).map_err(|error| format!("invalid OpenAPI JSON from {spec_url}: {error}"))
    } else {
        let text = sanitize_yaml_numeric_metadata(&text);
        let yaml = serde_yaml::from_str(&text)
            .map_err(|error| format!("invalid OpenAPI YAML from {spec_url}: {error}"))?;
        yaml_to_json(yaml)
    }
}

fn sanitize_yaml_numeric_metadata(text: &str) -> String {
    text.lines()
        .map(|line| {
            let trimmed = line.trim_start();
            let indent = &line[..line.len() - trimmed.len()];
            for key in ["minimum", "maximum", "exclusiveMinimum", "exclusiveMaximum"] {
                let Some(rest) = trimmed.strip_prefix(key).and_then(|value| value.strip_prefix(':')) else {
                    continue;
                };
                let value = rest.trim_start();
                let literal = value.split_whitespace().next().unwrap_or_default();
                if is_yaml_integer_out_of_i64_range(literal) {
                    return format!("{indent}{key}: null");
                }
            }
            line.to_string()
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn is_yaml_integer_out_of_i64_range(value: &str) -> bool {
    let Some(first) = value.chars().next() else {
        return false;
    };
    if !(first == '-' || first == '+' || first.is_ascii_digit()) {
        return false;
    }
    if value.contains(['.', 'e', 'E']) {
        return false;
    }
    value.parse::<i64>().is_err()
}

fn yaml_to_json(value: serde_yaml::Value) -> Result<Value, String> {
    match value {
        serde_yaml::Value::Null => Ok(Value::Null),
        serde_yaml::Value::Bool(value) => Ok(Value::Bool(value)),
        serde_yaml::Value::Number(value) => yaml_number_to_json(value),
        serde_yaml::Value::String(value) => Ok(Value::String(value)),
        serde_yaml::Value::Sequence(values) => values
            .into_iter()
            .map(yaml_to_json)
            .collect::<Result<Vec<_>, _>>()
            .map(Value::Array),
        serde_yaml::Value::Mapping(values) => {
            let mut object = Map::new();
            for (key, value) in values {
                let key = match yaml_to_json(key)? {
                    Value::String(value) => value,
                    other => other.to_string(),
                };
                object.insert(key, yaml_to_json(value)?);
            }
            Ok(Value::Object(object))
        }
        serde_yaml::Value::Tagged(value) => yaml_to_json(value.value),
    }
}

fn yaml_number_to_json(value: serde_yaml::Number) -> Result<Value, String> {
    if let Some(value) = value.as_i64() {
        return Ok(Value::Number(value.into()));
    }
    if let Some(value) = value.as_u64() {
        return Ok(Value::Number(value.into()));
    }
    if let Some(value) = value.as_f64() {
        return serde_json::Number::from_f64(value)
            .map(Value::Number)
            .ok_or_else(|| "OpenAPI YAML contains a non-finite number".to_string());
    }
    Ok(Value::Null)
}

fn parse_type(code: &str) -> TokenStream2 {
    code.parse::<TokenStream2>()
        .unwrap_or_else(|_| quote! { OpenAiJsonValue })
}

fn generated_entry_source() -> String {
    "//! Generated OpenAI REST contract modules.\n\n// Generated from the remote OpenAPI contract. Do not edit by hand.\n\nautomod::dir!(pub \"src/generated\");\n".to_string()
}

fn normalize_rust_source(tokens: TokenStream2) -> String {
    let file = syn::parse2(tokens).expect("generated OpenAPI Rust source should parse");
    prettyplease::unparse(&file)
}

fn render_property(prop: &PropertyDef) -> TokenStream2 {
    let ident = format_ident!("{}", prop.name);
    let ty = prop.type_ref.rendered(!prop.required);
    let docs = doc_attrs(prop.description.as_deref());
    let rename = (prop.original_name != prop.name).then(|| {
        let original = &prop.original_name;
        quote! { rename = #original, }
    });
    let optional_attrs = (!prop.required || prop.type_ref.optional).then(|| {
        quote! { default, skip_serializing_if = "Option::is_none", }
    });
    let serde_attr = (rename.is_some() || optional_attrs.is_some()).then(|| {
        quote! { #[serde(#rename #optional_attrs)] }
    });
    quote! {
        #(#docs)*
        #serde_attr
        pub #ident: #ty
    }
}

fn render_operation(operation: &OperationDef) -> TokenStream2 {
    let name = format_ident!("{}", operation.function_name);
    let docs = doc_attrs(operation.summary.as_deref());
    let rest_doc = format!("REST: `{} /{}`.", operation.http_method, operation.path);
    let path_doc = format!("Path constant: `OpenAiApiPath::{}`.", operation.path_const);
    let params = operation
        .parameters
        .iter()
        .map(|param| {
            let ident = format_ident!("{}", param.name);
            let ty = param.type_ref.rendered(!param.required);
            quote! { #ident: #ty }
        })
        .chain(operation.request_body.iter().map(|body| {
            let ty = body.rendered(!operation.request_body_required);
            quote! { body: #ty }
        }))
        .collect::<Vec<_>>();
    let return_ty = operation.response_type.rendered(false);
    quote! {
        #(#docs)*
        #[doc = ""]
        #[doc = #rest_doc]
        #[doc = #path_doc]
        async fn #name(&self, #(#params),*) -> Result<#return_ty, Self::Error>;
    }
}

fn doc_attrs(text: Option<&str>) -> Vec<TokenStream2> {
    let Some(text) = text else {
        return Vec::new();
    };
    let cleaned = text.split_whitespace().collect::<Vec<_>>().join(" ");
    if cleaned.is_empty() {
        return Vec::new();
    }
    vec![quote! { #[doc = #cleaned] }]
}

fn description(schema: &Value) -> Option<String> {
    schema
        .get("description")
        .or_else(|| schema.get("title"))
        .and_then(Value::as_str)
        .map(str::to_string)
}

fn required_set(schema: &Value) -> BTreeSet<String> {
    schema
        .get("required")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
        .map(str::to_string)
        .collect()
}

fn take_bool(schema: &mut Value, key: &str) -> bool {
    schema
        .as_object_mut()
        .and_then(|object| object.remove(key))
        .and_then(|value| value.as_bool())
        .unwrap_or(false)
}

fn non_null_alternatives(alternatives: Option<&Value>) -> Vec<Value> {
    alternatives
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter(|item| item.get("type").and_then(Value::as_str) != Some("null"))
        .cloned()
        .collect()
}

fn has_null_alternative(alternatives: Option<&Value>) -> bool {
    alternatives
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .any(|item| item.get("type").and_then(Value::as_str) == Some("null"))
}

fn item_model_name(suggested_name: &str) -> String {
    if let Some(prefix) = suggested_name.strip_suffix("ies") {
        format!("{prefix}y")
    } else if suggested_name.ends_with('s') && suggested_name.len() > 1 {
        suggested_name[..suggested_name.len() - 1].to_string()
    } else {
        format!("{suggested_name}Item")
    }
}

fn select_content(content: Option<&Value>, priorities: &[&str]) -> Option<(String, Value)> {
    let content = content.and_then(Value::as_object)?;
    for content_type in priorities {
        if let Some(media) = content.get(*content_type) {
            return Some((
                (*content_type).to_string(),
                media.get("schema").cloned().unwrap_or(Value::Null),
            ));
        }
    }
    content.iter().next().map(|(content_type, media)| {
        (
            content_type.clone(),
            media.get("schema").cloned().unwrap_or(Value::Null),
        )
    })
}

fn infer_tag_from_path(path: &str) -> String {
    let clean_path = path.trim_matches('/');
    if clean_path.starts_with("organization/admin_api_keys") {
        "AdminApiKeys".to_string()
    } else if clean_path.starts_with("chatkit/") {
        "Chatkit".to_string()
    } else if clean_path.starts_with("containers") {
        "Containers".to_string()
    } else if clean_path.starts_with("responses/") {
        "Responses".to_string()
    } else {
        "Default".to_string()
    }
}

fn operation_function_name(method: &str, path: &str, operation: &Value) -> String {
    match (method, path) {
        ("post", "/responses/input_tokens") => "get_input_token_counts".to_string(),
        ("post", "/responses/compact") => "compact_conversation".to_string(),
        _ => operation
            .get("operationId")
            .and_then(Value::as_str)
            .map(snake_case)
            .unwrap_or_else(|| snake_case(&format!("{method}_{path}"))),
    }
}

fn path_const_name(path: &str) -> String {
    path.trim_matches('/')
        .split('/')
        .filter(|segment| !segment.is_empty())
        .map(|segment| segment.trim_matches(['{', '}']).to_case(Case::UpperSnake))
        .collect::<Vec<_>>()
        .join("_BY_")
}

fn variant_name(schema: &Value, index: usize) -> String {
    if let Some(reference) = schema.get("$ref").and_then(Value::as_str) {
        return pascal_case(reference.rsplit('/').next().unwrap_or("Variant"));
    }
    if let Some(title) = schema.get("title").and_then(Value::as_str).filter(|value| !value.is_empty()) {
        return pascal_case(title);
    }
    if let Some(const_value) = schema.get("const").and_then(Value::as_str).filter(|value| !value.is_empty()) {
        return pascal_case(const_value);
    }
    if let Some(enum_values) = schema.get("enum").and_then(Value::as_array) {
        if enum_values.len() == 1 {
            if let Some(value) = enum_values[0].as_str() {
                return pascal_case(value);
            }
        }
    }
    match schema.get("type").and_then(Value::as_str) {
        Some("string") => "String".to_string(),
        Some("integer") => "Integer".to_string(),
        Some("number") => "Number".to_string(),
        Some("boolean") => "Boolean".to_string(),
        Some("array") => "Array".to_string(),
        Some("object") => "Object".to_string(),
        _ if schema.get("properties").is_some() => "Object".to_string(),
        _ => format!("Variant{index}"),
    }
}

fn pascal_case(value: &str) -> String {
    let converted = value.to_case(Case::Pascal);
    if converted.is_empty() {
        "Value".to_string()
    } else {
        sanitize_type_ident(&converted)
    }
}

fn snake_case(value: &str) -> String {
    let converted = value.to_case(Case::Snake);
    if converted.is_empty() {
        "value".to_string()
    } else {
        sanitize_field_ident(&converted)
    }
}

fn sanitize_variant_ident(value: &str) -> String {
    sanitize_type_ident(value)
}

fn sanitize_type_ident(value: &str) -> String {
    let mut ident = value
        .chars()
        .map(|ch| if ch.is_ascii_alphanumeric() { ch } else { '_' })
        .collect::<String>();
    if ident.chars().next().is_some_and(|ch| ch.is_ascii_digit()) {
        ident = format!("Value{ident}");
    }
    if ident.is_empty() {
        "Value".to_string()
    } else {
        ident
    }
}

fn sanitize_field_ident(value: &str) -> String {
    let mut ident = value
        .chars()
        .map(|ch| if ch.is_ascii_alphanumeric() || ch == '_' { ch } else { '_' })
        .collect::<String>();
    if ident.chars().next().is_some_and(|ch| ch.is_ascii_digit()) {
        ident = format!("value_{ident}");
    }
    if is_rust_keyword(&ident) {
        ident = format!("{ident}_value");
    }
    ident
}

fn is_rust_keyword(value: &str) -> bool {
    matches!(
        value,
        "as" | "break"
            | "const"
            | "continue"
            | "crate"
            | "else"
            | "enum"
            | "extern"
            | "false"
            | "fn"
            | "for"
            | "if"
            | "impl"
            | "in"
            | "let"
            | "loop"
            | "match"
            | "mod"
            | "move"
            | "mut"
            | "pub"
            | "ref"
            | "return"
            | "self"
            | "static"
            | "struct"
            | "super"
            | "trait"
            | "true"
            | "type"
            | "unsafe"
            | "use"
            | "where"
            | "while"
            | "async"
            | "await"
            | "dyn"
            | "try"
    )
}
