use anyhow::{Context, Result};
use axum::{Router, routing::get};
use az_drive_agent::{DriveAgent, DriveAgentConfig, LocalStateStore};
use az_drive_store::{
    DriveMetadataStore, DriveObjectStore, InMemoryDriveMetadataStore, InMemoryDriveObjectStore,
    PgDriveMetadataStore, S3DriveObjectStore,
};
use az_drive_webdav::{DriveWebdavState, drive_webdav_router};
use clap::{Args, Parser, Subcommand};
use std::io::{self, Write};
use std::net::SocketAddr;
use std::sync::Arc;

#[cfg(target_os = "macos")]
mod macos_actions;

#[derive(Debug, Parser)]
#[command(name = "az-drive-app")]
#[command(about = "Standalone headless realtime WebDAV drive")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Start the center WebDAV service.
    Serve(ServeArgs),
    /// Detect local Docker PostgreSQL/MinIO containers.
    Detect,
    /// Interactive setup for PostgreSQL/MinIO config.
    Setup,
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
    /// List unresolved conflicts.
    Conflicts,
    /// Manage local root aliases.
    Root(RootCommand),
    /// Run PostgreSQL migrations and exit.
    Migrate,
    /// Install macOS Finder Quick Actions for host/unhost.
    InstallMacosActions,
}

#[derive(Debug, Args)]
struct ServeArgs {
    /// Bind address, for example 0.0.0.0:8788.
    #[arg(long)]
    bind: Option<String>,
}

#[derive(Debug, Args)]
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

#[derive(Debug, Args)]
struct PathArgs {
    /// Local file or directory path.
    path: String,
}

#[derive(Debug, Args)]
struct StatusArgs {
    /// Optional local path filter.
    path: Option<String>,
}

#[derive(Debug, Args)]
struct RootCommand {
    #[command(subcommand)]
    command: RootSubcommand,
}

#[derive(Debug, Subcommand)]
enum RootSubcommand {
    /// List local logical roots.
    List,
    /// Add or replace a local logical root.
    Add(RootAddArgs),
}

#[derive(Debug, Args)]
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
        Command::Detect => {
            let detection = az_drive_app::setup::detect_docker();
            println!("{}", az_drive_app::setup::format_detection(&detection));
            print_json(&detection)
        }
        Command::Setup => {
            let result = az_drive_app::setup::interactive_setup().await?;
            print_json(&result)
        }
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
        Command::Conflicts => {
            let conflicts = build_agent().await?.conflicts().await?;
            print_json(&conflicts)
        }
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
        Command::Migrate => {
            let Some(database_url) = database_url() else {
                anyhow::bail!("AZ_DRIVE_DATABASE_URL or DATABASE_URL is required for migrate");
            };
            let store = PgDriveMetadataStore::connect(&database_url).await?;
            store.run_migrations().await?;
            print_json(&serde_json::json!({ "migrated": true }))
        }
        Command::InstallMacosActions => install_macos_actions(),
    }
}

async fn serve(args: ServeArgs) -> Result<()> {
    let (metadata, objects) = build_stores().await?;
    let state = DriveWebdavState::new(metadata, objects);
    let app = Router::new()
        .route("/health", get(|| async { "ok" }))
        .merge(drive_webdav_router(state));
    let bind = args
        .bind
        .unwrap_or_else(az_drive_app::default_bind_addr)
        .parse::<SocketAddr>()
        .context("invalid bind address")?;
    let listener = tokio::net::TcpListener::bind(bind)
        .await
        .with_context(|| format!("failed to bind {bind}"))?;
    println!("az-drive-app serving WebDAV at http://{bind}/dav/main/home");
    axum::serve(listener, app).await?;
    Ok(())
}

async fn build_agent() -> Result<DriveAgent> {
    let (metadata, objects) = build_stores().await?;
    let state_store = LocalStateStore::new(LocalStateStore::default_path());
    let state = state_store.load_or_init().await?;
    let config = DriveAgentConfig::new(
        az_drive_app::default_space_id(),
        state.device_id,
        state.device_name,
    );
    Ok(DriveAgent::new(metadata, objects, state_store, config))
}

async fn build_stores() -> Result<(Arc<dyn DriveMetadataStore>, Arc<dyn DriveObjectStore>)> {
    let metadata: Arc<dyn DriveMetadataStore> = if let Some(database_url) = database_url() {
        let store = PgDriveMetadataStore::connect(&database_url)
            .await
            .context("failed to connect drive postgres metadata store")?;
        store
            .run_migrations()
            .await
            .context("failed to run drive postgres migrations")?;
        Arc::new(store)
    } else {
        eprintln!(
            "AZ_DRIVE_DATABASE_URL/MSC_AIO_DATABASE_URL/DATABASE_URL not set; using non-persistent metadata store"
        );
        Arc::new(InMemoryDriveMetadataStore::new())
    };

    let objects: Arc<dyn DriveObjectStore> = if let Some(config) = s3_config() {
        let bucket = az_drive_app::default_bucket();
        let store = tokio::task::spawn_blocking(move || {
            let client = az_rustfs::create_storage_client(config);
            S3DriveObjectStore::new(client, bucket)
        })
        .await
        .context("S3 object store initialization task failed")?
        .context("failed to initialize S3 object store")?;
        Arc::new(store)
    } else {
        eprintln!("AZ_DRIVE_MINIO_*/AIO_MINIO_* not set; using non-persistent object store");
        Arc::new(InMemoryDriveObjectStore::new())
    };

    Ok((metadata, objects))
}

fn database_url() -> Option<String> {
    az_drive_app::setup::current_database_url().filter(|value| !value.trim().is_empty())
}

fn s3_config() -> Option<az_rustfs::S3ClientConfig> {
    let endpoint = az_drive_app::setup::current_minio_endpoint()?;
    let access_key = az_drive_app::setup::current_minio_access_key()?;
    let secret_key = az_drive_app::setup::current_minio_secret_key()?;
    let region = az_drive_app::setup::current_minio_region();
    Some(
        az_rustfs::S3ClientConfig::new(endpoint, access_key, secret_key)
            .with_region(region)
            .with_path_style_access(true),
    )
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
        "detect" => {
            let detection = az_drive_app::setup::detect_docker();
            println!("{}", az_drive_app::setup::format_detection(&detection));
            Ok(ReplAction::Continue)
        }
        "setup" => {
            let result = az_drive_app::setup::interactive_setup().await?;
            print_json(&result)?;
            Ok(ReplAction::Continue)
        }
        "show" => {
            print_json(&az_drive_app::setup::current_config_view())?;
            Ok(ReplAction::Continue)
        }
        "test" => {
            let result = az_drive_app::setup::test_current_config().await;
            print_json(&result)?;
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
  detect                         检测本机 Docker PostgreSQL/MinIO
  setup                          交互式写入 ~/.config/aio/aio.env
  show                           查看当前配置，隐藏 secret
  test                           测试 PostgreSQL migration 和 MinIO bucket
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
