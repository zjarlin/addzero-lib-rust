#![cfg(not(target_arch = "wasm32"))]

use std::fs;
use std::path::PathBuf;
use std::process::Command;

use az_derive_aliases::{apply, serde_eq_default};

#[apply(serde_eq_default)]
pub struct CloudflareTunnelStatusDto {
    pub config_path: String,
    pub config_exists: bool,
    pub tunnel_running: bool,
    pub host_count: usize,
    pub http_count: usize,
    pub tcp_count: usize,
    pub cli_commands: Vec<CloudflareTunnelCliCommandDto>,
    pub hosts: Vec<CloudflareTunnelHostDto>,
}

#[apply(serde_eq_default)]
pub struct CloudflareTunnelCliCommandDto {
    pub name: String,
    pub path: String,
    pub installed: bool,
}

#[apply(serde_eq_default)]
pub struct CloudflareTunnelHostDto {
    pub hostname: String,
    pub service: String,
    pub mode: String,
    pub local: String,
    pub port: String,
}

pub async fn cloudflare_tunnel_status_on_server() -> Result<CloudflareTunnelStatusDto, String> {
    tokio::task::spawn_blocking(cloudflare_tunnel_status_blocking)
        .await
        .map_err(|err| format!("read cloudflare tunnel status task failed: {err}"))?
}

fn cloudflare_tunnel_status_blocking() -> Result<CloudflareTunnelStatusDto, String> {
    let config_path = dirs::home_dir()
        .ok_or_else(|| "home directory is unavailable".to_string())?
        .join(".cloudflared/config.yml");
    let hosts =
        if config_path.is_file() {
            parse_ingress_hosts(&fs::read_to_string(&config_path).map_err(|err| {
                format!("read cloudflared config {}: {err}", config_path.display())
            })?)?
        } else {
            Vec::new()
        };
    let http_count = hosts
        .iter()
        .filter(|host| matches!(host.mode.as_str(), "http" | "https"))
        .count();
    let tcp_count = hosts.iter().filter(|host| host.mode == "tcp").count();
    Ok(CloudflareTunnelStatusDto {
        config_path: config_path.display().to_string(),
        config_exists: config_path.is_file(),
        tunnel_running: cloudflared_running(),
        host_count: hosts.len(),
        http_count,
        tcp_count,
        cli_commands: cli_command_statuses(),
        hosts,
    })
}

fn parse_ingress_hosts(content: &str) -> Result<Vec<CloudflareTunnelHostDto>, String> {
    let mut hosts = Vec::new();
    let mut in_ingress = false;
    let mut pending_hostname: Option<String> = None;
    for raw in content.lines() {
        let line = raw.trim();
        if line.is_empty() {
            continue;
        }
        if line == "ingress:" {
            in_ingress = true;
            continue;
        }
        if !in_ingress {
            continue;
        }
        if let Some(hostname) = line.strip_prefix("- hostname:") {
            pending_hostname = Some(hostname.trim().to_string());
            continue;
        }
        if let Some(service) = line.strip_prefix("service:") {
            let Some(hostname) = pending_hostname.take() else {
                continue;
            };
            hosts.push(host_from_service(hostname, service.trim()));
            continue;
        }
        if line.starts_with("- service:") {
            pending_hostname = None;
            continue;
        }
        if line.starts_with('-') {
            return Err(format!("unsupported ingress line: {line}"));
        }
    }
    Ok(hosts)
}

fn host_from_service(hostname: String, service: &str) -> CloudflareTunnelHostDto {
    let (mode, local, port) = split_service(service);
    CloudflareTunnelHostDto {
        hostname,
        service: service.to_string(),
        mode,
        local,
        port,
    }
}

fn split_service(service: &str) -> (String, String, String) {
    let Some((mode, rest)) = service.split_once("://") else {
        return ("unknown".to_string(), String::new(), String::new());
    };
    let Some((host, port)) = rest.rsplit_once(':') else {
        return (mode.to_string(), rest.to_string(), String::new());
    };
    (mode.to_string(), host.to_string(), port.to_string())
}

fn cloudflared_running() -> bool {
    let Ok(output) = Command::new("ps").arg("aux").output() else {
        return false;
    };
    let stdout = String::from_utf8_lossy(&output.stdout);
    stdout.contains("cloudflared --config") && stdout.contains("tunnel run")
}

fn cli_command_statuses() -> Vec<CloudflareTunnelCliCommandDto> {
    ["addhost", "showhost", "rmhost", "autohost"]
        .into_iter()
        .map(|name| {
            let path = cli_bin_path(name);
            CloudflareTunnelCliCommandDto {
                name: name.to_string(),
                installed: path.is_file(),
                path: path.display().to_string(),
            }
        })
        .collect()
}

fn cli_bin_path(name: &str) -> PathBuf {
    dirs::home_dir()
        .map(|home| home.join(".local/bin").join(name))
        .unwrap_or_else(|| PathBuf::from(name))
}
