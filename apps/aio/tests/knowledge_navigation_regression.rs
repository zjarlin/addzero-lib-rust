#[test]
fn migration_moves_admin_navigation_to_next_shell() {
    let readme = include_str!("../../../README.md");

    assert!(
        readme.contains("Next.js 管理界面"),
        "README should document the Next.js admin delivery"
    );
    assert!(
        !readme.contains("Dioxus 管理界面"),
        "README should not advertise the removed Dioxus admin shell"
    );
}
