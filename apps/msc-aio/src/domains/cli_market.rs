use addzero_cli_market_contract::{
    CliDocRef, CliEntryKind, CliImportFormat, CliImportMode, CliInstallMethod, CliInstallerKind,
    CliLocale, CliLocaleText, CliMarketEntry, CliMarketEntryUpsert, CliMarketExportRequest,
    CliMarketImportJobDetail, CliMarketImportRequest, CliMarketInstallHistoryItem,
    CliMarketInstallRequest, CliMarketInstallResult, CliMarketSourceType, CliMarketStatus,
    CliPlatform,
};
use base64::{Engine as _, engine::general_purpose::STANDARD};
use dioxus::prelude::*;
use dioxus_components::{
    ContentHeader, ResponsiveGrid, StatTile, Surface, SurfaceHeader, WorkbenchButton,
};

use crate::state::{AppServices, AuthSession};

#[derive(Clone, PartialEq, Eq)]
struct CliMarketEditorState {
    id: Option<String>,
    slug: String,
    status: CliMarketStatus,
    vendor_name: String,
    latest_version: String,
    homepage_url: String,
    repo_url: String,
    docs_url: String,
    entry_point: String,
    category_code: String,
    tags: String,
    display_name_zh: String,
    display_name_en: String,
    summary_zh: String,
    summary_en: String,
    requires_text_zh: String,
    requires_text_en: String,
    install_methods: Vec<CliInstallMethod>,
    doc_title: String,
    doc_url: String,
    docs_summary_zh: String,
    docs_summary_en: String,
}

impl Default for CliMarketEditorState {
    fn default() -> Self {
        Self {
            id: None,
            slug: String::new(),
            status: CliMarketStatus::Draft,
            vendor_name: String::new(),
            latest_version: String::new(),
            homepage_url: String::new(),
            repo_url: String::new(),
            docs_url: String::new(),
            entry_point: String::new(),
            category_code: String::new(),
            tags: String::new(),
            display_name_zh: String::new(),
            display_name_en: String::new(),
            summary_zh: String::new(),
            summary_en: String::new(),
            requires_text_zh: String::new(),
            requires_text_en: String::new(),
            install_methods: vec![empty_cli_install_method()],
            doc_title: String::new(),
            doc_url: String::new(),
            docs_summary_zh: String::new(),
            docs_summary_en: String::new(),
        }
    }
}

impl CliMarketEditorState {
    fn from_entry(entry: &CliMarketEntry) -> Self {
        let zh = entry
            .locales
            .iter()
            .find(|item| item.locale == CliLocale::ZhCn)
            .cloned()
            .unwrap_or_default();
        let en = entry
            .locales
            .iter()
            .find(|item| item.locale == CliLocale::EnUs)
            .cloned()
            .unwrap_or_default();
        let doc = entry.doc_refs.first().cloned().unwrap_or_default();
        Self {
            id: Some(entry.id.clone()),
            slug: entry.slug.clone(),
            status: entry.status,
            vendor_name: entry.vendor_name.clone(),
            latest_version: entry.latest_version.clone(),
            homepage_url: entry.homepage_url.clone(),
            repo_url: entry.repo_url.clone(),
            docs_url: entry.docs_url.clone(),
            entry_point: entry.entry_point.clone(),
            category_code: entry.category_code.clone(),
            tags: entry.tags.join(", "),
            display_name_zh: zh.display_name,
            display_name_en: en.display_name,
            summary_zh: zh.summary,
            summary_en: en.summary,
            requires_text_zh: zh.requires_text,
            requires_text_en: en.requires_text,
            install_methods: if entry.install_methods.is_empty() {
                vec![empty_cli_install_method()]
            } else {
                entry.install_methods.clone()
            },
            doc_title: doc.title,
            doc_url: doc.url,
            docs_summary_zh: zh.docs_summary,
            docs_summary_en: en.docs_summary,
        }
    }

    fn into_upsert(self) -> CliMarketEntryUpsert {
        let primary_method = self
            .install_methods
            .iter()
            .find(|method| !method.command_template.trim().is_empty())
            .cloned()
            .unwrap_or_else(empty_cli_install_method);

        CliMarketEntryUpsert {
            id: self.id,
            slug: self.slug,
            status: self.status,
            source_type: CliMarketSourceType::Manual,
            entry_kind: CliEntryKind::Cli,
            vendor_name: self.vendor_name,
            latest_version: self.latest_version,
            homepage_url: self.homepage_url,
            repo_url: self.repo_url,
            docs_url: self.docs_url,
            entry_point: self.entry_point,
            category_code: self.category_code,
            tags: split_tags(&self.tags),
            locales: vec![
                CliLocaleText {
                    locale: CliLocale::ZhCn,
                    display_name: self.display_name_zh,
                    summary: self.summary_zh,
                    description_md: String::new(),
                    install_guide_md: String::new(),
                    docs_summary: self.docs_summary_zh,
                    requires_text: self.requires_text_zh,
                    install_command: primary_method.command_template.clone(),
                },
                CliLocaleText {
                    locale: CliLocale::EnUs,
                    display_name: self.display_name_en,
                    summary: self.summary_en,
                    description_md: String::new(),
                    install_guide_md: String::new(),
                    docs_summary: self.docs_summary_en,
                    requires_text: self.requires_text_en,
                    install_command: primary_method.command_template.clone(),
                },
            ],
            install_methods: normalize_cli_install_methods(self.install_methods),
            doc_refs: if self.doc_url.trim().is_empty() {
                Vec::new()
            } else {
                vec![CliDocRef {
                    id: None,
                    locale: CliLocale::ZhCn,
                    title: self.doc_title,
                    url: self.doc_url,
                    version: String::new(),
                    source_label: "manual".to_string(),
                    summary: String::new(),
                }]
            },
            raw: serde_json::json!({}),
        }
    }
}

