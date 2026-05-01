use std::cell::RefCell;
use std::rc::Rc;
use std::sync::atomic::{AtomicU64, Ordering};

use dioxus::prelude::*;

use crate::interop;
use crate::types::{CursorPosition, LivePreviewVariant, Mode, ParsedDoc, Selection};
use crop::Rope;

static NEXT_ROOT_ID: AtomicU64 = AtomicU64::new(1);

/// Returns a unique per-instance number for generating DOM IDs.
pub(crate) fn make_instance_n() -> u64 {
    NEXT_ROOT_ID.fetch_add(1, Ordering::Relaxed)
}

/// Shared context for the markdown compound component.
///
/// Provided by `markdown::Root`. Consumed by Editor, Preview, Content,
/// Toolbar, ModeBar and their sub-parts.
///
/// Provider logic: `current_mode()`, `handle_mode_change()`,
/// `handle_value_change()`, `raw_value()`.
#[derive(Clone, Copy)]
pub struct MarkdownContext {
    // ── Mode state ──────────────────────────────────────────────────
    /// Current display mode.
    pub mode: Signal<Mode>,
    /// Whether mode is externally controlled.
    pub is_mode_controlled: bool,
    /// Callback fired when mode changes (controlled pattern).
    pub on_mode_change: Option<EventHandler<Mode>>,

    // ── Value / content state ───────────────────────────────────────
    /// The uncontrolled hot-path buffer for raw editor content.
    /// NOT a reactive Signal for the Rope itself — avoids textarea cursor
    /// reset on every re-render. Editor writes to the inner RefCell on
    /// every keystroke; debounce reads from it.
    pub raw_content: Signal<Rc<RefCell<Rope>>>,
    /// Whether value is externally controlled.
    pub is_value_controlled: bool,
    /// Callback fired when value changes (controlled pattern).
    pub on_value_change: Option<EventHandler<String>>,

    // ── Parsed output ───────────────────────────────────────────────
    /// Parsed document memo — derived from debounced parse trigger.
    /// Wrapped in Rc because ParsedDoc is not Clone (Element is not Clone).
    pub parsed_doc: Memo<Rc<ParsedDoc>>,

    // ── Scroll sync state ──────────────────────────────────────────
    /// Scroll lock: true while editor is driving a scroll sync (prevents feedback loop).
    pub is_editor_scrolling: Signal<bool>,
    /// Scroll lock: true while preview is driving a scroll sync.
    pub is_preview_scrolling: Signal<bool>,

    // ── Instance IDs ────────────────────────────────────────────────
    /// Unique instance number for generating DOM IDs.
    pub instance_n: u64,

    // ── Mounted element refs ─────────────────────────────────────────
    /// MountedData for the editor textarea (set by Editor's onmounted).
    pub editor_mount: Signal<Option<Rc<MountedData>>>,

    // ── Component state ─────────────────────────────────────────────
    /// Whether the component is disabled.
    pub disabled: bool,

    /// Trigger for the debounced parse pipeline.
    /// Editor calls this on every oninput event.
    pub trigger_parse: Callback<()>,

    // ── Inline editor variant ────────────────────────────────────────
    /// Controls `LivePreview` rendering style.
    /// `SplitPane` (default) = existing split-pane behaviour.
    /// `Inline` = Obsidian-style cursor-aware block switching.
    pub live_preview_variant: Signal<LivePreviewVariant>,

    /// CSS class prefix for syntax-highlighted code spans (e.g. `"hl-"` → `class="hl-keyword"`).
    pub highlight_class_prefix: Signal<String>,

    // ── Code block display config ─────────────────────────────────────
    /// Show line numbers on rendered code blocks in Preview/Content.
    pub show_code_line_numbers: bool,
    /// Show language label on rendered code blocks (fenced only).
    pub show_code_language: bool,
    /// Show line number gutter on source editor textarea.
    pub show_editor_line_numbers: bool,
}

