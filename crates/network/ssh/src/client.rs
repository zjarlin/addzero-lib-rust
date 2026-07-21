//! SSH 远程连接客户端。
//!
//! 基于 [`ssh2`]（libssh2 Rust 绑定）提供阻塞式的 SSH 会话管理、远程命令执行和 SFTP 文件传输。
//!
//! # 核心类型
//!
//! - [`SshConfig`] — 连接配置，支持密码和私钥两种认证方式，通过 builder 模式构造
//! - [`SshSession`] — 已认证的 SSH 会话，提供 `execute_sync`、`execute_stream`、`upload_file`、`download_file` 等操作
//! - [`SshExecutionResult`] — 命令执行结果，包含 `exit_code`、`stdout`、`stderr`
//!
//! # 快速开始
//!
//! ```no_run
//! use az_ssh::client::{SshConfig, execute_sync};
//!
//! # fn example() -> anyhow::Result<()> {
//! let config = SshConfig::builder("192.168.1.100", "root")
//!     .password("your-password")
//!     .build()?;
//!
//! let result = execute_sync(&config, "uname -a")?;
//! if result.is_success() {
//!     println!("{}", result.stdout);
//! }
//! # Ok(())
//! # }
//! ```

use anyhow::{Context, Result, bail};
use ssh2::Session;
use std::fs::{self, File};
use std::io::{self, BufRead, BufReader, Read, Write};
use std::net::{TcpStream, ToSocketAddrs};
use std::path::{Path, PathBuf};
use std::thread;
use std::time::Duration;

/// SSH 连接配置，`Debug` 输出会隐藏密码、私钥路径和私钥口令。
#[derive(Clone, derive_more::Debug, Eq, PartialEq)]
pub struct SshConfig {
    /// SSH 主机名或 IP 地址。
    pub host: String,
    /// SSH 端口，默认 `22`。
    pub port: u16,
    /// 登录用户名。
    pub username: String,
    /// 密码认证使用的明文密码；`Debug` 中会被跳过。
    #[debug(skip)]
    pub password: Option<String>,
    /// 私钥认证使用的本地私钥路径；支持 `~` 和 `~/...` 展开。
    #[debug(skip)]
    pub private_key_path: Option<String>,
    /// 私钥口令；`Debug` 中会被跳过。
    #[debug(skip)]
    pub private_key_passphrase: Option<String>,
    /// TCP 连接超时，单位毫秒。
    pub connect_timeout_ms: u32,
    /// SSH 读写超时，单位毫秒。
    pub read_timeout_ms: u32,
}

impl SshConfig {
    /// 创建默认配置构建器，默认端口为 `22`。
    pub fn builder(host: impl Into<String>, username: impl Into<String>) -> SshConfigBuilder {
        Self {
            host: host.into(),
            port: 22,
            username: username.into(),
            password: None,
            private_key_path: None,
            private_key_passphrase: None,
            connect_timeout_ms: 30_000,
            read_timeout_ms: 60_000,
        }
    }

    /// 校验主机、用户名、端口和认证材料是否完整。
    pub fn validate(&self) -> Result<()> {
        if self.host.trim().is_empty() {
            bail!("invalid ssh configuration: host cannot be blank");
        }
        if self.username.trim().is_empty() {
            bail!("invalid ssh configuration: username cannot be blank");
        }
        if self.port == 0 {
            bail!("invalid ssh configuration: port must be greater than zero");
        }
        if self.password.is_none() && self.private_key_path.is_none() {
            bail!("invalid ssh configuration: password or private_key_path is required");
        }
        Ok(())
    }

    /// 设置 SSH 端口。
    pub fn port(mut self, value: u16) -> Self {
        self.port = value;
        self
    }

    /// 设置密码认证材料。
    pub fn password(mut self, value: impl Into<String>) -> Self {
        self.password = Some(value.into());
        self
    }

    /// 设置私钥认证使用的本地私钥路径。
    pub fn private_key_path(mut self, value: impl Into<String>) -> Self {
        self.private_key_path = Some(value.into());
        self
    }

    /// 设置私钥认证使用的私钥口令。
    pub fn private_key_passphrase(mut self, value: impl Into<String>) -> Self {
        self.private_key_passphrase = Some(value.into());
        self
    }

