use anyhow::{Context, Result};
use axum::{Router, routing::get};
use az_derive_aliases::{apply, clap_args, clap_parser, clap_subcommand};
use az_drive_agent::{
    agent::{DriveAgent, DriveAgentConfig},
    local_state::LocalStateStore,
};
use az_drive_webdav::api::{DriveWebdavState, drive_webdav_router};
use clap::Parser;
use std::io::{self, Write};
use std::net::SocketAddr;
use std::sync::Arc;

#[cfg(target_os = "macos")]
use az_drive_app::macos_actions;

#[apply(clap_parser)]
#[command(name = "az-drive-app")]
#[command(about = "Standalone headless realtime WebDAV drive")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[apply(clap_subcommand)]
enum Command {
    /// Start the center WebDAV service.
    Serve(ServeArgs),
    /// Start an interactive headless drive REPL.
    Repl,
    /// Run the local realtime polling daemon.
    Daemon,
    /// Host a local file or directory.
    Host(HostArgs),
    /// Cancel local hosting without deleting local or remote content.
    Unhost(PathArgs),
    /// Show hosted status.
    Status(StatusArgs),
    /// List tracked files.
    Ls(az_drive_app::cli::DriveLsArgs),
    /// List unresolved conflicts.
    Conflicts,
    /// Manage Git Pool cloud-storage repositories.
    #[command(subcommand)]
    Pool(az_drive_app::cli::DrivePoolCommand),
    /// Manage the Drive storage backend.
    #[command(subcommand)]
    Backend(az_drive_app::cli::DriveBackendCommand),
    /// Inspect and retry durable sync queue items.
    #[command(subcommand)]
    Queue(az_drive_app::cli::DriveQueueCommand),
    /// Inspect and resolve suspended conflicts.
    #[command(subcommand)]
    Conflict(az_drive_app::cli::DriveConflictCommand),
    /// Manage local root aliases.
    Root(RootCommand),
    /// Install macOS Finder Quick Actions for host/unhost.
    InstallMacosActions,
}

#[apply(clap_args)]
struct ServeArgs {
    /// Bind address, for example 0.0.0.0:8788.
    #[arg(long)]
    bind: Option<String>,
}

#[apply(clap_args)]
struct HostArgs {
    /// Local file or directory path.
    path: String,
    /// Preferred root alias.
    #[arg(long)]
    root: Option<String>,
    /// Explicit remote relative path for single-file hosting.
    #[arg(long)]
    remote: Option<String>,
}

#[apply(clap_args)]
struct PathArgs {
    /// Local file or directory path.
    path: String,
}

#[apply(clap_args)]
struct StatusArgs {
    /// Optional local path filter.
    path: Option<String>,
}

#[apply(clap_args)]
struct RootCommand {
    #[command(subcommand)]
    command: RootSubcommand,
}

#[apply(clap_subcommand)]
enum RootSubcommand {
    /// List local logical roots.
    List,
    /// Add or replace a local logical root.
    Add(RootAddArgs),
}

#[apply(clap_args)]
struct RootAddArgs {
    /// Root alias such as home or workspace.
    alias: String,
    /// Local root path.
    #[arg(long)]
    path: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Command::Serve(args) => serve(args).await,
        Command::Repl => run_repl().await,
        Command::Daemon => build_agent()
            .await?
            .run_polling_daemon()
            .await
            .map_err(Into::into),
        Command::Host(args) => {
            let statuses = build_agent()
                .await?
                .host_path(&args.path, args.root.as_deref(), args.remote.as_deref())
                .await?;
            print_json(&statuses)
        }
        Command::Unhost(args) => {
            let removed = build_agent().await?.unhost_path(&args.path).await?;
            print_json(&serde_json::json!({ "unhosted_count": removed }))
        }
        Command::Status(args) => {
            let statuses = build_agent().await?.status(args.path.as_deref()).await?;
            print_json(&statuses)
        }
        Command::Ls(args) => az_drive_app::cli::run_drive_ls(args).await,
        Command::Conflicts => {
            let conflicts = build_agent().await?.conflicts().await?;
            print_json(&conflicts)
        }
        Command::Pool(command) => az_drive_app::cli::run_drive_pool(command).await,
        Command::Backend(command) => az_drive_app::cli::run_drive_backend(command).await,
        Command::Queue(command) => az_drive_app::cli::run_drive_queue(command).await,
        Command::Conflict(command) => az_drive_app::cli::run_drive_conflict(command).await,
        Command::Root(root) => match root.command {
            RootSubcommand::List => {
                let roots = build_agent().await?.list_roots().await?;
                print_json(&roots)
            }
            RootSubcommand::Add(args) => {
                let roots = build_agent()
                    .await?
                    .add_root(&args.alias, &args.path)
                    .await?;
                print_json(&roots)
            }
        },
        Command::InstallMacosActions => install_macos_actions(),
    }
}