fn empty_cli_install_method() -> CliInstallMethod {
    CliInstallMethod {
        id: None,
        platform: CliPlatform::CrossPlatform,
        installer_kind: CliInstallerKind::Custom,
        package_id: String::new(),
        command_template: String::new(),
        validation_note: String::new(),
        priority: 100,
    }
}

fn normalize_cli_install_methods(methods: Vec<CliInstallMethod>) -> Vec<CliInstallMethod> {
    let total = methods.len().max(1);
    let normalized = methods
        .into_iter()
        .enumerate()
        .map(|(index, mut method)| {
            method.package_id = method.package_id.trim().to_string();
            method.command_template = method.command_template.trim().to_string();
            method.validation_note = method.validation_note.trim().to_string();
            method.priority = i32::try_from(total.saturating_sub(index)).unwrap_or_default();
            method
        })
        .filter(|method| {
            !method.command_template.is_empty()
                || !method.package_id.is_empty()
                || !method.validation_note.is_empty()
        })
        .collect::<Vec<_>>();

    if normalized.is_empty() {
        vec![empty_cli_install_method()]
    } else {
        normalized
    }
}

#[component]
pub fn KnowledgeCliMarket() -> Element {
    let services = use_context::<AppServices>();
    let cli_market = services.cli_market.clone();
    let feedback = use_signal(|| None::<String>);
    let export_feedback = use_signal(|| None::<String>);
    let mut selected_id = use_signal(|| None::<String>);
    let mut selected_install_method_id = use_signal(|| None::<String>);
    let mut editor = use_signal(CliMarketEditorState::default);
    let install_result = use_signal(|| None::<CliMarketInstallResult>);
    let installing = use_signal(|| false);
    let mut catalog_resource = {
        let cli_market = cli_market.clone();
        use_resource(move || {
            let cli_market = cli_market.clone();
            async move { cli_market.catalog().await }
        })
    };
    let mut install_history_resource = {
        let cli_market = cli_market.clone();
        let selected_id = selected_id;
        use_resource(move || {
            let cli_market = cli_market.clone();
            let selected_id = selected_id();
            async move {
                match selected_id {
                    Some(id) => cli_market.install_history(id).await,
                    None => Ok(Vec::new()),
                }
            }
        })
    };

    let catalog = match catalog_resource.read().as_ref() {
        Some(Ok(catalog)) => catalog.clone(),
        Some(Err(err)) => {
            return rsx! {
                CliMarketHeader {}
                Surface { div { class: "callout", "无法加载 CLI 市场：{err}" } }
            };
        }
        None => {
            return rsx! {
                CliMarketHeader {}
                Surface { div { class: "empty-state", "正在加载 CLI 市场…" } }
            };
        }
    };
    let selected_entry = selected_id.read().as_ref().and_then(|id| {
        catalog
            .entries
            .iter()
            .find(|entry| entry.id == *id)
            .cloned()
    });
    let install_history = install_history_resource
        .read()
        .as_ref()
        .and_then(|state| state.as_ref().ok())
        .cloned()
        .unwrap_or_default();
    let install_history_error = install_history_resource
        .read()
        .as_ref()
        .and_then(|state| state.as_ref().err())
        .map(|err| err.to_string());

    rsx! {
        CliMarketHeader {}
        ResponsiveGrid { columns: 4,
            StatTile { label: "条目总数".to_string(), value: catalog.summary.total_entries.to_string(), detail: "PG 正式注册表".to_string() }
            StatTile { label: "已发布".to_string(), value: catalog.summary.published_entries.to_string(), detail: "public registry 可见".to_string() }
            StatTile { label: "导入任务".to_string(), value: catalog.summary.import_jobs.to_string(), detail: "JSON / XLSX".to_string() }
            StatTile { label: "分类数".to_string(), value: catalog.summary.categories.to_string(), detail: "分类代码".to_string() }
        }
        if let Some(message) = feedback.read().clone() {
            Surface { div { class: "callout callout--info", "{message}" } }
        }
        if let Some(message) = export_feedback.read().clone() {
            Surface { div { class: "callout", "{message}" } }
        }
        div { class: "software-market-layout",
            Surface {
                SurfaceHeader {
                    title: "CLI 注册表".to_string(),
                    subtitle: "维护中英双语条目、安装方式和文档引用。".to_string(),
                    actions: rsx!(
                        WorkbenchButton {
                            class: "action-button".to_string(),
                            onclick: {
                                let mut selected_id = selected_id;
                                let mut selected_install_method_id = selected_install_method_id;
                                let mut editor = editor;
                                let mut install_result = install_result;
                                move |_| {
                                    selected_id.set(None);
                                    selected_install_method_id.set(None);
                                    editor.set(CliMarketEditorState::default());
                                    install_result.set(None);
                                }
                            },
                            "新建条目"
                        }
                        WorkbenchButton {
                            class: "action-button".to_string(),
                            onclick: {
                                let cli_market = cli_market.clone();
                                let mut export_feedback = export_feedback;
                                move |_| {
                                    let cli_market = cli_market.clone();
                                    spawn(async move {
                                        match cli_market.export_json(CliMarketExportRequest { only_published: false, locale: None }).await {
                                            Ok(file) => export_feedback.set(Some(format!("已生成 {}。", file.file_name))),
                                            Err(err) => export_feedback.set(Some(format!("导出 JSON 失败：{err}"))),
                                        }
                                    });
                                }
                            },
                            "导出 JSON"
                        }
                        WorkbenchButton {
                            class: "action-button".to_string(),
                            onclick: {
                                let cli_market = cli_market.clone();
                                let mut export_feedback = export_feedback;
                                move |_| {
                                    let cli_market = cli_market.clone();
                                    spawn(async move {
                                        match cli_market.export_xlsx(CliMarketExportRequest { only_published: false, locale: None }).await {
                                            Ok(file) => export_feedback.set(Some(format!("已生成 {}。", file.file_name))),
                                            Err(err) => export_feedback.set(Some(format!("导出 XLSX 失败：{err}"))),
                                        }
                                    });
                                }
                            },
                            "导出 XLSX"
                        }
                    )
                }
                div { class: "stack",
                    for entry in catalog.entries.iter() {
                        button {
                            class: if selected_id.read().as_ref() == Some(&entry.id) { "software-card software-card--selected" } else { "software-card" },
                            onclick: {
                                let entry = entry.clone();
                                let mut editor = editor;
                                let mut selected_id = selected_id;
                                let mut selected_install_method_id = selected_install_method_id;
                                let mut install_result = install_result;
                                let mut install_history_resource = install_history_resource;
                                move |_| {
                                    selected_id.set(Some(entry.id.clone()));
                                    selected_install_method_id.set(
                                        entry.install_methods.first().and_then(|method| method.id.clone())
                                    );
                                    editor.set(CliMarketEditorState::from_entry(&entry));
                                    install_result.set(None);
                                    install_history_resource.restart();
                                }
                            },
                            div { class: "software-card__title",
                                "{entry.locales.iter().find(|item| item.locale == CliLocale::ZhCn).map(|item| item.display_name.clone()).unwrap_or_else(|| entry.slug.clone())}"
                            }
                            div { class: "software-card__meta", "{entry.slug} · {entry.category_code}" }
                            div { class: "software-card__summary", "{entry.vendor_name}" }
                        }
                    }
                }
            }
            CliMarketEditorPanel {
                editor,
                selected_entry,
                selected_install_method_id,
                install_history,
                install_history_error,
                feedback,
                install_result,
                installing,
                on_saved: move |saved: CliMarketEntry| {
                    selected_id.set(Some(saved.id.clone()));
                    selected_install_method_id.set(
                        saved.install_methods.first().and_then(|method| method.id.clone())
                    );
                    editor.set(CliMarketEditorState::from_entry(&saved));
                    catalog_resource.restart();
                    install_history_resource.restart();
                },
                on_installed: move |_| install_history_resource.restart()
            }
        }
        if *installing.read() {
            Surface { div { class: "callout callout--info", "正在当前主机执行安装命令…" } }
        }
        if let Some(result) = install_result.read().clone() {
            InstallResultSurface { result }
        }
        Surface {
            SurfaceHeader { title: "公开注册表".to_string(), subtitle: "仅展示已发布条目。".to_string() }
            div { class: "stack",
                a { href: "/api/cli-market/public/registry.json", target: "_blank", "registry.json" }
                a { href: "/api/cli-market/public/registry.xlsx", target: "_blank", "registry.xlsx" }
            }
        }
    }
}

