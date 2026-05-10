use anyhow::Result;
use az_drive_agent::{
    HostedStatus, ListTrackedOptions, LocalRootState, PullRemoteItem, PullRemoteOptions,
    PullRemoteStatus, TrackedItem, TrackedItemSource, TrackedItemStatus,
};
use clap::{Args, Subcommand, ValueEnum};

/// Drive commands embedded by both `az-drive-app` and `aio drive`.
#[derive(Debug, Subcommand)]
pub enum DriveCommand {
    /// Host a local file or directory.
    Host(DriveHostArgs),
    /// Cancel local hosting without deleting local or remote content.
    Unhost(DrivePathArgs),
    /// Show hosted status.
    Status(DriveStatusArgs),
    /// List tracked drive files.
    Ls(DriveLsArgs),
    /// Pull remote files into this computer's logical roots.
    Pull(DrivePullArgs),
    /// Run one sync scan for hosted files.
    Sync,
    /// Run the polling sync daemon in the foreground.
    Daemon,
    /// Manage local root aliases.
    #[command(subcommand)]
    Root(DriveRootCommand),
}

/// Arguments for `drive host`.
#[derive(Debug, Args)]
pub struct DriveHostArgs {
    /// Local file or directory path.
    pub path: String,
    /// Preferred root alias.
    #[arg(long)]
    pub root: Option<String>,
    /// Explicit remote relative path for single-file hosting.
    #[arg(long)]
    pub remote: Option<String>,
}

/// Arguments for a command that takes one local path.
#[derive(Debug, Args)]
pub struct DrivePathArgs {
    /// Local path.
    pub path: String,
}

/// Arguments for `drive status`.
#[derive(Debug, Args)]
pub struct DriveStatusArgs {
    /// Optional local path filter.
    pub path: Option<String>,
}

/// Arguments for `drive ls`.
#[derive(Debug, Args)]
pub struct DriveLsArgs {
    /// Optional local path filter.
    pub path: Option<String>,
    /// List server-side metadata entries.
    #[arg(long)]
    pub remote: bool,
    /// Merge local tracking state and server-side metadata entries.
    #[arg(long)]
    pub all: bool,
    /// List hosted directory roots.
    #[arg(long)]
    pub roots: bool,
    /// Include database and git ignore exclusions.
    #[arg(long)]
    pub ignored: bool,
    /// Output format.
    #[arg(long, value_enum, default_value_t = DriveListFormat::Table)]
    pub format: DriveListFormat,
}

/// Arguments for `drive pull`.
#[derive(Debug, Args)]
pub struct DrivePullArgs {
    /// Optional local path filter, for example ~/.agents/skills.
    pub path: Option<String>,
    /// Overwrite existing local files that differ from remote.
    #[arg(long)]
    pub overwrite: bool,
    /// Show what would be pulled without writing files or local state.
    #[arg(long)]
    pub dry_run: bool,
    /// Output format.
    #[arg(long, value_enum, default_value_t = DriveListFormat::Table)]
    pub format: DriveListFormat,
}

/// Root alias subcommands.
#[derive(Debug, Subcommand)]
pub enum DriveRootCommand {
    /// List local logical roots.
    List,
    /// Add or replace a local logical root.
    Add(DriveRootAddArgs),
}

/// Arguments for `drive root add`.
#[derive(Debug, Args)]
pub struct DriveRootAddArgs {
    /// Root alias such as home, workspace, or library.
    pub alias: String,
    /// Device-local root path.
    #[arg(long)]
    pub path: String,
}

/// Output format for `drive ls`.
#[derive(Clone, Copy, Debug, Eq, PartialEq, ValueEnum)]
pub enum DriveListFormat {
    /// Human-readable table.
    Table,
    /// Pretty JSON.
    Json,
}

/// Runs an embedded drive command.
///
/// # Errors
/// Returns an error when command execution or output serialization fails.
pub async fn run_drive_command(command: DriveCommand) -> Result<()> {
    match command {
        DriveCommand::Host(args) => run_drive_host(args).await,
        DriveCommand::Unhost(args) => run_drive_unhost(args).await,
        DriveCommand::Status(args) => run_drive_status(args).await,
        DriveCommand::Ls(args) => run_drive_ls(args).await,
        DriveCommand::Pull(args) => run_drive_pull(args).await,
        DriveCommand::Sync => run_drive_sync().await,
        DriveCommand::Daemon => crate::build_agent()
            .await?
            .run_polling_daemon()
            .await
            .map_err(Into::into),
        DriveCommand::Root(command) => run_drive_root(command).await,
    }
}

/// Runs `drive host`.
///
/// # Errors
/// Returns an error when hosting fails.
pub async fn run_drive_host(args: DriveHostArgs) -> Result<()> {
    let statuses = crate::build_agent()
        .await?
        .host_path(&args.path, args.root.as_deref(), args.remote.as_deref())
        .await?;
    println!("{}", serde_json::to_string_pretty(&statuses)?);
    Ok(())
}

/// Runs `drive unhost`.
///
/// # Errors
/// Returns an error when unhosting fails.
pub async fn run_drive_unhost(args: DrivePathArgs) -> Result<()> {
    let removed = crate::build_agent().await?.unhost_path(&args.path).await?;
    println!(
        "{}",
        serde_json::to_string_pretty(&serde_json::json!({ "unhosted_count": removed }))?
    );
    Ok(())
}

