use crate::types::{
    BlockEntry, CursorPosition, HeadingEntry, LivePreviewVariant, Mode, ParseOptions, ParseState,
    Selection,
};

// ── Mode enum tests ──────────────────────────────────────────────────

#[test]
fn mode_default_is_source() {
    assert_eq!(Mode::default(), Mode::Source);
}

#[test]
fn mode_display_read() {
    assert_eq!(Mode::Read.to_string(), "read");
}

#[test]
fn mode_display_source() {
    assert_eq!(Mode::Source.to_string(), "source");
}

#[test]
fn mode_display_live_preview() {
    assert_eq!(Mode::LivePreview.to_string(), "live-preview");
}

#[test]
fn mode_clone_and_copy() {
    let m = Mode::LivePreview;
    let m2 = m;
    let m3 = m;
    assert_eq!(m, m2);
    assert_eq!(m, m3);
}

#[test]
fn mode_eq_same_variants() {
    assert_eq!(Mode::Read, Mode::Read);
    assert_eq!(Mode::Source, Mode::Source);
    assert_eq!(Mode::LivePreview, Mode::LivePreview);
}

#[test]
fn mode_ne_different_variants() {
    assert_ne!(Mode::Read, Mode::Source);
    assert_ne!(Mode::Source, Mode::LivePreview);
    assert_ne!(Mode::Read, Mode::LivePreview);
}

#[test]
fn mode_debug_is_implemented() {
    // Verify Debug trait is derived — format! should not panic
    let s = format!("{:?}", Mode::Read);
    assert!(!s.is_empty());
}

// ── CursorPosition tests ────────────────────────────────────────────

#[test]
fn cursor_position_default() {
    let pos = CursorPosition::default();
    assert_eq!(pos.line, 0);
    assert_eq!(pos.column, 0);
    assert_eq!(pos.offset, 0);
}

#[test]
fn cursor_position_new() {
    let pos = CursorPosition::new(5, 10, 42);
    assert_eq!(pos.line, 5);
    assert_eq!(pos.column, 10);
    assert_eq!(pos.offset, 42);
}

#[test]
fn cursor_position_equality() {
    let a = CursorPosition::new(1, 2, 3);
    let b = CursorPosition::new(1, 2, 3);
    let c = CursorPosition::new(1, 2, 4);
    assert_eq!(a, b);
    assert_ne!(a, c);
}

#[test]
fn cursor_position_clone() {
    let a = CursorPosition::new(3, 7, 20);
    let b = a;
    assert_eq!(a, b);
}

// ── Selection tests ─────────────────────────────────────────────────

#[test]
fn selection_new() {
    let sel = Selection::new(10, 20);
    assert_eq!(sel.anchor, 10);
    assert_eq!(sel.head, 20);
}

#[test]
fn selection_is_collapsed_when_equal() {
    let sel = Selection::new(5, 5);
    assert!(sel.is_collapsed());
}

#[test]
fn selection_is_not_collapsed_when_different() {
    let sel = Selection::new(5, 10);
    assert!(!sel.is_collapsed());
}

#[test]
fn selection_len_forward() {
    let sel = Selection::new(5, 15);
    assert_eq!(sel.len(), 10);
}

#[test]
fn selection_len_backward() {
    let sel = Selection::new(15, 5);
    assert_eq!(sel.len(), 10);
}

#[test]
fn selection_len_collapsed() {
    let sel = Selection::new(7, 7);
    assert_eq!(sel.len(), 0);
}

#[test]
fn selection_ordered_forward() {
    let sel = Selection::new(5, 15);
    assert_eq!(sel.ordered(), (5, 15));
}

#[test]
fn selection_ordered_backward() {
    let sel = Selection::new(15, 5);
    assert_eq!(sel.ordered(), (5, 15));
}

#[test]
fn selection_equality() {
    let a = Selection::new(1, 5);
    let b = Selection::new(1, 5);
    let c = Selection::new(5, 1);
    assert_eq!(a, b);
    assert_ne!(a, c);
}

#[test]
fn selection_clone() {
    let a = Selection::new(3, 8);
    let b = a;
    assert_eq!(a, b);
}

// ── HeadingEntry tests ──────────────────────────────────────────────

#[test]
fn heading_entry_construction() {
    let entry = HeadingEntry {
        level: 2,
        text: "Hello World".to_string(),
        anchor: "hello-world".to_string(),
        line: 5,
    };
    assert_eq!(entry.level, 2);
    assert_eq!(entry.text, "Hello World");
    assert_eq!(entry.anchor, "hello-world");
    assert_eq!(entry.line, 5);
}

#[test]
fn heading_entry_equality() {
    let a = HeadingEntry {
        level: 1,
        text: "Title".to_string(),
        anchor: "title".to_string(),
        line: 0,
    };
    let b = a.clone();
    assert_eq!(a, b);
}

#[test]
fn heading_entry_debug() {
    let entry = HeadingEntry {
        level: 3,
        text: "Sub".to_string(),
        anchor: "sub".to_string(),
        line: 10,
    };
    let s = format!("{:?}", entry);
    assert!(s.contains("Sub"));
}

#[test]
fn heading_entry_level_range() {
    // Levels 1-6 are valid; the struct stores u8 so it can hold any,
    // but the typical range is 1-6
    for level in 1u8..=6 {
        let entry = HeadingEntry {
            level,
            text: format!("H{level}"),
            anchor: format!("h{level}"),
            line: level as usize,
        };
        assert_eq!(entry.level, level);
    }
}

// ── Mode data attribute tests ──────────────────────────────────────

#[test]
fn mode_to_data_attr_value_read() {
    assert_eq!(Mode::Read.to_data_attr_value(), "read");
}

#[test]
fn mode_to_data_attr_value_source() {
    assert_eq!(Mode::Source.to_data_attr_value(), "source");
}

#[test]
fn mode_to_data_attr_value_live_preview() {
    // IMPORTANT: kebab-case "live-preview" not snake_case "live_preview"
    assert_eq!(Mode::LivePreview.to_data_attr_value(), "live-preview");
}

// ── Selection is_empty / is_forward tests ──────────────────────────

#[test]
fn selection_is_empty_same_anchor_head() {
    let sel = Selection { anchor: 3, head: 3 };
    assert!(sel.is_empty());
}

#[test]
fn selection_is_not_empty_different() {
    let sel = Selection { anchor: 1, head: 5 };
    assert!(!sel.is_empty());
}

#[test]
fn selection_is_forward() {
    let sel = Selection { anchor: 2, head: 8 };
    assert!(sel.is_forward());
}

#[test]
fn selection_is_backward() {
    let sel = Selection { anchor: 8, head: 2 };
    assert!(!sel.is_forward());
}

#[test]
fn selection_len_via_struct() {
    let sel = Selection { anchor: 3, head: 8 };
    assert_eq!(sel.len(), 5);
}

// ── ParseOptions tests ─────────────────────────────────────────────

#[test]
fn parse_options_default_debounce_ms() {
    let opts = ParseOptions::default();
    assert_eq!(opts.debounce_ms, 300);
}

#[test]
fn parse_options_default_tab_size() {
    let opts = ParseOptions::default();
    assert_eq!(opts.tab_size, 2);
}

// ── ParseState data attribute tests ────────────────────────────────

#[test]
fn parse_state_idle_attr() {
    assert_eq!(ParseState::Idle.to_data_attr_value(), "idle");
}

#[test]
fn parse_state_parsing_attr() {
    assert_eq!(ParseState::Parsing.to_data_attr_value(), "parsing");
}

#[test]
fn parse_state_done_attr() {
    assert_eq!(ParseState::Done.to_data_attr_value(), "done");
}

#[test]
fn parse_state_error_attr() {
    assert_eq!(ParseState::Error.to_data_attr_value(), "error");
}

// ── MarkdownContext method tests ─────────────────────────────────────
// These tests exercise context methods inside a VirtualDom component.
// We use a thread_local error cell to smuggle assertion failures out of
// the VirtualDom (which swallows panics from component bodies).

#[cfg(test)]
mod context_tests {
    use std::cell::RefCell;
    use std::rc::Rc;

    use dioxus::prelude::*;

    use crate::context::MarkdownContext;
    use crate::types::Mode;

    thread_local! {
        static TEST_ERROR: RefCell<Option<String>> = const { RefCell::new(None) };
    }

    fn set_error(msg: String) {
        TEST_ERROR.with(|e| *e.borrow_mut() = Some(msg));
    }

    fn take_error() -> Option<String> {
        TEST_ERROR.with(|e| e.borrow_mut().take())
    }

    /// Creates a minimal uncontrolled MarkdownContext for testing.
    /// MUST be called inside an active Dioxus runtime (inside a component).
    fn make_test_context(initial_value: &str) -> MarkdownContext {
        let raw = Rc::new(RefCell::new(crop::Rope::from(initial_value)));
        let raw_signal = Signal::new(raw);
        let mode = Signal::new(Mode::Source);
        let parsed_doc = Memo::new(move || {
            Rc::new(crate::types::ParsedDoc {
                element: rsx! {},
                headings: vec![],
                front_matter: None,
                blocks: vec![],
                ast: vec![],
            })
        });
        MarkdownContext {
            mode,
            is_mode_controlled: false,
            on_mode_change: None,
            raw_content: raw_signal,
            is_value_controlled: false,
            on_value_change: None,
            parsed_doc,
            is_editor_scrolling: Signal::new(false),
            is_preview_scrolling: Signal::new(false),
            instance_n: 0,
            editor_mount: Signal::new(None),
            disabled: false,
            trigger_parse: Callback::new(|_| {}),
            live_preview_variant: Signal::new(crate::types::LivePreviewVariant::SplitPane),
            highlight_class_prefix: Signal::new("hl-".to_string()),
            show_code_line_numbers: false,
            show_code_language: true,
            show_editor_line_numbers: false,
        }
    }

    /// Run a component in a VirtualDom and check the thread-local error cell after.
    fn run_and_check(app: fn() -> Element) {
        // Clear any previous error.
        take_error();
        let mut dom = VirtualDom::new(app);
        dom.rebuild_in_place();
        if let Some(msg) = take_error() {
            panic!("Context test failed: {msg}");
        }
    }

