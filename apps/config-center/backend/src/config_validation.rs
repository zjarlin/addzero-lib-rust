use anyhow::{bail, Context, Result};

/// Normalizes an optional user-provided string.
#[must_use]
pub fn normalize_optional(value: Option<String>) -> Option<String> {
    value
        .map(|value| value.trim().to_owned())
        .filter(|value| !value.is_empty())
}

/// Normalizes a required string and rejects blank input.
///
/// # Errors
/// Returns an error when the trimmed value is empty.
pub fn normalize_required(label: &str, value: &str) -> Result<String> {
    let value = value.trim();
    if value.is_empty() {
        bail!("{label}不能为空");
    }
    Ok(value.to_owned())
}

/// Normalizes and validates the supported config value type.
///
/// # Errors
/// Returns an error for unsupported types.
pub fn normalize_value_type(value: &str) -> Result<String> {
    let value = value.trim();
    if matches!(value, "text" | "json" | "number" | "boolean" | "secret") {
        Ok(value.to_owned())
    } else {
        bail!("配置类型只能是 text/json/number/boolean/secret");
    }
}

/// Validates text according to the declared config value type.
///
/// # Errors
/// Returns an error when the value cannot be represented as the declared type.
pub fn validate_config_value(value_type: &str, value: &str) -> Result<()> {
    match value_type {
        "text" | "secret" => Ok(()),
        "json" => validate_json_config_value(value),
        "number" => validate_number_config_value(value),
        "boolean" => validate_boolean_config_value(value),
        _ => bail!("配置类型只能是 text/json/number/boolean/secret"),
    }
}

fn validate_json_config_value(value: &str) -> Result<()> {
    serde_json::from_str::<serde_json::Value>(value)
        .map(|_| ())
        .context("JSON 配置值不合法")
}

fn validate_number_config_value(value: &str) -> Result<()> {
    serde_json::from_str::<serde_json::Number>(value.trim())
        .map(|_| ())
        .context("数字配置值不合法")
}

fn validate_boolean_config_value(value: &str) -> Result<()> {
    if matches!(value.trim(), "true" | "false") {
        Ok(())
    } else {
        bail!("布尔配置值只能是 true 或 false");
    }
}

#[cfg(test)]
mod tests {
    use super::{normalize_value_type, validate_config_value};

    #[test]
    fn normalize_value_type_rejects_unknown_types() {
        assert!(normalize_value_type("json").is_ok());
        assert!(normalize_value_type("yaml").is_err());
    }

    #[test]
    fn validate_config_value_rejects_invalid_json() {
        let error = validate_config_value("json", "{bad").unwrap_err();

        // 关键断言：JSON 类型不能把错误文本写入正式配置。
        assert!(error.to_string().contains("JSON 配置值不合法"));
    }

    #[test]
    fn validate_config_value_rejects_invalid_number() {
        let error = validate_config_value("number", "NaN").unwrap_err();

        // 关键断言：数字类型必须保持 JSON number 兼容，方便跨语言 SDK 读取。
        assert!(error.to_string().contains("数字配置值不合法"));
    }

    #[test]
    fn validate_config_value_rejects_non_strict_boolean() {
        let error = validate_config_value("boolean", "yes").unwrap_err();

        // 关键断言：布尔类型和 Kotlin strict boolean 解码保持一致。
        assert!(error.to_string().contains("布尔配置值只能是 true 或 false"));
    }
}
