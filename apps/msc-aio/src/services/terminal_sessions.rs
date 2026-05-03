use std::rc::Rc;

#[cfg(not(target_arch = "wasm32"))]
use std::{
    collections::BTreeMap,
    io::{Read, Write},
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
    thread,
};

use chrono::{DateTime, Utc};
#[cfg(not(target_arch = "wasm32"))]
use once_cell::sync::Lazy;
#[cfg(not(target_arch = "wasm32"))]
use portable_pty::{
    Child, ChildKiller, CommandBuilder, MasterPty, PtySize, PtySystem, native_pty_system,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::LocalBoxFuture;

const DEFAULT_TERMINAL_ROWS: u16 = 32;
const DEFAULT_TERMINAL_COLS: u16 = 120;

#[derive(Clone, Copy, Debug, Default, Eq, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TerminalProfileDto {
    #[default]
    Codex,
    Claude,
    Shell,
}

impl TerminalProfileDto {
    pub const ALL: [Self; 3] = [Self::Codex, Self::Claude, Self::Shell];

    pub fn code(self) -> &'static str {
        match self {
            Self::Codex => "codex",
            Self::Claude => "claude",
            Self::Shell => "shell",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Codex => "Codex CLI",
            Self::Claude => "Claude Code",
            Self::Shell => "Shell",
        }
    }

    fn default_title(self, index: usize) -> String {
        match self {
            Self::Codex => format!("Codex {index}"),
            Self::Claude => format!("Claude {index}"),
            Self::Shell => format!("Shell {index}"),
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TerminalSessionStateDto {
    #[default]
    Running,
    Exited,
    Failed,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct TerminalSessionCreateDto {
    #[serde(default)]
    pub profile: TerminalProfileDto,
    pub cwd: Option<String>,
    pub title: Option<String>,
    pub rows: Option<u16>,
    pub cols: Option<u16>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct TerminalSessionInputDto {
    pub data: String,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct TerminalSessionResizeDto {
    pub rows: u16,
    pub cols: u16,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TerminalSessionSummaryDto {
    pub id: Uuid,
    pub title: String,
    pub profile: TerminalProfileDto,
    pub cwd: String,
    pub command_preview: String,
    pub rows: u16,
    pub cols: u16,
    pub state: TerminalSessionStateDto,
    pub exit_code: Option<i32>,
    pub last_error: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TerminalSessionSnapshotDto {
    pub summary: TerminalSessionSummaryDto,
    pub screen: String,
    pub cursor_row: u16,
    pub cursor_col: u16,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct TerminalSessionListDto {
    pub default_cwd: String,
    pub sessions: Vec<TerminalSessionSummaryDto>,
}

pub trait TerminalSessionsApi: 'static {
    fn list_sessions(&self) -> LocalBoxFuture<'_, Result<TerminalSessionListDto, String>>;
    fn create_session(
        &self,
        input: TerminalSessionCreateDto,
    ) -> LocalBoxFuture<'_, Result<TerminalSessionSnapshotDto, String>>;
    fn get_snapshot(
        &self,
        id: String,
    ) -> LocalBoxFuture<'_, Result<TerminalSessionSnapshotDto, String>>;
    fn send_input(
        &self,
        id: String,
        input: TerminalSessionInputDto,
    ) -> LocalBoxFuture<'_, Result<TerminalSessionSnapshotDto, String>>;
    fn resize_session(
        &self,
        id: String,
        input: TerminalSessionResizeDto,
    ) -> LocalBoxFuture<'_, Result<TerminalSessionSnapshotDto, String>>;
    fn close_session(&self, id: String) -> LocalBoxFuture<'_, Result<(), String>>;
}

pub type SharedTerminalSessionsApi = Rc<dyn TerminalSessionsApi>;

pub fn default_terminal_sessions_api() -> SharedTerminalSessionsApi {
    #[cfg(target_arch = "wasm32")]
    {
        Rc::new(BrowserTerminalSessionsApi)
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        Rc::new(EmbeddedTerminalSessionsApi)
    }
}

#[cfg(target_arch = "wasm32")]
struct BrowserTerminalSessionsApi;

#[cfg(target_arch = "wasm32")]
impl TerminalSessionsApi for BrowserTerminalSessionsApi {
    fn list_sessions(&self) -> LocalBoxFuture<'_, Result<TerminalSessionListDto, String>> {
        Box::pin(async move { super::browser_http::get_json("/api/admin/terminal/sessions").await })
    }

    fn create_session(
        &self,
        input: TerminalSessionCreateDto,
    ) -> LocalBoxFuture<'_, Result<TerminalSessionSnapshotDto, String>> {
        Box::pin(async move {
            super::browser_http::post_json("/api/admin/terminal/sessions", &input).await
        })
    }

    fn get_snapshot(
        &self,
        id: String,
    ) -> LocalBoxFuture<'_, Result<TerminalSessionSnapshotDto, String>> {
        Box::pin(async move {
            super::browser_http::get_json(&format!("/api/admin/terminal/sessions/{id}")).await
        })
    }

    fn send_input(
        &self,
        id: String,
        input: TerminalSessionInputDto,
    ) -> LocalBoxFuture<'_, Result<TerminalSessionSnapshotDto, String>> {
        Box::pin(async move {
            super::browser_http::post_json(
                &format!("/api/admin/terminal/sessions/{id}/input"),
                &input,
            )
            .await
        })
    }

    fn resize_session(
        &self,
        id: String,
        input: TerminalSessionResizeDto,
    ) -> LocalBoxFuture<'_, Result<TerminalSessionSnapshotDto, String>> {
        Box::pin(async move {
            super::browser_http::post_json(
                &format!("/api/admin/terminal/sessions/{id}/resize"),
                &input,
            )
            .await
        })
    }

    fn close_session(&self, id: String) -> LocalBoxFuture<'_, Result<(), String>> {
        Box::pin(async move {
            super::browser_http::delete_empty(&format!("/api/admin/terminal/sessions/{id}")).await
        })
    }
}

#[cfg(not(target_arch = "wasm32"))]
struct EmbeddedTerminalSessionsApi;

#[cfg(not(target_arch = "wasm32"))]
impl TerminalSessionsApi for EmbeddedTerminalSessionsApi {
    fn list_sessions(&self) -> LocalBoxFuture<'_, Result<TerminalSessionListDto, String>> {
        Box::pin(async move { list_terminal_sessions_on_server() })
    }

    fn create_session(
        &self,
        input: TerminalSessionCreateDto,
    ) -> LocalBoxFuture<'_, Result<TerminalSessionSnapshotDto, String>> {
        Box::pin(async move { create_terminal_session_on_server(input) })
    }

    fn get_snapshot(
        &self,
        id: String,
    ) -> LocalBoxFuture<'_, Result<TerminalSessionSnapshotDto, String>> {
        Box::pin(async move { get_terminal_session_snapshot_on_server(&id) })
    }

    fn send_input(
        &self,
        id: String,
        input: TerminalSessionInputDto,
    ) -> LocalBoxFuture<'_, Result<TerminalSessionSnapshotDto, String>> {
        Box::pin(async move { send_terminal_input_on_server(&id, input) })
    }

    fn resize_session(
        &self,
        id: String,
        input: TerminalSessionResizeDto,
    ) -> LocalBoxFuture<'_, Result<TerminalSessionSnapshotDto, String>> {
        Box::pin(async move { resize_terminal_session_on_server(&id, input) })
    }

    fn close_session(&self, id: String) -> LocalBoxFuture<'_, Result<(), String>> {
        Box::pin(async move { close_terminal_session_on_server(&id) })
    }
}

