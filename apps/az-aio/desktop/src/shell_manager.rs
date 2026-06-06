#![forbid(unsafe_code)]

use std::{
    collections::{HashMap, HashSet},
    env, fs, io,
    path::{Path, PathBuf},
    process::Command,
    time::{SystemTime, UNIX_EPOCH},
};

use az_aio_plugin_api::{
    GeneratedFileContribution, GeneratedFileStatus, ShellEntryContribution, ShellEntryKind,
};
use az_aio_plugin_host::HostSnapshot;
use az_dioxus_components::az_grammar_search::{
    AzGrammarSearchField, AzGrammarSearchInput, GrammarSearchQuery, parse_grammar_search_query,
};
use dioxus::prelude::*;
use serde::{Deserialize, Serialize};

use crate::shell_manager_store::{load_shell_manager_store, save_shell_manager_store};

const STORE_FILE: &str = "shell-manager.json";
const ADD_FN_MARKER: &str = "# AZ-AIO-Add-Fn: visual-manager-v1";
const SECTION_DELIMITER: &str = "#####";
const SHELL_SEARCH_PLACEHOLDER: &str =
    "关键词:addhost；标签:rust,java；定义:fun,export,alias；来源:~/.config";
const SHELL_HELPERS: &str = r#"shell_prepend_path() {
  [ -n "${1:-}" ] || return 0
  case ":${PATH:-}:" in
    *":$1:"*) ;;
    *) PATH="$1${PATH:+:$PATH}" ;;
  esac
  export PATH
}

shell_append_path() {
  [ -n "${1:-}" ] || return 0
  case ":${PATH:-}:" in
    *":$1:"*) ;;
    *) PATH="${PATH:+$PATH:}$1" ;;
  esac
  export PATH
}
"#;
const SHELL_LOADER_HELPER: &str = r#"add_fn_load() {
  local __add_fn_requested_mode
  __add_fn_requested_mode="${1:-interactive}"
  ADD_FN_MODE="$__add_fn_requested_mode" . "$HOME/.add_fn"
  unset __add_fn_requested_mode
}
"#;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum ShellPageMode {
    Cli,
    Env,
}

impl ShellPageMode {
    pub fn title(self) -> &'static str {
        match self {
            Self::Cli => "命令行接口",
            Self::Env => "操作系统环境变量",
        }
    }

    pub fn mark(self) -> &'static str {
        match self {
            Self::Cli => "⌁",
            Self::Env => "▣",
        }
    }

    fn section_name(self) -> &'static str {
        match self {
            Self::Cli => "cli",
            Self::Env => "env",
        }
    }

    fn default_body(self) -> &'static str {
        match self {
            Self::Cli => "alias new_command='echo hello'\n",
            Self::Env => "export NEW_ENV=value\n",
        }
    }

    fn default_name(self) -> &'static str {
        match self {
            Self::Cli => "new_command",
            Self::Env => "NEW_ENV",
        }
    }

    fn default_kind(self) -> ShellEntryKind {
        match self {
            Self::Cli => ShellEntryKind::Alias,
            Self::Env => ShellEntryKind::Export,
        }
    }
}

#[allow(non_snake_case)]
#[component]
pub fn ShellManagerRoutePage(snapshot: Signal<HostSnapshot>, mode: ShellPageMode) -> Element {
    let query = use_signal(String::new);
    rsx! {
        div { class: "metadata-page",
            ShellManagerPage { snapshot, mode, query }
        }
    }
}

