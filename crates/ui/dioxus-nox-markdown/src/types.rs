use std::fmt;
use std::ops::Range;
/// The three display modes of the markdown component.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Mode {
    /// Rendered HTML output only (read-only display).
    Read,
    /// Raw markdown textarea editor.
    #[default]
    Source,
    /// Split pane: editor + rendered preview.
    LivePreview,
}

impl Mode {
    /// Returns the string used for `data-md-mode` attribute values.
    /// Uses kebab-case: "read", "source", "live-preview".
    pub fn to_data_attr_value(&self) -> &'static str {
        match self {
            Mode::Read => "read",
            Mode::Source => "source",
            Mode::LivePreview => "live-preview",
        }
    }
}

impl fmt::Display for Mode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.to_data_attr_value())
    }
}

/// Layout direction for the LivePreview split pane.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Layout {
    /// Editor on left, preview on right (default).
    #[default]
    Horizontal,
    /// Editor on top, preview on bottom.
    Vertical,
}

impl Layout {
    /// Returns the `data-md-layout` attribute value for this layout.
    pub fn as_attr(self) -> &'static str {
        match self {
            Layout::Horizontal => "horizontal",
            Layout::Vertical => "vertical",
        }
    }
}

/// Orientation for toolbar and separator components.
/// Structurally identical to [`Layout`] — uses the same horizontal/vertical variants.
pub type Orientation = Layout;

/// Cursor position within the editor textarea.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct CursorPosition {
    /// 0-based line number.
    pub line: u32,
    /// 0-based column number.
    pub column: u32,
    /// Byte offset into the value string.
    pub offset: usize,
}

impl CursorPosition {
    pub fn new(line: u32, column: u32, offset: usize) -> Self {
        Self {
            line,
            column,
            offset,
        }
    }
}

/// A text selection range in the editor.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Selection {
    /// Byte offset where the selection started.
    pub anchor: usize,
    /// Byte offset of the cursor end (may differ from anchor).
    pub head: usize,
}

impl Selection {
    pub fn new(anchor: usize, head: usize) -> Self {
        Self { anchor, head }
    }

    /// Returns `true` when anchor equals head (no text selected).
    pub fn is_collapsed(&self) -> bool {
        self.anchor == self.head
    }

    /// Returns the absolute length of the selection in bytes.
    pub fn len(&self) -> usize {
        self.anchor.abs_diff(self.head)
    }

    /// Returns `true` when the selection has zero length (same as `is_collapsed`).
    pub fn is_empty(&self) -> bool {
        self.anchor == self.head
    }

    /// Returns `true` if the selection direction is forward (anchor <= head).
    pub fn is_forward(&self) -> bool {
        self.anchor <= self.head
    }

    /// Returns `(start, end)` with start <= end regardless of selection direction.
    pub fn ordered(&self) -> (usize, usize) {
        if self.anchor <= self.head {
            (self.anchor, self.head)
        } else {
            (self.head, self.anchor)
        }
    }
}

/// Parser pipeline state, reflected via `data-md-parse-state`.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ParseState {
    /// No parse has been triggered yet.
    #[default]
    Idle,
    /// A parse is currently in progress.
    Parsing,
    /// Parse completed successfully.
    Done,
    /// Parse encountered an error.
    Error,
}

impl ParseState {
    /// Returns the string used for `data-md-parse-state` attribute values.
    pub fn to_data_attr_value(&self) -> &'static str {
        match self {
            ParseState::Idle => "idle",
            ParseState::Parsing => "parsing",
            ParseState::Done => "done",
            ParseState::Error => "error",
        }
    }
}

impl fmt::Display for ParseState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.to_data_attr_value())
    }
}

/// Configuration options for the markdown parse pipeline.
#[derive(Debug, Clone, PartialEq)]
pub struct ParseOptions {
    /// Debounce delay in milliseconds before triggering a re-parse.
    pub debounce_ms: u64,
    /// Tab size for indentation in the editor.
    pub tab_size: u8,
    /// Enable GFM tables.
    pub tables: bool,
    /// Enable GFM task lists.
    pub task_lists: bool,
    /// Enable GFM strikethrough.
    pub strikethrough: bool,
    /// Enable footnotes.
    pub footnotes: bool,
    /// Enable front matter parsing with the given delimiter (e.g., `"---"`).
    pub front_matter_delimiter: Option<String>,
    /// Enable autolinks.
    pub autolink: bool,
}

impl Default for ParseOptions {
    fn default() -> Self {
        Self {
            debounce_ms: 300,
            tab_size: 2,
            tables: true,
            task_lists: true,
            strikethrough: true,
            footnotes: true,
            front_matter_delimiter: Some("---".to_string()),
            autolink: true,
        }
    }
}