async fn serve(args: ServeArgs) -> Result<()> {
    use az_drive_app::ws::CrdtSyncState;

    let (metadata, objects, sync) = az_drive_app::build_stores().await?;
    let webdav_state = DriveWebdavState::new(metadata.clone(), objects.clone());
    let owner_drive_id = az_drive_app::default_owner_drive_id();
    let crdt_state = Arc::new(CrdtSyncState::new(
        metadata.clone(), objects.clone(), owner_drive_id.clone(),
    ));

    // Build agent with on_file_synced → notify WS peers.
    let state_store = LocalStateStore::new(LocalStateStore::default_path());
    let state = state_store.load_or_init().await?;
    let crdt_for_agent = crdt_state.clone();
    let config = DriveAgentConfig::new(
        owner_drive_id.clone(),
        state.device_id,
        state.device_name,
    )
    .with_fused_space_ids(az_drive_app::default_fused_space_ids(&owner_drive_id))
    .with_auto_materialize_space_ids(az_drive_app::default_auto_materialize_space_ids(&owner_drive_id))
    .with_on_file_synced(move |remote_path| {
        let crdt = crdt_for_agent.clone();
        tokio::spawn(async move {
            crdt.notify_text_changed(&remote_path).await;
        });
    });

    let agent = DriveAgent::new_with_sync(metadata, objects, sync, state_store, config);
    if owner_drive_id != "main" {
        agent.migrate_legacy_owner_drive("main", &owner_drive_id).await
            .context("failed to migrate legacy main drive namespace")?;
    }

    let app = Router::new()
        .route("/health", get(|| async { "ok" }))
        .merge(drive_webdav_router(webdav_state))
        .route("/ws/sync", get({
            let crdt_state = crdt_state.clone();
            |ws: axum::extract::WebSocketUpgrade| async move {
                ws.on_upgrade(move |socket| az_drive_app::ws::handle_crdt_sync(socket, crdt_state))
            }
        }));
    let bind = args
        .bind
        .unwrap_or_else(az_drive_app::default_bind_addr)
        .parse::<SocketAddr>()
        .context("invalid bind address")?;
    let listener = tokio::net::TcpListener::bind(bind)
        .await
        .with_context(|| format!("failed to bind {bind}"))?;
    println!("az-drive-app serving WebDAV at http://{bind}/dav/main/home");
    println!("CRDT WebSocket at ws://{bind}/ws/sync");
    println!("file-polling daemon active (interval {:?})", agent_config_poll_interval(&agent));

    // Spawn the file-polling daemon alongside the HTTP server.
    let daemon = tokio::spawn(async move {
        if let Err(err) = agent.run_polling_daemon().await {
            log::error!("drive daemon exited: {err:#}");
        }
    });

    axum::serve(listener, app).await?;
    daemon.abort();
    Ok(())
}

fn agent_config_poll_interval(_agent: &DriveAgent) -> std::time::Duration {
    // The agent doesn't expose config publicly, so use a hardcoded fallback.
    // In practice this is 2 seconds from DriveAgentConfig::new().
    std::time::Duration::from_secs(2)
}

async fn build_agent() -> Result<DriveAgent> {
    let (metadata, objects, sync) = az_drive_app::build_stores().await?;
    let state_store = LocalStateStore::new(LocalStateStore::default_path());
    let state = state_store.load_or_init().await?;
    let primary_owner_drive_id = az_drive_app::default_owner_drive_id();
    let config = DriveAgentConfig::new(
        primary_owner_drive_id.clone(),
        state.device_id,
        state.device_name,
    )
    .with_fused_space_ids(az_drive_app::default_fused_space_ids(
        &primary_owner_drive_id,
    ))
    .with_auto_materialize_space_ids(az_drive_app::default_auto_materialize_space_ids(
        &primary_owner_drive_id,
    ));
    let agent = DriveAgent::new_with_sync(metadata, objects, sync, state_store, config);
    if primary_owner_drive_id != "main" {
        agent
            .migrate_legacy_owner_drive("main", &primary_owner_drive_id)
            .await
            .context("failed to migrate legacy main drive namespace")?;
    }
    Ok(agent)
}

