use chrono::Local;
use dioxus::prelude::*;
use dioxus_components::{
    ContentHeader, KeywordChips, ListItem, MetricRow, SidebarSection, Stack, Surface,
    SurfaceHeader, Tone, WorkbenchButton,
};
use dioxus_nox_markdown::{
    markdown,
    prelude::{MarkdownHandle, Mode, use_markdown_handle},
};

use crate::{
    knowledge_catalog::{KNOWLEDGE_DOCS, KnowledgeDoc, total_bytes, total_sections},
    package_catalog::{PACKAGE_ASSETS, PackageAsset},
};

const DASHBOARD_SEARCH_ID: &str = "dashboard-note-search";

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
enum DashboardLens {
    Notes,
    Software,
    Packages,
}

impl DashboardLens {
    const ALL: [Self; 3] = [Self::Notes, Self::Software, Self::Packages];

    fn label(self) -> &'static str {
        match self {
            Self::Notes => "笔记",
            Self::Software => "软件",
            Self::Packages => "安装包",
        }
    }

    fn entry_kind(self) -> KnowledgeEntryKind {
        match self {
            Self::Notes => KnowledgeEntryKind::Note,
            Self::Software => KnowledgeEntryKind::Software,
            Self::Packages => KnowledgeEntryKind::Package,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum KnowledgeEntryKind {
    Note,
    Software,
    Package,
}

impl KnowledgeEntryKind {
    const ALL: [Self; 3] = [Self::Note, Self::Software, Self::Package];

    fn label(self) -> &'static str {
        match self {
            Self::Note => "笔记",
            Self::Software => "软件",
            Self::Package => "安装包",
        }
    }

    fn default_source(self) -> &'static str {
        match self {
            Self::Note => "阅读 / 对话 / 研究",
            Self::Software => "巡检 / 运行 / 集成",
            Self::Package => "下载 / 发布 / 归档",
        }
    }

    fn lens(self) -> DashboardLens {
        match self {
            Self::Note => DashboardLens::Notes,
            Self::Software => DashboardLens::Software,
            Self::Package => DashboardLens::Packages,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct KnowledgeEntryRecord {
    title: String,
    body: String,
    source: String,
    tags: Vec<String>,
    kind: KnowledgeEntryKind,
    captured_at: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct EntryFormState {
    body: String,
    tags: Vec<String>,
    kind: KnowledgeEntryKind,
}

impl EntryFormState {
    fn for_capture(kind: KnowledgeEntryKind) -> Self {
        Self {
            body: String::new(),
            tags: vec![kind.label().to_string()],
            kind,
        }
    }

    fn cleaned_tags(&self) -> Vec<String> {
        self.tags
            .iter()
            .map(|tag| tag.trim())
            .filter(|tag| !tag.is_empty())
            .map(ToOwned::to_owned)
            .collect()
    }
}

#[derive(Clone, Debug)]
struct NoteCluster {
    label: &'static str,
    docs: Vec<&'static KnowledgeDoc>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct SoftwareCard {
    title: &'static str,
    source: &'static str,
    body: &'static str,
    tags: &'static [&'static str],
}

#[component]
pub fn Dashboard() -> Element {
    let mut active_lens = use_signal(|| DashboardLens::Notes);
    let mut search = use_signal(String::new);
    let mut entry_log = use_signal(seed_entry_log);
    let mut entry_form =
        use_signal(|| EntryFormState::for_capture(DashboardLens::Notes.entry_kind()));

    let lens = *active_lens.read();
    let query = search.read().clone();
    let note_cards = filtered_note_docs(&query);
    let software_cards = filtered_software_cards(&query);
    let package_cards = filtered_package_cards(&query);
    let result_count = match lens {
        DashboardLens::Notes => note_cards.len(),
        DashboardLens::Software => software_cards.len(),
        DashboardLens::Packages => package_cards.len(),
    };
    let form = entry_form.read().clone();
    let queue_preview = entry_log.read().iter().take(6).cloned().collect::<Vec<_>>();

    rsx! {
        ContentHeader {
            title: "闪念".to_string(),
            subtitle: "不再展示知识图谱，默认以对齐的笔记卡片流浏览和检索。".to_string(),
            actions: rsx!(
                div { class: "shortcut-hint", "Cmd K" }
            )
        }
        div { class: "lens-strip",
            for item in DashboardLens::ALL {
                WorkbenchButton {
                    class: "lens-pill".to_string(),
                    tone: if item == lens { Some(Tone::Accent) } else { None },
                    onclick: move |_| {
                        active_lens.set(item);
                        search.set(String::new());
                        entry_form.set(EntryFormState::for_capture(item.entry_kind()));
                    },
                    "{item.label()}"
                }
            }
        }
        Surface {
            SurfaceHeader {
                title: format!("{}卡片", lens.label()),
                subtitle: format!("当前筛选出 {result_count} 条，按卡片网格对齐展示。"),
                actions: rsx!(
                    div { class: "graph-toolbar note-toolbar",
                        input {
                            id: DASHBOARD_SEARCH_ID,
                            "data-command-search": "true",
                            class: "graph-search note-search",
                            r#type: "search",
                            value: "{search}",
                            placeholder: "搜索标题、正文、标签或来源",
                            oninput: move |evt| search.set(evt.value())
                        }
                    }
                )
            }
            div { class: "note-feed-layout",
                div { class: "note-feed-main",
                    NoteCaptureEditor {
                        value: form.body.clone(),
                        on_change: move |value| {
                            let mut next = entry_form.read().clone();
                            next.body = value;
                            entry_form.set(next);
                        }
                    }
                    div { class: "note-feed-meta",
                        span { class: "badge badge--fs", "{lens.label()}" }
                        span { "{result_count} 条结果" }
                        if !query.trim().is_empty() {
                            span { class: "note-feed-meta__query", "搜索：{query}" }
                        }
                    }
                    match lens {
                        DashboardLens::Notes => rsx! {
                            NoteCardGrid { docs: note_cards }
                        },
                        DashboardLens::Software => rsx! {
                            SoftwareCardGrid { cards: software_cards }
                        },
                        DashboardLens::Packages => rsx! {
                            PackageCardGrid { cards: package_cards }
                        },
                    }
                }
                div { class: "note-feed-aside",
                    div { class: "entry-dock note-entry-dock",
                        div { class: "entry-dock__header",
                            h3 { class: "graph-detail__title", "条目录入" }
                            p { class: "graph-detail__copy graph-detail__copy--muted",
                                "正文来自左侧 Markdown，首行 # 会自动成为标题。"
                            }
                        }
                        div { class: "entry-kind-strip",
                            for kind in KnowledgeEntryKind::ALL {
                                WorkbenchButton {
                                    class: "entry-kind-pill".to_string(),
                                    tone: if kind == form.kind { Some(Tone::Accent) } else { None },
                                    onclick: move |_| {
                                        let mut next = entry_form.read().clone();
                                        next.kind = kind;
                                        if next.tags.is_empty() {
                                            next.tags = vec![kind.label().to_string()];
                                        }
                                        entry_form.set(next);
                                        active_lens.set(kind.lens());
                                    },
                                    "{kind.label()}"
                                }
                            }
                        }
                        KeywordChips {
                            value: form.tags.clone(),
                            placeholder: Some("补充标签、平台或专题名".to_string()),
                            on_change: move |next| {
                                let mut form = entry_form.read().clone();
                                form.tags = next;
                                entry_form.set(form);
                            }
                        }
                        div { class: "entry-actions",
                            WorkbenchButton {
                                class: "action-button action-button--primary".to_string(),
                                onclick: move |_| {
                                    let form = entry_form.read().clone();
                                    let body = form.body.trim();

                                    if body.is_empty() {
                                        return;
                                    }

                                    let title = derive_markdown_entry_title(body, form.kind);
                                    let tags = {
                                        let tags = form.cleaned_tags();
                                        if tags.is_empty() {
                                            vec![form.kind.label().to_string()]
                                        } else {
                                            tags
                                        }
                                    };
                                    entry_log.with_mut(|queue| {
                                        queue.insert(
                                            0,
                                            KnowledgeEntryRecord {
                                                title,
                                                body: body.to_string(),
                                                source: form.kind.default_source().to_string(),
                                                tags,
                                                kind: form.kind,
                                                captured_at: Local::now().format("%H:%M").to_string(),
                                            },
                                        );
                                        if queue.len() > 8 {
                                            queue.truncate(8);
                                        }
                                    });
                                    entry_form.set(EntryFormState::for_capture(lens.entry_kind()));
                                },
                                "录入条目"
                            }
                            span { class: "entry-actions__hint", "Cmd+K 可直接回到搜索框。" }
                        }
                    }
                    div { class: "entry-log note-entry-log",
                        div { class: "entry-log__title", "最近录入" }
                        Stack {
                            for item in queue_preview {
                                ListItem {
                                    title: format!("{} · {}", item.kind.label(), item.title),
                                    detail: truncate_copy(&item.body, 88),
                                    meta: format!("{} · {} · {}", item.captured_at, item.source, item.tags.join(" / "))
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn NoteCaptureEditor(value: String, on_change: EventHandler<String>) -> Element {
    let mut editor_mode = use_signal(|| Mode::Source);
    let mut editor_value = use_signal(|| value.clone());

    use_effect(move || {
        let incoming = value.clone();
        if *editor_value.read() != incoming {
            editor_value.set(incoming);
        }
    });

    rsx! {
        div { class: "note-rich-editor",
            markdown::Root {
                class: "note-rich-editor__root",
                mode: editor_mode,
                on_mode_change: move |mode: Mode| editor_mode.set(mode),
                value: editor_value,
                on_value_change: move |next: String| {
                    editor_value.set(next.clone());
                    on_change.call(next);
                },
                markdown::Toolbar {
                    class: "note-rich-editor__toolbar",
                    NoteCaptureToolbar {}
                    span { class: "note-rich-editor__spacer" }
                    markdown::ModeBar {
                        class: "note-rich-editor__modes",
                        markdown::ModeTab {
                            class: "note-rich-editor__mode",
                            mode: Mode::Source,
                            title: "编辑 Markdown".to_string(),
                            "MD"
                        }
                        markdown::ModeTab {
                            class: "note-rich-editor__mode",
                            mode: Mode::LivePreview,
                            title: "双栏预览".to_string(),
                            "预览"
                        }
                    }
                }
                div { class: "note-rich-editor__body",
                    markdown::Editor {
                        class: "note-rich-editor__editor",
                        placeholder: "第一行写 # 标题，下面直接记录笔记正文。",
                        editor_aria_label: "条目说明 Markdown 编辑器",
                        spell_check: true,
                    }
                    markdown::Divider {
                        class: "note-rich-editor__divider",
                    }
                    markdown::Preview {
                        class: "note-rich-editor__preview",
                    }
                }
            }
        }
    }
}

#[component]
fn NoteCaptureToolbar() -> Element {
    let handle: MarkdownHandle = use_markdown_handle();

    rsx! {
        ToolbarAction {
            title: "标题",
            label: "H",
            onclick: move |_| {
                spawn(async move {
                    handle.insert_text("## ").await;
                });
            }
        }
        ToolbarAction {
            title: "加粗",
            label: "B",
            onclick: move |_| {
                spawn(async move {
                    handle.wrap_selection("**", "**").await;
                });
            }
        }
        ToolbarAction {
            title: "斜体",
            label: "I",
            onclick: move |_| {
                spawn(async move {
                    handle.wrap_selection("_", "_").await;
                });
            }
        }
        ToolbarAction {
            title: "删除线",
            label: "S",
            onclick: move |_| {
                spawn(async move {
                    handle.wrap_selection("~~", "~~").await;
                });
            }
        }
        ToolbarAction {
            title: "链接",
            label: "↗",
            onclick: move |_| {
                spawn(async move {
                    handle.wrap_selection("[", "](url)").await;
                });
            }
        }
        ToolbarAction {
            title: "无序列表",
            label: "•",
            onclick: move |_| {
                spawn(async move {
                    handle.insert_text("- ").await;
                });
            }
        }
        ToolbarAction {
            title: "有序列表",
            label: "1.",
            onclick: move |_| {
                spawn(async move {
                    handle.insert_text("1. ").await;
                });
            }
        }
        ToolbarAction {
            title: "任务",
            label: "☑",
            onclick: move |_| {
                spawn(async move {
                    handle.insert_text("- [ ] ").await;
                });
            }
        }
        ToolbarAction {
            title: "表格",
            label: "▦",
            onclick: move |_| {
                spawn(async move {
                    handle.insert_text("| 字段 | 说明 |\n| --- | --- |\n|  |  |\n").await;
                });
            }
        }
        ToolbarAction {
            title: "行内代码",
            label: "`",
            onclick: move |_| {
                spawn(async move {
                    handle.wrap_selection("`", "`").await;
                });
            }
        }
        ToolbarAction {
            title: "代码块",
            label: "</>",
            onclick: move |_| {
                spawn(async move {
                    handle.insert_text("```text\n\n```\n").await;
                });
            }
        }
    }
}

#[component]
fn ToolbarAction(
    title: &'static str,
    label: &'static str,
    onclick: EventHandler<MouseEvent>,
) -> Element {
    rsx! {
        markdown::ToolbarButton {
            class: "note-rich-editor__button",
            title: title.to_string(),
            onclick,
            "{label}"
        }
    }
}

#[component]
fn NoteCardGrid(docs: Vec<&'static KnowledgeDoc>) -> Element {
    if docs.is_empty() {
        return rsx! {
            div { class: "empty-state", "没有匹配的笔记卡片。" }
        };
    }

    rsx! {
        div { class: "note-card-grid",
            for doc in docs {
                article { class: "note-card",
                    div { class: "note-card__time", "{note_doc_chapter_label(doc)} · {doc.source_name}" }
                    h3 { class: "note-card__title", "{doc.title}" }
                    p { class: "note-card__body", "{doc.preview}" }
                    if !doc.headings.is_empty() {
                        div { class: "note-card__outline",
                            for heading in doc.headings.iter().take(3) {
                                span { "{heading}" }
                            }
                        }
                    }
                    div { class: "note-card__tags",
                        span { class: "badge badge--fs", "闪念" }
                        span { class: "badge", "{doc.filename}" }
                        span { class: "badge", "{doc.section_count} 节" }
                        span { class: "badge", "{format_bytes(doc.bytes)}" }
                    }
                }
            }
        }
    }
}

#[component]
fn SoftwareCardGrid(cards: Vec<SoftwareCard>) -> Element {
    if cards.is_empty() {
        return rsx! {
            div { class: "empty-state", "没有匹配的软件卡片。" }
        };
    }

    rsx! {
        div { class: "note-card-grid",
            for card in cards {
                article { class: "note-card note-card--software",
                    div { class: "note-card__time", "{card.source}" }
                    h3 { class: "note-card__title", "{card.title}" }
                    p { class: "note-card__body", "{card.body}" }
                    div { class: "note-card__tags",
                        for tag in card.tags {
                            span { class: "badge", "{tag}" }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn PackageCardGrid(cards: Vec<&'static PackageAsset>) -> Element {
    if cards.is_empty() {
        return rsx! {
            div { class: "empty-state", "没有匹配的安装包卡片。" }
        };
    }

    rsx! {
        div { class: "note-card-grid",
            for asset in cards {
                article { class: "note-card note-card--package",
                    div { class: "note-card__time", "{asset.version} · {asset.platform}" }
                    h3 { class: "note-card__title", "{asset.software_title}" }
                    p { class: "note-card__body", "{asset.note}" }
                    div { class: "note-card__outline",
                        span { "{asset.package_name}" }
                        span { "{asset.source}" }
                    }
                    div { class: "note-card__tags",
                        span { class: "badge badge--fs", "{asset.status}" }
                        span { class: "badge", "{asset.format}" }
                        span { class: "badge", "{asset.checksum_state}" }
                    }
                }
            }
        }
    }
}

#[component]
pub fn Audit() -> Element {
    rsx! {
        ContentHeader {
            title: "审计日志".to_string(),
            subtitle: "右侧上下文栏继续承担辅助理解，不抢主内容权重。".to_string()
        }
        Surface {
            SurfaceHeader {
                title: "最近日志".to_string(),
                subtitle: "先有时间线，再接过滤器和详情抽屉。".to_string()
            }
            Stack {
                ListItem { title: "09:12 审计任务结束".to_string(), detail: "策略审计通过，没有新增风险项".to_string(), meta: "system".to_string() }
                ListItem { title: "08:41 权限模板变更".to_string(), detail: "P-09 删除了一条遗留白名单".to_string(), meta: "Chen".to_string() }
                ListItem { title: "昨天 18:20 发布完成".to_string(), detail: "release-0426 已部署到 production".to_string(), meta: "Luna".to_string() }
            }
        }
    }
}

#[component]
pub fn DashboardContext() -> Element {
    let note_docs = note_docs();
    let note_section_total = note_docs.iter().map(|doc| doc.section_count).sum::<usize>();

    rsx! {
        SidebarSection { label: "当前默认".to_string(),
            Stack {
                MetricRow { label: "标签".to_string(), value: "笔记".to_string(), tone: Tone::Accent }
                MetricRow { label: "已纳入文档".to_string(), value: note_docs.len().to_string() }
                MetricRow { label: "章节小节".to_string(), value: note_section_total.to_string() }
            }
        }
        SidebarSection { label: "笔记分组".to_string(),
            Stack {
                for cluster in note_clusters() {
                    MetricRow { label: cluster.label.to_string(), value: cluster.docs.len().to_string() }
                }
            }
        }
        SidebarSection { label: "快捷键".to_string(),
            div { class: "callout callout--info",
                "Cmd+K 会聚焦当前页面的搜索框。"
            }
        }
        SidebarSection { label: "体量".to_string(),
            Stack {
                MetricRow { label: "Rust 摘录".to_string(), value: format_bytes(note_docs.iter().map(|doc| doc.bytes).sum()) }
                MetricRow { label: "全部知识".to_string(), value: format_bytes(total_bytes()) }
                MetricRow { label: "全库小节".to_string(), value: total_sections().to_string() }
            }
        }
    }
}

#[component]
pub fn DefaultContext() -> Element {
    rsx! {
        SidebarSection { label: "概览".to_string(),
            Stack {
                MetricRow { label: "成功率".to_string(), value: "99.2%".to_string(), tone: Tone::Positive }
                MetricRow { label: "告警数".to_string(), value: "03".to_string(), tone: Tone::Warning }
                MetricRow { label: "挂起变更".to_string(), value: "07".to_string() }
            }
        }
        SidebarSection { label: "最近动作".to_string(),
            Stack {
                ListItem {
                    title: "规则 A-17 已提交".to_string(),
                    detail: "Luna 更新了命中条件".to_string(),
                    meta: "2 分钟前".to_string()
                }
                ListItem {
                    title: "执行器扩容完成".to_string(),
                    detail: "队列延迟恢复到基线".to_string(),
                    meta: "18 分钟前".to_string()
                }
                ListItem {
                    title: "审计任务结束".to_string(),
                    detail: "未发现新增风险项".to_string(),
                    meta: "今天 09:12".to_string()
                }
            }
        }
    }
}

fn filtered_note_docs(query: &str) -> Vec<&'static KnowledgeDoc> {
    let query = query.trim().to_lowercase();
    note_docs()
        .into_iter()
        .filter(|doc| {
            query.is_empty()
                || doc.title.to_lowercase().contains(&query)
                || doc.preview.to_lowercase().contains(&query)
                || doc.excerpt.to_lowercase().contains(&query)
                || doc.source_name.to_lowercase().contains(&query)
                || doc.filename.to_lowercase().contains(&query)
                || doc
                    .headings
                    .iter()
                    .any(|heading| heading.to_lowercase().contains(&query))
        })
        .collect()
}

fn filtered_software_cards(query: &str) -> Vec<SoftwareCard> {
    let query = query.trim().to_lowercase();
    software_cards()
        .into_iter()
        .filter(|card| {
            query.is_empty()
                || card.title.to_lowercase().contains(&query)
                || card.body.to_lowercase().contains(&query)
                || card.source.to_lowercase().contains(&query)
                || card
                    .tags
                    .iter()
                    .any(|tag| tag.to_lowercase().contains(&query))
        })
        .collect()
}

fn filtered_package_cards(query: &str) -> Vec<&'static PackageAsset> {
    let query = query.trim().to_lowercase();
    PACKAGE_ASSETS
        .iter()
        .filter(|asset| {
            query.is_empty()
                || asset.software_title.to_lowercase().contains(&query)
                || asset.package_name.to_lowercase().contains(&query)
                || asset.note.to_lowercase().contains(&query)
                || asset.platform.to_lowercase().contains(&query)
                || asset.status.to_lowercase().contains(&query)
                || asset.source.to_lowercase().contains(&query)
        })
        .collect()
}

fn software_cards() -> Vec<SoftwareCard> {
    vec![
        SoftwareCard {
            title: "Cursor",
            source: "IDE / Agent",
            body: "代理协作与代码编写主入口，适合和技能、插件、仓库工作流一起归档。",
            tags: &["软件", "开发", "Agent"],
        },
        SoftwareCard {
            title: "Rust Analyzer",
            source: "LSP",
            body: "语言服务与符号跳转分析，支撑 Rust 单体仓库的结构阅读和重构。",
            tags: &["软件", "Rust", "LSP"],
        },
        SoftwareCard {
            title: "Docker Desktop",
            source: "Runtime",
            body: "容器编排、网络与镜像运行，本地开发环境和服务验证都依赖它。",
            tags: &["软件", "容器", "本地基础设施"],
        },
        SoftwareCard {
            title: "cloudflared",
            source: "Ingress",
            body: "本地服务暴露和隧道入口，适合与反向代理、域名和发布记录放在一起。",
            tags: &["软件", "隧道", "公网访问"],
        },
        SoftwareCard {
            title: "Obsidian",
            source: "Notes",
            body: "笔记编辑与知识沉淀入口，可以作为知识条目和附件包之间的桥接对象。",
            tags: &["软件", "笔记", "知识库"],
        },
        SoftwareCard {
            title: "Cargo",
            source: "Build",
            body: "Rust 构建、测试与依赖管理工具，和安装资产、CI 记录天然相关。",
            tags: &["软件", "Rust", "构建"],
        },
    ]
}

fn seed_entry_log() -> Vec<KnowledgeEntryRecord> {
    vec![
        KnowledgeEntryRecord {
            title: "ownership 章节适合补图示".to_string(),
            body: "把 move / borrow / mutable borrow 的冲突关系整理成更容易检索的笔记卡片。"
                .to_string(),
            source: "阅读 Rust 章节".to_string(),
            tags: vec!["笔记".to_string(), "所有权".to_string()],
            kind: KnowledgeEntryKind::Note,
            captured_at: "09:18".to_string(),
        },
        KnowledgeEntryRecord {
            title: "cloudflared 可挂到软件台账".to_string(),
            body: "和 tunnel 相关安装包一起展示时，软件卡片与安装包卡片需要互相可查。".to_string(),
            source: "运维梳理".to_string(),
            tags: vec!["软件".to_string(), "安装包".to_string()],
            kind: KnowledgeEntryKind::Software,
            captured_at: "昨天".to_string(),
        },
    ]
}

fn derive_markdown_entry_title(body: &str, kind: KnowledgeEntryKind) -> String {
    let Some(first_line) = body.lines().map(str::trim).find(|line| !line.is_empty()) else {
        return format!("未命名{}", kind.label());
    };

    markdown_heading_title(first_line)
        .or_else(|| Some(strip_inline_markdown_title(first_line)))
        .filter(|title| !title.is_empty())
        .unwrap_or_else(|| format!("未命名{}", kind.label()))
}

fn markdown_heading_title(line: &str) -> Option<String> {
    let heading_level = line.bytes().take_while(|byte| *byte == b'#').count();
    if heading_level == 0 || heading_level > 6 {
        return None;
    }

    let title = line[heading_level..].trim();
    if title.is_empty() {
        return None;
    }

    Some(strip_inline_markdown_title(title.trim_end_matches('#').trim()))
}

fn strip_inline_markdown_title(line: &str) -> String {
    line.trim()
        .trim_matches(|ch| matches!(ch, '*' | '_' | '`' | '~'))
        .trim()
        .to_string()
}

fn note_docs() -> Vec<&'static KnowledgeDoc> {
    let rust_docs = KNOWLEDGE_DOCS
        .iter()
        .filter(|doc| doc.source_slug == "rust")
        .collect::<Vec<_>>();
    if rust_docs.is_empty() {
        KNOWLEDGE_DOCS
            .iter()
            .filter(|doc| doc.source_slug == "notes")
            .collect()
    } else {
        rust_docs
    }
}

fn note_clusters() -> Vec<NoteCluster> {
    let mut foundations = Vec::new();
    let mut core = Vec::new();
    let mut practice = Vec::new();
    let mut advanced = Vec::new();

    for doc in note_docs() {
        match note_cluster_for(doc) {
            "基础入门" => foundations.push(doc),
            "语言核心" => core.push(doc),
            "工程实践" => practice.push(doc),
            _ => advanced.push(doc),
        }
    }

    vec![
        NoteCluster {
            label: "基础入门",
            docs: foundations,
        },
        NoteCluster {
            label: "语言核心",
            docs: core,
        },
        NoteCluster {
            label: "工程实践",
            docs: practice,
        },
        NoteCluster {
            label: "高级专题",
            docs: advanced,
        },
    ]
    .into_iter()
    .filter(|cluster| !cluster.docs.is_empty())
    .collect()
}

fn note_cluster_for(doc: &KnowledgeDoc) -> &'static str {
    match note_chapter_no(doc) {
        1..=4 => "基础入门",
        5..=10 => "语言核心",
        11..=17 => "工程实践",
        _ => "高级专题",
    }
}

fn note_doc_chapter_label(doc: &KnowledgeDoc) -> String {
    let number = note_chapter_no(doc);
    if number == 0 {
        "CH".to_string()
    } else {
        format!("CH {:02}", number)
    }
}

fn note_chapter_no(doc: &KnowledgeDoc) -> usize {
    doc.filename
        .split('-')
        .next()
        .and_then(|raw| raw.parse::<usize>().ok())
        .unwrap_or_default()
}

fn truncate_copy(text: &str, limit: usize) -> String {
    let mut result = String::new();
    for (idx, ch) in text.chars().enumerate() {
        if idx == limit {
            result.push('…');
            break;
        }
        result.push(ch);
    }
    result
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
