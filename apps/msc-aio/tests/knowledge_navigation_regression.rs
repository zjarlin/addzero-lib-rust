#[test]
fn knowledge_scene_does_not_embed_a_second_section_tab_bar() {
    let source = include_str!("../src/scenes/knowledge_base.rs");

    for forbidden in ["WorkbenchTabs", "WorkbenchTabItem", "KnowledgeSectionTabs"] {
        assert!(
            !source.contains(forbidden),
            "knowledge_base.rs should not reintroduce duplicate section navigation: found {forbidden}"
        );
    }
}

#[test]
fn dashboard_home_uses_note_cards_instead_of_graph_canvas() {
    let source = include_str!("../src/scenes/dashboard.rs");

    assert!(
        source.contains("note-card-grid"),
        "dashboard should render the note-card grid surface"
    );

    for forbidden in [
        "graph-canvas",
        "layout_graph",
        "oncontextmenu",
        "知识图谱概览",
    ] {
        assert!(
            !source.contains(forbidden),
            "dashboard should not reintroduce graph overview behavior: found {forbidden}"
        );
    }
}

#[test]
fn command_search_entrypoints_are_declared() {
    let app_source = include_str!("../src/app.rs");
    let dashboard_source = include_str!("../src/scenes/dashboard.rs");
    let knowledge_source = include_str!("../src/scenes/knowledge_base.rs");

    assert!(app_source.contains("mscFocusCommandSearch"));
    assert!(app_source.contains("metaKey || event.ctrlKey"));
    assert!(dashboard_source.contains("data-command-search"));
    assert!(knowledge_source.contains("data-command-search"));
}
