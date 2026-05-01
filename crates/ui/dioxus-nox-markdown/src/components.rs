use std::cell::RefCell;
use std::rc::Rc;

use dioxus::prelude::*;

use crate::context::{
    CursorContext, MarkdownContext, handle_set_content_js, handle_set_content_with_cursor_js,
    make_instance_n, read_cursor_and_selection,
};
use crate::hooks::{
    select_all_children_js, sync_editor_to_preview, sync_preview_to_editor, tab_indent_js,
    wrap_selection_js,
};
use crate::inline_editor::InlineEditor;
use crate::interop;
use crate::types::{
    ActiveBlockInputEvent, CursorPosition, HtmlRenderPolicy, Layout, LivePreviewVariant, Mode,
    Orientation, Selection, VimAction, VimState,
};

// ── Internal hook: use_root_state ─────────────────────────────────────

use crop::Rope;

/// Internal state bundle produced by `use_root_state`.
struct RootState {
    mode: Signal<Mode>,
    raw_content: Signal<Rc<RefCell<Rope>>>,
    parsed_doc: Memo<Rc<crate::types::ParsedDoc>>,
    trigger_parse: Callback<()>,
}

// TODO(REF-002): consider unifying the gen+memo parse trigger with use_debounced_parse
// from hooks.rs. Currently use_debounced_parse returns Signal<Rc<ParsedDoc>> (not Memo)
// and uses gloo_timers for debouncing, while Root uses a synchronous gen counter + Memo.
// Unification would require use_debounced_parse to return Memo<Rc<ParsedDoc>> or Root
// to switch to Signal-based parsed state.

/// Configuration for `use_root_state` — bundles parse and render options.
struct RootConfig {
    initial_mode: Option<Mode>,
    controlled_mode: Option<Signal<Mode>>,
    default_value: Option<String>,
    controlled_value: Option<Signal<String>>,
    html_render_policy: HtmlRenderPolicy,
    highlight_class_prefix: String,
    show_code_line_numbers: bool,
    show_code_language: bool,
}

fn use_root_state(cfg: RootConfig) -> RootState {
    // Always allocate the internal mode signal unconditionally (rules of hooks).
    // If a controlled `mode` prop is provided, we use that instead.
    let internal_mode = use_signal(|| cfg.initial_mode.unwrap_or(Mode::Source));
    let mode = cfg.controlled_mode.unwrap_or(internal_mode);

    // Raw content buffer (hot-path, not a Signal<Rope>)
    let default_value = cfg.default_value;
    let controlled_value = cfg.controlled_value;
    let raw_content_ref: Rc<RefCell<Rope>> = use_hook(|| {
        Rc::new(RefCell::new(Rope::from(
            controlled_value
                .map(|s| s.read().clone())
                .or(default_value)
                .unwrap_or_default(),
        )))
    });
    let raw_content = use_signal(|| raw_content_ref.clone());

    // Parse generation counter — bumped by trigger_parse to force memo recomputation.
    // The memo reads this signal so it subscribes to changes.
    let mut parse_gen = use_signal(|| 0u64);

    // Parsed document memo — recomputes when parse_gen changes.
    let raw_for_memo = raw_content;
    let html_render_policy = cfg.html_render_policy;
    let highlight_class_prefix = cfg.highlight_class_prefix;
    let show_code_line_numbers = cfg.show_code_line_numbers;
    let show_code_language = cfg.show_code_language;
    let parsed_doc: Memo<Rc<crate::types::ParsedDoc>> = use_memo(move || {
        // Read parse_gen to subscribe to trigger_parse updates
        let _gen = (parse_gen)();
        let content_rope = raw_for_memo.read().borrow().clone();
        Rc::new(crate::parser::parse_document_full_with_config(
            &content_rope,
            html_render_policy,
            &highlight_class_prefix,
            show_code_line_numbers,
            show_code_language,
        ))
    });

    // trigger_parse bumps the generation counter, causing parsed_doc to recompute.
    let trigger_parse = Callback::new(move |_: ()| {
        parse_gen += 1;
    });

    RootState {
        mode,
        raw_content,
        parsed_doc,
        trigger_parse,
    }
}

// ── 1. Root ──────────────────────────────────────────────────────────