impl MarkdownContext {
    /// Returns the current mode value.
    pub fn current_mode(&self) -> Mode {
        *self.mode.read()
    }

    /// Handles a mode change request (from ModeTab click or external).
    ///
    /// If mode is externally controlled, fires `on_mode_change` callback.
    /// Otherwise updates the internal mode signal directly.
    pub fn handle_mode_change(&mut self, mode: Mode) {
        if self.is_mode_controlled {
            if let Some(handler) = &self.on_mode_change {
                handler.call(mode);
            }
        } else {
            let mut mode_signal = self.mode;
            mode_signal.set(mode);
        }
    }

    /// Handles a value change (from Editor oninput).
    ///
    /// Updates the raw_content buffer and fires `on_value_change` callback
    /// if one is provided (controlled pattern).
    pub fn handle_value_change(&self, value: String) {
        if let Some(handler) = &self.on_value_change {
            handler.call(value.clone());
        }
        *self.raw_content.read().borrow_mut() = Rope::from(value);
    }

    /// Returns a clone of the current raw markdown value as a String.
    pub fn raw_value(&self) -> String {
        self.raw_content.read().borrow().to_string()
    }

    /// Returns the DOM ID for the editor textarea.
    pub fn editor_id(&self) -> String {
        format!("nox-md-{}-editor", self.instance_n)
    }

    /// Returns the DOM ID for the preview container.
    pub fn preview_id(&self) -> String {
        format!("nox-md-{}-preview", self.instance_n)
    }

    /// Returns the DOM ID for the source panel.
    pub fn source_panel_id(&self) -> String {
        format!("nox-md-{}-source", self.instance_n)
    }

    /// Returns the DOM ID for the read panel.
    pub fn read_panel_id(&self) -> String {
        format!("nox-md-{}-read", self.instance_n)
    }

    /// Returns the DOM ID for the inline editor `<div contenteditable>`.
    pub fn inline_editor_id(&self) -> String {
        format!("nox-md-{}-inline", self.instance_n)
    }

    /// Returns the DOM ID for the editor line-number gutter.
    pub fn gutter_id(&self) -> String {
        format!("nox-md-{}-gutter", self.instance_n)
    }
}

/// Cursor and selection context, written only by Editor.
///
/// Separated from `MarkdownContext` so that cursor movement
/// does not trigger re-renders of Preview or Content.
#[derive(Clone, Copy)]
pub struct CursorContext {
    /// Current cursor position in the editor.
    pub cursor_position: Signal<CursorPosition>,
    /// Current text selection, if any.
    pub selection: Signal<Option<Selection>>,
    /// Current unsanitized IME Composition (Preedit) overlay text from mobile/native input.
    pub preedit: Signal<Option<String>>,
}

/// Convenience hook to consume `MarkdownContext` from a descendant of `markdown::Root`.
pub fn use_markdown_context() -> MarkdownContext {
    use_context::<MarkdownContext>()
}

/// Hook to optionally consume the cursor context.
/// Returns `None` when used outside a `markdown::Root`.
pub fn use_cursor_context() -> Option<CursorContext> {
    try_use_context::<CursorContext>()
}

// ── Cursor/selection reading via eval ────────────────────────────────

/// Reads cursor position and selection from the editor textarea via eval.
/// Returns `(CursorPosition, Option<Selection>)` with byte offsets into the text.
pub(crate) async fn read_cursor_and_selection(
    editor_id: &str,
    text: &str,
) -> Option<(CursorPosition, Option<Selection>)> {
    let js = interop::caret_adapter().read_textarea_selection_js(editor_id);
    let mut eval = interop::start_eval(&js);
    let arr = interop::recv_vec_u64(&mut eval).await?;
    let start_utf16 = *arr.first()? as usize;
    let end_utf16 = *arr.get(1)? as usize;

    // Convert UTF-16 code-unit indices to byte offsets
    let start_byte = utf16_to_byte_index_ctx(text, start_utf16).unwrap_or(0);
    let end_byte = utf16_to_byte_index_ctx(text, end_utf16).unwrap_or(start_byte);

    let pos = CursorPosition {
        offset: start_byte,
        line: 0, // line/column left as 0 — exact values require full text scan
        column: 0,
    };
    let sel = if start_byte == end_byte {
        None
    } else {
        Some(Selection {
            anchor: start_byte,
            head: end_byte,
        })
    };
    Some((pos, sel))
}

