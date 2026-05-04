use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::UNIX_EPOCH;

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, Row, postgres::PgRow};
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::services::LocalBoxFuture;

/// 文件分类映射
const CATEGORIES: &[(&str, &[&str])] = &[
    ("🎬 视频", &["mp4", "mkv", "avi", "mov", "wmv", "flv", "webm", "m4v", "ts", "rmvb", "3gp"]),
    ("🎵 音频", &["mp3", "flac", "wav", "aac", "ogg", "wma", "m4a", "ape", "opus", "mid", "midi"]),
    ("🖼️ 图片", &["jpg", "jpeg", "png", "gif", "bmp", "svg", "webp", "ico", "tiff", "tif", "psd", "raw", "heic", "heif", "avif"]),
    ("📄 文档", &["pdf", "doc", "docx", "xls", "xlsx", "ppt", "pptx", "txt", "md", "csv", "rtf", "odt", "ods", "odp", "epub", "mobi", "pages", "numbers", "key"]),
    ("📦 压缩包", &["zip", "rar", "7z", "tar", "gz", "bz2", "xz", "tgz", "tbz2", "txz", "zst", "lz4", "iso", "dmg"]),
    ("💻 安装包", &["exe", "msi", "apk", "appimage", "deb", "rpm", "pkg", "snap", "flatpak"]),
    ("🔧 代码/数据", &["py", "js", "ts", "java", "c", "cpp", "go", "rs", "rb", "php", "swift", "kt", "sh", "bash", "zsh", "json", "yaml", "yml", "toml", "xml", "html", "css", "sql", "db", "sqlite"]),
    ("🔑 密钥/证书", &["pem", "key", "crt", "cer", "p12", "pfx", "jks", "keystore"]),
];

/// 获取文件分类
fn get_category(ext: &str) -> &'static str {
    let ext_lower = ext.to_lowercase();
    for (category, extensions) in CATEGORIES {
        if extensions.contains(&ext_lower.as_str()) {
            return category;
        }
    }
    "📋 其他"
}

/// 格式化文件大小
fn format_size(size: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    let mut size = size as f64;
    for &unit in UNITS {
        if size < 1024.0 {
            return if unit == "B" {
                format!("{} {}", size as u64, unit)
            } else {
                format!("{:.1} {}", size, unit)
            };
        }
        size /= 1024.0;
    }
    format!("{:.1} PB", size)
}

/// 文件索引项
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileIndex {
    pub id: i64,
    pub source: String,
    pub path: String,
    pub full_path: String,
    pub name: String,
    pub dir: Option<String>,
    pub size: i64,
    pub size_h: String,
    pub ext: Option<String>,
    pub category: String,
    pub mtime: DateTime<Utc>,
    pub mtime_h: String,
}

/// 分享链接
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShareLink {
    pub token: String,
    pub source: String,
    pub path: String,
    pub file_name: String,
    pub expires_at: DateTime<Utc>,
    pub expires_hours: i64,
    pub url: String,
}

/// 扫描统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanStats {
    pub total_files: i64,
    pub source_counts: HashMap<String, i64>,
    pub category_counts: HashMap<String, i64>,
}

/// 筛选条件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilterOptions {
    pub source: Option<String>,
    pub category: Option<String>,
    pub query: Option<String>,
    pub offset: i64,
    pub limit: i64,
}

/// Download Station 服务
pub struct DownloadStationService {
    pool: PgPool,
    cache: RwLock<HashMap<String, Vec<FileIndex>>>,
    cache_time: RwLock<HashMap<String, i64>>,
}

impl DownloadStationService {
    pub async fn new(pool: PgPool) -> Result<Self> {
        Ok(Self {
            pool,
            cache: RwLock::new(HashMap::new()),
            cache_time: RwLock::new(HashMap::new()),
        })
    }

    /// 扫描目录并建立索引
    pub async fn scan_directories(&self, directories: Vec<String>) -> Result<ScanStats> {
        let mut all_files = Vec::new();
        let mut source_counts: HashMap<String, i64> = HashMap::new();
        let mut category_counts: HashMap<String, i64> = HashMap::new();

        for dir_str in directories {
            let dir_path = PathBuf::from(dir_str.replace("~", &std::env::var("HOME").unwrap_or_default()))
                .canonicalize()
                .context(format!("Failed to resolve directory: {}", dir_str))?;

            if !dir_path.is_dir() {
                log::warn!("Directory does not exist: {:?}", dir_path);
                continue;
            }

            let source_name = dir_path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("unknown")
                .to_string();

            let files = self.scan_directory(&dir_path, &source_name).await?;
            let count = files.len() as i64;
            source_counts.insert(source_name.clone(), count);

            for file in &files {
                *category_counts.entry(file.category.clone()).or_insert(0) += 1;
            }

            all_files.extend(files);
        }

        // 保存到数据库
        self.save_to_database(&all_files).await?;

        // 更新缓存
        let mut cache = self.cache.write().await;
        let mut cache_time = self.cache_time.write().await;
        let now = Utc::now().timestamp();

        // 按来源分组缓存
        for file in &all_files {
            cache.entry(file.source.clone())
                .or_insert_with(Vec::new)
                .push(file.clone());
        }
        cache_time.insert("all".to_string(), now);

        Ok(ScanStats {
            total_files: all_files.len() as i64,
            source_counts,
            category_counts,
        })
    }

