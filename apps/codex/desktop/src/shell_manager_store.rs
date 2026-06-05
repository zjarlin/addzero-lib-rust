#![forbid(unsafe_code)]

use std::{
    collections::BTreeMap,
    env, fs, io,
    path::{Path, PathBuf},
    time::Duration,
};

use crate::shell_manager::ShellManagerStore;
use sqlx::{PgPool, postgres::PgPoolOptions};
use url::Url;

const CODEX_DATABASE_NAME: &str = "rs-aio";
const STORE_KEY: &str = "default";
const STORE_TABLE: &str = "codex_shell_manager_store";
const CODEX_ENV_FILE: &str = ".config/addzero/codex/codex.env";
const CONNECT_TIMEOUT: Duration = Duration::from_millis(900);

pub(crate) fn load_shell_manager_store(json_path: &Path) -> io::Result<ShellManagerStore> {
    let Some(database_url) = codex_database_url() else {
        return load_json_store(json_path);
    };

    match block_on(load_pg_store_or_seed(&database_url, json_path)) {
        Ok(store) => Ok(store),
        Err(_) => load_json_store(json_path),
    }
}

pub(crate) fn save_shell_manager_store(
    json_path: &Path,
    store: &ShellManagerStore,
) -> io::Result<()> {
    let Some(database_url) = codex_database_url() else {
        return write_json_store(json_path, store);
    };

    match block_on(save_pg_store(&database_url, store)) {
        Ok(()) => write_json_store(json_path, store),
        Err(error) => {
            write_json_store(json_path, store)?;
            Err(io::Error::other(format!(
                "PostgreSQL 保存失败，已写入 JSON 降级文件：{error}"
            )))
        }
    }
}

async fn load_pg_store_or_seed(
    database_url: &str,
    json_path: &Path,
) -> io::Result<ShellManagerStore> {
    let pool = connect_codex_pool(database_url).await?;
    ensure_schema(&pool).await?;

    if let Some(store) = read_pg_store(&pool).await? {
        return Ok(store);
    }

    let store = load_json_store(json_path)?;
    if !store.is_empty() {
        upsert_pg_store(&pool, &store).await?;
    }
    Ok(store)
}

async fn save_pg_store(database_url: &str, store: &ShellManagerStore) -> io::Result<()> {
    let pool = connect_codex_pool(database_url).await?;
    ensure_schema(&pool).await?;
    upsert_pg_store(&pool, store).await
}

async fn connect_codex_pool(database_url: &str) -> io::Result<PgPool> {
    match connect_pool(database_url).await {
        Ok(pool) => Ok(pool),
        Err(error) if is_missing_database_error(&error) => {
            create_database(database_url).await?;
            connect_pool(database_url).await
        }
        Err(error) => Err(error),
    }
}

async fn connect_pool(database_url: &str) -> io::Result<PgPool> {
    PgPoolOptions::new()
        .max_connections(3)
        .acquire_timeout(CONNECT_TIMEOUT)
        .connect(database_url)
        .await
        .map_err(io::Error::other)
}

async fn create_database(database_url: &str) -> io::Result<()> {
    let database_name = database_name_from_url(database_url)?;
    let maintenance_url = database_url_for_database(database_url, "postgres")?;
    let pool = connect_pool(&maintenance_url).await?;
    let sql = format!("CREATE DATABASE {}", quote_pg_identifier(&database_name));

    match sqlx::query(&sql).execute(&pool).await {
        Ok(_) => Ok(()),
        Err(error) if is_duplicate_database_error(&io::Error::other(error.to_string())) => Ok(()),
        Err(error) => Err(io::Error::other(error)),
    }
}

async fn ensure_schema(pool: &PgPool) -> io::Result<()> {
    let sql = format!(
        r#"
        CREATE TABLE IF NOT EXISTS {STORE_TABLE} (
            store_key TEXT PRIMARY KEY,
            store JSONB NOT NULL,
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        )
        "#
    );
    sqlx::query(&sql)
        .execute(pool)
        .await
        .map(|_| ())
        .map_err(io::Error::other)
}

async fn read_pg_store(pool: &PgPool) -> io::Result<Option<ShellManagerStore>> {
    let sql = format!("SELECT store::text FROM {STORE_TABLE} WHERE store_key = $1");
    let row = sqlx::query_as::<_, (String,)>(&sql)
        .bind(STORE_KEY)
        .fetch_optional(pool)
        .await
        .map_err(io::Error::other)?;

    row.map(|(content,)| serde_json::from_str(&content).map_err(io::Error::other))
        .transpose()
}