/// An entry in the heading index extracted from the parsed AST.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HeadingEntry {
    /// Heading level (1-6).
    pub level: u8,
    /// Heading text content.
    pub text: String,
    /// Slugified anchor ID for deep linking.
    pub anchor: String,
    /// Source line number (0-based) for cursor sync.
    pub line: usize,
}

/// Sub-variant for `Mode::LivePreview`.
///
/// Controls whether the live preview renders as a side-by-side split pane
/// (existing behaviour) or as an inline Obsidian-style editor where all
/// blocks render as formatted HTML except the one under the cursor.
#[derive(Default, Clone, Copy, PartialEq, Eq, Debug)]
pub enum LivePreviewVariant {
    /// Editor on one side, rendered preview on the other (default, backwards-compatible).
    #[default]
    SplitPane,
    /// Single surface: every block is rendered HTML except the cursor block,
    /// which reverts to raw markdown for editing.
    Inline,
}

/// An entry in the block list extracted for the inline editor.
///
/// Each top-level AST block (paragraph, heading, code block, …) gets one
/// `BlockEntry`.  Top-level lists are **split into per-item blocks** — each
/// `Item` / `TaskItem` child becomes its own entry with `is_list_item: true`.
/// Front matter is excluded.
#[derive(Debug, Clone, PartialEq)]
pub struct BlockEntry {
    /// Zero-based index within the document's top-level block list (front matter excluded).
    pub index: usize,
    /// Raw markdown source text for this block, extracted via comrak `sourcepos`.
    pub raw: String,
    /// Pre-rendered HTML fragment for this block, wrapped in
    /// `<div data-block-index="{index}">…</div>` for use with `innerHTML`
    /// in the inline editor.
    pub html: String,
    /// First source line of this block (1-indexed, from comrak `sourcepos`).
    pub start_line: u32,
    /// Last source line of this block (1-indexed, from comrak `sourcepos`).
    pub end_line: u32,
    /// `true` when this entry represents a single list item (`Item` or `TaskItem`).
    /// Consecutive list-item blocks are joined with `"\n"` during reconstruction;
    /// all other block boundaries use `"\n\n"`.
    pub is_list_item: bool,
}

/// The type of an AST node, abstracting away from `pulldown_cmark::Event`
/// to provide a `'static`, owned structure suitable for Dioxus `Props`
/// and headless block component overrides.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NodeType {
    // Blocks
    Paragraph,
    Heading(u8),
    BlockQuote,
    CodeBlock(String), // Language
    List(Option<u64>), // Start index
    Item,
    Table,
    TableHead,
    TableRow,
    TableCell,
    Rule,
    HtmlBlock,
    DefinitionList,
    DefinitionListTitle,
    DefinitionListDefinition,
    Superscript,
    Subscript,
    // Inlines
    Text(String),
    Code(String),
    Html(String),
    Emphasis,
    Strong,
    Strikethrough,
    Link { url: String, title: String },
    Image { url: String, title: String },
    FootnoteReference(String),
    SoftBreak,
    HardBreak,
    TaskListMarker(bool),
    // Custom Extensions
    Wikilink(String),
    Tag(String),
}

/// A fully owned, `'static` AST node mapped from byte boundaries in the Rope.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OwnedAstNode {
    pub node_type: NodeType,
    pub range: Range<usize>,
    pub children: Vec<OwnedAstNode>,
}

/// The parsed document produced by `parse_document()`.
/// Contains the pre-rendered Dioxus Element plus extracted metadata.
///
/// `PartialEq` always returns `false` because `Element` is not comparable.
/// This ensures `Memo<Rc<ParsedDoc>>` always notifies subscribers on update,
/// which is correct — every re-parse produces a semantically new document.
pub struct ParsedDoc {
    /// The rendered Dioxus element tree.
    pub element: dioxus::prelude::Element,
    /// Headings extracted from the AST for `use_heading_index()`.
    pub headings: Vec<HeadingEntry>,
    /// Raw front matter string (consumer parses YAML/TOML).
    pub front_matter: Option<String>,
    /// Top-level blocks for the inline editor (cursor-aware block switching).
    pub blocks: Vec<BlockEntry>,
    /// The owned, strictly-typed Abstract Syntax Tree representing the entire document.
    pub ast: Vec<OwnedAstNode>,
}

impl PartialEq for ParsedDoc {
    fn eq(&self, _other: &Self) -> bool {
        // Element is not comparable; every re-parse is a new document.
        false
    }
}

// ── HTML render policy ───────────────────────────────────────────────

