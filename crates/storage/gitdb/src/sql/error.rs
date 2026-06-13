//! SQL parsing error message helpers.

use anyhow::anyhow;

pub fn syntax(message: impl Into<String>) -> anyhow::Error {
    anyhow!("syntax error: {}", message.into())
}

pub fn unsupported_statement(message: impl Into<String>) -> anyhow::Error {
    anyhow!("unsupported statement: {}", message.into())
}

pub fn unsupported_expression(message: impl Into<String>) -> anyhow::Error {
    anyhow!("unsupported expression: {}", message.into())
}

pub fn unsupported_data_type(message: impl Into<String>) -> anyhow::Error {
    anyhow!("unsupported data type: {}", message.into())
}

pub fn invalid_identifier(message: impl Into<String>) -> anyhow::Error {
    anyhow!("invalid identifier: {}", message.into())
}

pub fn missing_clause(message: impl Into<String>) -> anyhow::Error {
    anyhow!("missing required clause: {}", message.into())
}

pub fn empty_query() -> anyhow::Error {
    anyhow!("empty query")
}

pub fn multiple_statements() -> anyhow::Error {
    anyhow!("multiple statements not supported")
}
