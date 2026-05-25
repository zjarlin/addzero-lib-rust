#![forbid(unsafe_code)]
#![doc = include_str!("../README.md")]

use std::{
    env,
    path::{Path, PathBuf},
};

use az_config_center_contract::{
    ShellComponent, ShellComponentBuildResult, ShellComponentKind, ShellComponentPatch,
    ShellComponentUpsert,
};
use az_derive_aliases::{apply, error_eq};

pub type ShellComponentResult<T> = Result<T, ShellComponentError>;

#[apply(error_eq)]
pub enum ShellComponentError {
    #[error("{0}")]
    Validation(String),
}

pub fn materialize_component(input: ShellComponentUpsert) -> ShellComponentResult<ShellComponent> {
    validate_upsert(&input)?;

    let mut component = ShellComponent {
        name: input.name.trim().to_string(),
        kind: input.kind,
        summary: input.summary.trim().to_string(),
        enabled: input.enabled,
        render_to_output: input.render_to_output,
        export_value: normalize_option(input.export_value),
        alias_command: normalize_option(input.alias_command),
        body: normalize_multiline_option(input.body),
        preview: String::new(),
    };
    component.preview = render_component(&component)?;
    Ok(component)
}

pub fn validate_upsert(input: &ShellComponentUpsert) -> ShellComponentResult<()> {
    validate_component_name(&input.name, input.kind)?;
    match input.kind {
        ShellComponentKind::Export => {
            require_non_empty(
                input.export_value.as_deref(),
                "export component requires --value",
            )?;
        }
        ShellComponentKind::Alias => {
            require_non_empty(
                input.alias_command.as_deref(),
                "alias component requires --command",
            )?;
        }
        ShellComponentKind::Function | ShellComponentKind::Snippet => {
            require_non_empty(
                input.body.as_deref(),
                "function/snippet component requires --body",
            )?;
        }
    }
    Ok(())
}

pub fn validate_patch(input: &ShellComponentPatch) -> ShellComponentResult<()> {
    if input.summary.is_none() && input.enabled.is_none() && input.render_to_output.is_none() {
        return Err(ShellComponentError::Validation(
            "patch request is empty".to_string(),
        ));
    }
    if let Some(summary) = &input.summary {
        let _ = summary.trim();
    }
    Ok(())
}

pub fn validate_component_name(name: &str, kind: ShellComponentKind) -> ShellComponentResult<()> {
    let trimmed = name.trim();
    if trimmed.is_empty() {
        return Err(ShellComponentError::Validation(
            "component name cannot be blank".to_string(),
        ));
    }
    if trimmed.contains(char::is_whitespace) || trimmed.contains('=') {
        return Err(ShellComponentError::Validation(
            "component name cannot contain whitespace or `=`".to_string(),
        ));
    }
    if matches!(kind, ShellComponentKind::Export)
        && !trimmed.chars().enumerate().all(|(idx, ch)| match idx {
            0 => ch == '_' || ch.is_ascii_alphabetic(),
            _ => ch == '_' || ch.is_ascii_alphanumeric(),
        })
    {
        return Err(ShellComponentError::Validation(
            "export component name must be a valid shell variable".to_string(),
        ));
    }
    Ok(())
}

pub fn render_component(component: &ShellComponent) -> ShellComponentResult<String> {
    match component.kind {
        ShellComponentKind::Export => {
            let value = component
                .export_value
                .as_deref()
                .filter(|value| !value.trim().is_empty())
                .ok_or_else(|| {
                    ShellComponentError::Validation(format!(
                        "export component `{}` is missing export_value",
                        component.name
                    ))
                })?;
            Ok(format!(
                "export {}={}",
                component.name.trim(),
                shell_quote(value.trim())
            ))
        }
        ShellComponentKind::Alias => {
            let command = component
                .alias_command
                .as_deref()
                .filter(|value| !value.trim().is_empty())
                .ok_or_else(|| {
                    ShellComponentError::Validation(format!(
                        "alias component `{}` is missing alias_command",
                        component.name
                    ))
                })?;
            Ok(format!(
                "alias {}={}",
                component.name.trim(),
                shell_quote(command.trim())
            ))
        }
        ShellComponentKind::Function | ShellComponentKind::Snippet => component
            .body
            .as_deref()
            .filter(|value| !value.trim().is_empty())
            .map(ToOwned::to_owned)
            .ok_or_else(|| {
                ShellComponentError::Validation(format!(
                    "component `{}` is missing body",
                    component.name
                ))
            }),
    }
}