#[cfg(not(target_arch = "wasm32"))]
static TERMINAL_STORE: Lazy<Mutex<TerminalSessionStore>> =
    Lazy::new(|| Mutex::new(TerminalSessionStore::from_env()));

#[cfg(not(target_arch = "wasm32"))]
struct TerminalSessionStore {
    default_cwd: PathBuf,
    sessions: BTreeMap<Uuid, ManagedTerminalSession>,
}

#[cfg(not(target_arch = "wasm32"))]
struct ManagedTerminalSession {
    id: Uuid,
    title: String,
    profile: TerminalProfileDto,
    cwd: PathBuf,
    command_preview: String,
    rows: u16,
    cols: u16,
    created_at: DateTime<Utc>,
    runtime: Arc<TerminalRuntime>,
    master: Box<dyn MasterPty + Send>,
    writer: Box<dyn Write + Send>,
    killer: Box<dyn ChildKiller + Send + Sync>,
}

#[cfg(not(target_arch = "wasm32"))]
struct TerminalRuntime {
    parser: Mutex<vt100::Parser>,
    state: Mutex<TerminalSessionStateDto>,
    exit_code: Mutex<Option<i32>>,
    last_error: Mutex<Option<String>>,
    updated_at: Mutex<DateTime<Utc>>,
}

