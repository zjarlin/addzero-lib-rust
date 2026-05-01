use dioxus::document::Eval;
use dioxus::prelude::document;

/// Adapter boundary for caret/selection interop.
///
/// All DOM/eval behavior should be expressed here, and consumers should call
/// helper functions in this module instead of `document::eval()` directly.
pub trait CaretAdapter: Send + Sync {
    /// JS to read `[selectionStart, selectionEnd]` from a textarea.
    fn read_textarea_selection_js(&self, editor_id: &str) -> String;
    /// JS to read `selectionStart` from a textarea.
    fn read_textarea_cursor_js(&self, editor_id: &str) -> String;
    /// JS to compute UTF-16 offset from block start to current DOM selection.
    fn read_block_visual_offset_js(&self, block_id: &str) -> String;
    /// JS to read contenteditable cursor selection as UTF-16 offset.
    fn read_contenteditable_selection_js(&self, block_id: &str) -> String;
    /// JS to read contenteditable selection details as compact string:
    /// `start<US>end<US>collapsed`.
    fn read_contenteditable_selection_detailed_js(&self, block_id: &str) -> String;
    /// JS to read cached beforeinput metadata as compact string:
    /// `start<US>end<US>collapsed<US>inputType<US>data`.
    fn read_contenteditable_beforeinput_meta_js(&self, block_id: &str) -> String;
    /// JS to focus and set textarea selection, with hydration-safe retries.
    fn mount_active_textarea_js(&self, textarea_id: &str, cursor_utf16: usize) -> String;
    /// JS to read contenteditable plain text.
    fn read_contenteditable_text_js(&self, block_id: &str) -> String;
    /// JS to place caret in a contenteditable block by UTF-16 code-unit index.
    fn set_contenteditable_selection_js(&self, block_id: &str, raw_utf16: usize) -> String;
    /// JS to restore a non-collapsed selection in a contenteditable block by
    /// visible UTF-16 offsets `[start, end]`.
    fn set_contenteditable_selection_range_js(
        &self,
        block_id: &str,
        start_utf16: usize,
        end_utf16: usize,
    ) -> String;
    /// JS hook for contenteditable input/traversal behavior.
    fn bind_contenteditable_input_js(&self, block_id: &str) -> String;
}

#[derive(Debug, Default)]
pub struct WebviewCaretAdapter;

impl CaretAdapter for WebviewCaretAdapter {
    fn read_textarea_selection_js(&self, editor_id: &str) -> String {
        format!(
            "var el = document.getElementById('{editor_id}');\
             if(el) dioxus.send([el.selectionStart ?? 0, el.selectionEnd ?? 0]);\
             else dioxus.send([0, 0]);"
        )
    }

    fn read_textarea_cursor_js(&self, editor_id: &str) -> String {
        format!(
            "var el = document.getElementById('{editor_id}');\
             if(el) dioxus.send(el.selectionStart ?? 0);\
             else dioxus.send(0);"
        )
    }

