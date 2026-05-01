use std::collections::BTreeSet;
use std::ops::Range;

use crate::types::{NodeType, OwnedAstNode};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MarkerKind {
    Inline,
    BlockPrefix,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MarkerToken {
    pub raw_range: Range<usize>,
    pub token_range: Range<usize>,
    pub kind: MarkerKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum InlineMark {
    Strong,
    Emphasis,
    Strikethrough,
    Code,
    Link,
    Image,
    Wikilink,
    Tag,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MarkSpan {
    pub raw_range: Range<usize>,
    pub mark: InlineMark,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SegmentKind {
    Text,
    Marker(MarkerKind),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InlineSegment {
    pub raw_range: Range<usize>,
    pub text: String,
    pub marks: Vec<InlineMark>,
    pub kind: SegmentKind,
    pub visible_utf16_start: usize,
    pub visible_utf16_end: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TokenizedBlock {
    pub raw_text: String,
    pub block_start: usize,
    pub block_end: usize,
    pub segments: Vec<InlineSegment>,
    pub visible_text: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MarkerVisibility {
    pub marker_idx: usize,
    pub visible: bool,
}

pub fn build_tokenized_block(
    node: &OwnedAstNode,
    full_raw: &str,
    marker_visibility: &[MarkerVisibility],
) -> TokenizedBlock {
    let block_end = node.range.end.min(full_raw.len());
    let block_start = node.range.start.min(block_end);
    let raw_text = full_raw[block_start..block_end].to_string();
    let raw_len = raw_text.len();

    let markers = collect_marker_tokens(node, &raw_text, block_start);
    let mark_spans = collect_mark_spans(node, &raw_text, block_start);

    let mut hidden_marker_ranges = Vec::new();
    for (idx, marker) in markers.iter().enumerate() {
        let visible = marker_visibility
            .iter()
            .find(|mv| mv.marker_idx == idx)
            .map(|mv| mv.visible)
            .unwrap_or(false);
        if !visible {
            hidden_marker_ranges.push(marker.raw_range.clone());
        }
    }

    let mut boundaries: BTreeSet<usize> = BTreeSet::new();
    boundaries.insert(0);
    boundaries.insert(raw_len);
    for marker in &markers {
        boundaries.insert(marker.raw_range.start.min(raw_len));
        boundaries.insert(marker.raw_range.end.min(raw_len));
    }
    for span in &mark_spans {
        boundaries.insert(span.raw_range.start.min(raw_len));
        boundaries.insert(span.raw_range.end.min(raw_len));
    }

    let points: Vec<usize> = boundaries.into_iter().collect();
    let mut segments = Vec::new();
    let mut visible_utf16 = 0usize;
    let mut visible_text = String::new();

    for window in points.windows(2) {
        let start = window[0];
        let end = window[1];
        if end <= start || end > raw_len {
            continue;
        }

        let local_range = start..end;
        if hidden_marker_ranges
            .iter()
            .any(|hidden| range_contains(hidden, &local_range))
        {
            continue;
        }

        let text = raw_text[start..end].to_string();
        let marker_kind = markers
            .iter()
            .find(|m| range_contains(&m.raw_range, &local_range))
            .map(|m| m.kind);

        let mut marks: Vec<InlineMark> = mark_spans
            .iter()
            .filter(|span| range_contains(&span.raw_range, &local_range))
            .map(|span| span.mark)
            .collect();
        marks.sort();
        marks.dedup();

        let seg_utf16_len = text.chars().map(char::len_utf16).sum();
        let seg = InlineSegment {
            raw_range: local_range,
            text: text.clone(),
            marks,
            kind: marker_kind.map_or(SegmentKind::Text, SegmentKind::Marker),
            visible_utf16_start: visible_utf16,
            visible_utf16_end: visible_utf16.saturating_add(seg_utf16_len),
        };
        visible_utf16 = seg.visible_utf16_end;
        visible_text.push_str(&text);
        segments.push(seg);
    }

    TokenizedBlock {
        raw_text,
        block_start,
        block_end,
        segments,
        visible_text,
    }
}

pub fn collect_marker_tokens(
    node: &OwnedAstNode,
    block_raw: &str,
    block_start: usize,
) -> Vec<MarkerToken> {
    let mut out = Vec::new();

    if let Some(prefix) = block_prefix_range(node, block_raw) {
        out.push(MarkerToken {
            raw_range: prefix.clone(),
            token_range: prefix,
            kind: MarkerKind::BlockPrefix,
        });
    }

    collect_inline_markers_recursive(node, block_start, &mut out);
    out
}

pub fn collect_mark_spans(
    node: &OwnedAstNode,
    block_raw: &str,
    block_start: usize,
) -> Vec<MarkSpan> {
    let mut out = Vec::new();
    collect_mark_spans_recursive(node, block_raw, block_start, &mut out);
    out
}

pub fn visible_utf16_to_raw_offset(model: &TokenizedBlock, visible_utf16: usize) -> usize {
    for seg in &model.segments {
        if visible_utf16 < seg.visible_utf16_start {
            continue;
        }
        if visible_utf16 <= seg.visible_utf16_end {
            let local_utf16 = visible_utf16.saturating_sub(seg.visible_utf16_start);
            let local_byte = utf16_to_byte_index(&seg.text, local_utf16).unwrap_or(seg.text.len());
            return seg.raw_range.start.saturating_add(local_byte);
        }
    }
    model.raw_text.len()
}

pub fn raw_offset_to_visible_utf16(model: &TokenizedBlock, raw_offset: usize) -> usize {
    for seg in &model.segments {
        if raw_offset < seg.raw_range.start {
            return seg.visible_utf16_start;
        }
        if raw_offset <= seg.raw_range.end {
            let local_byte = raw_offset
                .saturating_sub(seg.raw_range.start)
                .min(seg.text.len());
            let utf16_count = seg.text[..local_byte]
                .chars()
                .map(char::len_utf16)
                .sum::<usize>();
            return seg.visible_utf16_start.saturating_add(utf16_count);
        }
    }
    model.segments.last().map_or(0, |seg| seg.visible_utf16_end)
}

pub fn utf16_to_byte_index(s: &str, utf16_idx: usize) -> Option<usize> {
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

fn collect_inline_markers_recursive(
    node: &OwnedAstNode,
    block_start: usize,
    out: &mut Vec<MarkerToken>,
) {
    if is_inline_markup_node(&node.node_type)
        && !node.children.is_empty()
        && let (Some(first), Some(last)) = (node.children.first(), node.children.last())
    {
        let left_start = node.range.start.saturating_sub(block_start);
        let left_end = first.range.start.saturating_sub(block_start);
        let right_start = last.range.end.saturating_sub(block_start);
        let right_end = node.range.end.saturating_sub(block_start);
        let token_start = node.range.start.saturating_sub(block_start);
        let token_end = node.range.end.saturating_sub(block_start);

        if left_end > left_start {
            out.push(MarkerToken {
                raw_range: left_start..left_end,
                token_range: token_start..token_end,
                kind: MarkerKind::Inline,
            });
        }
        if right_end > right_start {
            out.push(MarkerToken {
                raw_range: right_start..right_end,
                token_range: token_start..token_end,
                kind: MarkerKind::Inline,
            });
        }
    }

    for child in &node.children {
        collect_inline_markers_recursive(child, block_start, out);
    }
}

fn collect_mark_spans_recursive(
    node: &OwnedAstNode,
    block_raw: &str,
    block_start: usize,
    out: &mut Vec<MarkSpan>,
) {
    if let Some(mark) = mark_kind_for_node(&node.node_type) {
        if node.children.is_empty() && mark == InlineMark::Code {
            let node_start = node.range.start.saturating_sub(block_start);
            let node_end = node
                .range
                .end
                .saturating_sub(block_start)
                .min(block_raw.len());
            if node_end > node_start {
                let slice = &block_raw[node_start..node_end];
                let mut left_ticks = 0usize;
                for ch in slice.chars() {
                    if ch == '`' {
                        left_ticks += ch.len_utf8();
                    } else {
                        break;
                    }
                }
                let mut right_ticks = 0usize;
                for ch in slice.chars().rev() {
                    if ch == '`' {
                        right_ticks += ch.len_utf8();
                    } else {
                        break;
                    }
                }
                if left_ticks + right_ticks < slice.len() {
                    let content_start = node_start.saturating_add(left_ticks);
                    let content_end = node_end.saturating_sub(right_ticks);
                    if content_end > content_start {
                        out.push(MarkSpan {
                            raw_range: content_start..content_end,
                            mark,
                        });
                    }
                }
            }
        } else if let (Some(first), Some(last)) = (node.children.first(), node.children.last()) {
            let start = first.range.start.saturating_sub(block_start);
            let end = last.range.end.saturating_sub(block_start);
            if end > start {
                out.push(MarkSpan {
                    raw_range: start..end,
                    mark,
                });
            }
        }
    }

    for child in &node.children {
        collect_mark_spans_recursive(child, block_raw, block_start, out);
    }
}

fn range_contains(outer: &Range<usize>, inner: &Range<usize>) -> bool {
    inner.start >= outer.start && inner.end <= outer.end
}

fn is_inline_markup_node(node_type: &NodeType) -> bool {
    matches!(
        node_type,
        NodeType::Emphasis
            | NodeType::Strong
            | NodeType::Strikethrough
            | NodeType::Link { .. }
            | NodeType::Image { .. }
            | NodeType::Code(_)
            | NodeType::Wikilink(_)
            | NodeType::Tag(_)
    )
}

fn mark_kind_for_node(node_type: &NodeType) -> Option<InlineMark> {
    match node_type {
        NodeType::Strong => Some(InlineMark::Strong),
        NodeType::Emphasis => Some(InlineMark::Emphasis),
        NodeType::Strikethrough => Some(InlineMark::Strikethrough),
        NodeType::Code(_) => Some(InlineMark::Code),
        NodeType::Link { .. } => Some(InlineMark::Link),
        NodeType::Image { .. } => Some(InlineMark::Image),
        NodeType::Wikilink(_) => Some(InlineMark::Wikilink),
        NodeType::Tag(_) => Some(InlineMark::Tag),
        _ => None,
    }
}

fn block_prefix_range(node: &OwnedAstNode, raw: &str) -> Option<Range<usize>> {
    let len = match node.node_type {
        NodeType::Heading(_) => heading_prefix_len(raw),
        NodeType::BlockQuote => blockquote_prefix_len(raw),
        NodeType::Item => list_prefix_len(raw),
        _ => 0,
    };
    (len > 0).then_some(0..len.min(raw.len()))
}

fn heading_prefix_len(raw: &str) -> usize {
    let mut idx = 0usize;
    for ch in raw.chars() {
        if ch == '#' || ch == ' ' {
            idx += ch.len_utf8();
        } else {
            break;
        }
    }
    idx
}

fn blockquote_prefix_len(raw: &str) -> usize {
    let mut idx = 0usize;
    for ch in raw.chars() {
        if ch == '>' || ch == ' ' {
            idx += ch.len_utf8();
        } else {
            break;
        }
    }
    idx
}

fn list_prefix_len(raw: &str) -> usize {
    let bytes = raw.as_bytes();
    let mut idx = 0usize;
    while idx < bytes.len() {
        let b = bytes[idx];
        let is_prefix = b == b'-'
            || b == b'*'
            || b == b'+'
            || b == b'['
            || b == b']'
            || b == b' '
            || b == b'.'
            || (b as char).is_ascii_digit();
        if is_prefix {
            idx += 1;
        } else {
            break;
        }
    }
    idx
}

#[cfg(test)]
mod tests {
    use super::{
        MarkerKind, MarkerVisibility, SegmentKind, TokenizedBlock, build_tokenized_block,
        collect_marker_tokens, raw_offset_to_visible_utf16, visible_utf16_to_raw_offset,
    };
    use crate::types::{NodeType, OwnedAstNode};

    fn text_node(start: usize, end: usize, text: &str) -> OwnedAstNode {
        OwnedAstNode {
            node_type: NodeType::Text(text.to_string()),
            range: start..end,
            children: vec![],
        }
    }

    #[test]
    fn marker_ranges_for_strong_are_extracted() {
        let strong = OwnedAstNode {
            node_type: NodeType::Strong,
            range: 3..9,
            children: vec![text_node(5, 7, "er")],
        };
        let node = OwnedAstNode {
            node_type: NodeType::Paragraph,
            range: 0..13,
            children: vec![text_node(0, 3, "ref"), strong, text_node(9, 13, "ence")],
        };
        let raw = "ref**er**ence";
        let markers = collect_marker_tokens(&node, raw, 0);
        assert_eq!(markers.len(), 2);
        assert_eq!(markers[0].raw_range, 3..5);
        assert_eq!(markers[1].raw_range, 7..9);
        assert_eq!(markers[0].kind, MarkerKind::Inline);
    }

    #[test]
    fn marker_ranges_for_list_prefix_are_extracted() {
        let node = OwnedAstNode {
            node_type: NodeType::Item,
            range: 0..20,
            children: vec![text_node(2, 20, "hello world")],
        };
        let raw = "- hello world";
        let markers = collect_marker_tokens(&node, raw, 0);
        assert!(!markers.is_empty());
        assert_eq!(markers[0].kind, MarkerKind::BlockPrefix);
        assert_eq!(markers[0].raw_range, 0..2);
    }

    #[test]
    fn hidden_markers_are_removed_from_visible_text() {
        let strong = OwnedAstNode {
            node_type: NodeType::Strong,
            range: 3..9,
            children: vec![text_node(5, 7, "er")],
        };
        let node = OwnedAstNode {
            node_type: NodeType::Paragraph,
            range: 0..13,
            children: vec![text_node(0, 3, "ref"), strong, text_node(9, 13, "ence")],
        };
        let raw = "ref**er**ence";
        let hidden = vec![
            MarkerVisibility {
                marker_idx: 0,
                visible: false,
            },
            MarkerVisibility {
                marker_idx: 1,
                visible: false,
            },
        ];
        let model = build_tokenized_block(&node, raw, &hidden);
        assert_eq!(model.visible_text, "reference");
        assert!(
            model
                .segments
                .iter()
                .all(|seg| !matches!(seg.kind, SegmentKind::Marker(_)))
        );
    }

    #[test]
    fn visible_raw_offset_roundtrip() {
        let node = OwnedAstNode {
            node_type: NodeType::Paragraph,
            range: 0..12,
            children: vec![text_node(0, 12, "hello world!")],
        };
        let raw = "hello world!";
        let model: TokenizedBlock = build_tokenized_block(&node, raw, &[]);
        let raw = visible_utf16_to_raw_offset(&model, 5);
        let visible = raw_offset_to_visible_utf16(&model, raw);
        assert_eq!(visible, 5);
    }

    #[test]
    fn visible_end_maps_past_hidden_closing_marker() {
        // "a single *owner*" — emphasis wraps "owner" with hidden * markers
        let emph = OwnedAstNode {
            node_type: NodeType::Emphasis,
            range: 9..17,
            children: vec![text_node(10, 16, "owner!")],
        };
        let node = OwnedAstNode {
            node_type: NodeType::Paragraph,
            range: 0..17,
            children: vec![text_node(0, 9, "a single "), emph],
        };
        let raw = "a single *owner!*";
        let hidden = vec![
            MarkerVisibility {
                marker_idx: 0,
                visible: false,
            },
            MarkerVisibility {
                marker_idx: 1,
                visible: false,
            },
        ];
        let model = build_tokenized_block(&node, raw, &hidden);
        // Visible text: "a single owner!" (15 chars, visible UTF-16 range 0..15)
        assert_eq!(model.visible_text, "a single owner!");
        let max_visible = model.segments.last().map_or(0, |s| s.visible_utf16_end);
        assert_eq!(max_visible, 15);

        // At max_visible, raw offset lands at the closing * position (16, not 17).
        // The last_caret_offset fix (returning range.end instead of range.end-1)
        // ensures this position is no longer clamped away by the inline editor.
        let raw_at_end = visible_utf16_to_raw_offset(&model, max_visible);
        assert_eq!(raw_at_end, 16); // at the closing *, within the allowed range
    }

    #[test]
    fn roundtrip_at_end_with_hidden_closing_marker() {
        // "hello **world**" — strong wraps "world" with hidden ** markers
        let strong = OwnedAstNode {
            node_type: NodeType::Strong,
            range: 6..15,
            children: vec![text_node(8, 13, "world")],
        };
        let node = OwnedAstNode {
            node_type: NodeType::Paragraph,
            range: 0..15,
            children: vec![text_node(0, 6, "hello "), strong],
        };
        let raw = "hello **world**";
        let hidden = vec![
            MarkerVisibility {
                marker_idx: 0,
                visible: false,
            },
            MarkerVisibility {
                marker_idx: 1,
                visible: false,
            },
        ];
        let model = build_tokenized_block(&node, raw, &hidden);
        assert_eq!(model.visible_text, "hello world");
        let max_vis = model.segments.last().map_or(0, |s| s.visible_utf16_end);

        // Round-trip: visible end → raw → visible should be stable
        let raw_end = visible_utf16_to_raw_offset(&model, max_vis);
        let vis_back = raw_offset_to_visible_utf16(&model, raw_end);
        assert_eq!(vis_back, max_vis);
    }
}