/// Top-level compound component. Provides `MarkdownContext` and `CursorContext`.
///
/// Supports both uncontrolled (default) and controlled patterns for mode and value.
#[component]
pub fn Root(
    // Mode — uncontrolled (initial_mode) OR controlled (mode + on_mode_change)
    initial_mode: Option<Mode>,
    mode: Option<Signal<Mode>>,
    on_mode_change: Option<EventHandler<Mode>>,
    // Value — uncontrolled (default_value) OR controlled (value + on_value_change)
    default_value: Option<String>,
    value: Option<Signal<String>>,
    on_value_change: Option<EventHandler<String>>,
    /// Optional signal for setting cursor position after a programmatic content change.
    ///
    /// When `Some(signal)` and the inner value is `Some(byte_offset)`, the controlled
    /// value effect will place the cursor at that byte offset after updating content.
    /// The signal is reset to `None` after consumption.
    pending_cursor: Option<Signal<Option<usize>>>,
    #[props(default = false)] disabled: bool,
    /// Layout orientation for split-pane mode.
    /// Sets `data-md-layout` attribute on the root div. Omitted when None.
    layout: Option<Layout>,
    /// Controls `LivePreview` rendering style.
    /// `SplitPane` (default) = side-by-side split pane (backwards-compatible).
    /// `Inline` = Obsidian-style cursor-aware single-surface editing.
    #[props(default)]
    live_preview_variant: LivePreviewVariant,
    /// Controls raw HTML rendering behavior. See [`HtmlRenderPolicy`] for details.
    ///
    /// - `Escape` (default): renders HTML tags as visible text. Safe for all inputs.
    /// - `Sanitized`: strips dangerous HTML while keeping safe formatting (requires `sanitize` feature).
    /// - `Trusted`: renders raw HTML with **no sanitization** — XSS risk with user content.
    #[props(default)]
    html_render_policy: HtmlRenderPolicy,
    /// CSS class prefix for syntax-highlighted code spans.
    ///
    /// Default `"hl-"` produces `<span class="hl-keyword">`. Set to `""` for
    /// unprefixed syntect classes, or a custom prefix for namespace isolation.
    #[props(default = "hl-".to_string())]
    #[props(into)]
    highlight_class_prefix: String,
    /// Show line numbers on rendered code blocks in Preview/Content.
    #[props(default = false)]
    show_code_line_numbers: bool,
    /// Show language label on rendered fenced code blocks.
    #[props(default = true)]
    show_code_language: bool,
    /// Show line number gutter on the source editor textarea.
    #[props(default = false)]
    show_editor_line_numbers: bool,
    class: Option<String>,
    #[props(extends = GlobalAttributes)] additional_attributes: Vec<Attribute>,
    children: Element,
) -> Element {
    let state = use_root_state(RootConfig {
        initial_mode,
        controlled_mode: mode,
        default_value,
        controlled_value: value,
        html_render_policy,
        highlight_class_prefix: highlight_class_prefix.clone(),
        show_code_line_numbers,
        show_code_language,
    });

    // Generate unique per-instance number for DOM IDs.
    let instance_n = use_hook(make_instance_n);

    // Hoist signal allocations outside of use_context_provider closures.
    // Calling use_signal inside a use_context_provider closure is a rules-of-hooks
    // violation (nested use_hook calls → "hook list already borrowed" panic).
    let is_editor_scrolling = use_signal(|| false);
    let is_preview_scrolling = use_signal(|| false);
    let cursor_position = use_signal(CursorPosition::default);
    let selection = use_signal(|| None::<Selection>);
    let preedit = use_signal(|| None::<String>);
    let editor_mount: Signal<Option<Rc<MountedData>>> = use_signal(|| None);
    let live_preview_variant_sig = use_signal(|| live_preview_variant);
    let highlight_prefix_sig = use_signal(|| highlight_class_prefix);

    use_context_provider(|| MarkdownContext {
        mode: state.mode,
        is_mode_controlled: mode.is_some(),
        on_mode_change,
        raw_content: state.raw_content,
        is_value_controlled: value.is_some(),
        on_value_change,
        parsed_doc: state.parsed_doc,
        is_editor_scrolling,
        is_preview_scrolling,
        instance_n,
        editor_mount,
        disabled,
        trigger_parse: state.trigger_parse,
        live_preview_variant: live_preview_variant_sig,
        highlight_class_prefix: highlight_prefix_sig,
        show_code_line_numbers,
        show_code_language,
        show_editor_line_numbers,
    });

    use_context_provider(|| CursorContext {
        cursor_position,
        selection,
        preedit,
    });

    // Controlled value: when external `value` signal changes, sync textarea via eval().
    // This runs inside use_effect to respect the "eval after mount" rule.
    // Does NOT create a loop because:
    // - use_effect subscribes to `controlled_value` (external signal owned by consumer)
    // - oninput calls ctx.handle_value_change() which writes to raw_content (Rc<RefCell<String>>)
    // - oninput fires on_value_change callback (consumer updates their signal)
    // - The controlled_value signal is NOT the same as raw_content
    // Always call use_effect unconditionally (rules of hooks).
    // The effect is a no-op when value is None.
    let raw_for_effect = state.raw_content;
    let trigger = state.trigger_parse;
    use_effect(move || {
        // Read pending_cursor's inner signal BEFORE the early-return guard
        // so the effect subscribes to it (Dioxus 0.7 subscription gotcha).
        let pending_offset = pending_cursor.and_then(|sig| (sig)());

        if let Some(cv) = value {
            let text = cv();
            // SEC-004: Skip eval if content hasn't actually changed
            let current = raw_for_effect.read().borrow().to_string();
            if current == text {
                return;
            }
            // Synchronously update raw_content and re-parse.
            // Primary sync for Inline/Read modes where no textarea exists.
            *raw_for_effect.read().borrow_mut() = Rope::from(text.clone());
            trigger.call(());

            // Update CursorContext if pending_cursor was provided.
            // cursor_position + selection signals are captured from the component
            // body (lines 181-182) — NOT via try_use_context inside the effect,
            // which would be a rules-of-hooks violation in Dioxus 0.7.
            if let Some(byte_offset) = pending_offset {
                let clamped = byte_offset.min(text.len());
                {
                    let mut cp = cursor_position;
                    cp.set(CursorPosition {
                        offset: clamped,
                        line: 0,
                        column: 0,
                    });
                    let mut sel = selection;
                    sel.set(None);
                }
                // Reset pending_cursor to None
                if let Some(mut pc) = pending_cursor {
                    pc.set(None);
                }
            }

            // Also sync textarea DOM value for Source mode.
            // No-op if textarea is absent (Inline/Read modes).
            let eid = format!("nox-md-{instance_n}-editor");
            spawn(async move {
                let js = if let Some(byte_offset) = pending_offset {
                    handle_set_content_with_cursor_js(&eid, &text, byte_offset)
                } else {
                    handle_set_content_js(&eid, &text)
                };
                interop::eval_void(&js).await;
            });
        }
    });

    let mode_attr = (state.mode)().to_data_attr_value();
    let disabled_attr: Option<&str> = if disabled { Some("") } else { None };
    let layout_attr = layout.map(|l| l.as_attr());

    rsx! {
        div {
            class: class,
            "data-md-root": "",
            "data-md-mode": mode_attr,
            "data-md-layout": layout_attr,
            "data-disabled": disabled_attr,
            ..additional_attributes,
            // aria-live region for mode change announcements
            span {
                aria_live: "polite",
                aria_atomic: "true",
                style: "position:absolute;width:1px;height:1px;overflow:hidden;clip:rect(0,0,0,0)",
                "Mode: {mode_attr}"
            }
            {children}
        }
    }
}

