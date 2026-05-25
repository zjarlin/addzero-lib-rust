use anyhow::Result;
use az_derive_aliases::{apply, clap_args, clap_subcommand, clap_value_enum};
use az_drive_agent::{
    ConflictResolution, HostedStatus, ListTrackedOptions, LocalRootState, PullRemoteItem,
    PullRemoteOptions, PullRemoteStatus, TrackedItem, TrackedItemSource, TrackedItemStatus,
};
use az_drive_store::{DriveConflict, DriveSyncQueueItem, DriveSyncTaskStatus};
use std::path::PathBuf;
use uuid::Uuid;

/// Drive commands embedded by both `az-drive-app` and `aio drive`.
#[apply(clap_subcommand)]
pub enum DriveCommand {
    /// Host a local file or directory.
    Host(DriveHostArgs),
    /// Cancel local hosting without deleting local or remote content.
    Unhost(DrivePathArgs),
    /// Show hosted status.
    Status(DriveStatusArgs),
    /// List tracked drive files.
    Ls(DriveLsArgs),
    /// Compatibility command: materialize visible remote files once.
    Pull(DrivePullArgs),
    /// Run one bidirectional sync scan, including visible fused remote files.
    Sync,
    /// Run the polling sync daemon in the foreground.
    Daemon,
    /// Manage Git Pool cloud-storage repositories.
    #[command(subcommand)]
    Pool(DrivePoolCommand),
    /// Manage the Drive storage backend.
    #[command(subcommand)]
    Backend(DriveBackendCommand),
    /// Inspect and retry durable sync queue items.
    #[command(subcommand)]
    Queue(DriveQueueCommand),
    /// Inspect and resolve suspended conflicts.
    #[command(subcommand)]
    Conflict(DriveConflictCommand),
    /// Manage local root aliases.
    #[command(subcommand)]
    Root(DriveRootCommand),
}

/// Arguments for `drive host`.
#[apply(clap_args)]
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
#[apply(clap_args)]
pub struct DrivePathArgs {
    /// Local path.
    pub path: String,
}

/// Arguments for `drive status`.
#[apply(clap_args)]
pub struct DriveStatusArgs {
    /// Optional local path filter.
    pub path: Option<String>,
}

/// Arguments for `drive ls`.
#[apply(clap_args)]
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

/// Arguments for the compatibility remote materialization command.
#[apply(clap_args)]
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
#[apply(clap_subcommand)]
pub enum DriveRootCommand {
    /// List local logical roots.
    List,
    /// Add or replace a local logical root.
    Add(DriveRootAddArgs),
}

/// Git Pool subcommands.
#[apply(clap_subcommand)]
pub enum DrivePoolCommand {
    /// Initialize the local Git Pool backend.
    Init(DrivePoolInitArgs),
    /// Add a writable content pool repository.
    Add(DrivePoolAddArgs),
    /// Mount another owner's pool as a fused source.
    Mount(DrivePoolMountArgs),
    /// Unmount a fused pool.
    Unmount(DrivePoolUnmountArgs),
    /// List configured pools and mounts.
    List(DrivePoolListArgs),
}

/// Drive backend subcommands.
#[apply(clap_subcommand)]
pub enum DriveBackendCommand {
    /// Show current backend status.
    Status,
    /// Select the default backend.
    Use(DriveBackendUseArgs),
    /// Import currently hosted local files into Git Pool.
    MigrateToGitPool,
}

/// Sync queue subcommands.
#[apply(clap_subcommand)]
pub enum DriveQueueCommand {
    /// List durable sync queue items.
    List(DriveQueueListArgs),
    /// Move failed items back to pending and run one sync pass.
    Retry,
}

/// Conflict subcommands.
#[apply(clap_subcommand)]
pub enum DriveConflictCommand {
    /// List unresolved conflicts.
    List(DriveConflictListArgs),
    /// Resolve a conflict and clear its suspension.
    Resolve(DriveConflictResolveArgs),
}

#[apply(clap_args)]
pub struct DriveQueueListArgs {
    /// Optional status filter.
    #[arg(long, value_enum)]
    pub status: Option<DriveQueueStatusArg>,
    /// Output format.
    #[arg(long, value_enum, default_value_t = DriveListFormat::Table)]
    pub format: DriveListFormat,
}

#[apply(clap_value_enum)]
pub enum DriveQueueStatusArg {
    /// Pending queue items.
    Pending,
    /// Running queue items.
    Running,
    /// Completed queue items.
    Done,
    /// Failed queue items.
    Failed,
}

#[apply(clap_args)]
pub struct DriveConflictListArgs {
    /// Output format.
    #[arg(long, value_enum, default_value_t = DriveListFormat::Table)]
    pub format: DriveListFormat,
}

