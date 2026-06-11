#![forbid(unsafe_code)]

use anyhow::{Context, Result};
use axum::{
    Json, Router,
    extract::{FromRequestParts, Query, State},
    http::{StatusCode, request::Parts},
    response::{Html, IntoResponse, Response},
    routing::{get, post},
};
use az_config_center_contract::{
    ApiResponse, ConfigItem, DeleteRequest, DeleteResult, ErrorBody, GetQuery, ListQuery,
    LoginPayload, LoginRequest, StatusPayload, ToggleRequest, UpsertRequest,
};
use clap::Parser;
use ring::{
    pbkdf2,
    rand::{SecureRandom, SystemRandom},
};
use sha2::{Digest, Sha256};
use sqlx::{PgPool, Row, postgres::PgPoolOptions};
use std::{env, net::SocketAddr, num::NonZeroU32, time::Duration};
use thiserror::Error;
use url::Url;
use uuid::Uuid;

const DEFAULT_BIND: &str = "0.0.0.0:8080";
const DEFAULT_DATABASE_NAME: &str = "config-center";
const DEFAULT_DATABASE_URL: &str =
    "postgresql://postgres:postgres@macmini.local:5432/config-center";
const DEFAULT_ADMIN_USERNAME: &str = "admin";
const CONNECT_TIMEOUT: Duration = Duration::from_secs(5);
const PASSWORD_HASH_VERSION: &str = "pbkdf2_sha256";
const PASSWORD_HASH_ITERATIONS: u32 = 210_000;
const PASSWORD_SALT_BYTES: usize = 16;
const PASSWORD_HASH_BYTES: usize = 32;

#[derive(Debug, Parser)]
#[command(name = "az-config-center-app")]
#[command(about = "独立部署的中文配置中心")]
struct Cli {
    /// 服务监听地址，例如 0.0.0.0:8080。
    #[arg(long)]
    bind: Option<String>,

    /// PostgreSQL 连接串，数据库名固定建议使用 config-center。
    #[arg(long)]
    database_url: Option<String>,
}

#[derive(Clone)]
struct AppState {
    pool: PgPool,
}

#[derive(Debug, Error)]
enum AppError {
    #[error("{0}")]
    BadRequest(String),
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),
    #[error("未登录或登录已失效")]
    Unauthorized,
}

#[derive(Debug, Clone)]
struct AuthSession {
    user_id: Uuid,
    username: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    let bind = cli
        .bind
        .or_else(|| env::var("CONFIG_CENTER_BIND").ok())
        .unwrap_or_else(|| {
            env::var("PORT")
                .map(|port| format!("0.0.0.0:{port}"))
                .unwrap_or_else(|_| DEFAULT_BIND.to_owned())
        })
        .parse::<SocketAddr>()
        .context("监听地址无效")?;
    let database_url = cli
        .database_url
        .or_else(|| env::var("CONFIG_CENTER_DATABASE_URL").ok())
        .or_else(|| env::var("DATABASE_URL").ok())
        .unwrap_or_else(|| DEFAULT_DATABASE_URL.to_owned());
    let database_url = database_url_for_database(&database_url, DEFAULT_DATABASE_NAME)?;
    let pool = connect_config_center_pool(&database_url).await?;
    ensure_schema(&pool).await?;
    ensure_default_admin(&pool).await?;

    let app = build_router(AppState { pool });
    let listener = tokio::net::TcpListener::bind(bind)
        .await
        .with_context(|| format!("绑定监听地址失败：{bind}"))?;
    println!("配置中心已启动：http://{bind}");
    axum::serve(listener, app).await?;
    Ok(())
}

fn build_router(state: AppState) -> Router {
    Router::new()
        .route("/", get(index_page))
        .route("/favicon.ico", get(favicon))
        .route("/health", get(health))
        .route("/api/v1/auth/login", post(login))
        .route("/api/v1/config/status", get(api_status))
        .route("/api/v1/config/list", get(list_configs))
        .route("/api/v1/config/detail", get(get_config))
        .route(
            "/api/v1/config/value",
            get(get_config_value).put(put_config_value),
        )
        .route("/api/v1/config/upsert", post(upsert_config))
        .route("/api/v1/config/toggle", post(toggle_config))
        .route("/api/v1/config/delete", post(delete_config))
        .with_state(state)
}

async fn health() -> &'static str {
    "ok"
}

async fn api_status(
    State(state): State<AppState>,
    session: AuthSession,
) -> Result<Json<ApiResponse<StatusPayload>>, AppError> {
    let _ = (session.user_id, session.username.as_str());
    sqlx::query_scalar::<_, i64>("SELECT 1::BIGINT")
        .fetch_one(&state.pool)
        .await?;
    Ok(success(
        "配置中心运行正常",
        StatusPayload {
            ok: true,
            database: DEFAULT_DATABASE_NAME.to_owned(),
        },
    ))
}

async fn login(
    State(state): State<AppState>,
    Json(request): Json<LoginRequest>,
) -> Result<Json<ApiResponse<LoginPayload>>, AppError> {
    let username = normalize_required("用户名", &request.username)?;
    let password = normalize_required("密码", &request.password)?;
    let row = sqlx::query(
        r#"
        SELECT id, username, password_hash
        FROM config_users
        WHERE username = $1 AND enabled = TRUE
        "#,
    )
    .bind(&username)
    .fetch_optional(&state.pool)
    .await?;

    let Some(row) = row else {
        return Err(AppError::Unauthorized);
    };
    let user_id: Uuid = row.get("id");
    let stored_hash: String = row.get("password_hash");
    if !verify_password_hash(&stored_hash, &password) {
        return Err(AppError::Unauthorized);
    }

    let token = format!("cc_{}", Uuid::new_v4().simple());
    let token_hash = token_hash(&token);
    sqlx::query(
        r#"
        INSERT INTO config_tokens (id, user_id, token_hash)
        VALUES ($1, $2, $3)
        "#,
    )
    .bind(Uuid::new_v4())
    .bind(user_id)
    .bind(token_hash)
    .execute(&state.pool)
    .await?;

    Ok(success("登录成功", LoginPayload { token, username }))
}