    /// 扫描单个目录
    async fn scan_directory(&self, root: &Path, source_name: &str) -> Result<Vec<FileIndex>> {
        let mut files = Vec::new();

        let entries = walkdir::WalkDir::new(root)
            .follow_links(true)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file());

        for entry in entries {
            let path = entry.path();
            if path.file_name()
                .and_then(|n| n.to_str())
                .map(|n| n.starts_with('.'))
                .unwrap_or(false)
            {
                continue;
            }

            let metadata = match std::fs::metadata(path) {
                Ok(m) => m,
                Err(_) => continue,
            };

            let rel_path = match path.strip_prefix(root) {
                Ok(p) => p,
                Err(_) => continue,
            };

            let name = path.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("unknown")
                .to_string();

            let ext = path.extension()
                .and_then(|e| e.to_str())
                .map(|e| e.to_lowercase());

            let category = ext.as_deref().map(get_category).unwrap_or("📋 其他");

            let mtime = metadata.modified()
                .ok()
                .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
                .map(|d| DateTime::<Utc>::from(UNIX_EPOCH + d))
                .unwrap_or_else(Utc::now);

            let dir = rel_path.parent()
                .and_then(|p| p.to_str())
                .filter(|s| !s.is_empty())
                .map(|s| s.to_string());

            files.push(FileIndex {
                id: 0, // 数据库生成
                source: source_name.to_string(),
                path: rel_path.to_string_lossy().to_string(),
                full_path: path.to_string_lossy().to_string(),
                name,
                dir,
                size: metadata.len() as i64,
                size_h: format_size(metadata.len()),
                ext,
                category: category.to_string(),
                mtime,
                mtime_h: mtime.format("%Y-%m-%d %H:%M").to_string(),
            });
        }

