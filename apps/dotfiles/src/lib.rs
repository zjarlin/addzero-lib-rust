pub mod cli;
pub mod config;
pub mod dotfile_links;
pub mod error;
pub mod git_sync;
pub mod init;
pub mod package_manager;
pub mod platform;
pub mod settings;
pub mod status;

use clap::Parser;

use crate::cli::{
    AddDotfilesCommand, Cli, Command, ConfigCommand, InitCommand, PackageCommand,
    RemoveDotfilesCommand,
};
use crate::config::ConfigStore;
use crate::dotfile_links::{LinkApplyMode, add_dotfiles, apply_configured_links, remove_dotfiles};
use crate::error::Result;
use crate::git_sync::GitSync;
use crate::init::{
    enable_all_sources, init_docker, init_git, init_homebrew, init_lazyvim, init_macos, init_node,
    init_ohmyzsh, init_packages,
};
use crate::package_manager::PackageManager;
use crate::platform::{kill_port, show_port};
use crate::settings::Settings;

pub fn run() -> Result<()> {
    let cli = Cli::parse();
    run_with_cli(cli)
}

pub fn run_with_cli(cli: Cli) -> Result<()> {
    let settings = Settings::load(cli.config)?;
    let store = ConfigStore::new(settings.config_file.clone());
    let command = cli.command.unwrap_or(Command::Repl);

    match command {
        Command::Repl => run_repl(settings),
        Command::Config { command } => run_config_command(&settings, &store, command),
        Command::Sync {
            force_links,
            replace_mismatch,
        } => {
            let config = store.load_or_init(&settings)?;
            GitSync::new(&settings, &config).pull(replace_mismatch)?;
            let config = store.load_or_init(&settings)?;
            apply_configured_links(&settings, &config, LinkApplyMode::from_force(force_links))
        }
        Command::Push { message } => {
            let config = store.load_or_init(&settings)?;
            GitSync::new(&settings, &config).commit_and_push(&message)
        }
        Command::AddDotfiles(command) => run_add_dotfiles(&settings, &store, command),
        Command::RemoveDotfiles(command) => run_remove_dotfiles(&settings, &store, command),
        Command::AddPkg(command) => run_add_pkg(&settings, &store, command),
        Command::RmPkg(command) => run_rm_pkg(&settings, &store, command),
        Command::Package { command } => run_package_command(&settings, &store, command),
        Command::Status => {
            let config = store.load_or_init(&settings)?;
            print_current_status(&settings, &config);
            Ok(())
        }
        Command::CatStatus => status::StatusStore::new(settings.status_file()).print(),
        Command::ShowPort { port } => {
            println!("{}", show_port(port)?);
            Ok(())
        }
        Command::KillPort { port } => {
            kill_port(port)?;
            println!("端口 {port} 的占用进程已处理");
            Ok(())
        }
        Command::GitConfig { username, email } => {
            platform::run_checked("git", ["config", "--global", "user.name", &username])?;
            platform::run_checked("git", ["config", "--global", "user.email", &email])?;
            println!("Git 用户名已设置: {username}");
            println!("Git 邮箱已设置: {email}");
            Ok(())
        }
        Command::Init(command) => run_init_command(&settings, &store, command),
        Command::InitDocker => init_docker(),
        Command::InitLazyvim => init_lazyvim(),
        Command::InitHomebrew => init_homebrew(),
        Command::InitOhmyzsh => init_ohmyzsh(),
        Command::EnableAllSources => enable_all_sources(),
    }
}

fn run_config_command(
    settings: &Settings,
    store: &ConfigStore,
    command: ConfigCommand,
) -> Result<()> {
    let mut config = store.load_or_init(settings)?;

    match command {
        ConfigCommand::Cat => {
            println!("{}", serde_json::to_string_pretty(&config)?);
        }
        ConfigCommand::SetCloudUrl { url } => {
            config.cloud_url = Some(url);
            store.save(&config)?;
        }
        ConfigCommand::SetSyncDir { dir } => {
            config.sync_dir = dir;
            store.save(&config)?;
        }
        ConfigCommand::SetPackageManager { name } => {
            config
                .current_platform_config_mut(settings.platform)
                .package_manager = Some(name);
            store.save(&config)?;
        }
    }

    Ok(())
}

fn run_add_dotfiles(
    settings: &Settings,
    store: &ConfigStore,
    command: AddDotfilesCommand,
) -> Result<()> {
    let mut config = store.load_or_init(settings)?;
    let changed = add_dotfiles(settings, &mut config, &command.paths, command.abs)?;
    if changed {
        store.save(&config)?;
        if !command.no_push {
            GitSync::new(settings, &config).commit_and_push("Update dotfiles")?;
        }
    }
    print_current_status(settings, &config);
    Ok(())
}

fn run_remove_dotfiles(
    settings: &Settings,
    store: &ConfigStore,
    command: RemoveDotfilesCommand,
) -> Result<()> {
    let mut config = store.load_or_init(settings)?;
    let changed = remove_dotfiles(settings, &mut config, &command.paths, command.abs)?;
    if changed {
        store.save(&config)?;
        if !command.no_push {
            GitSync::new(settings, &config).commit_and_push("Update dotfiles")?;
        }
    }
    print_current_status(settings, &config);
    Ok(())
}