// ── 2. Editor ────────────────────────────────────────────────────────

/// Wrapper around the `<textarea>` editor. Always mounted; visibility toggled via `data-state`.
#[component]
pub fn Editor(
    class: Option<String>,
    #[props(into)] placeholder: Option<String>,
    #[props(default = false)] auto_focus: bool,
    #[props(default = 2)] tab_size: u8,
    #[props(default = true)] spell_check: bool,
    #[props(default = "Markdown editor".to_string())]
    #[props(into)]
    editor_aria_label: String,
    /// Enable vim modal editing (opt-in, defaults to false).
    #[props(default = false)]
    vim: bool,
    /// Called when a "/" slash command trigger is detected at line start.
    /// Argument: byte offset of the "/" character in the editor text.
    on_slash_trigger: Option<EventHandler<usize>>,
    /// Called on each keystroke while a slash command is active.
    /// Argument: the filter text typed after "/" (e.g., "hea" for "/hea").
    on_slash_filter: Option<EventHandler<String>>,
    /// Fires on every `oninput` in inline mode with the active block's raw text + cursor.
    /// Forwarded to [`InlineEditor`] when `LivePreviewVariant::Inline` is active.
    on_active_block_input: Option<EventHandler<ActiveBlockInputEvent>>,
    /// Intercept keydown events before the editor processes them.
    ///
    /// Called with the key name (e.g. `"ArrowDown"`). Return `true` to consume
    /// the event (prevent_default + stop_propagation), `false` to let it through.
    /// Used by suggestion popovers to capture arrow/Enter/Escape navigation.
    on_key_intercept: Option<Callback<String, bool>>,
    #[props(extends = GlobalAttributes)] additional_attributes: Vec<Attribute>,
) -> Element {
    let ctx = use_context::<MarkdownContext>();

    // Fix 12: Hoist try_use_context out of event handler closure (rules of hooks).
    let cursor_ctx = try_use_context::<CursorContext>();

    // Vim state signal — always allocated (rules of hooks); inactive when vim = false.
    let mut vim_state = use_signal(VimState::default);

    // IME composition flag — non-reactive (Rc<RefCell<bool>>)
    // Skip trigger_parse during composition to avoid corrupting CJK input.
    let is_composing: Rc<RefCell<bool>> = use_hook(|| Rc::new(RefCell::new(false)));

    // Focus tracking signal for data-md-editor-focused attribute
    let mut is_focused = use_signal(|| false);

    // Line count signal for editor gutter (always allocated — rules of hooks)
    let mut line_count = use_signal(|| {
        let initial = ctx.raw_value();
        initial.chars().filter(|&c| c == '\n').count() + 1
    });

    // data-state: active when Source or LivePreview, inactive when Read
    let data_state = match (ctx.mode)() {
        Mode::Read => "inactive",
        Mode::Source | Mode::LivePreview => "active",
    };

    let disabled_attr: Option<&str> = if ctx.disabled { Some("") } else { None };
    let focused_attr = if (is_focused)() { "true" } else { "false" };

    // Capture clones for closures
    let composing_for_input = is_composing.clone();
    let composing_for_start = is_composing.clone();
    let composing_for_end = is_composing;

    let source_id = ctx.source_panel_id();
    let editor_id = ctx.editor_id();

    // In LivePreview + Inline mode, delegate to InlineEditor surface.
    // Early return so the complex textarea RSX below stays syntactically clean.
    if (ctx.mode)() == Mode::LivePreview
        && (ctx.live_preview_variant)() == LivePreviewVariant::Inline
    {
        return rsx! {
            div {
                id: "{source_id}",
                class: class,
                "data-md-editor": "",
                "data-state": data_state,
                "data-md-editor-focused": focused_attr,
                "data-md-word-wrap": "true",
                "data-disabled": disabled_attr,
                ..additional_attributes,
                InlineEditor { on_active_block_input, on_key_intercept }
            }
        };
    }

    rsx! {
        div {
            id: "{source_id}",
            class: class,
            "data-md-editor": "",
            "data-state": data_state,
            "data-md-editor-focused": focused_attr,
            "data-md-word-wrap": "true",
            "data-disabled": disabled_attr,
            ..additional_attributes,
            // ── Editor line number gutter ──
            if ctx.show_editor_line_numbers {
                {render_editor_gutter(ctx.gutter_id(), (line_count)())}
            }
            textarea {
                id: "{editor_id}",
                role: "textbox",
                aria_multiline: "true",
                aria_label: editor_aria_label,
                placeholder: placeholder,
                spellcheck: if spell_check { "true" } else { "false" },
                disabled: ctx.disabled,
                initial_value: ctx.raw_value(),

                // ── onkeydown: Tab indent and Ctrl+B/I/K formatting shortcuts ──
                onkeydown: move |evt: KeyboardEvent| {
                    // CRITICAL: ALL prevent_default() calls MUST be SYNCHRONOUS before any spawn
                    let key = evt.key().to_string();

                    // ── Key intercept (suggestion popover) ────────────────────
                    if let Some(ref interceptor) = on_key_intercept
                        && interceptor.call(key.clone())
                    {
                        evt.prevent_default();
                        evt.stop_propagation();
                        return;
                    }

                    let ctrl = evt.modifiers().ctrl();
                    let shift = evt.modifiers().shift();

                    // ── Vim modal handling (opt-in) ───────────────────────────
                    if vim {
                        let eid = ctx.editor_id();
                        let action = vim_state.write().handle_key(&key, ctrl, shift, &eid);
                        match action {
                            VimAction::PassThrough => {} // fall through to regular handling
                            VimAction::PreventAndEval(js) => {
                                evt.prevent_default(); // SYNC
                                spawn(async move { interop::eval_void(&js).await; });
                                return;
                            }
                            VimAction::ModeChange(_) => return, // mode changed, swallow key
                            VimAction::ExecuteCommand(_) => return, // future: handle commands
                        }
                    }

                    // ── Existing keyboard handling (Tab, Ctrl+B/I/K) ─────────

                    if key == "Tab" {
                        evt.prevent_default();
                        let size = tab_size;
                        let eid = ctx.editor_id();
                        spawn(async move {
                            interop::eval_void(&tab_indent_js(&eid, size)).await;
                        });
                        return;
                    }

                    if ctrl {
                        let eid = ctx.editor_id();
                        let maybe_js = match key.as_str() {
                            "b" | "B" => { evt.prevent_default(); Some(wrap_selection_js(&eid, "**", "**")) }
                            "i" | "I" => { evt.prevent_default(); Some(wrap_selection_js(&eid, "_", "_")) }
                            "k" | "K" => { evt.prevent_default(); Some(wrap_selection_js(&eid, "[", "](url)")) }
                            _ => None,
                        };
                        if let Some(js) = maybe_js {
                            spawn(async move { interop::eval_void(&js).await; });
                        }
                    }
                },

                // ── oninput: update raw content, trigger parse if not composing ──
                oninput: move |evt: FormEvent| {
                    let new_value = evt.value();
                    ctx.handle_value_change(new_value.clone());
                    // Update line count for gutter
                    line_count.set(new_value.chars().filter(|&c| c == '\n').count() + 1);
                    // Only trigger parse when not in IME composition
                    if !*composing_for_input.borrow() {
                        ctx.trigger_parse.call(());
                    }

                    // ── Slash command detection ──
                    if on_slash_trigger.is_some() || on_slash_filter.is_some() {
                        let text = new_value.clone();
                        let eid = ctx.editor_id();
                        spawn(async move {
                            // cursor_utf16 is a UTF-16 code-unit index from JS selectionStart
                            let cursor_utf16: usize = {
                                let js = format!(
                                    "dioxus.send(document.getElementById('{eid}')?.selectionStart ?? 0);"
                                );
                                let mut ev = interop::start_eval(&js);
                                match interop::recv_u64(&mut ev).await {
                                    Some(pos) => pos as usize,
                                    None => return,
                                }
                            };
                            if let Some(handler) = on_slash_trigger
                                && let Some(trigger_offset) = detect_slash_trigger(&text, cursor_utf16)
                            {
                                handler.call(trigger_offset);
                            }
                            if let Some(handler) = on_slash_filter
                                && let Some(filter) = extract_slash_filter(&text, cursor_utf16)
                            {
                                handler.call(filter);
                            }
                        });
                    }

                    // ── Update CursorContext with current cursor/selection ──
                    if let Some(mut cursor_ctx) = cursor_ctx {
                        let text_clone = new_value;
                        let eid = ctx.editor_id();
                        spawn(async move {
                            if let Some((pos, sel)) = read_cursor_and_selection(&eid, &text_clone).await {
                                cursor_ctx.cursor_position.set(pos);
                                cursor_ctx.selection.set(sel);
                            }
                        });
                    }
                },

                // ── IME composition events ──
                oncompositionstart: move |_| {
                    *composing_for_start.borrow_mut() = true;
                },
                oncompositionend: move |_| {
                    *composing_for_end.borrow_mut() = false;
                    // Composition complete — trigger parse now
                    ctx.trigger_parse.call(());
                },

                // ── Focus tracking ──
                onfocus: move |_| {
                    is_focused.set(true);
                },
                onblur: move |_| {
                    is_focused.set(false);
                },

                // ── Scroll sync: editor → preview + gutter ──
                onscroll: move |_| {
                    // Sync gutter scroll position to match textarea
                    if ctx.show_editor_line_numbers {
                        let eid = ctx.editor_id();
                        let gid = ctx.gutter_id();
                        spawn(async move {
                            crate::hooks::sync_gutter_scroll(&eid, &gid).await;
                        });
                    }
                    if (ctx.is_preview_scrolling)() { return; }
                    let mut editor_flag = ctx.is_editor_scrolling;
                    let eid = ctx.editor_id();
                    let pid = ctx.preview_id();
                    editor_flag.set(true);
                    spawn(async move {
                        sync_editor_to_preview(&eid, &pid).await;
                        editor_flag.set(false);
                    });
                },

                // ── Store MountedData + auto-focus on mount ──
                onmounted: move |evt: MountedEvent| {
                    let mut mount = ctx.editor_mount;
                    mount.set(Some(evt.data()));
                    if auto_focus {
                        spawn(async move {
                            let _ = evt.data().set_focus(true).await;
                        });
                    }
                },
            }
        }
    }
}