/// UTF-16 to byte index conversion for use in context.rs.
fn utf16_to_byte_index_ctx(s: &str, utf16_idx: usize) -> Option<usize> {
    let mut utf16_count = 0usize;
    for (byte_idx, ch) in s.char_indices() {
        if utf16_count == utf16_idx {
            return Some(byte_idx);
        }
        utf16_count += ch.len_utf16();
    }
    if utf16_count == utf16_idx {
        Some(s.len())
    } else {
        None
    }
}

// ── JS helper functions for MarkdownHandle ──────────────────────────

/// Escapes a string for safe embedding inside a JS single-quoted string literal.
pub(crate) fn escape_js(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for ch in s.chars() {
        match ch {
            '\\' => out.push_str("\\\\"),
            '\'' => out.push_str("\\'"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\u{2028}' => out.push_str("\\u2028"),
            '\u{2029}' => out.push_str("\\u2029"),
            _ => out.push(ch),
        }
    }
    out
}

/// Generates JS that inserts `text` at the current cursor position in the textarea,
/// then dispatches a synthetic `input` event to keep Rust state in sync.
pub(crate) fn handle_insert_text_js(editor_id: &str, text: &str) -> String {
    let text_escaped = escape_js(text);
    let text_utf16_len: usize = text
        .chars()
        .map(|c| if (c as u32) > 0xFFFF { 2 } else { 1 })
        .sum();
    format!(
        r#"(function() {{
    var el = document.getElementById('{editor_id}');
    if (!el) return null;
    var start = el.selectionStart;
    var end = el.selectionEnd;
    el.value = el.value.substring(0, start) + '{text}' + el.value.substring(end);
    el.setSelectionRange(start + {text_len}, start + {text_len});
    el.dispatchEvent(new Event('input', {{ bubbles: true }}));
    return null;
}})();"#,
        editor_id = editor_id,
        text = text_escaped,
        text_len = text_utf16_len,
    )
}

/// Generates JS that wraps the current textarea selection with `prefix` and `suffix`,
/// then dispatches a synthetic `input` event to keep Rust state in sync.
pub(crate) fn handle_wrap_selection_js(editor_id: &str, prefix: &str, suffix: &str) -> String {
    let prefix_escaped = escape_js(prefix);
    let suffix_escaped = escape_js(suffix);
    format!(
        r#"(function() {{
    var el = document.getElementById('{editor_id}');
    if (!el) return null;
    var start = el.selectionStart;
    var end = el.selectionEnd;
    var selected = el.value.substring(start, end);
    el.value = el.value.substring(0, start) + '{prefix}' + selected + '{suffix}' + el.value.substring(end);
    el.setSelectionRange(start + {prefix_len}, end + {prefix_len});
    el.dispatchEvent(new Event('input', {{ bubbles: true }}));
    return null;
}})();"#,
        editor_id = editor_id,
        prefix = prefix_escaped,
        suffix = suffix_escaped,
        prefix_len = prefix.len(),
    )
}

/// Generates JS that sets the textarea value to `text` and dispatches
/// a synthetic `input` event to keep Rust state in sync.
pub(crate) fn handle_set_content_js(editor_id: &str, text: &str) -> String {
    let text_escaped = escape_js(text);
    format!(
        r#"(function() {{
    var el = document.getElementById('{editor_id}');
    if (!el) return null;
    el.value = '{text}';
    el.dispatchEvent(new Event('input', {{ bubbles: true }}));
    return null;
}})();"#,
        editor_id = editor_id,
        text = text_escaped,
    )
}