#[component]
pub fn KnowledgeCliMarketImports() -> Element {
    let cli_market = use_context::<AppServices>().cli_market.clone();
    let auth = use_context::<AuthSession>();
    let feedback = use_signal(|| None::<String>);
    let import_mode = use_signal(|| CliImportMode::Native);
    let import_format = use_signal(|| CliImportFormat::Json);
    let selected_job_id = use_signal(|| None::<String>);
    let jobs_resource = {
        let cli_market = cli_market.clone();
        use_resource(move || {
            let cli_market = cli_market.clone();
            async move { cli_market.import_jobs().await }
        })
    };
    let job_detail_resource = {
        let cli_market = cli_market.clone();
        let selected_job_id = selected_job_id;
        use_resource(move || {
            let cli_market = cli_market.clone();
            let selected_job_id = selected_job_id();
            async move {
                match selected_job_id {
                    Some(id) => cli_market.import_job_detail(id).await,
                    None => Ok(None),
                }
            }
        })
    };

    let jobs = jobs_resource
        .read()
        .as_ref()
        .and_then(|state| state.as_ref().ok())
        .cloned()
        .unwrap_or_default();

    rsx! {
        ContentHeader {
            title: "知识库 · CLI 市场 · 导入任务".to_string(),
            subtitle: "支持 JSON / XLSX 批量导入。".to_string()
        }
        if let Some(message) = feedback.read().clone() {
            Surface { div { class: "callout", "{message}" } }
        }
        Surface {
            SurfaceHeader { title: "上传导入文件".to_string(), subtitle: "原生 JSON 支持完整双语字段；兼容模式支持 registry shape。".to_string() }
            div { class: "stack",
                select {
                    value: match *import_format.read() { CliImportFormat::Json => "json", CliImportFormat::Xlsx => "xlsx" },
                    onchange: {
                        let mut import_format = import_format;
                        move |evt| import_format.set(if evt.value() == "xlsx" { CliImportFormat::Xlsx } else { CliImportFormat::Json })
                    },
                    option { value: "json", "JSON" }
                    option { value: "xlsx", "XLSX" }
                }
                select {
                    value: match *import_mode.read() { CliImportMode::Native => "native", CliImportMode::RegistryCompat => "compat" },
                    onchange: {
                        let mut import_mode = import_mode;
                        move |evt| import_mode.set(if evt.value() == "compat" { CliImportMode::RegistryCompat } else { CliImportMode::Native })
                    },
                    option { value: "native", "原生模式" }
                    option { value: "compat", "兼容模式" }
                }
                input {
                    r#type: "file",
                    onchange: move |evt| {
                        let Some(file) = evt.files().into_iter().next() else {
                            return;
                        };
                        let username = auth.username.read().clone();
                        let selected_format = *import_format.read();
                        let selected_mode = *import_mode.read();
                        let cli_market = cli_market.clone();
                        let mut feedback = feedback;
                        let mut jobs_resource = jobs_resource;
                        let mut selected_job_id = selected_job_id;
                        let mut job_detail_resource = job_detail_resource;
                        feedback.set(Some("正在读取并导入文件…".to_string()));
                        spawn(async move {
                            match file.read_bytes().await {
                                Ok(bytes) => {
                                    let request = CliMarketImportRequest {
                                        format: selected_format,
                                        mode: selected_mode,
                                        file_name: file.name(),
                                        payload_base64: STANDARD.encode(bytes.as_ref()),
                                        submitted_by: username,
                                    };
                                    match cli_market.import_entries(request).await {
                                        Ok(report) => {
                                            feedback.set(Some(format!("导入完成：成功 {} 行，失败 {} 行。", report.success_rows, report.failed_rows)));
                                            jobs_resource.restart();
                                            selected_job_id.set(Some(report.job_id.clone()));
                                            job_detail_resource.restart();
                                        }
                                        Err(err) => feedback.set(Some(format!("导入失败：{err}"))),
                                    }
                                }
                                Err(err) => feedback.set(Some(format!("读取文件失败：{err}"))),
                            }
                        });
                    }
                }
            }
        }
        Surface {
            SurfaceHeader { title: "导入任务".to_string(), subtitle: "最近任务倒序排列。".to_string() }
            table { class: "data-table",
                thead { tr { th { "文件" } th { "格式" } th { "模式" } th { "成功/失败" } th { "提交人" } th { "时间" } } }
                tbody {
                    for job in jobs {
                        tr {
                            td {
                                button {
                                    class: "link-button",
                                    onclick: {
                                        let job_id = job.id.clone();
                                        let mut selected_job_id = selected_job_id;
                                        let mut job_detail_resource = job_detail_resource;
                                        move |_| {
                                            selected_job_id.set(Some(job_id.clone()));
                                            job_detail_resource.restart();
                                        }
                                    },
                                    "{job.file_name}"
                                }
                            }
                            td { if job.format == CliImportFormat::Xlsx { "XLSX" } else { "JSON" } }
                            td { if job.mode == CliImportMode::RegistryCompat { "兼容" } else { "原生" } }
                            td { "{job.success_rows}/{job.failed_rows}" }
                            td { "{job.submitted_by}" }
                            td { "{job.created_at.clone().unwrap_or_default()}" }
                        }
                    }
                }
            }
        }
        match job_detail_resource.read().as_ref() {
            Some(Ok(Some(detail))) => rsx! { ImportJobDetailSurface { detail: detail.clone() } },
            Some(Ok(None)) => rsx! {},
            Some(Err(err)) => rsx! { Surface { div { class: "callout", "无法加载导入详情：{err}" } } },
            None => rsx! {},
        }
    }
}