// ── 3. Preview ───────────────────────────────────────────────────────

/// Rendered preview pane. Always mounted; visibility toggled via `data-state`.
#[component]
pub fn Preview(
    class: Option<String>,
    #[props(default = "Markdown preview".to_string())]
    #[props(into)]
    preview_aria_label: String,
    /// Called when a block element with `data-source-start` is clicked.
    /// The argument is the source byte offset (usize) of the clicked block.
    on_block_click: Option<EventHandler<usize>>,
    #[props(extends = GlobalAttributes)] additional_attributes: Vec<Attribute>,
) -> Element {
    let ctx = use_context::<MarkdownContext>();

    // ANT-002: Store the block-click listener task so we can cancel on unmount
    let block_click_task: Rc<RefCell<Option<dioxus_core::Task>>> =
        use_hook(|| Rc::new(RefCell::new(None)));

    // Cancel the block-click eval loop when Preview unmounts
    {
        let task_handle = block_click_task.clone();
        use_drop(move || {
            if let Some(task) = task_handle.borrow_mut().take() {
                task.cancel();
            }
        });
    }

    // data-state: active in LivePreview (SplitPane only); inactive in Inline mode
    let data_state = match (ctx.mode)() {
        Mode::LivePreview if (ctx.live_preview_variant)() != LivePreviewVariant::Inline => "active",
        _ => "inactive",
    };

    let parsed = (ctx.parsed_doc)();
    let preview_id = ctx.preview_id();

    rsx! {
        div {
            id: "{preview_id}",
            class: class,
            role: "region",
            aria_label: preview_aria_label,
            "data-md-preview": "",
            "data-state": data_state,
            "data-md-preview-loading": "false",

            // ── Scroll sync: preview → editor ──
            onscroll: move |_| {
                if (ctx.is_editor_scrolling)() { return; }
                let mut preview_flag = ctx.is_preview_scrolling;
                let eid = ctx.editor_id();
                let pid = ctx.preview_id();
                preview_flag.set(true);
                spawn(async move {
                    sync_preview_to_editor(&eid, &pid).await;
                    preview_flag.set(false);
                });
            },

            // ── Block click delegation via eval() ──
            // Attaches a click listener after mount that walks up the DOM
            // from the clicked element looking for data-source-start, then
            // sends the source offset back via dioxus.send().
            onmounted: {
                let task_handle = block_click_task.clone();
                move |_| {
                    if on_block_click.is_some() {
                        let pid = ctx.preview_id();
                        let task = spawn(async move {
                            let mut ev = interop::start_eval(&block_click_js(&pid));
                            while let Some(line) = interop::recv_u64(&mut ev).await {
                                if let Some(handler) = on_block_click {
                                    handler.call(line as usize);
                                }
                            }
                        });
                        *task_handle.borrow_mut() = Some(task);
                    }
                }
            },
            ..additional_attributes,
            {parsed.element.clone()}
        }
    }
}