#[allow(non_snake_case)]
#[component]
pub fn ShellManagerPage(
    snapshot: Signal<HostSnapshot>,
    mode: ShellPageMode,
    query: Signal<String>,
) -> Element {
    let mut manager = use_signal(|| ShellManagerState::load(&snapshot.read()));
    let state = manager.read().clone();
    let query_text = query.read().clone();
    let parsed_query = parse_grammar_search_query(&query_text);
    let visible_items = state.visible_items(mode, &parsed_query);
    let generated_file = snapshot.read().generated_files.first().cloned();
    let deployable_count = state.deployable_count(mode);
    let recognized_count = state.recognized_count(mode);
    let user_count = state.user_count(mode);
    let canonical_path = state.canonical_path_display();

    rsx! {
        div { class: "metadata-panel shell-manager",
            ShellGeneratedSummary { generated_file }

            div { class: "metadata-header shell-manager__header",
                div { class: "metadata-header__mark", "{mode.mark()}" }
                div {
                    h1 { "{mode.title()}" }
                    p { "{visible_items.len()} 个当前显示，{recognized_count} 个来自扫描，{user_count} 个手动维护；固定部署到 {canonical_path}" }
                }
                div { class: "shell-manager__header-actions",
                    button {
                        class: "toolbar-button",
                        r#type: "button",
                        onclick: move |_| manager.write().merge_snapshot(&snapshot.read()),
                        "重新识别"
                    }
                    button {
                        class: "toolbar-button",
                        r#type: "button",
                        onclick: move |_| manager.write().create_item(mode),
                        "＋ 创建"
                    }
                    button {
                        class: "toolbar-button",
                        r#type: "button",
                        onclick: move |_| manager.write().save(),
                        "保存"
                    }
                    button {
                        class: "toolbar-button toolbar-button--primary",
                        r#type: "button",
                        disabled: deployable_count == 0,
                        onclick: move |_| manager.write().deploy_all(mode),
                        "一键部署全部"
                    }
                }
            }

            if let Some(message) = state.message.as_ref() {
                div { class: state.message_class(), "{message}" }
            }

            div { class: "metadata-controls shell-manager__controls",
                AzGrammarSearchInput {
                    value: query_text,
                    placeholder: SHELL_SEARCH_PLACEHOLDER.to_string(),
                    fields: shell_search_fields(),
                    oninput: move |value| query.set(value),
                }
            }

            if visible_items.is_empty() {
                div { class: "catalog-empty",
                    div { class: "empty-panel__mark", "{mode.mark()}" }
                    h2 { "没有匹配项" }
                }
            } else {
                div { class: "metadata-list",
                    for section in shell_item_sections(&visible_items) {
                        section { class: "metadata-section shell-manager-section",
                            h2 { "{section}" }
                            div { class: "metadata-section__grid shell-manager-grid",
                                for item in visible_items.iter().filter(|item| item.section == section) {
                                    ShellManagedCard {
                                        item: item.clone(),
                                        manager,
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

#[allow(non_snake_case)]
#[component]
fn ShellGeneratedSummary(generated_file: Option<GeneratedFileContribution>) -> Element {
    let generated_file = generated_file.unwrap_or_else(|| GeneratedFileContribution {
        id: "shell.add-fn.missing".to_string(),
        path: "~/.add_fn".to_string(),
        source_root: "~/.config/shell".to_string(),
        section_delimiter: "#####".to_string(),
        deprecated_source_root: true,
        entry_count: 0,
        backup_path: None,
        status: GeneratedFileStatus::Failed,
        message: "生成器尚未返回状态。".to_string(),
    });
    let status_class = generated_status_class(generated_file.status);
    let status_label = generated_status_label(generated_file.status);
    let generated_path = compact_home_path_str(&generated_file.path);
    let source_root = compact_home_path_str(&generated_file.source_root);
    let backup_path = generated_file
        .backup_path
        .as_deref()
        .map(compact_home_path_str);

    rsx! {
        section { class: "metadata-summary",
            div { class: "metadata-summary__main",
                p { class: "metadata-summary__eyebrow", "源文件" }
                h2 { "{generated_path}" }
                p { "{generated_file.message}" }
            }
            div { class: "metadata-summary__meta",
                span { class: status_class, "{status_label}" }
                span { "{generated_file.entry_count} 个条目" }
                span { "分隔符 {generated_file.section_delimiter}" }
                if generated_file.deprecated_source_root {
                    span { "旧来源 {source_root}" }
                }
                if let Some(backup_path) = backup_path.as_ref() {
                    span { "备份 {backup_path}" }
                }
            }
        }
    }
}

#[allow(non_snake_case)]
#[component]
fn ShellManagedCard(item: ShellManagedItem, manager: Signal<ShellManagerState>) -> Element {
    let item_id = item.id.clone();
    let deploy_disabled = item.deleted;
    let kind_class = shell_kind_class(item.kind);
    let source_class = if item.source_missing {
        "shell-path-box shell-path-box--missing"
    } else {
        "shell-path-box"
    };
    let source_display = item.source_display();
    let deployment_paths = item.deployment_path_displays();
    let tag_input = item.tags.join(",");

    rsx! {
        article { class: "metadata-card shell-managed-card",
            div { class: "shell-managed-card__top",
                div { class: kind_class, "{item.kind.label()}" }
                div { class: "shell-managed-card__actions",
                    button {
                        class: "shell-card-button shell-card-button--primary",
                        r#type: "button",
                        disabled: deploy_disabled,
                        onclick: {
                            let item_id = item_id.clone();
                            move |_| manager.write().deploy_item(&item_id)
                        },
                        "部署"
                    }
                    button {
                        class: "shell-card-button shell-card-button--danger",
                        r#type: "button",
                        onclick: {
                            let item_id = item_id.clone();
                            move |_| manager.write().delete_item(&item_id)
                        },
                        "逻辑删除"
                    }
                }
            }

            label { class: "shell-form-field",
                span { "名称" }
                input {
                    value: "{item.name}",
                    oninput: {
                        let item_id = item_id.clone();
                        move |event| manager.write().update_name(&item_id, event.value())
                    },
                }
            }

            label { class: "shell-form-field",
                span { "内容" }
                textarea {
                    value: "{item.body}",
                    oninput: {
                        let item_id = item_id.clone();
                        move |event| manager.write().update_body(&item_id, event.value())
                    },
                }
            }

            label { class: "shell-form-field shell-tag-editor",
                span { "标签" }
                input {
                    value: "{tag_input}",
                    placeholder: "rust,java,go,project",
                    oninput: {
                        let item_id = item_id.clone();
                        move |event| manager.write().update_tags(&item_id, event.value())
                    },
                }
                if !item.tags.is_empty() {
                    div { class: "shell-tag-list",
                        for tag in item.tags.iter() {
                            span { class: "shell-tag-chip", "{tag}" }
                        }
                    }
                }
            }

            div { class: "shell-path-compare",
                div { class: source_class,
                    span { "识别路径" }
                    code { "{source_display}" }
                }
                div { class: "shell-path-box",
                    span { "部署路径" }
                    div { class: "shell-path-list",
                        for path in deployment_paths {
                            span { class: "shell-path-chip", "{path}" }
                        }
                    }
                }
            }

            p { class: "shell-managed-card__hint", "~/.add_fn 是唯一写入目标；文件保持只读，由可视化界面执行受控替换。" }
        }
    }
}

#[derive(Clone, Debug)]
pub struct ShellManagerState {
    store_path: PathBuf,
    items: Vec<ShellManagedItem>,
    deleted_source_ids: HashSet<String>,
    message: Option<String>,
    message_kind: ShellManagerMessageKind,
    generated_paths: Vec<String>,
}

impl ShellManagerState {
    fn load(snapshot: &HostSnapshot) -> Self {
        let store_path = shell_manager_store_path();
        let store = load_shell_manager_store(&store_path).unwrap_or_default();
        let mut state = Self {
            store_path,
            items: store.items,
            deleted_source_ids: store.deleted_source_ids.into_iter().collect(),
            message: None,
            message_kind: ShellManagerMessageKind::Info,
            generated_paths: generated_paths(snapshot),
        };
        state.merge_snapshot(snapshot);
        state
    }

    fn merge_snapshot(&mut self, snapshot: &HostSnapshot) {
        self.generated_paths = generated_paths(snapshot);
        let mut stored_by_id = self
            .items
            .iter()
            .cloned()
            .map(|item| (item.id.clone(), item))
            .collect::<HashMap<_, _>>();
        let mut next_items = Vec::new();

        for entry in &snapshot.shell_entries {
            if self.deleted_source_ids.contains(&entry.id) {
                continue;
            }
            let Some(mut item) = ShellManagedItem::from_source_entry(entry) else {
                continue;
            };
            if let Some(saved) = stored_by_id.remove(&entry.id) {
                item.apply_saved_fields(saved);
            }
            next_items.push(item);
        }

        for mut saved in stored_by_id.into_values() {
            saved.source_missing = !saved.is_user_created;
            next_items.push(saved);
        }

        normalize_deployment_paths(&mut next_items);
        next_items.sort_by(|left, right| {
            left.mode()
                .section_name()
                .cmp(right.mode().section_name())
                .then_with(|| left.section.cmp(&right.section))
                .then_with(|| left.name.cmp(&right.name))
        });
        self.items = next_items;
        self.message = Some("已重新识别命令行和环境变量。".to_string());
        self.message_kind = ShellManagerMessageKind::Info;
    }

    fn visible_items(
        &self,
        mode: ShellPageMode,
        query: &GrammarSearchQuery,
    ) -> Vec<ShellManagedItem> {
        self.items
            .iter()
            .filter(|item| !item.deleted && item.mode() == mode && item.matches_query(query))
            .cloned()
            .collect()
    }

    fn deployable_count(&self, mode: ShellPageMode) -> usize {
        self.items
            .iter()
            .filter(|item| !item.deleted && item.mode() == mode)
            .count()
    }

    fn recognized_count(&self, mode: ShellPageMode) -> usize {
        self.items
            .iter()
            .filter(|item| {
                !item.deleted
                    && item.mode() == mode
                    && !item.is_user_created
                    && !item.source_missing
            })
            .count()
    }

    fn user_count(&self, mode: ShellPageMode) -> usize {
        self.items
            .iter()
            .filter(|item| !item.deleted && item.mode() == mode && item.is_user_created)
            .count()
    }

    fn canonical_path_display(&self) -> String {
        compact_home_path(&canonical_add_fn_path())
    }

    fn create_item(&mut self, mode: ShellPageMode) {
        let id = format!(
            "managed.{}.{}",
            mode.section_name(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|duration| duration.as_millis())
                .unwrap_or_default()
        );
        self.items.insert(
            0,
            ShellManagedItem {
                id,
                kind: mode.default_kind(),
                name: mode.default_name().to_string(),
                section: "user-managed".to_string(),
                source_path: "手动创建".to_string(),
                line_start: 0,
                body: mode.default_body().to_string(),
                deployment_paths: vec![canonical_add_fn_path().display().to_string()],
                draft_path: canonical_add_fn_path().display().to_string(),
                tags: default_shell_tags(mode.default_kind(), "user-managed", true),
                is_user_created: true,
                source_missing: false,
                deleted: false,
            },
        );
        self.message = Some("已创建一条托管项，编辑后可保存或部署。".to_string());
        self.message_kind = ShellManagerMessageKind::Info;
    }

    fn update_name(&mut self, item_id: &str, name: String) {
        if let Some(item) = self.find_item_mut(item_id) {
            item.name = name;
        }
    }

    fn update_body(&mut self, item_id: &str, body: String) {
        if let Some(item) = self.find_item_mut(item_id) {
            item.body = body;
        }
    }

    fn update_tags(&mut self, item_id: &str, tags: String) {
        if let Some(item) = self.find_item_mut(item_id) {
            item.tags = parse_shell_tags(&tags);
        }
    }

    fn delete_item(&mut self, item_id: &str) {
        let deleted_source_id = self
            .items
            .iter()
            .find(|item| item.id == item_id && !item.is_user_created)
            .map(|item| item.id.clone());
        if let Some(item) = self.find_item_mut(item_id) {
            item.deleted = true;
        }
        if let Some(deleted_source_id) = deleted_source_id {
            self.deleted_source_ids.insert(deleted_source_id);
        }
        self.save();
        self.message = Some("已逻辑删除托管项；保存状态仍保留，部署时会跳过。".to_string());
        self.message_kind = ShellManagerMessageKind::Info;
    }

    fn deploy_item(&mut self, item_id: &str) {
        let Some(item) = self
            .items
            .iter()
            .find(|item| item.id == item_id && !item.deleted)
        else {
            return;
        };
        match deploy_shell_items(&self.items, &canonical_add_fn_path()) {
            Ok(report) => {
                let _ = self.save_to_disk();
                self.message = Some(format!(
                    "已部署 {}，并同步全部 {} 个有效项到 {}{}。",
                    item.name,
                    report.item_count,
                    report.output_path_display(),
                    report.backup_suffix()
                ));
                self.message_kind = ShellManagerMessageKind::Success;
            }
            Err(error) => {
                self.message = Some(format!("部署 {} 失败：{error}", item.name));
                self.message_kind = ShellManagerMessageKind::Error;
            }
        }
    }

    fn deploy_all(&mut self, _mode: ShellPageMode) {
        if self.items.iter().all(|item| item.deleted) {
            self.message = Some("没有可部署的命令行或环境变量。".to_string());
            self.message_kind = ShellManagerMessageKind::Error;
            return;
        }

        match deploy_shell_items(&self.items, &canonical_add_fn_path()) {
            Ok(report) => {
                let _ = self.save_to_disk();
                self.message = Some(format!(
                    "一键部署完成，已同步 {} 个有效项到 {}{}。",
                    report.item_count,
                    report.output_path_display(),
                    report.backup_suffix()
                ));
                self.message_kind = ShellManagerMessageKind::Success;
            }
            Err(error) => {
                self.message = Some(format!("一键部署失败：{error}"));
                self.message_kind = ShellManagerMessageKind::Error;
            }
        }
    }

    fn save(&mut self) {
        match self.save_to_disk() {
            Ok(()) => {
                self.message = Some("命令行和环境变量托管配置已保存。".to_string());
                self.message_kind = ShellManagerMessageKind::Success;
            }
            Err(error) => {
                self.message = Some(format!("保存失败：{error}"));
                self.message_kind = ShellManagerMessageKind::Error;
            }
        }
    }

    fn message_class(&self) -> &'static str {
        match self.message_kind {
            ShellManagerMessageKind::Info => "settings-message settings-message--info",
            ShellManagerMessageKind::Success => "settings-message settings-message--success",
            ShellManagerMessageKind::Error => "settings-message settings-message--error",
        }
    }

    fn find_item_mut(&mut self, item_id: &str) -> Option<&mut ShellManagedItem> {
        self.items.iter_mut().find(|item| item.id == item_id)
    }

    fn save_to_disk(&self) -> io::Result<()> {
        let mut items = self.items.clone();
        normalize_deployment_paths(&mut items);
        let store = ShellManagerStore {
            items,
            deleted_source_ids: self.deleted_source_ids.iter().cloned().collect(),
        };
        save_shell_manager_store(&self.store_path, &store)
    }
}

pub fn deploy_saved_shell_manager_store() -> io::Result<String> {
    let store_path = shell_manager_store_path();
    let store = load_shell_manager_store(&store_path)?;
    if store.items.is_empty() {
        return Err(io::Error::other(format!(
            "命令和环境变量托管配置为空：{}",
            store_path.display()
        )));
    }

    let mut items = store.items;
    normalize_deployment_paths(&mut items);
    let report = deploy_shell_items(&items, &canonical_add_fn_path())?;

    let normalized_store = ShellManagerStore {
        items,
        deleted_source_ids: store.deleted_source_ids,
    };
    save_shell_manager_store(&store_path, &normalized_store)?;

    Ok(format!(
        "已部署 {} 个有效条目到 {}{}",
        report.item_count,
        report.output_path_display(),
        report.backup_suffix()
    ))
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ShellManagerMessageKind {
    Info,
    Success,
    Error,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub(crate) struct ShellManagerStore {
    #[serde(default)]
    items: Vec<ShellManagedItem>,
    #[serde(default)]
    deleted_source_ids: Vec<String>,
}

impl ShellManagerStore {
    pub(crate) fn is_empty(&self) -> bool {
        self.items.is_empty() && self.deleted_source_ids.is_empty()
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
struct ShellManagedItem {
    id: String,
    kind: ShellEntryKind,
    name: String,
    section: String,
    source_path: String,
    line_start: usize,
    body: String,
    #[serde(default)]
    deployment_paths: Vec<String>,
    #[serde(default)]
    draft_path: String,
    #[serde(default)]
    tags: Vec<String>,
    #[serde(default)]
    is_user_created: bool,
    #[serde(default)]
    deleted: bool,
    #[serde(skip)]
    source_missing: bool,
}

impl ShellManagedItem {
    fn from_source_entry(entry: &ShellEntryContribution) -> Option<Self> {
        if entry.kind != ShellEntryKind::Export && !entry.kind.is_cli() {
            return None;
        }
        let canonical_path = canonical_add_fn_path().display().to_string();
        Some(Self {
            id: entry.id.clone(),
            kind: entry.kind,
            name: entry.name.clone(),
            section: entry.section.clone(),
            source_path: entry.source_path.clone(),
            line_start: entry.line_start,
            body: shell_body_from_source(entry).unwrap_or_else(|| entry.preview.clone()),
            deployment_paths: vec![canonical_path.clone()],
            draft_path: canonical_path,
            tags: default_shell_tags(entry.kind, &entry.section, false),
            is_user_created: false,
            deleted: false,
            source_missing: false,
        })
    }

    fn apply_saved_fields(&mut self, saved: Self) {
        self.name = saved.name;
        self.body = saved.body;
        self.deployment_paths = saved.deployment_paths;
        self.draft_path = saved.draft_path;
        self.tags = merge_shell_tags(
            saved.tags,
            default_shell_tags(self.kind, &self.section, false),
        );
        self.is_user_created = saved.is_user_created;
        self.deleted = saved.deleted;
        self.source_missing = false;
    }

    fn mode(&self) -> ShellPageMode {
        if self.kind == ShellEntryKind::Export {
            ShellPageMode::Env
        } else {
            ShellPageMode::Cli
        }
    }

    fn source_display(&self) -> String {
        let source_path = compact_home_path_str(&self.source_path);
        if self.line_start == 0 {
            source_path
        } else {
            format!("{}:{}", source_path, self.line_start)
        }
    }

    fn deployment_path_displays(&self) -> Vec<String> {
        self.deployment_paths
            .iter()
            .map(|path| compact_home_path_str(path))
            .collect()
    }

    fn matches_query(&self, query: &GrammarSearchQuery) -> bool {
        if query.is_empty() {
            return true;
        }

        query
            .terms
            .iter()
            .all(|term| self.matches_keyword(&term.value))
            && query
                .filters
                .iter()
                .all(|filter| self.matches_filter(&filter.key, &filter.values))
    }

    fn matches_keyword(&self, value: &str) -> bool {
        contains_ci(&self.name, value)
            || contains_ci(&self.section, value)
            || contains_ci(&self.source_path, value)
            || contains_ci(&self.source_display(), value)
            || contains_ci(&self.body, value)
            || self.tags.iter().any(|tag| contains_ci(tag, value))
            || self.deployment_paths.iter().any(|path| {
                contains_ci(path, value) || contains_ci(&compact_home_path_str(path), value)
            })
    }

    fn matches_filter(&self, key: &str, values: &[String]) -> bool {
        match key {
            "keyword" | "q" | "text" | "关键词" | "全文" | "文本" => {
                self.matches_any_value(values, |item, value| item.matches_keyword(value))
            }
            "tag" | "tags" | "标签" => self.matches_any_value(values, |item, value| {
                item.tags.iter().any(|tag| eq_ci(tag, value))
            }),
            "def" | "kind" | "type" | "定义" | "类型" => {
                self.matches_any_value(values, |item, value| {
                    shell_kind_search_keys(item.kind)
                        .iter()
                        .any(|key| eq_ci(key, value))
                })
            }
            "name" | "名称" => {
                self.matches_any_value(values, |item, value| contains_ci(&item.name, value))
            }
            "section" | "group" | "分组" => {
                self.matches_any_value(values, |item, value| contains_ci(&item.section, value))
            }
            "source" | "path" | "来源" | "识别" | "识别路径" => {
                self.matches_any_value(values, |item, value| {
                    contains_ci(&item.source_path, value)
                        || contains_ci(&item.source_display(), value)
                })
            }
            "deploy" | "deployment" | "部署" | "部署路径" => {
                self.matches_any_value(values, |item, value| {
                    item.deployment_paths.iter().any(|path| {
                        contains_ci(path, value) || contains_ci(&compact_home_path_str(path), value)
                    })
                })
            }
            "body" | "content" | "内容" => {
                self.matches_any_value(values, |item, value| contains_ci(&item.body, value))
            }
            _ => self.matches_any_value(values, |item, value| item.matches_keyword(value)),
        }
    }

    fn matches_any_value<P>(&self, values: &[String], predicate: P) -> bool
    where
        P: Fn(&Self, &str) -> bool,
    {
        values.iter().any(|value| predicate(self, value))
    }
}

fn shell_body_from_source(entry: &ShellEntryContribution) -> Option<String> {
    let content = fs::read_to_string(&entry.source_path).ok()?;
    let lines = content.lines().collect::<Vec<_>>();
    let start = entry.line_start.checked_sub(1)?;
    let line = lines.get(start)?;
    match entry.kind {
        ShellEntryKind::Alias | ShellEntryKind::Export => Some(ensure_trailing_newline(line)),
        ShellEntryKind::Function => {
            let (body, _) = collect_function_body(&lines, start);
            Some(ensure_trailing_newline(&body))
        }
        ShellEntryKind::ScriptSnippet => {
            if start == 0 {
                Some(ensure_trailing_newline(&content))
            } else {
                Some(ensure_trailing_newline(line))
            }
        }
    }
}

fn collect_function_body(lines: &[&str], start_index: usize) -> (String, usize) {
    let mut body = Vec::new();
    let mut balance = 0;
    let mut saw_open = false;

    for (index, line) in lines.iter().enumerate().skip(start_index) {
        body.push(*line);
        for char in line.chars() {
            match char {
                '{' => {
                    saw_open = true;
                    balance += 1;
                }
                '}' if balance > 0 => balance -= 1,
                _ => {}
            }
        }
        if saw_open && balance == 0 {
            return (body.join("\n"), index);
        }
    }

    (body.join("\n"), lines.len().saturating_sub(1))
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct DeployReport {
    output_path: PathBuf,
    backup_path: Option<PathBuf>,
    item_count: usize,
}

impl DeployReport {
    fn backup_suffix(&self) -> String {
        self.backup_path
            .as_ref()
            .map(|path| format!("；已备份旧文件到 {}", compact_home_path(path)))
            .unwrap_or_default()
    }

    fn output_path_display(&self) -> String {
        compact_home_path(&self.output_path)
    }
}

fn deploy_shell_items(items: &[ShellManagedItem], output_path: &Path) -> io::Result<DeployReport> {
    let active_items = active_shell_items(items);
    let content = render_add_fn(&active_items);
    validate_rendered_add_fn(output_path, &content)?;
    let backup_path = backup_existing_add_fn(output_path, &content)?;
    write_readonly_atomic(output_path, &content)?;
    Ok(DeployReport {
        output_path: output_path.to_path_buf(),
        backup_path,
        item_count: active_items.len(),
    })
}

fn active_shell_items(items: &[ShellManagedItem]) -> Vec<ShellManagedItem> {
    let mut active = items
        .iter()
        .filter(|item| !item.deleted)
        .cloned()
        .collect::<Vec<_>>();
    normalize_deployment_paths(&mut active);
    active.sort_by(|left, right| {
        shell_kind_order(left.kind)
            .cmp(&shell_kind_order(right.kind))
            .then_with(|| left.section.cmp(&right.section))
            .then_with(|| left.name.cmp(&right.name))
            .then_with(|| left.id.cmp(&right.id))
    });
    active
}

fn shell_kind_order(kind: ShellEntryKind) -> u8 {
    match kind {
        ShellEntryKind::Alias => 0,
        ShellEntryKind::Export => 1,
        ShellEntryKind::Function => 2,
        ShellEntryKind::ScriptSnippet => 3,
    }
}

fn render_add_fn(items: &[ShellManagedItem]) -> String {
    let mut output = String::new();
    let names = ShellNameIndex::from_items(items);
    output.push_str("# 由 AZ AIO 桌面端可视化命令管理器生成。\n");
    output.push_str(ADD_FN_MARKER);
    output.push('\n');
    output.push_str("# 命令别名、环境变量、函数和脚本片段的唯一生效文件。\n");
    output.push_str("# 请不要手工编辑本文件；请使用 AZ AIO 桌面端命令管理器修改。\n\n");
    output.push_str("__add_fn_mode=\"${ADD_FN_MODE:-interactive}\"\n\n");
    output.push_str(&format!("{SECTION_DELIMITER} helpers\n"));
    output.push_str(SHELL_HELPERS);
    output.push('\n');
    output.push_str(SHELL_LOADER_HELPER);
    output.push('\n');

    output.push_str("if [ \"$__add_fn_mode\" = \"profile\" ] && [ \"${ADD_FN_PROFILE_LOADED:-0}\" != 1 ]; then\n");
    output.push_str("  ADD_FN_PROFILE_LOADED=1\n");
    render_direct_section(
        &mut output,
        items,
        ShellEntryKind::Export,
        "profile/export",
        &names,
        is_profile_item,
    );
    render_snippet_section(&mut output, items, "profile/sh", &names, is_profile_item);
    output.push_str("fi\n\n");

    output.push_str("if { [ \"$__add_fn_mode\" = \"interactive\" ] || [ \"$__add_fn_mode\" = \"rc\" ]; } && [ \"${ADD_FN_INTERACTIVE_LOADED:-0}\" != 1 ]; then\n");
    output.push_str("  ADD_FN_INTERACTIVE_LOADED=1\n");
    render_direct_section(
        &mut output,
        items,
        ShellEntryKind::Alias,
        "interactive/alias",
        &names,
        is_interactive_item,
    );
    render_direct_section(
        &mut output,
        items,
        ShellEntryKind::Export,
        "interactive/export",
        &names,
        is_interactive_item,
    );
    render_direct_section(
        &mut output,
        items,
        ShellEntryKind::Function,
        "interactive/fun",
        &names,
        is_interactive_item,
    );
    render_snippet_section(
        &mut output,
        items,
        "interactive/sh",
        &names,
        is_interactive_item,
    );
    output.push_str("fi\n\n");
    output.push_str("unset __add_fn_mode\n");

    ensure_trailing_newline(&output)
}

fn validate_rendered_add_fn(output_path: &Path, content: &str) -> io::Result<()> {
    let target_path = symlink_target_or_path(output_path);
    let parent = target_path.parent().unwrap_or_else(|| Path::new("/tmp"));
    fs::create_dir_all(parent)?;
    let temp_path = parent.join(format!(".add_fn.syntax-{}", timestamp_millis()));
    fs::write(&temp_path, content)?;

    // ~/.add_fn 由可视化管理器托管；替换只读目标前，生成内容必须先通过两种 shell 解析。
    let bash_result = syntax_check_shell("bash", &temp_path);
    let zsh_result = syntax_check_shell("zsh", &temp_path);
    let _ = fs::remove_file(&temp_path);

    bash_result?;
    zsh_result?;
    Ok(())
}

fn syntax_check_shell(shell: &str, path: &Path) -> io::Result<()> {
    let output = Command::new(shell).arg("-n").arg(path).output()?;
    if output.status.success() {
        return Ok(());
    }

    let stderr = String::from_utf8_lossy(&output.stderr);
    let stdout = String::from_utf8_lossy(&output.stdout);
    let detail = [stderr.trim(), stdout.trim()]
        .into_iter()
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join("\n");
    let message = if detail.is_empty() {
        format!("{shell} -n 检查失败，状态为 {}", output.status)
    } else {
        format!("{shell} -n 检查失败：\n{detail}")
    };
    Err(io::Error::other(message))
}

fn render_direct_section<P>(
    output: &mut String,
    items: &[ShellManagedItem],
    kind: ShellEntryKind,
    section_name: &str,
    names: &ShellNameIndex,
    should_include: P,
) where
    P: Fn(&ShellManagedItem) -> bool,
{
    output.push_str(&format!("{SECTION_DELIMITER} {section_name}\n"));

    for item in items
        .iter()
        .filter(|item| item.kind == kind && should_include(item) && should_render_direct_item(item))
    {
        if kind == ShellEntryKind::Alias && names.functions.contains(&item.name) {
            continue;
        }
        output.push_str(&format!("# 来源：{}\n", item.source_display()));
        if kind == ShellEntryKind::Function {
            output.push_str(&format!("unalias {} 2>/dev/null || true\n", item.name));
        }
        output.push_str(item.body.trim_end());
        output.push_str("\n\n");
    }
}

fn render_snippet_section<P>(
    output: &mut String,
    items: &[ShellManagedItem],
    section_name: &str,
    names: &ShellNameIndex,
    should_include: P,
) where
    P: Fn(&ShellManagedItem) -> bool,
{
    output.push_str(&format!("{SECTION_DELIMITER} {section_name}\n"));
    for item in items
        .iter()
        .filter(|item| item.kind == ShellEntryKind::ScriptSnippet && should_include(item))
    {
        if is_bin_script_section(&item.section) && names.aliases.contains(&item.name) {
            continue;
        }
        output.push_str(&format!("# 来源：{}\n", item.source_display()));
        if is_bin_script_section(&item.section) {
            render_script_function(output, item);
        } else {
            render_shell_snippet(output, item, names);
        }
        output.push('\n');
    }
}

fn is_profile_item(item: &ShellManagedItem) -> bool {
    if item.is_user_created {
        return item.kind == ShellEntryKind::Export;
    }
    is_profile_section(&item.section)
}

fn is_interactive_item(item: &ShellManagedItem) -> bool {
    if item.is_user_created {
        return item.kind != ShellEntryKind::Export;
    }
    !is_profile_section(&item.section)
}

fn is_profile_section(section: &str) -> bool {
    matches!(section, "profile.d" | "local.profile.d")
}

fn render_script_function(output: &mut String, item: &ShellManagedItem) {
    let delimiter = heredoc_delimiter(&item.id);
    output.push_str(&format!("unalias {} 2>/dev/null || true\n", item.name));
    output.push_str(&format!("{}() {{\n", item.name));
    output.push_str("  local __add_fn_script\n");
    output.push_str(&format!(
        "  __add_fn_script=\"${{TMPDIR:-/tmp}}/add_fn_{}_$$.sh\"\n",
        sanitize_id(&item.name)
    ));
    output.push_str("  cat >\"$__add_fn_script\" <<'");
    output.push_str(&delimiter);
    output.push_str("'\n");
    output.push_str(item.body.trim_end());
    output.push('\n');
    output.push_str(&delimiter);
    output.push('\n');
    output.push_str("  chmod 700 \"$__add_fn_script\"\n");
    output.push_str("  ADD_FN_MODE=script \"$__add_fn_script\" \"$@\"\n");
    output.push_str("  local __add_fn_status=$?\n");
    output.push_str("  rm -f \"$__add_fn_script\"\n");
    output.push_str("  return $__add_fn_status\n");
    output.push_str("}\n");
}

fn render_shell_snippet(output: &mut String, item: &ShellManagedItem, names: &ShellNameIndex) {
    let body = filtered_shell_snippet_body(&item.body, names);
    if body.trim().is_empty() {
        return;
    }
    if is_zsh_section(&item.section) {
        output.push_str(
            "if [ -n \"${ZSH_VERSION:-}\" ] && [ \"${ADD_FN_MODE:-interactive}\" != \"profile\" ]; then\n",
        );
        output.push_str(&body);
        output.push_str("fi\n");
    } else {
        output.push_str(&body);
    }
}

fn filtered_shell_snippet_body(body: &str, names: &ShellNameIndex) -> String {
    let lines = body.lines().collect::<Vec<_>>();
    let mut filtered = Vec::new();
    let mut index = 0;

    while index < lines.len() {
        let line = lines[index];
        let trimmed = line.trim();
        if is_deprecated_shell_bin_path_line(trimmed) {
            index += 1;
            continue;
        }
        if is_deprecated_recursive_add_fn_source_line(trimmed) {
            index += 1;
            continue;
        }
        if parse_alias_name(trimmed).is_some_and(|name| names.aliases.contains(&name)) {
            index += 1;
            continue;
        }
        if parse_export_name(trimmed).is_some_and(|name| names.exports.contains(&name)) {
            index += 1;
            continue;
        }
        if let Some(name) = parse_function_name(trimmed) {
            if names.functions.contains(&name) {
                let (_, end_index) = collect_function_body(&lines, index);
                index = end_index.saturating_add(1);
                continue;
            }
            if !last_filtered_line_is_unalias(&filtered, &name) {
                filtered.push(format!("unalias {name} 2>/dev/null || true"));
            }
        }
        filtered.push(line.to_string());
        index += 1;
    }

    ensure_trailing_newline(&filtered.join("\n"))
}

fn last_filtered_line_is_unalias(filtered: &[String], name: &str) -> bool {
    filtered
        .iter()
        .rev()
        .find(|line| !line.trim().is_empty())
        .is_some_and(|line| line.trim() == format!("unalias {name} 2>/dev/null || true"))
}

fn is_bin_script_section(section: &str) -> bool {
    matches!(
        section,
        "bin" | "home-bin" | "local-bin" | "user-bin" | "recovered-bin"
    )
}

fn is_zsh_section(section: &str) -> bool {
    matches!(section, "zshrc.d" | "local.zshrc.d")
}

fn is_deprecated_shell_bin_path_line(trimmed: &str) -> bool {
    trimmed.contains(".config/shell/bin")
        && (trimmed.starts_with("shell_prepend_path")
            || trimmed.starts_with("shell_append_path")
            || trimmed.starts_with("export PATH=")
            || trimmed.starts_with("PATH="))
}

fn is_deprecated_recursive_add_fn_source_line(trimmed: &str) -> bool {
    trimmed.contains("ENABLE_LEGACY_ADD_FN")
        && trimmed.contains("$HOME/.add_fn")
        && (trimmed.contains(". \"$HOME/.add_fn\"")
            || trimmed.contains("source \"$HOME/.add_fn\"")
            || trimmed.contains(". '$HOME/.add_fn'")
            || trimmed.contains("source '$HOME/.add_fn'"))
}

#[derive(Clone, Debug, Default)]
struct ShellNameIndex {
    aliases: HashSet<String>,
    exports: HashSet<String>,
    functions: HashSet<String>,
}

impl ShellNameIndex {
    fn from_items(items: &[ShellManagedItem]) -> Self {
        let mut index = Self::default();
        for item in items.iter().filter(|item| should_render_direct_item(item)) {
            match item.kind {
                ShellEntryKind::Alias => {
                    index.aliases.insert(item.name.clone());
                }
                ShellEntryKind::Export => {
                    index.exports.insert(item.name.clone());
                }
                ShellEntryKind::Function => {
                    index.functions.insert(item.name.clone());
                }
                ShellEntryKind::ScriptSnippet => {}
            }
        }
        index
    }
}

fn should_render_direct_item(item: &ShellManagedItem) -> bool {
    if is_file_snippet_section(&item.section) {
        return false;
    }
    match item.kind {
        ShellEntryKind::Alias => {
            parse_alias_name(item.body.trim()).is_some_and(|name| name == item.name)
        }
        ShellEntryKind::Export => {
            let body = item.body.trim();
            body.starts_with("export ")
                && parse_export_name(body).is_some_and(|name| name == item.name)
        }
        ShellEntryKind::Function => item
            .body
            .lines()
            .find(|line| !line.trim().is_empty())
            .and_then(parse_function_name)
            .is_some_and(|name| name == item.name),
        ShellEntryKind::ScriptSnippet => false,
    }
}

fn is_file_snippet_section(section: &str) -> bool {
    matches!(
        section,
        "lib"
            | "profile.d"
            | "local.profile.d"
            | "rc.d"
            | "local.rc.d"
            | "zshrc.d"
            | "local.zshrc.d"
    )
}

fn backup_existing_add_fn(output_path: &Path, next_content: &str) -> io::Result<Option<PathBuf>> {
    let Ok(existing) = fs::read_to_string(output_path) else {
        return Ok(None);
    };
    if existing == next_content {
        return Ok(None);
    }
    let backup_path = output_path.with_file_name(format!(
        "{}.backup-before-visual-manager-{}",
        output_path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("add_fn"),
        timestamp_millis()
    ));
    fs::copy(output_path, &backup_path)?;
    Ok(Some(backup_path))
}

fn write_readonly_atomic(output_path: &Path, content: &str) -> io::Result<()> {
    let target_path = symlink_target_or_path(output_path);
    if let Some(parent) = target_path.parent() {
        fs::create_dir_all(parent)?;
    }
    let temp_path = target_path.with_file_name(format!(
        ".{}.tmp-{}",
        target_path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("add_fn"),
        timestamp_millis()
    ));
    fs::write(&temp_path, content)?;
    set_owner_readonly(&temp_path)?;
    unlock_manual_edit_guard(&target_path)?;
    fs::rename(&temp_path, &target_path)?;
    harden_readonly(&target_path)
}

fn symlink_target_or_path(path: &Path) -> PathBuf {
    if fs::symlink_metadata(path)
        .map(|metadata| metadata.file_type().is_symlink())
        .unwrap_or(false)
    {
        return fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf());
    }
    path.to_path_buf()
}

#[cfg(unix)]
fn harden_readonly(path: &Path) -> io::Result<()> {
    set_owner_readonly(path)?;
    lock_manual_edit_guard(path)
}

#[cfg(unix)]
fn set_owner_readonly(path: &Path) -> io::Result<()> {
    use std::os::unix::fs::PermissionsExt;

    fs::set_permissions(path, fs::Permissions::from_mode(0o400))
}

#[cfg(not(unix))]
fn harden_readonly(path: &Path) -> io::Result<()> {
    let mut permissions = fs::metadata(path)?.permissions();
    permissions.set_readonly(true);
    fs::set_permissions(path, permissions)
}

#[cfg(target_os = "macos")]
fn lock_manual_edit_guard(path: &Path) -> io::Result<()> {
    run_chflags("uchg", path)
}

#[cfg(all(unix, not(target_os = "macos")))]
fn lock_manual_edit_guard(_path: &Path) -> io::Result<()> {
    Ok(())
}

#[cfg(target_os = "macos")]
fn unlock_manual_edit_guard(path: &Path) -> io::Result<()> {
    if !path.exists() {
        return Ok(());
    }
    run_chflags("nouchg", path)
}

#[cfg(all(unix, not(target_os = "macos")))]
fn unlock_manual_edit_guard(_path: &Path) -> io::Result<()> {
    Ok(())
}

#[cfg(not(unix))]
fn unlock_manual_edit_guard(_path: &Path) -> io::Result<()> {
    Ok(())
}

#[cfg(target_os = "macos")]
fn run_chflags(flag: &str, path: &Path) -> io::Result<()> {
    let status = Command::new("chflags").arg(flag).arg(path).status()?;
    if status.success() {
        Ok(())
    } else {
        Err(io::Error::other(format!(
            "chflags {flag} {} 执行失败，状态为 {status}",
            path.display()
        )))
    }
}

fn parse_alias_name(trimmed: &str) -> Option<String> {
    let rest = trimmed.strip_prefix("alias ")?;
    let name = rest.split_once('=')?.0.trim();
    valid_shell_name(name).then(|| name.to_string())
}

fn parse_export_name(trimmed: &str) -> Option<String> {
    let rest = trimmed.strip_prefix("export ")?;
    let name = rest
        .split(['=', ' ', '\t'])
        .next()
        .unwrap_or_default()
        .trim();
    valid_shell_name(name).then(|| name.to_string())
}

fn parse_function_name(trimmed: &str) -> Option<String> {
    if let Some(rest) = trimmed.strip_prefix("function ") {
        let name = rest
            .split([' ', '\t', '('])
            .next()
            .unwrap_or_default()
            .trim();
        return valid_shell_name(name).then(|| name.to_string());
    }

    let (name, rest) = trimmed.split_once("()")?;
    let name = name.trim();
    (valid_shell_name(name) && rest.trim_start().starts_with('{')).then(|| name.to_string())
}

fn valid_shell_name(name: &str) -> bool {
    let mut chars = name.chars();
    match chars.next() {
        Some(first) if first == '_' || first.is_ascii_alphabetic() => {}
        _ => return false,
    }
    chars.all(|char| char == '_' || char.is_ascii_alphanumeric())
}

fn heredoc_delimiter(id: &str) -> String {
    format!("__ADD_FN_{}__", sanitize_id(id).replace('-', "_"))
}

fn sanitize_id(value: &str) -> String {
    value
        .chars()
        .map(|char| {
            if char.is_ascii_alphanumeric() {
                char.to_ascii_lowercase()
            } else {
                '_'
            }
        })
        .collect()
}

fn normalize_deployment_paths(items: &mut [ShellManagedItem]) {
    let canonical_path = canonical_add_fn_path().display().to_string();
    for item in items {
        item.deployment_paths = vec![canonical_path.clone()];
        item.draft_path = canonical_path.clone();
    }
}

fn canonical_add_fn_path() -> PathBuf {
    home_dir().join(".add_fn")
}

fn timestamp_millis() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis())
        .unwrap_or_default()
}

fn generated_paths(snapshot: &HostSnapshot) -> Vec<String> {
    dedupe_non_empty(
        snapshot
            .generated_files
            .iter()
            .map(|file| file.path.clone())
            .collect(),
    )
}

fn shell_item_sections(items: &[ShellManagedItem]) -> Vec<String> {
    let mut sections = Vec::new();
    for item in items {
        if !sections.iter().any(|section| section == &item.section) {
            sections.push(item.section.clone());
        }
    }
    sections
}

fn shell_search_fields() -> Vec<AzGrammarSearchField> {
    vec![
        AzGrammarSearchField::new("关键词", "全文匹配"),
        AzGrammarSearchField::new("标签", "多标签"),
        AzGrammarSearchField::new("定义", "别名 / 函数 / 环境变量"),
        AzGrammarSearchField::new("分组", "来源分组"),
        AzGrammarSearchField::new("来源", "识别路径"),
        AzGrammarSearchField::new("部署", "部署路径"),
        AzGrammarSearchField::new("内容", "脚本内容"),
    ]
}

fn default_shell_tags(kind: ShellEntryKind, section: &str, is_user_created: bool) -> Vec<String> {
    let mut tags = vec![shell_kind_default_tag(kind).to_string()];

    if is_user_created {
        tags.push("手动".to_string());
    } else {
        tags.push("扫描".to_string());
    }

    if is_profile_section(section) {
        tags.push("登录环境".to_string());
    }
    if is_bin_script_section(section) {
        tags.push("脚本目录".to_string());
    }
    if is_zsh_section(section) {
        tags.push("zsh".to_string());
    }
    if section.contains("local") {
        tags.push("本地".to_string());
    }

    dedupe_non_empty(tags)
}

fn parse_shell_tags(input: &str) -> Vec<String> {
    dedupe_non_empty(
        input
            .split([',', '，', ';', '；', ' ', '\n', '\t'])
            .map(normalize_tag)
            .collect(),
    )
}

fn merge_shell_tags(saved_tags: Vec<String>, default_tags: Vec<String>) -> Vec<String> {
    let mut merged = default_tags;
    merged.extend(saved_tags.into_iter().map(|tag| normalize_tag(&tag)));
    dedupe_non_empty(merged)
}

fn normalize_tag(tag: &str) -> String {
    tag.trim()
        .trim_start_matches('#')
        .chars()
        .filter_map(|char| {
            if char.is_alphanumeric() || char == '_' || char == '-' {
                Some(char.to_ascii_lowercase())
            } else if char.is_whitespace() {
                Some('-')
            } else {
                None
            }
        })
        .collect()
}

fn shell_kind_default_tag(kind: ShellEntryKind) -> &'static str {
    match kind {
        ShellEntryKind::Alias => "别名",
        ShellEntryKind::Export => "环境变量",
        ShellEntryKind::Function => "函数",
        ShellEntryKind::ScriptSnippet => "脚本片段",
    }
}

fn shell_kind_search_keys(kind: ShellEntryKind) -> &'static [&'static str] {
    match kind {
        ShellEntryKind::Alias => &["别名", "命令", "alias", "cmd", "cli", "command"],
        ShellEntryKind::Export => &["环境变量", "变量", "export", "env", "var"],
        ShellEntryKind::Function => &["函数", "fun", "function", "fn", "cli"],
        ShellEntryKind::ScriptSnippet => &["脚本片段", "脚本", "snippet", "script", "sh"],
    }
}

fn contains_ci(value: &str, query: &str) -> bool {
    value.to_lowercase().contains(&query.to_lowercase())
}

fn eq_ci(value: &str, query: &str) -> bool {
    value.eq_ignore_ascii_case(query)
}

fn shell_manager_store_path() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| home_dir().join(".config"))
        .join("addzero")
        .join("az-aio")
        .join(STORE_FILE)
}

fn home_dir() -> PathBuf {
    env::var_os("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."))
}

fn compact_home_path(path: &Path) -> String {
    compact_home_path_str(&path.display().to_string())
}

fn compact_home_path_str(path: &str) -> String {
    let path = path.trim();
    if path.is_empty()
        || path == "~"
        || path == "$HOME"
        || path.starts_with("~/")
        || path.starts_with("$HOME/")
    {
        return path.to_string();
    }

    let home = home_dir();
    let Ok(relative_path) = Path::new(path).strip_prefix(&home) else {
        return path.to_string();
    };

    if relative_path.as_os_str().is_empty() {
        "~".to_string()
    } else {
        format!("~/{}", relative_path.display())
    }
}

fn dedupe_non_empty(values: Vec<String>) -> Vec<String> {
    let mut seen = HashSet::new();
    let mut deduped = Vec::new();
    for value in values {
        let value = value.trim().to_string();
        if !value.is_empty() && seen.insert(value.clone()) {
            deduped.push(value);
        }
    }
    deduped
}

fn ensure_trailing_newline(value: &str) -> String {
    let mut value = value.trim_end().to_string();
    value.push('\n');
    value
}

fn generated_status_class(status: GeneratedFileStatus) -> &'static str {
    match status {
        GeneratedFileStatus::Generated => "metadata-status metadata-status--generated",
        GeneratedFileStatus::Failed => "metadata-status metadata-status--failed",
    }
}

fn generated_status_label(status: GeneratedFileStatus) -> &'static str {
    match status {
        GeneratedFileStatus::Generated => "已生成",
        GeneratedFileStatus::Failed => "失败",
    }
}

fn shell_kind_class(kind: ShellEntryKind) -> &'static str {
    match kind {
        ShellEntryKind::Alias => "metadata-kind metadata-kind--alias",
        ShellEntryKind::Export => "metadata-kind metadata-kind--export",
        ShellEntryKind::Function => "metadata-kind metadata-kind--function",
        ShellEntryKind::ScriptSnippet => "metadata-kind metadata-kind--snippet",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_item(
        id: &str,
        kind: ShellEntryKind,
        name: &str,
        section: &str,
        body: &str,
    ) -> ShellManagedItem {
        ShellManagedItem {
            id: id.to_string(),
            kind,
            name: name.to_string(),
            section: section.to_string(),
            source_path: "manual".to_string(),
            line_start: 0,
            body: body.to_string(),
            deployment_paths: vec![canonical_add_fn_path().display().to_string()],
            draft_path: canonical_add_fn_path().display().to_string(),
            tags: default_shell_tags(kind, section, true),
            is_user_created: true,
            deleted: false,
            source_missing: false,
        }
    }

    #[test]
    fn render_add_fn_skips_logically_deleted_items() {
        let mut items = vec![
            test_item(
                "managed.cli.keep",
                ShellEntryKind::Alias,
                "ll",
                "manual",
                "alias ll='ls -la'\n",
            ),
            test_item(
                "managed.env.deleted",
                ShellEntryKind::Export,
                "REMOVED_TOKEN",
                "manual",
                "export REMOVED_TOKEN=secret\n",
            ),
        ];
        items[1].deleted = true;

        let output = render_add_fn(&active_shell_items(&items));

        assert!(output.contains("alias ll='ls -la'"));
        assert!(!output.contains("REMOVED_TOKEN"));
    }

    #[test]
    fn render_add_fn_wraps_scripts_as_functions() {
        let items = vec![test_item(
            "managed.script.demo",
            ShellEntryKind::ScriptSnippet,
            "demo-script",
            "home-bin",
            "#!/usr/bin/env bash\necho demo\n",
        )];

        let output = render_add_fn(&active_shell_items(&items));

        assert!(output.contains("demo-script() {"));
        assert!(output.contains("ADD_FN_MODE=script"));
        assert!(output.contains("echo demo"));
    }

    #[test]
    fn render_add_fn_prefers_alias_over_same_name_bin_script() {
        let items = vec![
            test_item(
                "managed.alias.pnpm",
                ShellEntryKind::Alias,
                "pnpm",
                "manual",
                "alias pnpm='bun'\n",
            ),
            test_item(
                "managed.script.pnpm",
                ShellEntryKind::ScriptSnippet,
                "pnpm",
                "home-bin",
                "#!/usr/bin/env bash\necho pnpm script\n",
            ),
        ];

        let output = render_add_fn(&active_shell_items(&items));

        assert!(output.contains("alias pnpm='bun'"));
        assert!(!output.contains("pnpm() {"));
        assert!(!output.contains("echo pnpm script"));
    }

    #[test]
    fn render_add_fn_keeps_profile_and_interactive_sections_separate() {
        let env_item = test_item(
            "managed.env.editor",
            ShellEntryKind::Export,
            "EDITOR",
            "user-managed",
            "export EDITOR=nvim\n",
        );
        let alias_item = test_item(
            "managed.alias.ll",
            ShellEntryKind::Alias,
            "ll",
            "user-managed",
            "alias ll='ls -la'\n",
        );

        let output = render_add_fn(&active_shell_items(&[env_item, alias_item]));

        assert!(output.contains("##### profile/export"));
        assert!(output.contains("##### interactive/alias"));
        assert!(output.contains("export EDITOR=nvim"));
        assert!(output.contains("alias ll='ls -la'"));
    }

    #[test]
    fn render_add_fn_keeps_file_snippet_defs_in_snippet_section() {
        let items = vec![
            test_item(
                "managed.env.path",
                ShellEntryKind::Export,
                "EDITOR",
                "profile.d",
                "export EDITOR=nvim\n",
            ),
            test_item(
                "managed.snippet.profile",
                ShellEntryKind::ScriptSnippet,
                "profile.d/user.sh",
                "profile.d",
                "export EDITOR=vim\necho keep\n",
            ),
        ];

        let output = render_add_fn(&active_shell_items(&items));

        assert!(!output.contains("# 来源：manual\nexport EDITOR=nvim"));
        assert!(output.contains("export EDITOR=vim"));
        assert!(output.contains("echo keep"));
    }

    #[test]
    fn render_add_fn_drops_legacy_recursive_self_source() {
        let items = vec![test_item(
            "managed.snippet.compat",
            ShellEntryKind::ScriptSnippet,
            "rc.d/90-local-compat.sh",
            "rc.d",
            "[ \"${ENABLE_LEGACY_ADD_FN:-0}\" = 1 ] && [ -f \"$HOME/.add_fn\" ] && . \"$HOME/.add_fn\"\n",
        )];

        let output = render_add_fn(&active_shell_items(&items));

        assert!(!output.contains("ENABLE_LEGACY_ADD_FN"));
        assert!(!output.contains("[ -f \"$HOME/.add_fn\" ] && . \"$HOME/.add_fn\""));
    }

    #[test]
    fn compact_home_path_shortens_current_home_paths() {
        let home = home_dir();
        let add_fn_path = home.join(".add_fn");
        let config_path = home.join(".config").join("shell");

        assert_eq!(compact_home_path(&add_fn_path), "~/.add_fn");
        assert_eq!(
            compact_home_path_str(&config_path.display().to_string()),
            "~/.config/shell"
        );
        assert_eq!(compact_home_path_str("~/.add_fn"), "~/.add_fn");
        assert_eq!(compact_home_path_str("$HOME/.add_fn"), "$HOME/.add_fn");
    }

    #[test]
    fn grammar_query_filters_by_keyword_tags_and_definition_type() {
        let mut alias = test_item(
            "managed.alias.addhost",
            ShellEntryKind::Alias,
            "addhost",
            "manual",
            "alias addhost='ssh addzero'\n",
        );
        alias.tags = parse_shell_tags("rust,java,ops");
        let mut export = test_item(
            "managed.env.java_home",
            ShellEntryKind::Export,
            "JAVA_HOME",
            "profile.d",
            "export JAVA_HOME=/Library/Java\n",
        );
        export.tags = parse_shell_tags("java,env");
        let query = parse_grammar_search_query("关键词:addhost；标签:rust,java；定义:alias");

        assert!(alias.matches_query(&query));
        assert!(!export.matches_query(&query));
    }

    #[test]
    fn write_readonly_atomic_updates_readonly_file() {
        let temp_dir =
            std::env::temp_dir().join(format!("az-aio-shell-manager-{}", timestamp_millis()));
        fs::create_dir_all(&temp_dir).expect("create temp dir");
        let path = temp_dir.join(".add_fn");
        fs::write(&path, "old\n").expect("write old content");
        harden_readonly(&path).expect("make read only");

        write_readonly_atomic(&path, "new\n").expect("update readonly file");
        let content = fs::read_to_string(&path).expect("read updated file");

        assert_eq!(content, "new\n");
        let _ = unlock_manual_edit_guard(&path);
        let _ = fs::remove_dir_all(temp_dir);
    }
}
