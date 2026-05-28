use std::rc::Rc;

#[cfg(not(target_arch = "wasm32"))]
use tokio::sync::OnceCell;

#[cfg(not(target_arch = "wasm32"))]
use std::{
    collections::{BTreeMap, BTreeSet},
    path::{Path, PathBuf},
    time::Duration,
};

use az_derive_aliases::{
    apply, error_eq, plain_clone, plain_clone_debug, serde_code_display_enum, serde_eq_default,
};
#[cfg(not(target_arch = "wasm32"))]
use chrono::{DateTime, Utc};

pub use super::LocalBoxFuture;

#[apply(serde_code_display_enum)]
pub enum AssetKindDto {
    #[display("笔记")]
    Note,
    #[display("软件")]
    Software,
    #[display("安装包")]
    Package,
}

#[apply(serde_eq_default)]
pub struct AssetGraphTagDto {
    pub id: String,
    pub label: String,
    pub item_count: usize,
}

#[apply(serde_eq_default)]
pub struct AssetGraphItemDto {
    pub id: String,
    pub kind: String,
    pub title: String,
    pub detail: String,
    pub source: String,
    pub local_path: Option<String>,
    pub relative_path: Option<String>,
    pub download_url: Option<String>,
    pub content_hash: Option<String>,
    pub hash_algorithm: Option<String>,
    pub size_bytes: Option<u64>,
    pub tags: Vec<String>,
    pub updated_at: Option<String>,
}

#[apply(serde_eq_default)]
pub struct AssetGraphEdgeDto {
    pub source: String,
    pub target: String,
    pub relation: String,
}

#[apply(serde_eq_default)]
pub struct AssetGraphDto {
    pub items: Vec<AssetGraphItemDto>,
    pub tags: Vec<AssetGraphTagDto>,
    pub edges: Vec<AssetGraphEdgeDto>,
    pub note_count: usize,
    pub software_count: usize,
    pub package_count: usize,
    pub warnings: Vec<String>,
}

#[apply(serde_eq_default)]
pub struct AssetSyncReportDto {
    pub notes_imported: usize,
    pub software_imported: usize,
    pub packages_indexed: usize,
    pub tags_indexed: usize,
    pub warnings: Vec<String>,
}

#[apply(error_eq)]
pub enum AssetGraphError {
    #[error("{0}")]
    Message(String),
}

impl AssetGraphError {
    fn new(message: impl Into<String>) -> Self {
        Self::Message(message.into())
    }
}

pub type AssetGraphResult<T> = Result<T, AssetGraphError>;

pub trait AssetGraphApi: 'static {
    fn sync_assets(&self) -> LocalBoxFuture<'_, AssetGraphResult<AssetSyncReportDto>>;

    fn graph(&self) -> LocalBoxFuture<'_, AssetGraphResult<AssetGraphDto>>;
}

pub type SharedAssetGraphApi = Rc<dyn AssetGraphApi>;

pub fn default_asset_graph_api() -> SharedAssetGraphApi {
    #[cfg(target_arch = "wasm32")]
    {
        Rc::new(BrowserAssetGraphApi)
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        Rc::new(EmbeddedAssetGraphApi)
    }
}

#[cfg(target_arch = "wasm32")]
struct BrowserAssetGraphApi;

#[cfg(target_arch = "wasm32")]
impl AssetGraphApi for BrowserAssetGraphApi {
    fn sync_assets(&self) -> LocalBoxFuture<'_, AssetGraphResult<AssetSyncReportDto>> {
        Box::pin(async move {
            let payload = serde_json::json!({});
            super::browser_http::post_json("/api/admin/assets/sync", &payload)
                .await
                .map_err(AssetGraphError::new)
        })
    }

    fn graph(&self) -> LocalBoxFuture<'_, AssetGraphResult<AssetGraphDto>> {
        Box::pin(async move {
            super::browser_http::get_json("/api/admin/assets/graph")
                .await
                .map_err(AssetGraphError::new)
        })
    }
}

#[cfg(not(target_arch = "wasm32"))]
struct EmbeddedAssetGraphApi;

#[cfg(not(target_arch = "wasm32"))]
impl AssetGraphApi for EmbeddedAssetGraphApi {
    fn sync_assets(&self) -> LocalBoxFuture<'_, AssetGraphResult<AssetSyncReportDto>> {
        Box::pin(async move { sync_assets_on_server().await })
    }

    fn graph(&self) -> LocalBoxFuture<'_, AssetGraphResult<AssetGraphDto>> {
        Box::pin(async move { load_asset_graph_on_server().await })
    }
}

