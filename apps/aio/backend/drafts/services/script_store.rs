use once_cell::sync::OnceCell;
use serde::Serialize;
use sqlx::{FromRow, postgres::PgPoolOptions};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, FromRow)]
pub struct StoredScriptDto {
    pub name: String,
    pub source: String,
}

static SCRIPT_STORE_POOL: OnceCell<sqlx::postgres::PgPool> = OnceCell::new();

pub async fn list_scripts() -> Result<Vec<String>, String> {
    let pool = pool().await?;
    let rows = sqlx::query_scalar::<_, String>(
        "SELECT name FROM admin_scripts ORDER BY updated_at DESC, name ASC",
    )
    .fetch_all(&pool)
    .await
    .map_err(|err| err.to_string())?;
    Ok(rows)
}

pub async fn get_script(name: &str) -> Result<Option<StoredScriptDto>, String> {
    let pool = pool().await?;
    sqlx::query_as::<_, StoredScriptDto>("SELECT name, source FROM admin_scripts WHERE name = $1")
        .bind(name)
        .fetch_optional(&pool)
        .await
        .map_err(|err| err.to_string())
}

pub async fn save_script(name: &str, source: &str) -> Result<(), String> {
    if name.trim().is_empty() {
        return Err("script name is required".to_string());
    }
    let pool = pool().await?;
    sqlx::query(
        r#"
        INSERT INTO admin_scripts (name, source, updated_at)
        VALUES ($1, $2, NOW())
        ON CONFLICT (name)
        DO UPDATE SET
            source = EXCLUDED.source,
            updated_at = NOW()
        "#,
    )
    .bind(name.trim())
    .bind(source)
    .execute(&pool)
    .await
    .map_err(|err| err.to_string())?;
    Ok(())
}

pub async fn delete_script(name: &str) -> Result<bool, String> {
    let pool = pool().await?;
    let affected = sqlx::query("DELETE FROM admin_scripts WHERE name = $1")
        .bind(name)
        .execute(&pool)
        .await
        .map_err(|err| err.to_string())?
        .rows_affected();
    Ok(affected > 0)
}

async fn pool() -> Result<sqlx::postgres::PgPool, String> {
    if let Some(pool) = SCRIPT_STORE_POOL.get() {
        return Ok(pool.clone());
    }

    let database_url = crate::server::resolved_database_url()
        .ok_or_else(|| {
            "MSC_AIO_DATABASE_URL / repo .env / DATABASE_URL / ~/.config/aio/aio.env must be set"
                .to_string()
        })?;
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .map_err(|err| err.to_string())?;
    ensure_schema_compat(&pool).await?;
    import_legacy_scripts(&pool).await?;
    let _ = SCRIPT_STORE_POOL.set(pool.clone());
    Ok(pool)
}

async fn ensure_schema_compat(pool: &sqlx::postgres::PgPool) -> Result<(), String> {
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS admin_scripts (
            name TEXT PRIMARY KEY,
            source TEXT NOT NULL,
            updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        )
        "#,
    )
    .execute(pool)
    .await
    .map_err(|err| err.to_string())?;
    sqlx::query(
        r#"
        CREATE INDEX IF NOT EXISTS idx_admin_scripts_updated_at
            ON admin_scripts (updated_at DESC)
        "#,
    )
    .execute(pool)
    .await
    .map_err(|err| err.to_string())?;
    Ok(())
}

async fn import_legacy_scripts(pool: &sqlx::postgres::PgPool) -> Result<(), String> {
    let legacy_dir = std::env::var("AIO_SCRIPTS_DIR")
        .ok()
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| std::path::PathBuf::from("scripts"));

    let Ok(mut entries) = tokio::fs::read_dir(&legacy_dir).await else {
        return Ok(());
    };

    while let Ok(Some(entry)) = entries.next_entry().await {
        let path = entry.path();
        if path.extension().and_then(|ext| ext.to_str()) != Some("rhai") {
            continue;
        }
        let Some(name) = path.file_stem().and_then(|stem| stem.to_str()) else {
            continue;
        };
        let Ok(source) = tokio::fs::read_to_string(&path).await else {
            continue;
        };
        sqlx::query(
            r#"
            INSERT INTO admin_scripts (name, source, updated_at)
            VALUES ($1, $2, NOW())
            ON CONFLICT (name) DO NOTHING
            "#,
        )
        .bind(name)
        .bind(source)
        .execute(pool)
        .await
        .map_err(|err| err.to_string())?;
    }

    Ok(())
}