        Ok(files)
    }

    /// 保存到数据库
    async fn save_to_database(&self, files: &[FileIndex]) -> Result<()> {
        // 先删除旧的索引
        sqlx::query("DELETE FROM download_station_files")
            .execute(&self.pool)
            .await?;

        // 批量插入
        for file in files {
            sqlx::query(
                r#"
                INSERT INTO download_station_files 
                (source, path, full_path, name, dir, size, ext, category, mtime)
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
                "#,
            )
            .bind(&file.source)
            .bind(&file.path)
            .bind(&file.full_path)
            .bind(&file.name)
            .bind(&file.dir)
            .bind(file.size)
            .bind(&file.ext)
            .bind(&file.category)
            .bind(file.mtime)
            .execute(&self.pool)
            .await?;
        }

        Ok(())
    }

    /// 获取文件列表
    pub async fn list_files(&self, filter: FilterOptions) -> Result<Vec<FileIndex>> {
        let mut query = String::from(
            "SELECT * FROM download_station_files WHERE 1=1",
        );
        let mut bind_count = 0;

        if let Some(source) = &filter.source {
            if source != "全部" {
                bind_count += 1;
                query.push_str(&format!(" AND source = ${}", bind_count));
            }
        }

        if let Some(category) = &filter.category {
            if category != "全部" {
                bind_count += 1;
                query.push_str(&format!(" AND category = ${}", bind_count));
            }
        }

        if let Some(query_str) = &filter.query {
            if !query_str.is_empty() {
                bind_count += 1;
                query.push_str(&format!(" AND (name ILIKE ${}", bind_count));
                bind_count += 1;
                query.push_str(&format!(" OR path ILIKE ${}", bind_count));
                bind_count += 1;
                query.push_str(&format!(" OR ext ILIKE ${})", bind_count));
            }
        }

        bind_count += 1;
        query.push_str(&format!(" ORDER BY mtime DESC LIMIT ${}", bind_count));
        bind_count += 1;
        query.push_str(&format!(" OFFSET ${}", bind_count));

        let pattern = filter.query.as_ref().filter(|q| !q.is_empty()).map(|q| format!("%{}%", q));
        let mut q = sqlx::query_as::<_, FileIndex>(&query);

        if let Some(source) = &filter.source {
            if source != "全部" {
                q = q.bind(source);
            }
        }

        if let Some(category) = &filter.category {
            if category != "全部" {
                q = q.bind(category);
            }
        }

        if let Some(ref pat) = pattern {
            q = q.bind(pat).bind(pat).bind(pat);
        }

        q = q.bind(filter.limit).bind(filter.offset);

        let files = q.fetch_all(&self.pool).await?;
        Ok(files)
    }

    /// 获取统计信息
    pub async fn get_stats(&self) -> Result<ScanStats> {
        let total_files: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM download_station_files")
            .fetch_one(&self.pool)
            .await?;

        let source_rows = sqlx::query("SELECT source, COUNT(*) as count FROM download_station_files GROUP BY source")
            .fetch_all(&self.pool)
            .await?;

        let mut source_counts: HashMap<String, i64> = HashMap::new();
        for row in source_rows {
            let source: String = row.get("source");
            let count: i64 = row.get("count");
            source_counts.insert(source, count);
        }

        let category_rows = sqlx::query("SELECT category, COUNT(*) as count FROM download_station_files GROUP BY category")
            .fetch_all(&self.pool)
            .await?;

        let mut category_counts: HashMap<String, i64> = HashMap::new();
        for row in category_rows {
            let category: String = row.get("category");
            let count: i64 = row.get("count");
            category_counts.insert(category, count);
        }

        Ok(ScanStats {
            total_files,
            source_counts,
            category_counts,
        })
    }

    /// 创建分享链接
    pub async fn create_share(
        &self,
        source: &str,
        path: &str,
        file_name: &str,
        hours: i64,
        created_by: Option<&str>,
    ) -> Result<ShareLink> {
        let token = Uuid::new_v4().to_string()[..16].to_string();
        let expires_at = Utc::now() + chrono::Duration::hours(hours);

        sqlx::query(
            r#"
            INSERT INTO download_station_shares (token, source, path, file_name, expires_at, created_by)
            VALUES ($1, $2, $3, $4, $5, $6)
            "#,
        )
        .bind(&token)
        .bind(source)
        .bind(path)
        .bind(file_name)
        .bind(expires_at)
        .bind(created_by)
        .execute(&self.pool)
        .await?;

        let url = format!("/api/admin/download-station/share/{}", token);

        Ok(ShareLink {
            token,
            source: source.to_string(),
            path: path.to_string(),
            file_name: file_name.to_string(),
            expires_at,
            expires_hours: hours,
            url,
        })
    }

    /// 获取分享链接信息
    pub async fn get_share(&self, token: &str) -> Result<Option<ShareLink>> {
        let row = sqlx::query(
            "SELECT * FROM download_station_shares WHERE token = $1 AND expires_at > NOW()"
        )
        .bind(token)
        .fetch_optional(&self.pool)
        .await?;

        if let Some(row) = row {
            let source: String = row.get("source");
            let path: String = row.get("path");
            let file_name: String = row.get("file_name");
            let expires_at: DateTime<Utc> = row.get("expires_at");

            // 计算剩余小时数
            let expires_hours = (expires_at - Utc::now()).num_hours().max(1);

            Ok(Some(ShareLink {
                token: token.to_string(),
                source,
                path,
                file_name,
                expires_at,
                expires_hours,
                url: format!("/api/admin/download-station/share/{}", token),
            }))
        } else {
            Ok(None)
        }
    }

    /// 获取配置
    pub async fn get_config(&self, key: &str) -> Result<Option<String>> {
        let value = sqlx::query_scalar("SELECT value FROM download_station_config WHERE key = $1")
            .bind(key)
            .fetch_optional(&self.pool)
            .await?;
        Ok(value)
    }

    /// 设置配置
    pub async fn set_config(&self, key: &str, value: &str) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO download_station_config (key, value, updated_at)
            VALUES ($1, $2, NOW())
            ON CONFLICT (key) DO UPDATE SET value = $2, updated_at = NOW()
            "#,
        )
        .bind(key)
        .bind(value)
        .execute(&self.pool)
        .await?;
        Ok(())
    }
}

/// API DTO
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileIndexDto {
    pub id: i64,
    pub source: String,
    pub path: String,
    pub full_path: String,
    pub name: String,
    pub dir: Option<String>,
    pub size: i64,
    pub size_h: String,
    pub ext: Option<String>,
    pub category: String,
    pub mtime: DateTime<Utc>,
    pub mtime_h: String,
}