    /// 设置 TCP 连接超时，单位毫秒。
    pub fn connect_timeout_ms(mut self, value: u32) -> Self {
        self.connect_timeout_ms = value;
        self
    }

    /// 设置 SSH 读写超时，单位毫秒。
    pub fn read_timeout_ms(mut self, value: u32) -> Self {
        self.read_timeout_ms = value;
        self
    }

    /// 校验并返回最终配置。
    pub fn build(self) -> Result<SshConfig> {
        self.validate()?;
        Ok(self)
    }
}

/// `SshConfig` 自身作为链式构建器使用。
pub type SshConfigBuilder = SshConfig;

/// 远程命令执行完成后的退出码和输出。
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SshExecutionResult {
    /// 远程进程退出码。
    pub exit_code: i32,
    /// 远程进程标准输出。
    pub stdout: String,
    /// 远程进程标准错误输出。
    pub stderr: String,
}

impl SshExecutionResult {
    /// 退出码为 `0` 时返回 `true`。
    pub fn is_success(&self) -> bool {
        self.exit_code == 0
    }

    /// 成功时返回标准输出，否则返回包含退出码和标准错误的 `anyhow::Error`。
    pub fn get_output_or_throw(&self) -> Result<&str> {
        if self.is_success() {
            Ok(&self.stdout)
        } else {
            bail!(
                "ssh command failed with exit code {}: {}",
                self.exit_code,
                self.stderr
            )
        }
    }
}

/// 已完成认证的阻塞式 SSH 会话。
pub struct SshSession {
    config: SshConfig,
    session: Session,
}

impl SshSession {
    /// 建立 TCP 连接、完成 SSH 握手并按配置进行认证。
    pub fn connect(config: SshConfig) -> Result<Self> {
        config.validate()?;

        let target = (config.host.as_str(), config.port)
            .to_socket_addrs()?
            .next()
            .with_context(|| {
                format!(
                    "failed to resolve ssh address `{}:{}`",
                    config.host, config.port
                )
            })?;

        let timeout = Duration::from_millis(u64::from(config.connect_timeout_ms));
        let stream = TcpStream::connect_timeout(&target, timeout).with_context(|| {
            format!("tcp connection to `{}:{}` failed", config.host, config.port)
        })?;
        stream.set_read_timeout(Some(Duration::from_millis(u64::from(
            config.read_timeout_ms,
        ))))?;
        stream.set_write_timeout(Some(Duration::from_millis(u64::from(
            config.read_timeout_ms,
        ))))?;

        let mut session = Session::new()?;
        session.set_timeout(config.read_timeout_ms);
        session.set_tcp_stream(stream);
        session.handshake().with_context(|| {
            format!("ssh handshake failed for `{}:{}`", config.host, config.port)
        })?;

        if let Some(private_key_path) = &config.private_key_path {
            let key_path = expand_local_path(Path::new(private_key_path));
            session
                .userauth_pubkey_file(
                    &config.username,
                    None,
                    &key_path,
                    config.private_key_passphrase.as_deref(),
                )
                .with_context(|| {
                    format!(
                        "ssh authentication failed for `{}:{}`",
                        config.host, config.port
                    )
                })?;
        } else if let Some(password) = &config.password {
            session
                .userauth_password(&config.username, password)
                .with_context(|| {
                    format!(
                        "ssh authentication failed for `{}:{}`",
                        config.host, config.port
                    )
                })?;
        }

        if !session.authenticated() {
            bail!(
                "ssh authentication failed for `{}:{}`: server rejected credentials",
                config.host,
                config.port
            );
        }

        Ok(Self { config, session })
    }

    /// 返回当前会话使用的连接配置。
    pub fn config(&self) -> &SshConfig {
        &self.config
    }

    /// 执行远程命令并在命令结束后一次性返回完整输出。
    pub fn execute_sync(&self, command: &str) -> Result<SshExecutionResult> {
        self.run_command(command, |_| {})
    }

    /// 执行远程命令，并在读取 stdout 每一行时调用回调。
    pub fn execute_stream<F>(&self, command: &str, on_stdout_line: F) -> Result<SshExecutionResult>
    where
        F: FnMut(String),
    {
        self.run_command(command, on_stdout_line)
    }

