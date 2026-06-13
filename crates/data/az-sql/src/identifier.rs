use anyhow::{Result, bail};

pub(crate) fn require_table_name(table: Option<&str>) -> Result<&str> {
    match table {
        Some(table) if !table.trim().is_empty() => Ok(table),
        _ => bail!("no table specified"),
    }
}

/// Quote a SQL identifier using ANSI SQL double-quote convention.
///
/// Escapes embedded double quotes by doubling them (`"` -> `""`).
/// This prevents SQL injection through identifier positions.
pub fn quote_identifier(identifier: &str) -> String {
    let escaped = identifier.replace('"', "\"\"");
    format!("\"{}\"", escaped)
}
