use std::rc::Rc;

#[cfg(not(target_arch = "wasm32"))]
use std::time::Duration;

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::services::StoredLogoDto;

pub use super::LocalBoxFuture;

const DEFAULT_SITE_NAME: &str = "MSC_AIO";
const DEFAULT_BRAND_COPY: &str = "顶部品牌区默认使用 App 图标，可切换为上传品牌资产。";
const DEFAULT_HEADER_BADGE: &str = "Knowledge Workspace";

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BrandingLogoSource {
    #[default]
    AppIcon,
    CustomUpload,
    TextOnly,
}

impl BrandingLogoSource {
    #[cfg(not(target_arch = "wasm32"))]
    fn as_db_value(self) -> &'static str {
        match self {
            Self::AppIcon => "app_icon",
            Self::CustomUpload => "custom_upload",
            Self::TextOnly => "text_only",
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn from_db_value(value: &str) -> Self {
        match value {
            "custom_upload" => Self::CustomUpload,
            "text_only" => Self::TextOnly,
            _ => Self::AppIcon,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::AppIcon => "App 图标",
            Self::CustomUpload => "自定义上传",
            Self::TextOnly => "仅文字",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct BrandingSettingsDto {
    pub site_name: String,
    pub logo_source: BrandingLogoSource,
    pub logo: Option<StoredLogoDto>,
    pub brand_copy: String,
    pub header_badge: String,
}

impl Default for BrandingSettingsDto {
    fn default() -> Self {
        Self {
            site_name: DEFAULT_SITE_NAME.to_string(),
            logo_source: BrandingLogoSource::AppIcon,
            logo: None,
            brand_copy: DEFAULT_BRAND_COPY.to_string(),
            header_badge: DEFAULT_HEADER_BADGE.to_string(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct BrandingSettingsUpdate {
    pub site_name: String,
    pub logo_source: BrandingLogoSource,
    pub logo: Option<StoredLogoDto>,
    pub brand_copy: String,
    pub header_badge: String,
}

impl From<BrandingSettingsDto> for BrandingSettingsUpdate {
    fn from(value: BrandingSettingsDto) -> Self {
        Self {
            site_name: value.site_name,
            logo_source: value.logo_source,
            logo: value.logo,
            brand_copy: value.brand_copy,
            header_badge: value.header_badge,
        }
    }
}

#[derive(Clone, Debug, Error, PartialEq, Eq)]
pub enum BrandingSettingsError {
    #[error("{0}")]
    Message(String),
}

impl BrandingSettingsError {
    fn new(message: impl Into<String>) -> Self {
        Self::Message(message.into())
    }
}

pub type BrandingSettingsResult<T> = Result<T, BrandingSettingsError>;

pub trait BrandingSettingsApi: 'static {
    fn load_settings(&self) -> LocalBoxFuture<'_, BrandingSettingsResult<BrandingSettingsDto>>;
    fn save_settings(
        &self,
        input: BrandingSettingsUpdate,
    ) -> LocalBoxFuture<'_, BrandingSettingsResult<BrandingSettingsDto>>;
}

pub type SharedBrandingSettingsApi = Rc<dyn BrandingSettingsApi>;

pub fn default_branding_settings_api() -> SharedBrandingSettingsApi {
    #[cfg(target_arch = "wasm32")]
    {
        Rc::new(BrowserBrandingSettingsApi)
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        Rc::new(EmbeddedBrandingSettingsApi)
    }
}

#[cfg(target_arch = "wasm32")]
struct BrowserBrandingSettingsApi;

#[cfg(target_arch = "wasm32")]
impl BrandingSettingsApi for BrowserBrandingSettingsApi {
    fn load_settings(&self) -> LocalBoxFuture<'_, BrandingSettingsResult<BrandingSettingsDto>> {
        Box::pin(async move {
            super::browser_http::get_json("/api/admin/settings/branding")
                .await
                .map_err(BrandingSettingsError::new)
        })
    }

    fn save_settings(
        &self,
        input: BrandingSettingsUpdate,
    ) -> LocalBoxFuture<'_, BrandingSettingsResult<BrandingSettingsDto>> {
        Box::pin(async move {
            super::browser_http::post_json("/api/admin/settings/branding", &input)
                .await
                .map_err(BrandingSettingsError::new)
        })
    }
}

#[cfg(not(target_arch = "wasm32"))]
struct EmbeddedBrandingSettingsApi;

#[cfg(not(target_arch = "wasm32"))]
impl BrandingSettingsApi for EmbeddedBrandingSettingsApi {
    fn load_settings(&self) -> LocalBoxFuture<'_, BrandingSettingsResult<BrandingSettingsDto>> {
        Box::pin(async move { load_branding_settings_on_server().await })
    }

    fn save_settings(
        &self,
        input: BrandingSettingsUpdate,
    ) -> LocalBoxFuture<'_, BrandingSettingsResult<BrandingSettingsDto>> {
        Box::pin(async move { save_branding_settings_on_server(input).await })
    }
}

#[cfg(not(target_arch = "wasm32"))]
const BRANDING_SETTINGS_SCHEMA_SQL: &str =
    include_str!("../server/migrations/0006_admin_branding_settings.sql");

#[cfg(not(target_arch = "wasm32"))]
pub async fn load_branding_settings_on_server() -> BrandingSettingsResult<BrandingSettingsDto> {
    let pool = connect_pool().await?;
    ensure_branding_schema(&pool).await?;
    read_branding_settings(&pool).await
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn save_branding_settings_on_server(
    input: BrandingSettingsUpdate,
) -> BrandingSettingsResult<BrandingSettingsDto> {
    validate_update(&input)?;
    let pool = connect_pool().await?;
    ensure_branding_schema(&pool).await?;
    let existing = read_branding_settings(&pool).await.unwrap_or_default();
    let logo = input.logo.or(existing.logo);

    sqlx::query(
        r#"
        INSERT INTO admin_branding_settings (
            singleton_key,
            site_name,
            logo_source,
            logo_object_key,
            logo_relative_path,
            logo_file_name,
            logo_content_type,
            logo_backend_label,
            brand_copy,
            header_badge,
            updated_at
        )
        VALUES ('default', $1, $2, $3, $4, $5, $6, $7, $8, $9, NOW())
        ON CONFLICT (singleton_key)
        DO UPDATE SET
            site_name = EXCLUDED.site_name,
            logo_source = EXCLUDED.logo_source,
            logo_object_key = EXCLUDED.logo_object_key,
            logo_relative_path = EXCLUDED.logo_relative_path,
            logo_file_name = EXCLUDED.logo_file_name,
            logo_content_type = EXCLUDED.logo_content_type,
            logo_backend_label = EXCLUDED.logo_backend_label,
            brand_copy = EXCLUDED.brand_copy,
            header_badge = EXCLUDED.header_badge,
            updated_at = NOW()
        "#,
    )
    .bind(input.site_name.trim())
    .bind(input.logo_source.as_db_value())
    .bind(logo.as_ref().map(|logo| logo.object_key.as_str()))
    .bind(logo.as_ref().map(|logo| logo.relative_path.as_str()))
    .bind(logo.as_ref().map(|logo| logo.file_name.as_str()))
    .bind(logo.as_ref().map(|logo| logo.content_type.as_str()))
    .bind(logo.as_ref().map(|logo| logo.backend_label.as_str()))
    .bind(input.brand_copy.trim())
    .bind(input.header_badge.trim())
    .execute(&pool)
    .await
    .map_err(query_error)?;

    read_branding_settings(&pool).await
}

#[cfg(not(target_arch = "wasm32"))]
async fn connect_pool() -> BrandingSettingsResult<sqlx::postgres::PgPool> {
    let database_url = addzero_persistence::database_url().ok_or_else(|| {
        BrandingSettingsError::new(
            "缺少 PostgreSQL 连接：请设置 MSC_AIO_DATABASE_URL 或 DATABASE_URL",
        )
    })?;

    sqlx::postgres::PgPoolOptions::new()
        .max_connections(4)
        .acquire_timeout(Duration::from_secs(5))
        .connect(&database_url)
        .await
        .map_err(|err| BrandingSettingsError::new(format!("连接 PostgreSQL 失败：{err}")))
}

#[cfg(not(target_arch = "wasm32"))]
async fn ensure_branding_schema(pool: &sqlx::postgres::PgPool) -> BrandingSettingsResult<()> {
    for statement in BRANDING_SETTINGS_SCHEMA_SQL.split(';') {
        let trimmed = statement.trim();
        if trimmed.is_empty() {
            continue;
        }
        sqlx::query(trimmed)
            .execute(pool)
            .await
            .map_err(query_error)?;
    }
    Ok(())
}

#[cfg(not(target_arch = "wasm32"))]
async fn read_branding_settings(
    pool: &sqlx::postgres::PgPool,
) -> BrandingSettingsResult<BrandingSettingsDto> {
    use sqlx::Row;

    let row = sqlx::query(
        r#"
        SELECT
            site_name,
            logo_source,
            logo_object_key,
            logo_relative_path,
            logo_file_name,
            logo_content_type,
            logo_backend_label,
            brand_copy,
            header_badge
        FROM admin_branding_settings
        WHERE singleton_key = 'default'
        "#,
    )
    .fetch_optional(pool)
    .await
    .map_err(query_error)?;

    let Some(row) = row else {
        return Ok(BrandingSettingsDto::default());
    };

    let logo = row_to_logo(&row);

    Ok(BrandingSettingsDto {
        site_name: row.get::<String, _>("site_name"),
        logo_source: BrandingLogoSource::from_db_value(
            row.get::<String, _>("logo_source").as_str(),
        ),
        logo,
        brand_copy: row.get::<String, _>("brand_copy"),
        header_badge: row.get::<String, _>("header_badge"),
    })
}

#[cfg(not(target_arch = "wasm32"))]
fn row_to_logo(row: &sqlx::postgres::PgRow) -> Option<StoredLogoDto> {
    use sqlx::Row;

    Some(StoredLogoDto {
        object_key: row.try_get::<String, _>("logo_object_key").ok()?,
        relative_path: row.try_get::<String, _>("logo_relative_path").ok()?,
        file_name: row.try_get::<String, _>("logo_file_name").ok()?,
        content_type: row.try_get::<String, _>("logo_content_type").ok()?,
        backend_label: row.try_get::<String, _>("logo_backend_label").ok()?,
    })
}

#[cfg(not(target_arch = "wasm32"))]
fn validate_update(input: &BrandingSettingsUpdate) -> BrandingSettingsResult<()> {
    if input.site_name.trim().is_empty() {
        return Err(BrandingSettingsError::new("品牌名称不能为空"));
    }

    if input.logo_source == BrandingLogoSource::CustomUpload && input.logo.is_none() {
        return Err(BrandingSettingsError::new(
            "选择自定义上传时需要先上传一个 logo",
        ));
    }

    Ok(())
}

#[cfg(not(target_arch = "wasm32"))]
fn query_error(err: sqlx::Error) -> BrandingSettingsError {
    BrandingSettingsError::new(format!("读写品牌设置失败：{err}"))
}

#[cfg(test)]
mod tests {
    use super::BrandingLogoSource;

    #[test]
    fn logo_source_should_serialize_as_snake_case() {
        let encoded = serde_json::to_string(&BrandingLogoSource::CustomUpload).expect("serialize");

        assert_eq!(encoded, "\"custom_upload\"");
    }
}
