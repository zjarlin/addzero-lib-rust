use std::path::Path;

use chrono::DateTime;
use dioxus::prelude::*;
use dioxus_components::{
    ContentHeader, DataTable, Field, MetricStrip, StatTile, Surface, SurfaceHeader, Tone,
    WorkbenchButton,
};

use crate::admin::domains::KNOWLEDGE_DOMAIN_ID;
use crate::services::{
    StorageBrowseRequestDto, StorageCreateFolderDto, StorageDeleteFolderDto,
    StorageDeleteObjectDto, StorageFileDto, StorageFolderDto, StorageShareRequestDto,
    StorageShareResultDto, StorageUploadFileDto, StorageUploadRequestDto,
};
use crate::state::AppServices;

const DOWNLOAD_STATION_PAGE_ID: &str = "download-station";

#[component]
pub fn DownloadStationScene() -> Element {
    rsx! { DownloadStationWorkspaceShell { show_header: true } }
}

#[component]
pub fn DownloadStationWorkspace() -> Element {
    rsx! { DownloadStationWorkspaceShell { show_header: false } }
}

#[component]
fn DownloadStationWorkspaceShell(show_header: bool) -> Element {
    let services = use_context::<AppServices>();
    let minio_files = services.minio_files.clone();

    let current_prefix = use_signal(String::new);
    let feedback = use_signal(|| None::<String>);
    let share_result = use_signal(|| None::<StorageShareResultDto>);
    let new_folder_path = use_signal(String::new);
    let pending = use_signal(|| false);
    let refresh_nonce = use_signal(|| 0_u64);
    let search_query = use_signal(String::new);
    let selected_category = use_signal(|| None::<FileCategory>);
    let file_sort = use_signal(|| FileSortKey::Updated);
    let sort_desc = use_signal(|| true);
    let file_view = use_signal(|| FileViewMode::List);
    let share_expiration_hours = use_signal(|| 24_u64);

    let explorer_resource = {
        let minio_files = minio_files.clone();
        use_resource(move || {
            let minio_files = minio_files.clone();
            let prefix = current_prefix.read().clone();
            let _refresh_nonce = *refresh_nonce.read();
            async move { minio_files.browse(StorageBrowseRequestDto { prefix }).await }
        })
    };
    let root_resource = {
        let minio_files = minio_files.clone();
        use_resource(move || {
            let minio_files = minio_files.clone();
            let current_prefix_value = current_prefix.read().clone();
            let _refresh_nonce = *refresh_nonce.read();
            async move {
                if current_prefix_value.is_empty() {
                    Ok(None)
                } else {
                    minio_files
                        .browse(StorageBrowseRequestDto {
                            prefix: String::new(),
                        })
                        .await
                        .map(Some)
                }
            }
        })
    };

    let explorer = match explorer_resource.read().as_ref() {
        Some(Ok(explorer)) => explorer.clone(),
        Some(Err(err)) => {
            let failure = describe_download_station_failure(err.to_string());
            return rsx! {
                if show_header {
                    ContentHeader {
                        title: "下载站".to_string(),
                        subtitle: "所有上传、下载、分享和目录管理都通过 MinIO 做对象存储，不再扫描本地目录。".to_string()
                    }
                }
                Surface {
                    SurfaceHeader {
                        title: failure.title,
                        subtitle: failure.subtitle
                    }
                    div {
                        class: if failure.is_info {
                            "callout callout--info"
                        } else {
                            "callout"
                        },
                        "{failure.detail}"
                    }
                }
            };
        }
        None => {
            return rsx! {
                if show_header {
                    ContentHeader {
                        title: "下载站".to_string(),
                        subtitle: "所有上传、下载、分享和目录管理都通过 MinIO 做对象存储，不再扫描本地目录。".to_string()
                    }
                }
                Surface { div { class: "empty-state", "正在连接 MinIO 并读取对象目录…" } }
            };
        }
    };

    let bucket = explorer.bucket.clone();
    let current_prefix_value = explorer.current_prefix.clone();
    let current_path_label = display_prefix(current_prefix_value.as_str());
    let parent_prefix_value = explorer.parent_prefix.clone();
    let backend_label = explorer.backend_label.clone();
    let breadcrumbs = explorer.breadcrumbs.clone();
    let folders = explorer.folders.clone();
    let files = explorer.files.clone();
    let folder_count = explorer.folder_count;
    let file_count = explorer.file_count;
    let root_folders = if current_prefix_value.is_empty() {
        folders.clone()
    } else {
        root_resource
            .read()
            .as_ref()
            .and_then(|result| result.as_ref().ok())
            .and_then(|root| root.as_ref())
            .map(|root| root.folders.clone())
            .unwrap_or_default()
    };
    let active_source_prefix = top_level_prefix(current_prefix_value.as_str());
    let active_source_label = active_source_prefix
        .as_deref()
        .and_then(|prefix| {
            root_folders
                .iter()
                .find(|folder| folder.prefix == prefix)
                .map(|folder| folder.name.clone())
        })
        .unwrap_or_else(|| "全部资源".to_string());
    let search_query_value = search_query.read().trim().to_lowercase();
    let active_category = *selected_category.read();
    let active_sort = *file_sort.read();
    let sort_descending = *sort_desc.read();
    let active_view = *file_view.read();
    let share_hours = *share_expiration_hours.read();
    let share_expiration_value = share_hours.to_string();
    let share_expiration_seconds = share_hours.saturating_mul(3600);
    let share_expiration_label = share_expiration_label(share_hours);
    let category_label = active_category.map(FileCategory::label).unwrap_or("全部");
    let category_counts = file_category_counts(&files);
    let installer_count = files
        .iter()
        .filter(|file| file_category(file) == FileCategory::Installer)
        .count();
    let visible_folders = folders
        .iter()
        .filter(|folder| folder_matches_query(folder, &search_query_value))
        .cloned()
        .collect::<Vec<_>>();
    let mut visible_files = files
        .iter()
        .filter(|file| file_matches_query(file, &search_query_value))
        .filter(|file| match active_category {
            Some(category) => file_category(file) == category,
            None => true,
        })
        .cloned()
        .collect::<Vec<_>>();
    visible_files.sort_by(|left, right| compare_files(left, right, active_sort, sort_descending));
    let should_group_visible_files = search_query_value.is_empty() && active_category.is_none();
    let grouped_visible_files = if should_group_visible_files {
        group_files_by_category(&visible_files)
    } else {
        Vec::new()
    };

    rsx! {
        if show_header {
            ContentHeader {
                title: "下载站".to_string(),
                subtitle: "以 MinIO 为唯一文件后端；新建目录本质上是在 bucket 内创建相对路径前缀。".to_string(),
                actions: rsx!(
                    div { class: "entry-actions",
                        WorkbenchButton {
                            class: "action-button".to_string(),
                            disabled: *pending.read() || current_prefix_value.is_empty(),
                            onclick: {
                                let mut current_prefix = current_prefix;
                                let mut share_result = share_result;
                                move |_| {
                                    current_prefix.set(String::new());
                                    share_result.set(None);
                                }
                            },
                            "根目录"
                        }
                        WorkbenchButton {
                            class: "action-button".to_string(),
                            disabled: *pending.read() || parent_prefix_value.is_none(),
                            onclick: {
                                let mut current_prefix = current_prefix;
                                let mut share_result = share_result;
                                let parent_prefix_value = parent_prefix_value.clone();
                                move |_| {
                                    current_prefix.set(parent_prefix_value.clone().unwrap_or_default());
                                    share_result.set(None);
                                }
                            },
                            "上一级"
                        }
                    }
                )
            }
        }

        if let Some(message) = feedback.read().clone() {
            div {
                class: if message.contains("失败") || message.contains("非法") || message.contains("不存在") {
                    "callout"
                } else {
                    "callout callout--info"
                },
                "{message}"
            }
        }

        MetricStrip { columns: 4,
            StatTile {
                label: "Bucket".to_string(),
                value: bucket,
                detail: "当前下载站统一挂在一个 MinIO bucket 下。".to_string()
            }
            StatTile {
                label: "当前路径".to_string(),
                value: current_path_label.clone(),
                detail: "目录就是对象 key 的相对前缀。".to_string()
            }
            StatTile {
                label: "子目录".to_string(),
                value: folder_count.to_string(),
                detail: "当前层级下的直接子目录数量。".to_string()
            }
            StatTile {
                label: "文件".to_string(),
                value: file_count.to_string(),
                detail: "当前层级下的直接文件数量。".to_string()
            }
        }

        Surface {
            SurfaceHeader {
                title: "资源操作".to_string(),
                subtitle: backend_label
            }
            div { class: "settings-grid",
                div { class: "settings-panel stack",
                    label { class: "upload-dropzone",
                        span { class: "upload-dropzone__eyebrow", "MinIO Upload" }
                        span { class: "upload-dropzone__title", "上传文件到当前前缀" }
                        span { class: "upload-dropzone__detail", "支持一次选择多个文件；上传后直接写入 MinIO，不落本地目录。" }
                        input {
                            class: "upload-dropzone__input",
                            r#type: "file",
                            multiple: true,
                            disabled: *pending.read(),
                            onchange: {
                                let minio_files = minio_files.clone();
                                let mut feedback = feedback;
                                let mut pending = pending;
                                let mut share_result = share_result;
                                let current_prefix = current_prefix;
                                move |evt| {
                                    let files = evt.files().into_iter().collect::<Vec<_>>();
                                    if files.is_empty() || *pending.read() {
                                        return;
                                    }

                                    let prefix = current_prefix.read().clone();
                                    let selected_count = files.len();
                                    pending.set(true);
                                    share_result.set(None);
                                    feedback.set(Some(format!("正在读取并上传 {selected_count} 个文件…")));

                                    let minio_files = minio_files.clone();
                                    let mut feedback = feedback;
                                    let mut pending = pending;
                                    let refresh_nonce = refresh_nonce;

                                    spawn(async move {
                                        let mut upload_files = Vec::with_capacity(selected_count);
                                        for file in files {
                                            let file_name = file.name();
                                            let content_type = file.content_type();
                                            match file.read_bytes().await {
                                                Ok(bytes) => upload_files.push(StorageUploadFileDto {
                                                    file_name,
                                                    content_type,
                                                    bytes: bytes.to_vec(),
                                                }),
                                                Err(err) => {
                                                    feedback.set(Some(format!(
                                                        "读取文件失败：{file_name}，{err}"
                                                    )));
                                                    pending.set(false);
                                                    return;
                                                }
                                            }
                                        }

                                        match minio_files
                                            .upload_files(StorageUploadRequestDto {
                                                prefix,
                                                files: upload_files,
                                            })
                                            .await
                                        {
                                            Ok(report) => {
                                                feedback.set(Some(report.message));
                                                bump_refresh_nonce(refresh_nonce);
                                            }
                                            Err(err) => {
                                                feedback.set(Some(format!("上传失败：{err}")));
                                            }
                                        }

                                        pending.set(false);
                                    });
                                }
                            }
                        }
                    }

                    div { class: "form-grid form-grid--compact",
                        Field {
                            label: "新建目录相对路径".to_string(),
                            value: new_folder_path.read().clone(),
                            placeholder: "例如 installers/mac/2026".to_string(),
                            on_input: {
                                let mut new_folder_path = new_folder_path;
                                move |value| new_folder_path.set(value)
                            }
                        }
                        Field {
                            label: "当前挂载前缀".to_string(),
                            value: current_path_label.clone(),
                            readonly: true
                        }
                    }

                    div { class: "entry-actions",
                        WorkbenchButton {
                            class: "action-button action-button--primary".to_string(),
                            disabled: *pending.read(),
                            onclick: {
                                let minio_files = minio_files.clone();
                                let mut feedback = feedback;
                                let mut pending = pending;
                                let refresh_nonce = refresh_nonce;
                                let mut share_result = share_result;
                                let new_folder_path = new_folder_path;
                                let current_prefix = current_prefix;
                                move |_| {
                                    let draft = new_folder_path.read().trim().to_string();
                                    if draft.is_empty() || *pending.read() {
                                        return;
                                    }

                                    let parent_prefix = current_prefix.read().clone();
                                    pending.set(true);
                                    share_result.set(None);
                                    feedback.set(Some(format!("正在创建目录 `{draft}`…")));

                                    let minio_files = minio_files.clone();
                                    let mut feedback = feedback;
                                    let mut pending = pending;
                                    let refresh_nonce = refresh_nonce;
                                    let mut new_folder_path = new_folder_path;

                                    spawn(async move {
                                        match minio_files
                                            .create_folder(StorageCreateFolderDto {
                                                parent_prefix,
                                                relative_path: draft,
                                            })
                                            .await
                                        {
                                            Ok(result) => {
                                                new_folder_path.set(String::new());
                                                feedback.set(Some(result.message));
                                                bump_refresh_nonce(refresh_nonce);
                                            }
                                            Err(err) => {
                                                feedback.set(Some(format!("创建目录失败：{err}")));
                                            }
                                        }

                                        pending.set(false);
                                    });
                                }
                            },
                            "新建目录"
                        }
                    }
                }

                div { class: "settings-panel stack",
                    div { class: "callout callout--info",
                        "当前路径：{current_path_label}"
                    }
                    label { class: "field",
                        span { class: "field__label", "分享默认有效期" }
                        select {
                            class: "field__input",
                            value: "{share_expiration_value}",
                            onchange: {
                                let mut share_expiration_hours = share_expiration_hours;
                                move |evt| {
                                    if let Ok(hours) = evt.value().parse::<u64>() {
                                        share_expiration_hours.set(hours);
                                    }
                                }
                            },
                            for option in SHARE_EXPIRY_OPTIONS {
                                option {
                                    value: "{option.hours}",
                                    selected: option.hours == share_hours,
                                    "{option.label}"
                                }
                            }
                        }
                    }
                    div { class: "settings-note",
                        "文件行里的“分享”会按当前默认时长生成 presigned URL；如果配置了分享密钥，也会额外给出加密链接。"
                    }
                    div { class: "file-breadcrumbs",
                        for crumb in breadcrumbs.iter() {
                            WorkbenchButton {
                                class: "segment-button".to_string(),
                                tone: if crumb.prefix == current_prefix_value {
                                    Some(Tone::Accent)
                                } else {
                                    None
                                },
                                disabled: *pending.read() || crumb.prefix == current_prefix_value,
                                onclick: {
                                    let mut current_prefix = current_prefix;
                                    let mut share_result = share_result;
                                    let target_prefix = crumb.prefix.clone();
                                    move |_| {
                                        current_prefix.set(target_prefix.clone());
                                        share_result.set(None);
                                    }
                                },
                                "{crumb.label}"
                            }
                        }
                    }
                    if let Some(share) = share_result.read().clone() {
                        div { class: "stack content-stack",
                            div { class: "callout callout--info",
                                "已生成分享链接：{share.relative_path}，有效期 {share_expiration_label_from_seconds(share.expires_in_seconds)}。"
                            }
                            Field {
                                label: "分享 URL".to_string(),
                                value: share.presigned_url.clone(),
                                readonly: true
                            }
                            if let Some(encrypted_url) = share.encrypted_url.clone() {
                                Field {
                                    label: "加密分享 URL".to_string(),
                                    value: encrypted_url,
                                    readonly: true
                                }
                            }
                        }
                    }
                }
            }
        }

        Surface {
            SurfaceHeader {
                title: "下载站视图".to_string(),
                subtitle: "参考 download-station.py，在当前 MinIO 前缀上提供搜索、分类、排序、卡片/列表切换和分享时长设置。".to_string()
            }
            div { class: "download-toolbar",
                div { class: "form-grid form-grid--compact",
                    label { class: "field",
                        span { class: "field__label", "搜索文件" }
                        input {
                            class: "field__input",
                            value: search_query.read().clone(),
                            placeholder: "搜索文件名、相对路径、类型…",
                            oninput: {
                                let mut search_query = search_query;
                                move |evt| search_query.set(evt.value())
                            }
                        }
                    }
                    div { class: "download-toolbar__summary",
                        div { class: "download-toolbar__summary-item",
                            span { class: "download-toolbar__summary-label", "文件匹配" }
                            strong { "{visible_files.len()} / {file_count}" }
                        }
                        div { class: "download-toolbar__summary-item",
                            span { class: "download-toolbar__summary-label", "目录匹配" }
                            strong { "{visible_folders.len()} / {folder_count}" }
                        }
                        div { class: "download-toolbar__summary-item",
                            span { class: "download-toolbar__summary-label", "安装包" }
                            strong { "{installer_count}" }
                        }
                        div { class: "download-toolbar__summary-item",
                            span { class: "download-toolbar__summary-label", "当前来源" }
                            strong { "{active_source_label}" }
                        }
                        div { class: "download-toolbar__summary-item",
                            span { class: "download-toolbar__summary-label", "分享默认" }
                            strong { "{share_expiration_label}" }
                        }
                    }
                }

                div { class: "download-toolbar__group",
                    span { class: "download-toolbar__group-label", "来源" }
                    div { class: "download-pill-row",
                        button {
                            r#type: "button",
                            class: if current_prefix_value.is_empty() {
                                "download-pill download-pill--active"
                            } else {
                                "download-pill"
                            },
                            onclick: {
                                let mut current_prefix = current_prefix;
                                let mut share_result = share_result;
                                move |_| {
                                    current_prefix.set(String::new());
                                    share_result.set(None);
                                }
                            },
                            "全部资源"
                            span { class: "download-pill__count", "{root_folders.len()}" }
                        }
                        for folder in root_folders.iter() {
                            button {
                                r#type: "button",
                                class: if active_source_prefix.as_deref() == Some(folder.prefix.as_str()) {
                                    "download-pill download-pill--active"
                                } else {
                                    "download-pill"
                                },
                                onclick: {
                                    let mut current_prefix = current_prefix;
                                    let mut share_result = share_result;
                                    let target_prefix = folder.prefix.clone();
                                    move |_| {
                                        current_prefix.set(target_prefix.clone());
                                        share_result.set(None);
                                    }
                                },
                                "{folder.name}"
                                if let Some(count) = source_badge_label(folder) {
                                    span { class: "download-pill__count", "{count}" }
                                }
                            }
                        }
                    }
                }

                div { class: "download-toolbar__row",
                    div { class: "download-toolbar__group",
                        span { class: "download-toolbar__group-label", "分类" }
                        div { class: "download-pill-row",
                            button {
                                r#type: "button",
                                class: if active_category.is_none() {
                                    "download-pill download-pill--active"
                                } else {
                                    "download-pill"
                                },
                                onclick: {
                                    let mut selected_category = selected_category;
                                    move |_| selected_category.set(None)
                                },
                                "全部"
                                span { class: "download-pill__count", "{file_count}" }
                            }
                            for (category, count) in category_counts.iter().copied() {
                                button {
                                    r#type: "button",
                                    class: if active_category == Some(category) {
                                        "download-pill download-pill--active"
                                    } else {
                                        "download-pill"
                                    },
                                    onclick: {
                                        let mut selected_category = selected_category;
                                        move |_| selected_category.set(Some(category))
                                    },
                                    "{category.label()}"
                                    span { class: "download-pill__count", "{count}" }
                                }
                            }
                        }
                    }

                    div { class: "download-toolbar__group",
                        span { class: "download-toolbar__group-label", "排序" }
                        div { class: "download-chip-row",
                            for key in FILE_SORT_KEYS {
                                button {
                                    r#type: "button",
                                    class: if active_sort == key {
                                        "download-chip download-chip--active"
                                    } else {
                                        "download-chip"
                                    },
                                    onclick: {
                                        let mut file_sort = file_sort;
                                        let mut sort_desc = sort_desc;
                                        move |_| {
                                            if *file_sort.read() == key {
                                                let next_desc = !*sort_desc.read();
                                                sort_desc.set(next_desc);
                                            } else {
                                                file_sort.set(key);
                                                sort_desc.set(key.default_descending());
                                            }
                                        }
                                    },
                                    "{key.label()}"
                                    if active_sort == key {
                                        span { class: "download-chip__direction",
                                            if sort_descending { "↓" } else { "↑" }
                                        }
                                    }
                                }
                            }
                        }
                    }

                    div { class: "download-toolbar__group",
                        span { class: "download-toolbar__group-label", "视图" }
                        div { class: "download-chip-row",
                            for view in FILE_VIEW_MODES {
                                button {
                                    r#type: "button",
                                    class: if active_view == view {
                                        "download-chip download-chip--active"
                                    } else {
                                        "download-chip"
                                    },
                                    onclick: {
                                        let mut file_view = file_view;
                                        move |_| file_view.set(view)
                                    },
                                    "{view.label()}"
                                }
                            }
                        }
                    }
                }

                div { class: "settings-note",
                    "当前来源 "
                    strong { "{active_source_label}" }
                    "；当前路径 "
                    strong { "{current_path_label}" }
                    " 下，分类过滤为 "
                    strong { "{category_label}" }
                    "；分享链接默认有效期 "
                    strong { "{share_expiration_label}" }
                    "。"
                }
            }
        }

        Surface {
            SurfaceHeader {
                title: "目录".to_string(),
                subtitle: "目录由对象 key 前缀推导；当前按层浏览，不递归统计整棵子树的对象数和体量。".to_string()
            }
            if visible_folders.is_empty() {
                div { class: "empty-state",
                    if search_query_value.is_empty() {
                        "当前目录下还没有子目录。"
                    } else {
                        "当前目录下没有匹配搜索条件的子目录。"
                    }
                }
            } else {
                DataTable {
                    columns: vec![
                        "名称".to_string(),
                        "前缀".to_string(),
                        "当前层文件数".to_string(),
                        "当前层体量".to_string(),
                        "操作".to_string(),
                    ],
                    for folder in visible_folders.iter() {
                        tr {
                            td { "{folder.name}" }
                            td { "/{folder.prefix}" }
                            td { "{folder_object_count_label(folder)}" }
                            td { "{folder_size_label(folder)}" }
                            td {
                                div { class: "file-row-actions",
                                    WorkbenchButton {
                                        class: "action-button".to_string(),
                                        disabled: *pending.read(),
                                        onclick: {
                                            let mut current_prefix = current_prefix;
                                            let mut share_result = share_result;
                                            let target_prefix = folder.prefix.clone();
                                            move |_| {
                                                current_prefix.set(target_prefix.clone());
                                                share_result.set(None);
                                            }
                                        },
                                        "进入"
                                    }
                                    WorkbenchButton {
                                        class: "action-button".to_string(),
                                        disabled: *pending.read(),
                                        onclick: {
                                            let minio_files = minio_files.clone();
                                            let mut feedback = feedback;
                                            let mut pending = pending;
                                            let refresh_nonce = refresh_nonce;
                                            let folder_prefix = folder.prefix.clone();
                                            move |_| {
                                                if *pending.read() {
                                                    return;
                                                }

                                                pending.set(true);
                                                feedback.set(Some(format!(
                                                    "正在删除目录 {} …",
                                                    display_prefix(folder_prefix.as_str())
                                                )));

                                                let minio_files = minio_files.clone();
                                                let mut feedback = feedback;
                                                let mut pending = pending;
                                                let refresh_nonce = refresh_nonce;
                                                let folder_prefix = folder_prefix.clone();

                                                spawn(async move {
                                                    match minio_files
                                                        .delete_folder(StorageDeleteFolderDto {
                                                            prefix: folder_prefix,
                                                        })
                                                        .await
                                                    {
                                                        Ok(result) => {
                                                            feedback.set(Some(result.message));
                                                            bump_refresh_nonce(refresh_nonce);
                                                        }
                                                        Err(err) => {
                                                            feedback.set(Some(format!(
                                                                "删除目录失败：{err}"
                                                            )));
                                                        }
                                                    }

                                                    pending.set(false);
                                                });
                                            }
                                        },
                                        "删除"
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        Surface {
            SurfaceHeader {
                title: "文件".to_string(),
                subtitle: format!(
                    "当前匹配 {} / {} 个文件；下载通过后台签发临时 URL，分享默认有效期 {}。",
                    visible_files.len(),
                    file_count,
                    share_expiration_label
                )
            }
            if visible_files.is_empty() {
                div { class: "empty-state",
                    if files.is_empty() {
                        "当前目录下还没有文件。"
                    } else {
                        "当前目录下没有匹配搜索或分类条件的文件。"
                    }
                }
            } else if active_view == FileViewMode::Cards {
                if should_group_visible_files {
                    div { class: "download-group-list",
                        for group in grouped_visible_files.iter() {
                            section { class: "download-group",
                                div { class: "download-group__header",
                                    strong { class: "download-group__title", "{group.category.label()}" }
                                    span { class: "download-group__count", "{group.files.len()} 个文件" }
                                }
                                div { class: "download-card-grid",
                                    for file in group.files.iter() {
                                        article {
                                            class: if file_category(file) == FileCategory::Installer {
                                                "download-card download-card--installer"
                                            } else {
                                                "download-card"
                                            },
                                            div { class: "download-card__eyebrow",
                                                span { class: "download-card__category", "{file_category(file).label()}" }
                                                span { class: "download-card__content-type", "{content_type_label(file)}" }
                                            }
                                            h3 { class: "download-card__title", "{file.name}" }
                                            div { class: "download-card__path", "/{file.relative_path}" }
                                            div { class: "download-card__stats",
                                                span { "{format_bytes(file.size_bytes)}" }
                                                span { "{file.last_modified}" }
                                            }
                                            DownloadFileActions {
                                                file: file.clone(),
                                                pending,
                                                feedback,
                                                share_result,
                                                refresh_nonce,
                                                share_expiration_seconds
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                } else {
                    div { class: "download-card-grid",
                        for file in visible_files.iter() {
                            article {
                                class: if file_category(file) == FileCategory::Installer {
                                    "download-card download-card--installer"
                                } else {
                                    "download-card"
                                },
                                div { class: "download-card__eyebrow",
                                    span { class: "download-card__category", "{file_category(file).label()}" }
                                    span { class: "download-card__content-type", "{content_type_label(file)}" }
                                }
                                h3 { class: "download-card__title", "{file.name}" }
                                div { class: "download-card__path", "/{file.relative_path}" }
                                div { class: "download-card__stats",
                                    span { "{format_bytes(file.size_bytes)}" }
                                    span { "{file.last_modified}" }
                                }
                                DownloadFileActions {
                                    file: file.clone(),
                                    pending,
                                    feedback,
                                    share_result,
                                    refresh_nonce,
                                    share_expiration_seconds
                                }
                            }
                        }
                    }
                }
            } else {
                if should_group_visible_files {
                    div { class: "download-group-list",
                        for group in grouped_visible_files.iter() {
                            section { class: "download-group",
                                div { class: "download-group__header",
                                    strong { class: "download-group__title", "{group.category.label()}" }
                                    span { class: "download-group__count", "{group.files.len()} 个文件" }
                                }
                                DataTable {
                                    columns: vec![
                                        "名称".to_string(),
                                        "分类".to_string(),
                                        "类型".to_string(),
                                        "大小".to_string(),
                                        "更新时间".to_string(),
                                        "操作".to_string(),
                                    ],
                                    for file in group.files.iter() {
                                        tr {
                                            td {
                                                div { class: "file-name-cell",
                                                    strong { "{file.name}" }
                                                    div { class: "file-name-cell__path", "/{file.relative_path}" }
                                                }
                                            }
                                            td { "{file_category(file).label()}" }
                                            td { "{content_type_label(file)}" }
                                            td { "{format_bytes(file.size_bytes)}" }
                                            td { "{file.last_modified}" }
                                            td {
                                                DownloadFileActions {
                                                    file: file.clone(),
                                                    pending,
                                                    feedback,
                                                    share_result,
                                                    refresh_nonce,
                                                    share_expiration_seconds
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                } else {
                    DataTable {
                        columns: vec![
                            "名称".to_string(),
                            "分类".to_string(),
                            "类型".to_string(),
                            "大小".to_string(),
                            "更新时间".to_string(),
                            "操作".to_string(),
                        ],
                        for file in visible_files.iter() {
                            tr {
                                td {
                                    div { class: "file-name-cell",
                                        strong { "{file.name}" }
                                        div { class: "file-name-cell__path", "/{file.relative_path}" }
                                    }
                                }
                                td { "{file_category(file).label()}" }
                                td { "{content_type_label(file)}" }
                                td { "{format_bytes(file.size_bytes)}" }
                                td { "{file.last_modified}" }
                                td {
                                    DownloadFileActions {
                                        file: file.clone(),
                                        pending,
                                        feedback,
                                        share_result,
                                        refresh_nonce,
                                        share_expiration_seconds
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

#[derive(Props, Clone, PartialEq)]
struct DownloadFileActionsProps {
    file: StorageFileDto,
    pending: Signal<bool>,
    feedback: Signal<Option<String>>,
    share_result: Signal<Option<StorageShareResultDto>>,
    refresh_nonce: Signal<u64>,
    share_expiration_seconds: u64,
}

#[component]
fn DownloadFileActions(props: DownloadFileActionsProps) -> Element {
    let services = use_context::<AppServices>();
    let minio_files = services.minio_files.clone();
    let file = props.file.clone();
    let download_label = download_action_label(&file);

    rsx! {
        div { class: "file-row-actions",
            a {
                class: "action-button",
                href: file.download_path.clone(),
                target: "_blank",
                rel: "noreferrer",
                "{download_label}"
            }
            WorkbenchButton {
                class: "action-button".to_string(),
                disabled: *props.pending.read(),
                onclick: {
                    let minio_files = minio_files.clone();
                    let mut feedback = props.feedback;
                    let mut pending = props.pending;
                    let share_result = props.share_result;
                    let object_key = file.object_key.clone();
                    let relative_path = file.relative_path.clone();
                    let share_expiration_seconds = props.share_expiration_seconds;
                    move |_| {
                        if *pending.read() {
                            return;
                        }

                        pending.set(true);
                        feedback.set(Some(format!(
                            "正在生成 `{relative_path}` 的分享链接…"
                        )));

                        let minio_files = minio_files.clone();
                        let mut feedback = feedback;
                        let mut pending = pending;
                        let mut share_result = share_result;
                        let object_key = object_key.clone();

                        spawn(async move {
                            match minio_files
                                .share_file(StorageShareRequestDto {
                                    object_key,
                                    expiration_seconds: Some(share_expiration_seconds),
                                })
                                .await
                            {
                                Ok(result) => {
                                    share_result.set(Some(result));
                                    feedback
                                        .set(Some("分享链接已生成，已更新到右侧面板。".to_string()));
                                }
                                Err(err) => {
                                    feedback.set(Some(format!("生成分享链接失败：{err}")));
                                }
                            }

                            pending.set(false);
                        });
                    }
                },
                "分享"
            }
            WorkbenchButton {
                class: "action-button".to_string(),
                disabled: *props.pending.read(),
                onclick: {
                    let minio_files = minio_files.clone();
                    let mut feedback = props.feedback;
                    let mut pending = props.pending;
                    let refresh_nonce = props.refresh_nonce;
                    let object_key = file.object_key.clone();
                    move |_| {
                        if *pending.read() {
                            return;
                        }

                        pending.set(true);
                        feedback.set(Some(format!("正在删除文件 `{object_key}`…")));

                        let minio_files = minio_files.clone();
                        let mut feedback = feedback;
                        let mut pending = pending;
                        let refresh_nonce = refresh_nonce;
                        let object_key = object_key.clone();

                        spawn(async move {
                            match minio_files
                                .delete_file(StorageDeleteObjectDto { object_key })
                                .await
                            {
                                Ok(result) => {
                                    feedback.set(Some(result.message));
                                    bump_refresh_nonce(refresh_nonce);
                                }
                                Err(err) => {
                                    feedback.set(Some(format!("删除文件失败：{err}")));
                                }
                            }

                            pending.set(false);
                        });
                    }
                },
                "删除"
            }
        }
    }
}

fn bump_refresh_nonce(mut refresh_nonce: Signal<u64>) {
    let next = (*refresh_nonce.read()).saturating_add(1);
    refresh_nonce.set(next);
}

fn folder_object_count_label(folder: &StorageFolderDto) -> String {
    if folder.object_count == 0 && folder.size_bytes == 0 {
        "按需统计".to_string()
    } else {
        folder.object_count.to_string()
    }
}

fn folder_size_label(folder: &StorageFolderDto) -> String {
    if folder.object_count == 0 && folder.size_bytes == 0 {
        "-".to_string()
    } else {
        format_bytes(folder.size_bytes)
    }
}

fn source_badge_label(folder: &StorageFolderDto) -> Option<String> {
    (folder.object_count > 0).then(|| folder.object_count.to_string())
}

fn display_prefix(prefix: &str) -> String {
    let trimmed = prefix.trim_matches('/');
    if trimmed.is_empty() {
        "/".to_string()
    } else {
        format!("/{trimmed}/")
    }
}

fn format_bytes(bytes: u64) -> String {
    if bytes >= 1_048_576 {
        format!("{:.1} MB", bytes as f64 / 1_048_576.0)
    } else if bytes >= 1_024 {
        format!("{:.0} KB", bytes as f64 / 1_024.0)
    } else {
        format!("{bytes} B")
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum FileCategory {
    Installer,
    Archive,
    Document,
    Image,
    Audio,
    Video,
    CodeData,
    KeyCert,
    Other,
}

impl FileCategory {
    fn label(self) -> &'static str {
        match self {
            Self::Installer => "安装包",
            Self::Archive => "压缩包",
            Self::Document => "文档",
            Self::Image => "图片",
            Self::Audio => "音频",
            Self::Video => "视频",
            Self::CodeData => "代码/数据",
            Self::KeyCert => "密钥/证书",
            Self::Other => "其他",
        }
    }
}

const FILE_CATEGORY_ORDER: [FileCategory; 9] = [
    FileCategory::Installer,
    FileCategory::Archive,
    FileCategory::Document,
    FileCategory::Image,
    FileCategory::Audio,
    FileCategory::Video,
    FileCategory::CodeData,
    FileCategory::KeyCert,
    FileCategory::Other,
];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum FileSortKey {
    Updated,
    Name,
    Size,
    Kind,
}

impl FileSortKey {
    fn label(self) -> &'static str {
        match self {
            Self::Updated => "日期",
            Self::Name => "名称",
            Self::Size => "大小",
            Self::Kind => "类型",
        }
    }

    fn default_descending(self) -> bool {
        matches!(self, Self::Updated | Self::Size)
    }
}

const FILE_SORT_KEYS: [FileSortKey; 4] = [
    FileSortKey::Updated,
    FileSortKey::Name,
    FileSortKey::Size,
    FileSortKey::Kind,
];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum FileViewMode {
    List,
    Cards,
}

impl FileViewMode {
    fn label(self) -> &'static str {
        match self {
            Self::List => "列表",
            Self::Cards => "卡片",
        }
    }
}

const FILE_VIEW_MODES: [FileViewMode; 2] = [FileViewMode::List, FileViewMode::Cards];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct ShareExpiryOption {
    hours: u64,
    label: &'static str,
}

const SHARE_EXPIRY_OPTIONS: [ShareExpiryOption; 5] = [
    ShareExpiryOption {
        hours: 1,
        label: "1 小时",
    },
    ShareExpiryOption {
        hours: 6,
        label: "6 小时",
    },
    ShareExpiryOption {
        hours: 24,
        label: "1 天",
    },
    ShareExpiryOption {
        hours: 168,
        label: "7 天",
    },
    ShareExpiryOption {
        hours: 720,
        label: "30 天",
    },
];

struct FileCategoryGroup {
    category: FileCategory,
    files: Vec<crate::services::StorageFileDto>,
}

fn file_category_counts(files: &[crate::services::StorageFileDto]) -> Vec<(FileCategory, usize)> {
    FILE_CATEGORY_ORDER
        .iter()
        .filter_map(|category| {
            let count = files
                .iter()
                .filter(|file| file_category(file) == *category)
                .count();
            (count > 0).then_some((*category, count))
        })
        .collect()
}

fn group_files_by_category(files: &[crate::services::StorageFileDto]) -> Vec<FileCategoryGroup> {
    FILE_CATEGORY_ORDER
        .iter()
        .filter_map(|category| {
            let files = files
                .iter()
                .filter(|file| file_category(file) == *category)
                .cloned()
                .collect::<Vec<_>>();
            (!files.is_empty()).then_some(FileCategoryGroup {
                category: *category,
                files,
            })
        })
        .collect()
}

fn folder_matches_query(folder: &crate::services::StorageFolderDto, query: &str) -> bool {
    if query.is_empty() {
        return true;
    }

    let haystack = format!("{} {}", folder.name, folder.relative_path).to_lowercase();
    haystack.contains(query)
}

fn file_matches_query(file: &crate::services::StorageFileDto, query: &str) -> bool {
    if query.is_empty() {
        return true;
    }

    let extension = file_extension_key(&file.name);
    let haystack = format!(
        "{} {} {} {}",
        file.name, file.relative_path, file.content_type, extension
    )
    .to_lowercase();
    haystack.contains(query)
}

fn compare_files(
    left: &crate::services::StorageFileDto,
    right: &crate::services::StorageFileDto,
    key: FileSortKey,
    descending: bool,
) -> std::cmp::Ordering {
    let ordering = match key {
        FileSortKey::Updated => compare_last_modified(left, right)
            .then_with(|| left.name.to_lowercase().cmp(&right.name.to_lowercase())),
        FileSortKey::Name => left
            .name
            .to_lowercase()
            .cmp(&right.name.to_lowercase())
            .then_with(|| left.object_key.cmp(&right.object_key)),
        FileSortKey::Size => left
            .size_bytes
            .cmp(&right.size_bytes)
            .then_with(|| left.name.to_lowercase().cmp(&right.name.to_lowercase())),
        FileSortKey::Kind => file_category(left)
            .label()
            .cmp(file_category(right).label())
            .then_with(|| content_type_label(left).cmp(&content_type_label(right)))
            .then_with(|| left.name.to_lowercase().cmp(&right.name.to_lowercase())),
    };

    if descending {
        ordering.reverse()
    } else {
        ordering
    }
}

fn compare_last_modified(
    left: &crate::services::StorageFileDto,
    right: &crate::services::StorageFileDto,
) -> std::cmp::Ordering {
    match (
        parse_sortable_timestamp(left.last_modified.as_str()),
        parse_sortable_timestamp(right.last_modified.as_str()),
    ) {
        (Some(left_ts), Some(right_ts)) => left_ts.cmp(&right_ts),
        (Some(_), None) => std::cmp::Ordering::Greater,
        (None, Some(_)) => std::cmp::Ordering::Less,
        (None, None) => left.last_modified.cmp(&right.last_modified),
    }
}

fn parse_sortable_timestamp(value: &str) -> Option<i64> {
    DateTime::parse_from_rfc3339(value)
        .ok()
        .map(|timestamp| timestamp.timestamp_millis())
}

fn file_category(file: &crate::services::StorageFileDto) -> FileCategory {
    classify_file_name(&file.name)
}

fn classify_file_name(file_name: &str) -> FileCategory {
    match file_extension_key(file_name).as_str() {
        ".exe" | ".msi" | ".apk" | ".appimage" | ".deb" | ".rpm" | ".pkg" | ".snap"
        | ".flatpak" => FileCategory::Installer,
        ".zip" | ".rar" | ".7z" | ".tar" | ".gz" | ".bz2" | ".xz" | ".tgz" | ".tar.gz"
        | ".tbz2" | ".txz" | ".zst" | ".lz4" | ".iso" | ".dmg" => FileCategory::Archive,
        ".pdf" | ".doc" | ".docx" | ".xls" | ".xlsx" | ".ppt" | ".pptx" | ".txt" | ".md"
        | ".csv" | ".rtf" | ".odt" | ".ods" | ".odp" | ".epub" | ".mobi" | ".pages"
        | ".numbers" | ".key" => FileCategory::Document,
        ".jpg" | ".jpeg" | ".png" | ".gif" | ".bmp" | ".svg" | ".webp" | ".ico" | ".tiff"
        | ".tif" | ".psd" | ".raw" | ".heic" | ".heif" | ".avif" => FileCategory::Image,
        ".mp3" | ".flac" | ".wav" | ".aac" | ".ogg" | ".wma" | ".m4a" | ".ape" | ".opus"
        | ".mid" | ".midi" => FileCategory::Audio,
        ".mp4" | ".mkv" | ".avi" | ".mov" | ".wmv" | ".flv" | ".webm" | ".m4v" | ".rmvb"
        | ".3gp" => FileCategory::Video,
        ".py" | ".js" | ".ts" | ".java" | ".c" | ".cpp" | ".go" | ".rs" | ".rb" | ".php"
        | ".swift" | ".kt" | ".sh" | ".bash" | ".zsh" | ".json" | ".yaml" | ".yml" | ".toml"
        | ".xml" | ".html" | ".css" | ".sql" | ".db" | ".sqlite" => FileCategory::CodeData,
        ".pem" | ".crt" | ".cer" | ".p12" | ".pfx" | ".jks" | ".keystore" => FileCategory::KeyCert,
        _ => FileCategory::Other,
    }
}

fn file_extension_key(file_name: &str) -> String {
    let lower = file_name.to_ascii_lowercase();
    if lower.ends_with(".tar.gz") {
        ".tar.gz".to_string()
    } else {
        Path::new(file_name)
            .extension()
            .and_then(|extension| extension.to_str())
            .map(|extension| format!(".{}", extension.to_ascii_lowercase()))
            .unwrap_or_default()
    }
}

fn content_type_label(file: &crate::services::StorageFileDto) -> String {
    let extension = file_extension_key(&file.name);
    if extension.is_empty() {
        file.content_type.clone()
    } else {
        format!(
            "{} · {}",
            file.content_type,
            extension.trim_start_matches('.').to_ascii_uppercase()
        )
    }
}

fn download_action_label(file: &crate::services::StorageFileDto) -> &'static str {
    if file_category(file) == FileCategory::Installer {
        "下载安装包"
    } else {
        "下载"
    }
}

fn share_expiration_label(hours: u64) -> &'static str {
    SHARE_EXPIRY_OPTIONS
        .iter()
        .find(|option| option.hours == hours)
        .map(|option| option.label)
        .unwrap_or("自定义")
}

fn share_expiration_label_from_seconds(seconds: u64) -> String {
    if seconds % 3600 == 0 {
        share_expiration_label(seconds / 3600).to_string()
    } else {
        format!("{seconds} 秒")
    }
}

fn top_level_prefix(prefix: &str) -> Option<String> {
    let trimmed = prefix.trim_matches('/');
    let segment = trimmed.split('/').next().unwrap_or_default().trim();
    if segment.is_empty() {
        None
    } else {
        Some(format!("{segment}/"))
    }
}

struct DownloadStationFailure {
    title: String,
    subtitle: String,
    detail: String,
    is_info: bool,
}

fn describe_download_station_failure(err: String) -> DownloadStationFailure {
    let compact = sanitize_http_error(err.as_str());

    if err.contains("HTTP 501")
        || err.contains("HTTP 405")
        || err.contains("Method Not Allowed")
        || err.contains("Unsupported method")
        || err.contains("<!DOCTYPE HTML>")
    {
        return DownloadStationFailure {
            title: "对象存储预览不可用".to_string(),
            subtitle: "当前页面运行在静态预览环境，没有接入 admin_api 文件接口。".to_string(),
            detail: "下载站需要 `/api/admin/storage/files/*` 后端接口；页面结构已经可见，但真实对象浏览、上传、分享和删除动作要在桌面模式或 admin_api 联调环境下验收。".to_string(),
            is_info: true,
        };
    }

    if err.contains("MinIO")
        || err.contains("S3")
        || err.contains("bucket")
        || err.contains("endpoint")
        || err.contains("环境变量")
    {
        return DownloadStationFailure {
            title: "MinIO 初始化失败".to_string(),
            subtitle: "请先配置对象存储连接，再进入下载站。".to_string(),
            detail: format!("下载站加载失败：{compact}"),
            is_info: false,
        };
    }

    DownloadStationFailure {
        title: "下载站加载失败".to_string(),
        subtitle: "当前环境没有返回可用的对象浏览结果。".to_string(),
        detail: format!("下载站加载失败：{compact}"),
        is_info: false,
    }
}

fn sanitize_http_error(err: &str) -> String {
    let compact = err.split_whitespace().collect::<Vec<_>>().join(" ");
    const MAX_LEN: usize = 220;
    if compact.len() > MAX_LEN {
        format!("{}…", &compact[..MAX_LEN])
    } else {
        compact
    }
}

#[cfg(test)]
mod tests {
    use super::{
        FileCategory, FileSortKey, compare_files, group_files_by_category, top_level_prefix,
    };
    use crate::services::StorageFileDto;

    fn file(name: &str, last_modified: &str) -> StorageFileDto {
        StorageFileDto {
            name: name.to_string(),
            object_key: format!("assets/{name}"),
            relative_path: format!("assets/{name}"),
            size_bytes: 1,
            content_type: "application/octet-stream".to_string(),
            last_modified: last_modified.to_string(),
            download_path: format!("/download/{name}"),
        }
    }

    #[test]
    fn top_level_prefix_should_extract_root_folder() {
        assert_eq!(top_level_prefix(""), None);
        assert_eq!(top_level_prefix("assets"), Some("assets/".to_string()));
        assert_eq!(
            top_level_prefix("assets/installers/mac/"),
            Some("assets/".to_string())
        );
    }

    #[test]
    fn updated_sort_should_put_newer_files_first_when_descending() {
        let older = file("older.dmg", "2026-05-01T08:00:00Z");
        let newer = file("newer.dmg", "2026-05-02T08:00:00Z");

        assert_eq!(
            compare_files(&newer, &older, FileSortKey::Updated, true),
            std::cmp::Ordering::Less
        );
        assert_eq!(
            compare_files(&older, &newer, FileSortKey::Updated, true),
            std::cmp::Ordering::Greater
        );
    }

    #[test]
    fn group_files_by_category_should_keep_reference_category_order() {
        let groups = group_files_by_category(&[
            file("notes.md", "2026-05-02T08:00:00Z"),
            file("installer.pkg", "2026-05-02T08:00:00Z"),
            file("archive.zip", "2026-05-02T08:00:00Z"),
        ]);

        assert_eq!(groups.len(), 3);
        assert_eq!(groups[0].category, FileCategory::Installer);
        assert_eq!(groups[1].category, FileCategory::Archive);
        assert_eq!(groups[2].category, FileCategory::Document);
    }
}

addzero_admin_plugin_registry::register_admin_page! {
    id: DOWNLOAD_STATION_PAGE_ID,
    domain: KNOWLEDGE_DOMAIN_ID,
    parent: None,
    label: "下载站",
    order: 40,
    href: "/download-station",
    active_patterns: &["/download-station", "/files"],
    permissions_any_of: &["knowledge:dl"],
}
