use std::{net::SocketAddr, path::PathBuf};

use anyhow::{Context, Result, bail};
use clap::Parser;

use crate::{
    command::SystemCommandRunner,
    config::{RelayConfig, StoragePaths, load_config, normalize_host, normalize_name, save_config},
    mapping::{load_mappings, save_mappings},
    relay, relay_server,
    route_table::RouteTable,
    tunnel,
};

/// 将本机 HTTP 端口发布到已配置公网域名的命令行入口。
#[derive(Debug, Parser)]
#[command(
    name = "addhost",
    version,
    about = "通过 SSH 反向隧道和内置 relay 将本机端口发布到公网域名",
    args_conflicts_with_subcommands = true
)]
pub struct Cli {
    /// 要发布的单级子域名，例如 demo。
    #[arg(value_name = "NAME")]
    name: Option<String>,

    /// 本机服务监听端口。
    #[arg(value_name = "PORT")]
    port: Option<u16>,

    #[command(subcommand)]
    command: Option<CliCommand>,
}

#[derive(Debug, clap::Subcommand)]
enum CliCommand {
    /// 保存中转机配置，并初始化远端 addhost relay。
    Init(InitArgs),
    /// 列出全部已保存映射。
    List,
    /// 检查全部或指定映射的隧道状态。
    Status(StatusArgs),
    /// 删除指定映射、SSH 隧道和 relay 路由。
    Remove(RemoveArgs),
    /// 运行或管理公网 HTTP relay。
    Relay(RelayArgs),
}

#[derive(Debug, clap::Args)]
struct InitArgs {
    /// SSH 目标，支持 user@host 或 ~/.ssh/config 中的别名。
    #[arg(long)]
    server: String,

    /// 用于发布服务的基础域名，例如 dev.example.com。
    #[arg(long)]
    domain: String,

    /// 只保存本地配置，不安装公网 relay 服务。
    #[arg(long)]
    skip_relay_prepare: bool,
}

#[derive(Debug, clap::Args)]
struct StatusArgs {
    /// 可选的单级子域名。
    name: Option<String>,
}

#[derive(Debug, clap::Args)]
struct RemoveArgs {
    /// 要删除的单级子域名。
    name: String,
}

#[derive(Debug, clap::Args)]
struct RelayArgs {
    #[command(subcommand)]
    command: RelayCommand,
}

#[derive(Debug, clap::Subcommand)]
enum RelayCommand {
    /// 启动公网 HTTP relay 服务。
    Serve(RelayServeArgs),
    /// 修改 relay 域名路由表。
    Route(RelayRouteArgs),
}

#[derive(Debug, clap::Args)]
struct RelayServeArgs {
    /// relay 对外监听地址。
    #[arg(long, default_value = "0.0.0.0:80")]
    listen: SocketAddr,

    /// 域名路由文件。
    #[arg(long, default_value = "/etc/addhost/routes.toml")]
    routes: PathBuf,
}

#[derive(Debug, clap::Args)]
struct RelayRouteArgs {
    #[command(subcommand)]
    command: RelayRouteCommand,

    /// 域名路由文件。
    #[arg(long, default_value = "/etc/addhost/routes.toml", global = true)]
    routes: PathBuf,
}

#[derive(Debug, clap::Subcommand)]
enum RelayRouteCommand {
    /// 新增或替换域名路由。
    Set { host: String, port: u16 },
    /// 删除域名路由。
    Remove { host: String },
    /// 列出域名路由。
    List,
}

/// 解析当前进程参数并执行命令。
pub fn run_from_env() -> Result<()> {
    run(Cli::parse())
}

/// 执行一个已经解析的 CLI 请求。
pub fn run(cli: Cli) -> Result<()> {
    let paths = StoragePaths::discover()?;
    let runner = SystemCommandRunner;

    match cli.command {
        Some(CliCommand::Init(args)) => run_init(&runner, &paths, args),
        Some(CliCommand::List) => run_list(&paths),
        Some(CliCommand::Status(args)) => run_status(&runner, &paths, args),
        Some(CliCommand::Remove(args)) => run_remove(&runner, &paths, args),
        Some(CliCommand::Relay(args)) => run_relay(args),
        None => {
            let name = cli
                .name
                .ok_or_else(|| anyhow::anyhow!("缺少子域名；用法：addhost <NAME> <PORT>"))?;
            let port = cli
                .port
                .ok_or_else(|| anyhow::anyhow!("缺少本地端口；用法：addhost <NAME> <PORT>"))?;
            run_publish(&runner, &paths, &name, port)
        }
    }
}

