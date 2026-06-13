#![forbid(unsafe_code)]
#![doc = include_str!("../README.md")]

use std::{
    env,
    path::{Path, PathBuf},
};

use anyhow::{anyhow, bail};
use az_config_center_contract::{
    ShellComponent, ShellComponentBuildResult, ShellComponentKind, ShellComponentPatch,
    ShellComponentUpsert,
};

/// 将创建 / 更新请求规范化为完整 Shell 组件快照。
///
/// 该函数会修剪空白、丢弃空字符串字段，并预生成 `preview`，供 API 层保存或返回。
pub fn materialize_component(input: ShellComponentUpsert) -> anyhow::Result<ShellComponent> {
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

/// 校验创建 / 更新请求是否满足组件类型约束。
///
/// `Export` 必须提供 `export_value`，`Alias` 必须提供 `alias_command`，
/// `Function` 与 `Snippet` 必须提供 `body`。
pub fn validate_upsert(input: &ShellComponentUpsert) -> anyhow::Result<()> {
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

/// 校验局部更新请求是否至少包含一个可修改字段。
pub fn validate_patch(input: &ShellComponentPatch) -> anyhow::Result<()> {
    if input.summary.is_none() && input.enabled.is_none() && input.render_to_output.is_none() {
        bail!("patch request is empty");
    }
    if let Some(summary) = &input.summary {
        let _ = summary.trim();
    }
    Ok(())
}

/// 校验组件名是否适用于指定组件类型。
///
/// 所有组件名都不能留空、包含空白或 `=`；`Export` 还必须符合 shell 变量命名规则。
pub fn validate_component_name(name: &str, kind: ShellComponentKind) -> anyhow::Result<()> {
    let trimmed = name.trim();
    if trimmed.is_empty() {
        bail!("component name cannot be blank");
    }
    if trimmed.contains(char::is_whitespace) || trimmed.contains('=') {
        bail!("component name cannot contain whitespace or `=`");
    }
    if matches!(kind, ShellComponentKind::Export)
        && !trimmed.chars().enumerate().all(|(idx, ch)| match idx {
            0 => ch == '_' || ch.is_ascii_alphabetic(),
            _ => ch == '_' || ch.is_ascii_alphanumeric(),
        })
    {
        bail!("export component name must be a valid shell variable");
    }
    Ok(())
}

/// 将单个组件渲染为可写入 shell 文件的脚本片段。
pub fn render_component(component: &ShellComponent) -> anyhow::Result<String> {
    match component.kind {
        ShellComponentKind::Export => {
            let value = component
                .export_value
                .as_deref()
                .filter(|value| !value.trim().is_empty())
                .ok_or_else(|| {
                    anyhow!(
                        "export component `{}` is missing export_value",
                        component.name
                    )
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
                    anyhow!(
                        "alias component `{}` is missing alias_command",
                        component.name
                    )
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
            .ok_or_else(|| anyhow!("component `{}` is missing body", component.name)),
    }
}

/// 按组件类型和名称稳定排序，构建完整 shell 输出内容。
///
/// 返回值中的 `written` 固定为 `false`；实际写文件由上层 IO 边界负责。
pub fn build_output(
    config_path: &str,
    output_path: &str,
    components: &[ShellComponent],
) -> anyhow::Result<ShellComponentBuildResult> {
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
        content.push_str(&format!("# {}\n", kind.to_string()));
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

/// 展开常见 home 路径前缀。
///
/// 支持 `~`、`~/...`、`$HOME/...` 与 `${HOME}/...`，不访问文件系统。
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

fn render_component_block(component: &ShellComponent) -> anyhow::Result<String> {
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
            .cmp(&right.kind)
            .then_with(|| left.name.cmp(&right.name))
    });
    items
}

fn require_non_empty(value: Option<&str>, message: &str) -> anyhow::Result<()> {
    if value.map(str::trim).is_none_or(str::is_empty) {
        bail!("{message}");
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
