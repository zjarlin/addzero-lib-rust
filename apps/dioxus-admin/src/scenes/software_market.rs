use addzero_software_catalog::{
    InstallerKind, SoftwareCatalogDto, SoftwareDraftInput, SoftwareEntryDto, SoftwareEntryInput,
    SoftwareInstallMethodDto, SoftwareMetadataFetchInput, SoftwarePlatform,
};
use dioxus::prelude::*;
use dioxus_components::{
    ConfirmDialog, ContentHeader, MetricRow, ResponsiveGrid, SidebarSection, Stack, StatTile,
    Surface, SurfaceHeader, Tone, WorkbenchButton,
};

use crate::{services::AssetGraphItemDto, state::AppServices};

#[derive(Clone, Copy, PartialEq, Eq)]
enum PlatformFilter {
    Host,
    All,
    Macos,
    Windows,
    Linux,
}

impl PlatformFilter {
    const ALL: [Self; 5] = [
        Self::Host,
        Self::All,
        Self::Macos,
        Self::Windows,
        Self::Linux,
    ];

    fn label(self, host: SoftwarePlatform) -> String {
        match self {
            Self::Host => format!("本机 · {}", host.label()),
            Self::All => "全部平台".to_string(),
            Self::Macos => "macOS".to_string(),
            Self::Windows => "Windows".to_string(),
            Self::Linux => "Linux".to_string(),
        }
    }

