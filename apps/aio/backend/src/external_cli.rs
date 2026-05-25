use std::{
    collections::BTreeMap,
    env, fs,
    io::Write as _,
    path::{Path, PathBuf},
    process::Stdio,
};

use anyhow::{Context, bail};
use az_derive_aliases::{apply, serde_eq_default};
use tokio::process::Command;

use crate::{
    cli::{
        AioCliCommand, AioCliComponentCommand, AioCliSkillCommand, CliOutputFormat,
        ExternalCliAddArgs, ExternalCliListArgs, ExternalCliRemoveArgs, ExternalCliRunArgs,
        ShellComponentBuildArgs, ShellComponentConfigArgs, ShellComponentGetArgs,
        ShellComponentKindArg, ShellComponentListArgs, ShellComponentRemoveArgs,
        ShellComponentSetArgs, ShellComponentUpsertArgs,
    },
    cli_metadata::{AIO_COMMANDS, render_metadata_table, render_skill_md, render_skill_sh},
};

const CONFIG_FILE_NAME: &str = "external-cli.json";
const SKILL_DIR_NAME: &str = "aio-cli";

#[apply(serde_eq_default)]
pub struct ExternalCliConfig {
    #[serde(default = "current_config_version")]
    pub version: u32,
    #[serde(default)]
    pub commands: BTreeMap<String, ExternalCliDefinition>,
}

#[apply(serde_eq_default)]
pub struct ExternalCliDefinition {
    pub name: String,
    pub command: String,
    #[serde(default)]
    pub args: Vec<String>,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub working_dir: Option<String>,
    #[serde(default)]
    pub env: BTreeMap<String, String>,
}

pub async fn run_aio_cli_command(command: AioCliCommand) -> anyhow::Result<()> {
    match command {
        AioCliCommand::Metadata(args) => print_metadata(args.format)?,
        AioCliCommand::Skill(AioCliSkillCommand::Install(args)) => {
            let path = install_skill(args.root.as_deref(), args.path.as_deref())?;
            println!("skill.sh 已写入: {}", path.display());
        }
        AioCliCommand::Skill(AioCliSkillCommand::Print) => {
            print!("{}", render_skill_sh());
        }
        AioCliCommand::Component(command) => {
            run_shell_component_command(command)?;
        }
        AioCliCommand::Add(args) => {
            let definition = add_external_cli(args)?;
            println!(
                "已添加外部 CLI `{}`: {}",
                definition.name,
                command_line(&definition)
            );
        }
        AioCliCommand::List(args) => {
            print_external_cli_list(args)?;
        }
        AioCliCommand::Remove(args) => {
            let name = args.name.clone();
            remove_external_cli(args)?;
            println!("已移除外部 CLI `{name}`");
        }
        AioCliCommand::Run(args) => {
            run_external_cli(args).await?;
        }
    }
    Ok(())
}

fn run_shell_component_command(command: AioCliComponentCommand) -> anyhow::Result<()> {
    match command {
        AioCliComponentCommand::List(args) => print_shell_component_list(args)?,
        AioCliComponentCommand::Get(args) => print_shell_component_detail(args)?,
        AioCliComponentCommand::Upsert(args) => upsert_shell_component(args)?,
        AioCliComponentCommand::Set(args) => patch_shell_component(args)?,
        AioCliComponentCommand::Remove(args) => remove_shell_component(args)?,
        AioCliComponentCommand::Config(args) => update_shell_component_config(args)?,
        AioCliComponentCommand::Build(args) => build_shell_components(args)?,
    }
    Ok(())
}

fn print_metadata(format: CliOutputFormat) -> anyhow::Result<()> {
    match format {
        CliOutputFormat::Table => print!("{}", render_metadata_table()),
        CliOutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(AIO_COMMANDS)?);
        }
    }
    Ok(())
}

fn print_shell_component_list(args: ShellComponentListArgs) -> anyhow::Result<()> {
    let registry = crate::services::shell_components::load_shell_component_registry_on_server()
        .map_err(anyhow::Error::msg)?;
    match args.format {
        CliOutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(&registry)?);
        }
        CliOutputFormat::Table => {
            println!("CONFIG  {}", registry.config_path);
            println!("OUTPUT  {}", registry.build.resolved_output_path);
            println!();
            println!("NAME               KIND       ENABLED  OUTPUT   SUMMARY");
            for component in registry.components {
                println!(
                    "{:<18} {:<10} {:<8} {:<8} {}",
                    component.name,
                    shell_component_kind_label(component.kind),
                    yes_no(component.enabled),
                    yes_no(component.render_to_output),
                    component.summary
                );
            }
        }
    }
    Ok(())
}

