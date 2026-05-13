//! Az-edge low-code card contracts and REST interface generation.
//!
//! An `az-edge` node is a configurable execution card, similar to a compact
//! flow node: it declares a runtime variant, input/output parameters, and a
//! template body that can reference input values through `{{param}}`
//! placeholders.

use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

/// Runtime variant used by an `az-edge` execution card.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AzEdgeVariant {
    /// Execute the template as a curl command.
    Curl,
    /// Execute the template as Python source.
    Python,
    /// Execute the template as Rhai source.
    Rhai,
    /// Execute the template as TypeScript source.
    #[serde(rename = "ts", alias = "typescript", alias = "type_script")]
    TypeScript,
}

impl AzEdgeVariant {
    /// Stable wire/runtime label.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Curl => "curl",
            Self::Python => "python",
            Self::Rhai => "rhai",
            Self::TypeScript => "ts",
        }
    }
}

/// HTTP method exposed by the generated REST interface.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AzEdgeHttpMethod {
    /// HTTP GET.
    #[serde(rename = "GET")]
    Get,
    /// HTTP POST.
    #[serde(rename = "POST")]
    Post,
    /// HTTP PUT.
    #[serde(rename = "PUT")]
    Put,
    /// HTTP PATCH.
    #[serde(rename = "PATCH")]
    Patch,
    /// HTTP DELETE.
    #[serde(rename = "DELETE")]
    Delete,
}

impl AzEdgeHttpMethod {
    /// Stable wire label.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Get => "GET",
            Self::Post => "POST",
            Self::Put => "PUT",
            Self::Patch => "PATCH",
            Self::Delete => "DELETE",
        }
    }
}

/// Supported input/output parameter value types.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AzEdgeParamType {
    /// String parameter.
    String,
    /// Numeric parameter.
    Number,
    /// Boolean parameter.
    Boolean,
    /// Arbitrary JSON value.
    Json,
}

impl AzEdgeParamType {
    fn json_schema(self) -> serde_json::Value {
        match self {
            Self::String => serde_json::json!({ "type": "string" }),
            Self::Number => serde_json::json!({ "type": "number" }),
            Self::Boolean => serde_json::json!({ "type": "boolean" }),
            Self::Json => serde_json::json!({}),
        }
    }
}

fn default_required() -> bool {
    true
}

/// A named parameter on an `az-edge` card.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AzEdgeParam {
    /// Parameter name used by templates and generated JSON schemas.
    pub name: String,
    /// Parameter value type.
    #[serde(rename = "type")]
    pub ty: AzEdgeParamType,
    /// Whether the generated REST schema marks this parameter as required.
    #[serde(default = "default_required")]
    pub required: bool,
    /// Optional human-readable description.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Optional default value supplied by the low-code editor.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default_value: Option<serde_json::Value>,
}

/// Configurable `az-edge` execution card specification.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AzEdgeSpec {
    /// Card title shown in the editor.
    pub title: String,
    /// Runtime variant used by this edge node.
    pub variant: AzEdgeVariant,
    /// REST method generated for this card.
    pub method: AzEdgeHttpMethod,
    /// REST path generated for this card, for example `/api/edge/weather`.
    pub path: String,
    /// Runtime template. Input values can be referenced as `{{name}}`.
    pub template: String,
    /// Input parameter definitions.
    #[serde(default)]
    pub inputs: Vec<AzEdgeParam>,
    /// Output parameter definitions.
    #[serde(default)]
    pub outputs: Vec<AzEdgeParam>,
    /// Optional execution timeout in seconds.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub timeout_secs: Option<u64>,
}

impl AzEdgeSpec {
    /// Validates this card and builds a generated REST interface contract.
    pub fn rest_contract(&self) -> Result<AzEdgeRestContract, AzEdgeError> {
        self.validate()?;
        Ok(AzEdgeRestContract {
            operation_id: operation_id_from_path(&self.path),
            method: self.method.as_str().to_string(),
            path: self.path.clone(),
            variant: self.variant,
            request_schema: params_schema(&self.inputs),
            response_schema: params_schema(&self.outputs),
            template: self.template.clone(),
            timeout_secs: self.timeout_secs,
        })
    }

    /// Renders the runtime template by replacing `{{param}}` placeholders.
    pub fn render_template(
        &self,
        values: &serde_json::Map<String, serde_json::Value>,
    ) -> Result<String, AzEdgeError> {
        self.validate_placeholders()?;
        render_template_with_values(&self.template, values)
    }

