use dioxus::prelude::*;
use dioxus_components::{
    ContentHeader, Field, GroupedListPanel, GroupedListPanelGroup, GroupedListPanelItem, ListItem,
    MetricRow, ResponsiveGrid, SidebarSection, Stack, StatTile, Surface, SurfaceHeader, Tone,
    WorkbenchButton,
};

use crate::{
    knowledge_catalog::{
        KNOWLEDGE_DATA_MODE, KNOWLEDGE_DOCS, KNOWLEDGE_SOURCE_AVAILABLE,
        KNOWLEDGE_SOURCE_SUMMARIES, KnowledgeDoc, knowledge_doc, total_bytes, total_sections,
        total_sources,
    },
    package_catalog::{
        PACKAGE_CHANNELS, PackageAsset, first_package_asset_slug_for_channel, package_asset,
        package_asset_count, package_assets, package_channel,
    },
};

#[derive(Clone, Debug, PartialEq, Eq)]
struct PackageAssetDraft {
    slug: String,
    channel_slug: String,
    software_title: String,
    package_name: String,
    version: String,
    platform: String,
    format: String,
    status: String,
    source: String,
    install_target: String,
    checksum_state: String,
    relation: String,
    note: String,
}

impl PackageAssetDraft {
    fn from_asset(asset: PackageAsset) -> Self {
        Self {
            slug: asset.slug.to_string(),
            channel_slug: asset.channel_slug.to_string(),
            software_title: asset.software_title.to_string(),
            package_name: asset.package_name.to_string(),
            version: asset.version.to_string(),
            platform: asset.platform.to_string(),
            format: asset.format.to_string(),
            status: asset.status.to_string(),
            source: asset.source.to_string(),
            install_target: asset.install_target.to_string(),
            checksum_state: asset.checksum_state.to_string(),
            relation: asset.relation.to_string(),
            note: asset.note.to_string(),
        }
    }

    fn empty(channel_slug: &str) -> Self {
        Self {
            slug: String::new(),
            channel_slug: channel_slug.to_string(),
            software_title: String::new(),
            package_name: String::new(),
            version: String::new(),
            platform: String::new(),
            format: String::new(),
            status: "整理中".to_string(),
            source: String::new(),
            install_target: String::new(),
            checksum_state: "待补 SHA256".to_string(),
            relation: String::new(),
            note: String::new(),
        }
    }