fn print_shell_component_detail(args: ShellComponentGetArgs) -> anyhow::Result<()> {
    let component = crate::services::shell_components::get_shell_component_on_server(&args.name)
        .map_err(anyhow::Error::msg)?;
    let Some(component) = component else {
        bail!("shell component `{}` does not exist", args.name);
    };
    println!("{}", serde_json::to_string_pretty(&component)?);
    Ok(())
}

fn upsert_shell_component(args: ShellComponentUpsertArgs) -> anyhow::Result<()> {
    let saved = crate::services::shell_components::upsert_shell_component_on_server(
        crate::services::ShellComponentUpsert {
            name: args.name,
            kind: shell_component_kind_from_arg(args.kind),
            summary: args.summary,
            enabled: args.enabled,
            render_to_output: args.render_to_output,
            export_value: args.value,
            alias_command: args.command,
            body: args.body,
        },
    )
    .map_err(anyhow::Error::msg)?;
    println!(
        "已保存 shell 组件 `{}` [{}] enabled={} output={}",
        saved.name,
        shell_component_kind_label(saved.kind),
        yes_no(saved.enabled),
        yes_no(saved.render_to_output)
    );
    Ok(())
}

fn patch_shell_component(args: ShellComponentSetArgs) -> anyhow::Result<()> {
    let saved = crate::services::shell_components::patch_shell_component_on_server(
        crate::services::ShellComponentPatch {
            name: args.name,
            summary: args.summary,
            enabled: args.enabled,
            render_to_output: args.render_to_output,
        },
    )
    .map_err(anyhow::Error::msg)?;
    println!(
        "已更新 shell 组件 `{}`: enabled={} output={}",
        saved.name,
        yes_no(saved.enabled),
        yes_no(saved.render_to_output)
    );
    Ok(())
}

fn remove_shell_component(args: ShellComponentRemoveArgs) -> anyhow::Result<()> {
    let removed = crate::services::shell_components::remove_shell_component_on_server(&args.name)
        .map_err(anyhow::Error::msg)?;
    println!("已移除 shell 组件 `{}`", removed.name);
    Ok(())
}

fn update_shell_component_config(args: ShellComponentConfigArgs) -> anyhow::Result<()> {
    let registry = crate::services::shell_components::save_shell_component_config_on_server(
        crate::services::ShellComponentConfigUpdate {
            output_path: args.output,
        },
    )
    .map_err(anyhow::Error::msg)?;
    println!("输出文件已更新为: {}", registry.build.resolved_output_path);
    Ok(())
}

fn build_shell_components(args: ShellComponentBuildArgs) -> anyhow::Result<()> {
    let result = crate::services::shell_components::build_shell_components_on_server(
        crate::services::ShellComponentBuildRequest {
            output_path: args.output,
            write: !args.stdout,
        },
    )
    .map_err(anyhow::Error::msg)?;
    if args.stdout {
        print!("{}", result.content);
    } else {
        println!(
            "已生成 {} (included {}/{})",
            result.output_path, result.included_components, result.total_components
        );
    }
    Ok(())
}

fn shell_component_kind_from_arg(
    kind: ShellComponentKindArg,
) -> crate::services::ShellComponentKind {
    match kind {
        ShellComponentKindArg::Export => crate::services::ShellComponentKind::Export,
        ShellComponentKindArg::Alias => crate::services::ShellComponentKind::Alias,
        ShellComponentKindArg::Function => crate::services::ShellComponentKind::Function,
        ShellComponentKindArg::Snippet => crate::services::ShellComponentKind::Snippet,
    }
}

fn shell_component_kind_label(kind: crate::services::ShellComponentKind) -> &'static str {
    match kind {
        crate::services::ShellComponentKind::Export => "export",
        crate::services::ShellComponentKind::Alias => "alias",
        crate::services::ShellComponentKind::Function => "function",
        crate::services::ShellComponentKind::Snippet => "snippet",
    }
}

fn yes_no(value: bool) -> &'static str {
    if value { "yes" } else { "no" }
}

fn add_external_cli(args: ExternalCliAddArgs) -> anyhow::Result<ExternalCliDefinition> {
    let path = external_cli_config_path()?;
    add_external_cli_at(&path, args)
}

fn add_external_cli_at(
    path: &Path,
    args: ExternalCliAddArgs,
) -> anyhow::Result<ExternalCliDefinition> {
    validate_external_cli_name(&args.name)?;
    let mut config = load_config_at(path)?;
    if config.commands.contains_key(&args.name) && !args.replace {
        bail!("外部 CLI `{}` 已存在；如需覆盖请加 --replace", args.name);
    }

    let mut argv = shlex::split(&args.command)
        .with_context(|| format!("解析 --command 失败: {}", args.command))?;
    if argv.is_empty() {
        bail!("--command 不能为空");
    }
    let command = argv.remove(0);
    let mut base_args = argv;
    base_args.extend(args.args);

    let definition = ExternalCliDefinition {
        name: args.name.clone(),
        command,
        args: base_args,
        description: args.description,
        working_dir: args.working_dir,
        env: parse_env_pairs(args.env)?,
    };
    config.commands.insert(args.name, definition.clone());
    save_config_at(path, &config)?;
    Ok(definition)
}