#[cfg(not(target_arch = "wasm32"))]
impl TerminalRuntime {
    fn new(rows: u16, cols: u16) -> Self {
        Self {
            parser: Mutex::new(vt100::Parser::new(rows, cols, 8_000)),
            state: Mutex::new(TerminalSessionStateDto::Running),
            exit_code: Mutex::new(None),
            last_error: Mutex::new(None),
            updated_at: Mutex::new(Utc::now()),
        }
    }

    fn process_output(&self, bytes: &[u8]) {
        if let Ok(mut parser) = self.parser.lock() {
            parser.process(bytes);
        }
        self.touch();
    }

    fn resize(&self, rows: u16, cols: u16) {
        if let Ok(mut parser) = self.parser.lock() {
            parser.screen_mut().set_size(rows, cols);
        }
        self.touch();
    }

    fn mark_exited(&self, exit_code: Option<i32>) {
        if let Ok(mut state) = self.state.lock() {
            *state = TerminalSessionStateDto::Exited;
        }
        if let Ok(mut code) = self.exit_code.lock() {
            *code = exit_code;
        }
        self.touch();
    }

    fn record_error(&self, message: String, state: Option<TerminalSessionStateDto>) {
        if let Ok(mut last_error) = self.last_error.lock() {
            *last_error = Some(message);
        }
        if let Some(next_state) = state {
            if let Ok(mut current_state) = self.state.lock() {
                *current_state = next_state;
            }
        }
        self.touch();
    }

    fn touch(&self) {
        if let Ok(mut updated_at) = self.updated_at.lock() {
            *updated_at = Utc::now();
        }
    }