#[component]
pub fn KnowledgeCliMarketDocs() -> Element {
    let cli_market = use_context::<AppServices>().cli_market.clone();
    let resource = {
        let cli_market = cli_market.clone();
        use_resource(move || {
            let cli_market = cli_market.clone();
            async move { cli_market.catalog().await }
        })
    };
    let entries = resource
        .read()
        .as_ref()
        .and_then(|state| state.as_ref().ok())
        .map(|catalog| catalog.entries.clone())
        .unwrap_or_default();
    rsx! {
        ContentHeader {
            title: "知识库 · CLI 市场 · CLI 文档".to_string(),
            subtitle: "查看条目的文档引用和维护入口。".to_string()
        }
        for entry in entries {
            Surface {
                SurfaceHeader {
                    title: entry.slug.clone(),
                    subtitle: entry.locales.iter().find(|item| item.locale == CliLocale::ZhCn).map(|item| item.summary.clone()).unwrap_or_default()
                }
                div { class: "stack",
                    for doc in entry.doc_refs {
                        div { class: "settings-note",
                            strong { "{doc.title}" }
                            div { "{doc.url}" }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn CliMarketHeader() -> Element {
    rsx! {
        ContentHeader {
            title: "知识库 · CLI 市场".to_string(),
            subtitle: "CLI 注册表、安装入口与文档元数据统一落在 PostgreSQL。".to_string()
        }
    }
}

#[derive(Props, Clone, PartialEq)]
struct InstallResultSurfaceProps {
    result: CliMarketInstallResult,
}

#[component]
fn InstallResultSurface(props: InstallResultSurfaceProps) -> Element {
    let tone = if props.result.success {
        "安装成功"
    } else {
        "安装失败"
    };
    rsx! {
        Surface {
            SurfaceHeader {
                title: format!("安装结果 · {} · {}", props.result.slug, tone),
                subtitle: format!(
                    "installer={} platform={} exit_code={:?}",
                    props.result.installer_kind.code(),
                    props.result.platform.code(),
                    props.result.exit_code
                )
            }
            div { class: "stack",
                div { class: "settings-note", "命令：{props.result.command}" }
                if !props.result.stdout.trim().is_empty() {
                    label { class: "textarea",
                        span { class: "field__label", "stdout" }
                        textarea {
                            class: "textarea__input textarea__input--mono",
                            readonly: true,
                            value: props.result.stdout.clone()
                        }
                    }
                }
                if !props.result.stderr.trim().is_empty() {
                    label { class: "textarea",
                        span { class: "field__label", "stderr" }
                        textarea {
                            class: "textarea__input textarea__input--mono",
                            readonly: true,
                            value: props.result.stderr.clone()
                        }
                    }
                }
            }
        }
    }
}

#[derive(Props, Clone, PartialEq)]
struct ImportJobDetailSurfaceProps {
    detail: CliMarketImportJobDetail,
}

#[component]
fn ImportJobDetailSurface(props: ImportJobDetailSurfaceProps) -> Element {
    rsx! {
        Surface {
            SurfaceHeader {
                title: format!("导入详情 · {}", props.detail.job.file_name),
                subtitle: format!(
                    "总计 {} 行，成功 {} 行，失败 {} 行。",
                    props.detail.job.total_rows,
                    props.detail.job.success_rows,
                    props.detail.job.failed_rows
                )
            }
            table { class: "data-table",
                thead { tr { th { "行" } th { "slug" } th { "结果" } th { "错误" } } }
                tbody {
                    for row in props.detail.rows.iter() {
                        tr {
                            td { "{row.row_index}" }
                            td { "{row.slug}" }
                            td { if row.success { "成功" } else { "失败" } }
                            td { "{row.error.clone().unwrap_or_default()}" }
                        }
                    }
                }
            }
        }
    }
}

#[derive(Props, Clone, PartialEq)]
struct CliMarketEditorPanelProps {
    editor: Signal<CliMarketEditorState>,
    selected_entry: Option<CliMarketEntry>,
    selected_install_method_id: Signal<Option<String>>,
    install_history: Vec<CliMarketInstallHistoryItem>,
    install_history_error: Option<String>,
    feedback: Signal<Option<String>>,
    install_result: Signal<Option<CliMarketInstallResult>>,
    installing: Signal<bool>,
    on_saved: EventHandler<CliMarketEntry>,
    on_installed: EventHandler<()>,
}

#[component]
fn CliMarketEditorPanel(props: CliMarketEditorPanelProps) -> Element {
    let cli_market = use_context::<AppServices>().cli_market.clone();
    let selected_method = props.selected_entry.as_ref().and_then(|entry| {
        selected_install_method(entry, props.selected_install_method_id.read().as_deref())
    });
    let install_methods = props.editor.read().install_methods.clone();
    rsx! {
        Surface {
            SurfaceHeader {
                title: "条目编辑".to_string(),
                subtitle: "双语名称、摘要、安装方式矩阵和文档引用统一维护。".to_string(),
                actions: rsx!(
                    WorkbenchButton {
                        class: "action-button action-button--primary".to_string(),
                        onclick: {
                            let cli_market = cli_market.clone();
                            let editor = props.editor;
                            let mut selected_install_method_id = props.selected_install_method_id;
                            let mut feedback = props.feedback;
                            let on_saved = props.on_saved;
                            move |_| {
                                let cli_market = cli_market.clone();
                                let input = editor.read().clone().into_upsert();
                                spawn(async move {
                                    match cli_market.upsert_entry(input).await {
                                        Ok(saved) => {
                                            feedback.set(Some(format!("已保存：{}", saved.slug)));
                                            selected_install_method_id.set(
                                                saved.install_methods.first().and_then(|method| method.id.clone())
                                            );
                                            on_saved.call(saved);
                                        }
                                        Err(err) => feedback.set(Some(format!("保存失败：{err}"))),
                                    }
                                });
                            }
                        },
                        "保存"
                    }
                    WorkbenchButton {
                        class: "action-button".to_string(),
                        onclick: {
                            let cli_market = cli_market.clone();
                            let editor = props.editor;
                            let selected_install_method_id = props.selected_install_method_id;
                            let mut feedback = props.feedback;
                            let mut install_result = props.install_result;
                            let mut installing = props.installing;
                            let on_installed = props.on_installed;
                            move |_| {
                                let Some(id) = editor.read().id.clone() else {
                                    feedback.set(Some("请先保存条目，再执行一键安装。".to_string()));
                                    return;
                                };
                                let method_id = selected_install_method_id.read().clone();
                                installing.set(true);
                                install_result.set(None);
                                let cli_market = cli_market.clone();
                                spawn(async move {
                                    match cli_market
                                        .install_entry(
                                            id,
                                            CliMarketInstallRequest { method_id },
                                        )
                                        .await
                                    {
                                        Ok(result) => {
                                            let summary = if result.success { "安装成功" } else { "安装失败" };
                                            feedback.set(Some(format!(
                                                "{}：退出码 {:?}",
                                                summary, result.exit_code
                                            )));
                                            install_result.set(Some(result));
                                            on_installed.call(());
                                        }
                                        Err(err) => {
                                            feedback.set(Some(format!("安装执行失败：{err}")));
                                        }
                                    }
                                    installing.set(false);
                                });
                            }
                        },
                        "一键安装"
                    }
                    WorkbenchButton {
                        class: "action-button".to_string(),
                        onclick: {
                            let cli_market = cli_market.clone();
                            let editor = props.editor;
                            let mut selected_install_method_id = props.selected_install_method_id;
                            let mut feedback = props.feedback;
                            let mut local_editor = props.editor;
                            let on_saved = props.on_saved;
                            move |_| {
                                let Some(id) = editor.read().id.clone() else {
                                    feedback.set(Some("请先保存条目。".to_string()));
                                    return;
                                };
                                let cli_market = cli_market.clone();
                                spawn(async move {
                                    match cli_market.publish_entry(id).await {
                                        Ok(saved) => {
                                            feedback.set(Some(format!("已发布：{}", saved.slug)));
                                            selected_install_method_id.set(
                                                saved.install_methods.first().and_then(|method| method.id.clone())
                                            );
                                            local_editor.set(CliMarketEditorState::from_entry(&saved));
                                            on_saved.call(saved);
                                        }
                                        Err(err) => feedback.set(Some(format!("发布失败：{err}"))),
                                    }
                                });
                            }
                        },
                        "发布"
                    }
                    WorkbenchButton {
                        class: "action-button".to_string(),
                        onclick: {
                            let cli_market = cli_market.clone();
                            let editor = props.editor;
                            let mut selected_install_method_id = props.selected_install_method_id;
                            let mut feedback = props.feedback;
                            let mut local_editor = props.editor;
                            let on_saved = props.on_saved;
                            move |_| {
                                let Some(id) = editor.read().id.clone() else {
                                    feedback.set(Some("请先保存条目。".to_string()));
                                    return;
                                };
                                let cli_market = cli_market.clone();
                                spawn(async move {
                                    match cli_market.archive_entry(id).await {
                                        Ok(saved) => {
                                            feedback.set(Some(format!("已归档：{}", saved.slug)));
                                            selected_install_method_id.set(
                                                saved.install_methods.first().and_then(|method| method.id.clone())
                                            );
                                            local_editor.set(CliMarketEditorState::from_entry(&saved));
                                            on_saved.call(saved);
                                        }
                                        Err(err) => feedback.set(Some(format!("归档失败：{err}"))),
                                    }
                                });
                            }
                        },
                        "归档"
                    }
                )
            }
            div { class: "stack",
                TextField { label: "slug", value: props.editor.read().slug.clone(), on_input: {
                    let mut editor = props.editor;
                    move |value| { let mut next = editor.read().clone(); next.slug = value; editor.set(next); }
                }}
                TextField { label: "vendor_name", value: props.editor.read().vendor_name.clone(), on_input: {
                    let mut editor = props.editor;
                    move |value| { let mut next = editor.read().clone(); next.vendor_name = value; editor.set(next); }
                }}
                TextField { label: "category_code", value: props.editor.read().category_code.clone(), on_input: {
                    let mut editor = props.editor;
                    move |value| { let mut next = editor.read().clone(); next.category_code = value; editor.set(next); }
                }}
                TextField { label: "tags", value: props.editor.read().tags.clone(), on_input: {
                    let mut editor = props.editor;
                    move |value| { let mut next = editor.read().clone(); next.tags = value; editor.set(next); }
                }}
                TextField { label: "display_name_zh", value: props.editor.read().display_name_zh.clone(), on_input: {
                    let mut editor = props.editor;
                    move |value| { let mut next = editor.read().clone(); next.display_name_zh = value; editor.set(next); }
                }}
                TextField { label: "display_name_en", value: props.editor.read().display_name_en.clone(), on_input: {
                    let mut editor = props.editor;
                    move |value| { let mut next = editor.read().clone(); next.display_name_en = value; editor.set(next); }
                }}
                TextareaField { label: "summary_zh", value: props.editor.read().summary_zh.clone(), on_input: {
                    let mut editor = props.editor;
                    move |value| { let mut next = editor.read().clone(); next.summary_zh = value; editor.set(next); }
                }}
                TextareaField { label: "summary_en", value: props.editor.read().summary_en.clone(), on_input: {
                    let mut editor = props.editor;
                    move |value| { let mut next = editor.read().clone(); next.summary_en = value; editor.set(next); }
                }}
                TextareaField { label: "requires_text_zh", value: props.editor.read().requires_text_zh.clone(), on_input: {
                    let mut editor = props.editor;
                    move |value| { let mut next = editor.read().clone(); next.requires_text_zh = value; editor.set(next); }
                }}
                TextareaField { label: "requires_text_en", value: props.editor.read().requires_text_en.clone(), on_input: {
                    let mut editor = props.editor;
                    move |value| { let mut next = editor.read().clone(); next.requires_text_en = value; editor.set(next); }
                }}
                TextField { label: "doc_title", value: props.editor.read().doc_title.clone(), on_input: {
                    let mut editor = props.editor;
                    move |value| { let mut next = editor.read().clone(); next.doc_title = value; editor.set(next); }
                }}
                TextField { label: "doc_url", value: props.editor.read().doc_url.clone(), on_input: {
                    let mut editor = props.editor;
                    move |value| { let mut next = editor.read().clone(); next.doc_url = value; editor.set(next); }
                }}
                TextareaField { label: "docs_summary_zh", value: props.editor.read().docs_summary_zh.clone(), on_input: {
                    let mut editor = props.editor;
                    move |value| { let mut next = editor.read().clone(); next.docs_summary_zh = value; editor.set(next); }
                }}
                TextareaField { label: "docs_summary_en", value: props.editor.read().docs_summary_en.clone(), on_input: {
                    let mut editor = props.editor;
                    move |value| { let mut next = editor.read().clone(); next.docs_summary_en = value; editor.set(next); }
                }}
                TextField { label: "homepage_url", value: props.editor.read().homepage_url.clone(), on_input: {
                    let mut editor = props.editor;
                    move |value| { let mut next = editor.read().clone(); next.homepage_url = value; editor.set(next); }
                }}
                TextField { label: "repo_url", value: props.editor.read().repo_url.clone(), on_input: {
                    let mut editor = props.editor;
                    move |value| { let mut next = editor.read().clone(); next.repo_url = value; editor.set(next); }
                }}
                TextField { label: "docs_url", value: props.editor.read().docs_url.clone(), on_input: {
                    let mut editor = props.editor;
                    move |value| { let mut next = editor.read().clone(); next.docs_url = value; editor.set(next); }
                }}
                TextField { label: "entry_point", value: props.editor.read().entry_point.clone(), on_input: {
                    let mut editor = props.editor;
                    move |value| { let mut next = editor.read().clone(); next.entry_point = value; editor.set(next); }
                }}
                TextField { label: "latest_version", value: props.editor.read().latest_version.clone(), on_input: {
                    let mut editor = props.editor;
                    move |value| { let mut next = editor.read().clone(); next.latest_version = value; editor.set(next); }
                }}
                if let Some(entry) = props.selected_entry.clone() {
                    label { class: "field",
                        span { class: "field__label", "install_method" }
                        select {
                            class: "field__input",
                            value: props.selected_install_method_id.read().clone().unwrap_or_default(),
                            onchange: {
                                let mut selected_install_method_id = props.selected_install_method_id;
                                move |evt| selected_install_method_id.set(Some(evt.value()))
                            },
                            for method in entry.install_methods.iter() {
                                option {
                                    value: method.id.clone().unwrap_or_default(),
                                    "{install_method_label(method)}"
                                }
                            }
                        }
                    }
                }
                select {
                    value: props.editor.read().status.code().to_string(),
                    onchange: {
                        let mut editor = props.editor;
                        move |evt| {
                            let mut next = editor.read().clone();
                            next.status = CliMarketStatus::ALL.into_iter().find(|item| item.code() == evt.value()).unwrap_or(CliMarketStatus::Draft);
                            editor.set(next);
                        }
                    },
                    for item in CliMarketStatus::ALL {
                        option { value: item.code().to_string(), "{item.code()}" }
                    }
                }
                Surface {
                    SurfaceHeader {
                        title: "安装方式矩阵".to_string(),
                        subtitle: "一个 CLI 可同时维护多平台、多安装器的命令模板。".to_string(),
                        actions: rsx!(
                            WorkbenchButton {
                                class: "action-button".to_string(),
                                onclick: {
                                    let mut editor = props.editor;
                                    move |_| {
                                        let mut next = editor.read().clone();
                                        next.install_methods.push(empty_cli_install_method());
                                        editor.set(next);
                                    }
                                },
                                "新增安装方式"
                            }
                        )
                    }
                    div { class: "software-method-list",
                        for (index, method) in install_methods.iter().cloned().enumerate() {
                            CliInstallMethodEditor {
                                index,
                                method,
                                on_change: {
                                    let mut editor = props.editor;
                                    move |(index, method): (usize, CliInstallMethod)| {
                                        let mut next = editor.read().clone();
                                        if let Some(slot) = next.install_methods.get_mut(index) {
                                            *slot = method;
                                        }
                                        editor.set(next);
                                    }
                                },
                                on_remove: {
                                    let mut editor = props.editor;
                                    move |index: usize| {
                                        let mut next = editor.read().clone();
                                        if next.install_methods.len() > 1 {
                                            next.install_methods.remove(index);
                                        } else {
                                            next.install_methods[0] = empty_cli_install_method();
                                        }
                                        editor.set(next);
                                    }
                                }
                            }
                        }
                    }
                }
                if let Some(method) = selected_method {
                    div { class: "callout callout--info",
                        strong { "当前安装方式" }
                        div { "platform={method.platform.code()} · installer={method.installer_kind.code()} · package={method.package_id}" }
                        div { "command={method.command_template}" }
                    }
                } else {
                    div { class: "settings-note", "保存条目后，可在这里选择安装方式并执行桌面端一键安装。" }
                }
                if let Some(message) = props.install_history_error.clone() {
                    div { class: "callout", "安装历史加载失败：{message}" }
                } else if props.editor.read().id.is_some() {
                    div { class: "stack",
                        span { class: "field__label", "install_history" }
                        if props.install_history.is_empty() {
                            div { class: "settings-note", "当前条目暂无本机安装历史。" }
                        } else {
                            table { class: "data-table",
                                thead {
                                    tr {
                                        th { "时间" }
                                        th { "方式" }
                                        th { "命令" }
                                        th { "结果" }
                                    }
                                }
                                tbody {
                                    for item in props.install_history.iter() {
                                        tr {
                                            td { "{item.created_at}" }
                                            td { "{item.platform.code()} / {item.installer_kind.code()}" }
                                            td { "{item.command}" }
                                            td {
                                                if item.success {
                                                    "成功"
                                                } else {
                                                    "失败"
                                                }
                                                " / {item.exit_code.map(|code| code.to_string()).unwrap_or_else(|| \"-\".to_string())}"
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
    }
}

#[derive(Props, Clone, PartialEq)]
struct CliInstallMethodEditorProps {
    index: usize,
    method: CliInstallMethod,
    on_change: EventHandler<(usize, CliInstallMethod)>,
    on_remove: EventHandler<usize>,
}

#[component]
fn CliInstallMethodEditor(props: CliInstallMethodEditorProps) -> Element {
    rsx! {
        div { class: "software-method-card",
            div { class: "software-method-card__toolbar",
                span { class: "badge", "安装方式 #{props.index + 1}" }
                WorkbenchButton {
                    class: "toolbar-button".to_string(),
                    onclick: move |_| props.on_remove.call(props.index),
                    "移除"
                }
            }
            div { class: "form-grid",
                SelectStringField {
                    label: "平台",
                    value: props.method.platform.code().to_string(),
                    options: CliPlatform::ALL
                        .iter()
                        .copied()
                        .map(|platform| (platform.code().to_string(), platform.code().to_string()))
                        .collect(),
                    on_input: {
                        let mut method = props.method.clone();
                        let on_change = props.on_change;
                        let index = props.index;
                        move |value: String| {
                            method.platform = parse_cli_platform(&value).unwrap_or(CliPlatform::CrossPlatform);
                            on_change.call((index, method.clone()));
                        }
                    }
                }
                SelectStringField {
                    label: "安装器",
                    value: props.method.installer_kind.code().to_string(),
                    options: CliInstallerKind::ALL
                        .iter()
                        .copied()
                        .map(|kind| (kind.code().to_string(), kind.code().to_string()))
                        .collect(),
                    on_input: {
                        let mut method = props.method.clone();
                        let on_change = props.on_change;
                        let index = props.index;
                        move |value: String| {
                            method.installer_kind = parse_cli_installer_kind(&value).unwrap_or(CliInstallerKind::Custom);
                            on_change.call((index, method.clone()));
                        }
                    }
                }
                TextField {
                    label: "package_id",
                    value: props.method.package_id.clone(),
                    on_input: {
                        let mut method = props.method.clone();
                        let on_change = props.on_change;
                        let index = props.index;
                        move |value| {
                            method.package_id = value;
                            on_change.call((index, method.clone()));
                        }
                    }
                }
                TextField {
                    label: "priority",
                    value: props.method.priority.to_string(),
                    on_input: {
                        let mut method = props.method.clone();
                        let on_change = props.on_change;
                        let index = props.index;
                        move |value: String| {
                            method.priority = value.trim().parse::<i32>().unwrap_or(method.priority);
                            on_change.call((index, method.clone()));
                        }
                    }
                }
            }
            TextareaField {
                label: "command_template",
                value: props.method.command_template.clone(),
                on_input: {
                    let mut method = props.method.clone();
                    let on_change = props.on_change;
                    let index = props.index;
                    move |value| {
                        method.command_template = value;
                        on_change.call((index, method.clone()));
                    }
                }
            }
            TextareaField {
                label: "validation_note",
                value: props.method.validation_note.clone(),
                on_input: {
                    let mut method = props.method.clone();
                    let on_change = props.on_change;
                    let index = props.index;
                    move |value| {
                        method.validation_note = value;
                        on_change.call((index, method.clone()));
                    }
                }
            }
        }
    }
}

#[derive(Props, Clone, PartialEq)]
struct TextFieldProps {
    label: &'static str,
    value: String,
    on_input: EventHandler<String>,
}

#[component]
fn TextField(props: TextFieldProps) -> Element {
    rsx! {
        label { class: "field",
            span { class: "field__label", "{props.label}" }
            input { class: "field__input", value: props.value, oninput: move |evt| props.on_input.call(evt.value()) }
        }
    }
}

#[derive(Props, Clone, PartialEq)]
struct TextareaFieldProps {
    label: &'static str,
    value: String,
    on_input: EventHandler<String>,
}

#[component]
fn TextareaField(props: TextareaFieldProps) -> Element {
    rsx! {
        label { class: "textarea",
            span { class: "field__label", "{props.label}" }
            textarea { class: "textarea__input textarea__input--mono", value: props.value, oninput: move |evt| props.on_input.call(evt.value()) }
        }
    }
}

#[derive(Props, Clone, PartialEq)]
struct SelectStringFieldProps {
    label: &'static str,
    value: String,
    options: Vec<(String, String)>,
    on_input: EventHandler<String>,
}

#[component]
fn SelectStringField(props: SelectStringFieldProps) -> Element {
    rsx! {
        label { class: "field",
            span { class: "field__label", "{props.label}" }
            select {
                class: "field__input",
                value: props.value.clone(),
                onchange: move |evt| props.on_input.call(evt.value()),
                for option in props.options {
                    option { value: option.0.clone(), "{option.1}" }
                }
            }
        }
    }
}

fn split_tags(value: &str) -> Vec<String> {
    value
        .split([',', '，', ';'])
        .map(|item| item.trim().to_string())
        .filter(|item| !item.is_empty())
        .collect()
}

fn selected_install_method(
    entry: &CliMarketEntry,
    selected_method_id: Option<&str>,
) -> Option<CliInstallMethod> {
    selected_method_id
        .and_then(|method_id| {
            entry
                .install_methods
                .iter()
                .find(|method| method.id.as_deref() == Some(method_id))
                .cloned()
        })
        .or_else(|| entry.install_methods.first().cloned())
}

fn install_method_label(method: &CliInstallMethod) -> String {
    format!(
        "{} / {} / {}",
        method.platform.code(),
        method.installer_kind.code(),
        method.package_id
    )
}

fn parse_cli_platform(value: &str) -> Option<CliPlatform> {
    CliPlatform::ALL
        .into_iter()
        .find(|platform| platform.code() == value)
}

fn parse_cli_installer_kind(value: &str) -> Option<CliInstallerKind> {
    CliInstallerKind::ALL
        .into_iter()
        .find(|kind| kind.code() == value)
}