/// Runs `drive status`.
///
/// # Errors
/// Returns an error when status loading fails.
pub async fn run_drive_status(args: DriveStatusArgs) -> Result<()> {
    let statuses = crate::build_agent()
        .await?
        .status(args.path.as_deref())
        .await?;
    print_status_table(&statuses)
}

/// Runs `drive ls`.
///
/// # Errors
/// Returns an error when listing tracked paths or serializing JSON fails.
pub async fn run_drive_ls(args: DriveLsArgs) -> Result<()> {
    let items = crate::build_agent()
        .await?
        .list_tracked(
            args.path.as_deref(),
            ListTrackedOptions {
                include_remote: args.remote,
                include_all: args.all,
                roots_only: args.roots,
                include_ignored: args.ignored,
            },
        )
        .await?;
    match args.format {
        DriveListFormat::Table => print_tracked_table(&items),
        DriveListFormat::Json => {
            println!("{}", serde_json::to_string_pretty(&items)?);
            Ok(())
        }
    }
}

/// Runs `drive pull`.
///
/// # Errors
/// Returns an error when remote entries cannot be materialized.
pub async fn run_drive_pull(args: DrivePullArgs) -> Result<()> {
    let items = crate::build_agent()
        .await?
        .pull_remote(
            args.path.as_deref(),
            PullRemoteOptions {
                overwrite: args.overwrite,
                dry_run: args.dry_run,
            },
        )
        .await?;
    match args.format {
        DriveListFormat::Table => print_pull_table(&items),
        DriveListFormat::Json => {
            println!("{}", serde_json::to_string_pretty(&items)?);
            Ok(())
        }
    }
}

/// Runs one `drive sync` scan.
///
/// # Errors
/// Returns an error when synchronization fails.
pub async fn run_drive_sync() -> Result<()> {
    let statuses = crate::build_agent().await?.sync_once().await?;
    print_status_table(&statuses)
}

/// Runs `drive root`.
///
/// # Errors
/// Returns an error when root state cannot be read or written.
pub async fn run_drive_root(command: DriveRootCommand) -> Result<()> {
    let agent = crate::build_agent().await?;
    let roots = match command {
        DriveRootCommand::List => agent.list_roots().await?,
        DriveRootCommand::Add(args) => agent.add_root(&args.alias, &args.path).await?,
    };
    print_roots_table(&roots)
}

fn print_tracked_table(items: &[TrackedItem]) -> Result<()> {
    println!("{:<13} {:<10} {:<48} LOCAL", "STATUS", "SOURCE", "PATH");
    for item in items {
        println!(
            "{:<13} {:<10} {:<48} {}",
            status_text(item.status),
            source_text(item.source),
            item.display_path,
            item.local_path
                .as_ref()
                .map(|path| path.display().to_string())
                .unwrap_or_else(|| "-".to_owned())
        );
    }
    Ok(())
}

fn print_pull_table(items: &[PullRemoteItem]) -> Result<()> {
    println!("{:<17} {:<48} LOCAL", "STATUS", "PATH");
    for item in items {
        println!(
            "{:<17} {:<48} {}",
            pull_status_text(item.status),
            item.display_path,
            item.local_path.display()
        );
    }
    Ok(())
}

fn print_status_table(items: &[HostedStatus]) -> Result<()> {
    println!("{:<48} {:<34} EXISTS", "LOCAL", "REMOTE");
    for item in items {
        println!(
            "{:<48} {:<34} {}",
            item.local_path.display(),
            item.remote_path,
            item.exists
        );
    }
    Ok(())
}

fn print_roots_table(items: &[LocalRootState]) -> Result<()> {
    println!("{:<16} PATH", "ALIAS");
    for item in items {
        println!("{:<16} {}", item.alias, item.path.display());
    }
    Ok(())
}

fn status_text(status: TrackedItemStatus) -> &'static str {
    match status {
        TrackedItemStatus::Tracked => "tracked",
        TrackedItemStatus::MissingLocal => "missing_local",
        TrackedItemStatus::RemoteOnly => "remote_only",
        TrackedItemStatus::Ignored => "ignored",
        TrackedItemStatus::Root => "root",
    }
}

fn source_text(source: TrackedItemSource) -> &'static str {
    match source {
        TrackedItemSource::Local => "local",
        TrackedItemSource::Remote => "remote",
        TrackedItemSource::Both => "both",
        TrackedItemSource::DbIgnore => "db_ignore",
        TrackedItemSource::Gitignore => "gitignore",
    }
}

fn pull_status_text(status: PullRemoteStatus) -> &'static str {
    match status {
        PullRemoteStatus::Pulled => "pulled",
        PullRemoteStatus::AlreadyCurrent => "already_current",
        PullRemoteStatus::SkippedExisting => "skipped_existing",
        PullRemoteStatus::SkippedIgnored => "skipped_ignored",
        PullRemoteStatus::SkippedNoVersion => "skipped_no_version",
        PullRemoteStatus::DryRun => "dry_run",
    }
}
