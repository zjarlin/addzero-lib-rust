use dioxus::prelude::*;
use dioxus_components::{ContentHeader, Surface, SurfaceHeader, Textarea, WorkbenchButton};
use dioxus_nox_markdown::{
    markdown,
    prelude::{MarkdownHandle, Mode, use_markdown_handle},
};

use crate::admin::domains::KNOWLEDGE_DOMAIN_ID;
use crate::services::{
    ChatMessageDto, ChatRequestDto, KnowledgeEntryDeleteDto, KnowledgeEntryUpsertDto,
    KnowledgeNoteDto, default_knowledge_entries_api, default_openai_chat_api,
};

const KNOWLEDGE_NOTES_PAGE_ID: &str = "knowledge-notes";

#[derive(Clone, Debug, PartialEq, Eq)]
struct NoteDraft {
    source_path: String,
    relative_path: String,
    body: String,
    dirty: bool,
}

impl NoteDraft {
    fn new_note() -> Self {
        Self {
            source_path: String::new(),
            relative_path: String::new(),
            body: "# 新笔记\n\n".to_string(),
            dirty: false,
        }
    }

    fn from_note(note: &KnowledgeNoteDto) -> Self {
        Self {
            source_path: note.source_path.clone(),
            relative_path: note.relative_path.clone(),
            body: note.body.clone(),
            dirty: false,
        }
    }

    fn title(&self) -> String {
        derive_markdown_title(&self.body)
    }
}