#[apply(clap_args)]
pub struct DriveConflictResolveArgs {
    /// Conflict id.
    pub id: Uuid,
    /// Keep the remote version.
    #[arg(long, conflicts_with_all = ["keep_local", "merged"])]
    pub keep_remote: bool,
    /// Restore the conflict copy and upload it on the next sync.
    #[arg(long, conflicts_with_all = ["keep_remote", "merged"])]
    pub keep_local: bool,
    /// Use a manually merged file and upload it on the next sync.
    #[arg(long, conflicts_with_all = ["keep_remote", "keep_local"])]
    pub merged: Option<PathBuf>,
}

#[apply(clap_args)]
pub struct DrivePoolInitArgs {
    /// Optional control repository remote URL.
    #[arg(long)]
    pub control_remote: Option<String>,
    /// Optional local directory that will auto-create bare pool repos on demand.
    #[arg(long)]
    pub auto_pool_root: Option<PathBuf>,
    /// Prefix used for auto-created pool names, for example `auto`.
    #[arg(long)]
    pub auto_pool_prefix: Option<String>,
    /// Object storage backend: `git-pool` or `gitdb`.
    #[arg(long)]
    pub object_backend: Option<String>,
    /// Root directory for GitDB-sharded object storage.
    #[arg(long)]
    pub gitdb_object_root: Option<PathBuf>,
    /// Prefix used for GitDB shard names, for example `shard`.
    #[arg(long)]
    pub gitdb_object_shard_prefix: Option<String>,
    /// Soft shard size limit for GitDB objects, for example 8gb or 512mb.
    #[arg(long)]
    pub gitdb_object_max_shard_size: Option<String>,
}

#[apply(clap_args)]
pub struct DrivePoolAddArgs {
    /// Pool name.
    pub name: String,
    /// Pool Git remote URL or local bare repo path.
    pub url: String,
    /// Pool soft size limit, for example 8gb or 512mb.
    #[arg(long)]
    pub max_size: Option<String>,
}

#[apply(clap_args)]
pub struct DrivePoolMountArgs {
    /// Local mount name.
    pub name: String,
    /// Pool Git remote URL or local bare repo path.
    pub url: String,
    /// Owner Drive id, for example user-zhangsan.
    #[arg(long)]
    pub owner: String,
    /// Mount as readonly. This is the default v1 behavior.
    #[arg(long, default_value_t = true)]
    pub readonly: bool,
}

#[apply(clap_args)]
pub struct DrivePoolUnmountArgs {
    /// Local mount name.
    pub name: String,
}

#[apply(clap_args)]
pub struct DrivePoolListArgs {
    /// Output format.
    #[arg(long, value_enum, default_value_t = DriveListFormat::Table)]
    pub format: DriveListFormat,
}

#[apply(clap_args)]
pub struct DriveBackendUseArgs {
    /// Backend name. Only `git-pool` is supported.
    pub backend: String,
}

/// Arguments for `drive root add`.
#[apply(clap_args)]
pub struct DriveRootAddArgs {
    /// Root alias such as home, workspace, or library.
    pub alias: String,
    /// Device-local root path.
    #[arg(long)]
    pub path: String,
}

/// Output format for `drive ls`.
#[apply(clap_value_enum)]
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
        DriveCommand::Pool(command) => run_drive_pool(command).await,
        DriveCommand::Backend(command) => run_drive_backend(command).await,
        DriveCommand::Queue(command) => run_drive_queue(command).await,
        DriveCommand::Conflict(command) => run_drive_conflict(command).await,
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

/// Runs the compatibility remote materialization command.
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

/// Runs `drive pool`.
///
/// # Errors
/// Returns an error when Git Pool setup or output serialization fails.
pub async fn run_drive_pool(command: DrivePoolCommand) -> Result<()> {
    match command {
        DrivePoolCommand::Init(args) => {
            let status = crate::init_git_pool_backend(
                args.control_remote.as_deref(),
                args.auto_pool_root,
                args.auto_pool_prefix.as_deref(),
                args.object_backend.as_deref(),
                args.gitdb_object_root,
                args.gitdb_object_shard_prefix.as_deref(),
                args.gitdb_object_max_shard_size
                    .as_deref()
                    .map(parse_size_bytes)
                    .transpose()?,
            )?;
            println!("{}", serde_json::to_string_pretty(&status)?);
            Ok(())
        }
        DrivePoolCommand::Add(args) => {
            let pool = crate::add_git_pool(
                &args.name,
                &args.url,
                args.max_size.as_deref().map(parse_size_bytes).transpose()?,
            )?;
            println!("{}", serde_json::to_string_pretty(&pool)?);
            Ok(())
        }
        DrivePoolCommand::Mount(args) => {
            let mount = crate::mount_git_pool(&args.name, &args.url, &args.owner, args.readonly)?;
            println!("{}", serde_json::to_string_pretty(&mount)?);
            Ok(())
        }
        DrivePoolCommand::Unmount(args) => {
            crate::unmount_git_pool(&args.name)?;
            println!(
                "{}",
                serde_json::to_string_pretty(&serde_json::json!({
                    "unmounted": true,
                    "name": args.name,
                }))?
            );
            Ok(())
        }
        DrivePoolCommand::List(args) => {
            let status = crate::git_pool_backend_status()?;
            match args.format {
                DriveListFormat::Json => {
                    println!("{}", serde_json::to_string_pretty(&status)?);
                    Ok(())
                }
                DriveListFormat::Table => print_pool_table(&status),
            }
        }
    }
}