    /// 上传本地文件或目录到远端路径。
    ///
    /// 当远端路径以 `/` 结尾或已经存在为目录时，会把本地文件名/目录名追加到远端目录下。
    pub fn upload_file(
        &self,
        local_path: impl AsRef<Path>,
        remote_path: impl AsRef<str>,
    ) -> Result<()> {
        let local_path = expand_local_path(local_path.as_ref());
        let local_path_str = local_path.display().to_string();
        let local_metadata = fs::metadata(&local_path).with_context(|| {
            format!("ssh file transfer failed: local path does not exist: {local_path_str}")
        })?;

        let sftp = self.session.sftp()?;
        let remote_path = remote_path.as_ref();
        let normalized_remote_path = normalize_remote_path(remote_path);
        let remote_hint_is_directory =
            remote_path.trim_end().ends_with('/') || normalized_remote_path.is_empty();
        let remote_exists_as_directory =
            remote_path_exists_as_directory(&sftp, &normalized_remote_path);
        let treat_remote_as_directory = remote_hint_is_directory || remote_exists_as_directory;

        if local_metadata.is_dir() {
            let target_directory = if treat_remote_as_directory {
                append_remote_path(
                    &normalized_remote_path,
                    file_name_string(&local_path)
                        .with_context(|| {
                            format!(
                                "ssh file transfer failed: unable to determine local directory name for {}",
                                local_path.display()
                            )
                        })?
                        .as_str(),
                )
            } else {
                normalized_remote_path
            };
            upload_directory(&sftp, &local_path, &target_directory)
        } else {
            let remote_file_path = if treat_remote_as_directory {
                append_remote_path(
                    &normalized_remote_path,
                    file_name_string(&local_path)
                        .with_context(|| {
                            format!(
                                "ssh file transfer failed: unable to determine local file name for {}",
                                local_path.display()
                            )
                        })?
                        .as_str(),
                )
            } else {
                normalized_remote_path
            };
            ensure_remote_parent_directories(&sftp, &remote_file_path)?;
            upload_single_file(&sftp, &local_path, &remote_file_path)
        }
    }

    /// 从远端路径下载文件或目录到本地路径。
    pub fn download_file(
        &self,
        remote_path: impl AsRef<str>,
        local_path: impl AsRef<Path>,
    ) -> Result<()> {
        let remote_path = remote_path.as_ref().trim().to_owned();
        if remote_path.is_empty() {
            bail!("ssh file transfer failed: remote_path cannot be blank");
        }

        let local_path = expand_local_path(local_path.as_ref());
        let sftp = self.session.sftp()?;
        let stat = sftp.stat(Path::new(&remote_path)).with_context(|| {
            format!("ssh file transfer failed: failed to stat remote path {remote_path}")
        })?;

        if stat.is_dir() {
            fs::create_dir_all(&local_path)?;
            download_directory(&sftp, Path::new(&remote_path), &local_path)
        } else {
            if let Some(parent) = local_path.parent() {
                fs::create_dir_all(parent)?;
            }
            download_single_file(&sftp, Path::new(&remote_path), &local_path)
        }
    }

    fn run_command<F>(&self, command: &str, mut on_stdout_line: F) -> Result<SshExecutionResult>
    where
        F: FnMut(String),
    {
        let mut channel = self
            .session
            .channel_session()
            .with_context(|| format!("ssh command `{command}` failed"))?;
        channel
            .exec(command)
            .with_context(|| format!("ssh command `{command}` failed"))?;

        let stderr_stream = channel.stderr();
        let stderr_reader = thread::spawn(move || -> io::Result<String> {
            let mut stderr = String::new();
            let mut reader = BufReader::new(stderr_stream);
            reader.read_to_string(&mut stderr)?;
            Ok(stderr)
        });

        let stdout = read_stdout_lines(channel.stream(0), &mut on_stdout_line)?;
        let stderr = stderr_reader.join().map_err(|_| {
            anyhow::anyhow!("ssh command `{command}` failed: stderr reader thread panicked")
        })??;

        channel
            .wait_close()
            .with_context(|| format!("ssh command `{command}` failed"))?;
        let exit_code = channel
            .exit_status()
            .with_context(|| format!("ssh command `{command}` failed"))?;

        Ok(SshExecutionResult {
            exit_code,
            stdout,
            stderr,
        })
    }
}