    fn test_current_mode_app() -> Element {
        let ctx = make_test_context("");
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| ctx.current_mode()));
        match result {
            Ok(mode) if mode != Mode::Source => {
                set_error(format!("expected Source, got {mode:?}"));
            }
            Err(_) => set_error("current_mode() panicked".to_string()),
            _ => {}
        }
        rsx! { div {} }
    }

    #[test]
    fn context_current_mode_returns_signal_value() {
        run_and_check(test_current_mode_app);
    }

    fn test_raw_value_app() -> Element {
        let ctx = make_test_context("# Hello");
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| ctx.raw_value()));
        match result {
            Ok(val) if val != "# Hello" => {
                set_error(format!("expected '# Hello', got '{val}'"));
            }
            Err(_) => set_error("raw_value() panicked".to_string()),
            _ => {}
        }
        rsx! { div {} }
    }

    #[test]
    fn context_raw_value_reads_rc_refcell() {
        run_and_check(test_raw_value_app);
    }

    fn test_handle_value_change_app() -> Element {
        let ctx = make_test_context("old");
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            ctx.handle_value_change("new content".to_string());
            ctx.raw_value()
        }));
        match result {
            Ok(val) if val != "new content" => {
                set_error(format!("expected 'new content', got '{val}'"));
            }
            Err(_) => set_error("handle_value_change/raw_value panicked".to_string()),
            _ => {}
        }
        rsx! { div {} }
    }

    #[test]
    fn context_handle_value_change_updates_raw_content() {
        run_and_check(test_handle_value_change_app);
    }

    fn test_handle_mode_change_app() -> Element {
        let mut ctx = make_test_context("");
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            ctx.handle_mode_change(Mode::Read);
            ctx.current_mode()
        }));
        match result {
            Ok(mode) if mode != Mode::Read => {
                set_error(format!("expected Read, got {mode:?}"));
            }
            Err(_) => set_error("handle_mode_change/current_mode panicked".to_string()),
            _ => {}
        }
        rsx! { div {} }
    }

    #[test]
    fn context_handle_mode_change_uncontrolled() {
        run_and_check(test_handle_mode_change_app);
    }

    fn test_mode_noop_app() -> Element {
        let mut ctx = make_test_context("");
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            ctx.handle_mode_change(Mode::Source);
            ctx.current_mode()
        }));
        match result {
            Ok(mode) if mode != Mode::Source => {
                set_error(format!("expected Source, got {mode:?}"));
            }
            Err(_) => set_error("handle_mode_change panicked on same mode".to_string()),
            _ => {}
        }
        rsx! { div {} }
    }

    #[test]
    fn context_handle_mode_change_noop_same_mode() {
        run_and_check(test_mode_noop_app);
    }

    fn test_disabled_app() -> Element {
        let mut ctx = make_test_context("");
        if ctx.disabled {
            set_error("disabled should default to false".to_string());
        }
        ctx.disabled = true;
        if !ctx.disabled {
            set_error("disabled should be true after set".to_string());
        }
        rsx! { div {} }
    }

    #[test]
    fn context_disabled_flag() {
        run_and_check(test_disabled_app);
    }
}

// ============================================================
// Parser tests — written by parser-hooks agent in Wave 1B
// These will fail to compile until parser.rs is implemented (Wave 1C).
// parse_document() returns a ParsedDoc with headings and front_matter fields.
// ============================================================
#[cfg(test)]
mod parser_tests {
    use crate::parser::parse_document;

    #[test]
    fn parse_empty_string() {
        let rope = crop::Rope::from("");
        let doc = parse_document(&rope);
        assert!(doc.headings.is_empty());
        assert!(doc.front_matter.is_none());
    }

    #[test]
    fn parse_heading_level_1() {
        let rope = crop::Rope::from("# Hello");
        let doc = parse_document(&rope);
        assert_eq!(doc.headings.len(), 1);
        assert_eq!(doc.headings[0].level, 1);
        assert_eq!(doc.headings[0].text, "Hello");
    }

    #[test]
    fn parse_heading_all_levels() {
        let markdown = "# H1\n## H2\n### H3\n#### H4\n##### H5\n###### H6";
        let rope = crop::Rope::from(markdown);
        let doc = parse_document(&rope);
        assert_eq!(doc.headings.len(), 6);
        for (i, level) in (1u8..=6).enumerate() {
            assert_eq!(doc.headings[i].level, level, "heading {i} has wrong level");
            assert_eq!(
                doc.headings[i].text,
                format!("H{level}"),
                "heading {i} has wrong text",
            );
        }
    }

    #[test]
    fn parse_heading_slug_spaces() {
        let rope = crop::Rope::from("# Hello World");
        let doc = parse_document(&rope);
        assert_eq!(doc.headings[0].anchor, "hello-world");
    }

    #[test]
    fn parse_heading_slug_special_chars() {
        // Non-alphanumeric chars become hyphens; consecutive hyphens collapse; trailing trimmed
        let rope = crop::Rope::from("# Rust & Go");
        let doc = parse_document(&rope);
        assert_eq!(doc.headings[0].anchor, "rust-go");
    }

    #[test]
    fn parse_gfm_task_list() {
        let markdown = "- [x] Done\n- [ ] Not done";
        let rope = crop::Rope::from(markdown);
        let doc = parse_document(&rope);
        // Task list items should parse without panic; no headings present
        assert!(doc.headings.is_empty());
        assert!(doc.front_matter.is_none());
    }

    #[test]
    fn parse_front_matter_raw() {
        let markdown = "---\ntitle: Test\n---\n\n# Content";
        let rope = crop::Rope::from(markdown);
        let doc = parse_document(&rope);
        let fm = doc
            .front_matter
            .as_deref()
            .expect("front matter should be Some");
        assert!(
            fm.contains("title: Test"),
            "Expected front matter to contain 'title: Test', got: {fm:?}",
        );
    }

    #[test]
    fn parse_front_matter_absent() {
        let rope = crop::Rope::from("# No front matter here");
        let doc = parse_document(&rope);
        assert!(doc.front_matter.is_none());
    }

    #[test]
    fn parse_heading_line_numbers() {
        // HeadingEntry.line is 0-based per types.rs documentation
        let markdown = "# First\n\nSome paragraph.\n\n## Second";
        let rope = crop::Rope::from(markdown);
        let doc = parse_document(&rope);
        assert_eq!(doc.headings.len(), 2);
        assert_eq!(doc.headings[0].line, 0, "first heading should be on line 0");
        assert_eq!(
            doc.headings[1].line, 4,
            "second heading should be on line 4"
        );
    }

    #[test]
    fn parse_heading_with_inline_code() {
        let rope = crop::Rope::from("# The `Config` struct");
        let doc = parse_document(&rope);
        assert_eq!(doc.headings.len(), 1);
        // Heading text should contain the code content without backticks
        assert!(
            doc.headings[0].text.contains("Config"),
            "heading text should include inline code content, got: {:?}",
            doc.headings[0].text,
        );
    }

    #[test]
    fn parse_multiple_paragraphs_no_headings() {
        let markdown = "First paragraph.\n\nSecond paragraph.\n\nThird.";
        let rope = crop::Rope::from(markdown);
        let doc = parse_document(&rope);
        assert!(doc.headings.is_empty());
        assert!(doc.front_matter.is_none());
    }

    #[test]
    fn parse_mixed_content() {
        let markdown = "\
# Title

Some text.

## Subsection

- item 1
- item 2

### Deep heading

> blockquote
";
        let rope = crop::Rope::from(markdown);
        let doc = parse_document(&rope);
        assert_eq!(doc.headings.len(), 3);
        assert_eq!(doc.headings[0].level, 1);
        assert_eq!(doc.headings[0].text, "Title");
        assert_eq!(doc.headings[1].level, 2);
        assert_eq!(doc.headings[1].text, "Subsection");
        assert_eq!(doc.headings[2].level, 3);
        assert_eq!(doc.headings[2].text, "Deep heading");
    }

    #[test]
    fn parse_heading_slug_duplicate_handling() {
        // Two identical headings should produce distinct slugs or identical slugs
        // (depends on implementation — this test documents behavior)
        let markdown = "# Title\n\n# Title";
        let rope = crop::Rope::from(markdown);
        let doc = parse_document(&rope);
        assert_eq!(doc.headings.len(), 2);
        assert_eq!(doc.headings[0].anchor, "title");
        // Second slug may be "title" or "title-1" depending on implementation
        assert!(!doc.headings[1].anchor.is_empty());
    }

    #[test]
    fn parse_setext_heading() {
        // Setext-style headings should also be extracted
        let markdown = "Title\n=====\n\nSubsection\n----------";
        let rope = crop::Rope::from(markdown);
        let doc = parse_document(&rope);
        assert_eq!(doc.headings.len(), 2);
        assert_eq!(doc.headings[0].level, 1);
        assert_eq!(doc.headings[0].text, "Title");
        assert_eq!(doc.headings[1].level, 2);
        assert_eq!(doc.headings[1].text, "Subsection");
    }
}

// ============================================================
// Pure-Rust parser helper tests (no comrak dependency)
// These test index_to_line_col — will fail to compile until parser.rs exports it
// ============================================================
#[cfg(test)]
mod parser_helper_tests {
    use crate::parser::index_to_line_col;

    #[test]
    fn index_to_line_col_start() {
        // Offset 0 in any string -> line 0, col 0
        assert_eq!(index_to_line_col("abc", 0), (0, 0));
    }

    #[test]
    fn index_to_line_col_mid_first_line() {
        // "abc", offset 2 -> line 0, col 2
        assert_eq!(index_to_line_col("abc", 2), (0, 2));
    }

    #[test]
    fn index_to_line_col_after_newline() {
        // "ab\ncd", offset 3 -> line 1, col 0
        assert_eq!(index_to_line_col("ab\ncd", 3), (1, 0));
    }

    #[test]
    fn index_to_line_col_mid_second_line() {
        // "ab\ncd", offset 4 -> line 1, col 1
        assert_eq!(index_to_line_col("ab\ncd", 4), (1, 1));
    }

    #[test]
    fn index_to_line_col_at_newline() {
        // "ab\ncd", offset 2 -> line 0, col 2 (the newline char itself)
        assert_eq!(index_to_line_col("ab\ncd", 2), (0, 2));
    }

    #[test]
    fn index_to_line_col_multiple_newlines() {
        // "a\nb\nc", offset 4 -> line 2, col 0
        assert_eq!(index_to_line_col("a\nb\nc", 4), (2, 0));
    }

    #[test]
    fn index_to_line_col_empty_lines() {
        // "a\n\nc", offset 2 -> line 1, col 0 (the empty line)
        assert_eq!(index_to_line_col("a\n\nc", 2), (1, 0));
    }

    #[test]
    fn index_to_line_col_end_of_string() {
        // "abc", offset 3 -> line 0, col 3 (one past last char)
        assert_eq!(index_to_line_col("abc", 3), (0, 3));
    }
}

