use std::{fs, path::{Path, PathBuf}, time::SystemTime};

use chrono::{DateTime, Duration, Local};
use dioxus::prelude::*;
use dioxus_components::{
    ContentHeader, DataTable, GroupedListPanel, GroupedListPanelGroup, GroupedListPanelItem,
    MetricStrip, StatTile, Surface, SurfaceHeader,
};

#[derive(Clone, PartialEq)]
struct FileRecord {
    scope: String,
    name: String,
    location: String,
    kind: String,
    status: String,
    note: String,
    modified_label: String,
}

#[derive(Clone, PartialEq)]
struct FileScopeSummary {
    key: &'static str,
    title: &'static str,
    eyebrow: &'static str,
    preview: &'static str,
    meta: [&'static str; 2],
}

const FILE_SCOPE_SUMMARIES: [FileScopeSummary; 4] = [
    FileScopeSummary { key: "yesterday-research", title: "昨天研究成果", eyebrow: "Focus", preview: "聚合昨天修改过的桌面研究产物与 recent outputs。", meta: ["按时间聚合", "桌面优先"] },
    FileScopeSummary { key: "recent-outputs", title: "Recent Outputs", eyebrow: "Cron", preview: "查看自动任务、批处理和生成结果文件。", meta: ["输出文件", "统一回看"] },
    FileScopeSummary { key: "desktop", title: "Desktop", eyebrow: "Local", preview: "浏览桌面上的文档、截图、压缩包和研究产物。", meta: ["桌面", "人工整理入口"] },
    FileScopeSummary { key: "downloads", title: "Downloads / 归档", eyebrow: "Local", preview: "浏览安装包、压缩包和待整理素材。", meta: ["本地文件", "安装包文件本体"] },
];

#[component]
pub fn FilesScene() -> Element {
    let selected_scope = use_signal(|| "downloads".to_string());
    let scope = selected_scope.read().clone();
    let records_resource = use_resource(load_file_records);

    let records = match records_resource.read().as_ref() {
        Some(Ok(records)) => records.clone(),
        Some(Err(err)) => {
            return rsx! {
                ContentHeader { title: "文件中心".to_string(), subtitle: "统一承接 recent outputs、桌面研究产物、下载记录与对象存储浏览。".to_string() }
                div { class: "callout", "文件清单加载失败：{err}" }
            };
        }
        None => {
            return rsx! {
                ContentHeader { title: "文件中心".to_string(), subtitle: "统一承接 recent outputs、桌面研究产物、下载记录与对象存储浏览。".to_string() }
                div { class: "empty-state", "正在扫描桌面、Downloads 与 recent outputs…" }
            };
        }
    };

    let rows: Vec<FileRecord> = records.iter().filter(|item| item.scope == scope).cloned().collect();
    let downloads_count = records.iter().filter(|item| item.scope == "downloads").count();

    rsx! {
        ContentHeader { title: "Download Station".to_string(), subtitle: "Rust 化的本地下载站，先承接 Downloads 与安装包归档入口。".to_string() }
        MetricStrip { columns: 4,
            StatTile { label: "Downloads".to_string(), value: downloads_count.to_string(), detail: "本地下载文件与安装包。".to_string() }
            StatTile { label: "桌面".to_string(), value: records.iter().filter(|item| item.scope == "desktop").count().to_string(), detail: "桌面研究产物。".to_string() }
            StatTile { label: "recent outputs".to_string(), value: records.iter().filter(|item| item.scope == "recent-outputs").count().to_string(), detail: "自动任务输出。".to_string() }
            StatTile { label: "昨天研究成果".to_string(), value: records.iter().filter(|item| item.scope == "yesterday-research").count().to_string(), detail: "昨日更新的研究文件。".to_string() }
        }
        div { class: "knowledge-board",
            GroupedListPanel { title: "下载域".to_string(), subtitle: "先看 Downloads，再看桌面/输出的关联文件。".to_string(), groups: vec![GroupedListPanelGroup { label: "统一入口".to_string(), count_label: Some("4 个分区".to_string()), description: Some("避免下载、桌面、outputs 反复各自维护。".to_string()), items: FILE_SCOPE_SUMMARIES.iter().map(|summary| { let key = summary.key.to_string(); let title = summary.title.to_string(); let eyebrow = Some(summary.eyebrow.to_string()); let preview = Some(summary.preview.to_string()); let meta = vec![summary.meta[0].to_string(), summary.meta[1].to_string()]; let active = scope == summary.key; let mut selected_scope = selected_scope; GroupedListPanelItem { key: key.clone(), title, eyebrow, preview, meta, active, onpress: EventHandler::new(move |_| selected_scope.set(key.clone())), } }).collect::<Vec<_>>() }] }
            Surface {
                SurfaceHeader { title: "文件清单".to_string(), subtitle: "先落地最小可运行版：扫描本机 Downloads / Desktop / recent outputs。".to_string() }
                if rows.is_empty() { div { class: "empty-state", "当前分区没有匹配文件。" } } else { DataTable { columns: vec!["名称".to_string(), "位置".to_string(), "类型".to_string(), "修改时间".to_string(), "说明".to_string(),], for row in rows.iter() { tr { td { "{row.name}" } td { "{row.location}" } td { "{row.kind}" } td { "{row.modified_label}" } td { "{row.note}" } } } } }
            }
        }
    }
}

async fn load_file_records() -> Result<Vec<FileRecord>, String> {
    #[cfg(target_arch = "wasm32")]
    { return Ok(Vec::new()); }
    #[cfg(not(target_arch = "wasm32"))]
    { load_file_records_blocking() }
}

#[cfg(not(target_arch = "wasm32"))]
fn load_file_records_blocking() -> Result<Vec<FileRecord>, String> {
    let home = resolve_home_dir()?;
    let desktop = home.join("Desktop");
    let downloads = home.join("Downloads");
    let recent_outputs = home.join(".hermes/cron/output");
    let mut records = Vec::new();
    records.extend(scan_directory("desktop", &desktop, 80, false)?);
    records.extend(scan_directory("downloads", &downloads, 80, false)?);
    records.extend(scan_directory("recent-outputs", &recent_outputs, 80, true)?);
    records.extend(collect_yesterday_research(&records));
    Ok(records)
}

#[cfg(not(target_arch = "wasm32"))]
fn resolve_home_dir() -> Result<PathBuf, String> { std::env::var("HOME").map(PathBuf::from).map_err(|_| "无法解析 HOME 目录".to_string()) }

#[cfg(not(target_arch = "wasm32"))]
fn scan_directory(scope: &str, dir: &Path, limit: usize, prefer_outputs_note: bool) -> Result<Vec<FileRecord>, String> {
    if !dir.exists() { return Ok(Vec::new()); }
    let mut entries = Vec::new();
    for entry in fs::read_dir(dir).map_err(|err| format!("读取 {} 失败：{err}", dir.display()))? {
        let entry = entry.map_err(|err| format!("遍历 {} 失败：{err}", dir.display()))?;
        let path = entry.path();
        let metadata = entry.metadata().map_err(|err| format!("读取元数据失败 {}：{err}", path.display()))?;
        if metadata.is_dir() { continue; }
        let modified = metadata.modified().unwrap_or(SystemTime::UNIX_EPOCH);
        entries.push((path, metadata, modified));
    }
    entries.sort_by(|a, b| b.2.cmp(&a.2));
    entries.truncate(limit);
    Ok(entries.into_iter().map(|(path, metadata, modified)| {
        let name = path.file_name().and_then(|value| value.to_str()).unwrap_or("unknown").to_string();
        let kind = infer_kind(&path, &metadata);
        let modified_label = format_modified(modified);
        let status = if is_yesterday(modified) { "昨天更新" } else { "已发现" };
        let note = if prefer_outputs_note { "自动任务或批处理输出文件".to_string() } else if scope == "desktop" { "桌面真实文件，适合人工整理或归档。".to_string() } else { "下载或归档素材文件。".to_string() };
        FileRecord { scope: scope.to_string(), name, location: path.display().to_string(), kind, status: status.to_string(), note, modified_label }
    }).collect())
}

#[cfg(not(target_arch = "wasm32"))]
fn collect_yesterday_research(records: &[FileRecord]) -> Vec<FileRecord> {
    let mut filtered: Vec<FileRecord> = records.iter().filter(|item| (item.scope == "desktop" || item.scope == "recent-outputs") && item.status == "昨天更新" && looks_like_research_artifact(&item.name)).cloned().collect();
    filtered.sort_by(|a, b| b.modified_label.cmp(&a.modified_label));
    filtered.truncate(40);
    filtered.into_iter().map(|mut item| { item.scope = "yesterday-research".to_string(); item.note = if item.location.contains("/Desktop/") { "昨天在桌面更新的研究成果。".to_string() } else { "昨天生成的 recent output 研究成果。".to_string() }; item }).collect()
}

#[cfg(not(target_arch = "wasm32"))]
fn looks_like_research_artifact(name: &str) -> bool {
    let lower = name.to_lowercase();
    let keywords = ["research", "study", "report", "analysis", "summary", "plan", "topic", "note", "notes", "日报", "总结", "研究", "分析", "方案", "计划", "课题", "报告"];
    let suffixes = [".md", ".txt", ".json", ".csv", ".pdf", ".html", ".docx", ".xlsx"];
    keywords.iter().any(|kw| lower.contains(kw)) || suffixes.iter().any(|s| lower.ends_with(s))
}

#[cfg(not(target_arch = "wasm32"))]
fn infer_kind(path: &Path, metadata: &fs::Metadata) -> String {
    if metadata.len() == 0 { return "Empty".to_string(); }
    match path.extension().and_then(|ext| ext.to_str()).map(|ext| ext.to_lowercase()).as_deref() {
        Some("md") => "Markdown".to_string(), Some("txt") => "Text".to_string(), Some("json") => "JSON".to_string(), Some("csv") => "CSV".to_string(), Some("pdf") => "PDF".to_string(), Some("html") => "HTML".to_string(), Some("docx") => "Word".to_string(), Some("xlsx") => "Spreadsheet".to_string(), Some("zip") => "Archive".to_string(), Some("tar") | Some("gz") | Some("tgz") => "Archive".to_string(), Some(other) => other.to_uppercase(), None => "File".to_string(), }
}

#[cfg(not(target_arch = "wasm32"))]
fn format_modified(modified: SystemTime) -> String { let dt: DateTime<Local> = modified.into(); dt.format("%Y-%m-%d %H:%M").to_string() }

#[cfg(not(target_arch = "wasm32"))]
fn is_yesterday(modified: SystemTime) -> bool { let dt: DateTime<Local> = modified.into(); let now = Local::now(); dt.date_naive() == (now - Duration::days(1)).date_naive() }