/// Controls how raw HTML blocks and inline HTML in markdown are rendered.
///
/// By default, raw HTML is **escaped** (displayed as visible text) to prevent
/// cross-site scripting (XSS) attacks. Choose a policy based on how much you
/// trust the markdown source:
///
/// | Policy | Use when | XSS safe? |
/// |-----------|----------------------------------------------|-----------|
/// | `Escape` | Untrusted / user-generated markdown (default)| Yes |
/// | `Sanitized` | User-generated markdown where you want HTML formatting but not scripts (requires `sanitize` feature) | Yes |
/// | `Trusted` | You control the markdown source entirely | **No** |
///
/// # Security
///
/// **`Trusted` mode renders arbitrary HTML without any sanitization.** If the
/// markdown contains `<script>`, `<iframe>`, `onload=`, or any other active
/// content, it **will** be injected into the DOM. Never use `Trusted` with
/// user-generated or untrusted markdown — this is a direct XSS vector.
///
/// For user-generated content that needs HTML rendering, enable the `sanitize`
/// feature and use [`HtmlRenderPolicy::Sanitized`], which strips dangerous
/// elements and attributes via the [`ammonia`](https://docs.rs/ammonia) crate.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub enum HtmlRenderPolicy {
    /// Escape HTML — render as visible text. Safe for all inputs.
    #[default]
    Escape,

    /// Sanitize HTML with [`ammonia`](https://docs.rs/ammonia) before rendering.
    ///
    /// Strips dangerous elements (`<script>`, `<iframe>`, `<object>`, etc.) and
    /// event-handler attributes (`onload`, `onclick`, etc.) while preserving safe
    /// formatting tags (`<b>`, `<i>`, `<a>`, `<code>`, etc.).
    ///
    /// Requires the `sanitize` Cargo feature. Falls back to `Escape` if the
    /// feature is not enabled.
    Sanitized,

    /// Render raw HTML via `dangerous_inner_html` **without any sanitization**.
    ///
    /// # Security Warning
    ///
    /// **This is a direct XSS vector.** Only use this when you fully control the
    /// markdown source (e.g., static content compiled into your binary). Never
    /// use with user-generated input.
    Trusted,
}

// ── Vim modal editing types ──────────────────────────────────────────

/// Vim modal editing mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum VimMode {
    /// Default: all keystrokes pass through to the textarea (browser default behavior).
    #[default]
    Insert,
    /// Normal mode: hjkl navigation, mode transitions.
    Normal,
    /// Visual mode: text selection.
    Visual,
    /// Command mode: colon commands.
    Command,
}

/// Action returned by `VimState::handle_key`.
#[derive(Debug, Clone, PartialEq)]
pub enum VimAction {
    /// Key passes through to the browser/textarea unchanged.
    PassThrough,
    /// Prevent default and run this eval() JS string.
    PreventAndEval(String),
    /// Transition to a new vim mode.
    ModeChange(VimMode),
    /// Execute a command string (from Command mode).
    ExecuteCommand(String),
}

/// Vim modal editing state.
#[derive(Debug, Clone, Default)]
pub struct VimState {
    /// Current mode.
    pub mode: VimMode,
    /// Accumulated command buffer (for Command mode input).
    pub command_buffer: String,
}

impl VimState {
    /// Handle a key event and return the action to take.
    /// Pure function — no side effects, no eval, no DOM access.
    ///
    /// # Arguments
    /// - `key`: the key string from `KeyboardEvent::key().to_string()`
    /// - `ctrl`: whether Ctrl is held
    /// - `shift`: whether Shift is held
    /// - `editor_id`: DOM ID of the editor textarea for generated JS
    pub fn handle_key(&mut self, key: &str, ctrl: bool, shift: bool, editor_id: &str) -> VimAction {
        match self.mode {
            VimMode::Insert => {
                if key == "Escape" {
                    self.mode = VimMode::Normal;
                    return VimAction::ModeChange(VimMode::Normal);
                }
                VimAction::PassThrough
            }
            VimMode::Normal => match key {
                "i" if !ctrl && !shift => {
                    self.mode = VimMode::Insert;
                    VimAction::ModeChange(VimMode::Insert)
                }
                "v" if !ctrl && !shift => {
                    self.mode = VimMode::Visual;
                    VimAction::ModeChange(VimMode::Visual)
                }
                ":" => {
                    self.mode = VimMode::Command;
                    self.command_buffer.clear();
                    VimAction::ModeChange(VimMode::Command)
                }
                "Escape" => VimAction::PassThrough, // already Normal
                "h" => VimAction::PreventAndEval(vim_move_js(editor_id, "left")),
                "l" => VimAction::PreventAndEval(vim_move_js(editor_id, "right")),
                "j" => VimAction::PreventAndEval(vim_move_js(editor_id, "down")),
                "k" => VimAction::PreventAndEval(vim_move_js(editor_id, "up")),
                _ => VimAction::PassThrough,
            },
            VimMode::Visual => {
                if key == "Escape" {
                    self.mode = VimMode::Normal;
                    return VimAction::ModeChange(VimMode::Normal);
                }
                VimAction::PassThrough
            }
            VimMode::Command => {
                if key == "Escape" {
                    self.mode = VimMode::Normal;
                    self.command_buffer.clear();
                    return VimAction::ModeChange(VimMode::Normal);
                }
                if key == "Enter" {
                    let cmd = self.command_buffer.clone();
                    self.command_buffer.clear();
                    self.mode = VimMode::Normal;
                    return VimAction::ExecuteCommand(cmd);
                }
                if key.len() == 1 {
                    self.command_buffer.push_str(key);
                }
                VimAction::PassThrough
            }
        }
    }
}