fn print_external_cli_list(args: ExternalCliListArgs) -> anyhow::Result<()> {
    let config = load_config_at(&external_cli_config_path()?)?;
    match args.format {
        CliOutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(&config.commands)?);
        }
        CliOutputFormat::Table => {
            println!("NAME         COMMAND                         DESCRIPTION");
            for definition in config.commands.values() {
                println!(
                    "{:<12} {:<31} {}",
                    definition.name,
                    command_line(definition),
                    definition.description
                );
            }
        }
    }
    Ok(())
}

fn remove_external_cli(args: ExternalCliRemoveArgs) -> anyhow::Result<()> {
    let path = external_cli_config_path()?;
    let mut config = load_config_at(&path)?;
    if config.commands.remove(&args.name).is_none() {
        bail!("外部 CLI `{}` 不存在", args.name);
    }
    save_config_at(&path, &config)
}

async fn run_external_cli(args: ExternalCliRunArgs) -> anyhow::Result<()> {
    let config = load_config_at(&external_cli_config_path()?)?;
    let definition = config
        .commands
        .get(&args.name)
        .with_context(|| format!("外部 CLI `{}` 不存在，请先运行 aio cli add", args.name))?;
    let program = expand_home_env(&definition.command);
    let mut process = Command::new(program);
    process.args(&definition.args);
    process.args(&args.args);
    if let Some(working_dir) = &definition.working_dir {
        process.current_dir(expand_home_env(working_dir));
    }
    for (key, value) in &definition.env {
        process.env(key, value);
    }
    process
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit());

    let status = process
        .status()
        .await
        .with_context(|| format!("启动外部 CLI `{}` 失败", definition.name))?;
    if !status.success() {
        bail!("外部 CLI `{}` 退出失败: {status}", definition.name);
    }
    Ok(())
}

fn install_skill(root: Option<&str>, path: Option<&str>) -> anyhow::Result<PathBuf> {
    let script_path = match path {
        Some(path) => expand_home_env(path),
        None => {
            let root = match root {
                Some(root) => expand_home_env(root),
                None => default_skills_root()?,
            };
            root.join(SKILL_DIR_NAME).join("skill.sh")
        }
    };
    install_skill_at(&script_path)?;
    Ok(script_path)
}

fn install_skill_at(script_path: &Path) -> anyhow::Result<()> {
    let parent = script_path.parent().context("skill.sh 路径缺少父目录")?;
    fs::create_dir_all(parent)
        .with_context(|| format!("创建 skill 目录失败: {}", parent.display()))?;
    write_file(script_path, render_skill_sh().as_bytes(), 0o755)?;
    write_file(
        &parent.join("SKILL.md"),
        render_skill_md().as_bytes(),
        0o644,
    )?;
    Ok(())
}

fn load_config_at(path: &Path) -> anyhow::Result<ExternalCliConfig> {
    if !path.exists() {
        return Ok(ExternalCliConfig {
            version: current_config_version(),
            commands: BTreeMap::new(),
        });
    }
    let raw = fs::read_to_string(path)
        .with_context(|| format!("读取外部 CLI 配置失败: {}", path.display()))?;
    if raw.trim().is_empty() {
        return Ok(ExternalCliConfig {
            version: current_config_version(),
            commands: BTreeMap::new(),
        });
    }
    serde_json::from_str(&raw).with_context(|| format!("解析外部 CLI 配置失败: {}", path.display()))
}

fn save_config_at(path: &Path, config: &ExternalCliConfig) -> anyhow::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("创建 AIO 配置目录失败: {}", parent.display()))?;
    }
    let encoded = serde_json::to_vec_pretty(config).context("编码外部 CLI 配置失败")?;
    write_file(path, &encoded, 0o600)?;
    append_newline(path)?;
    Ok(())
}

fn write_file(path: &Path, bytes: &[u8], mode: u32) -> anyhow::Result<()> {
    let mut file = fs::OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(path)
        .with_context(|| format!("打开文件失败: {}", path.display()))?;
    file.write_all(bytes)
        .with_context(|| format!("写入文件失败: {}", path.display()))?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(path, fs::Permissions::from_mode(mode))
            .with_context(|| format!("设置文件权限失败: {}", path.display()))?;
    }
    Ok(())
}

