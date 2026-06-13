use anyhow::{Context, Result};
use url::Url;

/// Rewrites a PostgreSQL connection string to target a specific database.
///
/// # Errors
/// Returns an error when the URL is invalid or not a PostgreSQL URL.
pub fn database_url_for_database(database_url: &str, database_name: &str) -> Result<String> {
    let mut url = Url::parse(database_url).context("PostgreSQL 连接串格式无效")?;
    if !is_postgres_url(database_url) {
        anyhow::bail!("仅支持 postgres:// 或 postgresql:// 连接");
    }
    url.set_path(database_name);
    Ok(url.to_string())
}

/// Extracts the database name from a PostgreSQL connection string.
///
/// # Errors
/// Returns an error when the URL is invalid or the path has no database name.
pub fn database_name_from_url(database_url: &str) -> Result<String> {
    let url = Url::parse(database_url).context("PostgreSQL 连接串格式无效")?;
    let name = url.path().trim_start_matches('/').trim();
    if name.is_empty() {
        anyhow::bail!("PostgreSQL 连接缺少数据库名");
    }
    Ok(name.to_owned())
}

/// Quotes a PostgreSQL identifier.
#[must_use]
pub fn quote_pg_identifier(value: &str) -> String {
    format!("\"{}\"", value.replace('"', "\"\""))
}

fn is_postgres_url(value: &str) -> bool {
    value.starts_with("postgres://") || value.starts_with("postgresql://")
}

#[cfg(test)]
mod tests {
    use super::{database_url_for_database, quote_pg_identifier};

    #[test]
    fn database_url_is_for_config_center_database() {
        let url = database_url_for_database(
            "postgresql://postgres:secret@macmini.local:5432/postgres?sslmode=disable",
            "config-center",
        )
        .expect("rewrite database url");

        assert_eq!(
            url,
            "postgresql://postgres:secret@macmini.local:5432/config-center?sslmode=disable"
        );
    }

    #[test]
    fn quote_pg_identifier_preserves_hyphenated_database_name() {
        assert_eq!(quote_pg_identifier("config-center"), "\"config-center\"");
    }
}