/// Runs `drive backend`.
///
/// # Errors
/// Returns an error when backend configuration or migration fails.
pub async fn run_drive_backend(command: DriveBackendCommand) -> Result<()> {
    match command {
        DriveBackendCommand::Status => {
            println!(
                "{}",
                serde_json::to_string_pretty(&crate::drive_backend_status()?)?
            );
            Ok(())
        }
        DriveBackendCommand::Use(args) => {
            crate::use_drive_backend(&args.backend)?;
            println!(
                "{}",
                serde_json::to_string_pretty(&serde_json::json!({
                    "backend": args.backend,
                    "selected": true,
                }))?
            );
            Ok(())
        }
        DriveBackendCommand::MigrateToGitPool => {
            let statuses = crate::migrate_local_state_to_git_pool().await?;
            println!("{}", serde_json::to_string_pretty(&statuses)?);
            Ok(())
        }
    }
}

/// Runs `drive queue`.
///
/// # Errors
/// Returns an error when queue metadata cannot be loaded or retried.
pub async fn run_drive_queue(command: DriveQueueCommand) -> Result<()> {
    match command {
        DriveQueueCommand::List(args) => {
            let items = crate::build_agent()
                .await?
                .sync_queue(args.status.map(DriveSyncTaskStatus::from))
                .await?;
            match args.format {
                DriveListFormat::Json => {
                    println!("{}", serde_json::to_string_pretty(&items)?);
                    Ok(())
                }
                DriveListFormat::Table => print_queue_table(&items),
            }
        }
        DriveQueueCommand::Retry => {
            let retried = crate::build_agent().await?.retry_sync_queue().await?;
            println!(
                "{}",
                serde_json::to_string_pretty(&serde_json::json!({ "retried": retried }))?
            );
            Ok(())
        }
    }
}