async fn upsert_pg_store(pool: &PgPool, store: &ShellManagerStore) -> io::Result<()> {
    let content = serde_json::to_string(store).map_err(io::Error::other)?;
    let sql = format!(
        r#"
        INSERT INTO {STORE_TABLE} (store_key, store)
        VALUES ($1, $2::jsonb)
        ON CONFLICT (store_key) DO UPDATE
        SET store = EXCLUDED.store,
            updated_at = NOW()
        "#
    );
    sqlx::query(&sql)
        .bind(STORE_KEY)
        .bind(content)
        .execute(pool)
        .await
        .map(|_| ())
        .map_err(io::Error::other)
}

fn load_json_store(path: &Path) -> io::Result<ShellManagerStore> {
    if !path.exists() {
        return Ok(ShellManagerStore::default());
    }
    let content = fs::read_to_string(path)?;
    serde_json::from_str(&content).map_err(io::Error::other)
}

fn write_json_store(path: &Path, store: &ShellManagerStore) -> io::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let content = serde_json::to_string_pretty(store).map_err(io::Error::other)?;
    fs::write(path, content)
}

fn codex_database_url() -> Option<String> {
    env_database_url("CODEX_DATABASE_URL")
        .or_else(read_codex_env_database_url)
        .filter(|value| is_postgres_url(value))
        .and_then(|value| database_url_for_database(&value, CODEX_DATABASE_NAME).ok())
}

fn env_database_url(key: &str) -> Option<String> {
    env::var(key)
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

fn read_codex_env_database_url() -> Option<String> {
    let path = home_dir()?.join(CODEX_ENV_FILE);
    let content = fs::read_to_string(path).ok()?;
    let values = parse_env_pairs(&content);
    values
        .get("CODEX_DATABASE_URL")
        .or_else(|| values.get("DATABASE_URL"))
        .cloned()
        .filter(|value| is_postgres_url(value))
}

fn parse_env_pairs(content: &str) -> BTreeMap<String, String> {
    content
        .lines()
        .filter_map(|line| {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                return None;
            }
            let (key, value) = trimmed.split_once('=')?;
            Some((
                key.trim().to_string(),
                value
                    .trim()
                    .trim_matches('"')
                    .trim_matches('\'')
                    .to_string(),
            ))
        })
        .collect()
}

fn database_url_for_database(database_url: &str, database_name: &str) -> io::Result<String> {
    let mut url = Url::parse(database_url).map_err(io::Error::other)?;
    if !is_postgres_url(database_url) {
        return Err(io::Error::other("仅支持 postgres:// 或 postgresql:// 连接"));
    }
    url.set_path(database_name);
    Ok(url.to_string())
}

fn database_name_from_url(database_url: &str) -> io::Result<String> {
    let url = Url::parse(database_url).map_err(io::Error::other)?;
    let name = url.path().trim_start_matches('/').trim();
    if name.is_empty() {
        return Err(io::Error::other("PostgreSQL 连接缺少数据库名"));
    }
    Ok(name.to_string())
}

fn is_postgres_url(value: &str) -> bool {
    value.starts_with("postgres://") || value.starts_with("postgresql://")
}

fn is_missing_database_error(error: &io::Error) -> bool {
    error.to_string().contains("database") && error.to_string().contains("does not exist")
        || error.to_string().contains("3D000")
}

fn is_duplicate_database_error(error: &io::Error) -> bool {
    error.to_string().contains("already exists") || error.to_string().contains("42P04")
}

fn quote_pg_identifier(value: &str) -> String {
    format!("\"{}\"", value.replace('"', "\"\""))
}

fn block_on<T>(future: impl std::future::Future<Output = io::Result<T>>) -> io::Result<T> {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(io::Error::other)?
        .block_on(future)
}

fn home_dir() -> Option<PathBuf> {
    env::var_os("HOME").map(PathBuf::from)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn database_url_for_database_targets_rs_aio() {
        let url = database_url_for_database(
            "postgresql://postgres:secret@127.0.0.1:15432/msc_aio?sslmode=disable",
            CODEX_DATABASE_NAME,
        )
        .expect("rewrite database url");

        assert_eq!(
            url,
            "postgresql://postgres:secret@127.0.0.1:15432/rs-aio?sslmode=disable"
        );
    }

    #[test]
    fn quote_pg_identifier_preserves_hyphenated_database_name() {
        assert_eq!(quote_pg_identifier("rs-aio"), "\"rs-aio\"");
    }

    #[test]
    fn parse_env_pairs_reads_quoted_database_url() {
        let values =
            parse_env_pairs("CODEX_DATABASE_URL='postgresql://postgres:secret@localhost/aio'\n");

        assert_eq!(
            values.get("CODEX_DATABASE_URL").map(String::as_str),
            Some("postgresql://postgres:secret@localhost/aio")
        );
    }
}
