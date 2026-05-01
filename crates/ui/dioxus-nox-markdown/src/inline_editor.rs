use dioxus::prelude::*;

use crate::context::{CursorContext, MarkdownContext};
use crate::hooks::select_all_children_js;
use crate::inline_tokens::{
    InlineMark, InlineSegment, MarkerVisibility, SegmentKind, TokenizedBlock,
    build_tokenized_block, collect_marker_tokens, raw_offset_to_visible_utf16,
    visible_utf16_to_raw_offset,
};
use crate::interop;
use crate::reveal_engine::{RevealContext, marker_visibility};
use crate::types::{ActiveBlockInputEvent, CursorPosition, NodeType, OwnedAstNode};
use crate::viewport::ViewportNode;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum PendingCaretRestore {
    Raw(usize),
    Visible(usize),
    /// Visible UTF-16 offsets for a non-collapsed selection.
    Selection {
        start: usize,
        end: usize,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct SelectionDetails {
    start: usize,
    end: usize,
    collapsed: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct BeforeInputMeta {
    input_type: String,
    data: String,
    pre_visible_caret_utf16: usize,
    pre_visible_selection_end_utf16: usize,
    is_collapsed: bool,
}

#[derive(Clone, Copy)]
struct InlineNavCtx {
    goal_column: Signal<Option<usize>>,
}

/// Obsidian-style inline markdown editor surface.
///
/// In inline mode, this component renders fully formatted markdown blocks and
/// swaps only the active block into a raw-markdown `<textarea>`.
#[component]
pub fn InlineEditor(
    on_active_block_input: Option<EventHandler<ActiveBlockInputEvent>>,
    on_key_intercept: Option<Callback<String, bool>>,
) -> Element {
    let cursor_ctx = try_use_context::<CursorContext>();
    let cursor_offset = cursor_ctx
        .map(|c| c.cursor_position.read().offset)
        .unwrap_or(0);

    provide_context(InlineNavCtx {
        goal_column: use_signal(|| None),
    });

    let ctx = use_context::<MarkdownContext>();
    let inline_id = ctx.inline_editor_id();

    let parsed = (ctx.parsed_doc)();
    let raw = ctx.raw_value();
    let augmented_ast = inject_gap_paragraphs(&parsed.ast, &raw);

    // Cursor snapping: if cursor is in a gap between blocks, snap to nearest
    use_effect(move || {
        let offset = cursor_ctx
            .map(|c| c.cursor_position.read().offset)
            .unwrap_or(0);
        let parsed = (ctx.parsed_doc)();
        let raw = ctx.raw_value();
        let augmented = inject_gap_paragraphs(&parsed.ast, &raw);
        let snapped = snap_cursor_to_block(&augmented, offset);
        if snapped != offset
            && let Some(mut cctx) = cursor_ctx
        {
            cctx.cursor_position.set(CursorPosition {
                offset: snapped,
                line: 0,
                column: 0,
            });
        }
    });

    rsx! {
        div {
            id: "{inline_id}",
            "data-md-inline-editor": "true",
            "data-state": "active",
            onkeydown: move |evt: KeyboardEvent| {
                let key = evt.key().to_string();
                let ctrl_or_meta = evt.modifiers().ctrl() || evt.modifiers().meta();
                if ctrl_or_meta && (key == "a" || key == "A") {
                    evt.prevent_default();
                    let iid = ctx.inline_editor_id();
                    spawn(async move {
                        interop::eval_void(&select_all_children_js(&iid)).await;
                    });
                }
            },
            div {
                class: "nox-md-viewport",
                "data-md-viewport": "true",
                for node in augmented_ast.into_iter() {
                    InlineBlockNode {
                        node: node.clone(),
                        cursor_offset: cursor_offset,
                        on_active_block_input: on_active_block_input,
                        on_key_intercept: on_key_intercept,
                    }
                }
            }
        }
    }
}

fn is_editable_block(node: &OwnedAstNode) -> bool {
    matches!(
        node.node_type,
        NodeType::Paragraph
            | NodeType::Heading(_)
            | NodeType::BlockQuote
            | NodeType::CodeBlock(_)
            | NodeType::Item
    )
}

#[component]
fn InlineBlockNode(
    node: OwnedAstNode,
    cursor_offset: usize,
    on_active_block_input: Option<EventHandler<ActiveBlockInputEvent>>,
    on_key_intercept: Option<Callback<String, bool>>,
) -> Element {
    let is_active = cursor_offset >= node.range.start && cursor_offset < node.range.end;
    if !is_active {
        return rsx! { InactiveBlockView { node: node } };
    }

    if matches!(node.node_type, NodeType::CodeBlock(_)) {
        rsx! {
            ActiveBlockEditor {
                node: node,
                on_active_block_input: on_active_block_input,
                on_key_intercept: on_key_intercept,
            }
        }
    } else {
        rsx! {
            TokenAwareBlockEditor {
                node: node,
                cursor_offset: cursor_offset,
                on_active_block_input: on_active_block_input,
                on_key_intercept: on_key_intercept,
            }
        }
    }
}

#[cfg(test)]
fn uses_token_aware_surface(node: &OwnedAstNode) -> bool {
    match node.node_type {
        NodeType::CodeBlock(_) => false,
        NodeType::Paragraph | NodeType::Heading(_) | NodeType::BlockQuote | NodeType::Item => {
            block_has_inline_markup(node) || has_block_prefix_marker(node)
        }
        _ => false,
    }
}

#[cfg(test)]
fn has_block_prefix_marker(node: &OwnedAstNode) -> bool {
    matches!(
        node.node_type,
        NodeType::Heading(_) | NodeType::BlockQuote | NodeType::Item
    )
}

#[cfg(test)]
fn block_has_inline_markup(node: &OwnedAstNode) -> bool {
    is_markup_inline_node_type(&node.node_type) || node.children.iter().any(block_has_inline_markup)
}

#[cfg(test)]
fn cursor_within_inline_markup(node: &OwnedAstNode, cursor_offset: usize) -> bool {
    let in_this = is_markup_inline_node_type(&node.node_type)
        && cursor_offset >= node.range.start
        && cursor_offset <= node.range.end;

    if in_this {
        return true;
    }

    node.children
        .iter()
        .any(|child| cursor_within_inline_markup(child, cursor_offset))
}

#[cfg(test)]
fn is_markup_inline_node_type(node_type: &NodeType) -> bool {
    matches!(
        node_type,
        NodeType::Emphasis
            | NodeType::Strong
            | NodeType::Strikethrough
            | NodeType::Code(_)
            | NodeType::Link { .. }
            | NodeType::Image { .. }
            | NodeType::Wikilink(_)
            | NodeType::Tag(_)
    )
}

#[component]
fn InactiveBlockView(node: OwnedAstNode) -> Element {
    let ctx = use_context::<MarkdownContext>();
    let cursor_ctx = try_use_context::<CursorContext>();
    let block_id = format!("nox-md-inline-block-{}", node.range.start);
    let safe_start = node.range.start;
    let safe_end = node.range.end;

    match &node.node_type {
        // Keep list structure valid: ul/ol children must be li.
        NodeType::Item => {
            let block_id_for_click = block_id.clone();
            let node_for_click = node.clone();
            let node_for_render = node.clone();
            rsx! {
                li {
                    id: "{block_id}",
                    "data-md-inline-block": "true",
                    onclick: move |_| {
                        handle_inactive_block_click(
                            cursor_ctx,
                            ctx.raw_value(),
                            node_for_click.clone(),
                            block_id_for_click.clone(),
                            safe_start,
                            safe_end,
                        );
                    },
                    for child in node_for_render.children {
                        ViewportNode {
                            node: child,
                            overrides: vec![]
                        }
                    }
                }
            }
        }
        _ => {
            let is_blank_line =
                matches!(node.node_type, NodeType::Paragraph) && node.children.is_empty();
            let block_id_for_click = block_id.clone();
            let node_for_click = node.clone();
            let node_for_render = node.clone();
            rsx! {
                div {
                    id: "{block_id}",
                    "data-md-inline-block": "true",
                    "data-md-blank-line": if is_blank_line { Some("true") } else { None },
                    onclick: move |_| {
                        handle_inactive_block_click(
                            cursor_ctx,
                            ctx.raw_value(),
                            node_for_click.clone(),
                            block_id_for_click.clone(),
                            safe_start,
                            safe_end,
                        );
                    },
                    if is_blank_line {
                        p { br {} }
                    } else {
                        ViewportNode {
                            node: node_for_render,
                            overrides: vec![]
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn TokenAwareBlockEditor(
    node: OwnedAstNode,
    cursor_offset: usize,
    on_active_block_input: Option<EventHandler<ActiveBlockInputEvent>>,
    on_key_intercept: Option<Callback<String, bool>>,
) -> Element {
    let ctx = use_context::<MarkdownContext>();
    let cursor_ctx = try_use_context::<CursorContext>();
    let nav_ctx = try_use_context::<InlineNavCtx>();
    let raw = ctx.raw_value();

    let node_end = node.range.end.min(raw.len());
    let safe_start = node.range.start.min(node_end);
    let safe_end = trim_editable_block_end(&raw, safe_start, node_end);
    let local_cursor = cursor_offset
        .saturating_sub(safe_start)
        .min(last_caret_offset(&(0..safe_end.saturating_sub(safe_start))));
    let block_id = format!("nox-md-token-{}", safe_start);
    let current_len = use_signal(|| safe_end.saturating_sub(safe_start));
    let mut is_composing = use_signal(|| false);
    let mut input_revision = use_signal(|| 0u64);
    let applied_revision = use_signal(|| 0u64);
    let mut caret_generation = use_signal(|| 0u64);
    let restore_generation = use_signal(|| 0u64);
    let mut pending_restore_raw = use_signal(|| None::<PendingCaretRestore>);
    let block_id_input = block_id.clone();
    let block_id_comp_end = block_id.clone();
    let block_id_nav = block_id.clone();
    let block_id_enter = block_id.clone();
    let block_id_keyup = block_id.clone();
    let block_id_mouseup = block_id.clone();
    let block_id_mount = block_id.clone();
    let block_id_effect = block_id.clone();

    let block_raw = raw[safe_start..safe_end].to_string();

    // Empty block (synthetic gap paragraph): render minimal contenteditable with <br> placeholder
    if block_raw.is_empty() {
        let block_id_empty = block_id.clone();
        let block_id_mount_empty = block_id.clone();
        let empty_view = rsx! {
            div {
                id: "{block_id}",
                "data-md-token-editor": "true",
                "data-md-empty-block": "true",
                contenteditable: "true",
                style: "width:100%;min-width:100%;max-width:100%;box-sizing:border-box;outline:none;white-space:pre-wrap;word-break:break-word;",
                onkeydown: move |evt: KeyboardEvent| {
                    let key = evt.key().to_string();
                    if key == "Backspace" {
                        evt.prevent_default();
                        perform_block_join(ctx, cursor_ctx, safe_start);
                        return;
                    }
                    if key == "Enter" && !evt.modifiers().shift() {
                        evt.prevent_default();
                        let mut len_enter = current_len;
                        perform_block_split(
                            ctx,
                            cursor_ctx,
                            safe_start,
                            &mut len_enter,
                            0,
                        );
                        return;
                    }
                    // Arrow navigation for empty blocks
                    if key == "ArrowUp" || key == "ArrowDown" {
                        evt.prevent_default();
                        if let Some(mut cctx) = cursor_ctx {
                            let parsed = (ctx.parsed_doc)();
                            let raw_nav = ctx.raw_value();
                            let augmented = inject_gap_paragraphs(&parsed.ast, &raw_nav);
                            let direction = if key == "ArrowUp" {
                                NavDirection::Prev
                            } else {
                                NavDirection::Next
                            };
                            let mut nodes = Vec::new();
                            collect_editable_nodes(&augmented, &mut nodes);
                            let target = match direction {
                                NavDirection::Prev => nodes
                                    .iter()
                                    .rev()
                                    .find(|n| n.range.start < safe_start)
                                    .map(|n| n.range.start),
                                NavDirection::Next => nodes
                                    .iter()
                                    .find(|n| n.range.start > safe_start)
                                    .map(|n| n.range.start),
                            };
                            if let Some(target_offset) = target {
                                cctx.cursor_position.set(CursorPosition {
                                    offset: target_offset,
                                    line: 0,
                                    column: 0,
                                });
                            }
                        }
                    }
                },
                oninput: move |_| {
                    // When user types into empty block, read the input and update raw content
                    let block_id_inp = block_id_empty.clone();
                    let cursor_ctx_local = cursor_ctx;
                    let mut len_sig = current_len;
                    spawn(async move {
                        let new_text = {
                            let js = interop::caret_adapter().read_contenteditable_text_js(&block_id_inp);
                            let mut eval = interop::start_eval(&js);
                            interop::recv_string(&mut eval).await.unwrap_or_default()
                        };
                        let current_global = ctx.raw_value();
                        let start = safe_start.min(current_global.len());
                        let old_len = *len_sig.read();
                        let end = (start + old_len).min(current_global.len());
                        let rebuilt = format!(
                            "{}{}{}",
                            &current_global[..start],
                            new_text,
                            &current_global[end..]
                        );
                        len_sig.set(new_text.len());
                        ctx.handle_value_change(rebuilt);
                        ctx.trigger_parse.call(());
                        if let Some(mut cctx) = cursor_ctx_local {
                            cctx.cursor_position.set(CursorPosition {
                                offset: start + new_text.len(),
                                line: 0,
                                column: 0,
                            });
                        }
                    });
                },
                onmounted: move |_| {
                    let set_js = interop::caret_adapter()
                        .set_contenteditable_selection_js(&block_id_mount_empty, 0);
                    spawn(async move {
                        interop::eval_void(&set_js).await;
                    });
                },
                br {}
            }
        };

        return match &node.node_type {
            NodeType::Heading(1) => rsx! { h1 { {empty_view} } },
            NodeType::Heading(2) => rsx! { h2 { {empty_view} } },
            NodeType::Heading(3) => rsx! { h3 { {empty_view} } },
            NodeType::Heading(4) => rsx! { h4 { {empty_view} } },
            NodeType::Heading(5) => rsx! { h5 { {empty_view} } },
            NodeType::Heading(6) => rsx! { h6 { {empty_view} } },
            NodeType::BlockQuote => rsx! { blockquote { {empty_view} } },
            NodeType::Item => rsx! { li { {empty_view} } },
            _ => rsx! { p { {empty_view} } },
        };
    }

    let mut model_node = node.clone();
    model_node.range = safe_start..safe_end;
    let marker_tokens = collect_marker_tokens(&model_node, &block_raw, safe_start);
    let visibility_flags = marker_visibility(
        &marker_tokens,
        RevealContext {
            caret_raw_offset: local_cursor,
            selection: None,
        },
    );
    let visibility = visibility_flags
        .iter()
        .enumerate()
        .map(|(idx, visible)| MarkerVisibility {
            marker_idx: idx,
            visible: *visible,
        })
        .collect::<Vec<_>>();
    let model = build_tokenized_block(&model_node, &raw, &visibility);
    let model_for_input = model.clone();
    let model_for_comp_end = model.clone();
    let model_for_effect = model.clone();
    let model_for_nav = model.clone();
    let model_for_enter = model.clone();
    let model_for_keyup = model.clone();
    let model_for_mouseup = model.clone();
    let node_for_input = model_node.clone();
    let node_for_comp_end = node.clone();
    let target_visible_cursor = raw_offset_to_visible_utf16(&model, local_cursor);
    let target_visible_cursor_mount = target_visible_cursor;
    let inline_input_cursor = byte_to_utf16_index(&model.raw_text, local_cursor).unwrap_or(0);
    let visible_input_cursor = target_visible_cursor;
    let inline_visible_text = model.visible_text.clone();
    let inline_raw_text = model.raw_text.clone();
    let is_single_line_block = !model.raw_text.contains('\n');
    let pending_restore_for_keyup = pending_restore_raw;
    let pending_restore_for_mouseup = pending_restore_raw;
    let pending_restore_for_nav = pending_restore_raw;
    let caret_generation_for_keyup = caret_generation;
    let caret_generation_for_mouseup = caret_generation;
    let caret_generation_for_nav = caret_generation;

    use_effect(move || {
        let pending = *pending_restore_raw.read();
        if is_composing() {
            return;
        }
        let Some(pending) = pending else {
            return;
        };

        let js = match pending {
            PendingCaretRestore::Raw(abs_raw) => {
                let local_raw = abs_raw
                    .saturating_sub(safe_start)
                    .min(last_caret_offset(&(0..model_for_effect.raw_text.len())));
                let v = raw_offset_to_visible_utf16(&model_for_effect, local_raw);
                interop::caret_adapter().set_contenteditable_selection_js(&block_id_effect, v)
            }
            PendingCaretRestore::Visible(visible) => {
                let v = visible.min(utf16_len(&model_for_effect.visible_text));
                interop::caret_adapter().set_contenteditable_selection_js(&block_id_effect, v)
            }
            PendingCaretRestore::Selection { start, end } => interop::caret_adapter()
                .set_contenteditable_selection_range_js(&block_id_effect, start, end),
        };
        pending_restore_raw.set(None);
        let restore_gen = restore_generation();
        let restore_gen_sig = restore_generation;
        spawn(async move {
            // If oninput/oncompositionend fired since this restore was queued,
            // it is stale — skip to prevent overriding the correct cursor.
            if *restore_gen_sig.read() != restore_gen {
                return;
            }
            interop::eval_void(&js).await;
        });
    });

    let token_view = rsx! {
        div {
            id: "{block_id}",
            "data-md-token-editor": "true",
            contenteditable: "true",
            style: "width:100%;min-width:100%;max-width:100%;box-sizing:border-box;outline:none;white-space:pre-wrap;word-break:break-word;",
            onkeydown: move |evt: KeyboardEvent| {
                let key = evt.key().to_string();
                // ── Key intercept (suggestion popover) ────────────────────
                if let Some(ref interceptor) = on_key_intercept
                    && interceptor.call(key.clone())
                {
                    evt.prevent_default();
                    evt.stop_propagation();
                    return;
                }
                // ── Enter key: split block ──
                if key == "Enter" && !evt.modifiers().shift() {
                    evt.prevent_default();
                    if let Some(nc) = nav_ctx {
                        let mut gc = nc.goal_column;
                        gc.set(None);
                    }
                    let block_id_enter = block_id_enter.clone();
                    let model_enter = model_for_enter.clone();
                    let mut len_enter = current_len;
                    let cursor_ctx_enter = cursor_ctx;
                    spawn(async move {
                        let visible_now = {
                            let js = interop::caret_adapter()
                                .read_contenteditable_selection_js(&block_id_enter);
                            let mut eval = interop::start_eval(&js);
                            interop::recv_string(&mut eval)
                                .await
                                .and_then(|s| s.parse::<usize>().ok())
                                .unwrap_or(0)
                        };
                        let local_raw = visible_utf16_to_raw_offset(&model_enter, visible_now)
                            .min(last_caret_offset(&(0..model_enter.raw_text.len())));
                        perform_block_split(
                            ctx,
                            cursor_ctx_enter,
                            safe_start,
                            &mut len_enter,
                            local_raw,
                        );
                    });
                    return;
                }
                // ── Backspace at start of block: join with previous ──
                if key == "Backspace" && target_visible_cursor == 0 && safe_start > 0 {
                    evt.prevent_default();
                    if let Some(nc) = nav_ctx {
                        let mut gc = nc.goal_column;
                        gc.set(None);
                    }
                    perform_block_join(ctx, cursor_ctx, safe_start);
                    return;
                }
                if is_single_line_block && (key == "ArrowUp" || key == "ArrowDown") {
                    evt.prevent_default();
                    if let Some(mut cctx) = cursor_ctx {
                        let parsed = (ctx.parsed_doc)();
                        let direction = if key == "ArrowUp" {
                            NavDirection::Prev
                        } else {
                            NavDirection::Next
                        };
                        let block_id_vert = block_id_nav.clone();
                        let raw_vert = ctx.raw_value();
                        spawn(async move {
                            let visible_now = {
                                let js = interop::caret_adapter()
                                    .read_contenteditable_selection_js(&block_id_vert);
                                let mut eval = interop::start_eval(&js);
                                interop::recv_string(&mut eval)
                                    .await
                                    .and_then(|s| s.parse::<usize>().ok())
                                    .unwrap_or(0)
                            };
                            let goal_col = if let Some(nc) = nav_ctx {
                                if let Some(gc) = (nc.goal_column)() {
                                    gc
                                } else {
                                    let mut gc_sig = nc.goal_column;
                                    gc_sig.set(Some(visible_now));
                                    visible_now
                                }
                            } else {
                                visible_now
                            };
                            if let Some(target_node) = adjacent_editable_node(
                                &parsed.ast,
                                safe_start,
                                direction,
                            ) {
                                let t_start = target_node.range.start.min(raw_vert.len());
                                let t_end = target_node.range.end.min(raw_vert.len());
                                let clamped = resolve_visible_column_in_block(
                                    &target_node,
                                    &raw_vert,
                                    t_start,
                                    t_end,
                                    goal_col,
                                );
                                cctx.cursor_position.set(CursorPosition {
                                    offset: clamped,
                                    line: 0,
                                    column: 0,
                                });
                            }
                        });
                    }
                }
                if key == "ArrowLeft" || key == "ArrowRight" {
                    evt.prevent_default();
                    if let Some(nc) = nav_ctx {
                        let mut gc = nc.goal_column;
                        gc.set(None);
                    }
                    if let Some(mut cctx) = cursor_ctx {
                        let block_id = block_id_nav.clone();
                        let model = model_for_nav.clone();
                        let mut pending_restore = pending_restore_for_nav;
                        let generation = caret_generation_for_nav();
                        let generation_sig = caret_generation_for_nav;
                        spawn(async move {
                            let visible_now = {
                                let js =
                                    interop::caret_adapter().read_contenteditable_selection_js(&block_id);
                                let mut eval = interop::start_eval(&js);
                                interop::recv_string(&mut eval)
                                    .await
                                    .and_then(|s| s.parse::<usize>().ok())
                                    .unwrap_or(0)
                            };
                            if !is_latest_revision(generation, *generation_sig.read()) {
                                return;
                            }
                            let max_visible = utf16_len(&model.visible_text);
                            let target_visible = if key == "ArrowLeft" {
                                visible_now.saturating_sub(1)
                            } else {
                                visible_now.saturating_add(1).min(max_visible)
                            };
                            let local_raw = visible_utf16_to_raw_offset(&model, target_visible)
                                .min(last_caret_offset(&(0..model.raw_text.len())));
                            let abs_raw = safe_start.saturating_add(local_raw);
                            cctx.cursor_position.set(CursorPosition {
                                offset: abs_raw,
                                line: 0,
                                column: 0,
                            });
                            pending_restore.set(Some(PendingCaretRestore::Raw(abs_raw)));
                        });
                    }
                }
            },
            oninput: move |_| {
                if is_composing() {
                    return;
                }
                if let Some(nc) = nav_ctx {
                    let mut gc = nc.goal_column;
                    gc.set(None);
                }
                // Drop stale restore requests from earlier click/nav events.
                // Text mutations own caret placement via the input pipeline.
                pending_restore_raw.set(None);
                // Bump restore_generation so any in-flight use_effect spawn that
                // already captured the old generation will self-cancel.
                { let mut rg = restore_generation; rg.set(rg().saturating_add(1)); }
                let next_generation = caret_generation().saturating_add(1);
                caret_generation.set(next_generation);
                let next_revision = input_revision().saturating_add(1);
                input_revision.set(next_revision);
                let model = model_for_input.clone();
                let block_id = block_id_input.clone();
                let cursor_ctx_local = cursor_ctx;
                let handler = on_active_block_input;
                let node_local = node_for_input.clone();
                let len_sig = current_len;
                spawn_token_editor_sync(
                    block_id,
                    model,
                    ctx,
                    safe_start,
                    node_local,
                    len_sig,
                    cursor_ctx_local,
                    handler,
                    next_revision,
                    input_revision,
                    applied_revision,
                    pending_restore_raw,
                );
            },
            oncompositionstart: move |_| {
                is_composing.set(true);
            },
            oncompositionend: move |_| {
                is_composing.set(false);
                pending_restore_raw.set(None);
                // Bump restore_generation so any in-flight use_effect spawn that
                // already captured the old generation will self-cancel.
                { let mut rg = restore_generation; rg.set(rg().saturating_add(1)); }
                let next_generation = caret_generation().saturating_add(1);
                caret_generation.set(next_generation);
                let next_revision = input_revision().saturating_add(1);
                input_revision.set(next_revision);
                let model = model_for_comp_end.clone();
                let block_id = block_id_comp_end.clone();
                let cursor_ctx_local = cursor_ctx;
                let handler = on_active_block_input;
                let mut node_local = node_for_comp_end.clone();
                node_local.range = safe_start..safe_end;
                let len_sig = current_len;
                spawn_token_editor_sync(
                    block_id,
                    model,
                    ctx,
                    safe_start,
                    node_local,
                    len_sig,
                    cursor_ctx_local,
                    handler,
                    next_revision,
                    input_revision,
                    applied_revision,
                    pending_restore_raw,
                );
            },
            onkeyup: move |evt: KeyboardEvent| {
                let key = evt.key().to_string();
                if !is_navigation_key(&key) {
                    return;
                }
                if key == "ArrowLeft" || key == "ArrowRight" {
                    return;
                }
                if is_single_line_block && (key == "ArrowUp" || key == "ArrowDown") {
                    return;
                }
                if let Some(mut cctx) = cursor_ctx {
                    let block_id = block_id_keyup.clone();
                    let model = model_for_keyup.clone();
                    let mut pending_restore = pending_restore_for_keyup;
                    let generation = caret_generation_for_keyup();
                    let generation_sig = caret_generation_for_keyup;
                    spawn(async move {
                        let cursor_visible_utf16 = {
                            let js = interop::caret_adapter()
                                .read_contenteditable_selection_js(&block_id);
                            let mut eval = interop::start_eval(&js);
                            interop::recv_string(&mut eval)
                                .await
                                .and_then(|s| s.parse::<usize>().ok())
                                .unwrap_or(0)
                        };
                        if !is_latest_revision(generation, *generation_sig.read()) {
                            return;
                        }
                        let local_raw = visible_utf16_to_raw_offset(&model, cursor_visible_utf16)
                            .min(last_caret_offset(&(0..model.raw_text.len())));
                        let abs_raw = safe_start.saturating_add(local_raw);
                        cctx.cursor_position.set(CursorPosition {
                            offset: abs_raw,
                            line: 0,
                            column: 0,
                        });
                        pending_restore.set(Some(PendingCaretRestore::Raw(abs_raw)));
                    });
                }
            },
            onmouseup: move |_| {
                if let Some(nc) = nav_ctx {
                    let mut gc = nc.goal_column;
                    gc.set(None);
                }
                if let Some(mut cctx) = cursor_ctx {
                    let block_id = block_id_mouseup.clone();
                    let model = model_for_mouseup.clone();
                    let mut pending_restore = pending_restore_for_mouseup;
                    let generation = caret_generation_for_mouseup();
                    let generation_sig = caret_generation_for_mouseup;
                    spawn(async move {
                        let sel = {
                            let js = interop::caret_adapter()
                                .read_contenteditable_selection_detailed_js(&block_id);
                            let mut eval = interop::start_eval(&js);
                            interop::recv_string(&mut eval)
                                .await
                                .and_then(|s| parse_selection_details(&s))
                        };
                        if !is_latest_revision(generation, *generation_sig.read()) {
                            return;
                        }
                        let (cursor_visible_utf16, restore) = match sel {
                            Some(ref d) if !d.collapsed => {
                                // Non-collapsed: restore the full range; cursor context = end.
                                (
                                    d.end,
                                    PendingCaretRestore::Selection {
                                        start: d.start,
                                        end: d.end,
                                    },
                                )
                            }
                            Some(ref d) => {
                                let local_raw = visible_utf16_to_raw_offset(&model, d.end)
                                    .min(last_caret_offset(&(0..model.raw_text.len())));
                                let abs = safe_start.saturating_add(local_raw);
                                (d.end, PendingCaretRestore::Raw(abs))
                            }
                            None => (0, PendingCaretRestore::Raw(safe_start)),
                        };
                        let local_raw = visible_utf16_to_raw_offset(&model, cursor_visible_utf16)
                            .min(last_caret_offset(&(0..model.raw_text.len())));
                        let abs_raw = safe_start.saturating_add(local_raw);
                        cctx.cursor_position.set(CursorPosition {
                            offset: abs_raw,
                            line: 0,
                            column: 0,
                        });
                        pending_restore.set(Some(restore));
                    });
                }
            },
            onmounted: move |_| {
                let mount_target = nav_ctx
                    .and_then(|nc| {
                        (nc.goal_column)()
                            .map(|gc| gc.min(utf16_len(&inline_visible_text)))
                    })
                    .unwrap_or(target_visible_cursor_mount);
                let set_js = interop::caret_adapter()
                    .set_contenteditable_selection_js(&block_id_mount, mount_target);
                let bind_js = interop::caret_adapter().bind_contenteditable_input_js(&block_id_mount);
                spawn(async move {
                    interop::eval_void(&set_js).await;
                    interop::eval_void(&bind_js).await;
                });
                if let Some(handler) = on_active_block_input {
                    handler.call(ActiveBlockInputEvent {
                        raw_text: inline_raw_text.clone(),
                        visible_text: inline_visible_text.clone(),
                        cursor_raw_utf16: inline_input_cursor,
                        cursor_visible_utf16: visible_input_cursor,
                        block_start: safe_start,
                        block_end: safe_end,
                    });
                }
            },
            for seg in model.segments.clone() {
                { render_inline_segment(seg) }
            }
        }
    };

    match &node.node_type {
        NodeType::Heading(1) => rsx! { h1 { {token_view} } },
        NodeType::Heading(2) => rsx! { h2 { {token_view} } },
        NodeType::Heading(3) => rsx! { h3 { {token_view} } },
        NodeType::Heading(4) => rsx! { h4 { {token_view} } },
        NodeType::Heading(5) => rsx! { h5 { {token_view} } },
        NodeType::Heading(6) => rsx! { h6 { {token_view} } },
        NodeType::BlockQuote => rsx! { blockquote { {token_view} } },
        NodeType::Item => rsx! { li { {token_view} } },
        _ => rsx! { p { {token_view} } },
    }
}

/// Walk `visible_text` chars to find the UTF-16 position of the start of
/// the character that ends at `caret_vis_utf16`. Returns `None` if already at
/// position 0 (nothing to delete backwards).
fn previous_visible_char_utf16(visible_text: &str, caret_vis_utf16: usize) -> Option<usize> {
    if caret_vis_utf16 == 0 {
        return None;
    }
    let mut pos = 0usize;
    for ch in visible_text.chars() {
        let prev_pos = pos;
        pos += ch.len_utf16();
        if pos >= caret_vis_utf16 {
            return Some(prev_pos);
        }
    }
    None
}

/// Walk `visible_text` chars to find the UTF-16 position after the character
/// starting at `caret_vis_utf16`. Returns `None` if at or past end.
fn next_visible_char_utf16(visible_text: &str, caret_vis_utf16: usize) -> Option<usize> {
    let mut pos = 0usize;
    for ch in visible_text.chars() {
        if pos == caret_vis_utf16 {
            return Some(pos + ch.len_utf16());
        }
        pos += ch.len_utf16();
        if pos > caret_vis_utf16 {
            // caret was in the middle of a multi-unit char — treat as this char end
            return Some(pos);
        }
    }
    None
}

/// Given a visible character spanning `[vis_start..vis_end)`, find the raw
/// byte range it maps to. Resolves segment-boundary ambiguity by requiring
/// the character to be fully contained within a single segment.
fn visible_char_raw_range(
    model: &TokenizedBlock,
    vis_start: usize,
    vis_end: usize,
) -> Option<(usize, usize)> {
    for seg in &model.segments {
        if vis_start >= seg.visible_utf16_start && vis_end <= seg.visible_utf16_end {
            let local_start = vis_start - seg.visible_utf16_start;
            let local_end = vis_end - seg.visible_utf16_start;
            let byte_start = utf16_to_byte_index(&seg.text, local_start)?;
            let byte_end = utf16_to_byte_index(&seg.text, local_end)?;
            return Some((
                seg.raw_range.start + byte_start,
                seg.raw_range.start + byte_end,
            ));
        }
    }
    None
}

/// For single-character collapsed deletions (`deleteContentBackward` /
/// `deleteContentForward`), compute the exact raw byte range to delete
/// using the render-time model and pre-edit caret position.
///
/// Returns `Some((raw_del_start, raw_del_end, new_cursor_vis_utf16))` on
/// success, or `None` if this edit type is not handled (falls through to
/// the existing diff path).
fn direct_delete_from_beforeinput(
    meta: &BeforeInputMeta,
    model: &TokenizedBlock,
    block_raw: &str,
) -> Option<(usize, usize, usize)> {
    if !meta.is_collapsed {
        return None;
    }
    let caret = meta.pre_visible_caret_utf16;

    match meta.input_type.as_str() {
        "deleteContentBackward" => {
            let prev = previous_visible_char_utf16(&model.visible_text, caret)?;
            let (raw_start, raw_end) = visible_char_raw_range(model, prev, caret)?;
            if raw_start >= raw_end || raw_end > block_raw.len() {
                return None;
            }
            Some((raw_start, raw_end, prev))
        }
        "deleteContentForward" => {
            let next = next_visible_char_utf16(&model.visible_text, caret)?;
            let (raw_start, raw_end) = visible_char_raw_range(model, caret, next)?;
            if raw_start >= raw_end || raw_end > block_raw.len() {
                return None;
            }
            Some((raw_start, raw_end, caret))
        }
        _ => None,
    }
}

#[allow(clippy::too_many_arguments)]
fn spawn_token_editor_sync(
    block_id: String,
    model: TokenizedBlock,
    ctx: MarkdownContext,
    safe_start: usize,
    node_local: OwnedAstNode,
    mut len_sig: Signal<usize>,
    cursor_ctx_local: Option<CursorContext>,
    handler: Option<EventHandler<ActiveBlockInputEvent>>,
    captured_revision: u64,
    latest_revision: Signal<u64>,
    mut applied_revision: Signal<u64>,
    mut pending_restore: Signal<Option<PendingCaretRestore>>,
) {
    spawn(async move {
        let new_visible = {
            let js = interop::caret_adapter().read_contenteditable_text_js(&block_id);
            let mut eval = interop::start_eval(&js);
            interop::recv_string(&mut eval).await.unwrap_or_default()
        };
        let selection_details = {
            let js = interop::caret_adapter().read_contenteditable_selection_detailed_js(&block_id);
            let mut eval = interop::start_eval(&js);
            interop::recv_string(&mut eval)
                .await
                .and_then(|s| parse_selection_details(&s))
        };
        let cursor_visible_utf16 = selection_details.as_ref().map_or(0, |s| s.start);
        let before_input_meta = {
            let js = interop::caret_adapter().read_contenteditable_beforeinput_meta_js(&block_id);
            let mut eval = interop::start_eval(&js);
            interop::recv_string(&mut eval)
                .await
                .and_then(|s| parse_before_input_meta(&s))
        };

        // If a newer oninput has fired since this sync was spawned, bail out —
        // we are stale and would stomp the correct cursor position.
        if *latest_revision.read() != captured_revision {
            return;
        }

        let current_global = ctx.raw_value();
        let start = safe_start.min(current_global.len());
        let old_len = *len_sig.read();
        let end = (start + old_len).min(current_global.len());
        let block_raw_current = current_global[start..end].to_string();

        // ── Direct-delete fast path ──────────────────────────────────
        // For single-character collapsed deletions, bypass DOM text diffing
        // entirely: use the render-time model + beforeinput caret to compute
        // the exact raw byte range.
        if let Some(ref meta) = before_input_meta
            && let Some((raw_del_start, raw_del_end, new_cursor_vis)) =
                direct_delete_from_beforeinput(meta, &model, &block_raw_current)
        {
            let rebuilt_local = format!(
                "{}{}",
                &block_raw_current[..raw_del_start],
                &block_raw_current[raw_del_end..],
            );
            let rebuilt_global = format!(
                "{}{}{}",
                &current_global[..start],
                rebuilt_local,
                &current_global[end..],
            );
            len_sig.set(rebuilt_local.len());
            ctx.handle_value_change(rebuilt_global.clone());
            ctx.trigger_parse.call(());
            applied_revision.set(captured_revision);

            let raw_cursor_local = raw_del_start.min(last_caret_offset(&(0..rebuilt_local.len())));

            if let Some(mut cctx) = cursor_ctx_local {
                cctx.cursor_position.set(CursorPosition {
                    offset: start.saturating_add(raw_cursor_local),
                    line: 0,
                    column: 0,
                });
            }
            pending_restore.set(Some(PendingCaretRestore::Visible(new_cursor_vis)));

            if let Some(handler) = handler {
                let mut fresh_node = node_local.clone();
                fresh_node.range = start..start.saturating_add(rebuilt_local.len());
                let fresh_tokens = collect_marker_tokens(&fresh_node, &rebuilt_local, start);
                let fresh_visibility_flags = marker_visibility(
                    &fresh_tokens,
                    RevealContext {
                        caret_raw_offset: raw_cursor_local,
                        selection: None,
                    },
                );
                let fresh_visibility = fresh_visibility_flags
                    .iter()
                    .enumerate()
                    .map(|(idx, visible)| MarkerVisibility {
                        marker_idx: idx,
                        visible: *visible,
                    })
                    .collect::<Vec<_>>();
                let fresh_model =
                    build_tokenized_block(&fresh_node, &rebuilt_global, &fresh_visibility);
                handler.call(ActiveBlockInputEvent {
                    raw_text: fresh_model.raw_text.clone(),
                    visible_text: fresh_model.visible_text.clone(),
                    cursor_raw_utf16: byte_to_utf16_index(&fresh_model.raw_text, raw_cursor_local)
                        .unwrap_or(0),
                    cursor_visible_utf16: new_cursor_vis,
                    block_start: start,
                    block_end: start.saturating_add(fresh_model.raw_text.len()),
                });
            }
            return;
        }

        let mut candidate_models = vec![
            model.clone(),
            build_plain_text_model(&block_raw_current, start),
        ];
        let mut current_node = node_local.clone();
        current_node.range = start..end;
        let current_markers = collect_marker_tokens(&current_node, &block_raw_current, start);
        if !current_markers.is_empty() {
            let hidden_visibility = current_markers
                .iter()
                .enumerate()
                .map(|(idx, _)| MarkerVisibility {
                    marker_idx: idx,
                    visible: false,
                })
                .collect::<Vec<_>>();
            candidate_models.push(build_tokenized_block(
                &current_node,
                &current_global,
                &hidden_visibility,
            ));

            let visible_visibility = current_markers
                .iter()
                .enumerate()
                .map(|(idx, _)| MarkerVisibility {
                    marker_idx: idx,
                    visible: true,
                })
                .collect::<Vec<_>>();
            candidate_models.push(build_tokenized_block(
                &current_node,
                &current_global,
                &visible_visibility,
            ));
        }

        let (model_idx, edit) =
            select_best_input_projection(&candidate_models, &new_visible, cursor_visible_utf16);
        let selected_model = &candidate_models[model_idx];
        let effective_cursor_visible = compute_post_visible_caret(
            before_input_meta.as_ref(),
            &edit,
            cursor_visible_utf16,
            utf16_len(&new_visible),
        );
        let old_raw_start = visible_utf16_to_raw_offset(selected_model, edit.old_start_utf16)
            .min(block_raw_current.len());
        let old_raw_end = visible_utf16_to_raw_offset(selected_model, edit.old_end_utf16)
            .min(block_raw_current.len());
        let rebuilt_local = format!(
            "{}{}{}",
            &block_raw_current[..old_raw_start],
            edit.replacement,
            &block_raw_current[old_raw_end..]
        );
        let rebuilt_global = format!(
            "{}{}{}",
            &current_global[..start],
            rebuilt_local,
            &current_global[end..]
        );
        len_sig.set(rebuilt_local.len());
        ctx.handle_value_change(rebuilt_global.clone());
        ctx.trigger_parse.call(());
        applied_revision.set(captured_revision);

        let raw_cursor_local = cursor_after_visible_edit(
            selected_model,
            effective_cursor_visible,
            &edit,
            old_raw_start,
            old_raw_end,
        )
        .min(last_caret_offset(&(0..rebuilt_local.len())));

        if let Some(mut cctx) = cursor_ctx_local {
            cctx.cursor_position.set(CursorPosition {
                offset: start.saturating_add(raw_cursor_local),
                line: 0,
                column: 0,
            });
        }
        pending_restore.set(Some(PendingCaretRestore::Visible(effective_cursor_visible)));

        if let Some(handler) = handler {
            let mut fresh_node = node_local.clone();
            fresh_node.range = start..start.saturating_add(rebuilt_local.len());
            let fresh_tokens = collect_marker_tokens(&fresh_node, &rebuilt_local, start);
            let fresh_visibility_flags = marker_visibility(
                &fresh_tokens,
                RevealContext {
                    caret_raw_offset: raw_cursor_local,
                    selection: None,
                },
            );
            let fresh_visibility = fresh_visibility_flags
                .iter()
                .enumerate()
                .map(|(idx, visible)| MarkerVisibility {
                    marker_idx: idx,
                    visible: *visible,
                })
                .collect::<Vec<_>>();
            let fresh_model =
                build_tokenized_block(&fresh_node, &rebuilt_global, &fresh_visibility);
            handler.call(ActiveBlockInputEvent {
                raw_text: fresh_model.raw_text.clone(),
                visible_text: fresh_model.visible_text.clone(),
                cursor_raw_utf16: byte_to_utf16_index(&fresh_model.raw_text, raw_cursor_local)
                    .unwrap_or(0),
                cursor_visible_utf16: effective_cursor_visible,
                block_start: start,
                block_end: start.saturating_add(fresh_model.raw_text.len()),
            });
        }
    });
}

#[component]
fn ActiveBlockEditor(
    node: OwnedAstNode,
    on_active_block_input: Option<EventHandler<ActiveBlockInputEvent>>,
    on_key_intercept: Option<Callback<String, bool>>,
) -> Element {
    let ctx = use_context::<MarkdownContext>();
    let cursor_ctx = try_use_context::<CursorContext>();
    let raw = ctx.raw_value();

    let safe_end = node.range.end.min(raw.len());
    let safe_start = node.range.start.min(safe_end);
    let initial_text = raw[safe_start..safe_end].trim_end_matches('\n').to_string();
    let block_id = format!("nox-md-active-{}", safe_start);
    let mut current_len = use_signal(|| initial_text.len());

    let target_cursor = cursor_ctx
        .map(|c| c.cursor_position.read().offset.saturating_sub(safe_start))
        .unwrap_or(0);
    let block_id_input = block_id.clone();
    let block_id_keyup = block_id.clone();
    let block_id_mouseup = block_id.clone();
    let block_id_mount = block_id.clone();

    let wrapper = match &node.node_type {
        NodeType::Heading(1) => "h1",
        NodeType::Heading(2) => "h2",
        NodeType::Heading(3) => "h3",
        NodeType::Heading(4) => "h4",
        NodeType::Heading(5) => "h5",
        NodeType::Heading(6) => "h6",
        NodeType::BlockQuote => "blockquote",
        NodeType::CodeBlock(_) => "pre",
        NodeType::Item => "li",
        _ => "p",
    };

    let input_view = rsx! {
        textarea {
            id: "{block_id}",
            "data-md-active-block-editor": "true",
            rows: "1",
            // Ensure active raw editing matches the rendered block width.
            // Without an explicit width, browser default textarea cols can collapse
            // to a narrow measure and cause visual line-wrap jumps.
            style: "width:100%;min-width:100%;max-width:100%;box-sizing:border-box;resize:none;overflow:hidden;font:inherit;color:inherit;background:transparent;border:none;margin:0;padding:0;outline:none;line-height:inherit;display:block;",
            initial_value: "{initial_text}",
            onkeydown: move |evt: KeyboardEvent| {
                if let Some(ref interceptor) = on_key_intercept
                    && interceptor.call(evt.key().to_string())
                {
                    evt.prevent_default();
                    evt.stop_propagation();
                }
            },
            oninput: move |evt: FormEvent| {
                let new_local = evt.value();
                let current_global = ctx.raw_value();
                let start = safe_start.min(current_global.len());
                let old_len = *current_len.read();
                let end = (start + old_len).min(current_global.len());
                let before = &current_global[..start];
                let after = &current_global[end..];
                let new_global = format!("{before}{new_local}{after}");
                current_len.set(new_local.len());
                ctx.handle_value_change(new_global);
                ctx.trigger_parse.call(());

                if cursor_ctx.is_some() || on_active_block_input.is_some() {
                    let block_id = block_id_input.clone();
                    let text_clone = new_local.clone();
                    let block_idx = safe_start;
                    let cursor_ctx_local = cursor_ctx;
                    let handler = on_active_block_input;
                    spawn(async move {
                        let cursor_utf16 = {
                            let js = interop::caret_adapter().read_textarea_cursor_js(&block_id);
                            let mut eval = interop::start_eval(&js);
                            interop::recv_u64(&mut eval).await.unwrap_or(0) as usize
                        };
                        if let Some(mut cctx) = cursor_ctx_local {
                            let local_byte = utf16_to_byte_index(&text_clone, cursor_utf16)
                                .unwrap_or(text_clone.len());
                            let raw_offset = block_idx.saturating_add(local_byte);
                            let max_offset =
                                block_idx.saturating_add(last_caret_offset(&(0..text_clone.len())));
                            cctx.cursor_position.set(CursorPosition {
                                offset: raw_offset.min(max_offset),
                                line: 0,
                                column: 0,
                            });
                        }
                        if let Some(handler) = handler {
                            let text_len = text_clone.len();
                            handler.call(ActiveBlockInputEvent {
                                raw_text: text_clone.clone(),
                                visible_text: text_clone,
                                cursor_raw_utf16: cursor_utf16,
                                cursor_visible_utf16: cursor_utf16,
                                block_start: block_idx,
                                block_end: block_idx.saturating_add(text_len),
                            });
                        }
                    });
                }
            },
            onkeyup: move |_| {
                if let Some(mut cctx) = cursor_ctx {
                    let block_id = block_id_keyup.clone();
                    let current_global = ctx.raw_value();
                    let start = safe_start.min(current_global.len());
                    let old_len = *current_len.read();
                    let end = (start + old_len).min(current_global.len());
                    let block_text = current_global[start..end].to_string();
                    spawn(async move {
                        let cursor_utf16 = {
                            let js = interop::caret_adapter().read_textarea_cursor_js(&block_id);
                            let mut eval = interop::start_eval(&js);
                            interop::recv_u64(&mut eval).await.unwrap_or(0) as usize
                        };
                        let local_byte =
                            utf16_to_byte_index(&block_text, cursor_utf16).unwrap_or(block_text.len());
                        let raw_offset = start.saturating_add(local_byte);
                        let max_offset =
                            start.saturating_add(last_caret_offset(&(0..block_text.len())));
                        cctx.cursor_position.set(CursorPosition {
                            offset: raw_offset.min(max_offset),
                            line: 0,
                            column: 0,
                        });
                    });
                }
            },
            onmouseup: move |_| {
                if let Some(mut cctx) = cursor_ctx {
                    let block_id = block_id_mouseup.clone();
                    let current_global = ctx.raw_value();
                    let start = safe_start.min(current_global.len());
                    let old_len = *current_len.read();
                    let end = (start + old_len).min(current_global.len());
                    let block_text = current_global[start..end].to_string();
                    spawn(async move {
                        let cursor_utf16 = {
                            let js = interop::caret_adapter().read_textarea_cursor_js(&block_id);
                            let mut eval = interop::start_eval(&js);
                            interop::recv_u64(&mut eval).await.unwrap_or(0) as usize
                        };
                        let local_byte =
                            utf16_to_byte_index(&block_text, cursor_utf16).unwrap_or(block_text.len());
                        let raw_offset = start.saturating_add(local_byte);
                        let max_offset =
                            start.saturating_add(last_caret_offset(&(0..block_text.len())));
                        cctx.cursor_position.set(CursorPosition {
                            offset: raw_offset.min(max_offset),
                            line: 0,
                            column: 0,
                        });
                    });
                }
            },
            onmounted: move |_| {
                let js = interop::caret_adapter().mount_active_textarea_js(&block_id_mount, target_cursor);
                let mut len_sig = current_len;
                if let Some(mut cctx) = cursor_ctx {
                    spawn(async move {
                        let mut eval = interop::start_eval(&js);
                        while let Some(msg) = interop::recv_string(&mut eval).await {
                            if msg == "prev" {
                                let parsed = (ctx.parsed_doc)();
                                let target = adjacent_editable_offset(
                                    &parsed.ast,
                                    safe_start,
                                    safe_end,
                                    NavDirection::Prev,
                                );
                                cctx.cursor_position.set(CursorPosition {
                                    offset: target,
                                    line: 0,
                                    column: 0,
                                });
                                continue;
                            }
                            if msg == "next" {
                                let parsed = (ctx.parsed_doc)();
                                let target = adjacent_editable_offset(
                                    &parsed.ast,
                                    safe_start,
                                    safe_end,
                                    NavDirection::Next,
                                );
                                cctx.cursor_position.set(CursorPosition {
                                    offset: target,
                                    line: 0,
                                    column: 0,
                                });
                                continue;
                            }
                            if msg == "backjoin" {
                                perform_block_join(ctx, Some(cctx), safe_start);
                                continue;
                            }
                            if let Some(rest) = msg.strip_prefix("split:")
                                && let Ok(split_utf16) = rest.parse::<usize>()
                            {
                                let current_global = ctx.raw_value();
                                let start = safe_start.min(current_global.len());
                                let old_len = *len_sig.read();
                                let end = (start + old_len).min(current_global.len());
                                let block = &current_global[start..end];
                                let split_byte = utf16_to_byte_index(block, split_utf16).unwrap_or(block.len());
                                perform_block_split(
                                    ctx,
                                    Some(cctx),
                                    safe_start,
                                    &mut len_sig,
                                    split_byte,
                                );
                            }
                        }
                    });
                } else {
                    spawn(async move {
                        interop::eval_void(&js).await;
                    });
                }
            }
        }
    };

    match wrapper {
        "h1" => rsx! { h1 { {input_view} } },
        "h2" => rsx! { h2 { {input_view} } },
        "h3" => rsx! { h3 { {input_view} } },
        "h4" => rsx! { h4 { {input_view} } },
        "h5" => rsx! { h5 { {input_view} } },
        "h6" => rsx! { h6 { {input_view} } },
        "blockquote" => rsx! { blockquote { {input_view} } },
        "pre" => rsx! { pre { {input_view} } },
        "li" => rsx! { li { {input_view} } },
        _ => rsx! { p { {input_view} } },
    }
}

/// Convert a visible UTF-16 column to a raw byte offset within a block.
/// All markers are treated as hidden (cursor not yet in target block).
fn resolve_visible_column_in_block(
    node: &OwnedAstNode,
    full_raw: &str,
    block_start: usize,
    block_end: usize,
    visible_utf16: usize,
) -> usize {
    let editable_end = trim_editable_block_end(full_raw, block_start, block_end);
    let block_raw = &full_raw[block_start..editable_end];
    let mut model_node = node.clone();
    model_node.range = block_start..editable_end;
    let markers = collect_marker_tokens(&model_node, block_raw, block_start);
    let visibility = markers
        .iter()
        .enumerate()
        .map(|(idx, _)| MarkerVisibility {
            marker_idx: idx,
            visible: false,
        })
        .collect::<Vec<_>>();
    let model = build_tokenized_block(&model_node, full_raw, &visibility);
    let visible_byte = visible_utf16_to_raw_offset(&model, visible_utf16);
    block_start
        .saturating_add(visible_byte)
        .min(last_caret_offset(&(block_start..editable_end)))
}

fn handle_inactive_block_click(
    cursor_ctx: Option<CursorContext>,
    raw: String,
    node: OwnedAstNode,
    block_id: String,
    safe_start: usize,
    safe_end: usize,
) {
    if let Some(nc) = try_use_context::<InlineNavCtx>() {
        let mut gc = nc.goal_column;
        gc.set(None);
    }
    if let Some(mut cctx) = cursor_ctx {
        spawn(async move {
            let js = interop::caret_adapter().read_block_visual_offset_js(&block_id);
            let mut eval = interop::start_eval(&js);
            let visual_utf16 = interop::recv_string(&mut eval)
                .await
                .and_then(|s| s.parse::<usize>().ok())
                .unwrap_or(0);

            let slice_end = safe_end.min(raw.len());
            let slice_start = safe_start.min(slice_end);
            let clamped_offset =
                resolve_visible_column_in_block(&node, &raw, slice_start, slice_end, visual_utf16);

            cctx.cursor_position.set(CursorPosition {
                offset: clamped_offset,
                line: 0,
                column: 0,
            });
        });
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum NavDirection {
    Prev,
    Next,
}

fn adjacent_editable_offset(
    ast: &[OwnedAstNode],
    current_start: usize,
    current_end: usize,
    direction: NavDirection,
) -> usize {
    let mut nodes = Vec::new();
    collect_editable_nodes(ast, &mut nodes);

    match direction {
        NavDirection::Prev => nodes
            .iter()
            .rev()
            .find(|n| n.range.start < current_start)
            .map(|n| {
                // Inter-block navigation: use end - 1 because AST ranges may
                // include a trailing newline that we must not land on.
                if n.range.end > n.range.start {
                    n.range.end.saturating_sub(1)
                } else {
                    n.range.start
                }
            })
            .unwrap_or(current_start),
        NavDirection::Next => nodes
            .iter()
            .find(|n| n.range.start > current_start)
            .map(|n| n.range.start)
            .unwrap_or_else(|| {
                let range = &(current_start..current_end);
                if range.end > range.start {
                    range.end.saturating_sub(1)
                } else {
                    range.start
                }
            }),
    }
}

fn adjacent_editable_node(
    ast: &[OwnedAstNode],
    current_start: usize,
    direction: NavDirection,
) -> Option<OwnedAstNode> {
    let mut nodes = Vec::new();
    collect_editable_nodes(ast, &mut nodes);
    match direction {
        NavDirection::Prev => nodes
            .iter()
            .rev()
            .find(|n| n.range.start < current_start)
            .cloned(),
        NavDirection::Next => nodes
            .iter()
            .find(|n| n.range.start > current_start)
            .cloned(),
    }
}

fn collect_editable_nodes(nodes: &[OwnedAstNode], out: &mut Vec<OwnedAstNode>) {
    for node in nodes {
        if is_editable_block(node) {
            out.push(node.clone());
            continue;
        }
        collect_editable_nodes(&node.children, out);
    }
}

fn last_caret_offset(range: &std::ops::Range<usize>) -> usize {
    range.end
}

fn is_navigation_key(key: &str) -> bool {
    matches!(
        key,
        "ArrowLeft"
            | "ArrowRight"
            | "ArrowUp"
            | "ArrowDown"
            | "Home"
            | "End"
            | "PageUp"
            | "PageDown"
    )
}

fn utf16_len(s: &str) -> usize {
    s.chars().map(char::len_utf16).sum()
}

fn is_latest_revision(captured: u64, latest: u64) -> bool {
    captured == latest
}

fn trim_editable_block_end(raw: &str, start: usize, end: usize) -> usize {
    let mut trimmed_end = end.min(raw.len());
    while trimmed_end > start {
        let byte = raw.as_bytes()[trimmed_end - 1];
        if byte == b'\n' || byte == b'\r' {
            trimmed_end -= 1;
        } else {
            break;
        }
    }
    trimmed_end
}

fn utf16_to_byte_index(s: &str, utf16_idx: usize) -> Option<usize> {
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

fn byte_to_utf16_index(s: &str, byte_idx: usize) -> Option<usize> {
    if byte_idx > s.len() {
        return None;
    }
    let mut utf16_count = 0usize;
    for (idx, ch) in s.char_indices() {
        if idx == byte_idx {
            return Some(utf16_count);
        }
        if idx > byte_idx {
            break;
        }
        utf16_count += ch.len_utf16();
    }
    if byte_idx == s.len() {
        Some(utf16_count)
    } else {
        None
    }
}

fn render_inline_segment(seg: InlineSegment) -> Element {
    match seg.kind {
        SegmentKind::Marker(kind) => {
            let marker_kind = match kind {
                crate::inline_tokens::MarkerKind::Inline => "inline",
                crate::inline_tokens::MarkerKind::BlockPrefix => "block-prefix",
            };
            let text = seg.text.clone();
            rsx! {
                span {
                    "data-md-marker": "{marker_kind}",
                    "data-md-marker-start": "{seg.raw_range.start}",
                    "data-md-marker-end": "{seg.raw_range.end}",
                    "{text}"
                }
            }
        }
        SegmentKind::Text => render_text_with_marks(seg.text, &seg.marks),
    }
}

fn render_text_with_marks(text: String, marks: &[InlineMark]) -> Element {
    if marks.is_empty() {
        return rsx! { "{text}" };
    }

    let mut sorted = marks.to_vec();
    sorted.sort();
    let inner = render_text_with_marks(text, &sorted[1..]);
    match sorted[0] {
        InlineMark::Strong => rsx! { strong { {inner} } },
        InlineMark::Emphasis => rsx! { em { {inner} } },
        InlineMark::Strikethrough => rsx! { del { {inner} } },
        InlineMark::Code => rsx! { code { {inner} } },
        InlineMark::Link | InlineMark::Wikilink => rsx! { a { {inner} } },
        InlineMark::Image | InlineMark::Tag => rsx! { span { {inner} } },
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct VisibleEdit {
    old_start_utf16: usize,
    old_end_utf16: usize,
    replacement: String,
}

fn diff_visible_text(old_text: &str, new_text: &str) -> VisibleEdit {
    let old_chars: Vec<char> = old_text.chars().collect();
    let new_chars: Vec<char> = new_text.chars().collect();

    let mut prefix = 0usize;
    while prefix < old_chars.len()
        && prefix < new_chars.len()
        && old_chars[prefix] == new_chars[prefix]
    {
        prefix += 1;
    }

    let mut old_suffix = old_chars.len();
    let mut new_suffix = new_chars.len();
    while old_suffix > prefix
        && new_suffix > prefix
        && old_chars[old_suffix - 1] == new_chars[new_suffix - 1]
    {
        old_suffix -= 1;
        new_suffix -= 1;
    }

    let old_start_utf16 = old_chars[..prefix]
        .iter()
        .map(|ch| ch.len_utf16())
        .sum::<usize>();
    let old_end_utf16 = old_chars[..old_suffix]
        .iter()
        .map(|ch| ch.len_utf16())
        .sum::<usize>();
    let replacement = new_chars[prefix..new_suffix].iter().collect::<String>();

    VisibleEdit {
        old_start_utf16,
        old_end_utf16,
        replacement,
    }
}

fn build_plain_text_model(block_raw: &str, block_start: usize) -> TokenizedBlock {
    let visible_utf16_end = utf16_len(block_raw);
    TokenizedBlock {
        raw_text: block_raw.to_string(),
        block_start,
        block_end: block_start.saturating_add(block_raw.len()),
        segments: vec![InlineSegment {
            raw_range: 0..block_raw.len(),
            text: block_raw.to_string(),
            marks: vec![],
            kind: SegmentKind::Text,
            visible_utf16_start: 0,
            visible_utf16_end,
        }],
        visible_text: block_raw.to_string(),
    }
}

fn select_best_input_projection(
    candidates: &[TokenizedBlock],
    new_visible: &str,
    cursor_visible_utf16: usize,
) -> (usize, VisibleEdit) {
    let first = diff_visible_text(&candidates[0].visible_text, new_visible);
    let mut best_idx = 0usize;
    let mut best_edit = first;
    let mut best_rank = visible_edit_rank(&best_edit, cursor_visible_utf16);

    for (idx, candidate) in candidates.iter().enumerate().skip(1) {
        let candidate_edit = diff_visible_text(&candidate.visible_text, new_visible);
        let candidate_rank = visible_edit_rank(&candidate_edit, cursor_visible_utf16);
        if candidate_rank < best_rank {
            best_idx = idx;
            best_edit = candidate_edit;
            best_rank = candidate_rank;
        }
    }

    (best_idx, best_edit)
}

fn visible_edit_rank(edit: &VisibleEdit, cursor_visible_utf16: usize) -> (usize, usize, usize) {
    let removed = edit.old_end_utf16.saturating_sub(edit.old_start_utf16);
    let inserted = utf16_len(&edit.replacement);
    let span = removed.saturating_add(inserted);
    let distance = if cursor_visible_utf16 == 0 {
        0
    } else {
        cursor_visible_utf16.abs_diff(edit.old_start_utf16)
    };
    (span, distance, edit.old_start_utf16)
}

fn parse_selection_details(raw: &str) -> Option<SelectionDetails> {
    let mut parts = raw.splitn(3, '\u{1f}');
    let start = parts.next()?.parse::<usize>().ok()?;
    let end = parts.next()?.parse::<usize>().ok()?;
    let collapsed = matches!(parts.next()?, "1" | "true");
    Some(SelectionDetails {
        start,
        end,
        collapsed,
    })
}

fn parse_before_input_meta(raw: &str) -> Option<BeforeInputMeta> {
    if raw.is_empty() {
        return None;
    }
    let mut parts = raw.splitn(5, '\u{1f}');
    let start = parts.next()?.parse::<usize>().ok()?;
    let end = parts.next()?.parse::<usize>().ok()?;
    let is_collapsed = matches!(parts.next()?, "1" | "true");
    let input_type = parts.next()?.to_string();
    let data = parts.next().unwrap_or_default().to_string();

    Some(BeforeInputMeta {
        input_type,
        data,
        pre_visible_caret_utf16: start,
        pre_visible_selection_end_utf16: end,
        is_collapsed,
    })
}

fn compute_post_visible_caret(
    meta: Option<&BeforeInputMeta>,
    edit: &VisibleEdit,
    fallback_cursor_visible_utf16: usize,
    new_visible_utf16_len: usize,
) -> usize {
    let inserted_utf16 = utf16_len(&edit.replacement);

    if let Some(meta) = meta
        && meta.is_collapsed
    {
        if meta.input_type == "insertText" {
            let typed_utf16 = utf16_len(&meta.data);
            if typed_utf16 > 0 {
                return meta
                    .pre_visible_caret_utf16
                    .saturating_add(typed_utf16)
                    .min(new_visible_utf16_len);
            }
        }
        if meta.input_type.starts_with("deleteContent") {
            return edit
                .old_start_utf16
                .saturating_add(inserted_utf16)
                .min(new_visible_utf16_len);
        }
    } else {
        return edit
            .old_start_utf16
            .saturating_add(inserted_utf16)
            .min(new_visible_utf16_len);
    }

    normalize_cursor_visible_for_edit(fallback_cursor_visible_utf16, edit, new_visible_utf16_len)
}

fn normalize_cursor_visible_for_edit(
    cursor_visible_utf16: usize,
    edit: &VisibleEdit,
    new_visible_utf16_len: usize,
) -> usize {
    let replacement_utf16 = utf16_len(&edit.replacement);
    let inferred = edit.old_start_utf16.saturating_add(replacement_utf16);

    // For insertion-style edits, pin caret to end-of-replacement deterministically.
    // Browser-reported offsets can drift by one around adjacent text-node merges.
    if edit.old_start_utf16 == edit.old_end_utf16 && replacement_utf16 > 0 {
        return inferred.min(new_visible_utf16_len);
    }

    cursor_visible_utf16.min(new_visible_utf16_len)
}

fn cursor_after_visible_edit(
    old_model: &TokenizedBlock,
    new_cursor_visible_utf16: usize,
    edit: &VisibleEdit,
    old_raw_start: usize,
    old_raw_end: usize,
) -> usize {
    let old_visible_utf16 = old_model
        .visible_text
        .chars()
        .map(char::len_utf16)
        .sum::<usize>();
    let replacement_utf16 = edit.replacement.chars().map(char::len_utf16).sum::<usize>();
    let new_visible_utf16 = old_visible_utf16
        .saturating_sub(edit.old_end_utf16.saturating_sub(edit.old_start_utf16))
        .saturating_add(replacement_utf16);
    let visible_delta = new_visible_utf16 as isize - old_visible_utf16 as isize;
    let raw_delta =
        edit.replacement.len() as isize - (old_raw_end.saturating_sub(old_raw_start)) as isize;

    if new_cursor_visible_utf16 <= edit.old_start_utf16 {
        return visible_utf16_to_raw_offset(old_model, new_cursor_visible_utf16);
    }

    let replacement_end_utf16 = edit.old_start_utf16.saturating_add(replacement_utf16);
    if new_cursor_visible_utf16 < replacement_end_utf16 {
        let in_repl_utf16 = new_cursor_visible_utf16.saturating_sub(edit.old_start_utf16);
        let in_repl_byte =
            utf16_to_byte_index(&edit.replacement, in_repl_utf16).unwrap_or(edit.replacement.len());
        return old_raw_start.saturating_add(in_repl_byte);
    }

    let old_cursor_visible = (new_cursor_visible_utf16 as isize - visible_delta)
        .max(edit.old_end_utf16 as isize) as usize;
    let old_raw_cursor = visible_utf16_to_raw_offset(old_model, old_cursor_visible);
    (old_raw_cursor as isize + raw_delta).max(0) as usize
}

/// Inject synthetic empty `Paragraph` nodes for blank-line gaps between AST blocks.
///
/// pulldown-cmark absorbs one trailing `\n` into each block's range. The minimum
/// gap between two paragraphs is 1 byte (the second `\n` of the `\n\n` separator).
/// Every `\n` in a gap becomes a synthetic empty paragraph, so the standard `\n\n`
/// separator produces 1 visible blank line. Each Backspace at a block boundary
/// removes one `\n`, reducing synthetic paragraphs by 1 for immediate visual feedback.
pub(crate) fn inject_gap_paragraphs(ast: &[OwnedAstNode], raw: &str) -> Vec<OwnedAstNode> {
    let mut result = Vec::with_capacity(ast.len() * 2);
    let mut prev_end: usize = 0;

    for node in ast {
        let gap_start = prev_end;
        let gap_end = node.range.start;
        if gap_start < gap_end {
            let gap_bytes = &raw[gap_start..gap_end];
            // Every `\n` in the gap becomes a synthetic empty paragraph.
            // Standard `\n\n` separator → 1 synthetic node (visible blank line).
            let newline_count = gap_bytes.bytes().filter(|&b| b == b'\n').count();
            for i in 0..newline_count {
                let byte_pos = gap_start + i;
                result.push(OwnedAstNode {
                    node_type: NodeType::Paragraph,
                    range: byte_pos..byte_pos + 1,
                    children: vec![],
                });
            }
        }
        result.push(node.clone());
        prev_end = node.range.end;
    }

    // Trailing gap: after the last block
    if prev_end < raw.len() {
        let trailing = &raw[prev_end..];
        let newline_count = trailing.bytes().filter(|&b| b == b'\n').count();
        // Every `\n` in the trailing gap becomes a synthetic node
        for i in 0..newline_count {
            let byte_pos = prev_end + i;
            result.push(OwnedAstNode {
                node_type: NodeType::Paragraph,
                range: byte_pos..byte_pos + 1,
                children: vec![],
            });
        }
    }

    result
}

/// Snap a cursor byte offset to the nearest block range if it falls in a gap.
///
/// - If cursor is within any block's range -> return unchanged
/// - If cursor is between blocks -> snap to next block's start
/// - If cursor is past all blocks -> snap to last block's end (or 0 if empty)
pub(crate) fn snap_cursor_to_block(ast: &[OwnedAstNode], cursor: usize) -> usize {
    if ast.is_empty() {
        return 0;
    }
    for node in ast {
        if cursor >= node.range.start && cursor < node.range.end {
            return cursor; // within a block
        }
    }
    // Cursor is in a gap — find the next block
    for node in ast {
        if node.range.start > cursor {
            return node.range.start;
        }
    }
    // Past all blocks — snap to last block's range
    let last = &ast[ast.len() - 1];
    if last.range.end > last.range.start {
        last.range.end.saturating_sub(1)
    } else {
        last.range.start
    }
}

/// Perform a block split: insert `\n\n` at the cursor position within a block.
///
/// Shared between `TokenAwareBlockEditor` (Enter key) and `ActiveBlockEditor`
/// (textarea split message). Updates raw content, triggers reparse, and sets
/// cursor to the start of the new paragraph.
fn perform_block_split(
    ctx: MarkdownContext,
    cursor_ctx: Option<CursorContext>,
    safe_start: usize,
    len_sig: &mut Signal<usize>,
    split_byte: usize,
) {
    let current_global = ctx.raw_value();
    let start = safe_start.min(current_global.len());
    let old_len = *len_sig.read();
    let end = (start + old_len).min(current_global.len());
    let before = &current_global[..start];
    let block = &current_global[start..end];
    let after = &current_global[end..];
    let split_at = split_byte.min(block.len());
    let left = &block[..split_at];
    let right = &block[split_at..];
    let rebuilt = format!("{before}{left}\n\n{right}{after}");
    ctx.handle_value_change(rebuilt);
    ctx.trigger_parse.call(());
    len_sig.set(left.len());
    if let Some(mut cctx) = cursor_ctx {
        cctx.cursor_position.set(CursorPosition {
            offset: start + split_at + 2,
            line: 0,
            column: 0,
        });
    }
}

/// Remove exactly ONE `\n` before `block_start`.
///
/// Symmetric inverse of `perform_block_split` (which inserts `\n\n`).
/// Two paragraphs (`\n\n` gap): first Backspace → `\n` (soft break).
/// Multiple blank lines (`\n\n\n\n`): each Backspace removes one `\n`.
fn perform_block_join(ctx: MarkdownContext, cursor_ctx: Option<CursorContext>, block_start: usize) {
    if block_start == 0 {
        return;
    }
    let current_global = ctx.raw_value();
    let pos = block_start.min(current_global.len());
    if pos == 0 || current_global.as_bytes()[pos - 1] != b'\n' {
        return;
    }
    let join_point = pos - 1;
    let rebuilt = format!(
        "{}{}",
        &current_global[..join_point],
        &current_global[pos..]
    );
    ctx.handle_value_change(rebuilt);
    ctx.trigger_parse.call(());
    if let Some(mut cctx) = cursor_ctx {
        cctx.cursor_position.set(CursorPosition {
            offset: join_point,
            line: 0,
            column: 0,
        });
    }
}

#[cfg(test)]
mod tests {
    use super::{
        BeforeInputMeta, NavDirection, VisibleEdit, adjacent_editable_offset,
        block_has_inline_markup, compute_post_visible_caret, cursor_within_inline_markup,
        direct_delete_from_beforeinput, inject_gap_paragraphs, is_latest_revision,
        next_visible_char_utf16, normalize_cursor_visible_for_edit, previous_visible_char_utf16,
        select_best_input_projection, snap_cursor_to_block, uses_token_aware_surface,
    };
    use crate::inline_tokens::{InlineSegment, SegmentKind, TokenizedBlock};
    use crate::types::{NodeType, OwnedAstNode};

    fn text_node(start: usize, end: usize, text: &str) -> OwnedAstNode {
        OwnedAstNode {
            node_type: NodeType::Text(text.to_string()),
            range: start..end,
            children: vec![],
        }
    }

    #[test]
    fn plain_paragraph_is_always_editable() {
        let node = OwnedAstNode {
            node_type: NodeType::Paragraph,
            range: 0..18,
            children: vec![text_node(0, 18, "plain text only")],
        };
        assert!(!block_has_inline_markup(&node));
        assert!(!uses_token_aware_surface(&node));
    }

    #[test]
    fn mixed_paragraph_uses_token_aware_surface() {
        let strong = OwnedAstNode {
            node_type: NodeType::Strong,
            range: 19..27, // **er**
            children: vec![text_node(21, 23, "er")],
        };
        let node = OwnedAstNode {
            node_type: NodeType::Paragraph,
            range: 0..55,
            children: vec![
                text_node(0, 19, "Borrowing lets you ref"),
                strong,
                text_node(27, 55, "ence data without taking ownership."),
            ],
        };

        assert!(block_has_inline_markup(&node));
        assert!(cursor_within_inline_markup(&node, 22)); // inside "er"
        assert!(!cursor_within_inline_markup(&node, 10)); // plain text
        assert!(uses_token_aware_surface(&node));
    }

    #[test]
    fn nav_next_skips_non_editable_gap() {
        let ast = vec![
            OwnedAstNode {
                node_type: NodeType::Paragraph,
                range: 0..5,
                children: vec![text_node(0, 5, "first")],
            },
            OwnedAstNode {
                node_type: NodeType::Paragraph,
                range: 7..12,
                children: vec![text_node(7, 12, "second")],
            },
        ];

        let next = adjacent_editable_offset(&ast, 0, 5, NavDirection::Next);
        assert_eq!(next, 7);
    }

    #[test]
    fn nav_prev_targets_previous_editable_block_end() {
        let ast = vec![
            OwnedAstNode {
                node_type: NodeType::Paragraph,
                range: 0..5,
                children: vec![text_node(0, 5, "first")],
            },
            OwnedAstNode {
                node_type: NodeType::Paragraph,
                range: 7..12,
                children: vec![text_node(7, 12, "second")],
            },
        ];

        let prev = adjacent_editable_offset(&ast, 7, 12, NavDirection::Prev);
        assert_eq!(prev, 4);
    }

    #[test]
    fn revision_guard_accepts_latest_only() {
        assert!(is_latest_revision(4, 4));
        assert!(!is_latest_revision(3, 4));
    }

    #[test]
    fn projection_selector_prefers_closest_visible_model() {
        let plain = TokenizedBlock {
            raw_text: "ref**er**ence".to_string(),
            block_start: 0,
            block_end: 12,
            segments: vec![],
            visible_text: "reference".to_string(),
        };
        let raw_like = TokenizedBlock {
            raw_text: "ref**er**ence".to_string(),
            block_start: 0,
            block_end: 12,
            segments: vec![],
            visible_text: "ref**er**ence".to_string(),
        };
        let (idx, _) = select_best_input_projection(
            &[plain, raw_like],
            "ref**er**ence!",
            "ref**er**ence!".chars().count(),
        );
        assert_eq!(idx, 1);
    }

    #[test]
    fn insertion_cursor_normalization_avoids_transient_zero_jump() {
        let edit = VisibleEdit {
            old_start_utf16: 10,
            old_end_utf16: 10,
            replacement: "*".to_string(),
        };
        let normalized = normalize_cursor_visible_for_edit(0, &edit, 24);
        assert_eq!(normalized, 11);
    }

    #[test]
    fn compute_post_caret_uses_beforeinput_for_collapsed_star_insert() {
        let edit = VisibleEdit {
            old_start_utf16: 57,
            old_end_utf16: 57,
            replacement: "*".to_string(),
        };
        let meta = BeforeInputMeta {
            input_type: "insertText".to_string(),
            data: "*".to_string(),
            pre_visible_caret_utf16: 57,
            pre_visible_selection_end_utf16: 57,
            is_collapsed: true,
        };
        let post = compute_post_visible_caret(Some(&meta), &edit, 56, 80);
        assert_eq!(post, 58);
    }

    #[test]
    fn compute_post_caret_collapses_noncollapsed_insert_to_end_of_replacement() {
        let edit = VisibleEdit {
            old_start_utf16: 20,
            old_end_utf16: 22,
            replacement: "**".to_string(),
        };
        let meta = BeforeInputMeta {
            input_type: "insertText".to_string(),
            data: "**".to_string(),
            pre_visible_caret_utf16: 20,
            pre_visible_selection_end_utf16: 22,
            is_collapsed: false,
        };
        let post = compute_post_visible_caret(Some(&meta), &edit, 0, 80);
        assert_eq!(post, 22);
    }

    #[test]
    fn compute_post_caret_delete_prefers_edit_window_start() {
        let edit = VisibleEdit {
            old_start_utf16: 11,
            old_end_utf16: 12,
            replacement: String::new(),
        };
        let meta = BeforeInputMeta {
            input_type: "deleteContentBackward".to_string(),
            data: String::new(),
            pre_visible_caret_utf16: 12,
            pre_visible_selection_end_utf16: 12,
            is_collapsed: true,
        };
        let post = compute_post_visible_caret(Some(&meta), &edit, 12, 80);
        assert_eq!(post, 11);
    }

    // ── inject_gap_paragraphs tests ──────────────────────────────────

    #[test]
    fn inject_gap_paragraphs_no_gap() {
        // Two adjacent blocks with no gap between them
        let ast = vec![
            OwnedAstNode {
                node_type: NodeType::Paragraph,
                range: 0..6,
                children: vec![text_node(0, 5, "Hello")],
            },
            OwnedAstNode {
                node_type: NodeType::Paragraph,
                range: 6..12,
                children: vec![text_node(6, 11, "World")],
            },
        ];
        let raw = "Hello\nWorld\n";
        let result = inject_gap_paragraphs(&ast, raw);
        assert_eq!(result.len(), 2); // No synthetic nodes
    }

    #[test]
    fn inject_gap_paragraphs_minimal_gap() {
        // Standard \n\n separator: pulldown-cmark includes trailing \n in first block,
        // so the gap is 1 byte (the second \n). 1 newline in gap → 1 synthetic node
        // (visible blank line between blocks).
        let ast = vec![
            OwnedAstNode {
                node_type: NodeType::Paragraph,
                range: 0..6, // "Hello\n"
                children: vec![text_node(0, 5, "Hello")],
            },
            OwnedAstNode {
                node_type: NodeType::Paragraph,
                range: 7..13, // "World\n"
                children: vec![text_node(7, 12, "World")],
            },
        ];
        let raw = "Hello\n\nWorld\n";
        let result = inject_gap_paragraphs(&ast, raw);
        // Gap is raw[6..7] = "\n" → 1 newline → 1 synthetic node (loop 0..1)
        assert_eq!(result.len(), 3);
        assert_eq!(result[0].range, 0..6);
        assert_eq!(result[1].range, 6..7); // synthetic blank-line paragraph
        assert!(result[1].children.is_empty());
        assert_eq!(result[2].range, 7..13);
    }

    #[test]
    fn inject_gap_paragraphs_single_extra_blank_line() {
        // "Hello\n\n\nWorld\n" — gap between blocks is 2 bytes ("\n\n"),
        // which means 2 newlines in the gap → 2 synthetic nodes (loop 0..2)
        let ast = vec![
            OwnedAstNode {
                node_type: NodeType::Paragraph,
                range: 0..6, // "Hello\n"
                children: vec![text_node(0, 5, "Hello")],
            },
            OwnedAstNode {
                node_type: NodeType::Paragraph,
                range: 8..14, // "World\n"
                children: vec![text_node(8, 13, "World")],
            },
        ];
        let raw = "Hello\n\n\nWorld\n";
        let result = inject_gap_paragraphs(&ast, raw);
        // Gap is raw[6..8] = "\n\n" → 2 newlines → 2 synthetic nodes
        assert_eq!(result.len(), 4);
        assert_eq!(result[0].range, 0..6); // original first block
        assert_eq!(result[1].range, 6..7); // first synthetic
        assert_eq!(result[2].range, 7..8); // second synthetic
        assert!(result[1].children.is_empty());
        assert!(result[2].children.is_empty());
        assert_eq!(result[3].range, 8..14); // original second block
    }

    #[test]
    fn inject_gap_paragraphs_multiple_extra_blank_lines() {
        // "Hello\n\n\n\nWorld\n" — gap is 3 bytes ("\n\n\n"),
        // 3 newlines in gap → 3 synthetic nodes (loop 0..3)
        let ast = vec![
            OwnedAstNode {
                node_type: NodeType::Paragraph,
                range: 0..6, // "Hello\n"
                children: vec![text_node(0, 5, "Hello")],
            },
            OwnedAstNode {
                node_type: NodeType::Paragraph,
                range: 9..15, // "World\n"
                children: vec![text_node(9, 14, "World")],
            },
        ];
        let raw = "Hello\n\n\n\nWorld\n";
        let result = inject_gap_paragraphs(&ast, raw);
        // Gap is raw[6..9] = "\n\n\n" → 3 newlines → 3 synthetic nodes
        assert_eq!(result.len(), 5);
        assert_eq!(result[0].range, 0..6);
        assert_eq!(result[1].range, 6..7); // first synthetic
        assert_eq!(result[2].range, 7..8); // second synthetic
        assert_eq!(result[3].range, 8..9); // third synthetic
        assert_eq!(result[4].range, 9..15);
    }

    #[test]
    fn inject_gap_paragraphs_trailing_blank_lines() {
        // "Hello\n\n\n" — one block, then trailing newlines after the block
        let ast = vec![OwnedAstNode {
            node_type: NodeType::Paragraph,
            range: 0..6, // "Hello\n"
            children: vec![text_node(0, 5, "Hello")],
        }];
        let raw = "Hello\n\n\n";
        let result = inject_gap_paragraphs(&ast, raw);
        // Trailing is raw[6..8] = "\n\n" → 2 newlines → 2 synthetic nodes (loop 0..2)
        assert_eq!(result.len(), 3);
        assert_eq!(result[0].range, 0..6);
        assert_eq!(result[1].range, 6..7); // first synthetic trailing
        assert_eq!(result[2].range, 7..8); // second synthetic trailing
        assert!(result[1].children.is_empty());
        assert!(result[2].children.is_empty());
    }

    #[test]
    fn inject_gap_paragraphs_empty_ast() {
        let ast: Vec<OwnedAstNode> = vec![];
        let raw = "";
        let result = inject_gap_paragraphs(&ast, raw);
        assert!(result.is_empty());
    }

    // ── snap_cursor_to_block tests ───────────────────────────────────

    #[test]
    fn snap_cursor_within_block_unchanged() {
        let ast = vec![
            OwnedAstNode {
                node_type: NodeType::Paragraph,
                range: 0..6,
                children: vec![text_node(0, 5, "Hello")],
            },
            OwnedAstNode {
                node_type: NodeType::Paragraph,
                range: 8..14,
                children: vec![text_node(8, 13, "World")],
            },
        ];
        // Cursor at position 3 is inside first block (0..6)
        assert_eq!(snap_cursor_to_block(&ast, 3), 3);
        // Cursor at position 10 is inside second block (8..14)
        assert_eq!(snap_cursor_to_block(&ast, 10), 10);
    }

    #[test]
    fn snap_cursor_in_gap_to_next_block() {
        let ast = vec![
            OwnedAstNode {
                node_type: NodeType::Paragraph,
                range: 0..6,
                children: vec![text_node(0, 5, "Hello")],
            },
            OwnedAstNode {
                node_type: NodeType::Paragraph,
                range: 8..14,
                children: vec![text_node(8, 13, "World")],
            },
        ];
        // Cursor at 7 is in gap between blocks (6..8), snaps to next block start
        assert_eq!(snap_cursor_to_block(&ast, 7), 8);
    }

    #[test]
    fn snap_cursor_past_end_to_last_block() {
        let ast = vec![
            OwnedAstNode {
                node_type: NodeType::Paragraph,
                range: 0..6,
                children: vec![text_node(0, 5, "Hello")],
            },
            OwnedAstNode {
                node_type: NodeType::Paragraph,
                range: 8..14,
                children: vec![text_node(8, 13, "World")],
            },
        ];
        // Cursor at 20 is past all blocks, snaps to last block end - 1
        assert_eq!(snap_cursor_to_block(&ast, 20), 13);
    }

    #[test]
    fn snap_cursor_empty_ast_returns_zero() {
        let ast: Vec<OwnedAstNode> = vec![];
        assert_eq!(snap_cursor_to_block(&ast, 5), 0);
    }

    #[test]
    fn snap_cursor_at_block_start_unchanged() {
        let ast = vec![
            OwnedAstNode {
                node_type: NodeType::Paragraph,
                range: 0..6,
                children: vec![text_node(0, 5, "Hello")],
            },
            OwnedAstNode {
                node_type: NodeType::Paragraph,
                range: 8..14,
                children: vec![text_node(8, 13, "World")],
            },
        ];
        // Cursor at exact start of second block
        assert_eq!(snap_cursor_to_block(&ast, 8), 8);
        // Cursor at exact start of first block
        assert_eq!(snap_cursor_to_block(&ast, 0), 0);
    }

    // ── direct_delete_from_beforeinput tests ─────────────────────────

    /// Helper: build a plain-text TokenizedBlock (no hidden markers).
    fn plain_model(text: &str) -> TokenizedBlock {
        let utf16_end: usize = text.chars().map(char::len_utf16).sum();
        TokenizedBlock {
            raw_text: text.to_string(),
            block_start: 0,
            block_end: text.len(),
            segments: vec![InlineSegment {
                raw_range: 0..text.len(),
                text: text.to_string(),
                marks: vec![],
                kind: SegmentKind::Text,
                visible_utf16_start: 0,
                visible_utf16_end: utf16_end,
            }],
            visible_text: text.to_string(),
        }
    }

    /// Helper: build a model with hidden markers (e.g. `**bold**` → visible `bold`).
    /// `raw` = full raw text, `parts` = (raw_range, text, visible) segments.
    fn model_with_segments(
        raw: &str,
        parts: Vec<(std::ops::Range<usize>, &str, bool)>,
    ) -> TokenizedBlock {
        let mut segments = Vec::new();
        let mut vis_pos = 0usize;
        let mut visible_text = String::new();
        for (range, text, is_visible) in &parts {
            if *is_visible {
                let utf16_len: usize = text.chars().map(char::len_utf16).sum();
                segments.push(InlineSegment {
                    raw_range: range.clone(),
                    text: text.to_string(),
                    marks: vec![],
                    kind: SegmentKind::Text,
                    visible_utf16_start: vis_pos,
                    visible_utf16_end: vis_pos + utf16_len,
                });
                visible_text.push_str(text);
                vis_pos += utf16_len;
            }
            // Hidden markers are simply not in the segments list
        }
        TokenizedBlock {
            raw_text: raw.to_string(),
            block_start: 0,
            block_end: raw.len(),
            segments,
            visible_text,
        }
    }

    fn backspace_meta(caret: usize) -> BeforeInputMeta {
        BeforeInputMeta {
            input_type: "deleteContentBackward".to_string(),
            data: String::new(),
            pre_visible_caret_utf16: caret,
            pre_visible_selection_end_utf16: caret,
            is_collapsed: true,
        }
    }

    fn forward_delete_meta(caret: usize) -> BeforeInputMeta {
        BeforeInputMeta {
            input_type: "deleteContentForward".to_string(),
            data: String::new(),
            pre_visible_caret_utf16: caret,
            pre_visible_selection_end_utf16: caret,
            is_collapsed: true,
        }
    }

    #[test]
    fn direct_backspace_in_plain_text() {
        // "hello" → backspace at position 5 → delete 'o' (bytes 4..5)
        let model = plain_model("hello");
        let meta = backspace_meta(5);
        let result = direct_delete_from_beforeinput(&meta, &model, "hello");
        assert_eq!(result, Some((4, 5, 4)));
    }

    #[test]
    fn direct_backspace_adjacent_to_hidden_marker() {
        // Raw: "**bold**" (8 bytes), visible: "bold" (4 chars)
        // Hidden markers: ** at raw 0..2 and ** at raw 6..8
        // Cursor at visible pos 4 (end of "bold") → should delete 'd' (raw 5..6)
        let model = model_with_segments(
            "**bold**",
            vec![
                (0..2, "**", false),  // hidden opening **
                (2..6, "bold", true), // visible text
                (6..8, "**", false),  // hidden closing **
            ],
        );
        let meta = backspace_meta(4); // end of visible "bold"
        let result = direct_delete_from_beforeinput(&meta, &model, "**bold**");
        assert_eq!(result, Some((5, 6, 3))); // delete raw 5..6 ('d'), cursor → vis 3
    }

    #[test]
    fn direct_backspace_inside_revealed_marker() {
        // Raw: "**bold**" (8 bytes), all markers revealed (visible = "**bold**")
        // Cursor at visible pos 2 (after "**") → should delete '*' (raw 1..2)
        let model = model_with_segments(
            "**bold**",
            vec![
                (0..2, "**", true),   // visible opening **
                (2..6, "bold", true), // visible text
                (6..8, "**", true),   // visible closing **
            ],
        );
        let meta = backspace_meta(2); // after opening "**"
        let result = direct_delete_from_beforeinput(&meta, &model, "**bold**");
        assert_eq!(result, Some((1, 2, 1))); // delete raw 1..2 ('*'), cursor → vis 1
    }

    #[test]
    fn direct_backspace_after_hidden_closing_marker() {
        // Raw: "**bold** more" (13 bytes), visible: "bold more"
        // Hidden markers: ** at raw 0..2 and ** at raw 6..8
        // Cursor at visible pos 5 (the space after "bold") → maps to raw 8 (after hidden **)
        // Should delete the last char of content before cursor in visible space = ' ' which
        // is at raw 8. But wait — visible char before pos 5 is at pos 4 ("bold" ends).
        // Actually visible "bold more": pos 4 = ' ', pos 5 = 'm'
        // Let's put cursor at vis 4 (the space) → backspace deletes 'd' at vis 3
        let model = model_with_segments(
            "**bold** more",
            vec![
                (0..2, "**", false),    // hidden opening **
                (2..6, "bold", true),   // visible text
                (6..8, "**", false),    // hidden closing **
                (8..13, " more", true), // visible text
            ],
        );
        let meta = backspace_meta(5); // cursor at vis 5 (space before "more")
        let result = direct_delete_from_beforeinput(&meta, &model, "**bold** more");
        // vis 5 = ' ' (raw 8), vis 4 = 'd' (raw 5) — wait, vis 4 is the start of " more" seg
        // Actually: "bold" is vis 0..4, " more" is vis 4..9
        // vis 5 maps to raw 9 (' ' is at raw 8, 'm' at raw 9 — vis 4 = raw 8)
        // Backspace at vis 5: prev char is vis 4, so delete raw mapping of vis 4..vis 5
        // vis 4 → raw 8, vis 5 → raw 9, so delete raw 8..9 = ' '
        assert_eq!(result, Some((8, 9, 4)));
    }

    #[test]
    fn direct_delete_forward_plain_text() {
        // "hello" → Delete key at position 0 → delete 'h' (bytes 0..1)
        let model = plain_model("hello");
        let meta = forward_delete_meta(0);
        let result = direct_delete_from_beforeinput(&meta, &model, "hello");
        assert_eq!(result, Some((0, 1, 0)));
    }

    #[test]
    fn direct_backspace_with_emoji() {
        // "a😀b" — '😀' is 4 bytes, 2 UTF-16 units
        let model = plain_model("a😀b");
        // Cursor at vis pos 3 (after 😀: 'a'=1 UTF-16, '😀'=2 UTF-16)
        let meta = backspace_meta(3);
        let result = direct_delete_from_beforeinput(&meta, &model, "a😀b");
        // Should delete '😀' at raw bytes 1..5, cursor → vis 1
        assert_eq!(result, Some((1, 5, 1)));
    }

    #[test]
    fn direct_backspace_at_position_zero_returns_none() {
        let model = plain_model("hello");
        let meta = backspace_meta(0);
        let result = direct_delete_from_beforeinput(&meta, &model, "hello");
        assert_eq!(result, None);
    }

    #[test]
    fn direct_delete_non_collapsed_returns_none() {
        let model = plain_model("hello");
        let meta = BeforeInputMeta {
            input_type: "deleteContentBackward".to_string(),
            data: String::new(),
            pre_visible_caret_utf16: 2,
            pre_visible_selection_end_utf16: 4,
            is_collapsed: false,
        };
        let result = direct_delete_from_beforeinput(&meta, &model, "hello");
        assert_eq!(result, None);
    }

    #[test]
    fn direct_delete_insert_type_returns_none() {
        let model = plain_model("hello");
        let meta = BeforeInputMeta {
            input_type: "insertText".to_string(),
            data: "x".to_string(),
            pre_visible_caret_utf16: 3,
            pre_visible_selection_end_utf16: 3,
            is_collapsed: true,
        };
        let result = direct_delete_from_beforeinput(&meta, &model, "hello");
        assert_eq!(result, None);
    }

    #[test]
    fn direct_backspace_with_strikethrough_hidden() {
        // Raw: "~~strike~~" (10 bytes), visible: "strike" (6 chars)
        // Cursor at visible pos 6 (end) → delete 'e' (raw 7..8)
        let model = model_with_segments(
            "~~strike~~",
            vec![
                (0..2, "~~", false),    // hidden opening ~~
                (2..8, "strike", true), // visible text
                (8..10, "~~", false),   // hidden closing ~~
            ],
        );
        let meta = backspace_meta(6);
        let result = direct_delete_from_beforeinput(&meta, &model, "~~strike~~");
        assert_eq!(result, Some((7, 8, 5)));
    }

    #[test]
    fn previous_visible_char_utf16_basic() {
        assert_eq!(previous_visible_char_utf16("hello", 0), None);
        assert_eq!(previous_visible_char_utf16("hello", 1), Some(0));
        assert_eq!(previous_visible_char_utf16("hello", 5), Some(4));
    }

    #[test]
    fn previous_visible_char_utf16_emoji() {
        // "a😀b" — UTF-16: a(1) 😀(2) b(1) = total 4
        assert_eq!(previous_visible_char_utf16("a😀b", 3), Some(1)); // before 😀 = pos 1
        assert_eq!(previous_visible_char_utf16("a😀b", 1), Some(0)); // before a = 0
    }

    #[test]
    fn next_visible_char_utf16_basic() {
        assert_eq!(next_visible_char_utf16("hello", 0), Some(1));
        assert_eq!(next_visible_char_utf16("hello", 4), Some(5));
        assert_eq!(next_visible_char_utf16("hello", 5), None); // past end
    }

    #[test]
    fn next_visible_char_utf16_emoji() {
        // "a😀b" — UTF-16: a(1) 😀(2) b(1)
        assert_eq!(next_visible_char_utf16("a😀b", 1), Some(3)); // 😀 occupies 2 units
        assert_eq!(next_visible_char_utf16("a😀b", 3), Some(4)); // 'b' is 1 unit
    }
}