/// Runs `drive conflict`.
///
/// # Errors
/// Returns an error when conflict metadata or local files cannot be updated.
pub async fn run_drive_conflict(command: DriveConflictCommand) -> Result<()> {
    match command {
        DriveConflictCommand::List(args) => {
            let items = crate::build_agent().await?.conflicts().await?;
            match args.format {
                DriveListFormat::Json => {
                    println!("{}", serde_json::to_string_pretty(&items)?);
                    Ok(())
                }
                DriveListFormat::Table => print_conflict_table(&items),
            }
        }
        DriveConflictCommand::Resolve(args) => {
            let resolution = if args.keep_local {
                ConflictResolution::KeepLocal
            } else if let Some(path) = args.merged {
                ConflictResolution::UseMerged(path)
            } else {
                ConflictResolution::KeepRemote
            };
            let resolved = crate::build_agent()
                .await?
                .resolve_conflict(args.id, resolution)
                .await?;
            println!("{}", serde_json::to_string_pretty(&resolved)?);
            Ok(())
        }
    }
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

fn print_pool_table(status: &serde_json::Value) -> Result<()> {
    println!(
        "{:<10} {:<10} {:<12} {:<12} REMOTE",
        "KIND", "NAME", "OWNER", "MODE"
    );
    if let Some(pools) = status.get("pools").and_then(serde_json::Value::as_array) {
        for pool in pools {
            println!(
                "{:<10} {:<10} {:<12} {:<12} {}",
                "pool",
                pool.get("name")
                    .and_then(serde_json::Value::as_str)
                    .unwrap_or("-"),
                pool.get("owner_drive_id")
                    .and_then(serde_json::Value::as_str)
                    .unwrap_or("-"),
                if pool
                    .get("readonly")
                    .and_then(serde_json::Value::as_bool)
                    .unwrap_or(false)
                {
                    "readonly"
                } else {
                    "writable"
                },
                pool.get("remote_url")
                    .and_then(serde_json::Value::as_str)
                    .unwrap_or("-")
            );
        }
    }
    if let Some(mounts) = status.get("mounts").and_then(serde_json::Value::as_array) {
        for mount in mounts {
            println!(
                "{:<10} {:<10} {:<12} {:<12} {}",
                "mount",
                mount
                    .get("name")
                    .and_then(serde_json::Value::as_str)
                    .unwrap_or("-"),
                mount
                    .get("owner_drive_id")
                    .and_then(serde_json::Value::as_str)
                    .unwrap_or("-"),
                if mount
                    .get("readonly")
                    .and_then(serde_json::Value::as_bool)
                    .unwrap_or(true)
                {
                    "readonly"
                } else {
                    "writable"
                },
                mount
                    .get("remote_url")
                    .and_then(serde_json::Value::as_str)
                    .unwrap_or("-")
            );
        }
    }
    Ok(())
}

fn print_queue_table(items: &[DriveSyncQueueItem]) -> Result<()> {
    println!(
        "{:<8} {:<12} {:<9} {:<48} ERROR",
        "STATUS", "KIND", "ATTEMPTS", "REMOTE"
    );
    for item in items {
        println!(
            "{:<8} {:<12} {:<9} {:<48} {}",
            queue_status_text(item.status),
            queue_kind_text(item.kind),
            item.attempts,
            item.remote_path,
            item.last_error.as_deref().unwrap_or("-")
        );
    }
    Ok(())
}

fn print_conflict_table(items: &[DriveConflict]) -> Result<()> {
    println!(
        "{:<36} {:<36} {:<16} CONFLICT_COPY",
        "ID", "ENTRY", "DEVICE"
    );
    for item in items {
        println!(
            "{:<36} {:<36} {:<16} {}",
            item.id, item.entry_id, item.device_id, item.conflict_path
        );
    }
    Ok(())
}

fn parse_size_bytes(raw: &str) -> Result<u64> {
    let value = raw.trim().to_ascii_lowercase();
    let (number, multiplier) = if let Some(number) = value.strip_suffix("gb") {
        (number, 1024_u64.pow(3))
    } else if let Some(number) = value.strip_suffix("g") {
        (number, 1024_u64.pow(3))
    } else if let Some(number) = value.strip_suffix("mb") {
        (number, 1024_u64.pow(2))
    } else if let Some(number) = value.strip_suffix("m") {
        (number, 1024_u64.pow(2))
    } else if let Some(number) = value.strip_suffix("kb") {
        (number, 1024)
    } else if let Some(number) = value.strip_suffix("k") {
        (number, 1024)
    } else {
        (value.as_str(), 1)
    };
    let number = number.trim().parse::<u64>()?;
    Ok(number.saturating_mul(multiplier))
}

impl From<DriveQueueStatusArg> for DriveSyncTaskStatus {
    fn from(value: DriveQueueStatusArg) -> Self {
        match value {
            DriveQueueStatusArg::Pending => Self::Pending,
            DriveQueueStatusArg::Running => Self::Running,
            DriveQueueStatusArg::Done => Self::Done,
            DriveQueueStatusArg::Failed => Self::Failed,
        }
    }
}

fn status_text(status: TrackedItemStatus) -> &'static str {
    match status {
        TrackedItemStatus::Tracked => "tracked",
        TrackedItemStatus::MissingLocal => "missing_local",
        TrackedItemStatus::RemoteOnly => "remote_only",
        TrackedItemStatus::Ignored => "ignored",
        TrackedItemStatus::ConflictSuspended => "conflict_suspended",
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
        TrackedItemSource::Suspended => "suspended",
    }
}

fn queue_status_text(status: DriveSyncTaskStatus) -> &'static str {
    match status {
        DriveSyncTaskStatus::Pending => "pending",
        DriveSyncTaskStatus::Running => "running",
        DriveSyncTaskStatus::Done => "done",
        DriveSyncTaskStatus::Failed => "failed",
    }
}

fn queue_kind_text(kind: az_drive_store::DriveSyncTaskKind) -> &'static str {
    match kind {
        az_drive_store::DriveSyncTaskKind::Upload => "upload",
        az_drive_store::DriveSyncTaskKind::Download => "download",
        az_drive_store::DriveSyncTaskKind::Materialize => "materialize",
        az_drive_store::DriveSyncTaskKind::Conflict => "conflict",
    }
}

fn pull_status_text(status: PullRemoteStatus) -> &'static str {
    match status {
        PullRemoteStatus::Pulled => "synced",
        PullRemoteStatus::AlreadyCurrent => "already_current",
        PullRemoteStatus::SkippedExisting => "skipped_existing",
        PullRemoteStatus::SkippedIgnored => "skipped_ignored",
        PullRemoteStatus::SkippedSuspended => "skipped_suspended",
        PullRemoteStatus::SkippedNoVersion => "skipped_no_version",
        PullRemoteStatus::DryRun => "dry_run",
    }
}