    fn snapshot(
        &self,
        id: Uuid,
        title: &str,
        profile: TerminalProfileDto,
        cwd: &Path,
        command_preview: &str,
        rows: u16,
        cols: u16,
        created_at: DateTime<Utc>,
    ) -> TerminalSessionSnapshotDto {
        let state = self.state.lock().map(|value| *value).unwrap_or_default();
        let exit_code = self.exit_code.lock().map(|value| *value).unwrap_or(None);
        let last_error = self
            .last_error
            .lock()
            .ok()
            .and_then(|value| value.as_ref().cloned());
        let updated_at = self
            .updated_at
            .lock()
            .map(|value| *value)
            .unwrap_or(created_at);
        let (screen, cursor_row, cursor_col) = if let Ok(parser) = self.parser.lock() {
            let screen = parser.screen();
            let (cursor_row, cursor_col) = screen.cursor_position();
            (screen.contents(), cursor_row, cursor_col)
        } else {
            (String::new(), 0, 0)
        };

        TerminalSessionSnapshotDto {
            summary: TerminalSessionSummaryDto {
                id,
                title: title.to_string(),
                profile,
                cwd: cwd.display().to_string(),
                command_preview: command_preview.to_string(),
                rows,
                cols,
                state,
                exit_code,
                last_error,
                created_at,
                updated_at,
            },
            screen,
            cursor_row,
            cursor_col,
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl ManagedTerminalSession {
    fn snapshot(&self) -> TerminalSessionSnapshotDto {
        self.runtime.snapshot(
            self.id,
            &self.title,
            self.profile,
            &self.cwd,
            &self.command_preview,
            self.rows,
            self.cols,
            self.created_at,
        )
    }

    fn summary(&self) -> TerminalSessionSummaryDto {
        self.snapshot().summary
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl TerminalSessionStore {
    fn from_env() -> Self {
        Self {
            default_cwd: resolve_default_cwd(),
            sessions: BTreeMap::new(),
        }
    }

    fn list(&self) -> TerminalSessionListDto {
        let mut sessions = self
            .sessions
            .values()
            .map(ManagedTerminalSession::summary)
            .collect::<Vec<_>>();
        sessions.sort_by(|left, right| {
            right
                .updated_at
                .cmp(&left.updated_at)
                .then(left.title.cmp(&right.title))
        });
        TerminalSessionListDto {
            default_cwd: self.default_cwd.display().to_string(),
            sessions,
        }
    }

    fn create(
        &mut self,
        input: TerminalSessionCreateDto,
    ) -> Result<TerminalSessionSnapshotDto, String> {
        let profile = input.profile;
        let cwd = resolve_session_cwd(input.cwd.as_deref(), &self.default_cwd)?;
        let rows = normalize_terminal_rows(input.rows.unwrap_or(DEFAULT_TERMINAL_ROWS));
        let cols = normalize_terminal_cols(input.cols.unwrap_or(DEFAULT_TERMINAL_COLS));
        let launch = build_launch_spec(profile, &cwd)?;
        let title = input
            .title
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(ToOwned::to_owned)
            .unwrap_or_else(|| {
                let index = self
                    .sessions
                    .values()
                    .filter(|session| session.profile == profile)
                    .count()
                    + 1;
                profile.default_title(index)
            });
        let created_at = Utc::now();
        let id = Uuid::new_v4();
        let runtime = Arc::new(TerminalRuntime::new(rows, cols));

        let pty_system = native_pty_system();
        let mut pair = pty_system
            .openpty(PtySize {
                rows,
                cols,
                pixel_width: 0,
                pixel_height: 0,
            })
            .map_err(|err| format!("创建终端失败：{err}"))?;
        let mut command = launch.command;
        command.cwd(cwd.as_os_str());
        command.env("TERM", "xterm-256color");
        command.env("COLORTERM", "truecolor");
        command.env("TERM_PROGRAM", "msc-aio");

        let child = pair
            .slave
            .spawn_command(command)
            .map_err(|err| format!("启动 {} 失败：{err}", profile.label()))?;
        let killer = child.clone_killer();
        let reader = pair
            .master
            .try_clone_reader()
            .map_err(|err| format!("创建终端读流失败：{err}"))?;
        let writer = pair
            .master
            .take_writer()
            .map_err(|err| format!("创建终端写流失败：{err}"))?;
        spawn_reader_thread(reader, runtime.clone());
        spawn_wait_thread(child, runtime.clone());

        let session = ManagedTerminalSession {
            id,
            title,
            profile,
            cwd,
            command_preview: launch.command_preview,
            rows,
            cols,
            created_at,
            runtime,
            master: pair.master,
            writer,
            killer,
        };
        let snapshot = session.snapshot();
        self.sessions.insert(id, session);
        Ok(snapshot)
    }

    fn snapshot(&self, id: Uuid) -> Result<TerminalSessionSnapshotDto, String> {
        self.sessions
            .get(&id)
            .map(ManagedTerminalSession::snapshot)
            .ok_or_else(|| "终端会话不存在".to_string())
    }

    fn input(
        &mut self,
        id: Uuid,
        input: TerminalSessionInputDto,
    ) -> Result<TerminalSessionSnapshotDto, String> {
        let session = self
            .sessions
            .get_mut(&id)
            .ok_or_else(|| "终端会话不存在".to_string())?;
        if input.data.is_empty() {
            return session
                .runtime
                .state
                .lock()
                .map(|_| session.snapshot())
                .map_err(|_| "终端状态已损坏".to_string());
        }
        session
            .writer
            .write_all(input.data.as_bytes())
            .map_err(|err| format!("写入终端失败：{err}"))?;
        session
            .writer
            .flush()
            .map_err(|err| format!("刷新终端输入失败：{err}"))?;
        session.runtime.touch();
        Ok(session.snapshot())
    }

    fn resize(
        &mut self,
        id: Uuid,
        input: TerminalSessionResizeDto,
    ) -> Result<TerminalSessionSnapshotDto, String> {
        let session = self
            .sessions
            .get_mut(&id)
            .ok_or_else(|| "终端会话不存在".to_string())?;
        let rows = normalize_terminal_rows(input.rows);
        let cols = normalize_terminal_cols(input.cols);
        session
            .master
            .resize(PtySize {
                rows,
                cols,
                pixel_width: 0,
                pixel_height: 0,
            })
            .map_err(|err| format!("调整终端尺寸失败：{err}"))?;
        session.rows = rows;
        session.cols = cols;
        session.runtime.resize(rows, cols);
        Ok(session.snapshot())
    }

    fn close(&mut self, id: Uuid) -> Result<(), String> {
        let mut session = self
            .sessions
            .remove(&id)
            .ok_or_else(|| "终端会话不存在".to_string())?;
        let state = session
            .runtime
            .state
            .lock()
            .map(|value| *value)
            .unwrap_or_default();
        if state == TerminalSessionStateDto::Running {
            session
                .killer
                .kill()
                .map_err(|err| format!("终止终端会话失败：{err}"))?;
        }
        Ok(())
    }
}

#[cfg(not(target_arch = "wasm32"))]
struct TerminalLaunchSpec {
    command: CommandBuilder,
    command_preview: String,
}

#[cfg(not(target_arch = "wasm32"))]
pub fn list_terminal_sessions_on_server() -> Result<TerminalSessionListDto, String> {
    let store = lock_terminal_store()?;
    Ok(store.list())
}

#[cfg(not(target_arch = "wasm32"))]
pub fn create_terminal_session_on_server(
    input: TerminalSessionCreateDto,
) -> Result<TerminalSessionSnapshotDto, String> {
    let mut store = lock_terminal_store()?;
    store.create(input)
}

#[cfg(not(target_arch = "wasm32"))]
pub fn get_terminal_session_snapshot_on_server(
    id: &str,
) -> Result<TerminalSessionSnapshotDto, String> {
    let session_id = parse_session_id(id)?;
    let store = lock_terminal_store()?;
    store.snapshot(session_id)
}

#[cfg(not(target_arch = "wasm32"))]
pub fn send_terminal_input_on_server(
    id: &str,
    input: TerminalSessionInputDto,
) -> Result<TerminalSessionSnapshotDto, String> {
    let session_id = parse_session_id(id)?;
    let mut store = lock_terminal_store()?;
    store.input(session_id, input)
}

#[cfg(not(target_arch = "wasm32"))]
pub fn resize_terminal_session_on_server(
    id: &str,
    input: TerminalSessionResizeDto,
) -> Result<TerminalSessionSnapshotDto, String> {
    let session_id = parse_session_id(id)?;
    let mut store = lock_terminal_store()?;
    store.resize(session_id, input)
}

#[cfg(not(target_arch = "wasm32"))]
pub fn close_terminal_session_on_server(id: &str) -> Result<(), String> {
    let session_id = parse_session_id(id)?;
    let mut store = lock_terminal_store()?;
    store.close(session_id)
}

#[cfg(not(target_arch = "wasm32"))]
fn lock_terminal_store() -> Result<std::sync::MutexGuard<'static, TerminalSessionStore>, String> {
    TERMINAL_STORE
        .lock()
        .map_err(|_| "终端会话状态已损坏".to_string())
}

#[cfg(not(target_arch = "wasm32"))]
fn parse_session_id(id: &str) -> Result<Uuid, String> {
    Uuid::parse_str(id).map_err(|_| "终端会话标识非法".to_string())
}

#[cfg(not(target_arch = "wasm32"))]
fn resolve_default_cwd() -> PathBuf {
    let env_value = std::env::var("MSC_AIO_TERMINAL_DEFAULT_CWD")
        .ok()
        .filter(|value| !value.trim().is_empty())
        .map(PathBuf::from);
    let fallback = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let candidate = env_value.unwrap_or(fallback);
    if candidate.is_dir() {
        candidate
    } else {
        PathBuf::from(".")
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn resolve_session_cwd(input: Option<&str>, default_cwd: &Path) -> Result<PathBuf, String> {
    let candidate = input
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
        .unwrap_or_else(|| default_cwd.to_path_buf());
    if !candidate.exists() {
        return Err(format!("工作目录不存在：{}", candidate.display()));
    }
    if !candidate.is_dir() {
        return Err(format!("工作目录不是文件夹：{}", candidate.display()));
    }
    Ok(candidate)
}

#[cfg(not(target_arch = "wasm32"))]
fn build_launch_spec(
    profile: TerminalProfileDto,
    cwd: &Path,
) -> Result<TerminalLaunchSpec, String> {
    let shell = resolve_shell_path();
    match profile {
        TerminalProfileDto::Codex => build_login_shell_command(
            &shell,
            std::env::var("MSC_AIO_TERMINAL_CODEX_CMD")
                .ok()
                .filter(|value| !value.trim().is_empty())
                .unwrap_or_else(|| "codex".to_string()),
            cwd,
        ),
        TerminalProfileDto::Claude => build_login_shell_command(
            &shell,
            std::env::var("MSC_AIO_TERMINAL_CLAUDE_CMD")
                .ok()
                .filter(|value| !value.trim().is_empty())
                .unwrap_or_else(|| "claude".to_string()),
            cwd,
        ),
        TerminalProfileDto::Shell => {
            let mut command = CommandBuilder::new(shell.as_str());
            command.arg("-il");
            Ok(TerminalLaunchSpec {
                command,
                command_preview: format!("{} -il", shell),
            })
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn build_login_shell_command(
    shell: &str,
    command_text: String,
    cwd: &Path,
) -> Result<TerminalLaunchSpec, String> {
    let preview = command_text.trim().to_string();
    if preview.is_empty() {
        return Err("终端启动命令不能为空".to_string());
    }
    let mut command = CommandBuilder::new(shell);
    command.arg("-lc");
    command.arg(command_text);
    command.cwd(cwd.as_os_str());
    Ok(TerminalLaunchSpec {
        command,
        command_preview: preview,
    })
}

#[cfg(not(target_arch = "wasm32"))]
fn resolve_shell_path() -> String {
    std::env::var("SHELL")
        .ok()
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| "/bin/zsh".to_string())
}

#[cfg(not(target_arch = "wasm32"))]
fn spawn_reader_thread(mut reader: Box<dyn Read + Send>, runtime: Arc<TerminalRuntime>) {
    thread::spawn(move || {
        let mut buffer = [0_u8; 8_192];
        loop {
            match reader.read(&mut buffer) {
                Ok(0) => break,
                Ok(size) => runtime.process_output(&buffer[..size]),
                Err(err) => {
                    runtime.record_error(format!("终端输出读取失败：{err}"), None);
                    break;
                }
            }
        }
    });
}

#[cfg(not(target_arch = "wasm32"))]
fn spawn_wait_thread(mut child: Box<dyn Child + Send + Sync>, runtime: Arc<TerminalRuntime>) {
    thread::spawn(move || match child.wait() {
        Ok(status) => runtime.mark_exited(Some(i32::try_from(status.exit_code()).unwrap_or(1))),
        Err(err) => runtime.record_error(
            format!("终端进程等待失败：{err}"),
            Some(TerminalSessionStateDto::Failed),
        ),
    });
}

#[cfg(not(target_arch = "wasm32"))]
fn normalize_terminal_rows(rows: u16) -> u16 {
    rows.clamp(12, 80)
}

#[cfg(not(target_arch = "wasm32"))]
fn normalize_terminal_cols(cols: u16) -> u16 {
    cols.clamp(40, 240)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn terminal_profile_titles_are_stable() {
        assert_eq!(TerminalProfileDto::Codex.default_title(2), "Codex 2");
        assert_eq!(TerminalProfileDto::Claude.default_title(1), "Claude 1");
        assert_eq!(TerminalProfileDto::Shell.default_title(3), "Shell 3");
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn terminal_size_is_clamped() {
        assert_eq!(normalize_terminal_rows(2), 12);
        assert_eq!(normalize_terminal_rows(99), 80);
        assert_eq!(normalize_terminal_cols(10), 40);
        assert_eq!(normalize_terminal_cols(999), 240);
    }
}