async fn list_configs(
    State(state): State<AppState>,
    session: AuthSession,
    Query(query): Query<ListQuery>,
) -> Result<Json<ApiResponse<Vec<ConfigItem>>>, AppError> {
    let _ = (session.user_id, session.username.as_str());
    let namespace = normalize_optional(query.namespace);
    let keyword = normalize_optional(query.keyword);
    let include_disabled = query.include_disabled.unwrap_or(false);
    let rows = sqlx::query(
        r#"
        SELECT id, namespace, config_key, config_value, value_type, description,
               enabled, version, updated_by, created_at, updated_at
        FROM config_items
        WHERE ($1::TEXT IS NULL OR namespace = $1)
          AND ($2::TEXT IS NULL OR config_key ILIKE ('%' || $2 || '%') OR description ILIKE ('%' || $2 || '%'))
          AND ($3::BOOLEAN OR enabled = TRUE)
        ORDER BY namespace ASC, config_key ASC
        "#,
    )
    .bind(namespace)
    .bind(keyword)
    .bind(include_disabled)
    .fetch_all(&state.pool)
    .await?;

    Ok(success(
        "查询成功",
        rows.into_iter().map(config_item_from_row).collect(),
    ))
}

async fn get_config(
    State(state): State<AppState>,
    session: AuthSession,
    Query(query): Query<GetQuery>,
) -> Result<Json<ApiResponse<ConfigItem>>, AppError> {
    let _ = (session.user_id, session.username.as_str());
    let namespace = normalize_required("命名空间", &query.namespace)?;
    let key = normalize_required("配置键", &query.key)?;
    let row = sqlx::query(
        r#"
        SELECT id, namespace, config_key, config_value, value_type, description,
               enabled, version, updated_by, created_at, updated_at
        FROM config_items
        WHERE namespace = $1 AND config_key = $2
        "#,
    )
    .bind(namespace)
    .bind(key)
    .fetch_optional(&state.pool)
    .await?;

    let Some(row) = row else {
        return Err(AppError::BadRequest("配置不存在".to_owned()));
    };
    Ok(success("查询成功", config_item_from_row(row)))
}

async fn get_config_value(
    State(state): State<AppState>,
    session: AuthSession,
    Query(query): Query<GetQuery>,
) -> Result<Json<ApiResponse<Option<ConfigItem>>>, AppError> {
    let _ = (session.user_id, session.username.as_str());
    let namespace = normalize_required("命名空间", &query.namespace)?;
    let key = normalize_required("配置键", &query.key)?;
    let row = sqlx::query(
        r#"
        SELECT id, namespace, config_key, config_value, value_type, description,
               enabled, version, updated_by, created_at, updated_at
        FROM config_items
        WHERE namespace = $1 AND config_key = $2 AND enabled = TRUE
        "#,
    )
    .bind(namespace)
    .bind(key)
    .fetch_optional(&state.pool)
    .await?;
    let item = row.map(config_item_from_row);
    let message = if item.is_some() {
        "查询成功"
    } else {
        "配置不存在"
    };

    Ok(success(message, item))
}

async fn upsert_config(
    State(state): State<AppState>,
    session: AuthSession,
    Json(request): Json<UpsertRequest>,
) -> Result<Json<ApiResponse<ConfigItem>>, AppError> {
    let item = upsert_config_item(&state.pool, session, request).await?;
    Ok(success("保存成功", item))
}

async fn put_config_value(
    State(state): State<AppState>,
    session: AuthSession,
    Json(request): Json<UpsertRequest>,
) -> Result<Json<ApiResponse<ConfigItem>>, AppError> {
    let item = upsert_config_item(&state.pool, session, request).await?;
    Ok(success("写入成功", item))
}

