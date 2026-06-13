use az_config_center_contract::ShellComponentKind;

#[test]
fn shell_component_kind_keeps_wire_codes() {
    assert_eq!(ShellComponentKind::Alias.code(), "alias");
    assert_eq!(
        ShellComponentKind::from_code("function"),
        Some(ShellComponentKind::Function)
    );
    assert_eq!(ShellComponentKind::Export.to_string(), "exports");
    assert!(ShellComponentKind::Export < ShellComponentKind::Alias);
    assert_eq!(
        serde_json::to_string(&ShellComponentKind::Snippet)
            .expect("shell component kind should serialize"),
        "\"snippet\""
    );
}
