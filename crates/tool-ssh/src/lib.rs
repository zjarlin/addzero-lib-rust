#![forbid(unsafe_code)]

use ssh2::Session;
use std::fs::{self, File};
use std::io::{self, BufRead, BufReader, Read, Write};
use std::net::{TcpStream, ToSocketAddrs};
use std::path::{Path, PathBuf};
use std::thread;
use std::time::Duration;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum SshError {
    #[error("invalid ssh configuration: {0}")]
    InvalidConfig(String),
    #[error("failed to resolve ssh address `{host}:{port}`")]
    AddressResolution { host: String, port: u16 },
    #[error("tcp connection to `{host}:{port}` failed: {source}")]
    TcpConnect {
        host: String,
        port: u16,
        #[source]
        source: io::Error,
    },
    #[error("ssh handshake failed for `{host}:{port}`: {source}")]
    Handshake {
        host: String,
        port: u16,
        #[source]
        source: ssh2::Error,
    },
    #[error("ssh authentication failed for `{host}:{port}`: {message}")]
    Authentication {
        host: String,
        port: u16,
        message: String,
    },
    #[error("ssh command failed with exit code {exit_code}: {stderr}")]
    CommandFailed { exit_code: i32, stderr: String },
    #[error("ssh command `{command}` failed: {message}")]
    Execution { command: String, message: String },
    #[error("ssh file transfer failed: {message}")]
    FileTransfer { message: String },
    #[error("ssh library error: {0}")]
    Ssh(#[from] ssh2::Error),
    #[error("I/O error: {0}")]
    Io(#[from] io::Error),
}

pub type SshResult<T> = Result<T, SshError>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SshConfig {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: Option<String>,
    pub private_key_path: Option<String>,
    pub private_key_passphrase: Option<String>,
    pub connect_timeout_ms: u32,
    pub read_timeout_ms: u32,
}

impl SshConfig {
    pub fn builder(host: impl Into<String>, username: impl Into<String>) -> SshConfigBuilder {
        SshConfigBuilder {
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

    pub fn validate(&self) -> SshResult<()> {
        if self.host.trim().is_empty() {
            return Err(SshError::InvalidConfig("host cannot be blank".to_owned()));
        }
        if self.username.trim().is_empty() {
            return Err(SshError::InvalidConfig(
                "username cannot be blank".to_owned(),
            ));
        }
        if self.port == 0 {
            return Err(SshError::InvalidConfig(
                "port must be greater than zero".to_owned(),
            ));
        }
        if self.password.is_none() && self.private_key_path.is_none() {
            return Err(SshError::InvalidConfig(
                "password or private_key_path is required".to_owned(),
            ));
        }
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct SshConfigBuilder {
    host: String,
    port: u16,
    username: String,
    password: Option<String>,
    private_key_path: Option<String>,
    private_key_passphrase: Option<String>,
    connect_timeout_ms: u32,
    read_timeout_ms: u32,
}

impl SshConfigBuilder {
    pub fn port(mut self, value: u16) -> Self {
        self.port = value;
        self
    }

    pub fn password(mut self, value: impl Into<String>) -> Self {
        self.password = Some(value.into());
        self
    }

    pub fn private_key_path(mut self, value: impl Into<String>) -> Self {
        self.private_key_path = Some(value.into());
        self
    }

    pub fn private_key_passphrase(mut self, value: impl Into<String>) -> Self {
        self.private_key_passphrase = Some(value.into());
        self
    }

    pub fn connect_timeout_ms(mut self, value: u32) -> Self {
        self.connect_timeout_ms = value;
        self
    }

    pub fn read_timeout_ms(mut self, value: u32) -> Self {
        self.read_timeout_ms = value;
        self
    }

    pub fn build(self) -> SshResult<SshConfig> {
        let config = SshConfig {
            host: self.host,
            port: self.port,
            username: self.username,
            password: self.password,
            private_key_path: self.private_key_path,
            private_key_passphrase: self.private_key_passphrase,
            connect_timeout_ms: self.connect_timeout_ms,
            read_timeout_ms: self.read_timeout_ms,
        };
        config.validate()?;
        Ok(config)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SshExecutionResult {
    pub exit_code: i32,
    pub stdout: String,
    pub stderr: String,
}

impl SshExecutionResult {
    pub fn is_success(&self) -> bool {
        self.exit_code == 0
    }

    pub fn get_output_or_throw(&self) -> SshResult<&str> {
        if self.is_success() {
            Ok(&self.stdout)
        } else {
            Err(SshError::CommandFailed {
                exit_code: self.exit_code,
                stderr: self.stderr.clone(),
            })
        }
    }
}

pub struct SshSession {
    config: SshConfig,
    session: Session,
}

impl SshSession {
    pub fn connect(config: SshConfig) -> SshResult<Self> {
        config.validate()?;

        let target = (config.host.as_str(), config.port)
            .to_socket_addrs()?
            .next()
            .ok_or_else(|| SshError::AddressResolution {
                host: config.host.clone(),
                port: config.port,
            })?;

        let timeout = Duration::from_millis(u64::from(config.connect_timeout_ms));
        let stream = TcpStream::connect_timeout(&target, timeout).map_err(|source| {
            SshError::TcpConnect {
                host: config.host.clone(),
                port: config.port,
                source,
            }
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
        session.handshake().map_err(|source| SshError::Handshake {
            host: config.host.clone(),
            port: config.port,
            source,
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
                .map_err(|error| SshError::Authentication {
                    host: config.host.clone(),
                    port: config.port,
                    message: error.to_string(),
                })?;
        } else if let Some(password) = &config.password {
            session
                .userauth_password(&config.username, password)
                .map_err(|error| SshError::Authentication {
                    host: config.host.clone(),
                    port: config.port,
                    message: error.to_string(),
                })?;
        }

        if !session.authenticated() {
            return Err(SshError::Authentication {
                host: config.host.clone(),
                port: config.port,
                message: "server rejected credentials".to_owned(),
            });
        }

        Ok(Self { config, session })
    }

    pub fn config(&self) -> &SshConfig {
        &self.config
    }

    pub fn execute_sync(&self, command: &str) -> SshResult<SshExecutionResult> {
        self.run_command(command, |_| {})
    }

    pub fn execute_stream<F>(
        &self,
        command: &str,
        on_stdout_line: F,
    ) -> SshResult<SshExecutionResult>
    where
        F: FnMut(String),
    {
        self.run_command(command, on_stdout_line)
    }

    pub fn upload_file(
        &self,
        local_path: impl AsRef<Path>,
        remote_path: impl AsRef<str>,
    ) -> SshResult<()> {
        let local_path = expand_local_path(local_path.as_ref());
        let local_path_str = local_path.display().to_string();
        let local_metadata =
            fs::metadata(&local_path).map_err(|source| SshError::FileTransfer {
                message: format!("local path does not exist: {local_path_str} ({source})"),
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
                        .ok_or_else(|| SshError::FileTransfer {
                            message: format!(
                                "unable to determine local directory name for {}",
                                local_path.display()
                            ),
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
                        .ok_or_else(|| SshError::FileTransfer {
                            message: format!(
                                "unable to determine local file name for {}",
                                local_path.display()
                            ),
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

    pub fn download_file(
        &self,
        remote_path: impl AsRef<str>,
        local_path: impl AsRef<Path>,
    ) -> SshResult<()> {
        let remote_path = remote_path.as_ref().trim().to_owned();
        if remote_path.is_empty() {
            return Err(SshError::FileTransfer {
                message: "remote_path cannot be blank".to_owned(),
            });
        }

        let local_path = expand_local_path(local_path.as_ref());
        let sftp = self.session.sftp()?;
        let stat = sftp
            .stat(Path::new(&remote_path))
            .map_err(|error| SshError::FileTransfer {
                message: format!("failed to stat remote path {}: {error}", remote_path),
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

    fn run_command<F>(&self, command: &str, mut on_stdout_line: F) -> SshResult<SshExecutionResult>
    where
        F: FnMut(String),
    {
        let mut channel = self
            .session
            .channel_session()
            .map_err(|error| SshError::Execution {
                command: command.to_owned(),
                message: error.to_string(),
            })?;
        channel.exec(command).map_err(|error| SshError::Execution {
            command: command.to_owned(),
            message: error.to_string(),
        })?;

        let stderr_stream = channel.stderr();
        let stderr_reader = thread::spawn(move || -> io::Result<String> {
            let mut stderr = String::new();
            let mut reader = BufReader::new(stderr_stream);
            reader.read_to_string(&mut stderr)?;
            Ok(stderr)
        });

        let stdout = read_stdout_lines(channel.stream(0), &mut on_stdout_line)?;
        let stderr = stderr_reader.join().map_err(|_| SshError::Execution {
            command: command.to_owned(),
            message: "stderr reader thread panicked".to_owned(),
        })??;

        channel.wait_close().map_err(|error| SshError::Execution {
            command: command.to_owned(),
            message: error.to_string(),
        })?;
        let exit_code = channel.exit_status().map_err(|error| SshError::Execution {
            command: command.to_owned(),
            message: error.to_string(),
        })?;

        Ok(SshExecutionResult {
            exit_code,
            stdout,
            stderr,
        })
    }
}

pub fn connect(config: SshConfig) -> SshResult<SshSession> {
    SshSession::connect(config)
}

pub fn with_session<T, F>(config: SshConfig, block: F) -> SshResult<T>
where
    F: FnOnce(&SshSession) -> SshResult<T>,
{
    let session = SshSession::connect(config)?;
    block(&session)
}

pub fn execute_sync(config: &SshConfig, command: &str) -> SshResult<SshExecutionResult> {
    with_session(config.clone(), |session| session.execute_sync(command))
}

pub fn execute_stream<F>(
    config: &SshConfig,
    command: &str,
    on_stdout_line: F,
) -> SshResult<SshExecutionResult>
where
    F: FnMut(String),
{
    with_session(config.clone(), |session| {
        session.execute_stream(command, on_stdout_line)
    })
}

pub fn upload_file(
    config: &SshConfig,
    local_path: impl AsRef<Path>,
    remote_path: impl AsRef<str>,
) -> SshResult<()> {
    with_session(config.clone(), |session| {
        session.upload_file(local_path, remote_path)
    })
}

pub fn download_file(
    config: &SshConfig,
    remote_path: impl AsRef<str>,
    local_path: impl AsRef<Path>,
) -> SshResult<()> {
    with_session(config.clone(), |session| {
        session.download_file(remote_path, local_path)
    })
}

fn read_stdout_lines<F>(stream: ssh2::Stream, on_stdout_line: &mut F) -> SshResult<String>
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

fn upload_single_file(
    sftp: &ssh2::Sftp,
    local_path: &Path,
    remote_file_path: &str,
) -> SshResult<()> {
    let mut source = File::open(local_path)?;
    let mut target =
        sftp.create(Path::new(remote_file_path))
            .map_err(|error| SshError::FileTransfer {
                message: format!("failed to create remote file {}: {error}", remote_file_path),
            })?;
    io::copy(&mut source, &mut target)?;
    target.flush()?;
    Ok(())
}

fn upload_directory(sftp: &ssh2::Sftp, local_dir: &Path, remote_path: &str) -> SshResult<()> {
    ensure_remote_directory(sftp, remote_path)?;
    for entry in fs::read_dir(local_dir)? {
        let entry = entry?;
        let path = entry.path();
        let remote_child = append_remote_path(
            remote_path,
            &file_name_string(&path).ok_or_else(|| SshError::FileTransfer {
                message: format!("unable to determine file name for {}", path.display()),
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

fn download_single_file(sftp: &ssh2::Sftp, remote_path: &Path, local_path: &Path) -> SshResult<()> {
    let mut source = sftp
        .open(remote_path)
        .map_err(|error| SshError::FileTransfer {
            message: format!(
                "failed to open remote file {}: {error}",
                remote_path.display()
            ),
        })?;
    let mut target = File::create(local_path)?;
    io::copy(&mut source, &mut target)?;
    target.flush()?;
    Ok(())
}

fn download_directory(sftp: &ssh2::Sftp, remote_path: &Path, local_dir: &Path) -> SshResult<()> {
    for (entry_path, stat) in sftp
        .readdir(remote_path)
        .map_err(|error| SshError::FileTransfer {
            message: format!(
                "failed to read remote directory {}: {error}",
                remote_path.display()
            ),
        })?
    {
        let entry_name = entry_path
            .file_name()
            .ok_or_else(|| SshError::FileTransfer {
                message: format!(
                    "unable to determine remote entry name for {}",
                    entry_path.display()
                ),
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

fn ensure_remote_parent_directories(sftp: &ssh2::Sftp, remote_file_path: &str) -> SshResult<()> {
    match remote_file_path.rfind('/') {
        Some(0) => ensure_remote_directory(sftp, "/"),
        Some(index) => ensure_remote_directory(sftp, &remote_file_path[..index]),
        None => Ok(()),
    }
}

fn ensure_remote_directory(sftp: &ssh2::Sftp, remote_directory: &str) -> SshResult<()> {
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

fn verify_directory_node(
    sftp: &ssh2::Sftp,
    path: String,
    create_when_missing: bool,
) -> SshResult<()> {
    match sftp.stat(Path::new(&path)) {
        Ok(stat) => {
            if stat.is_dir() {
                Ok(())
            } else {
                Err(SshError::FileTransfer {
                    message: format!("remote path exists but is not a directory: {path}"),
                })
            }
        }
        Err(_error) if create_when_missing => {
            let mkdir_result = sftp.mkdir(Path::new(&path), 0o755);
            if let Err(mkdir_error) = mkdir_result {
                match sftp.stat(Path::new(&path)) {
                    Ok(stat) if stat.is_dir() => return Ok(()),
                    Ok(_) => {
                        return Err(SshError::FileTransfer {
                            message: format!("remote path exists but is not a directory: {path}"),
                        });
                    }
                    Err(_) => {
                        return Err(SshError::FileTransfer {
                            message: format!(
                                "failed to create remote directory {path}: {mkdir_error}"
                            ),
                        });
                    }
                }
            }
            Ok(())
        }
        Err(error) => Err(SshError::FileTransfer {
            message: format!("failed to stat remote directory {path}: {error}"),
        }),
    }
}

fn expand_local_path(path: &Path) -> PathBuf {
    let Some(raw) = path.to_str() else {
        return path.to_path_buf();
    };

    if raw == "~" {
        if let Ok(home) = std::env::var("HOME") {
            return PathBuf::from(home);
        }
    }
    if let Some(rest) = raw.strip_prefix("~/") {
        if let Ok(home) = std::env::var("HOME") {
            return PathBuf::from(home).join(rest);
        }
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