pub fn build_output(
    config_path: &str,
    output_path: &str,
    components: &[ShellComponent],
) -> ShellComponentResult<ShellComponentBuildResult> {
    let included = included_components(components);
    let mut content = String::new();
    content.push_str("# Generated by aio shell component builder.\n");
    content.push_str(&format!("# Source: {config_path}\n"));

    for &kind in ShellComponentKind::ALL {
        let mut section_items = Vec::new();
        for component in included.iter().filter(|component| component.kind == kind) {
            section_items.push(render_component_block(component)?);
        }
        if section_items.is_empty() {
            continue;
        }
        content.push('\n');
        content.push_str(&format!("# {}\n", kind.section_title()));
        for item in section_items {
            content.push('\n');
            content.push_str(&item);
            if !item.ends_with('\n') {
                content.push('\n');
            }
        }
    }

    if !content.ends_with('\n') {
        content.push('\n');
    }

    Ok(ShellComponentBuildResult {
        config_path: config_path.to_string(),
        output_path: output_path.to_string(),
        written: false,
        total_components: components.len(),
        included_components: included.len(),
        skipped_components: components.len().saturating_sub(included.len()),
        included_names: included
            .into_iter()
            .map(|component| component.name)
            .collect(),
        content,
    })
}

pub fn expand_home_path(path: impl AsRef<Path>) -> PathBuf {
    let path = path.as_ref().to_path_buf();
    let raw = path.to_string_lossy();
    let Some(home) = env::var_os("HOME").map(PathBuf::from) else {
        return path;
    };
    if raw == "~" || raw == "$HOME" || raw == "${HOME}" {
        return home;
    }
    if let Some(rest) = raw.strip_prefix("~/") {
        return home.join(rest);
    }
    if let Some(rest) = raw.strip_prefix("$HOME/") {
        return home.join(rest);
    }
    if let Some(rest) = raw.strip_prefix("${HOME}/") {
        return home.join(rest);
    }
    path
}

fn render_component_block(component: &ShellComponent) -> ShellComponentResult<String> {
    let mut block = String::new();
    if !component.summary.trim().is_empty() {
        block.push_str(&format!(
            "# {}: {}\n",
            component.name.trim(),
            component.summary.trim()
        ));
    }
    block.push_str(&render_component(component)?);
    Ok(block)
}

fn included_components(components: &[ShellComponent]) -> Vec<ShellComponent> {
    let mut items = components
        .iter()
        .filter(|component| component.enabled && component.render_to_output)
        .cloned()
        .collect::<Vec<_>>();
    items.sort_by(|left, right| {
        left.kind
            .sort_key()
            .cmp(&right.kind.sort_key())
            .then_with(|| left.name.cmp(&right.name))
    });
    items
}

fn require_non_empty(value: Option<&str>, message: &str) -> ShellComponentResult<()> {
    if value.map(str::trim).is_none_or(str::is_empty) {
        return Err(ShellComponentError::Validation(message.to_string()));
    }
    Ok(())
}

fn shell_quote(raw: &str) -> String {
    format!("'{}'", raw.replace('\'', "'\"'\"'"))
}

fn normalize_option(value: Option<String>) -> Option<String> {
    value
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

fn normalize_multiline_option(value: Option<String>) -> Option<String> {
    value
        .map(|value| value.replace("\r\n", "\n").trim_matches('\n').to_string())
        .filter(|value| !value.trim().is_empty())
}

#[cfg(test)]
mod tests {
    use az_config_center_contract::{
        ShellComponentKind, ShellComponentPatch, ShellComponentUpsert,
    };

    use super::{
        ShellComponentError, build_output, expand_home_path, materialize_component,
        render_component, validate_component_name, validate_patch,
    };

    #[test]
    fn validates_export_names() {
        let result = validate_component_name("1BAD", ShellComponentKind::Export);
        assert_eq!(
            result,
            Err(ShellComponentError::Validation(
                "export component name must be a valid shell variable".to_string()
            ))
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
            missing_export_value,
            Err(ShellComponentError::Validation(
                "export component requires --value".to_string()
            ))
        );

        let empty_patch = validate_patch(&ShellComponentPatch {
            name: "ll".to_string(),
            summary: None,
            enabled: None,
            render_to_output: None,
        });
        assert_eq!(
            empty_patch,
            Err(ShellComponentError::Validation(
                "patch request is empty".to_string()
            ))
        );
    }

    #[test]
    fn expands_home_tokens_without_touching_filesystem() {
        let path = expand_home_path("~/demo");
        assert!(path.to_string_lossy().contains("/demo"));
    }
}
