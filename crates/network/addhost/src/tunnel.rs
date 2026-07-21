use std::{
    collections::HashSet,
    fs,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result, bail};

use crate::{
    command::{CommandOutput, CommandRequest, CommandRunner},
    config::{RelayConfig, StoragePaths},
    mapping::{HostMapping, MappingRegistry},
    relay,
};

const MAX_PORT_ATTEMPTS: usize = 32;

pub(crate) fn publish(
    runner: &dyn CommandRunner,
    config: &RelayConfig,
    paths: &StoragePaths,
    registry: &MappingRegistry,
    name: &str,
    local_port: u16,
) -> Result<HostMapping> {
    if local_port == 0 {
        bail!("本地端口不能为 0");
    }

    fs::create_dir_all(&paths.state_dir)
        .with_context(|| format!("创建状态目录失败：{}", paths.state_dir.display()))?;

    let host = format!("{name}.{}", config.domain);
    let control_socket = control_socket(paths, &host);
    stop_controlled_tunnel(runner, config, &control_socket)?;

    let existing_remote_port = registry.find(name).map(|mapping| mapping.remote_port);
    let remote_port = start_available_tunnel(
        runner,
        config,
        registry,
        &host,
        local_port,
        existing_remote_port,
        &control_socket,
    )?;

    if let Err(error) = relay::install_route(runner, config, &host, remote_port) {
        let _ = stop_controlled_tunnel(runner, config, &control_socket);
        return Err(error);
    }

    Ok(HostMapping {
        name: name.to_owned(),
        host,
        local_port,
        remote_port,
    })
}

pub(crate) fn remove(
    runner: &dyn CommandRunner,
    config: &RelayConfig,
    paths: &StoragePaths,
    mapping: &HostMapping,
) -> Result<()> {
    let control_socket = control_socket(paths, &mapping.host);
    stop_controlled_tunnel(runner, config, &control_socket)?;
    relay::remove_route(runner, config, &mapping.host)
}

pub(crate) fn is_active(
    runner: &dyn CommandRunner,
    config: &RelayConfig,
    paths: &StoragePaths,
    mapping: &HostMapping,
) -> Result<bool> {
    let socket = control_socket(paths, &mapping.host);
    if !socket.exists() {
        return Ok(false);
    }

    let args = vec![
        "-S".to_owned(),
        socket.display().to_string(),
        "-O".to_owned(),
        "check".to_owned(),
        config.server.clone(),
    ];
    let output = runner.run(CommandRequest {
        program: "ssh",
        args: &args,
        stdin: None,
    })?;
    Ok(output.status == 0)
}

fn start_available_tunnel(
    runner: &dyn CommandRunner,
    config: &RelayConfig,
    registry: &MappingRegistry,
    host: &str,
    local_port: u16,
    existing_remote_port: Option<u16>,
    control_socket: &Path,
) -> Result<u16> {
    let used_ports: HashSet<u16> = registry
        .mappings
        .iter()
        .filter(|mapping| mapping.host != host)
        .map(|mapping| mapping.remote_port)
        .collect();

    for remote_port in candidate_ports(config, host, existing_remote_port) {
        if used_ports.contains(&remote_port) {
            continue;
        }

        let output = start_tunnel(runner, config, control_socket, local_port, remote_port)?;
        if output.status == 0 {
            return Ok(remote_port);
        }
        if !is_remote_port_conflict(&output) {
            output.ensure_success("建立 SSH 反向隧道")?;
        }
    }

    bail!("连续尝试 {MAX_PORT_ATTEMPTS} 个远端端口仍无法建立隧道")
}