#[component]
pub fn KnowledgeNotes() -> Element {
    let notes_api = default_knowledge_entries_api();
    let chat_api = default_openai_chat_api();
    let mut selected_source_path = use_signal(String::new);
    let mut editor = use_signal(NoteDraft::new_note);
    let mut search = use_signal(String::new);
    let mut feedback = use_signal(|| None::<String>);
    let mut chat_draft = use_signal(String::new);
    let mut chat_messages = use_signal(Vec::<ChatMessageDto>::new);
    let mut apply_candidate = use_signal(|| None::<String>);
    let mut creating_new = use_signal(|| false);
    let notes_reload = use_signal(|| 0u64);
    let chat_pending = use_signal(|| false);
    let save_pending = use_signal(|| false);
    let delete_pending = use_signal(|| false);

    let notes_resource = {
        let notes_api = notes_api.clone();
        use_resource(move || {
            let notes_api = notes_api.clone();
            let _reload = *notes_reload.read();
            async move { notes_api.list_entries().await }
        })
    };

    {
        let mut selected_source_path = selected_source_path;
        let mut editor = editor;
        let creating_new = creating_new;
        let notes_resource = notes_resource;
        use_effect(move || {
            let snapshot = notes_resource.read().as_ref().cloned();
            let Some(Ok(notes)) = snapshot else {
                return;
            };

            let current = selected_source_path.read().clone();
            if current.is_empty() {
                if *creating_new.read() {
                    return;
                }
                if let Some(first) = notes.first() {
                    selected_source_path.set(first.source_path.clone());
                    editor.set(NoteDraft::from_note(first));
                }
                return;
            }

            if notes.iter().all(|note| note.source_path != current) {
                if let Some(first) = notes.first() {
                    selected_source_path.set(first.source_path.clone());
                    editor.set(NoteDraft::from_note(first));
                } else {
                    selected_source_path.set(String::new());
                    editor.set(NoteDraft::new_note());
                }
            }
        });
    }

    let notes_snapshot = notes_resource.read().as_ref().cloned();
    let load_error = match notes_snapshot.as_ref() {
        Some(Err(err)) => Some(err.clone()),
        _ => None,
    };
    let all_notes = match notes_snapshot {
        Some(Ok(notes)) => notes,
        _ => Vec::new(),
    };

    let query = search.read().trim().to_lowercase();
    let visible_notes = all_notes
        .iter()
        .filter(|note| {
            query.is_empty()
                || note.title.to_lowercase().contains(&query)
                || note.preview.to_lowercase().contains(&query)
                || note.relative_path.to_lowercase().contains(&query)
                || note
                    .headings
                    .iter()
                    .any(|heading| heading.to_lowercase().contains(&query))
        })
        .cloned()
        .collect::<Vec<_>>();

    let editor_title = editor.read().title();
    let outline = extract_headings_from_body(&editor.read().body);
    let current_source_path = editor.read().source_path.clone();
    let current_relative_path = editor.read().relative_path.clone();
    let can_save = !*save_pending.read() && !editor.read().body.trim().is_empty();
    let can_delete = !*delete_pending.read()
        && (!editor.read().source_path.trim().is_empty() || editor.read().dirty);
    let can_send_chat = !*chat_pending.read() && !chat_draft.read().trim().is_empty();

    rsx! {
        ContentHeader {
            title: "笔记工作台".to_string(),
            subtitle: "中间直接编辑 Markdown 笔记，底部用对话整理内容；笔记增删改查统一走 PostgreSQL。".to_string()
        }
        if let Some(message) = feedback.read().clone() {
            div {
                class: if message.contains("失败") || message.contains("缺少") { "callout" } else { "callout callout--info" },
                "{message}"
            }
        }
        div { class: "note-workspace",
            Surface {
                SurfaceHeader {
                    title: "笔记列表".to_string(),
                    subtitle: format!("{} / {} 条", visible_notes.len(), all_notes.len()),
                    actions: rsx!(
                        WorkbenchButton {
                            class: "toolbar-button".to_string(),
                            onclick: move |_| {
                                selected_source_path.set(String::new());
                                editor.set(NoteDraft::new_note());
                                creating_new.set(true);
                                chat_messages.set(Vec::new());
                                apply_candidate.set(None);
                                feedback.set(Some("已切换到新建笔记。".to_string()));
                            },
                            "新建"
                        }
                    )
                }
                div { class: "toolbar",
                    input {
                        "data-command-search": "true",
                        class: "toolbar__search",
                        r#type: "search",
                        value: search.read().clone(),
                        placeholder: "搜索标题、摘要或路径",
                        oninput: move |evt| search.set(evt.value())
                    }
                }
                if let Some(err) = load_error {
                    div { class: "callout", "{err}" }
                }
                div { class: "note-list",
                    if visible_notes.is_empty() {
                        div { class: "empty-state", "还没有可编辑笔记，先新建一条。" }
                    } else {
                        for note in visible_notes {
                            NoteListRow {
                                note: note.clone(),
                                active: note.source_path == selected_source_path.read().as_str(),
                                on_select: move |selected: KnowledgeNoteDto| {
                                    creating_new.set(false);
                                    selected_source_path.set(selected.source_path.clone());
                                    editor.set(NoteDraft::from_note(&selected));
                                    chat_messages.set(Vec::new());
                                    apply_candidate.set(None);
                                }
                            }
                        }
                    }
                }
            }
            div { class: "note-workspace__main",
                Surface {
                    SurfaceHeader {
                        title: editor_title.clone(),
                        subtitle: if current_source_path.is_empty() {
                            "新建状态，保存后会写入 PG knowledge_documents。".to_string()
                        } else {
                            format!("{} · {}", current_relative_path, current_source_path)
                        },
                        actions: rsx!(
                            WorkbenchButton {
                                class: "toolbar-button".to_string(),
                                disabled: !can_save,
                                onclick: {
                                    let notes_api = notes_api.clone();
                                    let draft = editor.read().clone();
                                    let mut editor_signal = editor;
                                    let mut feedback = feedback;
                                    let mut notes_reload = notes_reload;
                                    let mut save_pending = save_pending;
                                    let mut creating_new = creating_new;
                                    let mut selected_source_path = selected_source_path;
                                    move |_| {
                                        if draft.body.trim().is_empty() {
                                            feedback.set(Some("笔记内容不能为空。".to_string()));
                                            return;
                                        }
                                        save_pending.set(true);
                                        let payload = KnowledgeEntryUpsertDto {
                                            source_path: draft.source_path.clone(),
                                            relative_path: draft.relative_path.clone(),
                                            title: derive_markdown_title(&draft.body),
                                            body: draft.body.clone(),
                                            tags: Vec::new(),
                                        };
                                        let notes_api = notes_api.clone();
                                        spawn(async move {
                                            match notes_api.save_entry(payload).await {
                                                Ok(saved) => {
                                                    creating_new.set(false);
                                                    selected_source_path.set(saved.source_path.clone());
                                                    editor_signal.set(NoteDraft::from_note(&saved));
                                                    notes_reload.with_mut(|value| *value += 1);
                                                    feedback.set(Some(format!("已保存：{}", saved.title)));
                                                }
                                                Err(err) => feedback.set(Some(format!("保存失败：{err}"))),
                                            }
                                            save_pending.set(false);
                                        });
                                    }
                                },
                                if *save_pending.read() { "保存中…" } else { "保存" }
                            }
                            WorkbenchButton {
                                class: "toolbar-button".to_string(),
                                disabled: !can_delete,
                                onclick: {
                                    let notes_api = notes_api.clone();
                                    let source_path = current_source_path.clone();
                                    let mut selected_source_path = selected_source_path;
                                    let mut editor = editor;
                                    let mut feedback = feedback;
                                    let mut notes_reload = notes_reload;
                                    let mut delete_pending = delete_pending;
                                    let mut creating_new = creating_new;
                                    move |_| {
                                        if source_path.trim().is_empty() {
                                            creating_new.set(true);
                                            editor.set(NoteDraft::new_note());
                                            feedback.set(Some("已清空当前草稿。".to_string()));
                                            return;
                                        }
                                        delete_pending.set(true);
                                        let notes_api = notes_api.clone();
                                        let source_path = source_path.clone();
                                        spawn(async move {
                                            match notes_api
                                                .delete_entry(KnowledgeEntryDeleteDto { source_path })
                                                .await
                                            {
                                                Ok(()) => {
                                                    creating_new.set(false);
                                                    selected_source_path.set(String::new());
                                                    editor.set(NoteDraft::new_note());
                                                    notes_reload.with_mut(|value| *value += 1);
                                                    feedback.set(Some("已删除当前笔记。".to_string()));
                                                }
                                                Err(err) => feedback.set(Some(format!("删除失败：{err}"))),
                                            }
                                            delete_pending.set(false);
                                        });
                                    }
                                },
                                if *delete_pending.read() { "删除中…" } else { "删除" }
                            }
                        )
                    }
                    div { class: "knowledge-meta",
                        span { class: "badge badge--fs", "PostgreSQL" }
                        if editor.read().dirty {
                            span { class: "badge", "未保存" }
                        } else {
                            span { class: "badge", "已同步" }
                        }
                        for heading in outline.iter().take(3) {
                            span { class: "badge", "{heading}" }
                        }
                    }
                    WorkspaceNoteEditor {
                        value: editor.read().body.clone(),
                        on_change: move |value| {
                            editor.with_mut(|draft| {
                                draft.body = value;
                                draft.dirty = true;
                            });
                        }
                    }
                }
                Surface {
                    SurfaceHeader {
                        title: "整理对话".to_string(),
                        subtitle: "在这里说“重写成周报 / 提炼 TODO / 改成更清晰结构”；助手返回的 Markdown 可直接回填到上面的笔记。".to_string(),
                        actions: rsx!(
                            WorkbenchButton {
                                class: "toolbar-button".to_string(),
                                disabled: apply_candidate.read().is_none(),
                                onclick: {
                                    let next = apply_candidate.read().clone();
                                    let mut editor = editor;
                                    let mut feedback = feedback;
                                    move |_| {
                                        let Some(next) = next.clone() else {
                                            return;
                                        };
                                        editor.with_mut(|draft| {
                                            draft.body = next.clone();
                                            draft.dirty = true;
                                        });
                                        feedback.set(Some("已把整理结果回填到编辑区。".to_string()));
                                    }
                                },
                                "应用到笔记"
                            }
                        )
                    }
                    div { class: "note-assistant__thread",
                        if chat_messages.read().is_empty() {
                            div { class: "empty-state", "先描述你的整理目标，例如：提炼成会议纪要、保留原文链接、拆成 TODO。"}
                        } else {
                            for message in chat_messages.read().iter() {
                                div {
                                    class: if message.role == "user" {
                                        "note-assistant__message note-assistant__message--user"
                                    } else {
                                        "note-assistant__message note-assistant__message--assistant"
                                    },
                                    strong {
                                        if message.role == "user" { "你" } else { "整理助手" }
                                    }
                                    div { class: "note-assistant__body", "{message.content}" }
                                }
                            }
                        }
                    }
                    div { class: "note-assistant__composer",
                        Textarea {
                            label: "整理要求".to_string(),
                            value: chat_draft.read().clone(),
                            rows: Some(5),
                            placeholder: Some("例如：把这份笔记改成更清晰的 Markdown，并单独列出 TODO / 风险 / 下一步。".to_string()),
                            on_input: move |value| chat_draft.set(value)
                        }
                        div { class: "entry-actions",
                            WorkbenchButton {
                                class: "action-button action-button--primary".to_string(),
                                disabled: !can_send_chat,
                                onclick: {
                                    let chat_api = chat_api.clone();
                                    let request_text = chat_draft.read().trim().to_string();
                                    let note_body = editor.read().body.clone();
                                    let mut chat_messages = chat_messages;
                                    let mut chat_draft = chat_draft;
                                    let mut apply_candidate = apply_candidate;
                                    let mut chat_pending = chat_pending;
                                    let mut feedback = feedback;
                                    move |_| {
                                        if request_text.is_empty() || note_body.trim().is_empty() {
                                            return;
                                        }
                                        chat_pending.set(true);
                                        chat_messages.with_mut(|items| {
                                            items.push(ChatMessageDto {
                                                role: "user".to_string(),
                                                content: request_text.clone(),
                                            });
                                        });
                                        chat_draft.set(String::new());
                                        let chat_api = chat_api.clone();
                                        let request = ChatRequestDto {
                                            messages: vec![
                                                ChatMessageDto {
                                                    role: "system".to_string(),
                                                    content: "你是笔记整理助手。请根据用户要求整理 Markdown 笔记，只返回可直接覆盖原笔记的 Markdown 正文，不要解释，不要使用代码围栏。".to_string(),
                                                },
                                                ChatMessageDto {
                                                    role: "user".to_string(),
                                                    content: format!(
                                                        "整理要求：\n{}\n\n当前笔记 Markdown：\n{}",
                                                        request_text, note_body
                                                    ),
                                                },
                                            ],
                                        };
                                        spawn(async move {
                                            match chat_api.chat(request).await {
                                                Ok(response) => {
                                                    let cleaned = strip_markdown_fences(&response.message.content);
                                                    apply_candidate.set(Some(cleaned.clone()));
                                                    chat_messages.with_mut(|items| {
                                                        items.push(ChatMessageDto {
                                                            role: "assistant".to_string(),
                                                            content: cleaned,
                                                        });
                                                    });
                                                    feedback.set(None);
                                                }
                                                Err(err) => feedback.set(Some(format!("整理失败：{err}"))),
                                            }
                                            chat_pending.set(false);
                                        });
                                    }
                                },
                                if *chat_pending.read() { "整理中…" } else { "发送整理要求" }
                            }
                            WorkbenchButton {
                                class: "action-button".to_string(),
                                disabled: chat_messages.read().is_empty(),
                                onclick: {
                                    let mut chat_messages = chat_messages;
                                    let mut apply_candidate = apply_candidate;
                                    move |_| {
                                        chat_messages.set(Vec::new());
                                        apply_candidate.set(None);
                                    }
                                },
                                "清空对话"
                            }
                        }
                    }
                }
            }
        }
    }
}