    fn title(&self) -> String {
        if self.software_title.trim().is_empty() {
            "未命名安装包".to_string()
        } else {
            self.software_title.clone()
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum PackageCrudMode {
    View,
    Create,
    Update,
}

#[component]
pub fn KnowledgeNotes() -> Element {
    let mut selected_slug = use_signal(|| {
        KNOWLEDGE_DOCS
            .first()
            .map(|doc| doc.slug.to_string())
            .unwrap_or_default()
    });
    let selected = knowledge_doc(selected_slug.read().as_str()).or_else(|| KNOWLEDGE_DOCS.first());

    rsx! {
        KnowledgeSceneHeader {
            subtitle: "当前笔记目录优先从 PostgreSQL 镜像生成，缺省会扫描本机候选知识目录后再渲染后台目录。"
        }
        KnowledgeSummary {}
        if KNOWLEDGE_DOCS.is_empty() {
            Surface {
                SurfaceHeader {
                    title: "笔记目录为空".to_string(),
                    subtitle: "还没有在可识别的知识目录里找到可导入文档。".to_string()
                }
                div { class: "stack content-stack",
                    for source in KNOWLEDGE_SOURCE_SUMMARIES.iter() {
                        div { class: "callout",
                            "{source.label}: "
                            "{source.root}"
                        }
                    }
                }
            }
        } else {
            div { class: "knowledge-board",
                GroupedListPanel {
                    title: "笔记目录".to_string(),
                    subtitle: format!("已纳入 {} 份知识文档，覆盖 {} 个来源目录。", KNOWLEDGE_DOCS.len(), total_sources()),
                    children: rsx!(
                        div { class: "knowledge-source",
                            span { class: "badge", "{data_mode_label()}" }
                            span { class: "badge badge--fs", "{KNOWLEDGE_DATA_MODE}" }
                        }
                    ),
                    groups: KNOWLEDGE_SOURCE_SUMMARIES
                        .iter()
                        .filter(|summary| summary.count > 0)
                        .map(|summary| GroupedListPanelGroup {
                            label: summary.label.to_string(),
                            count_label: Some(format!("{} 篇", summary.count)),
                            description: Some(summary.root.to_string()),
                            items: KNOWLEDGE_DOCS
                                .iter()
                                .filter(|doc| doc.source_slug == summary.slug)
                                .map(|doc| {
                                    let slug = doc.slug.to_string();
                                    GroupedListPanelItem {
                                        key: slug.clone(),
                                        title: doc.title.to_string(),
                                        eyebrow: Some(doc.source_name.to_string()),
                                        preview: Some(doc.preview.to_string()),
                                        meta: vec![
                                            doc.relative_path.to_string(),
                                            format!("{} 个章节标题", doc.section_count),
                                            format_bytes(doc.bytes),
                                        ],
                                        active: selected.map(|item| item.slug == doc.slug).unwrap_or(false),
                                        onpress: EventHandler::new(move |_| selected_slug.set(slug.clone())),
                                    }
                                })
                                .collect(),
                        })
                        .collect()
                }
                if let Some(doc) = selected {
                    KnowledgeDetailSurface { doc: *doc }
                }
            }
        }
    }
}

#[component]
pub fn KnowledgeSoftware() -> Element {
    rsx! {
        KnowledgeSceneHeader {
            subtitle: "软件资产和知识文档先拆成独立子场景，避免一开始把台账和笔记混在一起。"
        }
        SoftwareScene {}
    }
}

#[component]
pub fn KnowledgePackages() -> Element {
    rsx! {
        KnowledgeSceneHeader {
            subtitle: "安装包资产聚焦归档、校验与安装目标；文件浏览与 recent outputs 统一收敛到 /files，不再在这里重复承载下载列表。"
        }
        PackageAssetsScene {}
    }
}

#[component]
fn KnowledgeSceneHeader(subtitle: &'static str) -> Element {
    rsx! {
        ContentHeader {
            title: "知识库".to_string(),
            subtitle: subtitle.to_string()
        }
    }
}

#[component]
fn KnowledgeSummary() -> Element {
    rsx! {
        ResponsiveGrid { columns: 3,
            StatTile {
                label: "已纳入文档".to_string(),
                value: KNOWLEDGE_DOCS.len().to_string(),
                detail: "当前全部挂在“笔记”子场景下。".to_string()
            }
            StatTile {
                label: "来源目录".to_string(),
                value: total_sources().to_string(),
                detail: "每个来源目录都单独分组，避免不同资产混在一起。".to_string()
            }
            StatTile {
                label: "章节小节".to_string(),
                value: total_sections().to_string(),
                detail: format!("当前目录快照体量约 {}。", format_bytes(total_bytes()))
            }
        }
    }
}

#[component]
fn KnowledgeDetailSurface(doc: KnowledgeDoc) -> Element {
    rsx! {
        Surface {
            SurfaceHeader {
                title: doc.title.to_string(),
                subtitle: format!("{} · {} · {}", doc.source_name, doc.relative_path, format_bytes(doc.bytes))
            }
            div { class: "knowledge-detail",
                div { class: "knowledge-meta",
                    span { class: "badge badge--fs", "{doc.source_name}" }
                    span { class: "badge", "{doc.filename}" }
                    span { class: "badge", "{doc.section_count} 个章节标题" }
                }
                div { class: "callout callout--info",
                    "源文件路径："
                    "{doc.source_path}"
                }
                div { class: "callout",
                    "来源目录："
                    "{doc.source_root}"
                }
                if !doc.headings.is_empty() {
                    div { class: "knowledge-outline",
                        div { class: "knowledge-detail__label", "章节提要" }
                        for heading in doc.headings.iter() {
                            div { class: "knowledge-outline__item", "{heading}" }
                        }
                    }
                }
                div { class: "knowledge-detail__label", "内容摘录" }
                div { class: "knowledge-excerpt", "{doc.excerpt}" }
            }
        }
    }
}

#[component]
pub fn KnowledgeContext() -> Element {
    rsx! {
        SidebarSection { label: "知识库".to_string(),
            Stack {
                MetricRow { label: "数据源".to_string(), value: if KNOWLEDGE_SOURCE_AVAILABLE { data_mode_label().to_string() } else { "Missing".to_string() }, tone: if KNOWLEDGE_SOURCE_AVAILABLE { Tone::Positive } else { Tone::Warning } }
                MetricRow { label: "同步模式".to_string(), value: KNOWLEDGE_DATA_MODE.to_string() }
                MetricRow { label: "来源数".to_string(), value: total_sources().to_string() }
                MetricRow { label: "文档数".to_string(), value: KNOWLEDGE_DOCS.len().to_string() }
                MetricRow { label: "体量".to_string(), value: format_bytes(total_bytes()) }
            }
        }
        SidebarSection { label: "来源目录".to_string(),
            Stack {
                for summary in KNOWLEDGE_SOURCE_SUMMARIES.iter().filter(|summary| summary.count > 0) {
                    MetricRow { label: summary.label.to_string(), value: summary.count.to_string() }
                }
            }
        }
        SidebarSection { label: "根目录".to_string(),
            Stack {
                for summary in KNOWLEDGE_SOURCE_SUMMARIES.iter().filter(|summary| summary.count > 0) {
                    div { class: "callout callout--info",
                        "{summary.label}: "
                        "{summary.root}"
                    }
                }
            }
        }
    }
}

fn data_mode_label() -> &'static str {
    if KNOWLEDGE_DATA_MODE == "postgres-sync" {
        "Postgres"
    } else {
        "Filesystem"
    }
}

#[component]
pub fn SoftwareScene() -> Element {
    rsx! {
        Surface {
            SurfaceHeader {
                title: "软件矩阵".to_string(),
                subtitle: "软件对象按工作块组织，直接看对象、用途、依赖面和当前状态。".to_string()
            }
            div { class: "software-matrix",
                SoftwareGroup {
                    title: "创作与笔记".to_string(),
                    items: vec![
                        ("Cursor", "编码与 Agent 协作", "桌面 IDE", "稳定"),
                        ("Obsidian", "笔记编辑与知识沉淀", "桌面应用", "稳定"),
                        ("Raycast", "命令入口与启动器", "桌面工具", "观察中"),
                    ]
                }
                SoftwareGroup {
                    title: "构建与运行".to_string(),
                    items: vec![
                        ("Docker Desktop", "本地容器编排", "容器运行时", "待升级"),
                        ("Rust Toolchain", "Rust 构建与测试", "CLI 工具链", "稳定"),
                        ("Node.js", "前端构建与脚本运行", "CLI 运行时", "待升级"),
                    ]
                }
                SoftwareGroup {
                    title: "网络与分发".to_string(),
                    items: vec![
                        ("cloudflared", "隧道与公网访问", "CLI 服务", "稳定"),
                        ("GitHub", "代码托管与发布", "云端服务", "稳定"),
                        ("MinIO", "对象存储与品牌资源", "基础设施", "整理中"),
                    ]
                }
            }
        }
    }
}

#[component]
pub fn PackageAssetsScene() -> Element {
    let default_channel = PACKAGE_CHANNELS
        .first()
        .map(|channel| channel.slug.to_string())
        .unwrap_or_default();
    let default_asset = PACKAGE_CHANNELS
        .first()
        .and_then(|channel| first_package_asset_slug_for_channel(channel.slug))
        .map(str::to_string)
        .unwrap_or_default();
    let initial_assets = PACKAGE_CHANNELS
        .iter()
        .flat_map(|channel| package_assets(channel.slug))
        .map(|asset| PackageAssetDraft::from_asset(*asset))
        .collect::<Vec<_>>();
    let selected_channel_slug = use_signal(|| default_channel.clone());
    let selected_asset_slug = use_signal(|| default_asset.clone());
    let package_assets_state = use_signal(|| initial_assets);
    let editor_mode = use_signal(|| PackageCrudMode::View);
    let form_state = use_signal(|| {
        package_asset(default_asset.as_str())
            .map(|asset| PackageAssetDraft::from_asset(*asset))
            .unwrap_or_else(|| PackageAssetDraft::empty(""))
    });

    let selected_channel =
        package_channel(selected_channel_slug.read().as_str()).or_else(|| PACKAGE_CHANNELS.first());
    let selected_assets = selected_channel
        .map(|channel| {
            package_assets_state
                .read()
                .iter()
                .filter(|asset| asset.channel_slug == channel.slug)
                .cloned()
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    let selected_asset = selected_assets
        .iter()
        .find(|asset| asset.slug == *selected_asset_slug.read())
        .cloned()
        .or_else(|| selected_assets.first().cloned());
    let mode = *editor_mode.read();
    let form = form_state.read().clone();

    rsx! {
        div { class: "package-workbench",
            Surface {
                SurfaceHeader {
                    title: "分发组".to_string(),
                    subtitle: "先选工作面，再处理当前组内的安装包对象。".to_string()
                }
                for channel in PACKAGE_CHANNELS.iter() {
                    button {
                        r#type: "button",
                        class: if selected_channel.map(|item| item.slug == channel.slug).unwrap_or(false) {
                            "config-space config-space--active"
                        } else {
                            "config-space"
                        },
                        onclick: {
                            let mut selected_channel_slug = selected_channel_slug;
                            let mut selected_asset_slug = selected_asset_slug;
                            let mut editor_mode = editor_mode;
                            let mut form_state = form_state;
                            let channel_slug = channel.slug.to_string();
                            let next_asset = package_assets_state
                                .read()
                                .iter()
                                .find(|asset| asset.channel_slug == channel.slug)
                                .map(|asset| asset.slug.clone())
                                .unwrap_or_default();
                            move |_| {
                                selected_channel_slug.set(channel_slug.clone());
                                selected_asset_slug.set(next_asset.clone());
                                editor_mode.set(PackageCrudMode::View);
                                if let Some(asset) = package_assets_state
                                    .read()
                                    .iter()
                                    .find(|asset| asset.slug == next_asset)
                                    .cloned()
                                {
                                    form_state.set(asset);
                                } else {
                                    form_state.set(PackageAssetDraft::empty(&channel_slug));
                                }
                            }
                        },
                        div { class: "config-space__eyebrow",
                            span { class: "badge badge--fs", "{channel.distribution}" }
                            span { class: "badge", "{package_asset_count(channel.slug)} 项" }
                        }
                        div { class: "config-space__title", "{channel.title}" }
                        div { class: "config-space__copy", "{channel.description}" }
                        div { class: "config-space__meta", "{channel.rule}" }
                    }
                }
            }
            if let Some(channel) = selected_channel {
                Surface {
                    SurfaceHeader {
                        title: "安装包对象".to_string(),
                        subtitle: "安装包对象只维护归档元数据与安装目标；文件浏览、下载记录和 recent outputs 统一走 /files。".to_string()
                    }
                    div { class: "package-list",
                        for asset in selected_assets.iter() {
                            button {
                                r#type: "button",
                                class: if selected_asset.as_ref().map(|item| item.slug == asset.slug).unwrap_or(false) {
                                    "package-list__item package-list__item--active"
                                } else {
                                    "package-list__item"
                                },
                                onclick: {
                                    let mut selected_asset_slug = selected_asset_slug;
                                    let mut editor_mode = editor_mode;
                                    let mut form_state = form_state;
                                    let asset = asset.clone();
                                    move |_| {
                                        selected_asset_slug.set(asset.slug.clone());
                                        editor_mode.set(PackageCrudMode::View);
                                        form_state.set(asset.clone());
                                    }
                                },
                                div { class: "package-list__title", "{asset.software_title}" }
                                div { class: "package-list__meta",
                                    span { "{asset.version}" }
                                    span { "·" }
                                    span { "{asset.platform}" }
                                }
                                div { class: "package-list__copy", "{asset.package_name}" }
                            }
                        }
                    }
                }
                Surface {
                    SurfaceHeader {
                        title: package_editor_title(mode, &form, channel.title),
                        subtitle: channel.rule.to_string()
                    }
                    div { class: "package-editor__actions",
                        WorkbenchButton {
                            class: "action-button action-button--primary".to_string(),
                            onclick: {
                                let mut editor_mode = editor_mode;
                                let mut form_state = form_state;
                                let channel_slug = channel.slug.to_string();
                                move |_| {
                                    editor_mode.set(PackageCrudMode::Create);
                                    form_state.set(PackageAssetDraft::empty(&channel_slug));
                                }
                            },
                            "新增"
                        }
                        if let Some(asset) = selected_asset.clone() {
                            WorkbenchButton {
                                class: "action-button".to_string(),
                                onclick: {
                                    let mut editor_mode = editor_mode;
                                    let mut form_state = form_state;
                                    let edit_asset = asset.clone();
                                    move |_| {
                                        editor_mode.set(PackageCrudMode::Update);
                                        form_state.set(edit_asset.clone());
                                    }
                                },
                                "编辑"
                            }
                            WorkbenchButton {
                                class: "action-button".to_string(),
                                onclick: {
                                    let mut package_assets_state = package_assets_state;
                                    let mut selected_asset_slug = selected_asset_slug;
                                    let mut editor_mode = editor_mode;
                                    let mut form_state = form_state;
                                    let asset_slug = asset.slug.clone();
                                    let channel_slug = channel.slug.to_string();
                                    move |_| {
                                        package_assets_state.with_mut(|items| {
                                            items.retain(|item| item.slug != asset_slug);
                                        });
                                        let next_slug = package_assets_state
                                            .read()
                                            .iter()
                                            .find(|item| item.channel_slug == channel_slug)
                                            .map(|item| item.slug.clone())
                                            .unwrap_or_default();
                                        selected_asset_slug.set(next_slug.clone());
                                        editor_mode.set(PackageCrudMode::View);
                                        if let Some(next) = package_assets_state
                                            .read()
                                            .iter()
                                            .find(|item| item.slug == next_slug)
                                            .cloned()
                                        {
                                            form_state.set(next);
                                        } else {
                                            form_state.set(PackageAssetDraft::empty(&channel_slug));
                                        }
                                    }
                                },
                                "删除"
                            }
                        }
                    }
                    div { class: "package-editor" ,
                        ResponsiveGrid { columns: 2,
                            Field {
                                label: "软件对象".to_string(),
                                value: form.software_title.clone(),
                                placeholder: Some("例如 Cursor".to_string()),
                                on_input: {
                                    let mut form_state = form_state;
                                    move |value| {
                                        let mut next = form_state.read().clone();
                                        next.software_title = value;
                                        form_state.set(next);
                                    }
                                }
                            }
                            Field {
                                label: "安装包文件".to_string(),
                                value: form.package_name.clone(),
                                placeholder: Some("例如 cursor-macos-universal.dmg".to_string()),
                                on_input: {
                                    let mut form_state = form_state;
                                    move |value| {
                                        let mut next = form_state.read().clone();
                                        next.package_name = value;
                                        form_state.set(next);
                                    }
                                }
                            }
                            Field {
                                label: "版本".to_string(),
                                value: form.version.clone(),
                                on_input: {
                                    let mut form_state = form_state;
                                    move |value| {
                                        let mut next = form_state.read().clone();
                                        next.version = value;
                                        form_state.set(next);
                                    }
                                }
                            }
                            Field {
                                label: "平台".to_string(),
                                value: form.platform.clone(),
                                on_input: {
                                    let mut form_state = form_state;
                                    move |value| {
                                        let mut next = form_state.read().clone();
                                        next.platform = value;
                                        form_state.set(next);
                                    }
                                }
                            }
                            Field {
                                label: "格式".to_string(),
                                value: form.format.clone(),
                                on_input: {
                                    let mut form_state = form_state;
                                    move |value| {
                                        let mut next = form_state.read().clone();
                                        next.format = value;
                                        form_state.set(next);
                                    }
                                }
                            }
                            Field {
                                label: "状态".to_string(),
                                value: form.status.clone(),
                                on_input: {
                                    let mut form_state = form_state;
                                    move |value| {
                                        let mut next = form_state.read().clone();
                                        next.status = value;
                                        form_state.set(next);
                                    }
                                }
                            }
                            Field {
                                label: "来源".to_string(),
                                value: form.source.clone(),
                                on_input: {
                                    let mut form_state = form_state;
                                    move |value| {
                                        let mut next = form_state.read().clone();
                                        next.source = value;
                                        form_state.set(next);
                                    }
                                }
                            }
                            Field {
                                label: "落地位置".to_string(),
                                value: form.install_target.clone(),
                                on_input: {
                                    let mut form_state = form_state;
                                    move |value| {
                                        let mut next = form_state.read().clone();
                                        next.install_target = value;
                                        form_state.set(next);
                                    }
                                }
                            }
                            Field {
                                label: "校验状态".to_string(),
                                value: form.checksum_state.clone(),
                                on_input: {
                                    let mut form_state = form_state;
                                    move |value| {
                                        let mut next = form_state.read().clone();
                                        next.checksum_state = value;
                                        form_state.set(next);
                                    }
                                }
                            }
                            Field {
                                label: "关联关系".to_string(),
                                value: form.relation.clone(),
                                on_input: {
                                    let mut form_state = form_state;
                                    move |value| {
                                        let mut next = form_state.read().clone();
                                        next.relation = value;
                                        form_state.set(next);
                                    }
                                }
                            }
                        }
                        Field {
                            label: "备注".to_string(),
                            value: form.note.clone(),
                            on_input: {
                                let mut form_state = form_state;
                                move |value| {
                                    let mut next = form_state.read().clone();
                                    next.note = value;
                                    form_state.set(next);
                                }
                            }
                        }
                        div { class: "package-editor__footer",
                            WorkbenchButton {
                                class: "action-button action-button--primary".to_string(),
                                onclick: {
                                    let mut package_assets_state = package_assets_state;
                                    let mut selected_asset_slug = selected_asset_slug;
                                    let mut editor_mode = editor_mode;
                                    let channel_slug = channel.slug.to_string();
                                    move |_| {
                                        let mut draft = form_state.read().clone();
                                        draft.channel_slug = channel_slug.clone();
                                        if draft.slug.trim().is_empty() {
                                            draft.slug = format!(
                                                "{}-{}",
                                                channel_slug,
                                                draft.package_name
                                                    .to_lowercase()
                                                    .replace([' ', '/'], "-")
                                            );
                                        }
                                        package_assets_state.with_mut(|items| {
                                            if let Some(existing) = items.iter_mut().find(|item| item.slug == draft.slug) {
                                                *existing = draft.clone();
                                            } else {
                                                items.push(draft.clone());
                                            }
                                        });
                                        selected_asset_slug.set(draft.slug.clone());
                                        editor_mode.set(PackageCrudMode::View);
                                    }
                                },
                                if matches!(mode, PackageCrudMode::Create) { "创建安装包" } else { "保存修改" }
                            }
                            if !matches!(mode, PackageCrudMode::View) {
                                WorkbenchButton {
                                    class: "action-button".to_string(),
                                    onclick: {
                                        let mut editor_mode = editor_mode;
                                        let mut form_state = form_state;
                                        let selected_asset = selected_asset.clone();
                                        let channel_slug = channel.slug.to_string();
                                        move |_| {
                                            editor_mode.set(PackageCrudMode::View);
                                            if let Some(asset) = selected_asset.clone() {
                                                form_state.set(asset);
                                            } else {
                                                form_state.set(PackageAssetDraft::empty(&channel_slug));
                                            }
                                        }
                                    },
                                    "取消"
                                }
                            }
                        }
                    }
                }
                Surface {
                    SurfaceHeader {
                        title: "操作摘要".to_string(),
                        subtitle: "只保留当前对象做增删改查时最需要的判断信息。".to_string()
                    }
                    Stack {
                        ListItem {
                            title: "当前分发组".to_string(),
                            detail: channel.description.to_string(),
                            meta: channel.distribution.to_string()
                        }
                        ListItem {
                            title: "当前对象".to_string(),
                            detail: selected_asset
                                .as_ref()
                                .map(|asset| asset.package_name.clone())
                                .unwrap_or_else(|| "还没有选中安装包对象".to_string()),
                            meta: selected_asset
                                .as_ref()
                                .map(|asset| asset.status.clone())
                                .unwrap_or_else(|| "空".to_string())
                        }
                        ListItem {
                            title: "文件能力边界".to_string(),
                            detail: "文件浏览、下载记录与 recent outputs 已约定统一收敛到 /files；当前这里仅保留安装包资产台账。".to_string(),
                            meta: "Files".to_string()
                        }
                        ListItem {
                            title: "当前动作".to_string(),
                            detail: match mode {
                                PackageCrudMode::View => "查看当前对象并准备下一步动作".to_string(),
                                PackageCrudMode::Create => "录入新的安装包对象，保存后回到列表".to_string(),
                                PackageCrudMode::Update => "在当前上下文里直接修改对象字段".to_string(),
                            },
                            meta: match mode {
                                PackageCrudMode::View => "Read",
                                PackageCrudMode::Create => "Create",
                                PackageCrudMode::Update => "Update",
                            }
                            .to_string()
                        }
                    }
                }
            }
        }
    }
}

#[derive(Props, Clone, PartialEq)]
struct SoftwareGroupProps {
    title: String,
    items: Vec<(&'static str, &'static str, &'static str, &'static str)>,
}

#[component]
fn SoftwareGroup(props: SoftwareGroupProps) -> Element {
    rsx! {
        div { class: "software-group",
            div { class: "software-group__title", "{props.title}" }
            div { class: "software-group__grid",
                for (name, usage, kind, status) in props.items {
                    div { class: "software-tile",
                        div { class: "software-tile__head",
                            strong { "{name}" }
                            span { class: "badge badge--fs", "{status}" }
                        }
                        div { class: "software-tile__usage", "{usage}" }
                        div { class: "software-tile__meta", "{kind}" }
                    }
                }
            }
        }
    }
}

fn package_editor_title(
    mode: PackageCrudMode,
    draft: &PackageAssetDraft,
    channel_title: &str,
) -> String {
    match mode {
        PackageCrudMode::View => format!("{} · 当前对象", channel_title),
        PackageCrudMode::Create => format!("{} · 新增安装包", channel_title),
        PackageCrudMode::Update => format!("{} · 编辑 {}", channel_title, draft.title()),
    }
}

fn format_bytes(bytes: usize) -> String {
    if bytes >= 1_048_576 {
        format!("{:.1} MB", bytes as f64 / 1_048_576.0)
    } else if bytes >= 1_024 {
        format!("{:.0} KB", bytes as f64 / 1_024.0)
    } else {
        format!("{bytes} B")
    }
}
