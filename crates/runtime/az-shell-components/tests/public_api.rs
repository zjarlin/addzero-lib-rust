use az_config_center_contract::api::{
    ShellComponentKind, ShellComponentPatch, ShellComponentUpsert,
};
use az_shell_components::{
    build_output, expand_home_path, materialize_component, render_component,
    validate_component_name, validate_patch,
};

#[test]
fn validates_export_names() {
    let result = validate_component_name("1BAD", ShellComponentKind::Export);

    // Export 名最终会进入 shell 环境变量，必须拒绝数字开头。
    assert_eq!(
        result
            .expect_err("invalid export name should be rejected")
            .to_string(),
        "export component name must be a valid shell variable"
    );
}

#[test]
fn renders_export_alias_function_and_snippet() {
    let export = materialize_component(ShellComponentUpsert {
        name: "JAVA_HOME".to_string(),
        kind: ShellComponentKind::Export,
        summary: "jdk".to_string(),
        enabled: true,
        render_to_output: true,
        export_value: Some("/Library/Java".to_string()),
        alias_command: None,
        body: None,
    })
    .expect("export should materialize");
    assert_eq!(
        render_component(&export).expect("export should render"),
        "export JAVA_HOME='/Library/Java'"
    );

    let alias = materialize_component(ShellComponentUpsert {
        name: "ll".to_string(),
        kind: ShellComponentKind::Alias,
        summary: String::new(),
        enabled: true,
        render_to_output: true,
        export_value: None,
        alias_command: Some("ls -lah".to_string()),
        body: None,
    })
    .expect("alias should materialize");
    assert_eq!(
        render_component(&alias).expect("alias should render"),
        "alias ll='ls -lah'"
    );

    let function = materialize_component(ShellComponentUpsert {
        name: "commonip".to_string(),
        kind: ShellComponentKind::Function,
        summary: String::new(),
        enabled: true,
        render_to_output: true,
        export_value: None,
        alias_command: None,
        body: Some("commonip() {\n  hostname\n}".to_string()),
    })
    .expect("function should materialize");
    assert!(
        render_component(&function)
            .expect("function should render")
            .contains("commonip() {")
    );

    let snippet = materialize_component(ShellComponentUpsert {
        name: "snippet-demo".to_string(),
        kind: ShellComponentKind::Snippet,
        summary: String::new(),
        enabled: true,
        render_to_output: true,
        export_value: None,
        alias_command: None,
        body: Some("echo ok".to_string()),
    })
    .expect("snippet should materialize");
    assert_eq!(
        render_component(&snippet).expect("snippet should render"),
        "echo ok"
    );
}

#[test]
fn build_groups_components_by_section_and_order() {
    let export = materialize_component(ShellComponentUpsert {
        name: "TZ".to_string(),
        kind: ShellComponentKind::Export,
        summary: "timezone".to_string(),
        enabled: true,
        render_to_output: true,
        export_value: Some("Asia/Shanghai".to_string()),
        alias_command: None,
        body: None,
    })
    .expect("export should materialize");
    let alias = materialize_component(ShellComponentUpsert {
        name: "ll".to_string(),
        kind: ShellComponentKind::Alias,
        summary: String::new(),
        enabled: true,
        render_to_output: true,
        export_value: None,
        alias_command: Some("ls -lah".to_string()),
        body: None,
    })
    .expect("alias should materialize");
    let function = materialize_component(ShellComponentUpsert {
        name: "commonip".to_string(),
        kind: ShellComponentKind::Function,
        summary: String::new(),
        enabled: true,
        render_to_output: true,
        export_value: None,
        alias_command: None,
        body: Some("commonip() {\n  hostname\n}".to_string()),
    })
    .expect("function should materialize");
    let skipped = materialize_component(ShellComponentUpsert {
        name: "legacy".to_string(),
        kind: ShellComponentKind::Snippet,
        summary: String::new(),
        enabled: false,
        render_to_output: true,
        export_value: None,
        alias_command: None,
        body: Some("echo old".to_string()),
    })
    .expect("snippet should materialize");

    let result = build_output(
        "/tmp/shell-components.json",
        "/tmp/.add_fn",
        &[function, skipped, alias, export],
    )
    .expect("build should work");

    // 构建输出必须只包含启用且允许渲染的组件，并保持稳定分组顺序。
    assert_eq!(result.included_components, 3);
    assert!(result.content.contains("# exports"));
    assert!(result.content.contains("# aliases"));
    assert!(result.content.contains("# functions"));
    assert!(!result.content.contains("echo old"));
    assert_eq!(result.included_names, vec!["TZ", "ll", "commonip"]);
}

#[test]
fn rejects_invalid_field_combinations() {
    let missing_export_value = materialize_component(ShellComponentUpsert {
        name: "JAVA_HOME".to_string(),
        kind: ShellComponentKind::Export,
        summary: String::new(),
        enabled: true,
        render_to_output: true,
        export_value: None,
        alias_command: None,
        body: None,
    });
    assert_eq!(
        missing_export_value
            .expect_err("missing export value should be rejected")
            .to_string(),
        "export component requires --value"
    );

    let empty_patch = validate_patch(&ShellComponentPatch {
        name: "ll".to_string(),
        summary: None,
        enabled: None,
        render_to_output: None,
    });
    // 空 patch 没有可落库的语义，应该在服务层之前被拒绝。
    assert_eq!(
        empty_patch
            .expect_err("empty patch should be rejected")
            .to_string(),
        "patch request is empty"
    );
}

#[test]
fn expands_home_tokens_without_touching_filesystem() {
    let path = expand_home_path("~/demo");
    assert!(path.to_string_lossy().contains("/demo"));
}