fn run_add_pkg(settings: &Settings, store: &ConfigStore, command: PackageCommand) -> Result<()> {
    let mut config = store.load_or_init(settings)?;
    let platform_config = config.current_platform_config_mut(settings.platform);
    for package in command.packages {
        let package = package.trim();
        if !package.is_empty() {
            platform_config.default_packages.insert(package.to_string());
            println!("已添加软件包: {package}");
        }
    }
    store.save(&config)?;
    if !command.no_push {
        GitSync::new(settings, &config).commit_and_push("Update packages")?;
    }
    print_current_status(settings, &config);
    Ok(())
}

fn run_rm_pkg(settings: &Settings, store: &ConfigStore, command: PackageCommand) -> Result<()> {
    let mut config = store.load_or_init(settings)?;
    let platform_config = config.current_platform_config_mut(settings.platform);
    for package in command.packages {
        if platform_config.default_packages.remove(package.trim()) {
            println!("已删除软件包: {}", package.trim());
        }
    }
    store.save(&config)?;
    if !command.no_push {
        GitSync::new(settings, &config).commit_and_push("Update packages")?;
    }
    print_current_status(settings, &config);
    Ok(())
}

fn run_package_command(
    settings: &Settings,
    store: &ConfigStore,
    command: cli::PackageSubcommand,
) -> Result<()> {
    let config = store.load_or_init(settings)?;
    let manager = PackageManager::from_config(
        settings.platform,
        config.current_platform_config(settings.platform),
    )?;

    match command {
        cli::PackageSubcommand::Install { packages } => {
            let packages = if packages.is_empty() {
                config
                    .current_platform_config(settings.platform)
                    .default_packages
                    .iter()
                    .cloned()
                    .collect()
            } else {
                packages
            };
            init_packages(settings, &config, &packages, true)
        }
        cli::PackageSubcommand::Search { keyword } => {
            for package in manager.search(&keyword)? {
                println!("{package}");
            }
            Ok(())
        }
        cli::PackageSubcommand::Version { package } => {
            match manager.version(&package)? {
                Some(version) => println!("{version}"),
                None => println!("{package} 未安装或无法读取版本"),
            }
            Ok(())
        }
    }
}

fn run_init_command(settings: &Settings, store: &ConfigStore, command: InitCommand) -> Result<()> {
    settings.ensure_dirs()?;
    let mut config = store.load_or_init(settings)?;
    let status_store = status::StatusStore::new(settings.status_file());
    status_store.ensure_dir()?;

    init_git(settings, &config, command.yes, command.force)?;
    init_node(settings, &config, command.yes, command.force)?;

    if command.install_packages {
        let packages = config
            .current_platform_config(settings.platform)
            .default_packages
            .iter()
            .cloned()
            .collect::<Vec<_>>();
        init_packages(settings, &config, &packages, command.yes)?;
    }

    if command.sync {
        GitSync::new(settings, &config).pull(command.replace_mismatch)?;
        config = store.load_or_init(settings)?;
        apply_configured_links(
            settings,
            &config,
            LinkApplyMode::from_force(command.force_links),
        )?;
    }

    if command.macos_optimize {
        init_macos(settings, command.yes)?;
    }

    Ok(())
}

fn print_current_status(settings: &Settings, config: &config::Config) {
    let platform_config = config.current_platform_config(settings.platform);

    println!("\n当前已配置的软连接:");
    if platform_config.links.is_empty() {
        println!("  暂无软连接配置");
    } else {
        for link in &platform_config.links {
            println!("  {} -> {}", link.source, link.target);
        }
    }

    println!("\n当前已纳入同步的软件包:");
    if platform_config.default_packages.is_empty() {
        println!("  暂无软件包配置");
    } else {
        for package in &platform_config.default_packages {
            println!("  {package}");
        }
    }
}

fn run_repl(settings: Settings) -> Result<()> {
    use std::io::{self, Write};

    println!("dotfiles Rust REPL，输入 help 查看命令，输入 exit 退出。");
    loop {
        print!("dotfiles> ");
        io::stdout().flush()?;

        let mut line = String::new();
        if io::stdin().read_line(&mut line)? == 0 {
            break;
        }

        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        if matches!(line, "exit" | "quit" | "q") {
            break;
        }
        if line == "help" {
            let mut help_cli = Cli {
                config: Some(settings.config_file.clone()),
                command: None,
            };
            help_cli.command = Some(Command::Status);
            println!(
                "常用命令: status, sync, add-dotfiles, remove-dotfiles, add-pkg, rm-pkg, config cat"
            );
            println!("也可以直接输入任意 dotfiles 子命令参数，例如: show-port 3000");
            drop(help_cli);
            continue;
        }

        let Some(words) = shlex::split(line) else {
            eprintln!("无法解析命令行");
            continue;
        };

        let args = std::iter::once("dotfiles".to_string())
            .chain(std::iter::once("--config".to_string()))
            .chain(std::iter::once(
                settings.config_file.to_string_lossy().into_owned(),
            ))
            .chain(words)
            .collect::<Vec<_>>();

        match Cli::try_parse_from(args) {
            Ok(cli) => {
                if matches!(cli.command, Some(Command::Repl)) {
                    eprintln!("当前已经在 REPL 中");
                    continue;
                }
                if let Err(err) = run_with_cli(cli) {
                    eprintln!("error: {err}");
                }
            }
            Err(err) => eprintln!("{err}"),
        }
    }

    Ok(())
}
