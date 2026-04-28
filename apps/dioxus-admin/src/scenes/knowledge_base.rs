use dioxus::prelude::*;
use dioxus_components::{
    ContentHeader, Divider, ListItem, MetricRow, ResponsiveGrid, SidebarSection, Stack, StatTile,
    Surface, SurfaceHeader, Tone, WorkbenchButton,
};

use crate::{
    app::Route,
    knowledge_catalog::{
        KNOWLEDGE_DATA_MODE, KNOWLEDGE_DOCS, KNOWLEDGE_SOURCE_AVAILABLE,
        KNOWLEDGE_SOURCE_SUMMARIES, KnowledgeDoc, knowledge_doc, total_bytes, total_sections,
        total_sources,
    },
    package_catalog::{
        PACKAGE_CHANNELS, PackageAsset, first_package_asset_slug_for_channel, package_asset,
        package_asset_count, package_assets, package_channel, package_pending_checks,
        package_platform_count, total_package_assets, total_package_channels,
    },
};

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
        KnowledgeSectionTabs { active: "笔记" }
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
                Surface {
                    SurfaceHeader {
                        title: "笔记目录".to_string(),
                        subtitle: format!("已纳入 {} 份知识文档，覆盖 {} 个来源目录。", KNOWLEDGE_DOCS.len(), total_sources())
                    }
                    div { class: "knowledge-source",
                        span { class: "badge", "{data_mode_label()}" }
                        span { class: "badge badge--fs", "{KNOWLEDGE_DATA_MODE}" }
                    }
                    for summary in KNOWLEDGE_SOURCE_SUMMARIES.iter().filter(|summary| summary.count > 0) {
                        div { class: "knowledge-group",
                            div { class: "knowledge-group__label",
                                span { "{summary.label}" }
                                span { class: "knowledge-group__count", "{summary.count} 篇" }
                            }
                            div { class: "callout callout--info",
                                "{summary.root}"
                            }
                            for doc in KNOWLEDGE_DOCS.iter().filter(|doc| doc.source_slug == summary.slug) {
                                button {
                                    type: "button",
                                    class: if selected.map(|item| item.slug == doc.slug).unwrap_or(false) {
                                        "knowledge-doc knowledge-doc--active"
                                    } else {
                                        "knowledge-doc"
                                    },
                                    onclick: {
                                        let slug = doc.slug.to_string();
                                        move |_| selected_slug.set(slug.clone())
                                    },
                                    div { class: "knowledge-doc__eyebrow",
                                        span { class: "badge", "{doc.source_name}" }
                                        span { class: "knowledge-doc__file", "{doc.relative_path}" }
                                    }
                                    div { class: "knowledge-doc__title", "{doc.title}" }
                                    div { class: "knowledge-doc__preview", "{doc.preview}" }
                                    div { class: "knowledge-doc__meta",
                                        span { "{doc.section_count} 个章节标题" }
                                        span { "{format_bytes(doc.bytes)}" }
                                    }
                                }
                            }
                        }
                    }
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
        KnowledgeSectionTabs { active: "软件" }
        SoftwareScene {}
    }
}

