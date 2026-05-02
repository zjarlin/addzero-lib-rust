use std::fmt::Write as _;

use chrono::Local;
use dioxus::prelude::*;
use dioxus_components::{
    ContentHeader, ListItem, MetricRow, SidebarSection, Stack, Surface, SurfaceHeader, Tone,
    WorkbenchButton,
};
use dioxus_free_icons::{Icon, icons::ld_icons::LdSend};
use dioxus_nox_markdown::{
    markdown,
    prelude::{MarkdownHandle, Mode, use_markdown_handle},
};

use crate::{
    knowledge_catalog::{KNOWLEDGE_DOCS, KnowledgeDoc, total_bytes, total_sections},
    package_catalog::{PACKAGE_ASSETS, PackageAsset},
    services::{LogoUploadRequest, build_preview_url},
    state::AppServices,
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
    let app_services = use_context::<AppServices>();
    let logo_storage = app_services.logo_storage.clone();
    let mut active_axis = use_signal(|| WorkspaceAxis::Notes);
    let mut search = use_signal(String::new);
    let mut tree_items = use_signal(seed_chat_tree);
    let mut selected_item_id = use_signal(|| {
        seed_chat_tree()
            .first()
            .map(|item| item.id.clone())
            .unwrap_or_default()
    });
    let mut composer = use_signal(ComposerDraft::default);
    let mut attachment_menu_open = use_signal(|| false);
    let mut feedback = use_signal(|| None::<String>);

    let axis = *active_axis.read();
    let query = search.read().clone();
    let visible_items = filtered_tree_items(&tree_items.read(), axis, &query);
    let result_count = visible_items.len();
    let selected_item = selected_tree_item(&tree_items.read(), &selected_item_id.read())
        .or_else(|| visible_items.first().cloned());
    let draft = composer.read().clone();

    rsx! {
        ContentHeader {
            title: "Agent 工作台".to_string(),
            subtitle: "树形资产、CLI 安装、笔记录入和附件采集收敛到同一个对话入口。".to_string(),
            actions: rsx!(
                div { class: "shortcut-hint", "Cmd K" }
            )
        }
        div { class: "agent-workbench",
            div { class: "agent-workbench__axis",
                for item in WorkspaceAxis::ALL {
                    WorkbenchButton {
                        class: "lens-pill".to_string(),
                        tone: if item == axis { Some(Tone::Accent) } else { None },
                        onclick: move |_| {
                            active_axis.set(item);
                            search.set(String::new());
                            if let Some(first) = first_item_for_axis(&tree_items.read(), item) {
                                selected_item_id.set(first.id);
                            }
                        },
                        "{item.label()}"
                    }
                }
            }
            div { class: "agent-workbench__body",
                aside { class: "agent-tree",
                    div { class: "agent-tree__head",
                        div {
                            h2 { "{axis.tree_label()}" }
                            span { "{result_count} 个节点" }
                        }
                        input {
                            id: DASHBOARD_SEARCH_ID,
                            "data-command-search": "true",
                            class: "graph-search agent-tree__search",
                            r#type: "search",
                            value: "{search}",
                            placeholder: "搜索树节点",
                            oninput: move |evt| search.set(evt.value())
                        }
                    }
                    div { class: "agent-tree__list",
                        for item in visible_items.clone() {
                            AgentTreeRow {
                                item: item.clone(),
                                selected: selected_item.as_ref().is_some_and(|selected| selected.id == item.id),
                                on_select: move |id| selected_item_id.set(id),
                                on_edit: move |item: ChatTreeItem| {
                                    selected_item_id.set(item.id.clone());
                                    composer.set(ComposerDraft {
                                        text: item.body.clone(),
                                        attachments: item.attachments.clone(),
                                    });
                                    feedback.set(Some(format!("已载入 `{}`，可在下方对话框继续编辑。", item.title)));
                                },
                                on_delete: move |id| {
                                    tree_items.with_mut(|items| items.retain(|item| item.id != id));
                                    if selected_item_id.read().as_str() == id {
                                        if let Some(first) = first_item_for_axis(&tree_items.read(), axis) {
                                            selected_item_id.set(first.id);
                                        } else {
                                            selected_item_id.set(String::new());
                                        }
                                    }
                                    feedback.set(Some("已从当前树移除该节点。".to_string()));
                                },
                                on_copy: move |item: ChatTreeItem| copy_to_clipboard(item.copy_payload()),
                            }
                        }
                        if result_count == 0 {
                            div { class: "agent-tree__empty", "没有匹配的节点。" }
                        }
                    }
                }
                main { class: "agent-stage",
                    div { class: "agent-stage__content",
                        AgentDetailPanel { item: selected_item.clone(), axis }
                    }
                    div { class: "agent-composer-wrap",
                        if let Some(message) = feedback.read().clone() {
                            div { class: "agent-feedback", "{message}" }
                        }
                        div { class: "agent-composer",
                            div { class: "agent-composer__attachments",
                                for attachment in draft.attachments.iter() {
                                    div { class: "agent-attachment-chip",
                                        if attachment.kind == AttachmentKind::Image {
                                            span { class: "agent-attachment-chip__icon", "图" }
                                        } else {
                                            span { class: "agent-attachment-chip__icon", "文" }
                                        }
                                        span { "{attachment.name}" }
                                        span { class: "agent-attachment-chip__state", "{attachment.status}" }
                                    }
                                }
                            }
                            div { class: "agent-composer__row",
                                div { class: "agent-composer__attach",
                                    button {
                                        class: "agent-icon-button",
                                        r#type: "button",
                                        title: "添加附件",
                                        onclick: move |_| {
                                            let is_open = *attachment_menu_open.read();
                                            attachment_menu_open.set(!is_open);
                                        },
                                        "+"
                                    }
                                    if *attachment_menu_open.read() {
                                        div { class: "agent-attach-menu",
                                            label { class: "agent-attach-menu__item",
                                                "图片"
                                                input {
                                                    class: "agent-file-input",
                                                    r#type: "file",
                                                    accept: "image/*",
                                                    onchange: move |evt| {
                                                        let Some(file) = evt.files().into_iter().next() else {
                                                            return;
                                                        };
                                                        attachment_menu_open.set(false);
                                                        feedback.set(Some("正在上传图片到 MinIO…".to_string()));
                                                        let logo_storage = logo_storage.clone();
                                                        let mut composer = composer;
                                                        let mut feedback = feedback;
                                                        spawn(async move {
                                                            let file_name = file.name();
                                                            let content_type = file.content_type();
                                                            match file.read_bytes().await {
                                                                Ok(bytes) => {
                                                                    let upload = LogoUploadRequest {
                                                                        file_name: file_name.clone(),
                                                                        content_type,
                                                                        bytes: bytes.to_vec(),
                                                                    };
                                                                    match logo_storage.upload_logo(upload).await {
                                                                        Ok(stored) => {
                                                                            composer.with_mut(|draft| {
                                                                                draft.attachments.push(ChatAttachment {
                                                                                    kind: AttachmentKind::Image,
                                                                                    name: file_name,
                                                                                    status: "已上传".to_string(),
                                                                                    url: Some(build_preview_url(&stored.relative_path)),
                                                                                });
                                                                            });
                                                                            feedback.set(Some("图片已上传并加入当前对话。".to_string()));
                                                                        }
                                                                        Err(err) => feedback.set(Some(format!("图片上传失败：{err}"))),
                                                                    }
                                                                }
                                                                Err(err) => feedback.set(Some(format!("读取图片失败：{err}"))),
                                                            }
                                                        });
                                                    }
                                                }
                                            }
                                            label { class: "agent-attach-menu__item",
                                                "文件"
                                                input {
                                                    class: "agent-file-input",
                                                    r#type: "file",
                                                    onchange: move |evt| {
                                                        let Some(file) = evt.files().into_iter().next() else {
                                                            return;
                                                        };
                                                        attachment_menu_open.set(false);
                                                        composer.with_mut(|draft| {
                                                            draft.attachments.push(ChatAttachment {
                                                                kind: AttachmentKind::File,
                                                                name: file.name(),
                                                                status: "本地待发送".to_string(),
                                                                url: None,
                                                            });
                                                        });
                                                        feedback.set(Some("文件已加入当前对话，后续可接通用对象存储端点。".to_string()));
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                                textarea {
                                    class: "agent-composer__input",
                                    value: "{draft.text}",
                                    placeholder: "{axis.placeholder()}",
                                    oninput: move |evt| {
                                        composer.with_mut(|draft| draft.text = evt.value());
                                    }
                                }
                                button {
                                    class: "agent-send-button",
                                    r#type: "button",
                                    title: "发送并录入树节点",
                                    onclick: move |_| {
                                        let draft = composer.read().clone();
                                        if draft.text.trim().is_empty() && draft.attachments.is_empty() {
                                            feedback.set(Some("先输入内容，或添加图片 / 文件附件。".to_string()));
                                            return;
                                        }
                                        let item = ChatTreeItem::from_draft(axis, draft);
                                        let item_id = item.id.clone();
                                        tree_items.with_mut(|items| items.insert(0, item));
                                        selected_item_id.set(item_id);
                                        composer.set(ComposerDraft::default());
                                        feedback.set(Some("已把这条对话录入当前树。".to_string()));
                                    },
                                    Icon { width: 16, height: 16, icon: LdSend }
                                }
                            }
                        }
                    }
                }
                aside { class: "agent-inspector",
                    h2 { "上下文" }
                    div { class: "agent-inspector__section",
                        span { "当前树" }
                        strong { "{axis.label()}" }
                    }
                    div { class: "agent-inspector__section",
                        span { "节点数" }
                        strong { "{tree_items.read().iter().filter(|item| item.axis == axis).count()}" }
                    }
                    div { class: "agent-inspector__section",
                        span { "交互模型" }
                        p { "{axis.workflow_hint()}" }
                    }
                    if !draft.attachments.is_empty() {
                        div { class: "agent-inspector__section",
                            span { "待发送附件" }
                            for attachment in draft.attachments {
                                div { class: "agent-inspector__attachment",
                                    strong { "{attachment.name}" }
                                    small { "{attachment.status}" }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
enum WorkspaceAxis {
    Notes,
    Packages,
    Skills,
    Cli,
}

impl WorkspaceAxis {
    const ALL: [Self; 4] = [Self::Notes, Self::Packages, Self::Skills, Self::Cli];

    fn label(self) -> &'static str {
        match self {
            Self::Notes => "笔记树",
            Self::Packages => "安装包树",
            Self::Skills => "技能树",
            Self::Cli => "CLI 树",
        }
    }

    fn tree_label(self) -> &'static str {
        match self {
            Self::Notes => "Notes",
            Self::Packages => "Packages",
            Self::Skills => "Skills",
            Self::Cli => "CLI",
        }
    }

    fn placeholder(self) -> &'static str {
        match self {
            Self::Notes => "直接写 Markdown；第一行 # 会自动成为标题。",
            Self::Packages => "粘贴软件名、安装包路径、版本、校验信息；安装会被封装成 CLI 记录。",
            Self::Skills => "粘贴 skill 描述、触发词或 README；后续可生成双语技能元数据。",
            Self::Cli => "粘贴命令、仓库、安装方式或文档链接；会进入 CLI 字典。",
        }
    }

    fn workflow_hint(self) -> &'static str {
        match self {
            Self::Notes => "笔记先入树，再逐步提取实体、关系和来源。",
            Self::Packages => "安装包被视为 CLI 安装资产，和软件台账绑定。",
            Self::Skills => "技能节点用于承载 agent 能力、触发词和安装入口。",
            Self::Cli => "CLI 节点是插件市场的主索引，支持中英双语与渐进式搜索。",
        }
    }

    fn entry_kind(self) -> KnowledgeEntryKind {
        match self {
            Self::Notes => KnowledgeEntryKind::Note,
            Self::Packages => KnowledgeEntryKind::Package,
            Self::Skills | Self::Cli => KnowledgeEntryKind::Software,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum AttachmentKind {
    Image,
    File,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct ChatAttachment {
    kind: AttachmentKind,
    name: String,
    status: String,
    url: Option<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
struct ComposerDraft {
    text: String,
    attachments: Vec<ChatAttachment>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct ChatTreeItem {
    id: String,
    axis: WorkspaceAxis,
    title: String,
    body: String,
    source: String,
    tags: Vec<String>,
    attachments: Vec<ChatAttachment>,
    created_at: String,
}

impl ChatTreeItem {
    fn from_draft(axis: WorkspaceAxis, draft: ComposerDraft) -> Self {
        let trimmed = draft.text.trim();
        let title = if trimmed.is_empty() {
            draft
                .attachments
                .first()
                .map(|attachment| attachment.name.clone())
                .unwrap_or_else(|| format!("未命名{}", axis.label()))
        } else {
            derive_markdown_entry_title(trimmed, axis.entry_kind())
        };

        let body = if trimmed.is_empty() {
            format!("附件录入：{}", attachment_summary(&draft.attachments))
        } else {
            trimmed.to_string()
        };

        Self {
            id: format!(
                "draft-{}-{}",
                axis.tree_label().to_lowercase(),
                Local::now().timestamp_millis()
            ),
            axis,
            title,
            body,
            source: "对话录入".to_string(),
            tags: vec![axis.label().to_string(), "对话".to_string()],
            attachments: draft.attachments,
            created_at: Local::now().format("%H:%M").to_string(),
        }
    }

    fn copy_payload(&self) -> String {
        let mut payload = format!("{}\n\n{}", self.title, self.body);
        if !self.attachments.is_empty() {
            let _ = write!(
                payload,
                "\n\n附件：{}",
                attachment_summary(&self.attachments)
            );
        }
        payload
    }
}

#[component]
fn AgentTreeRow(
    item: ChatTreeItem,
    selected: bool,
    on_select: EventHandler<String>,
    on_edit: EventHandler<ChatTreeItem>,
    on_delete: EventHandler<String>,
    on_copy: EventHandler<ChatTreeItem>,
) -> Element {
    let class = if selected {
        "agent-tree-row is-selected"
    } else {
        "agent-tree-row"
    };

    rsx! {
        div { class,
            button {
                class: "agent-tree-row__main",
                r#type: "button",
                onclick: {
                    let id = item.id.clone();
                    move |_| on_select.call(id.clone())
                },
                span { class: "agent-tree-row__title", "{item.title}" }
                span { class: "agent-tree-row__meta", "{item.source} · {item.created_at}" }
            }
            div { class: "agent-tree-row__actions",
                button {
                    class: "agent-row-action",
                    r#type: "button",
                    title: "编辑",
                    onclick: {
                        let item = item.clone();
                        move |_| on_edit.call(item.clone())
                    },
                    "✎"
                }
                button {
                    class: "agent-row-action agent-row-action--danger",
                    r#type: "button",
                    title: "删除",
                    onclick: {
                        let id = item.id.clone();
                        move |_| on_delete.call(id.clone())
                    },
                    "⌫"
                }
                button {
                    class: "agent-row-action",
                    r#type: "button",
                    title: "复制",
                    onclick: {
                        let item = item.clone();
                        move |_| on_copy.call(item.clone())
                    },
                    "⧉"
                }
            }
        }
    }
}

#[component]
fn AgentDetailPanel(item: Option<ChatTreeItem>, axis: WorkspaceAxis) -> Element {
    let Some(item) = item else {
        return rsx! {
            div { class: "agent-detail agent-detail--empty",
                h2 { "空树" }
                p { "从下方聊天框录入一条内容后，会在这里展示节点详情。" }
            }
        };
    };

    rsx! {
        article { class: "agent-detail",
            div { class: "agent-detail__head",
                div {
                    span { class: "badge badge--fs", "{axis.label()}" }
                    h2 { "{item.title}" }
                }
                div { class: "agent-detail__time", "{item.created_at}" }
            }
            div { class: "agent-detail__body",
                p { "{item.body}" }
            }
            if !item.attachments.is_empty() {
                div { class: "agent-detail__attachments",
                    for attachment in item.attachments {
                        div { class: "agent-detail__attachment",
                            if let Some(url) = attachment.url {
                                img { src: "{url}", alt: "{attachment.name}" }
                            }
                            div {
                                strong { "{attachment.name}" }
                                span { "{attachment.status}" }
                            }
                        }
                    }
                }
            }
            div { class: "agent-detail__tags",
                for tag in item.tags {
                    span { class: "badge", "{tag}" }
                }
            }
        }
    }
}

fn filtered_tree_items(
    items: &[ChatTreeItem],
    axis: WorkspaceAxis,
    query: &str,
) -> Vec<ChatTreeItem> {
    let query = query.trim().to_lowercase();
    items
        .iter()
        .filter(|item| {
            item.axis == axis
                && (query.is_empty()
                    || item.title.to_lowercase().contains(&query)
                    || item.body.to_lowercase().contains(&query)
                    || item.source.to_lowercase().contains(&query)
                    || item
                        .tags
                        .iter()
                        .any(|tag| tag.to_lowercase().contains(&query)))
        })
        .cloned()
        .collect()
}

fn selected_tree_item(items: &[ChatTreeItem], selected_id: &str) -> Option<ChatTreeItem> {
    items.iter().find(|item| item.id == selected_id).cloned()
}

fn first_item_for_axis(items: &[ChatTreeItem], axis: WorkspaceAxis) -> Option<ChatTreeItem> {
    items.iter().find(|item| item.axis == axis).cloned()
}

fn copy_to_clipboard(text: String) {
    let payload = format!("{text:?}");
    spawn(async move {
        let _ = document::eval(&format!(
            "navigator.clipboard && navigator.clipboard.writeText({payload});"
        ))
        .await;
    });
}

fn seed_chat_tree() -> Vec<ChatTreeItem> {
    let mut items = Vec::new();

    for doc in note_docs().into_iter().take(8) {
        items.push(ChatTreeItem {
            id: format!("note-{}", doc.filename),
            axis: WorkspaceAxis::Notes,
            title: doc.title.to_string(),
            body: doc.preview.to_string(),
            source: doc.source_name.to_string(),
            tags: vec![
                "笔记".to_string(),
                doc.filename.to_string(),
                format!("{} 节", doc.section_count),
            ],
            attachments: Vec::new(),
            created_at: note_doc_chapter_label(doc),
        });
    }

    for asset in PACKAGE_ASSETS.iter().take(8) {
        items.push(ChatTreeItem {
            id: format!("package-{}-{}", asset.package_name, asset.version),
            axis: WorkspaceAxis::Packages,
            title: asset.software_title.to_string(),
            body: format!(
                "{}\n\n安装包：{}\n平台：{}\n来源：{}",
                asset.note, asset.package_name, asset.platform, asset.source
            ),
            source: asset.source.to_string(),
            tags: vec![
                "安装包".to_string(),
                asset.version.to_string(),
                asset.status.to_string(),
                asset.format.to_string(),
            ],
            attachments: Vec::new(),
            created_at: asset.platform.to_string(),
        });
    }

    for (idx, item) in seed_skill_items().into_iter().enumerate() {
        items.push(ChatTreeItem {
            id: format!("skill-{idx}-{}", item.0),
            axis: WorkspaceAxis::Skills,
            title: item.0.to_string(),
            body: item.1.to_string(),
            source: "本地 skill".to_string(),
            tags: vec!["Skill".to_string(), "Agent".to_string()],
            attachments: Vec::new(),
            created_at: "ready".to_string(),
        });
    }

    for (idx, item) in seed_cli_items().into_iter().enumerate() {
        items.push(ChatTreeItem {
            id: format!("cli-{idx}-{}", item.0),
            axis: WorkspaceAxis::Cli,
            title: item.0.to_string(),
            body: item.1.to_string(),
            source: "CLI 字典".to_string(),
            tags: vec!["CLI".to_string(), "install".to_string()],
            attachments: Vec::new(),
            created_at: "draft".to_string(),
        });
    }

    items
}

fn seed_skill_items() -> Vec<(&'static str, &'static str)> {
    vec![
        (
            "ui-convention",
            "三栏工作台、树形导航、右侧上下文和低噪音 admin 布局约束。",
        ),
        (
            "rust-best-practices",
            "Rust 工程实践、错误处理、测试与类型建模指南。",
        ),
        (
            "cli-hub-meta-skill",
            "发现 agent-native CLI，并沉淀成插件市场条目。",
        ),
        (
            "addzero-cloudflare-tunnel",
            "把本机服务通过 Cloudflare Tunnel 暴露到 addzero.site。",
        ),
    ]
}

fn seed_cli_items() -> Vec<(&'static str, &'static str)> {
    vec![
        (
            "cargo",
            "Rust 构建、测试、发布和依赖管理入口。安装可表达为 rustup + cargo 子命令。",
        ),
        (
            "cloudflared",
            "Cloudflare Tunnel CLI，用于本地服务公网暴露和隧道管理。",
        ),
        (
            "rg",
            "ripgrep，全库搜索默认工具，应作为 CLI 字典的高频开发项。",
        ),
        (
            "docker",
            "容器运行和镜像管理入口，软件安装记录可映射为 docker CLI 能力。",
        ),
    ]
}

fn attachment_summary(attachments: &[ChatAttachment]) -> String {
    attachments
        .iter()
        .map(|attachment| attachment.name.as_str())
        .collect::<Vec<_>>()
        .join("、")
}

#[component]
fn NoteCaptureEditor(
    value: String,
    on_change: EventHandler<String>,
    on_submit: EventHandler<()>,
) -> Element {
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
                    markdown::ToolbarButton {
                        class: "note-rich-editor__button note-rich-editor__submit".to_string(),
                        title: "录入条目",
                        aria_label: "录入条目",
                        onclick: move |_| on_submit.call(()),
                        Icon { width: 15, height: 15, icon: LdSend }
                    }
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
fn NoteCardGrid(entries: Vec<KnowledgeEntryRecord>, docs: Vec<&'static KnowledgeDoc>) -> Element {
    if entries.is_empty() && docs.is_empty() {
        return rsx! {
            div { class: "empty-state", "没有匹配的笔记卡片。" }
        };
    }

    rsx! {
        div { class: "note-card-grid",
            for entry in entries {
                EntryRecordCard { entry }
            }
            for doc in docs {
                article { class: "note-card",
                    CardActions { copy_text: format!("{}\n\n{}", doc.title, doc.preview) }
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
                        span { class: "badge badge--fs", "智能体工作台" }
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
fn SoftwareCardGrid(entries: Vec<KnowledgeEntryRecord>, cards: Vec<SoftwareCard>) -> Element {
    if entries.is_empty() && cards.is_empty() {
        return rsx! {
            div { class: "empty-state", "没有匹配的软件卡片。" }
        };
    }

    rsx! {
        div { class: "note-card-grid",
            for entry in entries {
                EntryRecordCard { entry }
            }
            for card in cards {
                article { class: "note-card note-card--software",
                    CardActions { copy_text: format!("{}\n\n{}", card.title, card.body) }
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
fn PackageCardGrid(
    entries: Vec<KnowledgeEntryRecord>,
    cards: Vec<&'static PackageAsset>,
) -> Element {
    if entries.is_empty() && cards.is_empty() {
        return rsx! {
            div { class: "empty-state", "没有匹配的安装包卡片。" }
        };
    }

    rsx! {
        div { class: "note-card-grid",
            for entry in entries {
                EntryRecordCard { entry }
            }
            for asset in cards {
                article { class: "note-card note-card--package",
                    CardActions { copy_text: format!("{}\n\n{}", asset.software_title, asset.note) }
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
fn EntryRecordCard(entry: KnowledgeEntryRecord) -> Element {
    let tone_class = match entry.kind {
        KnowledgeEntryKind::Note => "",
        KnowledgeEntryKind::Software => " note-card--software",
        KnowledgeEntryKind::Package => " note-card--package",
    };
    let class = format!("note-card note-card--record{tone_class}");

    rsx! {
        article { class,
            CardActions { copy_text: format!("{}\n\n{}", entry.title, entry.body) }
            div { class: "note-card__time", "{entry.captured_at} · {entry.source}" }
            h3 { class: "note-card__title", "{entry.title}" }
            p { class: "note-card__body", "{entry.body}" }
            div { class: "note-card__tags",
                span { class: "badge badge--fs", "{entry.kind.label()}" }
                for tag in entry.tags {
                    span { class: "badge", "{tag}" }
                }
            }
        }
    }
}

#[component]
fn CardActions(copy_text: String) -> Element {
    let copy_payload = format!("{copy_text:?}");

    rsx! {
        div { class: "note-card__actions", "aria-label": "卡片操作",
            button {
                class: "note-card__action",
                r#type: "button",
                title: "编辑",
                "aria-label": "编辑卡片",
                "✎"
            }
            button {
                class: "note-card__action note-card__action--danger",
                r#type: "button",
                title: "删除",
                "aria-label": "删除卡片",
                "⌫"
            }
            button {
                class: "note-card__action",
                r#type: "button",
                title: "复制",
                "aria-label": "复制卡片",
                onclick: move |_| {
                    let js = format!(
                        "navigator.clipboard && navigator.clipboard.writeText({copy_payload});"
                    );
                    spawn(async move {
                        let _ = document::eval(&js).await;
                    });
                },
                "⧉"
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

fn filtered_recorded_entries(
    entries: &[KnowledgeEntryRecord],
    lens: DashboardLens,
    query: &str,
) -> Vec<KnowledgeEntryRecord> {
    let query = query.trim().to_lowercase();
    entries
        .iter()
        .filter(|entry| {
            entry.kind.lens() == lens
                && (query.is_empty()
                    || entry.title.to_lowercase().contains(&query)
                    || entry.body.to_lowercase().contains(&query)
                    || entry.source.to_lowercase().contains(&query)
                    || entry
                        .tags
                        .iter()
                        .any(|tag| tag.to_lowercase().contains(&query)))
        })
        .cloned()
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

    Some(strip_inline_markdown_title(
        title.trim_end_matches('#').trim(),
    ))
}

fn strip_inline_markdown_title(line: &str) -> String {
    line.trim()
        .trim_matches(|ch| matches!(ch, '*' | '_' | '`' | '~'))
        .trim()
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn derive_markdown_entry_title_should_use_first_heading() {
        assert_eq!(
            derive_markdown_entry_title("# 自动识别为标题\n后面是正文", KnowledgeEntryKind::Note),
            "自动识别为标题"
        );
    }

    #[test]
    fn derive_markdown_entry_title_should_fallback_to_first_line() {
        assert_eq!(
            derive_markdown_entry_title("第一行普通文本\n第二行正文", KnowledgeEntryKind::Note),
            "第一行普通文本"
        );
    }

    #[test]
    fn derive_markdown_entry_title_should_fallback_when_empty() {
        assert_eq!(
            derive_markdown_entry_title("   \n\t", KnowledgeEntryKind::Note),
            "未命名笔记"
        );
    }
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

fn format_bytes(bytes: usize) -> String {
    if bytes >= 1_048_576 {
        format!("{:.1} MB", bytes as f64 / 1_048_576.0)
    } else if bytes >= 1_024 {
        format!("{:.0} KB", bytes as f64 / 1_024.0)
    } else {
        format!("{bytes} B")
    }
}
