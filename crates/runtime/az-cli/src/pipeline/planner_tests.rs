use super::planner::build_plan;

#[test]
fn splits_text_into_multiple_scenes() {
    let input = format!(
        "{}\n\n{}\n\n{}",
        "第一段".repeat(40),
        "第二段".repeat(40),
        "第三段".repeat(40)
    );
    let plan = build_plan(&input, 80);

    assert!(plan.scenes.len() >= 2);
    assert_eq!(plan.scenes[0].index, 1);
}