#[component]
pub fn KnowledgePackages() -> Element {
    rsx! {
        KnowledgeSceneHeader {
            subtitle: "安装包资产先按平台、来源、分发格式和落地位置纳入目录，后续再补版本与校验链路。"
        }
        KnowledgeSectionTabs { active: "安装包" }
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
fn KnowledgeSectionTabs(active: &'static str) -> Element {
    let tab = |label: &'static str| -> Option<Tone> {
        if label == active {
            Some(Tone::Accent)
        } else {
            None
        }
    };

    rsx! {
        div { class: "knowledge-tabs",
            Link { to: Route::KnowledgeNotes,
                WorkbenchButton { class: "segment-button".to_string(), tone: tab("笔记"), "笔记" }
            }
            Link { to: Route::KnowledgeSoftware,
                WorkbenchButton { class: "segment-button".to_string(), tone: tab("软件"), "软件" }
            }
            Link { to: Route::KnowledgePackages,
                WorkbenchButton { class: "segment-button".to_string(), tone: tab("安装包"), "安装包" }
            }
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
                title: "软件位".to_string(),
                subtitle: "这部分继续保留给软件台账和安装记录，不和知识文档混排。".to_string()
            }
            ResponsiveGrid { columns: 3,
                StatTile { label: "已登记软件".to_string(), value: "42".to_string(), detail: "含 CLI / GUI / 服务".to_string() }
                StatTile { label: "待升级".to_string(), value: "6".to_string(), detail: "已超过安全基线".to_string() }
                StatTile { label: "高风险依赖".to_string(), value: "2".to_string(), detail: "等待兼容性验证".to_string() }
            }
        }
        Surface {
            SurfaceHeader {
                title: "软件台账".to_string(),
                subtitle: "按用途和运行环境做分组。".to_string()
            }
            table { class: "data-table",
                thead { tr { th { "软件" } th { "版本" } th { "用途" } th { "状态" } } }
                tbody {
                    tr { td { "Cursor" } td { "0.52.x" } td { "编码与代理协作" } td { "稳定" } }
                    tr { td { "Docker Desktop" } td { "4.40.x" } td { "本地容器编排" } td { "待升级" } }
                    tr { td { "cloudflared" } td { "2026.4" } td { "隧道与公网访问" } td { "稳定" } }
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
    let selected_channel_slug = use_signal(|| default_channel);
    let selected_asset_slug = use_signal(|| default_asset);

    let selected_channel =
        package_channel(selected_channel_slug.read().as_str()).or_else(|| PACKAGE_CHANNELS.first());
    let selected_assets = selected_channel
        .map(|channel| package_assets(channel.slug).collect::<Vec<_>>())
        .unwrap_or_default();
    let selected_asset = package_asset(selected_asset_slug.read().as_str())
        .filter(|asset| selected_channel.map(|channel| channel.slug) == Some(asset.channel_slug))
        .or_else(|| selected_assets.first().copied());

    rsx! {
        ResponsiveGrid { columns: 4,
            StatTile {
                label: "分发组".to_string(),
                value: total_package_channels().to_string(),
                detail: "把桌面安装器、CLI 二进制和知识附件包拆开管理。".to_string()
            }
            StatTile {
                label: "安装包资产".to_string(),
                value: total_package_assets().to_string(),
                detail: "软件对象和可分发文件分开建模，避免把“软件”和“包”混成一层。".to_string()
            }
            StatTile {
                label: "平台覆盖".to_string(),
                value: package_platform_count().to_string(),
                detail: "显式保留平台与架构，避免下载链接和目标环境脱节。".to_string()
            }
            StatTile {
                label: "待补校验".to_string(),
                value: package_pending_checks().to_string(),
                detail: "优先补 checksum、来源和回滚版本，再考虑自动同步。".to_string()
            }
        }
        Surface {
            SurfaceHeader {
                title: "入湖约定".to_string(),
                subtitle: "先把安装包目录做清楚，再补镜像、同步和版本治理。".to_string()
            }
            Stack {
                div { class: "callout callout--info",
                    "安装包和软件对象分开记：软件是概念与用途，安装包是某个平台上的可分发物。"
                }
                div { class: "callout",
                    "每个安装包至少保留版本、平台、来源、格式、目标落地位置和 checksum 状态。"
                }
                div { class: "callout",
                    "先做“人能看懂的目录”，后面再让 agent 接镜像缓存、自动下载和分发校验。"
                }
            }
        }
        div { class: "knowledge-board",
            Surface {
                SurfaceHeader {
                    title: "分发组".to_string(),
                    subtitle: "按分发形态和用途分组，不把桌面安装器、CLI 包和知识附件混成一张表。".to_string()
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
                            let channel_slug = channel.slug.to_string();
                            let next_asset = package_assets(channel.slug)
                                .next()
                                .map(|asset| asset.slug.to_string())
                                .unwrap_or_default();
                            move |_| {
                                selected_channel_slug.set(channel_slug.clone());
                                selected_asset_slug.set(next_asset.clone());
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
                        title: channel.title.to_string(),
                        subtitle: channel.description.to_string()
                    }
                    div { class: "knowledge-source",
                        span { class: "badge badge--fs", "{channel.distribution}" }
                        span { class: "badge", "{package_asset_count(channel.slug)} 个安装包" }
                        span { class: "badge", "Package catalog" }
                    }
                    div { class: "callout callout--info",
                        "分组规则："
                        "{channel.rule}"
                    }
                    table { class: "data-table",
                        thead {
                            tr {
                                th { "软件对象" }
                                th { "安装包" }
                                th { "版本" }
                                th { "平台" }
                            }
                        }
                        tbody {
                            for asset in selected_assets.iter() {
                                PackageAssetRow {
                                    asset: **asset,
                                    active: selected_asset.map(|item| item.slug == asset.slug).unwrap_or(false),
                                    on_select: {
                                        let mut selected_asset_slug = selected_asset_slug;
                                        let slug = asset.slug.to_string();
                                        move || selected_asset_slug.set(slug.clone())
                                    }
                                }
                            }
                        }
                    }
                    Divider {}
                    if let Some(asset) = selected_asset {
                        PackageAssetDetail { asset: *asset }
                    }
                }
            }
        }
        Surface {
            SurfaceHeader {
                title: "后续补齐".to_string(),
                subtitle: "安装包目录先稳定，之后再接下载、镜像和自动分发能力。".to_string()
            }
            Stack {
                ListItem {
                    title: "阶段 1 · 目录与元数据".to_string(),
                    detail: "把来源、版本、checksum、平台、落地位置和关联软件对象记完整，先形成可靠目录。".to_string(),
                    meta: "当前".to_string()
                }
                ListItem {
                    title: "阶段 2 · 镜像与校验".to_string(),
                    detail: "补校验和、来源镜像、历史版本和过期包规则，避免知识库里只有名字没有分发事实。".to_string(),
                    meta: "后续".to_string()
                }
                ListItem {
                    title: "阶段 3 · Agent 下载与分发".to_string(),
                    detail: "Win/mac agent 再接下载、缓存、验证和落地流程，把“看到包”推进到“可自动分发”。".to_string(),
                    meta: "再后面".to_string()
                }
            }
        }
    }
}

#[component]
fn PackageAssetRow(asset: PackageAsset, active: bool, on_select: EventHandler<()>) -> Element {
    rsx! {
        tr {
            class: if active {
                "row-link config-row config-row--active"
            } else {
                "row-link config-row"
            },
            onclick: move |_| on_select.call(()),
            td {
                div { class: "stack",
                    strong { "{asset.software_title}" }
                    span { class: "cell-overflow", "{asset.package_name}" }
                }
            }
            td { "{asset.package_name}" }
            td { "{asset.version}" }
            td { "{asset.platform}" }
        }
    }
}

#[component]
fn PackageAssetDetail(asset: PackageAsset) -> Element {
    rsx! {
        div { class: "knowledge-detail",
            div { class: "knowledge-meta",
                span { class: "badge badge--fs", "{asset.status}" }
                span { class: "badge badge--fs", "{asset.version}" }
                span { class: "badge", "{asset.format}" }
                span { class: "badge", "{asset.platform}" }
                span { class: "badge", "{asset.checksum_state}" }
            }
            div { class: "callout callout--info",
                "来源："
                "{asset.source}"
            }
            div { class: "callout",
                "落地位置："
                "{asset.install_target}"
            }
            div { class: "callout",
                "关联关系："
                "{asset.relation}"
            }
            div { class: "callout",
                "备注："
                "{asset.note}"
            }
        }
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
