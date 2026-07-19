use serde::de::DeserializeOwned;
use serde_json::{Value, json};

/// Responses API 结构化输出的 JSON Schema 元数据。
#[derive(Debug, Clone, PartialEq)]
pub struct StructuredOutputSpec {
    /// 发送给模型的 schema 名称。
    pub name: String,
    /// 便于阅读的 schema 描述。
    pub description: Option<String>,
    /// JSON Schema 对象。
    pub schema: Value,
    /// 模型是否必须严格遵守 schema。
    pub strict: bool,
}

impl StructuredOutputSpec {
    /// 创建严格结构化输出 schema。
    pub fn strict(name: impl Into<String>, description: impl Into<String>, schema: Value) -> Self {
        Self {
            name: name.into(),
            description: Some(description.into()),
            schema,
            strict: true,
        }
    }

    /// 转换为 Responses API 的 `text.format` JSON payload。
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

/// 将模型返回的 JSON 文本输出解析为强类型值。
pub fn parse_structured_output<T>(text: &str) -> anyhow::Result<T>
where
    T: DeserializeOwned,
{
    Ok(serde_json::from_str(text)?)
}

/// 用于验证结构化输出接线的标准示例 schema。
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