impl From<FileIndex> for FileIndexDto {
    fn from(f: FileIndex) -> Self {
        FileIndexDto {
            id: f.id,
            source: f.source,
            path: f.path,
            full_path: f.full_path,
            name: f.name,
            dir: f.dir,
            size: f.size,
            size_h: f.size_h,
            ext: f.ext,
            category: f.category,
            mtime: f.mtime,
            mtime_h: f.mtime_h,
        }
    }
}

impl sqlx::FromRow<'_, PgRow> for FileIndex {
    fn from_row(row: &PgRow) -> sqlx::Result<Self> {
        Ok(FileIndex {
            id: row.try_get("id")?,
            source: row.try_get("source")?,
            path: row.try_get("path")?,
            full_path: row.try_get("full_path")?,
            name: row.try_get("name")?,
            dir: row.try_get("dir")?,
            size: row.try_get("size")?,
            size_h: row.try_get("size_h")?,
            ext: row.try_get("ext")?,
            category: row.try_get("category")?,
            mtime: row.try_get("mtime")?,
            mtime_h: row.try_get("mtime_h")?,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShareLinkDto {
    pub token: String,
    pub source: String,
    pub path: String,
    pub file_name: String,
    pub expires_at: DateTime<Utc>,
    pub expires_hours: i64,
    pub url: String,
}

impl From<ShareLink> for ShareLinkDto {
    fn from(s: ShareLink) -> Self {
        ShareLinkDto {
            token: s.token,
            source: s.source,
            path: s.path,
            file_name: s.file_name,
            expires_at: s.expires_at,
            expires_hours: s.expires_hours,
            url: s.url,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanStatsDto {
    pub total_files: i64,
    pub source_counts: HashMap<String, i64>,
    pub category_counts: HashMap<String, i64>,
}

impl From<ScanStats> for ScanStatsDto {
    fn from(s: ScanStats) -> Self {
        ScanStatsDto {
            total_files: s.total_files,
            source_counts: s.source_counts,
            category_counts: s.category_counts,
        }
    }
}

/// Shared API trait
pub trait SharedDownloadStationApi: Send + Sync {
    fn scan_directories(&self, directories: Vec<String>) -> LocalBoxFuture<Result<ScanStatsDto>>;
    fn list_files(&self, filter: FilterOptions) -> LocalBoxFuture<Result<Vec<FileIndexDto>>>;
    fn get_stats(&self) -> LocalBoxFuture<Result<ScanStatsDto>>;
    fn create_share(
        &self,
        source: String,
        path: String,
        file_name: String,
        hours: i64,
        created_by: Option<String>,
    ) -> LocalBoxFuture<Result<ShareLinkDto>>;
    fn get_share(&self, token: String) -> LocalBoxFuture<Result<Option<ShareLinkDto>>>;
}

impl SharedDownloadStationApi for DownloadStationService {
    fn scan_directories(&self, directories: Vec<String>) -> LocalBoxFuture<Result<ScanStatsDto>> {
        let service = self.clone();
        Box::pin(async move {
            let stats = service.scan_directories(directories).await?;
            Ok(stats.into())
        })
    }

    fn list_files(&self, filter: FilterOptions) -> LocalBoxFuture<Result<Vec<FileIndexDto>>> {
        let service = self.clone();
        Box::pin(async move {
            let files = service.list_files(filter).await?;
            Ok(files.into_iter().map(FileIndexDto::from).collect())
        })
    }

    fn get_stats(&self) -> LocalBoxFuture<Result<ScanStatsDto>> {
        let service = self.clone();
        Box::pin(async move {
            let stats = service.get_stats().await?;
            Ok(stats.into())
        })
    }

    fn create_share(
        &self,
        source: String,
        path: String,
        file_name: String,
        hours: i64,
        created_by: Option<String>,
    ) -> LocalBoxFuture<Result<ShareLinkDto>> {
        let service = self.clone();
        Box::pin(async move {
            let share = service
                .create_share(&source, &path, &file_name, hours, created_by.as_deref())
                .await?;
            Ok(share.into())
        })
    }

    fn get_share(&self, token: String) -> LocalBoxFuture<Result<Option<ShareLinkDto>>> {
        let service = self.clone();
        Box::pin(async move {
            let share = service.get_share(&token).await?;
            Ok(share.map(ShareLinkDto::from))
        })
    }
}

impl Clone for DownloadStationService {
    fn clone(&self) -> Self {
        Self {
            pool: self.pool.clone(),
            cache: RwLock::new(HashMap::new()),
            cache_time: RwLock::new(HashMap::new()),
        }
    }
}

/// Default API instance
pub fn default_download_station_api(pool: PgPool) -> DownloadStationService {
    DownloadStationService {
        pool,
        cache: RwLock::new(HashMap::new()),
        cache_time: RwLock::new(HashMap::new()),
    }
}