    /// Validates path, parameter names, duplicates, and placeholders.
    pub fn validate(&self) -> Result<(), AzEdgeError> {
        if self.title.trim().is_empty() {
            return Err(AzEdgeError::MissingField("title"));
        }
        validate_path(&self.path)?;
        validate_params("inputs", &self.inputs)?;
        validate_params("outputs", &self.outputs)?;
        self.validate_placeholders()
    }

    fn validate_placeholders(&self) -> Result<(), AzEdgeError> {
        let inputs = self
            .inputs
            .iter()
            .map(|param| param.name.as_str())
            .collect::<BTreeSet<_>>();
        for placeholder in extract_placeholders(&self.template) {
            if !inputs.contains(placeholder.as_str()) {
                return Err(AzEdgeError::UnknownPlaceholder(placeholder));
            }
        }
        Ok(())
    }
}

/// Generated REST interface contract for an `az-edge` card.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AzEdgeRestContract {
    /// Stable operation id derived from the generated REST path.
    pub operation_id: String,
    /// HTTP method.
    pub method: String,
    /// HTTP path.
    pub path: String,
    /// Runtime variant.
    pub variant: AzEdgeVariant,
    /// JSON schema for request body/query inputs.
    pub request_schema: serde_json::Value,
    /// JSON schema for the response body.
    pub response_schema: serde_json::Value,
    /// Runtime template that the generated handler executes.
    pub template: String,
    /// Optional execution timeout in seconds.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timeout_secs: Option<u64>,
}

/// Errors produced while validating or compiling an `az-edge` card.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum AzEdgeError {
    /// A required field is blank or absent.
    #[error("missing required field: {0}")]
    MissingField(&'static str),
    /// REST path is invalid.
    #[error("invalid REST path: {0}")]
    InvalidPath(String),
    /// A parameter name is invalid.
    #[error("invalid {scope} parameter name: {name}")]
    InvalidParamName {
        /// Parameter scope, usually `inputs` or `outputs`.
        scope: &'static str,
        /// Invalid parameter name.
        name: String,
    },
    /// A parameter is declared more than once.
    #[error("duplicate {scope} parameter name: {name}")]
    DuplicateParamName {
        /// Parameter scope, usually `inputs` or `outputs`.
        scope: &'static str,
        /// Duplicate parameter name.
        name: String,
    },
    /// Template references an input that is not declared.
    #[error("unknown template placeholder: {0}")]
    UnknownPlaceholder(String),
}

fn validate_path(path: &str) -> Result<(), AzEdgeError> {
    if !path.starts_with('/') || path.split_whitespace().count() > 1 {
        return Err(AzEdgeError::InvalidPath(path.to_string()));
    }
    Ok(())
}

fn validate_params(scope: &'static str, params: &[AzEdgeParam]) -> Result<(), AzEdgeError> {
    let mut names = BTreeSet::new();
    for param in params {
        if !is_valid_param_name(&param.name) {
            return Err(AzEdgeError::InvalidParamName {
                scope,
                name: param.name.clone(),
            });
        }
        if !names.insert(param.name.as_str()) {
            return Err(AzEdgeError::DuplicateParamName {
                scope,
                name: param.name.clone(),
            });
        }
    }
    Ok(())
}

fn is_valid_param_name(name: &str) -> bool {
    let mut chars = name.chars();
    match chars.next() {
        Some(first) if first == '_' || first.is_ascii_alphabetic() => {}
        _ => return false,
    }
    chars.all(|ch| ch == '_' || ch.is_ascii_alphanumeric())
}

fn extract_placeholders(template: &str) -> BTreeSet<String> {
    let mut placeholders = BTreeSet::new();
    let mut cursor = 0;
    while let Some(start) = template[cursor..].find("{{") {
        let name_start = cursor + start + 2;
        if let Some(end) = template[name_start..].find("}}") {
            let name_end = name_start + end;
            let name = template[name_start..name_end].trim();
            if !name.is_empty() {
                placeholders.insert(name.to_string());
            }
            cursor = name_end + 2;
        } else {
            break;
        }
    }
    placeholders
}