fn run_init(runner: &SystemCommandRunner, paths: &StoragePaths, args: InitArgs) -> Result<()> {
    let config = RelayConfig::create(args.server, args.domain)?;
    if !args.skip_relay_prepare {
        relay::prepare(runner, &config)?;
    }
    save_config(paths, &config)?;

    println!("已初始化 addhost");
    println!("域名：*.{}", config.domain);
    println!("公网机：{}", config.server);
    println!("下一步：addhost demo 8080");
    Ok(())
}

fn run_publish(
    runner: &SystemCommandRunner,
    paths: &StoragePaths,
    raw_name: &str,
    local_port: u16,
) -> Result<()> {
    let config = load_config(paths)?;
    let name = normalize_name(raw_name)?;
    let mut registry = load_mappings(paths)?;
    let mapping = tunnel::publish(runner, &config, paths, &registry, &name, local_port)?;
    registry.upsert(mapping.clone());
    save_mappings(paths, &registry)?;

    println!("已发布 http://127.0.0.1:{}", mapping.local_port);
    println!("公网地址：http://{}", mapping.host);
    Ok(())
}

fn run_list(paths: &StoragePaths) -> Result<()> {
    let registry = load_mappings(paths)?;
    if registry.mappings.is_empty() {
        println!("暂无已保存映射");
        return Ok(());
    }

    for mapping in registry.mappings {
        println!(
            "{} -> 127.0.0.1:{}（远端回环端口 {}）",
            mapping.host, mapping.local_port, mapping.remote_port
        );
    }
    Ok(())
}

fn run_status(runner: &SystemCommandRunner, paths: &StoragePaths, args: StatusArgs) -> Result<()> {
    let config = load_config(paths)?;
    let registry = load_mappings(paths)?;
    let requested_name = args.name.as_deref().map(normalize_name).transpose()?;
    let mut matched = false;

    for mapping in &registry.mappings {
        if requested_name
            .as_deref()
            .is_some_and(|name| name != mapping.name)
        {
            continue;
        }
        matched = true;
        let status = if tunnel::is_active(runner, &config, paths, mapping)? {
            "在线"
        } else {
            "离线"
        };
        println!("{} {}", status, mapping.host);
    }

    if requested_name.is_some() && !matched {
        bail!("未找到指定映射");
    }
    if requested_name.is_none() && registry.mappings.is_empty() {
        println!("暂无已保存映射");
    }
    Ok(())
}

fn run_remove(runner: &SystemCommandRunner, paths: &StoragePaths, args: RemoveArgs) -> Result<()> {
    let config = load_config(paths)?;
    let name = normalize_name(&args.name)?;
    let mut registry = load_mappings(paths)?;
    let mapping = registry
        .remove(&name)
        .ok_or_else(|| anyhow::anyhow!("未找到映射：{name}"))?;

    tunnel::remove(runner, &config, paths, &mapping)?;
    save_mappings(paths, &registry)?;
    println!("已删除 http://{}", mapping.host);
    Ok(())
}

fn run_relay(args: RelayArgs) -> Result<()> {
    match args.command {
        RelayCommand::Serve(args) => {
            let runtime = tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .build()
                .context("创建 relay 异步运行时失败")?;
            runtime.block_on(relay_server::serve(args.listen, args.routes))
        }
        RelayCommand::Route(args) => run_relay_route(args),
    }
}

fn run_relay_route(args: RelayRouteArgs) -> Result<()> {
    let mut table = RouteTable::load(&args.routes)?;
    match args.command {
        RelayRouteCommand::Set { host, port } => {
            if port == 0 {
                bail!("relay 回环端口不能为 0");
            }
            let host = table.set(&host, port)?;
            table.save(&args.routes)?;
            println!("已设置 {host} -> 127.0.0.1:{port}");
        }
        RelayRouteCommand::Remove { host } => {
            let host = normalize_host(&host)?;
            if table.remove(&host)?.is_none() {
                bail!("未找到 relay 路由：{host}");
            }
            table.save(&args.routes)?;
            println!("已删除 relay 路由：{host}");
        }
        RelayRouteCommand::List => {
            for (host, port) in table.routes {
                println!("{host} -> 127.0.0.1:{port}");
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use anyhow::Result;
    use clap::CommandFactory;

    use super::*;

    #[test]
    fn parses_short_publish_syntax() -> Result<()> {
        let cli = Cli::try_parse_from(["addhost", "demo", "12345"])?;
        assert_eq!(cli.name.as_deref(), Some("demo"));
        assert_eq!(cli.port, Some(12_345));
        assert!(cli.command.is_none());
        Ok(())
    }

    #[test]
    fn exposes_management_commands() {
        let command = Cli::command();
        let names: Vec<&str> = command
            .get_subcommands()
            .map(clap::Command::get_name)
            .collect();
        assert_eq!(names, vec!["init", "list", "status", "remove", "relay"]);
    }
}
