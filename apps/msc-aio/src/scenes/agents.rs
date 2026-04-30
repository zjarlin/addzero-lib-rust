use std::collections::BTreeMap;

use chrono::{DateTime, Utc};
use dioxus::prelude::*;
use dioxus_components::{
    Badge, ConfirmDialog, ContentHeader, Divider, Field, KeywordChips, MetricRow, MetricStrip,
    SidebarSection, Stack, Surface, SurfaceHeader, Textarea, Tone, WorkbenchButton,
};

use crate::app::Route;
use crate::services::{SkillDto, SkillSourceDto, SkillUpsertDto, SyncReportDto};
use crate::state::AppServices;

#[component]
pub fn Agents() -> Element {
    let mut search = use_signal(String::new);
    let mut feedback = use_signal::<Option<String>>(|| None);
    let skills_api = use_context::<AppServices>().skills.clone();

    let mut skills_resource = {
        let skills_api = skills_api.clone();
        use_resource(move || {
            let skills_api = skills_api.clone();
            async move { skills_api.list_skills().await }
        })
    };

    let do_sync = move || {
        let skills_api = skills_api.clone();
        spawn(async move {
            match skills_api.sync_skills().await {
                Ok(report) => {
                    feedback.set(Some(format_sync_report(&report)));
                    skills_resource.restart();
                }
                Err(err) => feedback.set(Some(format!("同步失败：{err}"))),
            }
        });
    };

    let skills_view = skills_resource.read();
    let raw_skills = match skills_view.as_ref() {
        Some(Ok(list)) => list.clone(),
        Some(Err(err)) => {
            return rsx! {
                ContentHeader {
                    title: "Agent资产".to_string(),
                    subtitle: "当前先把 SKILL.md 作为 Skills 资产管理，后续再扩到其它 Agent 资产。".to_string()
                }
                Surface {
                    div { class: "callout",
                        "无法加载技能列表：{err}"
                    }
                }
            };
        }
        None => {
            return rsx! {
                ContentHeader {
                    title: "Agent资产".to_string(),
                    subtitle: "当前先把 SKILL.md 作为 Skills 资产管理，后续再扩到其它 Agent 资产。".to_string()
                }
                Surface { div { class: "empty-state", "正在加载…" } }
            };
        }
    };

    let query = search.read().to_lowercase();
    let filtered: Vec<SkillDto> = raw_skills
        .iter()
        .filter(|skill| {
            if query.is_empty() {
                true
            } else {
                skill.name.to_lowercase().contains(&query)
                    || skill
                        .keywords
                        .iter()
                        .any(|keyword| keyword.to_lowercase().contains(&query))
                    || skill.description.to_lowercase().contains(&query)
            }
        })
        .cloned()
        .collect();
    let grouped_skills = build_skill_tree(&filtered);

    rsx! {
        ContentHeader {
            title: "Agent资产".to_string(),
            subtitle: "当前先把 SKILL.md 作为 Skills 资产管理，后续再扩到模型、提示词和工具配置等资产。".to_string(),
            actions: rsx!(
                Link { to: Route::AgentEditor { name: "_new".to_string() },
                    WorkbenchButton { class: "action-button".to_string(), tone: Tone::Accent, "新增 Skills 资产" }
                }
            )
        }
        Surface {
            SurfaceHeader {
                title: "Skills 资产".to_string(),
                subtitle: "按名称或关键词搜索；以树状目录呈现技能资产，可同时看到分组与条目。".to_string(),
                actions: rsx!(
                    WorkbenchButton {
                        class: "toolbar-button".to_string(),
                        onclick: move |_| do_sync(),
                        "手动同步"
                    }
                )
            }
            div { class: "toolbar",
                input {
                    class: "toolbar__search",
                    placeholder: "按名称 / 关键词搜索",
                    value: search.read().clone(),
                    oninput: move |evt| search.set(evt.value())
                }
                span { class: "toolbar__spacer" }
                span { class: "cell-overflow", "共 {raw_skills.len()} 条" }
            }
            if let Some(msg) = feedback.read().as_ref() {
                div { class: "callout callout--info", "{msg}" }
            }
            if filtered.is_empty() {
                div { class: "empty-state", "没有匹配的技能。" }
            } else {
                div { class: "knowledge-board",
                    Surface {
                        SurfaceHeader {
                            title: "技能目录树".to_string(),
                            subtitle: "按命名空间 / 前缀分组；点击节点直接进入编辑。".to_string()
                        }
                        div { class: "stack",
                            for group in grouped_skills.iter() {
                                div { class: "sidebar-section",
                                    div { class: "context-line",
                                        strong { "{group.label}" }
                                        span { class: "cell-overflow", "{group.items.len()} 项" }
                                    }
                                    div { class: "stack", style: "padding-left: 12px;",
                                        for skill in group.items.iter() {
                                            SkillTreeItem { skill: skill.clone() }
                                        }
                                    }
                                }
                            }
                        }
                    }
                    Surface {
                        SurfaceHeader {
                            title: "目录说明".to_string(),
                            subtitle: "支持多个部署路径；同一技能组下会并列展示多个条目。".to_string()
                        }
                        div { class: "stack",
                            for group in grouped_skills.iter() {
                                div { class: "context-line",
                                    strong { "{group.label}" }
                                    span { " · {group.items.len()} 项" }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

#[derive(Clone, PartialEq)]
struct SkillTreeGroup {
    label: String,
    items: Vec<SkillDto>,
}

#[component]
fn SkillTreeItem(skill: SkillDto) -> Element {
    let nav = use_navigator();
    let route = Route::AgentEditor {
        name: skill.name.clone(),
    };
    let preview = if skill.keywords.is_empty() {
        Some(skill.description.clone())
    } else {
        Some(skill.keywords.iter().take(3).cloned().collect::<Vec<_>>().join(" · "))
    };
    let badge = source_badge_props(&skill.source);
    let updated = skill.updated_at.format("%Y-%m-%d %H:%M").to_string();

    rsx! {
        div {
            class: "row-link",
            onclick: move |_| {
                nav.push(route.clone());
            },
            div { class: "context-line",
                strong { "{skill.name}" }
                span { class: "cell-overflow", "{updated}" }
            }
            div { class: "context-line",
                Badge { label: badge.0, variant: badge.1 }
                if let Some(text) = preview.as_ref() {
                    span { "{text}" }
                }
            }
        }
    }
}

fn build_skill_tree(skills: &[SkillDto]) -> Vec<SkillTreeGroup> {
    let mut groups: BTreeMap<String, Vec<SkillDto>> = BTreeMap::new();

    for skill in skills {
        let group = skill_group_label(&skill.name);
        groups.entry(group).or_default().push(skill.clone());
    }

    groups
        .into_iter()
        .map(|(label, mut items)| {
            items.sort_by(|a, b| a.name.cmp(&b.name));
            SkillTreeGroup { label, items }
        })
        .collect()
}

fn skill_group_label(name: &str) -> String {
    if let Some((head, _)) = name.split_once('/') {
        return head.to_string();
    }
    if let Some((head, _)) = name.split_once(':') {
        return head.to_string();
    }
    name.split('-')
        .next()
        .filter(|value| !value.is_empty())
        .unwrap_or("ungrouped")
        .to_string()
}

fn source_badge_props(source: &SkillSourceDto) -> (String, String) {
    match source {
        SkillSourceDto::Postgres => ("PG".into(), "pg".into()),
        SkillSourceDto::FileSystem => ("FS".into(), "fs".into()),
        SkillSourceDto::Both => ("Both".into(), "both".into()),
    }
}

#[component]
pub fn AgentEditor(name: String) -> Element {
    let nav = use_navigator();
    let skills_api = use_context::<AppServices>().skills.clone();
    let is_new = name == "_new";
    let load_name = name.clone();
    let mut name_state = use_signal(|| if is_new { String::new() } else { name.clone() });
    let mut keywords_state = use_signal(Vec::new);
    let mut description_state = use_signal(String::new);
    let mut body_state = use_signal(String::new);
    let mut source_state = use_signal(|| SkillSourceDto::FileSystem);
    let mut updated_at_state = use_signal::<Option<DateTime<Utc>>>(|| None);
    let mut hash_state = use_signal(String::new);
    let mut feedback = use_signal::<Option<String>>(|| None);
    let mut confirm_open = use_signal(|| false);
    let mut loading = use_signal(|| !is_new);

    let _loader = {
        let skills_api = skills_api.clone();
        use_resource(move || {
            let skills_api = skills_api.clone();
            let load_name = load_name.clone();
            async move {
                if is_new {
                    loading.set(false);
                    return;
                }
                match skills_api.get_skill(load_name).await {
                    Ok(Some(skill)) => {
                        name_state.set(skill.name.clone());
                        keywords_state.set(skill.keywords.clone());
                        description_state.set(skill.description.clone());
                        body_state.set(skill.body.clone());
                        source_state.set(skill.source.clone());
                        updated_at_state.set(Some(skill.updated_at));
                        hash_state.set(skill.content_hash.clone());
                        loading.set(false);
                    }
                    Ok(None) => {
                        feedback.set(Some("未找到该技能。".into()));
                        loading.set(false);
                    }
                    Err(err) => {
                        feedback.set(Some(format!("加载失败：{err}")));
                        loading.set(false);
                    }
                }
            }
        })
    };

    let save_skills_api = skills_api.clone();
    let save_nav = nav;
    let save = move |_| {
        let payload = SkillUpsertDto {
            name: name_state.read().trim().to_string(),
            keywords: keywords_state.read().clone(),
            description: description_state.read().clone(),
            body: body_state.read().clone(),
        };
        if payload.name.is_empty() {
            feedback.set(Some("名称不能为空。".into()));
            return;
        }

        let skills_api = save_skills_api.clone();
        spawn(async move {
            feedback.set(Some("正在保存…".into()));
            match skills_api.upsert_skill(payload).await {
                Ok(skill) => {
                    feedback.set(Some(format!("已保存（{}）。", source_label(&skill.source))));
                    name_state.set(skill.name.clone());
                    source_state.set(skill.source.clone());
                    updated_at_state.set(Some(skill.updated_at));
                    hash_state.set(skill.content_hash.clone());
                    if is_new {
                        save_nav.replace(Route::AgentEditor {
                            name: skill.name.clone(),
                        });
                    }
                }
                Err(err) => feedback.set(Some(format!("保存失败：{err}"))),
            }
        });
    };

    let request_delete = move |_| confirm_open.set(true);
    let cancel_delete = move |_: ()| confirm_open.set(false);
    let delete_skills_api = skills_api.clone();
    let delete_nav = nav;
    let confirm_delete = move |_: ()| {
        confirm_open.set(false);
        let skills_api = delete_skills_api.clone();
        let skill_name = name_state.read().clone();
        spawn(async move {
            match skills_api.delete_skill(skill_name).await {
                Ok(()) => {
                    delete_nav.replace(Route::Agents);
                }
                Err(err) => feedback.set(Some(format!("删除失败：{err}"))),
            }
        });
    };

    let header_title = if is_new {
        "新增 Skills 资产".to_string()
    } else {
        format!("编辑：{name}")
    };
    let updated_display = match *updated_at_state.read() {
        Some(timestamp) => timestamp.format("%Y-%m-%d %H:%M:%S").to_string(),
        None => "—".into(),
    };
    let hash_display = {
        let hash = hash_state.read().clone();
        if hash.is_empty() {
            "—".to_string()
        } else {
            hash.chars().take(12).collect::<String>() + "…"
        }
    };
    let source_display = source_label(&source_state.read());

    rsx! {
        ContentHeader {
            title: header_title,
            subtitle: "管理 Skills 资产的 SKILL.md 元信息、关键词触发与正文。".to_string(),
            actions: rsx!(
                Link { to: Route::Agents,
                    WorkbenchButton { class: "toolbar-button".to_string(), "返回列表" }
                }
            )
        }
        if *loading.read() {
            Surface { div { class: "empty-state", "正在加载…" } }
        } else {
            Surface {
                SurfaceHeader {
                    title: "元信息".to_string(),
                    subtitle: "name 是文件夹与 PG 的主键，关键词决定触发范围。".to_string()
                }
                MetricStrip { columns: 2,
                    Field {
                        label: "名称".to_string(),
                        value: name_state.read().clone(),
                        readonly: !is_new,
                        on_input: move |value: String| name_state.set(value),
                        placeholder: "kebab-case，例如 my-skill".to_string(),
                    }
                    Field {
                        label: "来源".to_string(),
                        value: source_display.clone(),
                        readonly: true,
                    }
                    Field {
                        label: "最后更新".to_string(),
                        value: updated_display,
                        readonly: true,
                    }
                    Field {
                        label: "Content hash".to_string(),
                        value: hash_display,
                        readonly: true,
                    }
                }
                Divider {}
                div {
                    div { class: "field__label", style: "margin-bottom: 8px;", "触发关键词" }
                    KeywordChips {
                        value: keywords_state.read().clone(),
                        on_change: move |next: Vec<String>| keywords_state.set(next),
                    }
                    div { class: "context-line",
                        span { "保存时会写回 description 头部 \"当用户提到 …\" 模板段。" }
                    }
                }
            }
            Surface {
                SurfaceHeader {
                    title: "说明与正文".to_string(),
                    subtitle: "description 是 SKILL.md frontmatter；正文写技能的具体指令。".to_string()
                }
                Stack {
                    Textarea {
                        label: "Description".to_string(),
                        value: description_state.read().clone(),
                        rows: 4,
                        placeholder: "保存时关键词会自动渲染到开头".to_string(),
                        on_input: move |value: String| description_state.set(value),
                    }
                    Textarea {
                        label: "Body (Markdown)".to_string(),
                        value: body_state.read().clone(),
                        rows: 14,
                        monospace: true,
                        placeholder: "# 技能名\n\n操作指南……".to_string(),
                        on_input: move |value: String| body_state.set(value),
                    }
                }
            }
            if let Some(msg) = feedback.read().as_ref() {
                div { class: "callout callout--info", "{msg}" }
            }
            div { class: "editor-footer",
                if !is_new {
                    WorkbenchButton {
                        class: "toolbar-button".to_string(),
                        onclick: request_delete,
                        "删除"
                    }
                }
                span { class: "editor-footer__spacer" }
                Link { to: Route::Agents,
                    WorkbenchButton { class: "toolbar-button".to_string(), "取消" }
                }
                WorkbenchButton {
                    class: "action-button".to_string(),
                    tone: Tone::Accent,
                    onclick: save,
                    "保存"
                }
            }
            ConfirmDialog {
                open: *confirm_open.read(),
                title: "确认删除".to_string(),
                message: format!(
                    "将删除 SKILL.md 与（如果在线）PG 中的 {} 记录。该操作不可撤销。",
                    name_state.read()
                ),
                confirm_label: "删除".to_string(),
                cancel_label: "取消".to_string(),
                on_confirm: confirm_delete,
                on_cancel: cancel_delete,
            }
        }
    }
}

fn format_sync_report(report: &SyncReportDto) -> String {
    if !report.pg_online {
        return "PG 未连接：仅 fs 模式，无需同步。".into();
    }
    let total = report.added_to_fs.len()
        + report.added_to_pg.len()
        + report.updated_in_fs.len()
        + report.updated_in_pg.len();
    if total == 0 && report.conflicts.is_empty() {
        "已同步：两侧一致。".into()
    } else {
        format!(
            "同步完成：fs+{}, pg+{}, fs↑{}, pg↑{}, 冲突 {}",
            report.added_to_fs.len(),
            report.added_to_pg.len(),
            report.updated_in_fs.len(),
            report.updated_in_pg.len(),
            report.conflicts.len()
        )
    }
}

fn source_label(source: &SkillSourceDto) -> String {
    match source {
        SkillSourceDto::Postgres => "Postgres".into(),
        SkillSourceDto::FileSystem => "FileSystem".into(),
        SkillSourceDto::Both => "PG + FS".into(),
    }
}

#[component]
pub fn AgentContext() -> Element {
    let skills_api = use_context::<AppServices>().skills.clone();
    let status = {
        let skills_api = skills_api.clone();
        use_resource(move || {
            let skills_api = skills_api.clone();
            async move { skills_api.server_status().await }
        })
    };
    let view = status.read();

    let (pg_online, fs_root, last_report) = match view.as_ref() {
        Some(Ok(report)) => (
            report.pg_online,
            report.fs_root.clone(),
            Some(report.clone()),
        ),
        Some(Err(_)) | None => (false, String::new(), None),
    };

    rsx! {
        SidebarSection { label: "Agent 上下文".to_string(),
            Stack {
                MetricRow {
                    label: "Postgres".to_string(),
                    value: if pg_online { "Online".to_string() } else { "Offline".to_string() },
                    tone: if pg_online { Tone::Positive } else { Tone::Warning },
                }
                MetricRow {
                    label: "fs 根目录".to_string(),
                    value: if fs_root.is_empty() { "—".to_string() } else { fs_root.clone() },
                }
                if let Some(report) = &last_report {
                    MetricRow {
                        label: "上次同步冲突".to_string(),
                        value: report.conflicts.len().to_string(),
                        tone: if report.conflicts.is_empty() { Tone::Default } else { Tone::Warning },
                    }
                }
            }
        }
        if let Some(report) = last_report {
            SidebarSection { label: "最近同步".to_string(),
                if report.finished_at.is_none() {
                    div { class: "context-line", span { "尚未触发过同步。" } }
                } else {
                    Stack {
                        ContextLine { label: "fs 新增".to_string(), value: report.added_to_fs.len().to_string() }
                        ContextLine { label: "PG 新增".to_string(), value: report.added_to_pg.len().to_string() }
                        ContextLine { label: "fs 更新".to_string(), value: report.updated_in_fs.len().to_string() }
                        ContextLine { label: "PG 更新".to_string(), value: report.updated_in_pg.len().to_string() }
                        if !report.conflicts.is_empty() {
                            div { class: "callout",
                                "冲突项："
                                "{report.conflicts.join(\"，\")}"
                            }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn ContextLine(label: String, value: String) -> Element {
    rsx! {
        div { class: "context-line",
            span { "{label}" }
            span { class: "context-line__value", "{value}" }
        }
    }
}
