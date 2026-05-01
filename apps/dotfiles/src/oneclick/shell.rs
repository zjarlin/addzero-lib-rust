use std::path::Path;

pub(crate) fn quote_ps(value: &str) -> String {
    format!("'{}'", value.replace('\'', "''"))
}

pub(crate) fn quote_ps_path(path: &Path) -> String {
    quote_ps(&path.to_string_lossy())
}