async fn upsert_config_item(
    pool: &PgPool,
    session: AuthSession,
    request: UpsertRequest,
) -> Result<ConfigItem, AppError> {
    let AuthSession {
        user_id: _,
        username,
    } = session;
    let UpsertRequest {
        namespace,
        key,
        value,
        value_type,
        description,
        enabled,
        updated_by,
    } = request;
    let namespace = normalize_required("命名空间", &namespace)?;
    let key = normalize_required("配置键", &key)?;
    let value_type = normalize_value_type(&value_type)?;
    validate_config_value(&value_type, &value)?;
    let updated_by = normalize_optional(Some(updated_by)).unwrap_or(username);
    let id = Uuid::new_v4();
    let row = sqlx::query(
        r#"
        INSERT INTO config_items (
            id, namespace, config_key, config_value, value_type, description, enabled, updated_by
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
        ON CONFLICT (namespace, config_key) DO UPDATE
        SET config_value = EXCLUDED.config_value,
            value_type = EXCLUDED.value_type,
            description = EXCLUDED.description,
            enabled = EXCLUDED.enabled,
            updated_by = EXCLUDED.updated_by,
            version = config_items.version + 1,
            updated_at = NOW()
        RETURNING id, namespace, config_key, config_value, value_type, description,
                  enabled, version, updated_by, created_at, updated_at
        "#,
    )
    .bind(id)
    .bind(namespace)
    .bind(key)
    .bind(value)
    .bind(value_type)
    .bind(description.trim())
    .bind(enabled)
    .bind(updated_by)
    .fetch_one(pool)
    .await?;

    Ok(config_item_from_row(row))
}

async fn toggle_config(
    State(state): State<AppState>,
    session: AuthSession,
    Json(request): Json<ToggleRequest>,
) -> Result<Json<ApiResponse<ConfigItem>>, AppError> {
    let _ = session.user_id;
    let namespace = normalize_required("命名空间", &request.namespace)?;
    let key = normalize_required("配置键", &request.key)?;
    let updated_by = normalize_optional(Some(request.updated_by)).unwrap_or(session.username);
    let row = sqlx::query(
        r#"
        UPDATE config_items
        SET enabled = $3,
            updated_by = $4,
            version = version + 1,
            updated_at = NOW()
        WHERE namespace = $1 AND config_key = $2
        RETURNING id, namespace, config_key, config_value, value_type, description,
                  enabled, version, updated_by, created_at, updated_at
        "#,
    )
    .bind(namespace)
    .bind(key)
    .bind(request.enabled)
    .bind(updated_by)
    .fetch_optional(&state.pool)
    .await?;

    let Some(row) = row else {
        return Err(AppError::BadRequest("配置不存在".to_owned()));
    };
    Ok(success("状态已更新", config_item_from_row(row)))
}

async fn delete_config(
    State(state): State<AppState>,
    session: AuthSession,
    Json(request): Json<DeleteRequest>,
) -> Result<Json<ApiResponse<DeleteResult>>, AppError> {
    let _ = (session.user_id, session.username.as_str());
    let namespace = normalize_required("命名空间", &request.namespace)?;
    let key = normalize_required("配置键", &request.key)?;
    let deleted = sqlx::query("DELETE FROM config_items WHERE namespace = $1 AND config_key = $2")
        .bind(namespace)
        .bind(key)
        .execute(&state.pool)
        .await?
        .rows_affected();
    Ok(success("删除完成", DeleteResult { deleted }))
}

async fn connect_config_center_pool(database_url: &str) -> Result<PgPool> {
    match connect_pool(database_url).await {
        Ok(pool) => Ok(pool),
        Err(error) if is_missing_database_error(&error) => {
            create_database(database_url).await?;
            connect_pool(database_url)
                .await
                .context("连接 config-center 数据库失败")
        }
        Err(error) => Err(error).context("连接 PostgreSQL 失败"),
    }
}

async fn connect_pool(database_url: &str) -> Result<PgPool, sqlx::Error> {
    PgPoolOptions::new()
        .max_connections(8)
        .acquire_timeout(CONNECT_TIMEOUT)
        .connect(database_url)
        .await
}

async fn create_database(database_url: &str) -> Result<()> {
    let database_name = database_name_from_url(database_url)?;
    let maintenance_url = database_url_for_database(database_url, "postgres")?;
    let pool = connect_pool(&maintenance_url)
        .await
        .context("连接 PostgreSQL 维护库失败，无法创建 config-center 数据库")?;
    let sql = format!("CREATE DATABASE {}", quote_pg_identifier(&database_name));
    match sqlx::query(&sql).execute(&pool).await {
        Ok(_) => Ok(()),
        Err(error) if is_duplicate_database_error(&error) => Ok(()),
        Err(error) => Err(error).context("创建 config-center 数据库失败"),
    }
}

async fn ensure_schema(pool: &PgPool) -> Result<()> {
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS config_users (
            id UUID PRIMARY KEY,
            username TEXT NOT NULL UNIQUE,
            password_hash TEXT NOT NULL,
            enabled BOOLEAN NOT NULL DEFAULT TRUE,
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            CONSTRAINT config_users_username_not_blank CHECK (length(trim(username)) > 0)
        )
        "#,
    )
    .execute(pool)
    .await
    .context("创建用户表失败")?;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS config_tokens (
            id UUID PRIMARY KEY,
            user_id UUID NOT NULL REFERENCES config_users(id) ON DELETE CASCADE,
            token_hash TEXT NOT NULL UNIQUE,
            revoked BOOLEAN NOT NULL DEFAULT FALSE,
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        )
        "#,
    )
    .execute(pool)
    .await
    .context("创建令牌表失败")?;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS config_items (
            id UUID PRIMARY KEY,
            namespace TEXT NOT NULL,
            config_key TEXT NOT NULL,
            config_value TEXT NOT NULL,
            value_type TEXT NOT NULL DEFAULT 'text',
            description TEXT NOT NULL DEFAULT '',
            enabled BOOLEAN NOT NULL DEFAULT TRUE,
            version INTEGER NOT NULL DEFAULT 1,
            updated_by TEXT NOT NULL DEFAULT 'system',
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            CONSTRAINT config_items_unique_key UNIQUE (namespace, config_key),
            CONSTRAINT config_items_namespace_not_blank CHECK (length(trim(namespace)) > 0),
            CONSTRAINT config_items_key_not_blank CHECK (length(trim(config_key)) > 0),
            CONSTRAINT config_items_value_type_valid CHECK (value_type IN ('text', 'json', 'number', 'boolean', 'secret'))
        )
        "#,
    )
    .execute(pool)
    .await
    .context("创建配置表失败")?;

    sqlx::query(
        r#"
        CREATE INDEX IF NOT EXISTS idx_config_items_namespace
        ON config_items (namespace)
        "#,
    )
    .execute(pool)
    .await
    .context("创建命名空间索引失败")?;

    sqlx::query(
        r#"
        CREATE INDEX IF NOT EXISTS idx_config_items_keyword
        ON config_items USING GIN (to_tsvector('simple', config_key || ' ' || description))
        "#,
    )
    .execute(pool)
    .await
    .context("创建关键字索引失败")?;

    Ok(())
}

async fn ensure_default_admin(pool: &PgPool) -> Result<()> {
    let user_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM config_users")
        .fetch_one(pool)
        .await
        .context("读取配置中心用户数量失败")?;
    if user_count > 0 {
        return Ok(());
    }

    let username = env::var("CONFIG_CENTER_ADMIN_USERNAME")
        .ok()
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| DEFAULT_ADMIN_USERNAME.to_owned());
    let password = env::var("CONFIG_CENTER_ADMIN_PASSWORD")
        .ok()
        .filter(|value| !value.trim().is_empty())
        .or_else(|| {
            env::var("CONFIG_CENTER_ALLOW_DEFAULT_ADMIN")
                .ok()
                .filter(|value| value == "true")
                .map(|_| "admin".to_owned())
        })
        .context("首次启动必须设置 CONFIG_CENTER_ADMIN_PASSWORD")?;
    let password_hash = create_password_hash(&password)?;
    sqlx::query(
        r#"
        INSERT INTO config_users (id, username, password_hash)
        VALUES ($1, $2, $3)
        ON CONFLICT (username) DO NOTHING
        "#,
    )
    .bind(Uuid::new_v4())
    .bind(username)
    .bind(password_hash)
    .execute(pool)
    .await
    .context("初始化默认管理员失败")?;
    Ok(())
}