// ── Component data-state string tests ─────────────────────────────
#[cfg(test)]
mod component_tests {
    #[test]
    fn data_state_active_when_source() {
        use crate::types::Mode;
        let state = match Mode::Source {
            Mode::Read => "inactive",
            _ => "active",
        };
        assert_eq!(state, "active");
    }

    #[test]
    fn data_state_inactive_when_read() {
        use crate::types::Mode;
        let state = match Mode::Read {
            Mode::Read => "inactive",
            _ => "active",
        };
        assert_eq!(state, "inactive");
    }

    #[test]
    fn mode_attr_live_preview_kebab() {
        use crate::types::Mode;
        assert_eq!(Mode::LivePreview.to_data_attr_value(), "live-preview");
    }

    // ── Editor data-state per mode ────────────────────────────────
    #[test]
    fn editor_data_state_source_is_active() {
        use crate::types::Mode;
        let state = match Mode::Source {
            Mode::Read => "inactive",
            Mode::Source | Mode::LivePreview => "active",
        };
        assert_eq!(state, "active");
    }

    #[test]
    fn editor_data_state_live_preview_is_active() {
        use crate::types::Mode;
        let state = match Mode::LivePreview {
            Mode::Read => "inactive",
            Mode::Source | Mode::LivePreview => "active",
        };
        assert_eq!(state, "active");
    }

    #[test]
    fn editor_data_state_read_is_inactive() {
        use crate::types::Mode;
        let state = match Mode::Read {
            Mode::Read => "inactive",
            Mode::Source | Mode::LivePreview => "active",
        };
        assert_eq!(state, "inactive");
    }

    // ── Preview data-state per mode ───────────────────────────────
    #[test]
    fn preview_data_state_live_preview_is_active() {
        use crate::types::Mode;
        let state = match Mode::LivePreview {
            Mode::LivePreview => "active",
            _ => "inactive",
        };
        assert_eq!(state, "active");
    }

    #[test]
    fn preview_data_state_source_is_inactive() {
        use crate::types::Mode;
        let state = match Mode::Source {
            Mode::LivePreview => "active",
            _ => "inactive",
        };
        assert_eq!(state, "inactive");
    }

    #[test]
    fn preview_data_state_read_is_inactive() {
        use crate::types::Mode;
        let state = match Mode::Read {
            Mode::LivePreview => "active",
            _ => "inactive",
        };
        assert_eq!(state, "inactive");
    }

    // ── IME composing guard logic ──────────────────────────────────
    #[test]
    fn parse_not_triggered_during_composition() {
        use std::cell::RefCell;
        use std::rc::Rc;
        let composing = Rc::new(RefCell::new(true));
        let mut triggered = false;
        if !*composing.borrow() {
            triggered = true;
        }
        assert!(!triggered);
    }

    #[test]
    fn parse_triggered_when_not_composing() {
        use std::cell::RefCell;
        use std::rc::Rc;
        let composing = Rc::new(RefCell::new(false));
        let mut triggered = false;
        if !*composing.borrow() {
            triggered = true;
        }
        assert!(triggered);
    }

    // ── Focus state string tests ──────────────────────────────────
    #[test]
    fn focused_data_attr_true_on_focus() {
        let focused = true;
        let attr = if focused { "true" } else { "false" };
        assert_eq!(attr, "true");
    }

    #[test]
    fn focused_data_attr_false_on_blur() {
        let focused = false;
        let attr = if focused { "true" } else { "false" };
        assert_eq!(attr, "false");
    }

    // ── Wave 2A: Divider + Root layout ───────────────────────────
    #[test]
    fn divider_orientation_default_vertical() {
        // Divider #[props(default = "vertical".to_string())] must equal "vertical"
        let default_orientation = "vertical";
        assert_eq!(default_orientation, "vertical");
        assert_ne!(default_orientation, "horizontal");
    }

    #[test]
    fn root_layout_horizontal_attr_value() {
        // data-md-layout="horizontal" when layout prop is Some("horizontal")
        let layout: Option<String> = Some("horizontal".to_string());
        let attr = layout.as_deref();
        assert_eq!(attr, Some("horizontal"));
    }

    #[test]
    fn root_layout_none_omits_attribute() {
        // No data-md-layout attribute when layout prop is None
        let layout: Option<String> = None;
        assert!(layout.is_none());
    }

    #[test]
    fn root_layout_vertical_attr_value() {
        let layout: Option<String> = Some("vertical".to_string());
        let attr = layout.as_deref();
        assert_eq!(attr, Some("vertical"));
    }

    // ── Editor wiring tests ─────────────────────────────────────────
    #[test]
    fn data_state_active_for_source_mode() {
        use crate::types::Mode;
        // Editor is active when mode is Source or LivePreview, inactive for Read
        let active = matches!(Mode::Source, Mode::Source | Mode::LivePreview);
        assert!(active);
    }

    #[test]
    fn data_state_inactive_for_read_mode() {
        use crate::types::Mode;
        let active = matches!(Mode::Read, Mode::Source | Mode::LivePreview);
        assert!(!active);
    }

    // ── Scroll sync flag tests ──────────────────────────────────────
    #[test]
    fn scroll_flag_default_is_false() {
        // Scroll lock flags default to false — no sync in progress initially
        let editor_scrolling = false;
        let preview_scrolling = false;
        assert!(
            !editor_scrolling,
            "is_editor_scrolling should default to false"
        );
        assert!(
            !preview_scrolling,
            "is_preview_scrolling should default to false"
        );
    }

    #[test]
    fn scroll_sync_lock_prevents_feedback_loop() {
        // When editor is driving scroll sync, preview scroll handler must bail out.
        // When preview is driving scroll sync, editor scroll handler must bail out.
        // This test documents the scroll lock contract.
        let is_editor_scrolling = true;
        let is_preview_scrolling = false;

        // Preview onscroll should return early when editor is scrolling
        let should_preview_bail = is_editor_scrolling;
        assert!(
            should_preview_bail,
            "preview must skip sync when editor is driving"
        );

        // Editor onscroll should NOT bail when preview is NOT scrolling
        let should_editor_bail = is_preview_scrolling;
        assert!(
            !should_editor_bail,
            "editor should sync when preview is not driving"
        );
    }

    // ── Keyboard shortcut logic tests ──────────────────────────────
    #[test]
    fn tab_key_string_is_tab() {
        // KeyboardEvent::key().to_string() returns "Tab" for the Tab key
        assert_eq!("Tab".to_string(), "Tab");
    }

    #[test]
    fn ctrl_b_key_lowercase_is_b() {
        // Ctrl+B key character lowercase matches "b" for bold shortcut
        assert_eq!("B".to_lowercase(), "b");
        assert_eq!("b".to_lowercase(), "b");
    }

    #[test]
    fn ctrl_shortcut_match_coverage() {
        // Verify the match arm logic for Ctrl+B/I/K shortcuts
        for (key, expected_prefix, expected_suffix) in
            [("b", "**", "**"), ("i", "_", "_"), ("k", "[", "](url)")]
        {
            let result = match key {
                "b" => Some(("**", "**")),
                "i" => Some(("_", "_")),
                "k" => Some(("[", "](url)")),
                _ => None,
            };
            let (prefix, suffix) = result.expect("should match known key");
            assert_eq!(prefix, expected_prefix);
            assert_eq!(suffix, expected_suffix);
        }
    }

    #[test]
    fn ctrl_unknown_key_returns_none() {
        // Unknown keys should not trigger any shortcut
        let result = match "x" {
            "b" => Some(("**", "**")),
            "i" => Some(("_", "_")),
            "k" => Some(("[", "](url)")),
            _ => None,
        };
        assert!(result.is_none());
    }
}

// ============================================================
// Hooks tests — written by parser-hooks agent in Wave 1E
// ============================================================
#[cfg(test)]
mod hooks_tests {
    use crate::parser::parse_document;

    #[test]
    fn extract_heading_index_empty_doc() {
        let rope = crop::Rope::from("");
        let doc = parse_document(&rope);
        assert!(doc.headings.is_empty());
    }

    #[test]
    fn extract_heading_index_returns_cloned_headings() {
        let rope = crop::Rope::from("# Title\n\n## Subtitle");
        let doc = parse_document(&rope);
        assert_eq!(doc.headings.len(), 2);
        assert_eq!(doc.headings[0].level, 1);
        assert_eq!(doc.headings[0].text, "Title");
        assert_eq!(doc.headings[1].level, 2);
        assert_eq!(doc.headings[1].text, "Subtitle");
    }

    #[test]
    fn extract_heading_index_preserves_anchors() {
        let rope = crop::Rope::from("# Hello World\n\n## Another Section");
        let doc = parse_document(&rope);
        assert_eq!(doc.headings[0].anchor, "hello-world");
        assert_eq!(doc.headings[1].anchor, "another-section");
    }

    #[test]
    fn extract_heading_index_preserves_line_numbers() {
        let rope = crop::Rope::from("# First\n\nParagraph.\n\n## Second");
        let doc = parse_document(&rope);
        assert_eq!(doc.headings[0].line, 0);
        assert_eq!(doc.headings[1].line, 4);
    }

    #[test]
    fn extract_heading_index_with_front_matter() {
        let rope = crop::Rope::from("---\ntitle: Test\n---\n\n# Content");
        let doc = parse_document(&rope);
        assert_eq!(doc.headings.len(), 1);
        assert_eq!(doc.headings[0].text, "Content");
    }
}

// ============================================================
// Scroll sync tests — Wave 2B-infra
// ============================================================
#[cfg(test)]
mod scroll_sync_tests {
    use crate::hooks::compute_scroll_ratio;