#[cfg(not(target_arch = "wasm32"))]
const ASSET_SCHEMA_SQL: &str = az_persistence::ADMIN_ASSET_GRAPH_SCHEMA_SQL;

#[cfg(not(target_arch = "wasm32"))]
#[apply(plain_clone)]
pub struct AssetRecordInput {
    pub id: String,
    pub kind: AssetKindDto,
    pub title: String,
    pub detail: String,
    pub source: String,
    pub local_path: Option<String>,
    pub relative_path: Option<String>,
    pub download_url: Option<String>,
    pub content_hash: Option<String>,
    pub hash_algorithm: Option<String>,
    pub size_bytes: Option<u64>,
    pub tags: Vec<String>,
    pub raw: serde_json::Value,
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn sync_assets_on_server() -> AssetGraphResult<AssetSyncReportDto> {
    let pool = pool().await?;
    ensure_asset_schema(&pool).await?;

    let mut report = AssetSyncReportDto::default();
    report.notes_imported = sync_blinko_notes(&pool, &mut report.warnings).await?;
    report.software_imported = sync_installed_software(&pool, &mut report.warnings).await?;
    report.packages_indexed = sync_package_inventory(&pool, &mut report.warnings).await?;
    report.tags_indexed = count_tags(&pool).await?;
    Ok(report)
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn load_asset_graph_on_server() -> AssetGraphResult<AssetGraphDto> {
    let pool = pool().await?;
    ensure_asset_schema(&pool).await?;
    read_asset_graph(&pool).await
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn upsert_asset_record_on_server(input: AssetRecordInput) -> AssetGraphResult<()> {
    let pool = pool().await?;
    ensure_asset_schema(&pool).await?;
    upsert_asset_record(&pool, input).await
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn existing_package_by_hash(
    hash_algorithm: &str,
    content_hash: &str,
) -> AssetGraphResult<Option<AssetGraphItemDto>> {
    let pool = pool().await?;
    ensure_asset_schema(&pool).await?;
    let row = sqlx::query(
        r#"
        SELECT
            i.id,
            i.kind,
            i.title,
            i.detail,
            i.source,
            i.local_path,
            i.relative_path,
            i.download_url,
            i.content_hash,
            i.hash_algorithm,
            i.size_bytes,
            i.updated_at,
            COALESCE(array_remove(array_agg(t.label ORDER BY t.label), NULL), ARRAY[]::text[]) AS tags
        FROM admin_asset_items i
        LEFT JOIN admin_asset_item_tags it ON it.item_id = i.id
        LEFT JOIN admin_asset_tags t ON t.id = it.tag_id
        WHERE i.kind = 'package'
          AND i.hash_algorithm = $1
          AND i.content_hash = $2
          AND i.relative_path IS NOT NULL
        GROUP BY i.id
        ORDER BY i.updated_at DESC
        LIMIT 1
        "#,
    )
    .bind(hash_algorithm)
    .bind(content_hash)
    .fetch_optional(&pool)
    .await
    .map_err(query_error)?;

    Ok(row.map(row_to_item))
}

#[cfg(not(target_arch = "wasm32"))]
static ASSET_GRAPH_POOL: OnceCell<sqlx::postgres::PgPool> = OnceCell::const_new();

#[cfg(not(target_arch = "wasm32"))]
async fn pool() -> AssetGraphResult<sqlx::postgres::PgPool> {
    Ok(ASSET_GRAPH_POOL
        .get_or_try_init(|| async {
            let database_url = az_knowledge::database_url().ok_or_else(|| {
                AssetGraphError::new(
                    "缺少 PostgreSQL 连接：请设置 MSC_AIO_DATABASE_URL，或在 ~/.config/aio/aio.env 中配置 MSC_AIO_DATABASE_URL / DATABASE_URL",
                )
            })?;
            sqlx::postgres::PgPoolOptions::new()
                .max_connections(4)
                .acquire_timeout(Duration::from_secs(5))
                .connect(&database_url)
                .await
                .map_err(|err| AssetGraphError::new(format!("连接 PostgreSQL 失败：{err}")))
        })
        .await?
        .clone())
}

#[cfg(not(target_arch = "wasm32"))]
async fn ensure_asset_schema(pool: &sqlx::postgres::PgPool) -> AssetGraphResult<()> {
    for statement in ASSET_SCHEMA_SQL.split(';') {
        let trimmed = statement.trim();
        if trimmed.is_empty() {
            continue;
        }
        sqlx::query(trimmed)
            .execute(pool)
            .await
            .map_err(|err| AssetGraphError::new(format!("初始化资产表失败：{err}")))?;
    }
    Ok(())
}

#[cfg(not(target_arch = "wasm32"))]
async fn sync_blinko_notes(
    pool: &sqlx::postgres::PgPool,
    warnings: &mut Vec<String>,
) -> AssetGraphResult<usize> {
    if !table_exists(pool, "public.notes").await? {
        warnings.push("未发现 Blinko notes 表，跳过笔记导入。".to_string());
        return Ok(0);
    }

    let rows = match sqlx::query(
        r#"
        SELECT
            n.id,
            n.content,
            n."updatedAt",
            COALESCE(array_remove(array_agg(t.name ORDER BY t.name), NULL), ARRAY[]::text[]) AS tags
        FROM notes n
        LEFT JOIN "tagsToNote" tn ON tn."noteId" = n.id
        LEFT JOIN tag t ON t.id = tn."tagId"
        WHERE n."isRecycle" = FALSE
        GROUP BY n.id
        ORDER BY n."updatedAt" DESC
        LIMIT 2000
        "#,
    )
    .fetch_all(pool)
    .await
    {
        Ok(rows) => rows,
        Err(err) => {
            warnings.push(format!("读取 Blinko 笔记失败：{err}"));
            return Ok(0);
        }
    };

    let mut imported = 0usize;
    for row in rows {
        let id = row_get_i32(&row, "id");
        let content = row_get_string(&row, "content");
        if content.trim().is_empty() {
            continue;
        }
        let updated_at = row
            .try_get::<DateTime<Utc>, _>("updatedAt")
            .ok()
            .map(|value| value.to_rfc3339());
        let mut tags = row.try_get::<Vec<String>, _>("tags").unwrap_or_default();
        tags.push("笔记".to_string());

        upsert_asset_record(
            pool,
            AssetRecordInput {
                id: format!("blinko-note-{id}"),
                kind: AssetKindDto::Note,
                title: first_content_line(&content),
                detail: truncate_chars(&content, 320),
                source: format!("Blinko PG · note #{id}"),
                local_path: None,
                relative_path: None,
                download_url: None,
                content_hash: Some(blake3_hex(content.as_bytes())),
                hash_algorithm: Some("blake3".to_string()),
                size_bytes: Some(content.len() as u64),
                tags,
                raw: serde_json::json!({
                    "external_id": id,
                    "updated_at": updated_at,
                }),
            },
        )
        .await?;
        imported += 1;
    }

    Ok(imported)
}

#[cfg(not(target_arch = "wasm32"))]
async fn sync_installed_software(
    pool: &sqlx::postgres::PgPool,
    warnings: &mut Vec<String>,
) -> AssetGraphResult<usize> {
    let apps = discover_installed_apps();
    if apps.is_empty() {
        warnings.push("未扫描到本机 .app 软件。".to_string());
    }

    let mut imported = 0usize;
    for app in apps {
        upsert_asset_record(
            pool,
            AssetRecordInput {
                id: format!("software-{}", slugify(&app.path)),
                kind: AssetKindDto::Software,
                title: app.name.clone(),
                detail: app.detail.clone(),
                source: "本机 /Applications 扫描".to_string(),
                local_path: Some(app.path.clone()),
                relative_path: None,
                download_url: None,
                content_hash: Some(blake3_hex(app.path.as_bytes())),
                hash_algorithm: Some("blake3-path".to_string()),
                size_bytes: None,
                tags: app.tags.clone(),
                raw: serde_json::json!({
                    "path": app.path,
                    "version": app.version,
                    "bundle_id": app.bundle_id,
                }),
            },
        )
        .await?;
        imported += 1;
    }

    Ok(imported)
}

#[cfg(not(target_arch = "wasm32"))]
async fn sync_package_inventory(
    pool: &sqlx::postgres::PgPool,
    warnings: &mut Vec<String>,
) -> AssetGraphResult<usize> {
    let removed = sqlx::query("DELETE FROM admin_asset_items WHERE id LIKE 'package-minio-%'")
        .execute(pool)
        .await
        .map_err(query_error)?
        .rows_affected();
    if removed > 0 {
        warnings.push(format!("已清理 {removed} 条旧对象存储安装包索引记录。"));
    }

    let mut packages = crate::package_catalog::PACKAGE_ASSETS
        .iter()
        .map(package_record_from_catalog)
        .collect::<Vec<_>>();
    packages.sort_by(|left, right| {
        left.title
            .cmp(&right.title)
            .then(left.relative_path.cmp(&right.relative_path))
    });
    if packages.is_empty() {
        warnings.push("静态 package catalog 里还没有可索引的安装包条目。".to_string());
    }

    let mut indexed = 0usize;
    for package in packages {
        upsert_asset_record(pool, package).await?;
        indexed += 1;
    }

    Ok(indexed)
}

#[cfg(not(target_arch = "wasm32"))]
async fn table_exists(pool: &sqlx::postgres::PgPool, name: &str) -> AssetGraphResult<bool> {
    let row = sqlx::query("SELECT to_regclass($1) IS NOT NULL AS exists")
        .bind(name)
        .fetch_one(pool)
        .await
        .map_err(query_error)?;
    row.try_get("exists").map_err(query_error)
}

#[cfg(not(target_arch = "wasm32"))]
async fn upsert_asset_record(
    pool: &sqlx::postgres::PgPool,
    input: AssetRecordInput,
) -> AssetGraphResult<()> {
    let size_bytes = input.size_bytes.and_then(|value| i64::try_from(value).ok());
    sqlx::query(
        r#"
        INSERT INTO admin_asset_items (
            id,
            kind,
            title,
            detail,
            source,
            local_path,
            relative_path,
            download_url,
            content_hash,
            hash_algorithm,
            size_bytes,
            raw,
            seen_at,
            updated_at
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, NOW(), NOW())
        ON CONFLICT (id) DO UPDATE SET
            kind = EXCLUDED.kind,
            title = EXCLUDED.title,
            detail = EXCLUDED.detail,
            source = EXCLUDED.source,
            local_path = COALESCE(EXCLUDED.local_path, admin_asset_items.local_path),
            relative_path = COALESCE(EXCLUDED.relative_path, admin_asset_items.relative_path),
            download_url = COALESCE(EXCLUDED.download_url, admin_asset_items.download_url),
            content_hash = COALESCE(EXCLUDED.content_hash, admin_asset_items.content_hash),
            hash_algorithm = COALESCE(EXCLUDED.hash_algorithm, admin_asset_items.hash_algorithm),
            size_bytes = COALESCE(EXCLUDED.size_bytes, admin_asset_items.size_bytes),
            raw = EXCLUDED.raw,
            seen_at = EXCLUDED.seen_at,
            updated_at = EXCLUDED.updated_at
        "#,
    )
    .bind(&input.id)
    .bind(input.kind.as_str())
    .bind(input.title)
    .bind(input.detail)
    .bind(input.source)
    .bind(input.local_path)
    .bind(input.relative_path)
    .bind(input.download_url)
    .bind(input.content_hash)
    .bind(input.hash_algorithm)
    .bind(size_bytes)
    .bind(input.raw)
    .execute(pool)
    .await
    .map_err(query_error)?;

    sqlx::query("DELETE FROM admin_asset_item_tags WHERE item_id = $1")
        .bind(&input.id)
        .execute(pool)
        .await
        .map_err(query_error)?;

    let tags = normalized_tags(input.kind, &input.tags);
    for tag in tags {
        let tag_id = format!("tag-{}", slugify(&tag));
        sqlx::query(
            r#"
            INSERT INTO admin_asset_tags (id, label, updated_at)
            VALUES ($1, $2, NOW())
            ON CONFLICT (label) DO UPDATE SET
                updated_at = EXCLUDED.updated_at
            "#,
        )
        .bind(&tag_id)
        .bind(&tag)
        .execute(pool)
        .await
        .map_err(query_error)?;

        sqlx::query(
            r#"
            INSERT INTO admin_asset_item_tags (item_id, tag_id)
            VALUES ($1, $2)
            ON CONFLICT DO NOTHING
            "#,
        )
        .bind(&input.id)
        .bind(tag_id)
        .execute(pool)
        .await
        .map_err(query_error)?;
    }

    Ok(())
}

#[cfg(not(target_arch = "wasm32"))]
async fn read_asset_graph(pool: &sqlx::postgres::PgPool) -> AssetGraphResult<AssetGraphDto> {
    let rows = sqlx::query(
        r#"
        SELECT
            i.id,
            i.kind,
            i.title,
            i.detail,
            i.source,
            i.local_path,
            i.relative_path,
            i.download_url,
            i.content_hash,
            i.hash_algorithm,
            i.size_bytes,
            i.updated_at,
            COALESCE(array_remove(array_agg(t.label ORDER BY t.label), NULL), ARRAY[]::text[]) AS tags
        FROM admin_asset_items i
        LEFT JOIN admin_asset_item_tags it ON it.item_id = i.id
        LEFT JOIN admin_asset_tags t ON t.id = it.tag_id
        GROUP BY i.id
        ORDER BY i.kind, i.updated_at DESC, i.title
        "#,
    )
    .fetch_all(pool)
    .await
    .map_err(query_error)?;

    let items = rows.into_iter().map(row_to_item).collect::<Vec<_>>();
    let mut counts = BTreeMap::<String, usize>::new();
    for item in &items {
        for tag in &item.tags {
            *counts.entry(tag.clone()).or_default() += 1;
        }
    }
    let tags = counts
        .into_iter()
        .map(|(label, item_count)| AssetGraphTagDto {
            id: format!("tag-{}", slugify(&label)),
            label,
            item_count,
        })
        .collect::<Vec<_>>();
    let edges = build_graph_edges(&items);

    Ok(AssetGraphDto {
        note_count: items.iter().filter(|item| item.kind == "note").count(),
        software_count: items.iter().filter(|item| item.kind == "software").count(),
        package_count: items.iter().filter(|item| item.kind == "package").count(),
        items,
        tags,
        edges,
        warnings: Vec::new(),
    })
}

#[cfg(not(target_arch = "wasm32"))]
async fn count_tags(pool: &sqlx::postgres::PgPool) -> AssetGraphResult<usize> {
    let row = sqlx::query("SELECT COUNT(*)::BIGINT AS count FROM admin_asset_tags")
        .fetch_one(pool)
        .await
        .map_err(query_error)?;
    let count = row.try_get::<i64, _>("count").unwrap_or_default();
    Ok(usize::try_from(count).unwrap_or_default())
}

#[cfg(not(target_arch = "wasm32"))]
fn row_to_item(row: sqlx::postgres::PgRow) -> AssetGraphItemDto {
    let size_bytes = row
        .try_get::<Option<i64>, _>("size_bytes")
        .ok()
        .flatten()
        .and_then(|value| u64::try_from(value).ok());
    let updated_at = row
        .try_get::<DateTime<Utc>, _>("updated_at")
        .ok()
        .map(|value| value.to_rfc3339());

    AssetGraphItemDto {
        id: row_get_string(&row, "id"),
        kind: row_get_string(&row, "kind"),
        title: row_get_string(&row, "title"),
        detail: row_get_string(&row, "detail"),
        source: row_get_string(&row, "source"),
        local_path: row.try_get("local_path").ok().flatten(),
        relative_path: row.try_get("relative_path").ok().flatten(),
        download_url: row.try_get("download_url").ok().flatten(),
        content_hash: row.try_get("content_hash").ok().flatten(),
        hash_algorithm: row.try_get("hash_algorithm").ok().flatten(),
        size_bytes,
        tags: row.try_get("tags").unwrap_or_default(),
        updated_at,
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn build_graph_edges(items: &[AssetGraphItemDto]) -> Vec<AssetGraphEdgeDto> {
    let mut seen = BTreeSet::new();
    let mut edges = Vec::new();
    for item in items {
        for tag in &item.tags {
            let tag_id = format!("tag-{}", slugify(tag));
            let key = format!("{tag_id}:{}", item.id);
            if seen.insert(key) {
                edges.push(AssetGraphEdgeDto {
                    source: tag_id,
                    target: item.id.clone(),
                    relation: "tagged".to_string(),
                });
            }
        }
    }
    edges
}

#[cfg(not(target_arch = "wasm32"))]
#[apply(plain_clone_debug)]
struct InstalledApp {
    name: String,
    path: String,
    version: Option<String>,
    bundle_id: Option<String>,
    detail: String,
    tags: Vec<String>,
}

#[cfg(not(target_arch = "wasm32"))]
fn discover_installed_apps() -> Vec<InstalledApp> {
    let mut apps = Vec::new();
    let mut seen = BTreeSet::new();
    for root in application_roots() {
        let Ok(entries) = std::fs::read_dir(&root) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|value| value.to_str()) != Some("app") {
                continue;
            }
            let path_string = path.display().to_string();
            if !seen.insert(path_string.clone()) {
                continue;
            }
            let name = path
                .file_stem()
                .map(|value| value.to_string_lossy().to_string())
                .unwrap_or_else(|| "Unknown App".to_string());
            let (version, bundle_id) = read_app_info(&path);
            let detail = match (&version, &bundle_id) {
                (Some(version), Some(bundle_id)) => format!("{bundle_id} · {version}"),
                (Some(version), None) => format!("version {version}"),
                (None, Some(bundle_id)) => bundle_id.clone(),
                (None, None) => path_string.clone(),
            };
            let mut tags = vec![
                "软件".to_string(),
                "macOS".to_string(),
                root.file_name()
                    .map(|value| value.to_string_lossy().to_string())
                    .unwrap_or_else(|| "Applications".to_string()),
            ];
            if let Some(category) = software_category(&name) {
                tags.push(category.to_string());
            }
            apps.push(InstalledApp {
                name,
                path: path_string,
                version,
                bundle_id,
                detail,
                tags,
            });
        }
    }
    apps.sort_by(|left, right| left.name.cmp(&right.name));
    apps
}

#[cfg(not(target_arch = "wasm32"))]
fn application_roots() -> Vec<PathBuf> {
    let mut roots = vec![
        PathBuf::from("/Applications"),
        PathBuf::from("/System/Applications"),
    ];
    if let Ok(home) = std::env::var("HOME") {
        roots.push(Path::new(&home).join("Applications"));
    }
    roots.into_iter().filter(|path| path.exists()).collect()
}

#[cfg(not(target_arch = "wasm32"))]
fn read_app_info(path: &Path) -> (Option<String>, Option<String>) {
    let plist = path.join("Contents/Info.plist");
    let Ok(content) = std::fs::read_to_string(plist) else {
        return (None, None);
    };
    (
        plist_value(&content, "CFBundleShortVersionString")
            .or_else(|| plist_value(&content, "CFBundleVersion")),
        plist_value(&content, "CFBundleIdentifier"),
    )
}

#[cfg(not(target_arch = "wasm32"))]
fn plist_value(content: &str, key: &str) -> Option<String> {
    let key_marker = format!("<key>{key}</key>");
    let (_, tail) = content.split_once(&key_marker)?;
    let (_, value_tail) = tail.split_once("<string>")?;
    let (value, _) = value_tail.split_once("</string>")?;
    let trimmed = value.trim();
    (!trimmed.is_empty()).then(|| trimmed.to_string())
}

#[cfg(not(target_arch = "wasm32"))]
fn software_category(name: &str) -> Option<&'static str> {
    let lower = name.to_ascii_lowercase();
    if ["cursor", "code", "zed", "rustrover", "intellij", "xcode"]
        .iter()
        .any(|needle| lower.contains(needle))
    {
        Some("开发工具")
    } else if ["chrome", "safari", "firefox", "edge"]
        .iter()
        .any(|needle| lower.contains(needle))
    {
        Some("浏览器")
    } else if ["docker", "postgres", "redis"]
        .iter()
        .any(|needle| lower.contains(needle))
    {
        Some("基础设施")
    } else if ["obsidian", "notion", "wps", "office"]
        .iter()
        .any(|needle| lower.contains(needle))
    {
        Some("知识与文档")
    } else {
        None
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn package_record_from_catalog(asset: &crate::package_catalog::PackageAsset) -> AssetRecordInput {
    let channel = crate::package_catalog::package_channel(asset.channel_slug);
    let detail = format!(
        "{} · {} · {}",
        asset.platform, asset.version, asset.install_target
    );
    AssetRecordInput {
        id: format!("package-catalog-{}", asset.slug),
        kind: AssetKindDto::Package,
        title: asset.package_name.to_string(),
        detail,
        source: asset.source.to_string(),
        local_path: None,
        relative_path: None,
        download_url: None,
        content_hash: Some(blake3_hex(
            format!("{}:{}:{}", asset.slug, asset.version, asset.package_name).as_bytes(),
        )),
        hash_algorithm: Some("blake3-catalog".to_string()),
        size_bytes: None,
        tags: vec![
            "安装包".to_string(),
            asset.format.to_string(),
            asset.platform.to_string(),
            asset.status.to_string(),
            channel
                .map(|value| value.title.to_string())
                .unwrap_or_else(|| asset.channel_slug.to_string()),
        ],
        raw: serde_json::json!({
            "slug": asset.slug,
            "software_title": asset.software_title,
            "channel_slug": asset.channel_slug,
            "version": asset.version,
            "platform": asset.platform,
            "format": asset.format,
            "status": asset.status,
            "install_target": asset.install_target,
            "checksum_state": asset.checksum_state,
            "relation": asset.relation,
            "note": asset.note,
        }),
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub fn blake3_file_hex(path: &Path) -> std::io::Result<String> {
    let mut hasher = blake3::Hasher::new();
    let mut file = std::fs::File::open(path)?;
    std::io::copy(&mut file, &mut hasher)?;
    Ok(hasher.finalize().to_hex().to_string())
}

#[cfg(not(target_arch = "wasm32"))]
fn blake3_hex(bytes: &[u8]) -> String {
    blake3::hash(bytes).to_hex().to_string()
}

fn normalized_tags(kind: AssetKindDto, raw: &[String]) -> Vec<String> {
    let mut tags = BTreeSet::new();
    tags.insert(kind.to_string());
    for tag in raw {
        let cleaned = normalize_tag_label(tag);
        if !cleaned.is_empty() {
            tags.insert(cleaned);
        }
    }
    tags.into_iter().collect()
}

#[cfg(not(target_arch = "wasm32"))]
fn normalize_tag_label(raw: &str) -> String {
    raw.trim()
        .trim_start_matches('#')
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(not(target_arch = "wasm32"))]
fn first_content_line(content: &str) -> String {
    content
        .lines()
        .map(str::trim)
        .find(|line| !line.is_empty())
        .map(|line| truncate_chars(line, 88))
        .unwrap_or_else(|| "未命名笔记".to_string())
}

#[cfg(not(target_arch = "wasm32"))]
fn truncate_chars(text: &str, limit: usize) -> String {
    let mut result = String::new();
    for (index, ch) in text.chars().enumerate() {
        if index == limit {
            result.push('…');
            break;
        }
        result.push(ch);
    }
    result
}

fn slugify(value: &str) -> String {
    let mut slug = String::new();
    let mut last_dash = false;
    for ch in value.chars() {
        let lowered = ch.to_ascii_lowercase();
        if lowered.is_ascii_alphanumeric() {
            slug.push(lowered);
            last_dash = false;
        } else if !last_dash {
            slug.push('-');
            last_dash = true;
        }
    }
    let trimmed = slug.trim_matches('-');
    if trimmed.is_empty() {
        blake3_hex(value.as_bytes())[..12].to_string()
    } else {
        trimmed.to_string()
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn row_get_string(row: &sqlx::postgres::PgRow, column: &str) -> String {
    row.try_get(column).unwrap_or_default()
}

#[cfg(not(target_arch = "wasm32"))]
fn row_get_i32(row: &sqlx::postgres::PgRow, column: &str) -> i32 {
    row.try_get(column).unwrap_or_default()
}

#[cfg(not(target_arch = "wasm32"))]
fn query_error(err: sqlx::Error) -> AssetGraphError {
    AssetGraphError::new(format!("资产数据查询失败：{err}"))
}

#[cfg(test)]
mod tests {
    use super::{AssetKindDto, package_record_from_catalog};

    #[test]
    fn asset_kind_code_helpers_follow_wire_values() {
        assert_eq!(AssetKindDto::Package.as_str(), "package");
        assert_eq!(AssetKindDto::Software.code(), "software");
        assert_eq!(AssetKindDto::from_code("note"), Some(AssetKindDto::Note));
        assert_eq!(AssetKindDto::Note.to_string(), "笔记");
    }

    #[test]
    fn package_record_from_catalog_should_use_static_catalog_source() {
        let asset = crate::package_catalog::package_asset("cursor-macos")
            .expect("catalog asset should exist");
        let record = package_record_from_catalog(asset);

        assert_eq!(record.id, "package-catalog-cursor-macos");
        assert_eq!(record.relative_path, None);
        assert_eq!(record.download_url, None);
        assert!(record.tags.iter().any(|tag| tag == "安装包"));
    }
}

#[cfg(not(target_arch = "wasm32"))]
use sqlx::Row;
