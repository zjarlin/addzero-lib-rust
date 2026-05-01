use std::cell::RefCell;
use std::rc::Rc;
use std::sync::atomic::{AtomicU64, Ordering};

#[cfg(target_arch = "wasm32")]
use gloo_timers::callback::Timeout;

use dioxus::prelude::*;
use dioxus_core::use_drop;

pub(crate) static NEXT_VH_ID: AtomicU64 = AtomicU64::new(1);

use crate::context::{MarkdownContext, escape_js};
use crate::interop;
use crate::parser::parse_document;
use crate::types::{HeadingEntry, ParsedDoc};

/// Computes the scroll ratio for scroll synchronization.
///
/// Returns `scroll_top / (scroll_height - client_height)` clamped to `[0.0, 1.0]`.
/// Returns `0.0` when `scroll_height <= client_height` (no overflow / no scrollable area).
#[cfg(test)]
pub(crate) fn compute_scroll_ratio(scroll_top: f64, scroll_height: f64, client_height: f64) -> f64 {
    let max_scroll = scroll_height - client_height;
    if max_scroll <= 0.0 {
        return 0.0;
    }
    (scroll_top / max_scroll).clamp(0.0, 1.0)
}

/// Generates JS that wraps the current textarea selection with `prefix` and `suffix`,
/// then dispatches an `input` event to keep Rust state in sync.
///
/// The generated JS:
/// 1. Gets the editor element by ID
/// 2. Reads `selectionStart` and `selectionEnd`
/// 3. Wraps the selected text with prefix/suffix
/// 4. Sets cursor position after the prefix (around the selected text)
/// 5. Dispatches a synthetic `input` event
pub(crate) fn wrap_selection_js(editor_id: &str, prefix: &str, suffix: &str) -> String {
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
    return el.value;
}})();"#,
        editor_id = editor_id,
        prefix = prefix_escaped,
        suffix = suffix_escaped,
        prefix_len = prefix.len(),
    )
}

/// Generates JS that inserts `tab_size` spaces at the cursor position,
/// then dispatches an `input` event to keep Rust state in sync.
pub(crate) fn tab_indent_js(editor_id: &str, tab_size: u8) -> String {
    let spaces = " ".repeat(tab_size as usize);
    format!(
        r#"(function() {{
    var el = document.getElementById('{editor_id}');
    if (!el) return null;
    var start = el.selectionStart;
    el.value = el.value.substring(0, start) + '{spaces}' + el.value.substring(start);
    el.setSelectionRange(start + {tab_size}, start + {tab_size});
    el.dispatchEvent(new Event('input', {{ bubbles: true }}));
    return el.value;
}})();"#,
        editor_id = editor_id,
        spaces = spaces,
        tab_size = tab_size,
    )
}

/// Generates JS that selects all content within the element with `element_id`.
///
/// Uses `window.getSelection()` + `Range.selectNodeContents()` to create a
/// selection spanning the entire element. Used by Content (Read mode) and
/// InlineEditor (Inline mode) to scope Ctrl+A to the markdown region.
pub(crate) fn select_all_children_js(element_id: &str) -> String {
    format!(
        r#"(function() {{
    var el = document.getElementById('{element_id}');
    if (!el) return;
    var sel = window.getSelection();
    sel.removeAllRanges();
    var range = document.createRange();
    range.selectNodeContents(el);
    sel.addRange(range);
}})();"#,
        element_id = element_id,
    )
}

/// Reactive hook returning the heading index from the current parsed document.
///
/// Consumes `MarkdownContext` — must be called inside a descendant of `markdown::Root`.
/// Returns a `Vec<HeadingEntry>` that updates whenever the parsed document changes.
///
/// ```rust,ignore
/// let headings = use_heading_index();
/// for h in headings.iter() {
///     // h.level, h.text, h.anchor, h.line
/// }
/// ```
pub fn use_heading_index() -> Memo<Vec<HeadingEntry>> {
    let ctx = use_context::<MarkdownContext>();
    use_memo(move || (ctx.parsed_doc)().headings.clone())
}

/// Synchronizes the preview pane scroll position to match the editor's scroll ratio.
///
/// Reads the editor's current scroll ratio and applies it to the preview pane.
/// Must be called inside `spawn(async { ... })` or `use_effect` — never in component body.
///
/// # Eval Justification
///
/// `document::eval()` is used here because `MountedData` has no `scrollTo(offset)` write API
/// in Dioxus 0.7.3. Reads could use `get_scroll_offset` / `get_scroll_size`, but the write
/// still requires eval. `document::eval()` works on all WebView targets (Web, Desktop/Wry,
/// iOS, Android). Revisit when Dioxus adds a scroll-position write API.
pub(crate) async fn sync_editor_to_preview(editor_id: &str, preview_id: &str) {
    let js = format!(
        r#"
        const ed = document.getElementById('{editor_id}');
        const pr = document.getElementById('{preview_id}');
        if (!ed || !pr) return null;
        const maxEd = Math.max(0, ed.scrollHeight - ed.clientHeight);
        const ratio = maxEd > 0 ? ed.scrollTop / maxEd : 0;
        pr.scrollTop = ratio * Math.max(0, pr.scrollHeight - pr.clientHeight);
        null
    "#
    );
    interop::eval_void(&js).await;
}