fn config_item_from_row(row: sqlx::postgres::PgRow) -> ConfigItem {
    ConfigItem {
        id: row.get("id"),
        namespace: row.get("namespace"),
        config_key: row.get("config_key"),
        config_value: row.get("config_value"),
        value_type: row.get("value_type"),
        description: row.get("description"),
        enabled: row.get("enabled"),
        version: row.get("version"),
        updated_by: row.get("updated_by"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

fn success<T>(message: impl Into<String>, data: T) -> Json<ApiResponse<T>> {
    Json(ApiResponse {
        success: true,
        message: message.into(),
        data: Some(data),
    })
}

fn normalize_optional(value: Option<String>) -> Option<String> {
    value
        .map(|value| value.trim().to_owned())
        .filter(|value| !value.is_empty())
}

fn normalize_required(label: &str, value: &str) -> Result<String, AppError> {
    let value = value.trim();
    if value.is_empty() {
        return Err(AppError::BadRequest(format!("{label}不能为空")));
    }
    Ok(value.to_owned())
}

fn normalize_value_type(value: &str) -> Result<String, AppError> {
    let value = value.trim();
    if matches!(value, "text" | "json" | "number" | "boolean" | "secret") {
        Ok(value.to_owned())
    } else {
        Err(AppError::BadRequest(
            "配置类型只能是 text/json/number/boolean/secret".to_owned(),
        ))
    }
}

fn validate_config_value(value_type: &str, value: &str) -> Result<(), AppError> {
    match value_type {
        "text" | "secret" => Ok(()),
        "json" => validate_json_config_value(value),
        "number" => validate_number_config_value(value),
        "boolean" => validate_boolean_config_value(value),
        _ => Err(AppError::BadRequest(
            "配置类型只能是 text/json/number/boolean/secret".to_owned(),
        )),
    }
}

fn validate_json_config_value(value: &str) -> Result<(), AppError> {
    serde_json::from_str::<serde_json::Value>(value)
        .map(|_| ())
        .map_err(|error| AppError::BadRequest(format!("JSON 配置值不合法：{error}")))
}

fn validate_number_config_value(value: &str) -> Result<(), AppError> {
    serde_json::from_str::<serde_json::Number>(value.trim())
        .map(|_| ())
        .map_err(|error| AppError::BadRequest(format!("数字配置值不合法：{error}")))
}

fn validate_boolean_config_value(value: &str) -> Result<(), AppError> {
    if matches!(value.trim(), "true" | "false") {
        Ok(())
    } else {
        Err(AppError::BadRequest(
            "布尔配置值只能是 true 或 false".to_owned(),
        ))
    }
}

fn create_password_hash(password: &str) -> Result<String> {
    let mut salt = [0_u8; PASSWORD_SALT_BYTES];
    SystemRandom::new()
        .fill(&mut salt)
        .map_err(|_| anyhow::anyhow!("生成密码盐失败"))?;
    let mut hash = [0_u8; PASSWORD_HASH_BYTES];
    pbkdf2::derive(
        pbkdf2::PBKDF2_HMAC_SHA256,
        password_iterations(),
        &salt,
        password.as_bytes(),
        &mut hash,
    );
    Ok(format!(
        "{}${}${}${}",
        PASSWORD_HASH_VERSION,
        PASSWORD_HASH_ITERATIONS,
        hex_encode(&salt),
        hex_encode(&hash)
    ))
}

fn verify_password_hash(stored_hash: &str, password: &str) -> bool {
    let parts = stored_hash.split('$').collect::<Vec<_>>();
    let [version, iterations, salt, hash] = parts.as_slice() else {
        return false;
    };
    if *version != PASSWORD_HASH_VERSION {
        return false;
    }
    let Ok(iterations) = iterations.parse::<u32>() else {
        return false;
    };
    let Some(iterations) = NonZeroU32::new(iterations) else {
        return false;
    };
    let Some(salt) = hex_decode(salt) else {
        return false;
    };
    let Some(hash) = hex_decode(hash) else {
        return false;
    };
    pbkdf2::verify(
        pbkdf2::PBKDF2_HMAC_SHA256,
        iterations,
        &salt,
        password.as_bytes(),
        &hash,
    )
    .is_ok()
}

fn password_iterations() -> NonZeroU32 {
    match NonZeroU32::new(PASSWORD_HASH_ITERATIONS) {
        Some(iterations) => iterations,
        None => NonZeroU32::MIN,
    }
}

fn token_hash(token: &str) -> String {
    hex_sha256(&format!("config-center-token:{token}"))
}

fn hex_sha256(value: &str) -> String {
    let digest = Sha256::digest(value.as_bytes());
    hex_encode(&digest)
}

fn hex_encode(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn hex_decode(value: &str) -> Option<Vec<u8>> {
    if !value.len().is_multiple_of(2) {
        return None;
    }
    value
        .as_bytes()
        .chunks(2)
        .map(|chunk| {
            let high = hex_value(chunk[0])?;
            let low = hex_value(chunk[1])?;
            Some((high << 4) | low)
        })
        .collect()
}

fn hex_value(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        b'A'..=b'F' => Some(byte - b'A' + 10),
        _ => None,
    }
}

fn database_url_for_database(database_url: &str, database_name: &str) -> Result<String> {
    let mut url = Url::parse(database_url).context("PostgreSQL 连接串格式无效")?;
    if !is_postgres_url(database_url) {
        anyhow::bail!("仅支持 postgres:// 或 postgresql:// 连接");
    }
    url.set_path(database_name);
    Ok(url.to_string())
}

fn database_name_from_url(database_url: &str) -> Result<String> {
    let url = Url::parse(database_url).context("PostgreSQL 连接串格式无效")?;
    let name = url.path().trim_start_matches('/').trim();
    if name.is_empty() {
        anyhow::bail!("PostgreSQL 连接缺少数据库名");
    }
    Ok(name.to_owned())
}

fn is_postgres_url(value: &str) -> bool {
    value.starts_with("postgres://") || value.starts_with("postgresql://")
}

fn is_missing_database_error(error: &sqlx::Error) -> bool {
    error.to_string().contains("database") && error.to_string().contains("does not exist")
        || error.to_string().contains("3D000")
}

fn is_duplicate_database_error(error: &sqlx::Error) -> bool {
    error.to_string().contains("already exists") || error.to_string().contains("42P04")
}

fn quote_pg_identifier(value: &str) -> String {
    format!("\"{}\"", value.replace('"', "\"\""))
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let status = match self {
            AppError::BadRequest(_) => StatusCode::BAD_REQUEST,
            AppError::Sqlx(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::Unauthorized => StatusCode::UNAUTHORIZED,
        };
        let body = Json(ErrorBody {
            success: false,
            message: self.to_string(),
        });
        (status, body).into_response()
    }
}

impl FromRequestParts<AppState> for AuthSession {
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let Some(header) = parts.headers.get("authorization") else {
            return Err(AppError::Unauthorized);
        };
        let Ok(header) = header.to_str() else {
            return Err(AppError::Unauthorized);
        };
        let Some(token) = header.strip_prefix("Bearer ").map(str::trim) else {
            return Err(AppError::Unauthorized);
        };
        if token.is_empty() {
            return Err(AppError::Unauthorized);
        }
        let row = sqlx::query(
            r#"
            SELECT u.id, u.username
            FROM config_tokens t
            JOIN config_users u ON u.id = t.user_id
            WHERE t.token_hash = $1
              AND t.revoked = FALSE
              AND u.enabled = TRUE
            "#,
        )
        .bind(token_hash(token))
        .fetch_optional(&state.pool)
        .await?;
        let Some(row) = row else {
            return Err(AppError::Unauthorized);
        };
        Ok(Self {
            user_id: row.get("id"),
            username: row.get("username"),
        })
    }
}

async fn index_page() -> Html<&'static str> {
    Html(INDEX_HTML)
}

async fn favicon() -> impl IntoResponse {
    (
        StatusCode::OK,
        [("content-type", "image/svg+xml; charset=utf-8")],
        r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 64 64"><rect width="64" height="64" rx="10" fill="#1f7a4c"/><text x="32" y="42" font-size="34" font-family="serif" text-anchor="middle" fill="#fff">配</text></svg>"##,
    )
}

const INDEX_HTML: &str = r#"<!doctype html>
<html lang="zh-CN">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <title>配置中心</title>
  <style>
    :root {
      --bg: #f4f1e8;
      --paper: #fffaf0;
      --ink: #17211a;
      --muted: #66715f;
      --line: #d8d0bf;
      --green: #1f7a4c;
      --green-deep: #125c38;
      --red: #ba3d30;
      --blue: #225d8f;
      --gold: #b07a22;
      --shadow: 0 16px 36px rgba(35, 39, 28, 0.12);
    }
    * { box-sizing: border-box; }
    body {
      margin: 0;
      min-height: 100vh;
      background:
        linear-gradient(90deg, rgba(31, 122, 76, 0.08) 1px, transparent 1px),
        linear-gradient(0deg, rgba(31, 122, 76, 0.06) 1px, transparent 1px),
        var(--bg);
      background-size: 34px 34px;
      color: var(--ink);
      font-family: ui-serif, "Songti SC", "Noto Serif CJK SC", Georgia, serif;
    }
    button, input, select, textarea {
      font: inherit;
    }
    .shell {
      min-height: 100vh;
      display: grid;
      grid-template-columns: 280px minmax(0, 1fr);
    }
    .rail {
      padding: 24px;
      border-right: 1px solid var(--line);
      background: rgba(255, 250, 240, 0.82);
      backdrop-filter: blur(10px);
      position: sticky;
      top: 0;
      height: 100vh;
    }
    .brand {
      display: flex;
      align-items: center;
      gap: 12px;
      margin-bottom: 28px;
    }
    .seal {
      width: 42px;
      height: 42px;
      display: grid;
      place-items: center;
      background: var(--green);
      color: #fff;
      border-radius: 6px;
      box-shadow: 6px 6px 0 rgba(176, 122, 34, 0.25);
      font-weight: 800;
    }
    .brand h1 {
      margin: 0;
      font-size: 24px;
      line-height: 1.1;
      letter-spacing: 0;
    }
    .brand p {
      margin: 4px 0 0;
      color: var(--muted);
      font-size: 13px;
    }
    .metric {
      border: 1px solid var(--line);
      background: rgba(255, 250, 240, 0.9);
      padding: 14px;
      margin-bottom: 12px;
      border-radius: 8px;
    }
    .metric strong {
      display: block;
      font-size: 24px;
      color: var(--green-deep);
    }
    .metric span {
      color: var(--muted);
      font-size: 13px;
    }
    .nav-title {
      margin: 24px 0 10px;
      color: var(--muted);
      font-size: 12px;
      text-transform: uppercase;
    }
    .namespace-list {
      display: grid;
      gap: 8px;
    }
    .namespace-btn {
      border: 1px solid transparent;
      background: transparent;
      color: var(--ink);
      width: 100%;
      text-align: left;
      padding: 9px 10px;
      border-radius: 7px;
      cursor: pointer;
    }
    .namespace-btn.active, .namespace-btn:hover {
      border-color: var(--line);
      background: #fff7df;
    }
    .main {
      padding: 28px;
      display: grid;
      gap: 18px;
      align-content: start;
    }
    .topbar {
      display: grid;
      grid-template-columns: minmax(0, 1fr) auto;
      gap: 16px;
      align-items: end;
    }
    .title h2 {
      margin: 0;
      font-size: clamp(30px, 4vw, 54px);
      line-height: 0.95;
      letter-spacing: 0;
    }
    .title p {
      max-width: 720px;
      margin: 12px 0 0;
      color: var(--muted);
      line-height: 1.7;
    }
    .actions {
      display: flex;
      gap: 10px;
      flex-wrap: wrap;
      justify-content: flex-end;
    }
    .btn {
      border: 1px solid var(--green-deep);
      background: var(--green);
      color: #fff;
      border-radius: 8px;
      padding: 10px 14px;
      cursor: pointer;
      min-height: 42px;
    }
    .btn.secondary {
      background: #fffaf0;
      color: var(--green-deep);
    }
    .btn.danger {
      background: var(--red);
      border-color: var(--red);
    }
    .filters, .editor, .client-skill {
      border: 1px solid var(--line);
      background: rgba(255, 250, 240, 0.92);
      border-radius: 8px;
      box-shadow: var(--shadow);
    }
    .filters {
      display: grid;
      grid-template-columns: 1fr 1fr auto;
      gap: 12px;
      padding: 14px;
      align-items: end;
    }
    label {
      display: grid;
      gap: 6px;
      color: var(--muted);
      font-size: 13px;
    }
    input, select, textarea {
      width: 100%;
      border: 1px solid var(--line);
      background: #fffdf7;
      color: var(--ink);
      border-radius: 7px;
      padding: 10px 11px;
      outline: none;
    }
    input:focus, select:focus, textarea:focus {
      border-color: var(--green);
      box-shadow: 0 0 0 3px rgba(31, 122, 76, 0.14);
    }
    .content-grid {
      display: grid;
      grid-template-columns: minmax(0, 1fr) minmax(320px, 420px);
      gap: 18px;
      align-items: start;
    }
    .table-wrap {
      border: 1px solid var(--line);
      border-radius: 8px;
      background: rgba(255, 250, 240, 0.92);
      overflow: hidden;
      box-shadow: var(--shadow);
    }
    table {
      width: 100%;
      border-collapse: collapse;
      font-size: 14px;
    }
    th, td {
      padding: 12px;
      border-bottom: 1px solid var(--line);
      vertical-align: top;
      text-align: left;
    }
    th {
      background: #efe8d4;
      color: #3d4a36;
      white-space: nowrap;
    }
    tbody tr {
      cursor: pointer;
    }
    tbody tr:hover, tbody tr.selected {
      background: #fff3ce;
    }
    .mono {
      font-family: "SFMono-Regular", Consolas, monospace;
      word-break: break-word;
    }
    .pill {
      display: inline-flex;
      align-items: center;
      border-radius: 999px;
      padding: 3px 8px;
      font-size: 12px;
      border: 1px solid var(--line);
      background: #fffdf7;
      white-space: nowrap;
    }
    .pill.on { color: var(--green-deep); border-color: rgba(31, 122, 76, 0.35); }
    .pill.off { color: var(--red); border-color: rgba(186, 61, 48, 0.35); }
    .editor {
      padding: 16px;
      display: grid;
      gap: 12px;
      position: sticky;
      top: 24px;
    }
    .editor h3 {
      margin: 0;
      font-size: 22px;
    }
    textarea {
      min-height: 168px;
      resize: vertical;
      font-family: "SFMono-Regular", Consolas, monospace;
      line-height: 1.5;
    }
    .row {
      display: grid;
      grid-template-columns: 1fr 1fr;
      gap: 10px;
    }
    .checkline {
      display: flex;
      align-items: center;
      gap: 8px;
      color: var(--ink);
    }
    .checkline input {
      width: 18px;
      height: 18px;
    }
    .message {
      min-height: 24px;
      color: var(--blue);
      font-size: 14px;
    }
    .empty {
      padding: 30px;
      color: var(--muted);
      text-align: center;
    }
    .client-skill {
      padding: 18px;
      display: grid;
      gap: 14px;
    }
    .client-skill__head {
      display: flex;
      align-items: start;
      justify-content: space-between;
      gap: 14px;
      flex-wrap: wrap;
    }
    .client-skill h3 {
      margin: 0;
      font-size: 24px;
    }
    .client-skill p {
      margin: 6px 0 0;
      color: var(--muted);
      line-height: 1.7;
    }
    .skill-grid {
      display: grid;
      grid-template-columns: minmax(0, 0.8fr) minmax(0, 1.2fr);
      gap: 14px;
    }
    .skill-notes {
      margin: 0;
      padding-left: 18px;
      color: var(--ink);
      line-height: 1.8;
    }
    .skill-notes li::marker {
      color: var(--green);
    }
    pre.code-block {
      margin: 0;
      overflow: auto;
      border: 1px solid var(--line);
      border-radius: 8px;
      background: #15231c;
      color: #f2f8ea;
      padding: 14px;
      font-family: "SFMono-Regular", Consolas, monospace;
      font-size: 13px;
      line-height: 1.55;
      white-space: pre;
    }
    .login-panel {
      min-height: 100vh;
      display: grid;
      place-items: center;
      padding: 24px;
    }
    .login-card {
      width: min(420px, 100%);
      display: grid;
      gap: 14px;
      border: 1px solid var(--line);
      background: rgba(255, 250, 240, 0.95);
      border-radius: 8px;
      box-shadow: var(--shadow);
      padding: 24px;
    }
    .login-card h1 {
      margin: 0;
      font-size: 34px;
      line-height: 1;
    }
    .login-card p {
      margin: 0 0 8px;
      color: var(--muted);
      line-height: 1.7;
    }
    body.authed .login-panel { display: none; }
    body:not(.authed) .shell { display: none; }
    @media (max-width: 980px) {
      .shell { grid-template-columns: 1fr; }
      .rail {
        position: static;
        height: auto;
        border-right: 0;
        border-bottom: 1px solid var(--line);
      }
      .content-grid { grid-template-columns: 1fr; }
      .skill-grid { grid-template-columns: 1fr; }
      .editor { position: static; }
      .topbar { grid-template-columns: 1fr; }
      .actions { justify-content: flex-start; }
    }
    @media (max-width: 680px) {
      .main, .rail { padding: 18px; }
      .filters, .row { grid-template-columns: 1fr; }
      table, thead, tbody, th, td, tr { display: block; }
      thead { display: none; }
      tr { border-bottom: 1px solid var(--line); }
      td { border-bottom: 0; padding: 8px 12px; }
      td::before {
        content: attr(data-label);
        display: block;
        color: var(--muted);
        font-size: 12px;
        margin-bottom: 2px;
      }
    }
  </style>
</head>
<body>
  <section class="login-panel">
    <form id="loginForm" class="login-card">
      <div class="seal">配</div>
      <h1>配置中心登录</h1>
      <p>登录后选择命名空间并管理运行配置。</p>
      <label>用户名
        <input id="loginUsername" autocomplete="username" required placeholder="admin">
      </label>
      <label>密码
        <input id="loginPassword" autocomplete="current-password" type="password" required>
      </label>
      <button class="btn" type="submit">登录</button>
      <div id="loginMessage" class="message"></div>
    </form>
  </section>
  <div class="shell">
    <aside class="rail">
      <div class="brand">
        <div class="seal">配</div>
        <div>
          <h1>配置中心</h1>
          <p>config-center / PostgreSQL</p>
        </div>
      </div>
      <div class="metric"><strong id="totalCount">0</strong><span>配置总数</span></div>
      <div class="metric"><strong id="enabledCount">0</strong><span>已启用</span></div>
      <div class="nav-title">命名空间</div>
      <div id="namespaceList" class="namespace-list"></div>
    </aside>
    <main class="main">
      <section class="topbar">
        <div class="title">
          <h2>集中管理运行配置</h2>
          <p>面向服务、环境与功能开关的轻量配置中心。所有配置写入 Mac mini 服务器上的 PostgreSQL 数据库 config-center。</p>
        </div>
        <div class="actions">
          <button class="btn secondary" id="refreshBtn" type="button">刷新</button>
          <button class="btn secondary" id="logoutBtn" type="button">退出</button>
          <button class="btn" id="newBtn" type="button">新建配置</button>
        </div>
      </section>

      <section class="filters">
        <label>命名空间
          <input id="namespaceFilter" placeholder="例如 prod / dev / aio">
        </label>
        <label>关键词
          <input id="keywordFilter" placeholder="搜索配置键或说明">
        </label>
        <label class="checkline">
          <input id="includeDisabled" type="checkbox">
          显示停用配置
        </label>
      </section>

      <section class="content-grid">
        <div class="table-wrap">
          <table>
            <thead>
              <tr>
                <th>命名空间</th>
                <th>配置键</th>
                <th>类型</th>
                <th>状态</th>
                <th>版本</th>
                <th>更新时间</th>
              </tr>
            </thead>
            <tbody id="configTable"></tbody>
          </table>
          <div id="emptyState" class="empty" hidden>暂无配置</div>
        </div>

        <form id="editor" class="editor">
          <h3>配置编辑</h3>
          <div class="row">
            <label>命名空间
              <input id="namespaceInput" required placeholder="prod">
            </label>
            <label>配置键
              <input id="keyInput" required placeholder="service.timeout">
            </label>
          </div>
          <div class="row">
            <label>类型
              <select id="typeInput">
                <option value="text">文本</option>
                <option value="json">JSON</option>
                <option value="number">数字</option>
                <option value="boolean">布尔</option>
                <option value="secret">密钥</option>
              </select>
            </label>
            <label>更新人
              <input id="updatedByInput" value="admin">
            </label>
          </div>
          <label>配置值
            <textarea id="valueInput" required placeholder="配置值"></textarea>
          </label>
          <label>说明
            <input id="descriptionInput" placeholder="这条配置影响什么">
          </label>
          <label class="checkline">
            <input id="enabledInput" type="checkbox" checked>
            启用
          </label>
          <div class="actions">
            <button class="btn" type="submit">保存</button>
            <button class="btn secondary" id="toggleBtn" type="button">切换状态</button>
            <button class="btn danger" id="deleteBtn" type="button">删除</button>
          </div>
          <div id="message" class="message"></div>
        </form>
      </section>

      <section class="client-skill">
        <div class="client-skill__head">
          <div>
            <h3>客户端 Skill：Kotlin SDK 接入</h3>
            <p>服务端通过登录接口签发 Bearer token；SDK 登录后切换命名空间，再按 key 读写强类型配置。</p>
          </div>
          <button class="btn secondary" id="copyKotlinSkillBtn" type="button">复制 Markdown Skill</button>
        </div>
        <div class="skill-grid">
          <ul class="skill-notes">
            <li>右侧是可直接保存为 <span class="mono">SKILL.md</span> 的 Markdown 文本。</li>
            <li>Skill 内包含 Kotlin SDK 登录、命名空间绑定、读取和写入规范。</li>
            <li><span class="mono">set</span> 已覆盖 String、Int/Number、Boolean、data class/JSON 对象。</li>
            <li>密码必须从环境变量或密钥系统读取，不写入源码。</li>
          </ul>
          <pre id="kotlinSkillCode" class="code-block"><code># Config Center Kotlin Multiplatform SDK Skill

Use this skill when a Kotlin service or Compose Multiplatform frontend needs to read or write runtime configuration from Config Center.

## Dependency

Use the KMP SDK module:

```text
site.addzero:tool-config-center-client:2026.06.11
```

Source module:

```text
/Users/zjarlin/aio/workspace/zjarlin/addzero-lib-jvm/lib/tool-kmp/tool-config-center-client
```

## Required Imports

```kotlin
import site.addzero.configcenter.ConfigCenter
```

## Login And Namespace

Do not hardcode passwords. Read them from environment variables, secret stores, or runtime injection.

```kotlin
val instance = ConfigCenter("http://127.0.0.1:18080")
    .login("zjarlin", System.getenv("CONFIG_CENTER_PASSWORD"))
    .checkoutNamespace("cmp-aio.dev")
```

`checkoutNamespace("cmp-aio.dev")` binds all following `get` and `set` calls to that namespace.

## Read Config

```kotlin
val text: String? = instance.get("app.name")
val timeout: Int? = instance.get("service.timeout")
val enabled: Boolean? = instance.get("feature.enabled")
val redis: RedisConfig? = instance.get("redis")
```

For object configs, define a serializable Kotlin shape:

```kotlin
@kotlinx.serialization.Serializable
data class RedisConfig(
    val host: String,
    val port: Int,
)
```

## Write Config With set

`set(key, value)` infers the config value type:

```kotlin
// text: value_type = "text"
instance.set("app.name", "cmp-aio")

// number: value_type = "number"
instance.set("service.timeout", 30)
instance.set("rate.limit", 12.5)

// boolean: value_type = "boolean"
instance.set("feature.enabled", true)

// json object: value_type = "json"
instance.set("redis", RedisConfig("127.0.0.1", 6379))
```

Use the overload with description when the meaning is not obvious:

```kotlin
instance.set(
    "redis",
    RedisConfig("127.0.0.1", 6379),
    "CMP AIO development Redis connection"
)
```

## Rules

- Always call `login(...)` before config access.
- Always call `checkoutNamespace(...)` before `get` or `set`.
- Treat returned values as nullable: missing config returns `null`.
- Never store production passwords in code or config files.
- Prefer data classes for structured JSON values.</code></pre>
        </div>
      </section>
    </main>
  </div>

  <script>
    const state = {
      items: [],
      selected: null,
      token: localStorage.getItem("configCenterToken") || "",
      username: localStorage.getItem("configCenterUsername") || "",
    };
    const $ = (id) => document.getElementById(id);

    function message(text, isError = false) {
      $("message").textContent = text;
      $("message").style.color = isError ? "var(--red)" : "var(--blue)";
    }

    async function api(url, options = {}) {
      const headers = {
        "content-type": "application/json",
        ...(options.headers || {}),
      };
      if (state.token) {
        headers.authorization = `Bearer ${state.token}`;
      }
      const response = await fetch(url, {
        ...options,
        headers,
      });
      const body = await response.json();
      if (!response.ok || body.success === false) {
        if (response.status === 401) {
          logout("登录已失效，请重新登录");
        }
        throw new Error(body.message || "请求失败");
      }
      return body.data;
    }

    async function login(event) {
      event.preventDefault();
      $("loginMessage").textContent = "";
      try {
        const data = await api("/api/v1/auth/login", {
          method: "POST",
          body: JSON.stringify({
            username: $("loginUsername").value,
            password: $("loginPassword").value,
          }),
        });
        state.token = data.token;
        state.username = data.username;
        localStorage.setItem("configCenterToken", state.token);
        localStorage.setItem("configCenterUsername", state.username);
        $("updatedByInput").value = state.username;
        document.body.classList.add("authed");
        await loadConfigs();
      } catch (error) {
        $("loginMessage").textContent = error.message;
        $("loginMessage").style.color = "var(--red)";
      }
    }

    function logout(note = "") {
      state.token = "";
      state.username = "";
      localStorage.removeItem("configCenterToken");
      localStorage.removeItem("configCenterUsername");
      document.body.classList.remove("authed");
      state.items = [];
      state.selected = null;
      render();
      $("loginMessage").textContent = note;
      $("loginMessage").style.color = "var(--red)";
    }

    function queryString() {
      const params = new URLSearchParams();
      const namespace = $("namespaceFilter").value.trim();
      const keyword = $("keywordFilter").value.trim();
      if (namespace) params.set("namespace", namespace);
      if (keyword) params.set("keyword", keyword);
      params.set("include_disabled", $("includeDisabled").checked ? "true" : "false");
      return params.toString();
    }

    async function loadConfigs() {
      try {
        const data = await api(`/api/v1/config/list?${queryString()}`);
        state.items = data;
        render();
        message("已刷新");
      } catch (error) {
        message(error.message, true);
      }
    }

    function render() {
      const tbody = $("configTable");
      tbody.innerHTML = "";
      $("emptyState").hidden = state.items.length > 0;
      $("totalCount").textContent = state.items.length;
      $("enabledCount").textContent = state.items.filter((item) => item.enabled).length;

      const namespaces = [...new Set(state.items.map((item) => item.namespace))];
      $("namespaceList").innerHTML = "";
      const allBtn = namespaceButton("全部", "");
      $("namespaceList").appendChild(allBtn);
      for (const namespace of namespaces) {
        $("namespaceList").appendChild(namespaceButton(namespace, namespace));
      }

      for (const item of state.items) {
        const tr = document.createElement("tr");
        if (state.selected && state.selected.id === item.id) tr.classList.add("selected");
        tr.innerHTML = `
          <td data-label="命名空间">${escapeHtml(item.namespace)}</td>
          <td data-label="配置键" class="mono">${escapeHtml(item.config_key)}</td>
          <td data-label="类型"><span class="pill">${escapeHtml(typeLabel(item.value_type))}</span></td>
          <td data-label="状态"><span class="pill ${item.enabled ? "on" : "off"}">${item.enabled ? "启用" : "停用"}</span></td>
          <td data-label="版本">v${item.version}</td>
          <td data-label="更新时间">${formatDate(item.updated_at)}</td>
        `;
        tr.addEventListener("click", () => selectItem(item));
        tbody.appendChild(tr);
      }
    }

    function namespaceButton(label, value) {
      const btn = document.createElement("button");
      btn.type = "button";
      btn.className = "namespace-btn";
      btn.textContent = label;
      if ($("namespaceFilter").value.trim() === value) btn.classList.add("active");
      btn.addEventListener("click", () => {
        $("namespaceFilter").value = value;
        loadConfigs();
      });
      return btn;
    }

    function selectItem(item) {
      state.selected = item;
      $("namespaceInput").value = item.namespace;
      $("keyInput").value = item.config_key;
      $("typeInput").value = item.value_type;
      $("valueInput").value = item.config_value;
      $("descriptionInput").value = item.description;
      $("enabledInput").checked = item.enabled;
      $("updatedByInput").value = item.updated_by || "admin";
      render();
      message(`已选择 ${item.namespace}/${item.config_key}`);
    }

    function clearEditor() {
      state.selected = null;
      $("editor").reset();
      $("typeInput").value = "text";
      $("enabledInput").checked = true;
      $("updatedByInput").value = state.username || "admin";
      render();
      message("准备新建配置");
    }

    async function saveConfig(event) {
      event.preventDefault();
      try {
        const payload = {
          namespace: $("namespaceInput").value,
          key: $("keyInput").value,
          value: $("valueInput").value,
          value_type: $("typeInput").value,
          description: $("descriptionInput").value,
          enabled: $("enabledInput").checked,
          updated_by: $("updatedByInput").value,
        };
        const item = await api("/api/v1/config/upsert", {
          method: "POST",
          body: JSON.stringify(payload),
        });
        state.selected = item;
        await loadConfigs();
        message("保存成功");
      } catch (error) {
        message(error.message, true);
      }
    }

    async function toggleConfig() {
      if (!state.selected) return message("请先选择一条配置", true);
      try {
        const item = await api("/api/v1/config/toggle", {
          method: "POST",
          body: JSON.stringify({
            namespace: state.selected.namespace,
            key: state.selected.config_key,
            enabled: !state.selected.enabled,
            updated_by: $("updatedByInput").value || "admin",
          }),
        });
        state.selected = item;
        await loadConfigs();
        message("状态已更新");
      } catch (error) {
        message(error.message, true);
      }
    }

    async function deleteConfig() {
      if (!state.selected) return message("请先选择一条配置", true);
      if (!confirm(`删除 ${state.selected.namespace}/${state.selected.config_key}？`)) return;
      try {
        await api("/api/v1/config/delete", {
          method: "POST",
          body: JSON.stringify({
            namespace: state.selected.namespace,
            key: state.selected.config_key,
          }),
        });
        clearEditor();
        await loadConfigs();
        message("删除完成");
      } catch (error) {
        message(error.message, true);
      }
    }

    function typeLabel(value) {
      return { text: "文本", json: "JSON", number: "数字", boolean: "布尔", secret: "密钥" }[value] || value;
    }

    function formatDate(value) {
      return new Intl.DateTimeFormat("zh-CN", {
        dateStyle: "short",
        timeStyle: "short",
      }).format(new Date(value));
    }

    function escapeHtml(value) {
      return String(value).replace(/[&<>"']/g, (char) => ({
        "&": "&amp;",
        "<": "&lt;",
        ">": "&gt;",
        '"': "&quot;",
        "'": "&#039;",
      }[char]));
    }

    async function copyKotlinSkill() {
      const code = $("kotlinSkillCode").innerText;
      try {
        await navigator.clipboard.writeText(code);
        message("Markdown Skill 已复制");
      } catch (_error) {
        message("复制失败，请手动选择代码", true);
      }
    }

    $("editor").addEventListener("submit", saveConfig);
    $("loginForm").addEventListener("submit", login);
    $("refreshBtn").addEventListener("click", loadConfigs);
    $("logoutBtn").addEventListener("click", () => logout());
    $("newBtn").addEventListener("click", clearEditor);
    $("toggleBtn").addEventListener("click", toggleConfig);
    $("deleteBtn").addEventListener("click", deleteConfig);
    $("copyKotlinSkillBtn").addEventListener("click", copyKotlinSkill);
    $("namespaceFilter").addEventListener("input", () => loadConfigs());
    $("keywordFilter").addEventListener("input", () => loadConfigs());
    $("includeDisabled").addEventListener("change", () => loadConfigs());
    if (state.token) {
      document.body.classList.add("authed");
      $("updatedByInput").value = state.username || "admin";
      loadConfigs();
    }
  </script>
</body>
</html>
"#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn database_url_is_for_config_center_database() {
        let url = database_url_for_database(
            "postgresql://postgres:secret@macmini.local:5432/postgres?sslmode=disable",
            DEFAULT_DATABASE_NAME,
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
