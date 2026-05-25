use std::{
    env,
    net::TcpListener,
    path::{Path, PathBuf},
    process::{Child, Command, Stdio},
    thread,
    time::{Duration, Instant},
};

use anyhow::{Context, Result, bail};
use az_aio_client::AioClient;
use az_derive_aliases::{apply, plain_clone_debug, plain_debug};
use uuid::Uuid;

const DEFAULT_STARTUP_TIMEOUT: Duration = Duration::from_secs(90);
const DEFAULT_READY_POLL_INTERVAL: Duration = Duration::from_millis(150);

#[apply(plain_clone_debug)]
pub struct DesktopRuntimeOptions {
    pub backend_bin: Option<PathBuf>,
    pub bind: Option<String>,
    pub desktop_token: Option<String>,
    pub extra_env: Vec<(String, String)>,
    pub startup_timeout: Duration,
}

#[apply(plain_debug)]
enum BackendLaunchSpec {
    Binary(PathBuf),
    CargoWorkspace(PathBuf),
}

#[apply(plain_debug)]
pub struct DesktopRuntime {
    _backend: EmbeddedBackendProcess,
    #[allow(dead_code)]
    client: AioClient,
    base_url: String,
    desktop_token: String,
}

impl DesktopRuntime {
    pub fn start() -> Result<Self> {
        Self::start_with_options(DesktopRuntimeOptions::default())
    }

    pub fn start_with_options(options: DesktopRuntimeOptions) -> Result<Self> {
        let bind = options.bind.unwrap_or_else(choose_loopback_bind);
        let desktop_token = options
            .desktop_token
            .unwrap_or_else(|| Uuid::new_v4().to_string());
        let base_url = format!("http://{bind}");
        let mut backend = EmbeddedBackendProcess::spawn(
            resolve_backend_launch_spec(options.backend_bin.clone())?,
            &bind,
            &desktop_token,
            &options.extra_env,
        )?;
        let client = AioClient::with_desktop_token(base_url.clone(), desktop_token.clone());

        wait_until_ready(&client, &mut backend.child, options.startup_timeout)
            .with_context(|| format!("waiting for embedded aio backend at {base_url}"))?;

        Ok(Self {
            _backend: backend,
            client,
            base_url,
            desktop_token,
        })
    }

    #[allow(dead_code)]
    pub fn client(&self) -> &AioClient {
        &self.client
    }

    #[allow(dead_code)]
    pub fn client_clone(&self) -> AioClient {
        self.client.clone()
    }

    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    pub fn desktop_token(&self) -> &str {
        &self.desktop_token
    }
}

#[apply(plain_debug)]
struct EmbeddedBackendProcess {
    child: Child,
}

impl EmbeddedBackendProcess {
    fn spawn(
        spec: BackendLaunchSpec,
        bind: &str,
        desktop_token: &str,
        extra_env: &[(String, String)],
    ) -> Result<Self> {
        let mut command = match spec {
            BackendLaunchSpec::Binary(path) => {
                let mut command = Command::new(&path);
                command.arg("serve");
                command
            }
            BackendLaunchSpec::CargoWorkspace(workspace_root) => {
                let mut command = Command::new("cargo");
                command
                    .current_dir(workspace_root)
                    .arg("run")
                    .arg("-p")
                    .arg("aio")
                    .arg("--bin")
                    .arg("aio")
                    .arg("--")
                    .arg("serve");
                command
            }
        };

        command
            .arg("--bind")
            .arg(bind)
            .arg("--desktop-token")
            .arg(desktop_token)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::inherit());
        for (key, value) in extra_env {
            command.env(key, value);
        }

        let child = command
            .spawn()
            .context("spawn embedded aio backend process")?;
        Ok(Self { child })
    }
}

impl Drop for EmbeddedBackendProcess {
    fn drop(&mut self) {
        if self.child.try_wait().ok().flatten().is_none() {
            let _ = self.child.kill();
            let _ = self.child.wait();
        }
    }
}

fn choose_loopback_bind() -> String {
    TcpListener::bind("127.0.0.1:0")
        .and_then(|listener| listener.local_addr())
        .map(|address| address.to_string())
        .unwrap_or_else(|_| "127.0.0.1:8787".to_string())
}

fn wait_until_ready(client: &AioClient, child: &mut Child, timeout: Duration) -> Result<()> {
    let deadline = Instant::now() + timeout;
    loop {
        if client.desktop_status().is_ok() {
            return Ok(());
        }

        if let Some(status) = child.try_wait().context("poll aio backend child process")? {
            bail!("embedded aio backend exited early with status {status}");
        }

        if Instant::now() >= deadline {
            bail!("timed out waiting for embedded aio backend readiness");
        }

        thread::sleep(DEFAULT_READY_POLL_INTERVAL);
    }
}

fn resolve_backend_launch_spec(override_bin: Option<PathBuf>) -> Result<BackendLaunchSpec> {
    if let Some(path) = override_bin.filter(|path| path.is_file()) {
        return Ok(BackendLaunchSpec::Binary(path));
    }

    if let Some(path) = env::var_os("AIO_BACKEND_BIN")
        .map(PathBuf::from)
        .filter(|path| path.is_file())
    {
        return Ok(BackendLaunchSpec::Binary(path));
    }

    let workspace_root = workspace_root()?;
    if workspace_root.join("Cargo.toml").is_file() {
        return Ok(BackendLaunchSpec::CargoWorkspace(workspace_root));
    }

    if let Some(path) = backend_binary_candidates()
        .into_iter()
        .find(|path| path.is_file())
    {
        return Ok(BackendLaunchSpec::Binary(path));
    }

    bail!("cannot resolve aio backend launch strategy")
}

