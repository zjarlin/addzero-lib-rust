use std::{
    collections::BTreeMap,
    env, fs,
    io::Write as _,
    path::{Path, PathBuf},
};

use az_config_center_contract::{
    DEFAULT_SHELL_OUTPUT_PATH, ShellComponent, ShellComponentBuildConfig,
    ShellComponentBuildRequest, ShellComponentBuildResult, ShellComponentConfigUpdate,
    ShellComponentPatch, ShellComponentRegistry, ShellComponentUpsert,
};
use az_shell_components::{build_output, expand_home_path, materialize_component, validate_patch};
use serde::{Deserialize, Serialize};

const CONFIG_FILE_NAME: &str = "shell-components.json";
const CONFIG_PATH_ENV: &str = "AIO_SHELL_COMPONENTS_CONFIG";
const OUTPUT_PATH_ENV: &str = "AIO_SHELL_COMPONENTS_OUTPUT";

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
struct PersistedShellComponentRegistry {
    #[serde(default = "current_config_version")]
    version: u32,
    #[serde(default)]
    build: PersistedShellComponentBuildConfig,
    #[serde(default)]
    components: BTreeMap<String, PersistedShellComponent>,
}

impl Default for PersistedShellComponentRegistry {
    fn default() -> Self {
        Self {
            version: current_config_version(),
            build: PersistedShellComponentBuildConfig::default(),
            components: BTreeMap::new(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
struct PersistedShellComponentBuildConfig {
    #[serde(default = "default_output_path")]
    output_path: String,
}

impl Default for PersistedShellComponentBuildConfig {
    fn default() -> Self {
        Self {
            output_path: default_output_path(),
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
struct PersistedShellComponent {
    name: String,
    kind: az_config_center_contract::ShellComponentKind,
    #[serde(default)]
    summary: String,
    #[serde(default = "default_true")]
    enabled: bool,
    #[serde(default = "default_true")]
    render_to_output: bool,
    #[serde(default)]
    export_value: Option<String>,
    #[serde(default)]
    alias_command: Option<String>,
    #[serde(default)]
    body: Option<String>,
}

pub fn load_shell_component_registry_on_server() -> Result<ShellComponentRegistry, String> {
    let path = registry_path()?;
    let registry = load_registry_at(&path)?;
    registry_to_contract(&path, registry)
}

pub fn get_shell_component_on_server(name: &str) -> Result<Option<ShellComponent>, String> {
    let path = registry_path()?;
    let registry = load_registry_at(&path)?;
    registry
        .components
        .get(name)
        .cloned()
        .map(materialize_persisted_component)
        .transpose()
}

pub fn upsert_shell_component_on_server(
    input: ShellComponentUpsert,
) -> Result<ShellComponent, String> {
    let path = registry_path()?;
    upsert_shell_component_at(&path, input)
}

pub fn patch_shell_component_on_server(
    input: ShellComponentPatch,
) -> Result<ShellComponent, String> {
    let path = registry_path()?;
    patch_shell_component_at(&path, input)
}

pub fn remove_shell_component_on_server(name: &str) -> Result<ShellComponent, String> {
    let path = registry_path()?;
    remove_shell_component_at(&path, name)
}

pub fn save_shell_component_config_on_server(
    input: ShellComponentConfigUpdate,
) -> Result<ShellComponentRegistry, String> {
    let path = registry_path()?;
    let mut registry = load_registry_at(&path)?;
    if let Some(output_path) = input.output_path {
        let trimmed = output_path.trim();
        if trimmed.is_empty() {
            return Err("output_path cannot be blank".to_string());
        }
        registry.build.output_path = trimmed.to_string();
    }
    save_registry_at(&path, &registry)?;
    registry_to_contract(&path, registry)
}

pub fn build_shell_components_on_server(
    input: ShellComponentBuildRequest,
) -> Result<ShellComponentBuildResult, String> {
    let path = registry_path()?;
    let registry = load_registry_at(&path)?;
    build_shell_components_at(&path, &registry, input)
}

pub fn current_shell_component_output_config() -> Result<ShellComponentBuildConfig, String> {
    let path = registry_path()?;
    let registry = load_registry_at(&path)?;
    let resolved_output_path = expand_home_path(PathBuf::from(&registry.build.output_path));
    Ok(ShellComponentBuildConfig {
        output_path: registry.build.output_path,
        resolved_output_path: resolved_output_path.display().to_string(),
    })
}

pub fn current_shell_component_registry_path() -> Result<PathBuf, String> {
    registry_path()
}

fn upsert_shell_component_at(
    path: &Path,
    input: ShellComponentUpsert,
) -> Result<ShellComponent, String> {
    let component = materialize_component(input).map_err(|err| err.to_string())?;
    let mut registry = load_registry_at(path)?;
    registry
        .components
        .insert(component.name.clone(), persisted_from_component(&component));
    save_registry_at(path, &registry)?;
    Ok(component)
}

fn patch_shell_component_at(
    path: &Path,
    input: ShellComponentPatch,
) -> Result<ShellComponent, String> {
    validate_patch(&input).map_err(|err| err.to_string())?;
    let mut registry = load_registry_at(path)?;
    let component = registry
        .components
        .get_mut(&input.name)
        .ok_or_else(|| format!("shell component `{}` does not exist", input.name))?;
    if let Some(summary) = input.summary {
        component.summary = summary.trim().to_string();
    }
    if let Some(enabled) = input.enabled {
        component.enabled = enabled;
    }
    if let Some(render_to_output) = input.render_to_output {
        component.render_to_output = render_to_output;
    }
    let materialized = materialize_persisted_component(component.clone())?;
    save_registry_at(path, &registry)?;
    Ok(materialized)
}

fn remove_shell_component_at(path: &Path, name: &str) -> Result<ShellComponent, String> {
    let mut registry = load_registry_at(path)?;
    let removed = registry
        .components
        .remove(name)
        .ok_or_else(|| format!("shell component `{name}` does not exist"))?;
    save_registry_at(path, &registry)?;
    materialize_persisted_component(removed)
}

fn build_shell_components_at(
    path: &Path,
    registry: &PersistedShellComponentRegistry,
    input: ShellComponentBuildRequest,
) -> Result<ShellComponentBuildResult, String> {
    let configured_output = input
        .output_path
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
        .unwrap_or_else(|| registry.build.output_path.clone());
    let output_path = expand_home_path(PathBuf::from(&configured_output));
    let components = registry
        .components
        .values()
        .cloned()
        .map(materialize_persisted_component)
        .collect::<Result<Vec<_>, _>>()?;
    let mut result = build_output(
        &path.display().to_string(),
        &output_path.display().to_string(),
        &components,
    )
    .map_err(|err| err.to_string())?;
    if input.write {
        write_file(&output_path, result.content.as_bytes(), 0o600)?;
        result.written = true;
    }
    Ok(result)
}

fn registry_to_contract(
    path: &Path,
    registry: PersistedShellComponentRegistry,
) -> Result<ShellComponentRegistry, String> {
    let output_path = registry.build.output_path.clone();
    let resolved_output_path = expand_home_path(PathBuf::from(&output_path));
    let mut components = registry
        .components
        .into_values()
        .map(materialize_persisted_component)
        .collect::<Result<Vec<_>, _>>()?;
    components.sort_by(|left, right| {
        left.kind
            .sort_key()
            .cmp(&right.kind.sort_key())
            .then_with(|| left.name.cmp(&right.name))
    });
    Ok(ShellComponentRegistry {
        config_path: path.display().to_string(),
        build: ShellComponentBuildConfig {
            output_path,
            resolved_output_path: resolved_output_path.display().to_string(),
        },
        components,
    })
}

fn materialize_persisted_component(
    component: PersistedShellComponent,
) -> Result<ShellComponent, String> {
    materialize_component(ShellComponentUpsert {
        name: component.name,
        kind: component.kind,
        summary: component.summary,
        enabled: component.enabled,
        render_to_output: component.render_to_output,
        export_value: component.export_value,
        alias_command: component.alias_command,
        body: component.body,
    })
    .map_err(|err| err.to_string())
}

fn persisted_from_component(component: &ShellComponent) -> PersistedShellComponent {
    PersistedShellComponent {
        name: component.name.clone(),
        kind: component.kind,
        summary: component.summary.clone(),
        enabled: component.enabled,
        render_to_output: component.render_to_output,
        export_value: component.export_value.clone(),
        alias_command: component.alias_command.clone(),
        body: component.body.clone(),
    }
}

fn load_registry_at(path: &Path) -> Result<PersistedShellComponentRegistry, String> {
    if !path.exists() {
        return Ok(PersistedShellComponentRegistry::default());
    }
    let raw = fs::read_to_string(path)
        .map_err(|err| format!("read shell component registry {}: {err}", path.display()))?;
    if raw.trim().is_empty() {
        return Ok(PersistedShellComponentRegistry::default());
    }
    serde_json::from_str(&raw)
        .map_err(|err| format!("parse shell component registry {}: {err}", path.display()))
}

fn save_registry_at(path: &Path, registry: &PersistedShellComponentRegistry) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|err| format!("create config dir {}: {err}", parent.display()))?;
    }
    let encoded = serde_json::to_vec_pretty(registry)
        .map_err(|err| format!("encode shell component registry: {err}"))?;
    write_file(path, &encoded, 0o600)?;
    append_newline(path)
}

fn registry_path() -> Result<PathBuf, String> {
    if let Some(path) = non_empty_env_path(CONFIG_PATH_ENV) {
        return Ok(expand_home_path(path));
    }
    if let Some(path) = az_persistence::local_env_path() {
        if let Some(parent) = path.parent() {
            return Ok(parent.join(CONFIG_FILE_NAME));
        }
    }
    aio_config_dir()
        .map(|dir| dir.join(CONFIG_FILE_NAME))
        .ok_or_else(|| "cannot resolve ~/.config/aio for shell components".to_string())
}

fn aio_config_dir() -> Option<PathBuf> {
    env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .or_else(|| env::var_os("HOME").map(|home| PathBuf::from(home).join(".config")))
        .map(|path| path.join("aio"))
}

fn non_empty_env_path(key: &str) -> Option<PathBuf> {
    env::var_os(key)
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
}

fn write_file(path: &Path, bytes: &[u8], mode: u32) -> Result<(), String> {
    if let Some(parent) = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
    {
        fs::create_dir_all(parent)
            .map_err(|err| format!("create parent dir {}: {err}", parent.display()))?;
    }
    let mut file = fs::OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(path)
        .map_err(|err| format!("open {}: {err}", path.display()))?;
    file.write_all(bytes)
        .map_err(|err| format!("write {}: {err}", path.display()))?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(path, fs::Permissions::from_mode(mode))
            .map_err(|err| format!("chmod {}: {err}", path.display()))?;
    }
    Ok(())
}

fn append_newline(path: &Path) -> Result<(), String> {
    let mut file = fs::OpenOptions::new()
        .append(true)
        .open(path)
        .map_err(|err| format!("open {} for append: {err}", path.display()))?;
    file.write_all(b"\n")
        .map_err(|err| format!("append newline to {}: {err}", path.display()))
}

fn default_output_path() -> String {
    non_empty_env_path(OUTPUT_PATH_ENV)
        .map(|path| path.to_string_lossy().to_string())
        .unwrap_or_else(|| DEFAULT_SHELL_OUTPUT_PATH.to_string())
}

fn default_true() -> bool {
    true
}

fn current_config_version() -> u32 {
    1
}

#[cfg(test)]
mod tests {
    use az_config_center_contract::{
        ShellComponentBuildRequest, ShellComponentKind, ShellComponentPatch, ShellComponentUpsert,
    };

    use super::{
        PersistedShellComponentRegistry, build_shell_components_at, load_registry_at,
        patch_shell_component_at, registry_to_contract, save_registry_at,
        upsert_shell_component_at,
    };

    #[test]
    fn upsert_should_persist_shell_component_registry() {
        let dir = tempfile::tempdir().expect("temp dir should work");
        let path = dir.path().join("shell-components.json");

        let saved = upsert_shell_component_at(
            &path,
            ShellComponentUpsert {
                name: "JAVA_HOME".to_string(),
                kind: ShellComponentKind::Export,
                summary: "jdk path".to_string(),
                enabled: true,
                render_to_output: true,
                export_value: Some("/Library/Java".to_string()),
                alias_command: None,
                body: None,
            },
        )
        .expect("upsert should succeed");

        assert_eq!(saved.name, "JAVA_HOME");
        assert_eq!(saved.preview, "export JAVA_HOME='/Library/Java'");

        let loaded = load_registry_at(&path).expect("registry should load");
        assert!(loaded.components.contains_key("JAVA_HOME"));
    }

    #[test]
    fn patch_should_toggle_component_flags() {
        let dir = tempfile::tempdir().expect("temp dir should work");
        let path = dir.path().join("shell-components.json");
        upsert_shell_component_at(
            &path,
            ShellComponentUpsert {
                name: "ll".to_string(),
                kind: ShellComponentKind::Alias,
                summary: String::new(),
                enabled: true,
                render_to_output: true,
                export_value: None,
                alias_command: Some("ls -lah".to_string()),
                body: None,
            },
        )
        .expect("upsert should work");

        let patched = patch_shell_component_at(
            &path,
            ShellComponentPatch {
                name: "ll".to_string(),
                summary: Some("list files".to_string()),
                enabled: Some(false),
                render_to_output: Some(false),
            },
        )
        .expect("patch should succeed");

        assert!(!patched.enabled);
        assert!(!patched.render_to_output);
        assert_eq!(patched.summary, "list files");
    }

    #[test]
    fn build_should_render_only_enabled_materialized_components() {
        let dir = tempfile::tempdir().expect("temp dir should work");
        let path = dir.path().join("shell-components.json");
        let output = dir.path().join(".add_fn");
        let mut registry = PersistedShellComponentRegistry::default();
        registry.build.output_path = output.display().to_string();
        save_registry_at(&path, &registry).expect("save empty registry should work");

        upsert_shell_component_at(
            &path,
            ShellComponentUpsert {
                name: "TZ".to_string(),
                kind: ShellComponentKind::Export,
                summary: "timezone".to_string(),
                enabled: true,
                render_to_output: true,
                export_value: Some("Asia/Shanghai".to_string()),
                alias_command: None,
                body: None,
            },
        )
        .expect("export upsert should work");
        upsert_shell_component_at(
            &path,
            ShellComponentUpsert {
                name: "ll".to_string(),
                kind: ShellComponentKind::Alias,
                summary: String::new(),
                enabled: true,
                render_to_output: true,
                export_value: None,
                alias_command: Some("ls -lah".to_string()),
                body: None,
            },
        )
        .expect("alias upsert should work");
        upsert_shell_component_at(
            &path,
            ShellComponentUpsert {
                name: "commonip".to_string(),
                kind: ShellComponentKind::Function,
                summary: String::new(),
                enabled: true,
                render_to_output: true,
                export_value: None,
                alias_command: None,
                body: Some("commonip() {\n  hostname\n}".to_string()),
            },
        )
        .expect("function upsert should work");
        upsert_shell_component_at(
            &path,
            ShellComponentUpsert {
                name: "legacy".to_string(),
                kind: ShellComponentKind::Snippet,
                summary: String::new(),
                enabled: false,
                render_to_output: true,
                export_value: None,
                alias_command: None,
                body: Some("echo old".to_string()),
            },
        )
        .expect("snippet upsert should work");

        let loaded = load_registry_at(&path).expect("registry should load");
        let result = build_shell_components_at(
            &path,
            &loaded,
            ShellComponentBuildRequest {
                output_path: None,
                write: true,
            },
        )
        .expect("build should succeed");

        assert!(result.written);
        assert_eq!(result.included_components, 3);
        assert!(result.content.contains("export TZ='Asia/Shanghai'"));
        assert!(result.content.contains("alias ll='ls -lah'"));
        assert!(result.content.contains("commonip() {"));
        assert!(!result.content.contains("echo old"));

        let written = std::fs::read_to_string(output).expect("output file should exist");
        assert_eq!(written, result.content);
    }

    #[test]
    fn registry_contract_should_resolve_output_path() {
        let dir = tempfile::tempdir().expect("temp dir should work");
        let path = dir.path().join("shell-components.json");
        let mut registry = PersistedShellComponentRegistry::default();
        registry.build.output_path = dir.path().join("custom.sh").display().to_string();

        let dto = registry_to_contract(&path, registry).expect("registry conversion should work");

        assert_eq!(
            dto.build.output_path,
            dir.path().join("custom.sh").display().to_string()
        );
        assert_eq!(
            dto.build.resolved_output_path,
            dir.path().join("custom.sh").display().to_string()
        );
    }
}