// ── 4. Content ───────────────────────────────────────────────────────

/// Read-mode display component. May be conditionally rendered by consumers.
#[component]
pub fn Content(
    class: Option<String>,
    #[props(extends = GlobalAttributes)] additional_attributes: Vec<Attribute>,
) -> Element {
    let ctx = use_context::<MarkdownContext>();
    let parsed = (ctx.parsed_doc)();
    let read_id = ctx.read_panel_id();
    let mut mounted: Signal<Option<Rc<MountedData>>> = use_signal(|| None);

    // Auto-focus the Content div when entering Read mode so Ctrl+A is scoped.
    use_effect(move || {
        let mode = (ctx.mode)();
        if mode == Mode::Read
            && let Some(node) = mounted.read().as_ref()
        {
            let node = node.clone();
            spawn(async move {
                let _ = node.set_focus(true).await;
            });
        }
    });

    rsx! {
        div {
            id: "{read_id}",
            class: class,
            role: "article",
            tabindex: "-1",
            "data-md-mode": "read",
            onmounted: move |evt: MountedEvent| {
                mounted.set(Some(evt.data()));
            },
            onkeydown: move |evt: KeyboardEvent| {
                let key = evt.key().to_string();
                let ctrl_or_meta = evt.modifiers().ctrl() || evt.modifiers().meta();
                if ctrl_or_meta && (key == "a" || key == "A") {
                    evt.prevent_default();
                    let rid = ctx.read_panel_id();
                    spawn(async move {
                        interop::eval_void(&select_all_children_js(&rid)).await;
                    });
                }
            },
            ..additional_attributes,
            {parsed.element.clone()}
        }
    }
}