fn backend_binary_candidates() -> Vec<PathBuf> {
    let mut candidates = Vec::new();
    if let Ok(root) = workspace_root() {
        candidates.push(root.join("target/debug/aio"));
        candidates.push(root.join("target/release/aio"));
    }
    if let Ok(current_exe) = env::current_exe() {
        if let Some(grand_parent) = current_exe.parent().and_then(|parent| parent.parent()) {
            candidates.push(grand_parent.join("aio"));
        }
        if let Some(parent) = current_exe.parent() {
            candidates.push(parent.join("aio"));
        }
    }
    candidates
}

fn workspace_root() -> Result<PathBuf> {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    manifest_dir
        .ancestors()
        .nth(3)
        .map(PathBuf::from)
        .context("resolve workspace root from apps/aio/desktop")
}

#[cfg(test)]
mod tests {
    use std::{fs, path::PathBuf, process::Command};

    use anyhow::Result;
    use az_aio_client::{AioClient, AioClientError};
    use az_config_center_contract::{
        ShellComponentBuildRequest, ShellComponentConfigUpdate, ShellComponentKind,
        ShellComponentPatch, ShellComponentRemove, ShellComponentUpsert,
    };

    use super::{DesktopRuntime, DesktopRuntimeOptions};

    #[test]
    fn embedded_backend_flow_persists_preview_apply_and_reload() -> Result<()> {
        let dir = tempfile::tempdir()?;
        let config_path = dir.path().join("shell-components.json");
        let output_path = dir.path().join(".add_fn");
        let backend_bin = build_backend_bin()?;
        let extra_env = vec![
            (
                "AIO_SHELL_COMPONENTS_CONFIG".to_string(),
                config_path.display().to_string(),
            ),
            (
                "AIO_SHELL_COMPONENTS_OUTPUT".to_string(),
                output_path.display().to_string(),
            ),
        ];

        let runtime = DesktopRuntime::start_with_options(DesktopRuntimeOptions {
            backend_bin: Some(backend_bin.clone()),
            extra_env: extra_env.clone(),
            ..DesktopRuntimeOptions::default()
        })?;
        let client = runtime.client_clone();

        let unauthorized = AioClient::new(runtime.base_url()).desktop_status();
        assert!(
            matches!(unauthorized, Err(AioClientError::Http { status, .. }) if status.as_u16() == 401)
        );

        let status = client.desktop_status()?;
        assert!(status.desktop_mode);
        assert_eq!(
            status.shell_registry_path,
            config_path.display().to_string()
        );

        let registry = client.list_shell_components()?;
        assert!(registry.components.is_empty());

        client.upsert_shell_component(&ShellComponentUpsert {
            name: "JAVA_HOME".to_string(),
            kind: ShellComponentKind::Export,
            summary: "jdk path".to_string(),
            enabled: true,
            render_to_output: true,
            export_value: Some("/Library/Java/JavaVirtualMachines".to_string()),
            alias_command: None,
            body: None,
        })?;
        client.upsert_shell_component(&ShellComponentUpsert {
            name: "ll".to_string(),
            kind: ShellComponentKind::Alias,
            summary: "list files".to_string(),
            enabled: true,
            render_to_output: true,
            export_value: None,
            alias_command: Some("ls -lah".to_string()),
            body: None,
        })?;
        client.patch_shell_component(&ShellComponentPatch {
            name: "ll".to_string(),
            summary: Some("compact ls".to_string()),
            enabled: Some(false),
            render_to_output: Some(false),
        })?;
        client.save_shell_component_config(&ShellComponentConfigUpdate {
            output_path: Some(output_path.display().to_string()),
        })?;

        let preview = client.build_shell_components(&ShellComponentBuildRequest {
            output_path: None,
            write: false,
        })?;
        assert!(!preview.written);
        assert!(!output_path.exists());
        assert!(preview.content.contains("export JAVA_HOME="));
        assert!(!preview.content.contains("alias ll="));

        let applied = client.build_shell_components(&ShellComponentBuildRequest {
            output_path: None,
            write: true,
        })?;
        assert!(applied.written);
        let written = fs::read_to_string(&output_path)?;
        assert_eq!(written, applied.content);

        client.remove_shell_component(&ShellComponentRemove {
            name: "ll".to_string(),
        })?;

        drop(runtime);

        let restarted = DesktopRuntime::start_with_options(DesktopRuntimeOptions {
            backend_bin: Some(backend_bin),
            extra_env,
            ..DesktopRuntimeOptions::default()
        })?;
        let registry = restarted.client().list_shell_components()?;
        assert_eq!(registry.components.len(), 1);
        assert_eq!(registry.components[0].name, "JAVA_HOME");

        Ok(())
    }

    fn build_backend_bin() -> Result<PathBuf> {
        let workspace_root = super::workspace_root()?;
        let status = Command::new("cargo")
            .current_dir(&workspace_root)
            .arg("build")
            .arg("-p")
            .arg("aio")
            .arg("--bin")
            .arg("aio")
            .status()?;
        if !status.success() {
            anyhow::bail!("cargo build -p aio --bin aio failed with {status}");
        }
        Ok(workspace_root.join("target/debug/aio"))
    }
}
impl Default for DesktopRuntimeOptions {
    fn default() -> Self {
        Self {
            backend_bin: None,
            bind: None,
            desktop_token: None,
            extra_env: Vec::new(),
            startup_timeout: DEFAULT_STARTUP_TIMEOUT,
        }
    }
}