/// Synchronizes the editor pane scroll position to match the preview's scroll ratio.
///
/// Reads the preview's current scroll ratio and applies it to the editor pane.
/// Must be called inside `spawn(async { ... })` or `use_effect` — never in component body.
///
/// # Eval Justification
///
/// `document::eval()` is used here because `MountedData` has no `scrollTo(offset)` write API
/// in Dioxus 0.7.3. Reads could use `get_scroll_offset` / `get_scroll_size`, but the write
/// still requires eval. `document::eval()` works on all WebView targets (Web, Desktop/Wry,
/// iOS, Android). Revisit when Dioxus adds a scroll-position write API.
pub(crate) async fn sync_preview_to_editor(editor_id: &str, preview_id: &str) {
    let js = format!(
        r#"
        const pr = document.getElementById('{preview_id}');
        const ed = document.getElementById('{editor_id}');
        if (!ed || !pr) return null;
        const maxPr = Math.max(0, pr.scrollHeight - pr.clientHeight);
        const ratio = maxPr > 0 ? pr.scrollTop / maxPr : 0;
        ed.scrollTop = ratio * Math.max(0, ed.scrollHeight - ed.clientHeight);
        null
    "#
    );
    interop::eval_void(&js).await;
}

/// Synchronizes the editor gutter scroll position to match the textarea's scrollTop.
///
/// Called from Editor's onscroll handler when editor line numbers are enabled.
/// Sets gutter.scrollTop = textarea.scrollTop so line numbers stay aligned.
pub(crate) async fn sync_gutter_scroll(editor_id: &str, gutter_id: &str) {
    let js = format!(
        r#"
        const ed = document.getElementById('{editor_id}');
        const gt = document.getElementById('{gutter_id}');
        if (ed && gt) {{ gt.scrollTop = ed.scrollTop; }}
        null
    "#
    );
    interop::eval_void(&js).await;
}

/// Hook for viewport height tracking (iOS/Android virtual keyboard).
///
/// Returns a `Signal<f64>` that tracks `window.visualViewport.height` (or
/// `window.innerHeight` as fallback). Updates on resize events, including
/// when the virtual keyboard appears or disappears on mobile.
///
/// Uses `document::eval()` — safe on all Dioxus targets (Web, Desktop, iOS, Android).
pub fn use_viewport_height() -> Signal<f64> {
    let mut height = use_signal(|| 0.0_f64);
    let cleanup_id = use_hook(|| NEXT_VH_ID.fetch_add(1, Ordering::Relaxed));

    use_effect(move || {
        spawn(async move {
            let mut ev = interop::start_eval(&format!(
                r#"
                const send = () => dioxus.send(
                    window.visualViewport ? window.visualViewport.height : window.innerHeight
                );
                send();
                const target = window.visualViewport || window;
                target.addEventListener('resize', send);
                window.__nox_md_cleanup = window.__nox_md_cleanup || {{}};
                window.__nox_md_cleanup['nox_md_vh_{cleanup_id}'] = function() {{
                    target.removeEventListener('resize', send);
                }};
            "#
            ));
            while let Some(h) = interop::recv_f64(&mut ev).await {
                height.set(h);
            }
        });
    });

    use_drop(move || {
        let key = format!("nox_md_vh_{cleanup_id}");
        spawn(async move {
            interop::eval_void(&format!(
                "if(window.__nox_md_cleanup && window.__nox_md_cleanup['{key}']){{window.__nox_md_cleanup['{key}']();delete window.__nox_md_cleanup['{key}'];}}"
            ))
            .await;
        });
    });

    height
}

/// Debounced parse hook. Triggers a re-parse after `delay_ms` of inactivity.
///
/// Returns a `Signal<Rc<ParsedDoc>>` that updates after the debounce fires.
/// The caller provides the raw content buffer and a debounce delay.
///
/// On wasm32, uses `gloo_timers::callback::Timeout` for the debounce timer (works on all
/// WebView-based Dioxus targets: Web, Desktop via Wry, iOS, Android).
/// On non-wasm targets (native), fires immediately (no async timer without a tokio dep).
pub fn use_debounced_parse(
    raw_content: Signal<Rc<RefCell<crop::Rope>>>,
    delay_ms: u32,
) -> (Signal<Rc<ParsedDoc>>, Callback<()>) {
    let parsed = use_signal(|| {
        Rc::new(ParsedDoc {
            element: rsx! {},
            headings: vec![],
            front_matter: None,
            blocks: vec![],
            ast: vec![],
        })
    });

    #[cfg(target_arch = "wasm32")]
    let timer_handle: Rc<RefCell<Option<Timeout>>> = use_hook(|| Rc::new(RefCell::new(None)));

    let trigger = {
        #[cfg(target_arch = "wasm32")]
        let timer_handle = timer_handle.clone();
        Callback::new(move |_: ()| {
            // Clone the Rc to the content buffer; read actual content when timer fires
            // so we always parse the latest text, not a stale snapshot from trigger time.
            let content_rc = raw_content.read().clone();
            let mut parsed = parsed;

            #[cfg(target_arch = "wasm32")]
            {
                let timer_rc2 = timer_handle.clone();
                let mut guard = timer_handle.borrow_mut();
                // Dropping the previous Some(Timeout) cancels the old timer.
                let _ = guard.take();
                *guard = Some(Timeout::new(delay_ms, move || {
                    let rope = content_rc.borrow().clone();
                    let doc = parse_document(&rope);
                    parsed.set(Rc::new(doc));
                    *timer_rc2.borrow_mut() = None;
                }));
            }

            #[cfg(not(target_arch = "wasm32"))]
            {
                // web_sys comment: confirmed no Dioxus 0.7 native timer API as of 2026-02-27.
                let _ = delay_ms;
                let rope = content_rc.borrow().clone();
                let doc = parse_document(&rope);
                parsed.set(Rc::new(doc));
            }
        })
    };

    (parsed, trigger)
}