/// 建立一个已认证的 SSH 会话。
pub fn connect(config: SshConfig) -> Result<SshSession> {
    SshSession::connect(config)
}

/// 建立临时会话并在闭包完成后关闭连接。
pub fn with_session<T, F>(config: SshConfig, block: F) -> Result<T>
where
    F: FnOnce(&SshSession) -> Result<T>,
{
    let session = SshSession::connect(config)?;
    block(&session)
}

/// 建立临时会话并同步执行一条远程命令。
pub fn execute_sync(config: &SshConfig, command: &str) -> Result<SshExecutionResult> {
    with_session(config.clone(), |session| session.execute_sync(command))
}

/// 建立临时会话并流式执行一条远程命令。
pub fn execute_stream<F>(
    config: &SshConfig,
    command: &str,
    on_stdout_line: F,
) -> Result<SshExecutionResult>
where
    F: FnMut(String),
{
    with_session(config.clone(), |session| {
        session.execute_stream(command, on_stdout_line)
    })
}

/// 建立临时会话并上传本地文件或目录。
pub fn upload_file(
    config: &SshConfig,
    local_path: impl AsRef<Path>,
    remote_path: impl AsRef<str>,
) -> Result<()> {
    with_session(config.clone(), |session| {
        session.upload_file(local_path, remote_path)
    })
}

/// 建立临时会话并下载远端文件或目录。
pub fn download_file(
    config: &SshConfig,
    remote_path: impl AsRef<str>,
    local_path: impl AsRef<Path>,
) -> Result<()> {
    with_session(config.clone(), |session| {
        session.download_file(remote_path, local_path)
    })
}

fn read_stdout_lines<F>(stream: ssh2::Stream, on_stdout_line: &mut F) -> Result<String>
where
    F: FnMut(String),
{
    let mut reader = BufReader::new(stream);
    let mut stdout = String::new();
    let mut line = String::new();

    loop {
        line.clear();
        let bytes = reader.read_line(&mut line)?;
        if bytes == 0 {
            break;
        }
        stdout.push_str(&line);
        on_stdout_line(trim_line_ending(&line).to_owned());
    }

    Ok(stdout)
}

fn upload_single_file(sftp: &ssh2::Sftp, local_path: &Path, remote_file_path: &str) -> Result<()> {
    let mut source = File::open(local_path)?;
    let mut target = sftp.create(Path::new(remote_file_path)).with_context(|| {
        format!("ssh file transfer failed: failed to create remote file {remote_file_path}")
    })?;
    io::copy(&mut source, &mut target)?;
    target.flush()?;
    Ok(())
}

fn upload_directory(sftp: &ssh2::Sftp, local_dir: &Path, remote_path: &str) -> Result<()> {
    ensure_remote_directory(sftp, remote_path)?;
    for entry in fs::read_dir(local_dir)? {
        let entry = entry?;
        let path = entry.path();
        let remote_child = append_remote_path(
            remote_path,
            &file_name_string(&path).with_context(|| {
                format!(
                    "ssh file transfer failed: unable to determine file name for {}",
                    path.display()
                )
            })?,
        );
        if entry.file_type()?.is_dir() {
            upload_directory(sftp, &path, &remote_child)?;
        } else {
            upload_single_file(sftp, &path, &remote_child)?;
        }
    }
    Ok(())
}

fn download_single_file(sftp: &ssh2::Sftp, remote_path: &Path, local_path: &Path) -> Result<()> {
    let mut source = sftp.open(remote_path).with_context(|| {
        format!(
            "ssh file transfer failed: failed to open remote file {}",
            remote_path.display()
        )
    })?;
    let mut target = File::create(local_path)?;
    io::copy(&mut source, &mut target)?;
    target.flush()?;
    Ok(())
}

fn download_directory(sftp: &ssh2::Sftp, remote_path: &Path, local_dir: &Path) -> Result<()> {
    for (entry_path, stat) in sftp.readdir(remote_path).with_context(|| {
        format!(
            "ssh file transfer failed: failed to read remote directory {}",
            remote_path.display()
        )
    })? {
        let entry_name = entry_path.file_name().with_context(|| {
            format!(
                "ssh file transfer failed: unable to determine remote entry name for {}",
                entry_path.display()
            )
        })?;
        let local_path = local_dir.join(entry_name);
        if stat.is_dir() {
            fs::create_dir_all(&local_path)?;
            download_directory(sftp, &entry_path, &local_path)?;
        } else {
            if let Some(parent) = local_path.parent() {
                fs::create_dir_all(parent)?;
            }
            download_single_file(sftp, &entry_path, &local_path)?;
        }
    }
    Ok(())
}