// ── 5. Toolbar ───────────────────────────────────────────────────────

/// Consumer-composed toolbar container.
#[component]
pub fn Toolbar(
    class: Option<String>,
    #[props(extends = GlobalAttributes)] additional_attributes: Vec<Attribute>,
    children: Element,
) -> Element {
    let ctx = use_context::<MarkdownContext>();
    let disabled_attr: Option<&str> = if ctx.disabled { Some("") } else { None };

    rsx! {
        div {
            class: class,
            role: "toolbar",
            aria_orientation: "horizontal",
            "data-disabled": disabled_attr,
            ..additional_attributes,
            {children}
        }
    }
}

// ── 6. ToolbarButton ─────────────────────────────────────────────────

/// Individual toolbar button. When `as_child` is true, renders as a `<span>` with
/// `role="button"` for use with custom trigger components.
#[component]
pub fn ToolbarButton(
    class: Option<String>,
    #[props(default = false)] disabled: bool,
    #[props(default = false)] as_child: bool,
    #[props(extends = GlobalAttributes)] additional_attributes: Vec<Attribute>,
    onclick: Option<EventHandler<MouseEvent>>,
    /// Activation callback for keyboard (Enter/Space) when `as_child` is true.
    on_activate: Option<EventHandler<()>>,
    children: Element,
) -> Element {
    let disabled_attr: Option<&str> = if disabled { Some("") } else { None };
    let click_handler = move |e: MouseEvent| {
        if let Some(handler) = &onclick {
            handler.call(e);
        }
    };

    if as_child {
        rsx! {
            span {
                role: "button",
                tabindex: "0",
                aria_disabled: if disabled { "true" } else { "false" },
                class: class,
                "data-disabled": disabled_attr,
                onclick: click_handler,
                onkeydown: move |evt: KeyboardEvent| {
                    let key = evt.key().to_string();
                    if !disabled && (key == "Enter" || key == " ") {
                        evt.prevent_default();
                        if let Some(handler) = on_activate {
                            handler.call(());
                        }
                    }
                },
                ..additional_attributes,
                {children}
            }
        }
    } else {
        rsx! {
            button {
                class: class,
                r#type: "button",
                disabled: disabled,
                "data-disabled": disabled_attr,
                onclick: click_handler,
                ..additional_attributes,
                {children}
            }
        }
    }
}