#[derive(Props, Clone, PartialEq)]
struct NoteListRowProps {
    note: KnowledgeNoteDto,
    active: bool,
    on_select: EventHandler<KnowledgeNoteDto>,
}

#[component]
fn NoteListRow(props: NoteListRowProps) -> Element {
    let class = if props.active {
        "note-list__row is-active"
    } else {
        "note-list__row"
    };
    let note = props.note.clone();
    let on_select = props.on_select.clone();

    rsx! {
        button {
            class,
            r#type: "button",
            onclick: {
                move |_| on_select.call(note.clone())
            },
            div { class: "note-list__row-head",
                strong { "{props.note.title}" }
                span { class: "cell-overflow", "{props.note.filename}" }
            }
            div { class: "note-list__row-body", "{props.note.preview}" }
            div { class: "note-list__row-meta",
                span { "{props.note.relative_path}" }
                span { "{props.note.headings.len()} 个标题" }
            }
        }
    }
}

#[derive(Props, Clone, PartialEq)]
struct WorkspaceNoteEditorProps {
    value: String,
    on_change: EventHandler<String>,
}

#[component]
fn WorkspaceNoteEditor(props: WorkspaceNoteEditorProps) -> Element {
    let mut editor_mode = use_signal(|| Mode::LivePreview);
    let mut editor_value = use_signal(|| props.value.clone());
    let external_value = props.value.clone();
    let on_change = props.on_change.clone();

    use_effect(move || {
        let incoming = external_value.clone();
        if editor_value.read().as_str() != incoming.as_str() {
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
                    WorkspaceNoteEditorToolbar {}
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
                        placeholder: "第一行写 # 标题，下面继续编辑正文。",
                        editor_aria_label: "笔记 Markdown 编辑器",
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
fn WorkspaceNoteEditorToolbar() -> Element {
    let handle: MarkdownHandle = use_markdown_handle();

    rsx! {
        ToolbarButton {
            title: "标题",
            label: "H",
            onclick: move |_| {
                spawn(async move {
                    handle.insert_text("## ").await;
                });
            }
        }
        ToolbarButton {
            title: "加粗",
            label: "B",
            onclick: move |_| {
                spawn(async move {
                    handle.wrap_selection("**", "**").await;
                });
            }
        }
        ToolbarButton {
            title: "斜体",
            label: "I",
            onclick: move |_| {
                spawn(async move {
                    handle.wrap_selection("_", "_").await;
                });
            }
        }
        ToolbarButton {
            title: "链接",
            label: "↗",
            onclick: move |_| {
                spawn(async move {
                    handle.wrap_selection("[", "](url)").await;
                });
            }
        }
        ToolbarButton {
            title: "列表",
            label: "•",
            onclick: move |_| {
                spawn(async move {
                    handle.insert_text("- ").await;
                });
            }
        }
        ToolbarButton {
            title: "任务",
            label: "☑",
            onclick: move |_| {
                spawn(async move {
                    handle.insert_text("- [ ] ").await;
                });
            }
        }
        ToolbarButton {
            title: "引用",
            label: "❝",
            onclick: move |_| {
                spawn(async move {
                    handle.insert_text("> ").await;
                });
            }
        }
        ToolbarButton {
            title: "代码",
            label: "</>",
            onclick: move |_| {
                spawn(async move {
                    handle.wrap_selection("```text\n", "\n```").await;
                });
            }
        }
    }
}

#[derive(Props, Clone, PartialEq)]
struct ToolbarButtonProps {
    title: &'static str,
    label: &'static str,
    onclick: EventHandler<MouseEvent>,
}

#[component]
fn ToolbarButton(props: ToolbarButtonProps) -> Element {
    let onclick = props.onclick.clone();
    rsx! {
        markdown::ToolbarButton {
            class: "note-rich-editor__button".to_string(),
            title: props.title.to_string(),
            aria_label: props.title.to_string(),
            onclick: move |event| onclick.call(event),
            "{props.label}"
        }
    }
}

fn derive_markdown_title(body: &str) -> String {
    if let Some(heading) = body
        .lines()
        .map(str::trim)
        .find(|line| line.starts_with('#') && line.trim_start_matches('#').trim().len() > 0)
    {
        return heading.trim_start_matches('#').trim().to_string();
    }

    if let Some(line) = body.lines().map(str::trim).find(|line| !line.is_empty()) {
        return line
            .trim_start_matches(['#', '-', '*', ' '])
            .chars()
            .take(48)
            .collect::<String>();
    }

    "新笔记".to_string()
}

fn extract_headings_from_body(body: &str) -> Vec<String> {
    body.lines()
        .map(str::trim)
        .filter(|line| line.starts_with('#'))
        .filter_map(|line| {
            let heading = line.trim_start_matches('#').trim();
            (!heading.is_empty()).then(|| heading.to_string())
        })
        .take(8)
        .collect()
}

fn strip_markdown_fences(content: &str) -> String {
    let trimmed = content.trim();
    let Some(rest) = trimmed.strip_prefix("```") else {
        return trimmed.to_string();
    };
    let rest = rest
        .split_once('\n')
        .map(|(_, body)| body)
        .unwrap_or(rest)
        .trim_end();
    rest.strip_suffix("```")
        .map(str::trim)
        .unwrap_or(rest)
        .to_string()
}

addzero_admin_plugin_registry::register_admin_page! {
    id: KNOWLEDGE_NOTES_PAGE_ID,
    domain: KNOWLEDGE_DOMAIN_ID,
    parent: None,
    label: "笔记",
    order: 10,
    href: "/knowledge/notes",
    active_patterns: &["/knowledge/notes"],
    permissions_any_of: &["knowledge:note"],
}
