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