fn render_template_with_values(
    template: &str,
    values: &serde_json::Map<String, serde_json::Value>,
) -> Result<String, AzEdgeError> {
    let mut output = String::with_capacity(template.len());
    let mut cursor = 0;
    while let Some(start) = template[cursor..].find("{{") {
        let token_start = cursor + start;
        output.push_str(&template[cursor..token_start]);
        let name_start = token_start + 2;
        let Some(end) = template[name_start..].find("}}") else {
            output.push_str(&template[token_start..]);
            return Ok(output);
        };
        let name_end = name_start + end;
        let name = template[name_start..name_end].trim();
        let Some(value) = values.get(name) else {
            return Err(AzEdgeError::UnknownPlaceholder(name.to_string()));
        };
        output.push_str(&template_value_to_string(value));
        cursor = name_end + 2;
    }
    output.push_str(&template[cursor..]);
    Ok(output)
}

fn template_value_to_string(value: &serde_json::Value) -> String {
    match value {
        serde_json::Value::Null => String::new(),
        serde_json::Value::String(value) => value.clone(),
        serde_json::Value::Bool(value) => value.to_string(),
        serde_json::Value::Number(value) => value.to_string(),
        serde_json::Value::Array(_) | serde_json::Value::Object(_) => value.to_string(),
    }
}

fn params_schema(params: &[AzEdgeParam]) -> serde_json::Value {
    let mut properties = serde_json::Map::new();
    let mut required = Vec::new();
    for param in params {
        let mut schema = param.ty.json_schema();
        if let Some(description) = &param.description {
            if let Some(object) = schema.as_object_mut() {
                object.insert(
                    "description".to_string(),
                    serde_json::Value::String(description.clone()),
                );
            }
        }
        if let Some(default_value) = &param.default_value {
            if let Some(object) = schema.as_object_mut() {
                object.insert("default".to_string(), default_value.clone());
            }
        }
        properties.insert(param.name.clone(), schema);
        if param.required {
            required.push(serde_json::Value::String(param.name.clone()));
        }
    }

    serde_json::json!({
        "type": "object",
        "properties": properties,
        "required": required
    })
}

fn operation_id_from_path(path: &str) -> String {
    let id = path
        .trim_matches('/')
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() {
                ch.to_ascii_lowercase()
            } else {
                '_'
            }
        })
        .collect::<String>()
        .split('_')
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join("_");

    if id.is_empty() {
        "az_edge".to_string()
    } else {
        id
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn weather_spec() -> AzEdgeSpec {
        AzEdgeSpec {
            title: "Weather bridge".into(),
            variant: AzEdgeVariant::Curl,
            method: AzEdgeHttpMethod::Post,
            path: "/api/edge/weather".into(),
            template: "curl https://api.example.com/weather?q={{city}}".into(),
            inputs: vec![AzEdgeParam {
                name: "city".into(),
                ty: AzEdgeParamType::String,
                required: true,
                description: Some("City name".into()),
                default_value: None,
            }],
            outputs: vec![AzEdgeParam {
                name: "temperature".into(),
                ty: AzEdgeParamType::Number,
                required: true,
                description: None,
                default_value: None,
            }],
            timeout_secs: Some(10),
        }
    }

    #[test]
    fn rest_contract_should_generate_request_and_response_schema() {
        let contract = weather_spec().rest_contract().unwrap();

        assert_eq!(contract.method, "POST");
        assert_eq!(contract.path, "/api/edge/weather");
        assert_eq!(contract.variant, AzEdgeVariant::Curl);
        assert_eq!(
            contract.request_schema["required"],
            serde_json::json!(["city"])
        );
        assert_eq!(
            contract.response_schema["properties"]["temperature"]["type"],
            "number"
        );
    }

    #[test]
    fn render_template_should_replace_declared_inputs() {
        let mut values = serde_json::Map::new();
        values.insert("city".into(), serde_json::json!("Guangzhou"));

        let rendered = weather_spec().render_template(&values).unwrap();

        assert_eq!(rendered, "curl https://api.example.com/weather?q=Guangzhou");
    }

    #[test]
    fn validate_should_reject_unknown_template_placeholder() {
        let mut spec = weather_spec();
        spec.template = "python run.py --city {{missing}}".into();

        let error = spec.validate().unwrap_err();

        assert_eq!(error, AzEdgeError::UnknownPlaceholder("missing".into()));
    }

    #[test]
    fn variant_should_accept_ts_alias() {
        let variant: AzEdgeVariant = serde_json::from_str(r#""typescript""#).unwrap();

        assert_eq!(variant, AzEdgeVariant::TypeScript);
    }
}