    fn matches(self, entry: &SoftwareEntryDto, host: SoftwarePlatform) -> bool {
        match self {
            Self::Host => {
                entry.methods.iter().any(|method| method.platform == host)
                    || entry.trial_platforms.contains(&host)
            }
            Self::All => true,
            Self::Macos => has_platform(entry, SoftwarePlatform::Macos),
            Self::Windows => has_platform(entry, SoftwarePlatform::Windows),
            Self::Linux => has_platform(entry, SoftwarePlatform::Linux),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum EditorMode {
    New,
    Edit,
}

#[derive(Clone, PartialEq, Eq)]
struct SoftwareEditorState {
    id: Option<String>,
    slug: String,
    title: String,
    vendor: String,
    summary: String,
    homepage_url: String,
    icon_url: String,
    tag_input: String,
    trial_macos: bool,
    trial_windows: bool,
    trial_linux: bool,
    methods: Vec<SoftwareInstallMethodDto>,
}

impl Default for SoftwareEditorState {
    fn default() -> Self {
        Self {
            id: None,
            slug: String::new(),
            title: String::new(),
            vendor: String::new(),
            summary: String::new(),
            homepage_url: String::new(),
            icon_url: String::new(),
            tag_input: String::new(),
            trial_macos: true,
            trial_windows: false,
            trial_linux: false,
            methods: vec![empty_method(SoftwarePlatform::Macos)],
        }
    }
}

impl SoftwareEditorState {
    fn from_entry(entry: &SoftwareEntryDto) -> Self {
        Self {
            id: Some(entry.id.clone()),
            slug: entry.slug.clone(),
            title: entry.title.clone(),
            vendor: entry.vendor.clone(),
            summary: entry.summary.clone(),
            homepage_url: entry.homepage_url.clone(),
            icon_url: entry.icon_url.clone(),
            tag_input: entry.tags.join(", "),
            trial_macos: entry.trial_platforms.contains(&SoftwarePlatform::Macos),
            trial_windows: entry.trial_platforms.contains(&SoftwarePlatform::Windows),
            trial_linux: entry.trial_platforms.contains(&SoftwarePlatform::Linux),
            methods: entry.methods.clone(),
        }
    }

    fn into_input(self) -> SoftwareEntryInput {
        let mut trial_platforms = Vec::new();
        if self.trial_macos {
            trial_platforms.push(SoftwarePlatform::Macos);
        }
        if self.trial_windows {
            trial_platforms.push(SoftwarePlatform::Windows);
        }
        if self.trial_linux {
            trial_platforms.push(SoftwarePlatform::Linux);
        }

        let methods = self
            .methods
            .into_iter()
            .filter(|method| {
                !method.label.trim().is_empty()
                    || !method.package_id.trim().is_empty()
                    || !method.command.trim().is_empty()
            })
            .collect();

        SoftwareEntryInput {
            id: self.id,
            slug: self.slug,
            title: self.title,
            vendor: self.vendor,
            summary: self.summary,
            homepage_url: self.homepage_url,
            icon_url: self.icon_url,
            trial_platforms,
            tags: split_tags(&self.tag_input),
            methods,
        }
    }
}

#[component]
pub fn KnowledgeSoftwareMarket() -> Element {
    let services = use_context::<AppServices>();
    let software_catalog = services.software_catalog.clone();
    let asset_graph = services.asset_graph.clone();
    let feedback = use_signal(|| None::<String>);
    let filter = use_signal(|| PlatformFilter::Host);
    let search = use_signal(String::new);
    let mut selected_id = use_signal(|| None::<String>);
    let mut editor = use_signal(SoftwareEditorState::default);
    let mut editor_mode = use_signal(|| EditorMode::New);

    let catalog_resource = {
        let software_catalog = software_catalog.clone();
        use_resource(move || {
            let software_catalog = software_catalog.clone();
            async move { software_catalog.catalog().await }
        })
    };
    let package_assets_resource = {
        let asset_graph = asset_graph.clone();
        use_resource(move || {
            let asset_graph = asset_graph.clone();
            async move { asset_graph.graph().await }
        })
    };
    let save_software_catalog = software_catalog.clone();
    let fetch_software_catalog = software_catalog.clone();
    let draft_software_catalog = software_catalog.clone();
    let delete_software_catalog = software_catalog.clone();

    let catalog = match catalog_resource.read().as_ref() {
        Some(Ok(catalog)) => catalog.clone(),
        Some(Err(err)) => {
            return rsx! {
                ContentHeader {
                    title: "知识库 · 软件市场".to_string(),
                    subtitle: "管理不同操作系统的软件目录、包管理器命令和安装包入口。".to_string()
                }
                Surface { div { class: "callout", "无法加载软件市场：{err}" } }
            };
        }
        None => {
            return rsx! {
                ContentHeader {
                    title: "知识库 · 软件市场".to_string(),
                    subtitle: "管理不同操作系统的软件目录、包管理器命令和安装包入口。".to_string()
                }
                Surface { div { class: "empty-state", "正在加载软件目录…" } }
            };
        }
    };

    let query = search.read().trim().to_lowercase();
    let visible_items = catalog
        .items
        .iter()
        .filter(|item| filter.read().matches(item, catalog.host_platform))
        .filter(|item| matches_query(item, &query))
        .cloned()
        .collect::<Vec<_>>();
    let package_assets = package_assets_resource
        .read()
        .as_ref()
        .and_then(|state| state.as_ref().ok())
        .map(package_asset_items)
        .unwrap_or_default();
    let package_assets_error = package_assets_resource
        .read()
        .as_ref()
        .and_then(|state| state.as_ref().err())
        .map(|err| err.to_string());

    rsx! {
        ContentHeader {
            title: "知识库 · 软件市场".to_string(),
            subtitle: "默认按本机平台过滤展示，但后台可以同时维护 Windows / macOS / Linux 软件和安装命令。".to_string()
        }

        ResponsiveGrid { columns: 4,
            StatTile {
                label: "软件条目".to_string(),
                value: catalog.items.len().to_string(),
                detail: "目录条目，不等于安装包文件。".to_string()
            }
            StatTile {
                label: "安装方式".to_string(),
                value: catalog.items.iter().map(|item| item.methods.len()).sum::<usize>().to_string(),
                detail: "brew / winget / bun / curl 等命令并存。".to_string()
            }
            StatTile {
                label: "本机可见".to_string(),
                value: visible_items.len().to_string(),
                detail: format!("当前过滤：{}", filter.read().label(catalog.host_platform))
            }
            StatTile {
                label: "试用平台".to_string(),
                value: catalog.items.iter().map(|item| item.trial_platforms.len()).sum::<usize>().to_string(),
                detail: "可手动标注平台，而不是从命令推断。".to_string()
            }
        }

        if let Some(msg) = feedback.read().clone() {
            Surface { div { class: "callout callout--info", "{msg}" } }
        }

        div { class: "software-market-layout",
            SoftwareCatalogPanel {
                catalog: catalog.clone(),
                visible_items: visible_items.clone(),
                selected_id,
                filter,
                search,
                on_select: move |entry: SoftwareEntryDto| {
                    selected_id.set(Some(entry.id.clone()));
                    editor_mode.set(EditorMode::Edit);
                    editor.set(SoftwareEditorState::from_entry(&entry));
                },
                on_create: move |_| {
                    selected_id.set(None);
                    editor_mode.set(EditorMode::New);
                    editor.set(SoftwareEditorState::default());
                }
            }
            SoftwareEditorPanel {
                editor,
                editor_mode: *editor_mode.read(),
                host_platform: catalog.host_platform,
                package_assets,
                package_assets_error,
                feedback,
                on_save: move |_| {
                    let payload = editor.read().clone().into_input();
                    let software_catalog = save_software_catalog.clone();
                    let mut feedback = feedback;
                    let mut catalog_resource = catalog_resource;
                    let mut selected_id = selected_id;
                    let mut editor_mode = editor_mode;
                    let mut editor = editor;
                    spawn(async move {
                        match software_catalog.save_entry(payload).await {
                            Ok(saved) => {
                                selected_id.set(Some(saved.id.clone()));
                                editor_mode.set(EditorMode::Edit);
                                editor.set(SoftwareEditorState::from_entry(&saved));
                                feedback.set(Some(format!(
                                    "已保存 {}，现在可同时管理 {} 个安装方式。",
                                    saved.title,
                                    saved.methods.len()
                                )));
                                catalog_resource.restart();
                            }
                            Err(err) => feedback.set(Some(format!("保存失败：{err}"))),
                        }
                    });
                },
                on_fetch_metadata: move |_| {
                    let payload = SoftwareMetadataFetchInput {
                        homepage_url: editor.read().homepage_url.clone(),
                    };
                    let software_catalog = fetch_software_catalog.clone();
                    let mut feedback = feedback;
                    let mut editor = editor;
                    spawn(async move {
                        feedback.set(Some("正在抓取官网标题、描述和 favicon…".to_string()));
                        match software_catalog.fetch_metadata(payload).await {
                            Ok(metadata) => {
                                let mut next = editor.read().clone();
                                if next.title.trim().is_empty() {
                                    next.title = metadata.title.clone();
                                }
                                if next.summary.trim().is_empty() {
                                    next.summary = metadata.summary.clone();
                                }
                                next.homepage_url = metadata.homepage_url.clone();
                                if !metadata.icon_url.trim().is_empty() {
                                    next.icon_url = metadata.icon_url.clone();
                                }
                                editor.set(next);
                                feedback.set(Some("官网元数据已回填。".to_string()));
                            }
                            Err(err) => feedback.set(Some(format!("抓取失败：{err}"))),
                        }
                    });
                },
                on_build_draft: move |_| {
                    let payload = SoftwareDraftInput {
                        homepage_url: editor.read().homepage_url.clone(),
                        preferred_platforms: preferred_platforms(&editor.read()),
                    };
                    let software_catalog = draft_software_catalog.clone();
                    let mut feedback = feedback;
                    let mut editor = editor;
                    spawn(async move {
                        feedback.set(Some("正在根据官网生成软件草稿…".to_string()));
                        match software_catalog.build_draft(payload).await {
                            Ok(draft) => {
                                let mut next = editor.read().clone();
                                next.slug = draft.slug;
                                next.title = draft.title;
                                next.vendor = draft.vendor;
                                next.summary = draft.summary;
                                next.homepage_url = draft.homepage_url;
                                next.icon_url = draft.icon_url;
                                next.tag_input = draft.tags.join(", ");
                                next.trial_macos =
                                    draft.trial_platforms.contains(&SoftwarePlatform::Macos);
                                next.trial_windows =
                                    draft.trial_platforms.contains(&SoftwarePlatform::Windows);
                                next.trial_linux =
                                    draft.trial_platforms.contains(&SoftwarePlatform::Linux);
                                next.methods = if draft.methods.is_empty() {
                                    vec![empty_method(SoftwarePlatform::Macos)]
                                } else {
                                    draft.methods
                                };
                                editor.set(next);
                                feedback.set(Some(
                                    "软件草稿已生成，保存前请确认包管理器 ID、curl 链接和安装包关联。"
                                        .to_string(),
                                ));
                            }
                            Err(err) => feedback.set(Some(format!("生成草稿失败：{err}"))),
                        }
                    });
                },
                on_delete: move |_| {
                    let mut feedback = feedback;
                    let Some(id) = editor.read().id.clone() else {
                        feedback.set(Some("当前条目还没有保存，不能删除。".to_string()));
                        return;
                    };
                    let software_catalog = delete_software_catalog.clone();
                    let mut selected_id = selected_id;
                    let mut editor = editor;
                    let mut editor_mode = editor_mode;
                    let mut catalog_resource = catalog_resource;
                    spawn(async move {
                        match software_catalog.delete_entry(id).await {
                            Ok(()) => {
                                selected_id.set(None);
                                editor.set(SoftwareEditorState::default());
                                editor_mode.set(EditorMode::New);
                                feedback.set(Some("软件条目已删除。".to_string()));
                                catalog_resource.restart();
                            }
                            Err(err) => feedback.set(Some(format!("删除失败：{err}"))),
                        }
                    });
                }
            }
        }
    }
}

#[component]
fn SoftwareCatalogPanel(
    catalog: SoftwareCatalogDto,
    visible_items: Vec<SoftwareEntryDto>,
    selected_id: Signal<Option<String>>,
    filter: Signal<PlatformFilter>,
    search: Signal<String>,
    on_select: EventHandler<SoftwareEntryDto>,
    on_create: EventHandler<()>,
) -> Element {
    rsx! {
        Surface {
            SurfaceHeader {
                title: "软件目录".to_string(),
                subtitle: "默认只看本机可安装的软件，但过滤只是视图，不限制你维护其它平台。".to_string(),
                actions: rsx!(
                    WorkbenchButton {
                        class: "action-button action-button--primary".to_string(),
                        onclick: move |_| on_create.call(()),
                        "新增软件"
                    }
                )
            }
            div { class: "software-filter-row",
                for option in PlatformFilter::ALL {
                    button {
                        class: if *filter.read() == option { "form-tab form-tab--active" } else { "form-tab" },
                        onclick: {
                            let mut filter = filter;
                            move |_| filter.set(option)
                        },
                        "{option.label(catalog.host_platform)}"
                    }
                }
            }
            div { class: "toolbar",
                input {
                    class: "toolbar__search",
                    placeholder: "按名称 / slug / 标签 / 包名搜索",
                    value: search.read().clone(),
                    oninput: {
                        let mut search = search;
                        move |evt| search.set(evt.value())
                    }
                }
                span { class: "toolbar__spacer" }
                span { class: "cell-overflow", "{visible_items.len()} / {catalog.items.len()}" }
            }
            if visible_items.is_empty() {
                div { class: "empty-state", "当前过滤下没有软件条目。" }
            } else {
                div { class: "software-card-list",
                    for entry in visible_items.into_iter() {
                        SoftwareCatalogCard {
                            entry: entry.clone(),
                            selected: selected_id.read().as_ref() == Some(&entry.id),
                            host_platform: catalog.host_platform,
                            onclick: move |_| on_select.call(entry.clone())
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn SoftwareCatalogCard(
    entry: SoftwareEntryDto,
    selected: bool,
    host_platform: SoftwarePlatform,
    onclick: EventHandler<MouseEvent>,
) -> Element {
    let host_methods = entry
        .methods
        .iter()
        .filter(|method| method.platform == host_platform)
        .count();

    rsx! {
        button {
            class: if selected { "software-card software-card--selected" } else { "software-card" },
            onclick,
            div { class: "software-card__head",
                if !entry.icon_url.is_empty() {
                    img { class: "software-card__icon", src: "{entry.icon_url}", alt: "{entry.title}" }
                } else {
                    div { class: "software-card__icon software-card__icon--fallback", "{entry.title.chars().next().unwrap_or('S')}" }
                }
                div { class: "software-card__meta",
                    h3 { class: "software-card__title", "{entry.title}" }
                    div { class: "software-card__vendor", "{entry.vendor}" }
                }
            }
            p { class: "software-card__summary", "{entry.summary}" }
            div { class: "software-card__chips",
                for platform in entry.trial_platforms.iter().copied() {
                    span { class: "badge badge--fs", "{platform.label()}" }
                }
                for tag in entry.tags.iter().take(3) {
                    span { class: "badge", "{tag}" }
                }
            }
            div { class: "software-card__footer",
                span { "{entry.methods.len()} 条安装方式" }
                span { "本机可用 {host_methods} 条" }
            }
        }
    }
}

#[component]
fn SoftwareEditorPanel(
    editor: Signal<SoftwareEditorState>,
    editor_mode: EditorMode,
    host_platform: SoftwarePlatform,
    package_assets: Vec<AssetGraphItemDto>,
    package_assets_error: Option<String>,
    feedback: Signal<Option<String>>,
    on_save: EventHandler<()>,
    on_fetch_metadata: EventHandler<()>,
    on_build_draft: EventHandler<()>,
    on_delete: EventHandler<()>,
) -> Element {
    let _ = feedback;
    let title = if editor_mode == EditorMode::New {
        "新增软件条目"
    } else {
        "编辑软件条目"
    };
    let icon_suggestions = suggested_icon_urls(&editor.read());
    let mut confirm_open = use_signal(|| false);

    rsx! {
        div { class: "software-editor-stack",
            Surface {
                SurfaceHeader {
                    title: title.to_string(),
                    subtitle: "软件本体和安装方式分开维护：试用平台是手工标注，命令列表则按平台和安装器拆开。".to_string(),
                    actions: rsx!(
                        WorkbenchButton {
                            class: "toolbar-button".to_string(),
                            onclick: move |_| on_fetch_metadata.call(()),
                            "抓取官网元数据"
                        }
                        WorkbenchButton {
                            class: "toolbar-button".to_string(),
                            onclick: move |_| on_build_draft.call(()),
                            "按官网生成草稿"
                        }
                        WorkbenchButton {
                            class: "action-button action-button--primary".to_string(),
                            onclick: move |_| on_save.call(()),
                            "保存软件"
                        }
                    )
                }
                div { class: "form-grid",
                    EditorField { label: "Slug", value: editor.read().slug.clone(), on_input: {
                        let mut editor = editor;
                        move |value: String| {
                            let mut next = editor.read().clone();
                            next.slug = value;
                            editor.set(next);
                        }
                    }}
                    EditorField { label: "软件名称", value: editor.read().title.clone(), on_input: {
                        let mut editor = editor;
                        move |value: String| {
                            let mut next = editor.read().clone();
                            next.title = value;
                            editor.set(next);
                        }
                    }}
                    EditorField { label: "厂商", value: editor.read().vendor.clone(), on_input: {
                        let mut editor = editor;
                        move |value| {
                            let mut next = editor.read().clone();
                            next.vendor = value;
                            editor.set(next);
                        }
                    }}
                    EditorField { label: "官网 URL", value: editor.read().homepage_url.clone(), on_input: {
                        let mut editor = editor;
                        move |value| {
                            let mut next = editor.read().clone();
                            next.homepage_url = value;
                            editor.set(next);
                        }
                    }}
                    EditorField { label: "图标 URL", value: editor.read().icon_url.clone(), on_input: {
                        let mut editor = editor;
                        move |value| {
                            let mut next = editor.read().clone();
                            next.icon_url = value;
                            editor.set(next);
                        }
                    }}
                }
                if !icon_suggestions.is_empty() {
                    div { class: "software-icon-suggestions",
                        div { class: "graph-composer__label", "常见图标候选" }
                        div { class: "software-icon-suggestions__list",
                            for (label, url) in icon_suggestions {
                                button {
                                    class: "entry-kind-pill",
                                    onclick: {
                                        let mut editor = editor;
                                        move |_| {
                                            let mut next = editor.read().clone();
                                            next.icon_url = url.clone();
                                            editor.set(next);
                                        }
                                    },
                                    "{label}"
                                }
                            }
                        }
                    }
                }
                if !editor.read().icon_url.trim().is_empty() {
                    div { class: "software-icon-preview",
                        img {
                            class: "software-card__icon",
                            src: "{editor.read().icon_url}",
                            alt: "{editor.read().title}"
                        }
                        div { class: "software-card__vendor", "{editor.read().icon_url}" }
                    }
                }
                div { class: "form-grid form-grid--single",
                    EditorTextarea {
                        label: "摘要",
                        value: editor.read().summary.clone(),
                        on_input: {
                            let mut editor = editor;
                            move |value| {
                                let mut next = editor.read().clone();
                                next.summary = value;
                                editor.set(next);
                            }
                        }
                    }
                    EditorField {
                        label: "标签",
                        value: editor.read().tag_input.clone(),
                        placeholder: "ide, notes, package-manager",
                        on_input: {
                            let mut editor = editor;
                            move |value| {
                                let mut next = editor.read().clone();
                                next.tag_input = value;
                                editor.set(next);
                            }
                        }
                    }
                }
                div { class: "software-trial-strip",
                    PlatformToggle {
                        label: "试用 macOS".to_string(),
                        enabled: editor.read().trial_macos,
                        onclick: {
                            let mut editor = editor;
                            move |_| {
                                let mut next = editor.read().clone();
                                next.trial_macos = !next.trial_macos;
                                editor.set(next);
                            }
                        }
                    }
                    PlatformToggle {
                        label: "试用 Windows".to_string(),
                        enabled: editor.read().trial_windows,
                        onclick: {
                            let mut editor = editor;
                            move |_| {
                                let mut next = editor.read().clone();
                                next.trial_windows = !next.trial_windows;
                                editor.set(next);
                            }
                        }
                    }
                    PlatformToggle {
                        label: "试用 Linux".to_string(),
                        enabled: editor.read().trial_linux,
                        onclick: {
                            let mut editor = editor;
                            move |_| {
                                let mut next = editor.read().clone();
                                next.trial_linux = !next.trial_linux;
                                editor.set(next);
                            }
                        }
                    }
                }
                if let Some(id) = editor.read().id.as_ref() {
                    div { class: "editor-footer",
                        WorkbenchButton {
                            class: "toolbar-button".to_string(),
                            onclick: move |_| confirm_open.set(true),
                            "删除软件"
                        }
                        span { class: "editor-footer__spacer" }
                        span { class: "cell-overflow", "ID {id}" }
                    }
                }
                ConfirmDialog {
                    open: *confirm_open.read(),
                    title: "确认删除软件".to_string(),
                    message: format!(
                        "将删除 {} 及其全部安装方式记录。该操作不可撤销。",
                        editor.read().title
                    ),
                    confirm_label: "删除".to_string(),
                    cancel_label: "取消".to_string(),
                    on_confirm: move |_| {
                        confirm_open.set(false);
                        on_delete.call(());
                    },
                    on_cancel: move |_| confirm_open.set(false),
                }
            }

            Surface {
                SurfaceHeader {
                    title: "安装方式".to_string(),
                    subtitle: format!(
                        "同一个软件下可以同时维护 {} 和其它平台的多条命令，也可以把方法挂到真实安装包资产。",
                        host_platform.label()
                    ),
                    actions: rsx!(
                        WorkbenchButton {
                            class: "toolbar-button".to_string(),
                            onclick: {
                                let mut editor = editor;
                                move |_| {
                                    let mut next = editor.read().clone();
                                    next.methods.push(empty_method(host_platform));
                                    editor.set(next);
                                }
                            },
                            "新增安装方式"
                        }
                    )
                }
                if let Some(error) = package_assets_error.clone() {
                    div { class: "callout", "安装包资产加载失败：{error}" }
                } else if package_assets.is_empty() {
                    div { class: "callout callout--info", "当前还没有可选的安装包资产。先去“知识库”同步或上传安装包后，这里就能关联。" }
                }
                div { class: "software-method-list",
                    for (index, method) in editor.read().methods.iter().cloned().enumerate() {
                        SoftwareMethodEditor {
                            index,
                            method,
                            package_assets: package_assets.clone(),
                            on_change: {
                                let mut editor = editor;
                                move |(index, method)| {
                                    let mut next = editor.read().clone();
                                    if let Some(slot) = next.methods.get_mut(index) {
                                        *slot = method;
                                    }
                                    editor.set(next);
                                }
                            },
                            on_remove: {
                                let mut editor = editor;
                                move |index| {
                                    let mut next = editor.read().clone();
                                    if next.methods.len() > 1 {
                                        next.methods.remove(index);
                                    } else {
                                        next.methods[0] = empty_method(host_platform);
                                    }
                                    editor.set(next);
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
fn SoftwareMethodEditor(
    index: usize,
    method: SoftwareInstallMethodDto,
    package_assets: Vec<AssetGraphItemDto>,
    on_change: EventHandler<(usize, SoftwareInstallMethodDto)>,
    on_remove: EventHandler<usize>,
) -> Element {
    let selected_asset = method
        .asset_item_id
        .as_deref()
        .and_then(|asset_id| package_assets.iter().find(|item| item.id == asset_id))
        .cloned();
    let asset_options = package_assets
        .iter()
        .map(|item| (item.id.clone(), package_option_label(item)))
        .collect::<Vec<_>>();
    let show_asset_selector = matches!(
        method.kind,
        InstallerKind::Curl | InstallerKind::DirectPackage
    ) || method
        .asset_item_id
        .as_deref()
        .is_some_and(|value| !value.trim().is_empty());

    rsx! {
        div { class: "software-method-card",
            div { class: "software-method-card__toolbar",
                span { class: "badge", "安装方式 #{index + 1}" }
                WorkbenchButton {
                    class: "toolbar-button".to_string(),
                    onclick: move |_| on_remove.call(index),
                    "移除"
                }
            }
            div { class: "form-grid",
                SelectField {
                    label: "平台",
                    value: method.platform.code().to_string(),
                    options: SoftwarePlatform::ALL.iter().copied().map(|platform| (platform.code().to_string(), platform.label().to_string())).collect(),
                    on_input: {
                        let mut method = method.clone();
                        move |value: String| {
                            if let Some(platform) = parse_platform(&value) {
                                method.platform = platform;
                                on_change.call((index, method.clone()));
                            }
                        }
                    }
                }
                SelectField {
                    label: "安装器",
                    value: method.kind.code().to_string(),
                    options: InstallerKind::ALL.iter().copied().map(|kind| (kind.code().to_string(), kind.label().to_string())).collect(),
                    on_input: {
                        let mut method = method.clone();
                        move |value: String| {
                            if let Some(kind) = parse_installer_kind(&value) {
                                method.kind = kind;
                                on_change.call((index, method.clone()));
                            }
                        }
                    }
                }
                EditorField {
                    label: "显示名",
                    value: method.label.clone(),
                    on_input: {
                        let mut method = method.clone();
                        move |value| {
                            method.label = value;
                            on_change.call((index, method.clone()));
                        }
                    }
                }
                EditorField {
                    label: "包名 / 文件名",
                    value: method.package_id.clone(),
                    on_input: {
                        let mut method = method.clone();
                        move |value| {
                            method.package_id = value;
                            on_change.call((index, method.clone()));
                        }
                    }
                }
            }
            if show_asset_selector {
                SelectField {
                    label: "关联安装包资产".to_string(),
                    value: method.asset_item_id.clone().unwrap_or_default(),
                    options: asset_options,
                    placeholder: "不关联也可以，只维护命令".to_string(),
                    on_input: {
                        let mut method = method.clone();
                        let package_assets = package_assets.clone();
                        move |value: String| {
                            let asset = package_assets
                                .iter()
                                .find(|item| item.id == value)
                                .cloned();
                            if value.trim().is_empty() {
                                method.asset_item_id = None;
                            } else {
                                method.asset_item_id = Some(value);
                            }
                            if let Some(asset) = asset {
                                if method.package_id.trim().is_empty() {
                                    method.package_id = preferred_package_id(&asset);
                                }
                                if method.command.trim().is_empty() {
                                    if let Some(command) = suggested_asset_command(
                                        method.kind,
                                        method.platform,
                                        &asset,
                                    ) {
                                        method.command = command;
                                    }
                                }
                                if method.note.trim().is_empty() {
                                    method.note = asset_note(&asset);
                                }
                            }
                            on_change.call((index, method.clone()));
                        }
                    }
                }
            }
            if let Some(asset) = selected_asset {
                LinkedAssetSummary { item: asset }
            }
            EditorTextarea {
                label: "安装命令",
                value: method.command.clone(),
                mono: true,
                on_input: {
                    let mut method = method.clone();
                    move |value| {
                        method.command = value;
                        on_change.call((index, method.clone()));
                    }
                }
            }
            EditorTextarea {
                label: "备注",
                value: method.note.clone(),
                on_input: {
                    let mut method = method.clone();
                    move |value| {
                        method.note = value;
                        on_change.call((index, method.clone()));
                    }
                }
            }
        }
    }
}

#[component]
fn LinkedAssetSummary(item: AssetGraphItemDto) -> Element {
    rsx! {
        div { class: "software-linked-asset",
            div { class: "software-linked-asset__head",
                span { class: "badge badge--fs", "已关联安装包" }
                span { class: "software-linked-asset__title", "{item.title}" }
            }
            div { class: "software-linked-asset__meta",
                span { "{item.source}" }
                if let Some(relative_path) = item.relative_path.as_ref() {
                    span { "{relative_path}" }
                } else if let Some(local_path) = item.local_path.as_ref() {
                    span { "{local_path}" }
                }
                if let Some(hash) = item.content_hash.as_ref() {
                    span { "BLAKE3 {short_hash(hash)}" }
                }
            }
            if let Some(url) = item.download_url.as_ref() {
                a { class: "asset-item-card__link", href: "{url}", target: "_blank", "打开下载链接" }
            }
        }
    }
}

#[component]
fn EditorField(
    label: String,
    value: String,
    #[props(default = String::new())] placeholder: String,
    on_input: EventHandler<String>,
) -> Element {
    rsx! {
        label { class: "field",
            span { class: "field__label", "{label}" }
            input {
                class: "field__input",
                value: "{value}",
                placeholder: "{placeholder}",
                oninput: move |evt| on_input.call(evt.value())
            }
        }
    }
}

#[component]
fn SelectField(
    label: String,
    value: String,
    options: Vec<(String, String)>,
    #[props(default = String::new())] placeholder: String,
    on_input: EventHandler<String>,
) -> Element {
    rsx! {
        label { class: "field",
            span { class: "field__label", "{label}" }
            select {
                class: "field__input",
                value: "{value}",
                onchange: move |evt| on_input.call(evt.value()),
                if !placeholder.is_empty() {
                    option { value: "", selected: value.is_empty(), "{placeholder}" }
                }
                for option in options {
                    option { value: "{option.0}", selected: option.0 == value, "{option.1}" }
                }
            }
        }
    }
}

#[component]
fn EditorTextarea(
    label: String,
    value: String,
    #[props(default = false)] mono: bool,
    on_input: EventHandler<String>,
) -> Element {
    rsx! {
        label { class: "textarea",
            span { class: "field__label", "{label}" }
            textarea {
                class: if mono { "textarea__input textarea__input--mono" } else { "textarea__input" },
                value: "{value}",
                oninput: move |evt| on_input.call(evt.value())
            }
        }
    }
}

#[component]
fn PlatformToggle(label: String, enabled: bool, onclick: EventHandler<MouseEvent>) -> Element {
    rsx! {
        button {
            class: if enabled { "segment-button segment-button--active" } else { "segment-button" },
            onclick,
            "{label}"
        }
    }
}

#[component]
pub fn KnowledgeSoftwareMarketContext() -> Element {
    let software_catalog = use_context::<AppServices>().software_catalog.clone();
    let catalog = use_resource(move || {
        let software_catalog = software_catalog.clone();
        async move { software_catalog.catalog().await }
    });
    let snapshot = catalog
        .read()
        .as_ref()
        .and_then(|result| result.clone().ok());

    rsx! {
        SidebarSection { label: "软件市场".to_string(),
            Stack {
                MetricRow {
                    label: "本机平台".to_string(),
                    value: snapshot.as_ref().map(|catalog| catalog.host_platform.label().to_string()).unwrap_or_else(|| "加载中".to_string()),
                    tone: Tone::Accent
                }
                MetricRow {
                    label: "软件".to_string(),
                    value: snapshot.as_ref().map(|catalog| catalog.items.len().to_string()).unwrap_or_else(|| "加载中".to_string())
                }
                MetricRow {
                    label: "安装方式".to_string(),
                    value: snapshot.as_ref().map(|catalog| catalog.items.iter().map(|item| item.methods.len()).sum::<usize>().to_string()).unwrap_or_else(|| "加载中".to_string())
                }
            }
        }
        SidebarSection { label: "规则".to_string(),
            div { class: "callout callout--info",
                "默认过滤只影响展示，不限制你维护其它平台的软件和命令。"
            }
        }
    }
}

fn has_platform(entry: &SoftwareEntryDto, platform: SoftwarePlatform) -> bool {
    entry.trial_platforms.contains(&platform)
        || entry
            .methods
            .iter()
            .any(|method| method.platform == platform)
}

fn matches_query(entry: &SoftwareEntryDto, query: &str) -> bool {
    if query.is_empty() {
        return true;
    }
    let query = query.trim();
    entry.title.to_lowercase().contains(query)
        || entry.slug.to_lowercase().contains(query)
        || entry.vendor.to_lowercase().contains(query)
        || entry.summary.to_lowercase().contains(query)
        || entry.homepage_url.to_lowercase().contains(query)
        || entry
            .tags
            .iter()
            .any(|tag: &String| tag.to_lowercase().contains(query))
        || entry.methods.iter().any(|method| {
            method.label.to_lowercase().contains(query)
                || method.package_id.to_lowercase().contains(query)
                || method.command.to_lowercase().contains(query)
        })
}

fn empty_method(platform: SoftwarePlatform) -> SoftwareInstallMethodDto {
    SoftwareInstallMethodDto {
        id: String::new(),
        platform,
        kind: InstallerKind::Curl,
        label: String::new(),
        package_id: String::new(),
        asset_item_id: None,
        command: String::new(),
        note: String::new(),
    }
}

fn split_tags(value: &str) -> Vec<String> {
    value
        .split(',')
        .map(|tag| tag.trim().to_string())
        .filter(|tag| !tag.is_empty())
        .collect()
}

fn preferred_platforms(editor: &SoftwareEditorState) -> Vec<SoftwarePlatform> {
    let mut platforms = Vec::new();
    if editor.trial_macos {
        platforms.push(SoftwarePlatform::Macos);
    }
    if editor.trial_windows {
        platforms.push(SoftwarePlatform::Windows);
    }
    if editor.trial_linux {
        platforms.push(SoftwarePlatform::Linux);
    }
    platforms
}

fn parse_platform(value: &str) -> Option<SoftwarePlatform> {
    match value {
        "macos" => Some(SoftwarePlatform::Macos),
        "windows" => Some(SoftwarePlatform::Windows),
        "linux" => Some(SoftwarePlatform::Linux),
        _ => None,
    }
}

fn parse_installer_kind(value: &str) -> Option<InstallerKind> {
    match value {
        "brew" => Some(InstallerKind::Brew),
        "bun" => Some(InstallerKind::Bun),
        "winget" => Some(InstallerKind::Winget),
        "scoop" => Some(InstallerKind::Scoop),
        "choco" => Some(InstallerKind::Choco),
        "curl" => Some(InstallerKind::Curl),
        "package" => Some(InstallerKind::DirectPackage),
        "custom" => Some(InstallerKind::Custom),
        _ => None,
    }
}

fn package_asset_items(graph: &crate::services::AssetGraphDto) -> Vec<AssetGraphItemDto> {
    let mut items = graph
        .items
        .iter()
        .filter(|item| item.kind == "package")
        .cloned()
        .collect::<Vec<_>>();
    items.sort_by(|left, right| left.title.cmp(&right.title));
    items
}

fn package_option_label(item: &AssetGraphItemDto) -> String {
    let location = item
        .relative_path
        .as_deref()
        .or(item.local_path.as_deref())
        .unwrap_or(&item.source);
    format!("{} · {}", item.title, shorten_text(location, 56))
}

fn preferred_package_id(item: &AssetGraphItemDto) -> String {
    item.relative_path
        .as_deref()
        .or(item.local_path.as_deref())
        .and_then(file_name_only)
        .unwrap_or_else(|| item.title.clone())
}

fn suggested_asset_command(
    kind: InstallerKind,
    platform: SoftwarePlatform,
    item: &AssetGraphItemDto,
) -> Option<String> {
    match kind {
        InstallerKind::Curl => {
            let url = item.download_url.as_ref()?;
            let file_name = preferred_package_id(item);
            Some(format!("curl -L {url} -o {file_name}"))
        }
        InstallerKind::DirectPackage => item
            .local_path
            .as_deref()
            .or(item.relative_path.as_deref())
            .map(|path| direct_package_command(platform, path)),
        _ => None,
    }
}

fn asset_note(item: &AssetGraphItemDto) -> String {
    let mut parts = Vec::new();
    if let Some(relative_path) = item.relative_path.as_ref() {
        parts.push(format!("资产路径：{relative_path}"));
    } else if let Some(local_path) = item.local_path.as_ref() {
        parts.push(format!("本地路径：{local_path}"));
    }
    if let Some(hash) = item.content_hash.as_ref() {
        parts.push(format!("BLAKE3 {}", short_hash(hash)));
    }
    parts.join("；")
}

fn file_name_only(path: &str) -> Option<String> {
    path.rsplit(['/', '\\'])
        .next()
        .filter(|value| !value.is_empty())
        .map(str::to_string)
}

fn direct_package_command(platform: SoftwarePlatform, path: &str) -> String {
    match platform {
        SoftwarePlatform::Macos => format!("open \"{path}\""),
        SoftwarePlatform::Windows => format!("Start-Process -FilePath \"{path}\""),
        SoftwarePlatform::Linux => format!("xdg-open \"{path}\""),
    }
}

fn shorten_text(value: &str, limit: usize) -> String {
    let mut chars = value.chars();
    let shortened = chars.by_ref().take(limit).collect::<String>();
    if chars.next().is_some() {
        format!("{shortened}…")
    } else {
        shortened
    }
}

fn short_hash(hash: &str) -> String {
    hash.chars().take(12).collect()
}

fn suggested_icon_urls(editor: &SoftwareEditorState) -> Vec<(String, String)> {
    let mut keys = vec![
        editor.slug.clone(),
        editor.title.clone(),
        editor.vendor.clone(),
    ];
    keys.extend(split_tags(&editor.tag_input));

    let mut seen = std::collections::BTreeSet::new();
    let mut suggestions = Vec::new();
    for key in keys {
        let normalized = normalize_icon_key(&key);
        if normalized.is_empty() {
            continue;
        }
        let icon_slug = icon_slug_alias(&normalized).unwrap_or(&normalized);
        if seen.insert(icon_slug.to_string()) {
            suggestions.push((
                format!("{icon_slug} · Simple Icons"),
                format!("https://cdn.simpleicons.org/{icon_slug}"),
            ));
        }
    }
    suggestions
}

fn normalize_icon_key(value: &str) -> String {
    value
        .chars()
        .filter(|ch| ch.is_ascii_alphanumeric())
        .flat_map(|ch| ch.to_lowercase())
        .collect()
}

fn icon_slug_alias(value: &str) -> Option<&'static str> {
    match value {
        "cursor" | "anysphere" => Some("cursor"),
        "obsidian" => Some("obsidian"),
        "bun" | "ovensh" | "oven" => Some("bun"),
        "docker" | "dockerdesktop" => Some("docker"),
        "wechat" | "weixin" | "tencentwechat" | "tencent" => Some("wechat"),
        "homebrew" | "brew" => Some("homebrew"),
        "winget" => Some("windows"),
        "node" | "nodejs" => Some("nodedotjs"),
        "vscode" | "visualstudiocode" => Some("visualstudiocode"),
        "githubdesktop" | "github" => Some("github"),
        "chocolatey" | "choco" => Some("chocolatey"),
        "scoop" => Some("scoop"),
        _ => None,
    }
}