fn append_newline(path: &Path) -> anyhow::Result<()> {
    let mut file = fs::OpenOptions::new()
        .append(true)
        .open(path)
        .with_context(|| format!("打开文件失败: {}", path.display()))?;
    file.write_all(b"\n")
        .with_context(|| format!("写入文件失败: {}", path.display()))
}

fn command_line(definition: &ExternalCliDefinition) -> String {
    std::iter::once(definition.command.as_str())
        .chain(definition.args.iter().map(String::as_str))
        .collect::<Vec<_>>()
        .join(" ")
}

fn parse_env_pairs(pairs: Vec<String>) -> anyhow::Result<BTreeMap<String, String>> {
    let mut envs = BTreeMap::new();
    for pair in pairs {
        let Some((key, value)) = pair.split_once('=') else {
            bail!("--env 必须是 KEY=VALUE: {pair}");
        };
        let key = key.trim();
        if key.is_empty() {
            bail!("--env 的 KEY 不能为空: {pair}");
        }
        envs.insert(key.to_string(), value.to_string());
    }
    Ok(envs)
}

fn validate_external_cli_name(name: &str) -> anyhow::Result<()> {
    if name.trim().is_empty() {
        bail!("外部 CLI 名称不能为空");
    }
    if !name
        .chars()
        .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_' | '.'))
    {
        bail!("外部 CLI 名称只能包含字母、数字、`-`、`_` 和 `.`");
    }
    Ok(())
}

fn external_cli_config_path() -> anyhow::Result<PathBuf> {
    if let Some(path) = non_empty_env_path("AIO_EXTERNAL_CLI_CONFIG") {
        return Ok(expand_home_env_path(path));
    }
    aio_config_dir()
        .map(|dir| dir.join(CONFIG_FILE_NAME))
        .context("无法定位 AIO 配置目录: 缺少 XDG_CONFIG_HOME/HOME")
}

fn default_skills_root() -> anyhow::Result<PathBuf> {
    if let Some(path) = non_empty_env_path("AIO_SKILLS_ROOT")
        .or_else(|| non_empty_env_path("ADDZERO_SKILLS_FS_ROOT"))
    {
        return Ok(expand_home_env_path(path));
    }
    env::var_os("HOME")
        .map(PathBuf::from)
        .map(|home| home.join(".agents/skills"))
        .context("无法定位 skill 根目录: 缺少 HOME")
}

fn aio_config_dir() -> Option<PathBuf> {
    env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .or_else(|| env::var_os("HOME").map(|home| PathBuf::from(home).join(".config")))
        .map(|config| config.join("aio"))
}

fn non_empty_env_path(key: &str) -> Option<PathBuf> {
    env::var_os(key)
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
}

fn expand_home_env(raw: &str) -> PathBuf {
    expand_home_env_path(PathBuf::from(raw))
}

fn expand_home_env_path(path: PathBuf) -> PathBuf {
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

fn current_config_version() -> u32 {
    1
}

#[cfg(test)]
mod tests {
    use super::{add_external_cli_at, install_skill_at, load_config_at};
    use crate::cli::ExternalCliAddArgs;

    #[test]
    fn add_external_cli_should_persist_command_metadata() -> anyhow::Result<()> {
        let dir = tempfile::tempdir()?;
        let path = dir.path().join("external-cli.json");

        add_external_cli_at(
            &path,
            ExternalCliAddArgs {
                name: "demo".to_string(),
                command: "echo hello".to_string(),
                args: vec!["world".to_string()],
                description: "demo command".to_string(),
                working_dir: Some("$HOME".to_string()),
                env: vec!["DEMO=1".to_string()],
                replace: false,
            },
        )?;

        let config = load_config_at(&path)?;
        let definition = config.commands.get("demo").expect("demo CLI should exist");
        assert_eq!(definition.command, "echo");
        assert_eq!(definition.args, ["hello", "world"]);
        assert_eq!(definition.env.get("DEMO").map(String::as_str), Some("1"));
        Ok(())
    }

    #[test]
    fn install_skill_should_write_skill_shell_and_manifest() -> anyhow::Result<()> {
        let dir = tempfile::tempdir()?;
        let script_path = dir.path().join("aio-cli/skill.sh");

        install_skill_at(&script_path)?;

        assert!(script_path.exists());
        assert!(dir.path().join("aio-cli/SKILL.md").exists());
        Ok(())
    }

    #[test]
    fn empty_config_file_should_load_as_default_config() -> anyhow::Result<()> {
        let dir = tempfile::tempdir()?;
        let path = dir.path().join("external-cli.json");
        std::fs::write(&path, "")?;

        let config = load_config_at(&path)?;

        assert!(config.commands.is_empty());
        Ok(())
    }
}