// ── 7. ToolbarSeparator ──────────────────────────────────────────────

/// Visual separator in toolbar.
#[component]
pub fn ToolbarSeparator(
    class: Option<String>,
    #[props(extends = GlobalAttributes)] additional_attributes: Vec<Attribute>,
) -> Element {
    rsx! {
        div {
            class: class,
            role: "separator",
            "data-orientation": "vertical",
            ..additional_attributes,
        }
    }
}

// ── 8. ModeBar ───────────────────────────────────────────────────────

/// Mode tab strip container.
#[component]
pub fn ModeBar(
    class: Option<String>,
    #[props(extends = GlobalAttributes)] additional_attributes: Vec<Attribute>,
    children: Element,
) -> Element {
    let ctx = use_context::<MarkdownContext>();
    let mode_attr = (ctx.mode)().to_data_attr_value();

    rsx! {
        div {
            class: class,
            role: "tablist",
            "data-md-mode": mode_attr,
            ..additional_attributes,
            {children}
        }
    }
}

// ── 9. ModeTab ───────────────────────────────────────────────────────

/// Returns the next mode in the cycle: Read → Source → LivePreview → Read.
pub(crate) fn next_mode(m: Mode) -> Mode {
    match m {
        Mode::Read => Mode::Source,
        Mode::Source => Mode::LivePreview,
        Mode::LivePreview => Mode::Read,
    }
}

/// Returns the previous mode in the cycle: Read → LivePreview → Source → Read.
pub(crate) fn prev_mode(m: Mode) -> Mode {
    match m {
        Mode::Read => Mode::LivePreview,
        Mode::Source => Mode::Read,
        Mode::LivePreview => Mode::Source,
    }
}

/// Returns the panel ID for a given mode, using per-instance IDs.
pub(crate) fn panel_id_for_mode(mode: Mode, ctx: &MarkdownContext) -> String {
    match mode {
        Mode::Source => ctx.source_panel_id(),
        Mode::LivePreview => ctx.preview_id(),
        Mode::Read => ctx.read_panel_id(),
    }
}

/// Individual mode tab. Clicking activates the associated mode.
///
/// Supports WAI-ARIA keyboard navigation: Arrow keys cycle through modes,
/// Home/End jump to first/last mode.
#[component]
pub fn ModeTab(
    mode: Mode,
    class: Option<String>,
    #[props(extends = GlobalAttributes)] additional_attributes: Vec<Attribute>,
    children: Element,
) -> Element {
    let mut ctx = use_context::<MarkdownContext>();
    let current_mode = (ctx.mode)();
    let is_active = current_mode == mode;
    let data_state = if is_active { "active" } else { "inactive" };
    let aria_selected = if is_active { "true" } else { "false" };
    let mode_attr = mode.to_data_attr_value();
    let panel_id = panel_id_for_mode(mode, &ctx);

    rsx! {
        button {
            class: class,
            role: "tab",
            tabindex: if is_active { "0" } else { "-1" },
            aria_selected: aria_selected,
            aria_controls: "{panel_id}",
            "data-state": data_state,
            "data-mode": mode_attr,
            onclick: move |_| {
                ctx.handle_mode_change(mode);
            },
            onkeydown: move |evt: KeyboardEvent| {
                let key = evt.key().to_string();
                let new_mode = match key.as_str() {
                    "ArrowRight" | "ArrowDown" => Some(next_mode(current_mode)),
                    "ArrowLeft" | "ArrowUp" => Some(prev_mode(current_mode)),
                    "Home" => Some(Mode::Read),
                    "End" => Some(Mode::LivePreview),
                    _ => None,
                };
                if let Some(m) = new_mode {
                    evt.prevent_default();
                    ctx.handle_mode_change(m);
                }
            },
            ..additional_attributes,
            {children}
        }
    }
}

// ── 10. Divider ──────────────────────────────────────────────────────

