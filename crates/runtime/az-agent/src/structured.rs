use serde::de::DeserializeOwned;
use serde_json::{Value, json};

/// JSON schema metadata for Responses API structured outputs.
#[derive(Debug, Clone, PartialEq)]
pub struct StructuredOutputSpec {
    /// Schema name sent to the model.
    pub name: String,
    /// Human-readable schema description.
    pub description: Option<String>,
    /// JSON Schema object.
    pub schema: Value,
    /// Whether the model must strictly follow the schema.
    pub strict: bool,
}

impl StructuredOutputSpec {
    /// Creates a strict structured output schema.
    pub fn strict(name: impl Into<String>, description: impl Into<String>, schema: Value) -> Self {
        Self {
            name: name.into(),
            description: Some(description.into()),
            schema,
            strict: true,
        }
    }

    /// Converts this spec to the Responses API `text.format` JSON payload.
    pub fn to_response_text_json(&self) -> Value {
        json!({
            "format": {
                "type": "json_schema",
                "name": self.name,
                "description": self.description,
                "schema": self.schema,
                "strict": self.strict,
            }
        })
    }
}

/// Parses a model JSON text output into a typed value.
pub fn parse_structured_output<T>(text: &str) -> anyhow::Result<T>
where
    T: DeserializeOwned,
{
    Ok(serde_json::from_str(text)?)
}

/// Standard demo schema proving structured output wiring.
pub fn time_answer_schema() -> StructuredOutputSpec {
    StructuredOutputSpec::strict(
        "az_time_answer",
        "Answer containing current local time metadata.",
        json!({
            "type": "object",
            "properties": {
                "answer": { "type": "string" },
                "local_datetime": { "type": "string" },
                "timezone": { "type": "string" },
                "utc_offset": { "type": "string" }
            },
            "required": ["answer", "local_datetime", "timezone", "utc_offset"],
            "additionalProperties": false
        }),
    )
}