    fn read_block_visual_offset_js(&self, block_id: &str) -> String {
        format!(
            r#"(function() {{
    var el = document.getElementById('{block_id}');
    if (!el) {{ dioxus.send("0"); return; }}
    var sel = window.getSelection();
    if (!sel || sel.rangeCount === 0) {{ dioxus.send("0"); return; }}
    var range = sel.getRangeAt(0);
    var pre = range.cloneRange();
    pre.selectNodeContents(el);
    pre.setEnd(range.endContainer, range.endOffset);
    dioxus.send(pre.toString().length.toString());
}})();"#
        )
    }

    fn mount_active_textarea_js(&self, textarea_id: &str, cursor_utf16: usize) -> String {
        format!(
            r#"(function() {{
    var el = document.getElementById('{textarea_id}');
    if (!el) return;
    var tryFocus = function(attempts) {{
        if (attempts > 10) return;
        if (el.value.length === 0 && {cursor_utf16} > 0) {{
            setTimeout(function() {{ tryFocus(attempts + 1); }}, 10);
            return;
        }}
        el.focus();
        try {{
            el.setSelectionRange({cursor_utf16}, {cursor_utf16});
        }} catch (e) {{}}
        var resize = function() {{
            el.style.height = 'auto';
            el.style.height = el.scrollHeight + 'px';
        }};
        resize();
        if (!el._noxResizeBound) {{
            el.addEventListener('input', resize);
            el._noxResizeBound = true;
        }}
        if (!el._noxTraversalBound) {{
            el.addEventListener('keydown', function(e) {{
                if (e.key === 'ArrowUp') {{
                    var pos = el.selectionStart;
                    var text = el.value;
                    var isFirstLine = text.lastIndexOf('\n', pos - 1) === -1;
                    if (isFirstLine) {{
                        e.preventDefault();
                        dioxus.send("prev");
                    }}
                }} else if (e.key === 'ArrowDown') {{
                    var pos = el.selectionStart;
                    var text = el.value;
                    var isLastLine = text.indexOf('\n', pos) === -1;
                    if (isLastLine) {{
                        e.preventDefault();
                        dioxus.send("next");
                    }}
                }} else if (e.key === 'Backspace') {{
                    var pos = el.selectionStart;
                    if (pos === 0 && el.selectionStart === el.selectionEnd) {{
                        e.preventDefault();
                        dioxus.send("backjoin");
                    }}
                }} else if (e.key === 'Enter') {{
                    if (!e.shiftKey) {{
                        e.preventDefault();
                        dioxus.send("split:" + (el.selectionStart ?? 0));
                    }}
                }}
            }});
            el._noxTraversalBound = true;
        }}
    }};
    tryFocus(0);
}})();"#
        )
    }

    fn read_contenteditable_selection_js(&self, block_id: &str) -> String {
        self.read_block_visual_offset_js(block_id)
    }

    fn read_contenteditable_selection_detailed_js(&self, block_id: &str) -> String {
        format!(
            r#"(function() {{
    var root = document.getElementById('{block_id}');
    if (!root) {{ dioxus.send("0\u001f0\u001f1"); return; }}
    var sel = window.getSelection();
    if (!sel || sel.rangeCount === 0) {{ dioxus.send("0\u001f0\u001f1"); return; }}
    var range = sel.getRangeAt(0);
    var toOffset = function(node, offset) {{
        if (!node || !root.contains(node)) return 0;
        try {{
            var r = document.createRange();
            r.selectNodeContents(root);
            r.setEnd(node, offset);
            return r.toString().length;
        }} catch (_e) {{
            return 0;
        }}
    }};
    var start = toOffset(range.startContainer, range.startOffset);
    var end = toOffset(range.endContainer, range.endOffset);
    var collapsed = start === end ? "1" : "0";
    dioxus.send(start.toString() + "\u001f" + end.toString() + "\u001f" + collapsed);
}})();"#
        )
    }

    fn read_contenteditable_beforeinput_meta_js(&self, block_id: &str) -> String {
        format!(
            r#"(function() {{
    var root = document.getElementById('{block_id}');
    if (!root || typeof root._noxBeforeInputMeta !== "string") {{
        dioxus.send("");
        return;
    }}
    var meta = root._noxBeforeInputMeta;
    root._noxBeforeInputMeta = null;
    dioxus.send(meta);
}})();"#
        )
    }

    fn read_contenteditable_text_js(&self, block_id: &str) -> String {
        format!(
            "var el = document.getElementById('{block_id}');\
             if(el) dioxus.send(el.innerText ?? '');\
             else dioxus.send('');"
        )
    }

    fn set_contenteditable_selection_js(&self, block_id: &str, raw_utf16: usize) -> String {
        format!(
            r#"(function() {{
    var root = document.getElementById('{block_id}');
    if (!root) return;
    root.focus();
    var walker = document.createTreeWalker(root, NodeFilter.SHOW_TEXT);
    var remaining = {raw_utf16};
    var node = null;
    while ((node = walker.nextNode())) {{
        var len = (node.nodeValue || '').length;
        if (remaining <= len) {{
            try {{
                var range = document.createRange();
                range.setStart(node, remaining);
                range.collapse(true);
                var sel = window.getSelection();
                sel.removeAllRanges();
                sel.addRange(range);
            }} catch (e) {{}}
            return;
        }}
        remaining -= len;
    }}
}})();"#
        )
    }

    fn set_contenteditable_selection_range_js(
        &self,
        block_id: &str,
        start_utf16: usize,
        end_utf16: usize,
    ) -> String {
        format!(
            r#"(function() {{
    var root = document.getElementById('{block_id}');
    if (!root) return;
    root.focus();
    function findPos(target) {{
        var walker = document.createTreeWalker(root, NodeFilter.SHOW_TEXT);
        var remaining = target;
        var node = null;
        while ((node = walker.nextNode())) {{
            var len = (node.nodeValue || '').length;
            if (remaining <= len) return {{ node: node, offset: remaining }};
            remaining -= len;
        }}
        return null;
    }}
    var s = findPos({start_utf16});
    var e = findPos({end_utf16});
    if (!s || !e) return;
    try {{
        var range = document.createRange();
        range.setStart(s.node, s.offset);
        range.setEnd(e.node, e.offset);
        var sel = window.getSelection();
        sel.removeAllRanges();
        sel.addRange(range);
    }} catch (ex) {{}}
}})();"#
        )
    }

    fn bind_contenteditable_input_js(&self, block_id: &str) -> String {
        format!(
            r#"(function() {{
    var root = document.getElementById('{block_id}');
    if (!root) {{ dioxus.send("missing"); return; }}
    if (root._noxBeforeInputBound) {{ dioxus.send("bound"); return; }}
    root._noxBeforeInputMeta = null;

    var selectionDetails = function() {{
        var sel = window.getSelection();
        if (!sel || sel.rangeCount === 0) {{
            return {{ start: 0, end: 0, collapsed: true }};
        }}
        var range = sel.getRangeAt(0);
        var toOffset = function(node, offset) {{
            if (!node || !root.contains(node)) return 0;
            try {{
                var r = document.createRange();
                r.selectNodeContents(root);
                r.setEnd(node, offset);
                return r.toString().length;
            }} catch (_e) {{
                return 0;
            }}
        }};
        var start = toOffset(range.startContainer, range.startOffset);
        var end = toOffset(range.endContainer, range.endOffset);
        if (end < start) {{
            var t = start; start = end; end = t;
        }}
        return {{ start: start, end: end, collapsed: start === end }};
    }};

    root.addEventListener('beforeinput', function(e) {{
        var sel = selectionDetails();
        var inputType = (e && typeof e.inputType === 'string') ? e.inputType : '';
        var data = (e && typeof e.data === 'string') ? e.data : '';
        root._noxBeforeInputMeta =
            sel.start.toString() + '\u001f' +
            sel.end.toString() + '\u001f' +
            (sel.collapsed ? '1' : '0') + '\u001f' +
            inputType + '\u001f' +
            data;
    }});

    root._noxBeforeInputBound = true;
    dioxus.send("bound");
}})();"#
        )
    }
}