fn candidate_ports(
    config: &RelayConfig,
    host: &str,
    existing_remote_port: Option<u16>,
) -> Vec<u16> {
    let port_count = u32::from(config.remote_port_end - config.remote_port_start) + 1;
    let first_offset = (stable_hash(host) % u64::from(port_count)) as u32;
    let mut ports = Vec::with_capacity(MAX_PORT_ATTEMPTS + 1);

    if let Some(remote_port) = existing_remote_port {
        ports.push(remote_port);
    }

    for offset in 0..MAX_PORT_ATTEMPTS as u32 {
        let candidate_offset = (first_offset + offset) % port_count;
        let candidate = u32::from(config.remote_port_start) + candidate_offset;
        let candidate = candidate as u16;
        if !ports.contains(&candidate) {
            ports.push(candidate);
        }
    }
    ports
}

fn start_tunnel(
    runner: &dyn CommandRunner,
    config: &RelayConfig,
    control_socket: &Path,
    local_port: u16,
    remote_port: u16,
) -> Result<CommandOutput> {
    let forward = format!("127.0.0.1:{remote_port}:127.0.0.1:{local_port}");
    let args = vec![
        "-fNT".to_owned(),
        "-M".to_owned(),
        "-S".to_owned(),
        control_socket.display().to_string(),
        "-o".to_owned(),
        "ExitOnForwardFailure=yes".to_owned(),
        "-o".to_owned(),
        "ServerAliveInterval=30".to_owned(),
        "-o".to_owned(),
        "ServerAliveCountMax=3".to_owned(),
        "-R".to_owned(),
        forward,
        config.server.clone(),
    ];
    runner.run(CommandRequest {
        program: "ssh",
        args: &args,
        stdin: None,
    })
}

fn stop_controlled_tunnel(
    runner: &dyn CommandRunner,
    config: &RelayConfig,
    control_socket: &Path,
) -> Result<()> {
    if !control_socket.exists() {
        return Ok(());
    }

    let args = vec![
        "-S".to_owned(),
        control_socket.display().to_string(),
        "-O".to_owned(),
        "exit".to_owned(),
        config.server.clone(),
    ];
    let output = runner.run(CommandRequest {
        program: "ssh",
        args: &args,
        stdin: None,
    })?;

    if output.status != 0 && !is_missing_control_socket(&output) {
        output.ensure_success("停止旧 SSH 反向隧道")?;
    }
    if control_socket.exists() {
        fs::remove_file(control_socket).with_context(|| {
            format!("删除失效 SSH 控制套接字失败：{}", control_socket.display())
        })?;
    }
    Ok(())
}

fn control_socket(paths: &StoragePaths, host: &str) -> PathBuf {
    paths
        .state_dir
        .join(format!("{:016x}.sock", stable_hash(host)))
}

fn stable_hash(value: &str) -> u64 {
    let mut hash = 0xcbf29ce484222325_u64;
    for byte in value.as_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash
}

fn is_remote_port_conflict(output: &CommandOutput) -> bool {
    let detail = format!("{}\n{}", output.stdout, output.stderr).to_ascii_lowercase();
    detail.contains("remote port forwarding failed")
        || detail.contains("port forwarding failed")
        || detail.contains("address already in use")
}

fn is_missing_control_socket(output: &CommandOutput) -> bool {
    let detail = format!("{}\n{}", output.stdout, output.stderr).to_ascii_lowercase();
    detail.contains("control socket connect") || detail.contains("no such file")
}

#[cfg(test)]
mod tests {
    use anyhow::Result;

    use super::*;

    #[test]
    fn candidate_ports_are_stable_and_in_range() -> Result<()> {
        let config = RelayConfig::create("root@example.com".to_owned(), "example.com".to_owned())?;
        let first = candidate_ports(&config, "demo.example.com", None);
        let second = candidate_ports(&config, "demo.example.com", None);

        assert_eq!(first, second);
        assert!(
            first.iter().all(|port| {
                *port >= config.remote_port_start && *port <= config.remote_port_end
            })
        );
        Ok(())
    }

    #[test]
    fn existing_remote_port_is_reused_first() -> Result<()> {
        let config = RelayConfig::create("root@example.com".to_owned(), "example.com".to_owned())?;
        let ports = candidate_ports(&config, "demo.example.com", Some(21_234));
        assert_eq!(ports.first(), Some(&21_234));
        Ok(())
    }
}