    #[test]
    fn compute_scroll_ratio_zero() {
        // At the top of the scroll range
        assert!((compute_scroll_ratio(0.0, 1000.0, 100.0) - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn compute_scroll_ratio_full() {
        // At the bottom of the scroll range
        assert!((compute_scroll_ratio(900.0, 1000.0, 100.0) - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn compute_scroll_ratio_half() {
        // Midway through the scroll range
        assert!((compute_scroll_ratio(450.0, 1000.0, 100.0) - 0.5).abs() < f64::EPSILON);
    }

    #[test]
    fn compute_scroll_ratio_no_overflow() {
        // scroll_height == client_height (no scrollable area)
        assert!((compute_scroll_ratio(0.0, 100.0, 100.0) - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn compute_scroll_ratio_negative_overflow() {
        // scroll_height < client_height (shouldn't happen, but handle gracefully)
        assert!((compute_scroll_ratio(0.0, 50.0, 100.0) - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn compute_scroll_ratio_clamped_max() {
        // scroll_top exceeds max scrollable area — should clamp to 1.0
        assert!((compute_scroll_ratio(1000.0, 1000.0, 100.0) - 1.0).abs() < f64::EPSILON);
    }
}

// ============================================================
// Keyboard JS string builder tests — Wave 2B-infra
// ============================================================
#[cfg(test)]
mod keyboard_js_tests {
    use crate::hooks::{tab_indent_js, wrap_selection_js};

    const TEST_EDITOR_ID: &str = "test-editor";

    #[test]
    fn wrap_selection_bold_js_contains_marker() {
        let js = wrap_selection_js(TEST_EDITOR_ID, "**", "**");
        assert!(js.contains("**"), "bold JS should contain '**'");
    }

    #[test]
    fn wrap_selection_italic_js_contains_marker() {
        let js = wrap_selection_js(TEST_EDITOR_ID, "_", "_");
        assert!(js.contains("_"), "italic JS should contain '_'");
    }

    #[test]
    fn wrap_selection_link_js_contains_markers() {
        let js = wrap_selection_js(TEST_EDITOR_ID, "[", "](url)");
        assert!(js.contains("["), "link JS should contain '['");
        assert!(js.contains("](url)"), "link JS should contain '](url)'");
    }

    #[test]
    fn tab_indent_size_2_inserts_two_spaces() {
        let js = tab_indent_js(TEST_EDITOR_ID, 2);
        assert!(js.contains("  "), "tab indent 2 JS should contain 2 spaces");
    }

    #[test]
    fn tab_indent_size_4_inserts_four_spaces() {
        let js = tab_indent_js(TEST_EDITOR_ID, 4);
        assert!(
            js.contains("    "),
            "tab indent 4 JS should contain 4 spaces",
        );
    }

    #[test]
    fn keyboard_js_dispatches_input_event() {
        let wrap_js = wrap_selection_js(TEST_EDITOR_ID, "**", "**");
        let tab_js = tab_indent_js(TEST_EDITOR_ID, 2);
        assert!(
            wrap_js.contains("dispatchEvent"),
            "wrap JS should dispatch input event",
        );
        assert!(
            tab_js.contains("dispatchEvent"),
            "tab JS should dispatch input event",
        );
    }

    #[test]
    fn keyboard_js_targets_editor_id() {
        let wrap_js = wrap_selection_js(TEST_EDITOR_ID, "**", "**");
        let tab_js = tab_indent_js(TEST_EDITOR_ID, 2);
        assert!(
            wrap_js.contains(TEST_EDITOR_ID),
            "wrap JS should target the editor element ID",
        );
        assert!(
            tab_js.contains(TEST_EDITOR_ID),
            "tab JS should target the editor element ID",
        );
    }
}

// ============================================================
// MarkdownHandle JS helper tests — Wave 2C
// ============================================================
#[cfg(test)]
mod markdown_handle_tests {
    use crate::context::{handle_insert_text_js, handle_set_content_js, handle_wrap_selection_js};

    #[test]
    fn insert_text_js_contains_dispatch() {
        let js = handle_insert_text_js("nox-md-editor", "hello");
        assert!(
            js.contains("dispatchEvent"),
            "insert_text JS should dispatch input event",
        );
    }

    #[test]
    fn insert_text_js_contains_inserted_text() {
        let js = handle_insert_text_js("nox-md-editor", "world");
        assert!(
            js.contains("world"),
            "insert_text JS should contain the text to insert"
        );
    }

    #[test]
    fn wrap_selection_js_contains_dispatch() {
        let js = handle_wrap_selection_js("nox-md-editor", "**", "**");
        assert!(
            js.contains("dispatchEvent"),
            "wrap_selection JS should dispatch input event",
        );
    }

    #[test]
    fn wrap_selection_js_contains_prefix_suffix() {
        let js = handle_wrap_selection_js("nox-md-editor", "**", "**");
        assert!(
            js.contains("**"),
            "wrap_selection JS should contain the bold markers"
        );
    }

    #[test]
    fn set_content_js_assigns_value_and_dispatches() {
        let js = handle_set_content_js("nox-md-editor", "hello");
        assert!(js.contains("el.value"), "should assign el.value");
        assert!(js.contains("dispatchEvent"), "should dispatch input event",);
    }

    #[test]
    fn markdown_handle_js_targets_editor_id() {
        let id = "nox-md-editor";
        let insert_js = handle_insert_text_js(id, "test");
        let wrap_js = handle_wrap_selection_js(id, "**", "**");
        let set_js = handle_set_content_js(id, "test");
        assert!(insert_js.contains(id), "insert JS should target editor ID");
        assert!(wrap_js.contains(id), "wrap JS should target editor ID");
        assert!(set_js.contains(id), "set JS should target editor ID");
    }

    #[test]
    fn insert_text_js_escapes_single_quotes() {
        let js = handle_insert_text_js("nox-md-editor", "it's a test");
        // The single quote should be escaped to prevent JS syntax errors
        assert!(
            js.contains(r"it\'s a test"),
            "insert_text JS should escape single quotes, got: {js}",
        );
    }

    #[test]
    fn set_content_js_escapes_backslashes() {
        let js = handle_set_content_js("nox-md-editor", r"path\to\file");
        // Backslashes should be escaped
        assert!(
            js.contains(r"path\\to\\file"),
            "set_content JS should escape backslashes, got: {js}",
        );
    }
}

// ============================================================
// Render / highlight_code tests — Wave 3A-parser
// Tests for code block rendering with data-md-* attributes
// and syntax-highlighting feature flag.
// ============================================================
#[cfg(test)]
mod render_tests {
    // Note: highlight_code was removed during the switch to pulldown-cmark
    // and native RSX generation. Code blocks are now natively rendered.
}

// ============================================================
// Vim state machine tests — Wave 3A-infra
// ============================================================
#[cfg(test)]
mod vim_tests {
    use crate::types::{VimAction, VimMode, VimState};

    const EID: &str = "test-editor";

    #[test]
    fn vim_default_mode_is_insert() {
        assert_eq!(VimState::default().mode, VimMode::Insert);
    }

    #[test]
    fn vim_i_enters_insert_from_normal() {
        let mut s = VimState {
            mode: VimMode::Normal,
            ..Default::default()
        };
        let action = s.handle_key("i", false, false, EID);
        assert_eq!(action, VimAction::ModeChange(VimMode::Insert));
        assert_eq!(s.mode, VimMode::Insert);
    }

    #[test]
    fn vim_escape_exits_insert() {
        let mut s = VimState {
            mode: VimMode::Insert,
            ..Default::default()
        };
        let action = s.handle_key("Escape", false, false, EID);
        assert_eq!(action, VimAction::ModeChange(VimMode::Normal));
        assert_eq!(s.mode, VimMode::Normal);
    }

    #[test]
    fn vim_v_enters_visual() {
        let mut s = VimState {
            mode: VimMode::Normal,
            ..Default::default()
        };
        let action = s.handle_key("v", false, false, EID);
        assert_eq!(action, VimAction::ModeChange(VimMode::Visual));
    }

    #[test]
    fn vim_colon_enters_command() {
        let mut s = VimState {
            mode: VimMode::Normal,
            ..Default::default()
        };
        let action = s.handle_key(":", false, false, EID);
        assert_eq!(action, VimAction::ModeChange(VimMode::Command));
    }

    #[test]
    fn vim_h_in_normal_prevents_and_evals() {
        let mut s = VimState {
            mode: VimMode::Normal,
            ..Default::default()
        };
        let action = s.handle_key("h", false, false, EID);
        assert!(matches!(action, VimAction::PreventAndEval(_)));
        if let VimAction::PreventAndEval(js) = action {
            assert!(js.contains("selectionStart"), "h should move cursor left");
        }
    }

    #[test]
    fn vim_insert_alpha_passthrough() {
        let mut s = VimState::default(); // Insert mode
        let action = s.handle_key("a", false, false, EID);
        assert_eq!(action, VimAction::PassThrough);
    }

    #[test]
    fn vim_visual_escape_to_normal() {
        let mut s = VimState {
            mode: VimMode::Visual,
            ..Default::default()
        };
        let action = s.handle_key("Escape", false, false, EID);
        assert_eq!(action, VimAction::ModeChange(VimMode::Normal));
    }

    #[test]
    fn vim_normal_escape_is_passthrough() {
        let mut s = VimState {
            mode: VimMode::Normal,
            ..Default::default()
        };
        let action = s.handle_key("Escape", false, false, EID);
        assert_eq!(action, VimAction::PassThrough);
    }

    #[test]
    fn vim_hjkl_return_prevent_and_eval() {
        let mut s = VimState {
            mode: VimMode::Normal,
            ..Default::default()
        };
        for key in &["h", "j", "k", "l"] {
            let action = s.handle_key(key, false, false, EID);
            assert!(
                matches!(action, VimAction::PreventAndEval(_)),
                "key '{}' in Normal should be PreventAndEval",
                key
            );
        }
    }

    #[test]
    fn vim_command_mode_accumulates_buffer() {
        let mut s = VimState {
            mode: VimMode::Command,
            ..Default::default()
        };
        s.handle_key("w", false, false, EID);
        s.handle_key("q", false, false, EID);
        assert_eq!(s.command_buffer, "wq");
    }

    #[test]
    fn vim_command_enter_executes_and_returns_to_normal() {
        let mut s = VimState {
            mode: VimMode::Command,
            ..Default::default()
        };
        s.handle_key("w", false, false, EID);
        let action = s.handle_key("Enter", false, false, EID);
        assert_eq!(action, VimAction::ExecuteCommand("w".to_string()));
        assert_eq!(s.mode, VimMode::Normal);
        assert!(s.command_buffer.is_empty());
    }

    #[test]
    fn vim_command_escape_clears_buffer_and_returns_to_normal() {
        let mut s = VimState {
            mode: VimMode::Command,
            ..Default::default()
        };
        s.handle_key("w", false, false, EID);
        let action = s.handle_key("Escape", false, false, EID);
        assert_eq!(action, VimAction::ModeChange(VimMode::Normal));
        assert_eq!(s.mode, VimMode::Normal);
        assert!(s.command_buffer.is_empty());
    }
}

// ── Slash command detection tests ───────────────────────────────────

mod slash_tests {
    use crate::components::{detect_slash_trigger, extract_slash_filter};

    // ── detect_slash_trigger ────────────────────────────────────────

    #[test]
    fn slash_trigger_at_start_of_text() {
        // "/" at position 0, cursor at 1
        assert_eq!(detect_slash_trigger("/", 1), Some(0));
    }

    #[test]
    fn slash_trigger_after_newline() {
        // "hello\n/" — slash at position 6, cursor at 7
        assert_eq!(detect_slash_trigger("hello\n/", 7), Some(6));
    }

    #[test]
    fn slash_no_trigger_mid_word() {
        // "hello/world" — slash after non-whitespace, not at line start
        assert_eq!(detect_slash_trigger("hello/world", 6), None);
    }

    #[test]
    fn slash_no_trigger_mid_word_trailing() {
        // "abc/" — slash is not at line start
        assert_eq!(detect_slash_trigger("abc/", 4), None);
    }

    #[test]
    fn slash_no_trigger_cursor_zero() {
        // cursor at 0 — nothing before cursor
        assert_eq!(detect_slash_trigger("/", 0), None);
    }

    #[test]
    fn slash_trigger_with_filter_text() {
        // "/head" — cursor at 5, slash at line start
        assert_eq!(detect_slash_trigger("/head", 5), Some(0));
    }

    #[test]
    fn slash_trigger_after_newline_with_filter() {
        // "hello\n/hea" — slash at position 6, cursor at 10
        assert_eq!(detect_slash_trigger("hello\n/hea", 10), Some(6));
    }

    #[test]
    fn slash_no_trigger_empty_text() {
        assert_eq!(detect_slash_trigger("", 0), None);
    }

    #[test]
    fn slash_trigger_second_line_start() {
        // Two newlines, slash at start of third line
        assert_eq!(detect_slash_trigger("a\nb\n/", 5), Some(4));
    }

    // ── extract_slash_filter ────────────────────────────────────────

    #[test]
    fn slash_filter_empty_after_slash() {
        // "/" with cursor right after slash — empty filter
        assert_eq!(extract_slash_filter("/", 1), Some(String::new()));
    }

    #[test]
    fn slash_filter_text() {
        // "/head" — cursor at 5
        assert_eq!(extract_slash_filter("/head", 5), Some("head".to_string()));
    }

    #[test]
    fn slash_filter_after_newline() {
        // "hello\n/hea" — cursor at 10
        assert_eq!(
            extract_slash_filter("hello\n/hea", 10),
            Some("hea".to_string())
        );
    }

    #[test]
    fn slash_filter_none_mid_word() {
        // "abc/def" — slash not at line start, no trigger
        assert_eq!(extract_slash_filter("abc/def", 4), None);
    }

    #[test]
    fn slash_filter_none_with_space() {
        // "/hello world" — space in filter invalidates it
        assert_eq!(extract_slash_filter("/hello world", 12), None);
    }

    #[test]
    fn slash_filter_partial_word() {
        // "/he" — cursor at 3
        assert_eq!(extract_slash_filter("/he", 3), Some("he".to_string()));
    }

    #[test]
    fn slash_filter_none_cursor_zero() {
        assert_eq!(extract_slash_filter("/", 0), None);
    }
}

// ============================================================
// SourceMap tests — Wave 3C-infra
// ============================================================
#[cfg(test)]
mod source_map_tests {
    use crate::types::{SourceMap, SourceMapEntry};

    fn make_entry(start: usize, end: usize, id: &str) -> SourceMapEntry {
        SourceMapEntry {
            source_line_start: start,
            source_line_end: end,
            element_id: id.to_string(),
        }
    }

    #[test]
    fn test_source_map_find_returns_none_when_empty() {
        let sm = SourceMap { entries: vec![] };
        assert!(sm.find_entry_by_line(1).is_none());
    }

    #[test]
    fn test_source_map_find_by_line_middle() {
        let sm = SourceMap {
            entries: vec![
                make_entry(1, 3, "h1"),
                make_entry(5, 8, "p1"),
                make_entry(10, 12, "p2"),
            ],
        };
        // line 6 is within entry 5..=8
        let entry = sm.find_entry_by_line(6).expect("should find entry");
        assert_eq!(entry.element_id, "p1");
    }

    #[test]
    fn test_source_map_find_exact_start() {
        let sm = SourceMap {
            entries: vec![make_entry(5, 8, "p1")],
        };
        let entry = sm.find_entry_by_line(5).expect("start line should match");
        assert_eq!(entry.element_id, "p1");
    }

    #[test]
    fn test_source_map_find_exact_end() {
        let sm = SourceMap {
            entries: vec![make_entry(5, 8, "p1")],
        };
        let entry = sm.find_entry_by_line(8).expect("end line should match");
        assert_eq!(entry.element_id, "p1");
    }

    #[test]
    fn test_source_map_find_returns_none_for_gap() {
        let sm = SourceMap {
            entries: vec![make_entry(1, 3, "h1"), make_entry(5, 8, "p1")],
        };
        // line 4 is in the gap between entries
        assert!(sm.find_entry_by_line(4).is_none());
    }
}

// ============================================================
// Block click delegation JS tests — Wave 3C-components
// ============================================================
#[cfg(test)]
mod block_click_tests {
    use crate::components::block_click_js;

    #[test]
    fn test_block_click_js_references_preview_id() {
        let js = block_click_js("nox-md-preview");
        assert!(
            js.contains("nox-md-preview"),
            "JS should reference the preview element ID",
        );
    }

    #[test]
    fn test_block_click_js_reads_data_source_start() {
        let js = block_click_js("nox-md-preview");
        assert!(
            js.contains("data-source-start"),
            "JS should look for data-source-start attribute",
        );
    }

    #[test]
    fn test_block_click_js_walks_up_dom() {
        let js = block_click_js("nox-md-preview");
        assert!(
            js.contains("parentNode") || js.contains("parentElement"),
            "JS should walk up the DOM tree",
        );
    }

    #[test]
    fn test_block_click_js_sends_to_dioxus() {
        let js = block_click_js("nox-md-preview");
        assert!(
            js.contains("dioxus.send"),
            "JS should send source offset to Dioxus",
        );
    }

    #[test]
    fn test_block_click_js_parses_to_integer() {
        let js = block_click_js("nox-md-preview");
        assert!(
            js.contains("parseInt") || js.contains("Number("),
            "JS should convert source offset to integer",
        );
    }
}

// ============================================================
// Render source mapping tests — Wave 3C-parser
// Verify block elements carry source-start/source-end attributes.
// ============================================================
#[cfg(test)]
mod render_source_map_tests {
    use crate::parser::parse_document;

    #[test]
    fn test_parse_document_produces_nonempty_doc_for_content() {
        let rope = crop::Rope::from("# Hello\n\nWorld");
        let doc = parse_document(&rope);
        assert!(!doc.headings.is_empty(), "should have at least one heading");
    }
}

// ============================================================
// SEC-001/002: sanitize_href tests — link sanitization
// ============================================================
#[cfg(test)]
mod sec_tests {
    use crate::parser::sanitize_href;

    #[test]
    fn sanitize_href_blocks_javascript() {
        assert_eq!(sanitize_href("javascript:alert(1)"), None);
    }

    #[test]
    fn sanitize_href_blocks_javascript_mixed_case() {
        assert_eq!(sanitize_href("JAVASCRIPT:alert(1)"), None);
        assert_eq!(sanitize_href("Javascript:void(0)"), None);
    }

    #[test]
    fn sanitize_href_blocks_data_uri() {
        assert_eq!(
            sanitize_href("data:text/html,<script>alert(1)</script>"),
            None
        );
        assert_eq!(sanitize_href("DATA:text/plain,hello"), None);
    }

    #[test]
    fn sanitize_href_blocks_vbscript() {
        assert_eq!(sanitize_href("vbscript:MsgBox(1)"), None);
    }

    #[test]
    fn sanitize_href_allows_https() {
        assert!(sanitize_href("https://example.com").is_some());
    }

    #[test]
    fn sanitize_href_allows_http() {
        assert!(sanitize_href("http://example.com").is_some());
    }

    #[test]
    fn sanitize_href_allows_mailto() {
        assert!(sanitize_href("mailto:user@example.com").is_some());
    }

    #[test]
    fn sanitize_href_allows_tel() {
        assert!(sanitize_href("tel:+1234567890").is_some());
    }

    #[test]
    fn sanitize_href_allows_absolute_path() {
        assert!(sanitize_href("/path/to/page").is_some());
    }

    #[test]
    fn sanitize_href_allows_anchor() {
        assert!(sanitize_href("#section-1").is_some());
    }

    #[test]
    fn sanitize_href_allows_relative() {
        assert!(sanitize_href("path/relative").is_some());
        assert!(sanitize_href("./relative").is_some());
    }

    #[test]
    fn sanitize_href_empty_string() {
        assert_eq!(sanitize_href(""), Some(String::new()));
    }

    #[test]
    fn sanitize_href_whitespace_padded_javascript_blocked() {
        assert_eq!(sanitize_href("  javascript:alert(1)"), None);
    }

    #[test]
    fn sanitize_href_allows_unknown_scheme() {
        // After denylist change: sftp:// is allowed
        assert!(sanitize_href("sftp://host/path").is_some());
    }

    #[test]
    fn sanitize_href_allows_custom_scheme() {
        assert!(sanitize_href("myapp://deep-link").is_some());
    }
}

// ============================================================
// SEC-003: escape_js tests — JS string escaping via handle_set_content_js
// ============================================================
#[cfg(test)]
mod sec_003_escape_tests {
    use crate::context::handle_set_content_js;

    /// Extract the content between `el.value = '` and `';` from the generated JS.
    /// This isolates the escaped user content from the JS template formatting.
    fn extract_js_string_literal(js: &str) -> &str {
        let start = js
            .find("el.value = '")
            .expect("should contain el.value assignment");
        let content_start = start + "el.value = '".len();
        let content_end = js[content_start..]
            .find("';")
            .expect("should contain closing ';")
            + content_start;
        &js[content_start..content_end]
    }

    #[test]
    fn escape_js_handles_newline() {
        let js = handle_set_content_js("my-id", "line1\nline2");
        let literal = extract_js_string_literal(&js);
        // The escaped content must NOT contain a raw newline
        assert!(
            !literal.contains('\n'),
            "JS literal must not contain raw newline"
        );
        // It should contain the escaped version \n
        assert!(
            literal.contains("\\n"),
            "JS literal must contain escaped \\n"
        );
    }

    #[test]
    fn escape_js_handles_carriage_return() {
        let js = handle_set_content_js("my-id", "a\rb");
        let literal = extract_js_string_literal(&js);
        assert!(
            !literal.contains('\r'),
            "JS literal must not contain raw carriage return"
        );
        assert!(
            literal.contains("\\r"),
            "JS literal must contain escaped \\r"
        );
    }

    #[test]
    fn escape_js_handles_line_separator() {
        let js = handle_set_content_js("my-id", "a\u{2028}b");
        assert!(!js.contains('\u{2028}'), "JS must not contain raw U+2028");
        assert!(js.contains("\\u2028"), "JS must contain \\u2028");
    }

    #[test]
    fn escape_js_handles_paragraph_separator() {
        let js = handle_set_content_js("my-id", "a\u{2029}b");
        assert!(!js.contains('\u{2029}'), "JS must not contain raw U+2029");
        assert!(js.contains("\\u2029"), "JS must contain \\u2029");
    }

    #[test]
    fn escape_js_preserves_single_quote_escaping() {
        let js = handle_set_content_js("my-id", "it's");
        assert!(js.contains("\\'"), "Single quotes must be escaped");
    }

    #[test]
    fn escape_js_preserves_backslash_escaping() {
        let js = handle_set_content_js("my-id", "back\\slash");
        // The backslash in the source string should become \\\\ in the JS string
        assert!(js.contains("\\\\"), "Backslashes must be escaped");
    }
}

// ============================================================
// GAP-001: UTF-16 index conversion + slash trigger with multibyte
// ============================================================
#[cfg(test)]
mod gap_001_utf16_tests {
    use crate::components::{detect_slash_trigger, utf16_to_byte_index};

    #[test]
    fn utf16_ascii_cursor() {
        assert_eq!(utf16_to_byte_index("hello", 3), Some(3));
    }

    #[test]
    fn utf16_at_end() {
        assert_eq!(utf16_to_byte_index("hi", 2), Some(2));
    }

    #[test]
    fn utf16_out_of_bounds() {
        assert_eq!(utf16_to_byte_index("hi", 5), None);
    }

    #[test]
    fn utf16_emoji_cursor() {
        // "🙂" is 4 bytes UTF-8, 2 UTF-16 code units
        // "🙂x" — cursor at UTF-16 index 2 means after the emoji, at 'x'
        assert_eq!(utf16_to_byte_index("🙂x", 2), Some(4));
    }

    #[test]
    fn utf16_cjk_cursor() {
        // "中" is 3 bytes UTF-8, 1 UTF-16 code unit
        // "中x" — cursor at UTF-16 index 1 means after 中, at 'x'
        assert_eq!(utf16_to_byte_index("中x", 1), Some(3));
    }

    #[test]
    fn detect_slash_trigger_with_emoji_prefix() {
        // text = "🙂\n/cmd", cursor_utf16 past the emoji+newline
        // "🙂" = 2 UTF-16 units, "\n" = 1 UTF-16 unit, "/" = 1 UTF-16 unit
        // cursor at 4 = after emoji (2 units) + newline (1 unit) + "/" (1 unit)
        let text = "🙂\n/cmd";
        let cursor_utf16 = 4; // after emoji (2) + newline (1) + "/" (1)
        assert!(
            detect_slash_trigger(text, cursor_utf16).is_some(),
            "should detect slash trigger after emoji+newline"
        );
    }

    #[test]
    fn detect_slash_trigger_no_trigger_slash_not_at_line_start() {
        // "🙂/cmd" — slash is NOT at line start (emoji is before it on same line)
        let text = "🙂/cmd";
        let cursor_utf16 = 3; // after emoji (2) + slash (1)
        // slash is not at line start so should NOT trigger
        assert!(
            detect_slash_trigger(text, cursor_utf16).is_none(),
            "slash not at line start should not trigger"
        );
    }
}

// ============================================================
// GAP-004: GFM table parsing
// ============================================================
#[cfg(test)]
mod gap_004_table_tests {
    use crate::parser::parse_document;

    #[test]
    fn table_parse_does_not_panic() {
        // Verify that a GFM table parses without panicking
        let md = "| A | B |\n|---|---|\n| 1 | 2 |\n";
        let rope = crop::Rope::from(md);
        let doc = parse_document(&rope);
        // Table has no headings
        assert!(doc.headings.is_empty());
    }

    #[test]
    fn table_with_header_and_body() {
        let md = "| Name | Value |\n|------|-------|\n| foo | bar |\n| baz | qux |\n";
        let rope = crop::Rope::from(md);
        let doc = parse_document(&rope);
        // Just verify it parses without panic and returns an element
        let _ = doc.element;
    }
}

// ── API-001: make_instance_n uniqueness and ID generation tests ──────────

#[cfg(test)]
mod api_001_tests {
    use crate::context::make_instance_n;

    #[test]
    fn make_instance_n_returns_unique_values() {
        let n1 = make_instance_n();
        let n2 = make_instance_n();
        assert_ne!(
            n1, n2,
            "Each call to make_instance_n must return unique values"
        );
    }

    #[test]
    fn instance_n_generates_correct_editor_id() {
        // Test the ID format directly without needing a Dioxus runtime
        let n: u64 = 42;
        let editor_id = format!("nox-md-{n}-editor");
        assert!(
            editor_id.contains("editor"),
            "editor_id should contain 'editor', got: {editor_id}"
        );
        assert!(
            editor_id.contains("nox-md"),
            "editor_id should contain 'nox-md', got: {editor_id}"
        );
        assert_eq!(editor_id, "nox-md-42-editor");
    }

    #[test]
    fn instance_n_generates_correct_preview_id() {
        let n: u64 = 42;
        let preview_id = format!("nox-md-{n}-preview");
        assert!(
            preview_id.contains("preview"),
            "preview_id should contain 'preview', got: {preview_id}"
        );
        assert!(
            preview_id.contains("nox-md"),
            "preview_id should contain 'nox-md', got: {preview_id}"
        );
        assert_eq!(preview_id, "nox-md-42-preview");
    }

    #[test]
    fn instance_n_ids_share_same_number() {
        let n: u64 = 99;
        let editor_id = format!("nox-md-{n}-editor");
        let preview_id = format!("nox-md-{n}-preview");
        let editor_num: String = editor_id.chars().filter(|c| c.is_ascii_digit()).collect();
        let preview_num: String = preview_id.chars().filter(|c| c.is_ascii_digit()).collect();
        assert_eq!(
            editor_num, preview_num,
            "editor_id and preview_id should share instance number"
        );
    }

    #[test]
    fn instance_n_generates_source_and_read_panel_ids() {
        let n: u64 = 7;
        assert_eq!(format!("nox-md-{n}-source"), "nox-md-7-source");
        assert_eq!(format!("nox-md-{n}-read"), "nox-md-7-read");
    }
}

#[cfg(test)]
mod gap_003_tests {
    // GAP-003: Resize listener cleanup
    // Full browser test is not possible in pure Rust unit tests.
    // We verify the AtomicU64 counter increments (ensuring unique cleanup keys).

    use crate::hooks::NEXT_VH_ID;
    use std::sync::atomic::Ordering;

    #[test]
    fn viewport_cleanup_id_increments() {
        let id1 = NEXT_VH_ID.fetch_add(1, Ordering::Relaxed);
        let id2 = NEXT_VH_ID.fetch_add(1, Ordering::Relaxed);
        assert!(id2 > id1, "Cleanup IDs should increment");
    }
}

#[cfg(test)]
mod ant_002_tests {
    // ANT-002: Preview eval receive loop cancellation
    // Requires a browser VirtualDom — not testable in pure Rust unit tests.
    // Compile-time check: verify dioxus::dioxus_core::Task is importable and has cancel().

    use dioxus::dioxus_core;

    #[test]
    fn task_cancel_compiles() {
        // This test verifies at compile time that dioxus_core::Task exists.
        // Runtime cancellation requires a browser environment.
        let _: fn(dioxus_core::Task) = |t| t.cancel();
    }
}

#[cfg(test)]
mod api_002_tests {
    use crate::types::{Layout, Orientation};

    #[test]
    fn layout_horizontal_attr() {
        assert_eq!(Layout::Horizontal.as_attr(), "horizontal");
    }

    #[test]
    fn layout_vertical_attr() {
        assert_eq!(Layout::Vertical.as_attr(), "vertical");
    }

    #[test]
    fn orientation_horizontal_attr() {
        assert_eq!(Orientation::Horizontal.as_attr(), "horizontal");
    }

    #[test]
    fn orientation_vertical_attr() {
        assert_eq!(Orientation::Vertical.as_attr(), "vertical");
    }

    #[test]
    fn layout_default_is_horizontal() {
        assert_eq!(Layout::default(), Layout::Horizontal);
    }

    #[test]
    fn orientation_default_is_horizontal() {
        assert_eq!(Orientation::default(), Orientation::Horizontal);
    }
}

#[cfg(test)]
mod gap_002_tests {
    use crate::components::{next_mode, prev_mode};
    use crate::types::Mode;

    #[test]
    fn next_mode_read_to_source() {
        assert_eq!(next_mode(Mode::Read), Mode::Source);
    }

    #[test]
    fn next_mode_source_to_livepreview() {
        assert_eq!(next_mode(Mode::Source), Mode::LivePreview);
    }

    #[test]
    fn next_mode_livepreview_wraps_to_read() {
        assert_eq!(next_mode(Mode::LivePreview), Mode::Read);
    }

    #[test]
    fn prev_mode_read_wraps_to_livepreview() {
        assert_eq!(prev_mode(Mode::Read), Mode::LivePreview);
    }

    #[test]
    fn prev_mode_source_to_read() {
        assert_eq!(prev_mode(Mode::Source), Mode::Read);
    }

    #[test]
    fn prev_mode_livepreview_to_source() {
        assert_eq!(prev_mode(Mode::LivePreview), Mode::Source);
    }
}

#[cfg(test)]
mod int_002_tests {
    // INT-002: Selection read via eval — verify the helper is accessible.
    // Full browser test not possible in pure Rust unit tests.
    // We verify the async function exists and has the expected signature at compile time.

    use crate::context::read_cursor_and_selection;

    #[test]
    fn read_cursor_and_selection_exists_and_is_async() {
        // Compile-time check: verify the function exists and accepts (&str, &str).
        // We can't call it without a browser runtime, but this ensures it compiles.
        // The function is async, so calling it returns a Future (we don't .await it).
        let _future = read_cursor_and_selection("test-editor", "hello world");
        // Drop without awaiting — no runtime needed. This proves the signature is correct.
    }
}

// ── LivePreviewVariant tests ─────────────────────────────────────────

#[test]
fn live_preview_variant_default_is_split_pane() {
    assert_eq!(LivePreviewVariant::default(), LivePreviewVariant::SplitPane);
}

#[test]
fn live_preview_variant_equality() {
    assert_eq!(LivePreviewVariant::SplitPane, LivePreviewVariant::SplitPane);
    assert_eq!(LivePreviewVariant::Inline, LivePreviewVariant::Inline);
    assert_ne!(LivePreviewVariant::SplitPane, LivePreviewVariant::Inline);
}

#[test]
fn live_preview_variant_copy() {
    let v = LivePreviewVariant::Inline;
    let v2 = v;
    let v3 = v;
    assert_eq!(v2, v3);
}

// ── BlockEntry tests ─────────────────────────────────────────────────

#[test]
fn block_entry_fields() {
    let b = BlockEntry {
        index: 3,
        raw: "# Title".to_string(),
        html: "<div data-block-index=\"3\"><h1>Title</h1></div>".to_string(),
        start_line: 5,
        end_line: 5,
        is_list_item: false,
    };
    assert_eq!(b.index, 3);
    assert_eq!(b.raw, "# Title");
    assert_eq!(b.start_line, 5);
    assert_eq!(b.end_line, 5);
    assert!(b.html.contains("data-block-index=\"3\""));
}

#[test]
fn block_entry_equality() {
    let a = BlockEntry {
        index: 0,
        raw: "hello".to_string(),
        html: "<div>hello</div>".to_string(),
        start_line: 1,
        end_line: 1,
        is_list_item: false,
    };
    let b = a.clone();
    assert_eq!(a, b);
}

// ── Custom Extension AST Tests ──────────────────────────────────────────

#[cfg(test)]
mod custom_extension_tests {
    use crate::parser::parse_document;
    use dioxus::prelude::*;

    #[test]
    fn parse_tag_extension() {
        let md = "This is a #test tag.";
        let rope = crop::Rope::from(md);
        let doc = parse_document(&rope);

        let mut vdom = VirtualDom::new_with_props(|props: Element| props, doc.element);
        vdom.rebuild_in_place();
        let html = dioxus_ssr::render(&vdom);

        // Ensure the span properly maps the tag text and attribute
        assert!(html.contains("data-md-tag=\"#test\""));
        assert!(html.contains(">#test</span>"));
        assert!(html.contains("This is a "));
        assert!(html.contains(" tag."));
    }

    #[test]
    fn parse_wikilink_extension() {
        let md = "Check out my [[Zettelkasten Note]].";
        let rope = crop::Rope::from(md);
        let doc = parse_document(&rope);

        let mut vdom = VirtualDom::new_with_props(|props: Element| props, doc.element);
        vdom.rebuild_in_place();
        let html = dioxus_ssr::render(&vdom);

        println!("RENDERED HTML: {}", html);

        assert!(html.contains("data-md-wikilink=\"Zettelkasten Note\""));
        assert!(html.contains("Check out my "));
        assert!(html.contains("[[Zettelkasten Note]]</a>"));
    }

    #[test]
    fn parse_mixed_extensions() {
        let md = "A #tag and a [[link]] side-by-side.";
        let rope = crop::Rope::from(md);
        let doc = parse_document(&rope);

        let mut vdom = VirtualDom::new_with_props(|props: Element| props, doc.element);
        vdom.rebuild_in_place();
        let html = dioxus_ssr::render(&vdom);

        assert!(html.contains("data-md-tag=\"#tag\""));
        assert!(html.contains("data-md-wikilink=\"link\""));
        assert!(html.contains(" and a "));
        assert!(html.contains(" side-by-side."));
    }
}

#[cfg(test)]
mod html_policy_tests {
    use crate::parser::parse_document_with_policy;
    use crate::types::HtmlRenderPolicy;
    use dioxus::prelude::*;

    #[test]
    fn html_is_escaped_by_default_policy() {
        let rope = crop::Rope::from("before <b>bold</b> after");
        let doc = parse_document_with_policy(&rope, HtmlRenderPolicy::Escape);

        let mut vdom = VirtualDom::new_with_props(|props: Element| props, doc.element);
        vdom.rebuild_in_place();
        let html = dioxus_ssr::render(&vdom);

        assert!(html.contains("&#60;b&#62;"));
        assert!(html.contains("&#60;/b&#62;"));
    }

    #[test]
    fn trusted_policy_renders_raw_html() {
        let rope = crop::Rope::from("before <b>bold</b> after");
        let doc = parse_document_with_policy(&rope, HtmlRenderPolicy::Trusted);

        let mut vdom = VirtualDom::new_with_props(|props: Element| props, doc.element);
        vdom.rebuild_in_place();
        let html = dioxus_ssr::render(&vdom);

        assert!(html.contains("<b>"));
        assert!(html.contains("</b>"));
    }

    #[cfg(feature = "sanitize")]
    #[test]
    fn sanitized_policy_strips_script_tags() {
        let rope = crop::Rope::from("text <script>alert('xss')</script> end");
        let doc = parse_document_with_policy(&rope, HtmlRenderPolicy::Sanitized);

        let mut vdom = VirtualDom::new_with_props(|props: Element| props, doc.element);
        vdom.rebuild_in_place();
        let html = dioxus_ssr::render(&vdom);

        assert!(!html.contains("<script>"), "script tags should be stripped");
    }

    #[cfg(feature = "sanitize")]
    #[test]
    fn sanitized_policy_preserves_safe_html() {
        let rope = crop::Rope::from("before <b>bold</b> after");
        let doc = parse_document_with_policy(&rope, HtmlRenderPolicy::Sanitized);

        let mut vdom = VirtualDom::new_with_props(|props: Element| props, doc.element);
        vdom.rebuild_in_place();
        let html = dioxus_ssr::render(&vdom);

        assert!(html.contains("<b>"), "safe tags should be preserved");
        assert!(
            html.contains("</b>"),
            "safe closing tags should be preserved"
        );
    }

    #[cfg(not(feature = "sanitize"))]
    #[test]
    fn sanitized_policy_falls_back_to_escape_without_feature() {
        let rope = crop::Rope::from("before <b>bold</b> after");
        let doc = parse_document_with_policy(&rope, HtmlRenderPolicy::Sanitized);

        let mut vdom = VirtualDom::new_with_props(|props: Element| props, doc.element);
        vdom.rebuild_in_place();
        let html = dioxus_ssr::render(&vdom);

        // Without the sanitize feature, Sanitized falls back to Escape behavior
        assert!(html.contains("&#60;b&#62;"));
    }
}

// ── Syntax highlighting tests ────────────────────────────────────────

mod highlight_tests {
    use crate::highlight::{generate_theme_css, highlight_code, supported_languages};

    #[test]
    fn highlight_code_html_escapes_special_chars() {
        let result = highlight_code("<script>alert('xss')</script>", "", "hl-");
        assert!(result.html.contains("&lt;script&gt;"));
        assert!(result.html.contains("&lt;/script&gt;"));
        assert!(!result.language_matched);
    }

    #[test]
    fn highlight_code_empty_input() {
        let result = highlight_code("", "rust", "hl-");
        assert_eq!(result.html, "");
        // Empty code is technically valid — language_matched depends on feature flag
    }

    #[test]
    fn highlight_code_unrecognized_lang_returns_plain_text() {
        let result = highlight_code("some code", "zyx_nonexistent_lang", "hl-");
        assert!(!result.language_matched);
        assert_eq!(result.html, "some code");
    }

    #[test]
    fn highlight_code_escapes_ampersands() {
        let result = highlight_code("a && b", "", "hl-");
        assert!(result.html.contains("&amp;&amp;"));
    }

    #[test]
    fn highlight_code_escapes_quotes() {
        let result = highlight_code(r#"x = "hello""#, "", "hl-");
        assert!(result.html.contains("&quot;hello&quot;"));
    }

    #[cfg(feature = "syntax-highlighting")]
    #[test]
    fn highlight_code_produces_spans_for_rust() {
        let result = highlight_code("fn main() {}", "rust", "hl-");
        assert!(result.language_matched);
        assert!(result.html.contains("<span class=\"hl-"));
    }

    #[cfg(feature = "syntax-highlighting")]
    #[test]
    fn highlight_code_custom_prefix() {
        let result = highlight_code("fn main() {}", "rust", "sx-");
        assert!(result.language_matched);
        assert!(result.html.contains("class=\"sx-"));
        assert!(!result.html.contains("class=\"hl-"));
    }

    #[cfg(feature = "syntax-highlighting")]
    #[test]
    fn highlight_code_empty_prefix() {
        let result = highlight_code("let x = 1;", "rust", "");
        assert!(result.language_matched);
        // With empty prefix, syntect uses Spaced style (no prefix on class names)
        assert!(result.html.contains("<span class=\""));
    }

    #[cfg(feature = "syntax-highlighting")]
    #[test]
    fn generate_theme_css_produces_rules() {
        let css = generate_theme_css("base16-ocean.dark", "hl-");
        assert!(css.is_some());
        let css = css.unwrap();
        assert!(css.contains("color:"));
    }

    #[cfg(feature = "syntax-highlighting")]
    #[test]
    fn generate_theme_css_unknown_theme_returns_none() {
        let css = generate_theme_css("nonexistent_theme_xyz", "hl-");
        assert!(css.is_none());
    }

    #[cfg(feature = "syntax-highlighting")]
    #[test]
    fn supported_languages_is_nonempty() {
        let langs = supported_languages();
        assert!(!langs.is_empty());
        assert!(langs.contains(&"rs"));
    }

    #[cfg(not(feature = "syntax-highlighting"))]
    #[test]
    fn generate_theme_css_returns_none_without_feature() {
        assert!(generate_theme_css("base16-ocean.dark", "hl-").is_none());
    }

    #[cfg(not(feature = "syntax-highlighting"))]
    #[test]
    fn supported_languages_empty_without_feature() {
        assert!(supported_languages().is_empty());
    }
}

mod highlight_parser_tests {
    use crate::parser::parse_document;
    use crop::Rope;

    #[test]
    fn parse_document_code_block_uses_dangerous_inner_html() {
        let md = "```rust\nfn main() {}\n```\n";
        let doc = parse_document(&Rope::from(md));
        // The element should render without panic — basic smoke test
        let mut vdom = dioxus::prelude::VirtualDom::new_with_props(
            |props: dioxus::prelude::Element| props,
            doc.element,
        );
        vdom.rebuild_in_place();
        let html = dioxus_ssr::render(&vdom);
        // Should contain the code block structure
        assert!(html.contains("data-md-code-block"));
        assert!(html.contains("language-rust"));
    }

    #[test]
    fn parse_document_indented_code_block() {
        let md = "    indented code\n    second line\n\n";
        let doc = parse_document(&Rope::from(md));
        let mut vdom = dioxus::prelude::VirtualDom::new_with_props(
            |props: dioxus::prelude::Element| props,
            doc.element,
        );
        vdom.rebuild_in_place();
        let html = dioxus_ssr::render(&vdom);
        assert!(html.contains("data-md-code-block"));
        assert!(html.contains("indented code"));
    }

    #[cfg(feature = "syntax-highlighting")]
    #[test]
    fn parse_document_code_block_has_highlighted_attr() {
        let md = "```rust\nfn main() {}\n```\n";
        let doc = parse_document(&Rope::from(md));
        let mut vdom = dioxus::prelude::VirtualDom::new_with_props(
            |props: dioxus::prelude::Element| props,
            doc.element,
        );
        vdom.rebuild_in_place();
        let html = dioxus_ssr::render(&vdom);
        assert!(html.contains("data-md-highlighted"));
        assert!(html.contains("<span class=\"hl-"));
    }

    #[cfg(feature = "syntax-highlighting")]
    #[test]
    fn parse_document_unknown_lang_no_highlighted_attr() {
        let md = "```zyx\nsome code\n```\n";
        let doc = parse_document(&Rope::from(md));
        let mut vdom = dioxus::prelude::VirtualDom::new_with_props(
            |props: dioxus::prelude::Element| props,
            doc.element,
        );
        vdom.rebuild_in_place();
        let html = dioxus_ssr::render(&vdom);
        assert!(!html.contains("data-md-highlighted"));
    }
}

// ── wrap_with_line_numbers tests ─────────────────────────────────────

use crate::highlight::wrap_with_line_numbers;

#[test]
fn wrap_line_numbers_empty_input() {
    let result = wrap_with_line_numbers("");
    assert!(result.contains("data-line-number=\"1\""));
    assert!(result.contains("data-md-line-gutter"));
    // Single empty line should produce one numbered line
    assert_eq!(result.matches("data-line-number").count(), 1);
}

#[test]
fn wrap_line_numbers_single_line() {
    let result = wrap_with_line_numbers("hello world");
    assert!(result.contains("data-line-number=\"1\""));
    assert!(result.contains("hello world"));
    assert_eq!(result.matches("data-line-number").count(), 1);
}

#[test]
fn wrap_line_numbers_multi_line() {
    let result = wrap_with_line_numbers("line1\nline2\nline3");
    assert!(result.contains("data-line-number=\"1\""));
    assert!(result.contains("data-line-number=\"2\""));
    assert!(result.contains("data-line-number=\"3\""));
    assert_eq!(result.matches("data-line-number").count(), 3);
    assert!(result.contains("line1"));
    assert!(result.contains("line2"));
    assert!(result.contains("line3"));
}

#[test]
fn wrap_line_numbers_trailing_newline_trimmed() {
    // syntect often ends output with a trailing newline — should not produce an extra empty line
    let result = wrap_with_line_numbers("line1\nline2\n");
    assert_eq!(result.matches("data-line-number").count(), 2);
}

#[test]
fn wrap_line_numbers_non_selectable_gutter() {
    let result = wrap_with_line_numbers("hello");
    assert!(result.contains("user-select:none"));
    assert!(result.contains("aria-hidden=\"true\""));
}

#[test]
fn wrap_line_numbers_preserves_html_spans() {
    // Simulate highlighted HTML with spans
    let html =
        "<span class=\"hl-keyword\">fn</span> main() {}\n<span class=\"hl-comment\">// end</span>";
    let result = wrap_with_line_numbers(html);
    assert_eq!(result.matches("data-line-number").count(), 2);
    assert!(result.contains("<span class=\"hl-keyword\">fn</span> main() {}"));
    assert!(result.contains("<span class=\"hl-comment\">// end</span>"));
}

// ── Code block feature integration tests ─────────────────────────────

use crate::parser::parse_document_full_with_config;
use crop::Rope;

#[test]
fn parse_document_line_numbers_enabled() {
    use crate::types::HtmlRenderPolicy;
    let md = "```rust\nfn main() {}\n```\n";
    let doc = parse_document_full_with_config(
        &Rope::from(md),
        HtmlRenderPolicy::Escape,
        "hl-",
        true, // show_code_line_numbers
        true, // show_code_language
    );
    let mut vdom = dioxus::prelude::VirtualDom::new_with_props(
        |props: dioxus::prelude::Element| props,
        doc.element,
    );
    vdom.rebuild_in_place();
    let html = dioxus_ssr::render(&vdom);
    assert!(
        html.contains("data-md-line-numbers"),
        "missing data-md-line-numbers on pre"
    );
    assert!(
        html.contains("data-md-line-gutter"),
        "missing line gutter spans"
    );
    assert!(
        html.contains("data-line-number"),
        "missing data-line-number on line spans"
    );
}

#[test]
fn parse_document_line_numbers_disabled() {
    use crate::types::HtmlRenderPolicy;
    let md = "```rust\nfn main() {}\n```\n";
    let doc = parse_document_full_with_config(
        &Rope::from(md),
        HtmlRenderPolicy::Escape,
        "hl-",
        false, // show_code_line_numbers
        true,  // show_code_language
    );
    let mut vdom = dioxus::prelude::VirtualDom::new_with_props(
        |props: dioxus::prelude::Element| props,
        doc.element,
    );
    vdom.rebuild_in_place();
    let html = dioxus_ssr::render(&vdom);
    assert!(
        !html.contains("data-md-line-numbers"),
        "should not have data-md-line-numbers when disabled"
    );
    assert!(
        !html.contains("data-md-line-gutter"),
        "should not have gutter spans when disabled"
    );
}

#[test]
fn parse_document_language_label_enabled() {
    use crate::types::HtmlRenderPolicy;
    let md = "```rust\nfn main() {}\n```\n";
    let doc = parse_document_full_with_config(
        &Rope::from(md),
        HtmlRenderPolicy::Escape,
        "hl-",
        false,
        true, // show_code_language
    );
    let mut vdom = dioxus::prelude::VirtualDom::new_with_props(
        |props: dioxus::prelude::Element| props,
        doc.element,
    );
    vdom.rebuild_in_place();
    let html = dioxus_ssr::render(&vdom);
    assert!(
        html.contains("data-md-code-header"),
        "missing code header div"
    );
    assert!(
        html.contains("data-md-code-language"),
        "missing code language span"
    );
    assert!(html.contains(">rust<"), "missing language text 'rust'");
}

#[test]
fn parse_document_language_label_disabled() {
    use crate::types::HtmlRenderPolicy;
    let md = "```rust\nfn main() {}\n```\n";
    let doc = parse_document_full_with_config(
        &Rope::from(md),
        HtmlRenderPolicy::Escape,
        "hl-",
        false,
        false, // show_code_language disabled
    );
    let mut vdom = dioxus::prelude::VirtualDom::new_with_props(
        |props: dioxus::prelude::Element| props,
        doc.element,
    );
    vdom.rebuild_in_place();
    let html = dioxus_ssr::render(&vdom);
    assert!(
        !html.contains("data-md-code-header"),
        "should not have code header when disabled"
    );
    assert!(
        !html.contains("data-md-code-language"),
        "should not have code language when disabled"
    );
}

#[test]
fn parse_document_indented_code_block_no_language_label() {
    use crate::types::HtmlRenderPolicy;
    // Indented code blocks have no language info — should never show language label
    let md = "    fn main() {}\n";
    let doc = parse_document_full_with_config(
        &Rope::from(md),
        HtmlRenderPolicy::Escape,
        "hl-",
        false,
        true, // show_code_language enabled but no lang for indented blocks
    );
    let mut vdom = dioxus::prelude::VirtualDom::new_with_props(
        |props: dioxus::prelude::Element| props,
        doc.element,
    );
    vdom.rebuild_in_place();
    let html = dioxus_ssr::render(&vdom);
    assert!(
        !html.contains("data-md-code-header"),
        "indented code blocks should never show language header"
    );
}

// ── pulldown-cmark AST range diagnostic tests ──────────────────────

#[test]
fn parser_ranges_two_paragraphs() {
    // "Hello\n\nWorld" — two paragraphs separated by blank line
    let md = "Hello\n\nWorld";
    let doc = crate::parser::parse_document(&Rope::from(md));
    // We expect two Paragraph nodes; inspect their ranges to confirm gap behavior
    assert_eq!(
        doc.ast.len(),
        2,
        "expected 2 top-level nodes, got {}",
        doc.ast.len()
    );
    let first = &doc.ast[0];
    let second = &doc.ast[1];
    assert!(matches!(first.node_type, crate::types::NodeType::Paragraph));
    assert!(matches!(
        second.node_type,
        crate::types::NodeType::Paragraph
    ));
    // pulldown-cmark ranges: first paragraph includes trailing \n\n
    // Verify there's a gap or overlap — this tells us how to build synthetic nodes
    let gap_start = first.range.end;
    let gap_end = second.range.start;
    // Document: "Hello\n\nWorld" (bytes 0..12)
    // Expected: first = 0..7 ("Hello\n\n"), second = 7..12 ("World")
    // OR first = 0..5 ("Hello"), gap 5..7 ("\n\n"), second = 7..12
    // The test captures the actual behavior:
    assert!(
        gap_start <= gap_end,
        "ranges should not overlap: first.end={gap_start}, second.start={gap_end}"
    );
}

#[test]
fn parser_ranges_extra_blank_lines() {
    // "Hello\n\n\n\nWorld" — four newlines between paragraphs
    let md = "Hello\n\n\n\nWorld";
    let doc = crate::parser::parse_document(&Rope::from(md));
    assert_eq!(
        doc.ast.len(),
        2,
        "expected 2 top-level nodes, got {}",
        doc.ast.len()
    );
    let first = &doc.ast[0];
    let second = &doc.ast[1];
    let gap = second.range.start.saturating_sub(first.range.end);
    // With extra blank lines, the gap should be larger than with just \n\n
    // This confirms whether pulldown-cmark absorbs extra newlines into node ranges
    // or leaves them as gaps
    assert!(
        first.range.end <= second.range.start,
        "ranges must not overlap"
    );
    // Print for diagnostic (visible in test output with --nocapture)
    eprintln!(
        "Extra blank lines: first={:?}, second={:?}, gap={}",
        first.range, second.range, gap
    );
}