/// Generates JS that sets the textarea value to `text`, positions the cursor
/// at the given byte offset (converted to UTF-16), and dispatches a synthetic
/// `input` event.
pub(crate) fn handle_set_content_with_cursor_js(
    editor_id: &str,
    text: &str,
    cursor_byte_offset: usize,
) -> String {
    let text_escaped = escape_js(text);
    // Convert byte offset to UTF-16 code-unit offset for JS selectionStart/End
    let clamped = cursor_byte_offset.min(text.len());
    let cursor_utf16: usize = text[..clamped].encode_utf16().count();
    format!(
        r#"(function() {{
    var el = document.getElementById('{editor_id}');
    if (!el) return null;
    el.value = '{text}';
    el.setSelectionRange({cursor}, {cursor});
    el.dispatchEvent(new Event('input', {{ bubbles: true }}));
    el.focus();
    return null;
}})();"#,
        editor_id = editor_id,
        text = text_escaped,
        cursor = cursor_utf16,
    )
}

// ── MarkdownHandle imperative API ───────────────────────────────────

/// Imperative handle for programmatic control of the markdown editor.
///
/// All async methods use `document::eval()` to interact with the DOM.
/// Safe on all Dioxus targets (Web, Desktop via Wry, iOS, Android).
/// Focus/blur uses `MountedData::set_focus()` when available.
///
/// Obtain via `use_markdown_handle()` inside a descendant of `markdown::Root`.
///
/// ```rust,ignore
/// let handle = use_markdown_handle();
/// spawn(async move {
///     handle.insert_text("**bold**").await;
///     handle.focus().await;
/// });
/// ```
#[derive(Clone, Copy)]
pub struct MarkdownHandle {
    instance_n: u64,
    editor_mount: Signal<Option<Rc<MountedData>>>,
}

impl MarkdownHandle {
    fn editor_id(&self) -> String {
        format!("nox-md-{}-editor", self.instance_n)
    }

    /// Inserts `text` at the current cursor position, replacing any selection.
    /// Dispatches a synthetic `input` event to sync Rust state.
    pub async fn insert_text(&self, text: &str) {
        interop::eval_void(&handle_insert_text_js(&self.editor_id(), text)).await;
    }

    /// Wraps the current selection with `prefix` and `suffix`.
    /// If no text is selected, inserts `prefix` + `suffix` at cursor.
    /// Dispatches a synthetic `input` event to sync Rust state.
    pub async fn wrap_selection(&self, prefix: &str, suffix: &str) {
        interop::eval_void(&handle_wrap_selection_js(&self.editor_id(), prefix, suffix)).await;
    }

    /// Focuses the editor textarea via `MountedData::set_focus(true)`.
    pub async fn focus(&self) {
        if let Some(node) = self.editor_mount.read().as_ref() {
            let _ = node.set_focus(true).await;
        }
    }

    /// Blurs (unfocuses) the editor textarea via `MountedData::set_focus(false)`.
    pub async fn blur(&self) {
        if let Some(node) = self.editor_mount.read().as_ref() {
            let _ = node.set_focus(false).await;
        }
    }

    /// Replaces the entire editor content with `text`.
    /// Dispatches a synthetic `input` event to sync Rust state.
    pub async fn set_content(&self, text: &str) {
        interop::eval_void(&handle_set_content_js(&self.editor_id(), text)).await;
    }
}

/// Hook to get a `MarkdownHandle` for programmatic editor control.
///
/// Must be called inside a descendant of `markdown::Root`.
/// The returned handle can be used inside `spawn(async { ... })` or
/// event handlers to programmatically interact with the editor textarea.
pub fn use_markdown_handle() -> MarkdownHandle {
    let ctx = use_context::<MarkdownContext>();
    MarkdownHandle {
        instance_n: ctx.instance_n,
        editor_mount: ctx.editor_mount,
    }
}