fn print_json(value: &impl serde::Serialize) -> Result<()> {
    println!("{}", serde_json::to_string_pretty(value)?);
    Ok(())
}

async fn run_repl() -> Result<()> {
    println!("AIO Drive REPL。输入 help 查看命令，exit 退出。");
    let mut line = String::new();
    loop {
        print!("drive> ");
        io::stdout()
            .flush()
            .context("failed to flush repl prompt")?;
        line.clear();
        if io::stdin()
            .read_line(&mut line)
            .context("failed to read repl input")?
            == 0
        {
            break;
        }
        let Some(parts) = shlex::split(line.trim()) else {
            println!("错误: 命令解析失败，请检查引号是否成对");
            continue;
        };
        if parts.is_empty() {
            continue;
        }
        match run_repl_command(&parts).await {
            Ok(ReplAction::Continue) => {}
            Ok(ReplAction::Exit) => break,
            Err(error) => println!("错误: {error:#}"),
        }
    }
    Ok(())
}

enum ReplAction {
    Continue,
    Exit,
}

async fn run_repl_command(parts: &[String]) -> Result<ReplAction> {
    match parts[0].as_str() {
        "exit" | "quit" | "q" => Ok(ReplAction::Exit),
        "help" | "h" => {
            print_repl_help();
            Ok(ReplAction::Continue)
        }
        "root" => {
            run_repl_root_command(parts).await?;
            Ok(ReplAction::Continue)
        }
        "host" => {
            let path = required_arg(parts, 1, "host <path>")?;
            let statuses = build_agent().await?.host_path(path, None, None).await?;
            print_json(&statuses)?;
            Ok(ReplAction::Continue)
        }
        "unhost" => {
            let path = required_arg(parts, 1, "unhost <path>")?;
            let removed = build_agent().await?.unhost_path(path).await?;
            print_json(&serde_json::json!({ "unhosted_count": removed }))?;
            Ok(ReplAction::Continue)
        }
        "status" => {
            let statuses = build_agent()
                .await?
                .status(parts.get(1).map(String::as_str))
                .await?;
            print_json(&statuses)?;
            Ok(ReplAction::Continue)
        }
        "conflicts" => {
            let conflicts = build_agent().await?.conflicts().await?;
            print_json(&conflicts)?;
            Ok(ReplAction::Continue)
        }
        "install-macos-actions" => {
            install_macos_actions()?;
            Ok(ReplAction::Continue)
        }
        command => {
            anyhow::bail!("未知命令 `{command}`，输入 help 查看可用命令");
        }
    }
}

async fn run_repl_root_command(parts: &[String]) -> Result<()> {
    match parts.get(1).map(String::as_str) {
        Some("list") => {
            let roots = build_agent().await?.list_roots().await?;
            print_json(&roots)
        }
        Some("add") => {
            let alias = required_arg(parts, 2, "root add <alias> <path>")?;
            let path = required_arg(parts, 3, "root add <alias> <path>")?;
            let roots = build_agent().await?.add_root(alias, path).await?;
            print_json(&roots)
        }
        _ => anyhow::bail!("用法: root list | root add <alias> <path>"),
    }
}

fn required_arg<'a>(parts: &'a [String], index: usize, usage: &str) -> Result<&'a str> {
    parts
        .get(index)
        .map(String::as_str)
        .ok_or_else(|| anyhow::anyhow!("缺少参数，用法: {usage}"))
}

fn print_repl_help() {
    println!(
        r#"可用命令:
  root list                      查看本机 root alias
  root add <alias> <path>        添加 root alias，例如 root add workspace ~/workspace
  host <path>                    托管文件或目录
  unhost <path>                  解除本机托管，不删除本地/远端文件
  status [path]                  查看同步状态
  conflicts                      查看未解决冲突
  install-macos-actions          安装 Finder 右键快速操作
  help                           查看帮助
  exit                           退出"#
    );
}

#[cfg(target_os = "macos")]
fn install_macos_actions() -> Result<()> {
    let result = macos_actions::install()?;
    print_json(&result)
}

#[cfg(not(target_os = "macos"))]
fn install_macos_actions() -> Result<()> {
    anyhow::bail!("macOS Finder Quick Actions can only be installed on macOS")
}