// ── Source map types ─────────────────────────────────────────────────

/// Entry in the source map linking a rendered DOM element to its source line range.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceMapEntry {
    /// First source line (1-indexed) covered by this element.
    pub source_line_start: usize,
    /// Last source line (1-indexed) covered by this element.
    pub source_line_end: usize,
    /// The `id` attribute of the rendered DOM element.
    pub element_id: String,
}

/// Source map linking rendered DOM elements back to source line ranges.
#[derive(Debug, Clone, Default)]
pub struct SourceMap {
    /// Entries sorted by `source_line_start` ascending.
    pub entries: Vec<SourceMapEntry>,
}

impl SourceMap {
    /// Returns the first entry whose line range contains `line`.
    ///
    /// `line` is 1-indexed. Returns `None` if no entry covers the given line.
    pub fn find_entry_by_line(&self, line: usize) -> Option<&SourceMapEntry> {
        self.entries
            .iter()
            .find(|e| e.source_line_start <= line && line <= e.source_line_end)
    }
}

/// Fired by [`InlineEditor`] on every `oninput` with the active block's raw text
/// and cursor offset within that block.  Used by consumers to wire inline-trigger
/// suggestions (e.g. `dioxus-nox-suggest`) without coupling markdown to suggest.
#[derive(Debug, Clone, PartialEq)]
pub struct ActiveBlockInputEvent {
    /// Raw markdown text of the active block.
    pub raw_text: String,
    /// Visible text projection of the active block (markers may be concealed).
    pub visible_text: String,
    /// Cursor position as UTF-16 code-unit offset in `raw_text`.
    pub cursor_raw_utf16: usize,
    /// Cursor position as UTF-16 code-unit offset in `visible_text`.
    pub cursor_visible_utf16: usize,
    /// Absolute start byte offset of the active block in the full document.
    pub block_start: usize,
    /// Absolute end byte offset of the active block in the full document.
    pub block_end: usize,
}

/// Generate JS for vim cursor movement via `eval()`.
/// Targets the textarea with the given `editor_id`.
pub(crate) fn vim_move_js(editor_id: &str, direction: &str) -> String {
    match direction {
        "left" => format!(
            "(function(){{ const el = document.getElementById('{editor_id}'); if(!el) return; \
            el.selectionStart = el.selectionEnd = Math.max(0, el.selectionStart - 1); }})();"
        ),
        "right" => format!(
            "(function(){{ const el = document.getElementById('{editor_id}'); if(!el) return; \
            const max = el.value.length; \
            el.selectionStart = el.selectionEnd = Math.min(max, el.selectionEnd + 1); }})();"
        ),
        "up" => format!(
            "(function(){{ const el = document.getElementById('{editor_id}'); if(!el) return; \
            const pos = el.selectionStart; const text = el.value; \
            const lineStart = text.lastIndexOf('\\n', pos - 1) + 1; \
            const col = pos - lineStart; \
            const prevLineEnd = lineStart > 0 ? lineStart - 1 : 0; \
            const prevLineStart = text.lastIndexOf('\\n', prevLineEnd - 1) + 1; \
            const newPos = Math.min(prevLineStart + col, prevLineEnd); \
            el.selectionStart = el.selectionEnd = newPos; }})();"
        ),
        "down" => format!(
            "(function(){{ const el = document.getElementById('{editor_id}'); if(!el) return; \
            const pos = el.selectionStart; const text = el.value; \
            const lineStart = text.lastIndexOf('\\n', pos - 1) + 1; \
            const col = pos - lineStart; \
            const lineEnd = text.indexOf('\\n', pos); \
            if(lineEnd === -1) return; \
            const nextLineStart = lineEnd + 1; \
            const nextLineEnd = text.indexOf('\\n', nextLineStart); \
            const nextLineLen = (nextLineEnd === -1 ? text.length : nextLineEnd) - nextLineStart; \
            el.selectionStart = el.selectionEnd = nextLineStart + Math.min(col, nextLineLen); }})();"
        ),
        _ => String::new(),
    }
}