/// Visual separator between editor and preview panes.
///
/// Renders nothing in `LivePreviewVariant::Inline` mode (no split pane to separate).
#[component]
pub fn Divider(
    class: Option<String>,
    #[props(default)] orientation: Orientation,
    #[props(extends = GlobalAttributes)] additional_attributes: Vec<Attribute>,
) -> Element {
    let ctx = use_context::<MarkdownContext>();
    if (ctx.mode)() == Mode::LivePreview
        && (ctx.live_preview_variant)() == LivePreviewVariant::Inline
    {
        return rsx! {};
    }

    rsx! {
        div {
            class: class,
            role: "separator",
            "data-md-splitter": "",
            "data-orientation": orientation.as_attr(),
            "data-md-splitter-dragging": "false",
            ..additional_attributes,
        }
    }
}

// ── Editor line number gutter helper ──────────────────────────────────

/// Renders the editor line-number gutter div.
fn render_editor_gutter(gutter_id: String, line_count: usize) -> Element {
    let lines: Vec<usize> = (1..=line_count).collect();
    rsx! {
        div {
            id: "{gutter_id}",
            "data-md-line-gutter": "",
            "data-md-editor-gutter": "",
            aria_hidden: "true",
            style: "user-select:none;overflow:hidden",
            for i in lines {
                div { "data-md-line-number": "{i}", "{i}" }
            }
        }
    }
}

// ── UTF-16 ↔ byte-index conversion ──────────────────────────────────

/// Convert a UTF-16 code-unit index (e.g. JavaScript `selectionStart`) to a
/// Rust `str` byte offset. Returns `None` if `utf16_idx` falls outside or in
/// the middle of a character boundary.
pub(crate) fn utf16_to_byte_index(s: &str, utf16_idx: usize) -> Option<usize> {
    let mut utf16_count = 0usize;
    for (byte_idx, ch) in s.char_indices() {
        if utf16_count == utf16_idx {
            return Some(byte_idx);
        }
        utf16_count += ch.len_utf16();
    }
    // Cursor may sit right after the last character (end-of-string).
    if utf16_count == utf16_idx {
        Some(s.len())
    } else {
        None
    }
}

// ── Slash command detection helpers ──────────────────────────────────

/// Generates JS that attaches a click listener to the preview element.
///
/// When a click occurs, the listener walks up the DOM from the clicked element,
/// looking for `data-source-start`. If found, it sends the integer source offset
/// to Dioxus via `dioxus.send()`. The eval() loop in Rust reads these values.
pub(crate) fn block_click_js(preview_id: &str) -> String {
    format!(
        r#"(function() {{
    var el = document.getElementById('{preview_id}');
    if (!el) return;
    el.addEventListener('click', function(e) {{
        var target = e.target;
        while (target && target !== el) {{
            var line = target.getAttribute('data-source-start');
            if (line !== null) {{
                dioxus.send(parseInt(line, 10));
                return;
            }}
            target = target.parentNode;
        }}
    }});
}})();"#,
        preview_id = preview_id,
    )
}

/// Detect if a slash command is triggered at the cursor position.
///
/// A slash trigger occurs when "/" is at the start of a line (position 0 or
/// immediately after a newline), and the cursor is positioned after it.
///
/// `cursor_utf16` is a UTF-16 code-unit index (from JS `selectionStart`).
///
/// Returns `Some(slash_offset)` — the byte offset of the "/" character — if
/// triggered, `None` otherwise.
pub(crate) fn detect_slash_trigger(text: &str, cursor_utf16: usize) -> Option<usize> {
    if cursor_utf16 == 0 {
        return None;
    }
    let cursor = utf16_to_byte_index(text, cursor_utf16)?;
    let before = &text[..cursor];
    // Find the start of the current line
    let line_start = before.rfind('\n').map(|i| i + 1).unwrap_or(0);
    let line_content = &before[line_start..];
    if line_content.starts_with('/') {
        Some(line_start)
    } else {
        None
    }
}

/// Extract the filter text typed after a "/" slash command trigger.
///
/// `cursor_utf16` is a UTF-16 code-unit index (from JS `selectionStart`).
///
/// Returns `Some(filter)` where filter is the text between "/" and the cursor
/// (e.g., "hea" for "/hea"). Returns `None` if no slash trigger is active or
/// if the filter contains spaces or newlines (which end the slash command).
pub(crate) fn extract_slash_filter(text: &str, cursor_utf16: usize) -> Option<String> {
    let slash_offset = detect_slash_trigger(text, cursor_utf16)?;
    let cursor = utf16_to_byte_index(text, cursor_utf16)?;
    let filter = &text[slash_offset + 1..cursor];
    if filter.contains(' ') || filter.contains('\n') {
        None
    } else {
        Some(filter.to_string())
    }
}