fn normalize_remote_path(remote_path: &str) -> String {
    let trimmed = remote_path.trim();
    if trimmed.is_empty() {
        return String::new();
    }
    if trimmed == "/" {
        "/".to_owned()
    } else {
        trimmed.trim_end_matches('/').to_owned()
    }
}

fn append_remote_path(base: &str, child: &str) -> String {
    if child.is_empty() {
        return base.to_owned();
    }
    if base.is_empty() {
        return child.to_owned();
    }
    if base == "/" {
        return format!("/{child}");
    }
    format!("{base}/{child}")
}

fn remote_path_exists_as_directory(sftp: &ssh2::Sftp, remote_path: &str) -> bool {
    if remote_path.is_empty() {
        return false;
    }
    sftp.stat(Path::new(remote_path))
        .map(|stat| stat.is_dir())
        .unwrap_or(false)
}

fn ensure_remote_parent_directories(sftp: &ssh2::Sftp, remote_file_path: &str) -> Result<()> {
    match remote_file_path.rfind('/') {
        Some(0) => ensure_remote_directory(sftp, "/"),
        Some(index) => ensure_remote_directory(sftp, &remote_file_path[..index]),
        None => Ok(()),
    }
}

fn ensure_remote_directory(sftp: &ssh2::Sftp, remote_directory: &str) -> Result<()> {
    let trimmed = remote_directory.trim();
    if trimmed.is_empty() {
        return Ok(());
    }
    if trimmed == "/" {
        return verify_directory_node(sftp, "/".into(), false);
    }

    let normalized = trimmed.trim_end_matches('/');
    let absolute = normalized.starts_with('/');
    let segments = normalized.split('/').filter(|segment| !segment.is_empty());
    let mut current = if absolute {
        "/".to_owned()
    } else {
        String::new()
    };

    for segment in segments {
        current = if current == "/" {
            format!("/{segment}")
        } else if current.is_empty() {
            segment.to_owned()
        } else {
            format!("{current}/{segment}")
        };
        verify_directory_node(sftp, current.clone(), true)?;
    }

    Ok(())
}

fn verify_directory_node(sftp: &ssh2::Sftp, path: String, create_when_missing: bool) -> Result<()> {
    match sftp.stat(Path::new(&path)) {
        Ok(stat) => {
            if stat.is_dir() {
                Ok(())
            } else {
                bail!(
                    "ssh file transfer failed: remote path exists but is not a directory: {path}"
                );
            }
        }
        Err(_error) if create_when_missing => {
            let mkdir_result = sftp.mkdir(Path::new(&path), 0o755);
            if let Err(mkdir_error) = mkdir_result {
                match sftp.stat(Path::new(&path)) {
                    Ok(stat) if stat.is_dir() => return Ok(()),
                    Ok(_) => {
                        bail!(
                            "ssh file transfer failed: remote path exists but is not a directory: {path}"
                        );
                    }
                    Err(_) => {
                        bail!(
                            "ssh file transfer failed: failed to create remote directory {path}: {mkdir_error}"
                        );
                    }
                }
            }
            Ok(())
        }
        Err(error) => {
            bail!("ssh file transfer failed: failed to stat remote directory {path}: {error}");
        }
    }
}

fn expand_local_path(path: &Path) -> PathBuf {
    let Some(raw) = path.to_str() else {
        return path.to_path_buf();
    };

    if raw == "~"
        && let Ok(home) = std::env::var("HOME")
    {
        return PathBuf::from(home);
    }
    if let Some(rest) = raw.strip_prefix("~/")
        && let Ok(home) = std::env::var("HOME")
    {
        return PathBuf::from(home).join(rest);
    }
    path.to_path_buf()
}

fn file_name_string(path: &Path) -> Option<String> {
    path.file_name()
        .and_then(|name| name.to_str())
        .map(ToOwned::to_owned)
}

fn trim_line_ending(value: &str) -> &str {
    value.trim_end_matches(&['\r', '\n'][..])
}