#[derive(Debug, Default)]
pub struct NoopCaretAdapter;

impl CaretAdapter for NoopCaretAdapter {
    fn read_textarea_selection_js(&self, _editor_id: &str) -> String {
        "dioxus.send([0, 0]);".to_string()
    }

    fn read_textarea_cursor_js(&self, _editor_id: &str) -> String {
        "dioxus.send(0);".to_string()
    }

    fn read_block_visual_offset_js(&self, _block_id: &str) -> String {
        "dioxus.send(\"0\");".to_string()
    }

    fn mount_active_textarea_js(&self, _textarea_id: &str, _cursor_utf16: usize) -> String {
        "dioxus.send(\"noop\");".to_string()
    }

    fn read_contenteditable_selection_js(&self, _block_id: &str) -> String {
        "dioxus.send(\"0\");".to_string()
    }

    fn read_contenteditable_selection_detailed_js(&self, _block_id: &str) -> String {
        "dioxus.send(\"0\\u001f0\\u001f1\");".to_string()
    }

    fn read_contenteditable_beforeinput_meta_js(&self, _block_id: &str) -> String {
        "dioxus.send(\"\");".to_string()
    }

    fn read_contenteditable_text_js(&self, _block_id: &str) -> String {
        "dioxus.send(\"\");".to_string()
    }

    fn set_contenteditable_selection_js(&self, _block_id: &str, _raw_utf16: usize) -> String {
        "dioxus.send(\"noop\");".to_string()
    }

    fn set_contenteditable_selection_range_js(
        &self,
        _block_id: &str,
        _start_utf16: usize,
        _end_utf16: usize,
    ) -> String {
        String::new()
    }

    fn bind_contenteditable_input_js(&self, _block_id: &str) -> String {
        "dioxus.send(\"noop\");".to_string()
    }
}

#[cfg(any(
    target_arch = "wasm32",
    target_os = "windows",
    target_os = "macos",
    target_os = "linux",
    target_os = "ios",
    target_os = "android"
))]
static WEBVIEW_ADAPTER: WebviewCaretAdapter = WebviewCaretAdapter;
#[cfg(not(any(
    target_arch = "wasm32",
    target_os = "windows",
    target_os = "macos",
    target_os = "linux",
    target_os = "ios",
    target_os = "android"
)))]
static NOOP_ADAPTER: NoopCaretAdapter = NoopCaretAdapter;

/// Returns the platform adapter for caret/selection interop.
pub fn caret_adapter() -> &'static dyn CaretAdapter {
    #[cfg(any(
        target_arch = "wasm32",
        target_os = "windows",
        target_os = "macos",
        target_os = "linux",
        target_os = "ios",
        target_os = "android"
    ))]
    {
        &WEBVIEW_ADAPTER
    }

    #[cfg(not(any(
        target_arch = "wasm32",
        target_os = "windows",
        target_os = "macos",
        target_os = "linux",
        target_os = "ios",
        target_os = "android"
    )))]
    {
        &NOOP_ADAPTER
    }
}

/// Start an eval session.
pub fn start_eval(js: &str) -> Eval {
    document::eval(js)
}

/// Evaluate JS and ignore the result.
pub async fn eval_void(js: &str) {
    let _ = document::eval(js).await;
}

/// Receive a string from an eval session.
pub async fn recv_string(eval: &mut Eval) -> Option<String> {
    eval.recv::<String>().await.ok()
}

/// Receive `u64` from an eval session.
pub async fn recv_u64(eval: &mut Eval) -> Option<u64> {
    eval.recv::<u64>().await.ok()
}

/// Receive `f64` from an eval session.
pub async fn recv_f64(eval: &mut Eval) -> Option<f64> {
    eval.recv::<f64>().await.ok()
}

/// Receive `Vec<u64>` from an eval session.
pub async fn recv_vec_u64(eval: &mut Eval) -> Option<Vec<u64>> {
    eval.recv::<Vec<u64>>().await.ok()
}
