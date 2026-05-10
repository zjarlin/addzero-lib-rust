use std::fmt::Write as _;

use az_cli_market_contract::CliMarketCatalog;

use crate::services::asset_graph::{AssetGraphDto, default_asset_graph_api};
use crate::services::cli_market::default_cli_market_api;
use crate::services::cloudflare_tunnel::{
    CloudflareTunnelStatusDto, cloudflare_tunnel_status_on_server,
};
use crate::services::desktop_bootstrap::{
    BootstrapPlatformSetupDto, BootstrapStatusDto, save_platform_setup_on_server,
};
use crate::services::knowledge_entries::{
    KnowledgeEntryDeleteDto, KnowledgeEntryUpsertDto, KnowledgeNoteDto,
    default_knowledge_entries_api,
};
use crate::services::knowledge_graph::{
    KnowledgeExceptionCardDto, KnowledgeFeedDto, default_knowledge_graph_api,
};
use crate::services::minio_files::{
    StorageBrowseRequestDto, StorageBrowseResultDto, default_minio_files_api,
};
use crate::services::platform_config::{
    MinioConfigUpdateDto, PlatformConfigDto, PostgresConfigUpdateDto,
    load_platform_config_on_server,
};
use crate::services::skills::{SkillDto, SyncReportDto, default_skills_api};
use crate::services::system_management::{
    DepartmentDto, DictGroupDto, MenuDto, RoleDto, UserWithRolesDto, default_system_management_api,
};
use crate::services::terminal_sessions::{
    TerminalProfileDto, TerminalSessionCreateDto, TerminalSessionInputDto, TerminalSessionListDto,
    TerminalSessionSnapshotDto, default_terminal_sessions_api,
};
use crate::tui::commands::{InternalCommand, execute_system_command, parse_internal_command};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub enum Screen {
    #[default]
    Dashboard,
    Setup,
    Assets,
    Knowledge,
    Notes,
    Storage,
    Console,
    Skills,
    System,
    CliMarket,
    Cloudflare,
}

impl Screen {
    pub const ALL: [Self; 11] = [
        Self::Dashboard,
        Self::Setup,
        Self::Assets,
        Self::Knowledge,
        Self::Notes,
        Self::Storage,
        Self::Console,
        Self::Skills,
        Self::System,
        Self::CliMarket,
        Self::Cloudflare,
    ];

    pub fn title(self) -> &'static str {
        match self {
            Self::Dashboard => "总览",
            Self::Setup => "环境",
            Self::Assets => "资产",
            Self::Knowledge => "知识图谱",
            Self::Notes => "碎片笔记",
            Self::Storage => "对象存储",
            Self::Console => "终端",
            Self::Skills => "技能",
            Self::System => "系统治理",
            Self::CliMarket => "CLI 市场",
            Self::Cloudflare => "Cloudflare",
        }
    }

    pub fn help(self) -> &'static str {
        match self {
            Self::Dashboard => "Tab 切模块，r 刷新，q 退出",
            Self::Setup => "e 编辑配置，Ctrl+S 保存，Esc 取消",
            Self::Assets => "r 刷新资产图，查看摘要",
            Self::Knowledge => "r 刷新知识图谱摘要",
            Self::Notes => "n 新建，d 删除，方向键选中，r 刷新",
            Self::Storage => "Enter 进目录，Backspace 返回上级，r 刷新",
            Self::Console => "c 新建 Shell，会话中 i 发送输入，x 关闭，r 刷新",
            Self::Skills => "s 同步技能，r 刷新列表",
            Self::System => ": 打开命令栏执行 `system ...`，r 刷新",
            Self::CliMarket => "方向键查看条目，r 刷新",
            Self::Cloudflare => "r 刷新 tunnel 与 host 状态",
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Mode {
    #[default]
    Normal,
    SetupEdit,
    NoteEdit,
    Command,
    TerminalInput,
}

#[derive(Clone, Debug, Default)]
pub struct TextBuffer {
    text: String,
    cursor: usize,
}

impl TextBuffer {
    pub fn with_text(text: impl Into<String>) -> Self {
        let text = text.into();
        let cursor = text.len();
        Self { text, cursor }
    }

    pub fn text(&self) -> &str {
        &self.text
    }

    pub fn set_text(&mut self, text: impl Into<String>) {
        self.text = text.into();
        self.cursor = self.text.len();
    }

    pub fn clear(&mut self) {
        self.text.clear();
        self.cursor = 0;
    }

    pub fn insert_char(&mut self, ch: char) {
        self.text.insert(self.cursor, ch);
        self.cursor += ch.len_utf8();
    }

    pub fn insert_newline(&mut self) {
        self.insert_char('\n');
    }

    pub fn backspace(&mut self) {
        if self.cursor == 0 {
            return;
        }
        if let Some((index, ch)) = self.text[..self.cursor].char_indices().last() {
            self.text.replace_range(index..self.cursor, "");
            self.cursor -= ch.len_utf8();
        }
    }

    pub fn delete(&mut self) {
        if self.cursor >= self.text.len() {
            return;
        }
        let next = self.next_boundary(self.cursor);
        self.text.replace_range(self.cursor..next, "");
    }

    pub fn move_left(&mut self) {
        if let Some((index, _)) = self.text[..self.cursor].char_indices().last() {
            self.cursor = index;
        }
    }

    pub fn move_right(&mut self) {
        if self.cursor < self.text.len() {
            self.cursor = self.next_boundary(self.cursor);
        }
    }

    pub fn move_home(&mut self) {
        self.cursor = 0;
    }

    pub fn move_end(&mut self) {
        self.cursor = self.text.len();
    }

    pub fn cursor_line_col(&self) -> (u16, u16) {
        let before = &self.text[..self.cursor];
        let row = before.lines().count().saturating_sub(1) as u16;
        let col = before.lines().last().unwrap_or("").chars().count() as u16;
        (row, col)
    }

    fn next_boundary(&self, index: usize) -> usize {
        self.text[index..]
            .char_indices()
            .nth(1)
            .map(|(offset, _)| index + offset)
            .unwrap_or(self.text.len())
    }
}

#[derive(Clone, Debug)]
pub struct SetupForm {
    pub database_url: String,
    pub minio_endpoint: String,
    pub minio_access_key: String,
    pub minio_secret_key: String,
    pub minio_region: String,
    pub field: usize,
}

impl Default for SetupForm {
    fn default() -> Self {
        Self {
            database_url: "postgresql://postgres:postgres@127.0.0.1:5432/aio".to_string(),
            minio_endpoint: "http://127.0.0.1:9000".to_string(),
            minio_access_key: "minioadmin".to_string(),
            minio_secret_key: "minioadmin".to_string(),
            minio_region: "us-east-1".to_string(),
            field: 0,
        }
    }
}

impl SetupForm {
    pub fn labels() -> [&'static str; 5] {
        [
            "PostgreSQL URL",
            "MinIO Endpoint",
            "MinIO Access Key",
            "MinIO Secret Key",
            "MinIO Region",
        ]
    }

    pub fn next_field(&mut self) {
        self.field = (self.field + 1) % Self::labels().len();
    }

    pub fn prev_field(&mut self) {
        self.field = if self.field == 0 {
            Self::labels().len() - 1
        } else {
            self.field - 1
        };
    }

    pub fn current_value(&self) -> &str {
        match self.field {
            0 => &self.database_url,
            1 => &self.minio_endpoint,
            2 => &self.minio_access_key,
            3 => &self.minio_secret_key,
            _ => &self.minio_region,
        }
    }

    pub fn replace_current(&mut self, value: String) {
        match self.field {
            0 => self.database_url = value,
            1 => self.minio_endpoint = value,
            2 => self.minio_access_key = value,
            3 => self.minio_secret_key = value,
            _ => self.minio_region = value,
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct SystemSnapshot {
    pub users: Vec<UserWithRolesDto>,
    pub roles: Vec<RoleDto>,
    pub menus: Vec<MenuDto>,
    pub departments: Vec<DepartmentDto>,
    pub dict_groups: Vec<DictGroupDto>,
    pub tab: usize,
}

impl SystemSnapshot {
    pub fn tab_titles() -> [&'static str; 5] {
        ["Users", "Roles", "Menus", "Departments", "Dict Groups"]
    }

    pub fn next_tab(&mut self) {
        self.tab = (self.tab + 1) % Self::tab_titles().len();
    }

    pub fn prev_tab(&mut self) {
        self.tab = if self.tab == 0 {
            Self::tab_titles().len() - 1
        } else {
            self.tab - 1
        };
    }
}

#[derive(Clone, Debug, Default)]
pub struct App {
    pub current: Screen,
    pub mode: Mode,
    pub should_quit: bool,
    pub status_line: String,
    pub last_error: Option<String>,
    pub bootstrap: Option<BootstrapStatusDto>,
    pub platform: Option<PlatformConfigDto>,
    pub asset_graph: Option<AssetGraphDto>,
    pub knowledge_feed: Option<KnowledgeFeedDto>,
    pub knowledge_exceptions: Vec<KnowledgeExceptionCardDto>,
    pub notes: Vec<KnowledgeNoteDto>,
    pub notes_selected: usize,
    pub storage: Option<StorageBrowseResultDto>,
    pub storage_selected: usize,
    pub terminals: Option<TerminalSessionListDto>,
    pub terminal_snapshot: Option<TerminalSessionSnapshotDto>,
    pub terminal_selected: usize,
    pub skills: Vec<SkillDto>,
    pub skills_status: Option<SyncReportDto>,
    pub system: SystemSnapshot,
    pub system_output: String,
    pub cli_market: Option<CliMarketCatalog>,
    pub cli_market_selected: usize,
    pub cloudflare: Option<CloudflareTunnelStatusDto>,
    pub setup_form: SetupForm,
    pub setup_buffer: TextBuffer,
    pub note_editor: TextBuffer,
    pub command_buffer: TextBuffer,
    pub terminal_input: TextBuffer,
}

impl App {
    pub fn new() -> Self {
        Self {
            current: Screen::Dashboard,
            mode: Mode::Normal,
            status_line: "AIO Rust TUI".to_string(),
            setup_buffer: TextBuffer::default(),
            note_editor: TextBuffer::default(),
            command_buffer: TextBuffer::with_text("system "),
            terminal_input: TextBuffer::default(),
            ..Self::default()
        }
    }

    pub async fn bootstrap(&mut self) {
        if let Err(err) = self.refresh_bootstrap().await {
            self.set_error(err);
        }
        if self
            .bootstrap
            .as_ref()
            .map(|status| status.setup_required)
            .unwrap_or(false)
        {
            self.current = Screen::Setup;
            self.enter_setup_mode();
        } else if let Err(err) = self.refresh_dashboard().await {
            self.set_error(err);
        }
    }

    pub async fn refresh_current(&mut self) -> Result<(), String> {
        match self.current {
            Screen::Dashboard => self.refresh_dashboard().await,
            Screen::Setup => self.refresh_bootstrap().await,
            Screen::Assets => self.refresh_assets().await,
            Screen::Knowledge => self.refresh_knowledge().await,
            Screen::Notes => self.refresh_notes().await,
            Screen::Storage => self.refresh_storage(None).await,
            Screen::Console => self.refresh_console().await,
            Screen::Skills => self.refresh_skills().await,
            Screen::System => self.refresh_system().await,
            Screen::CliMarket => self.refresh_cli_market().await,
            Screen::Cloudflare => self.refresh_cloudflare().await,
        }
    }

    pub fn next_screen(&mut self) {
        let current_index = Screen::ALL
            .iter()
            .position(|screen| *screen == self.current)
            .unwrap_or_default();
        self.current = Screen::ALL[(current_index + 1) % Screen::ALL.len()];
    }

    pub fn prev_screen(&mut self) {
        let current_index = Screen::ALL
            .iter()
            .position(|screen| *screen == self.current)
            .unwrap_or_default();
        self.current = if current_index == 0 {
            Screen::ALL[Screen::ALL.len() - 1]
        } else {
            Screen::ALL[current_index - 1]
        };
    }

    pub fn set_status(&mut self, message: impl Into<String>) {
        self.status_line = message.into();
        self.last_error = None;
    }

    pub fn set_error(&mut self, message: impl Into<String>) {
        let message = message.into();
        self.status_line = message.clone();
        self.last_error = Some(message);
    }

    pub fn enter_setup_mode(&mut self) {
        self.mode = Mode::SetupEdit;
        self.setup_buffer
            .set_text(self.setup_form.current_value().to_string());
    }

    pub fn enter_note_mode(&mut self) {
        self.mode = Mode::NoteEdit;
        self.note_editor.clear();
    }

    pub fn enter_command_mode(&mut self) {
        self.mode = Mode::Command;
        if self.command_buffer.text().trim().is_empty() {
            self.command_buffer.set_text("system ");
        }
    }

    pub fn enter_terminal_input_mode(&mut self) {
        self.mode = Mode::TerminalInput;
        self.terminal_input.clear();
    }

    pub fn cancel_mode(&mut self) {
        self.mode = Mode::Normal;
        self.setup_buffer.clear();
        self.command_buffer.set_text("system ");
        self.terminal_input.clear();
    }

    pub fn active_note(&self) -> Option<&KnowledgeNoteDto> {
        self.notes.get(self.notes_selected)
    }

    pub fn storage_entries(&self) -> Vec<(String, bool)> {
        let mut entries = Vec::new();
        if let Some(storage) = &self.storage {
            for folder in &storage.folders {
                entries.push((format!("{}/", folder.name), true));
            }
            for file in &storage.files {
                entries.push((file.name.clone(), false));
            }
        }
        entries
    }

    pub fn active_terminal_id(&self) -> Option<String> {
        self.terminal_snapshot
            .as_ref()
            .map(|snapshot| snapshot.summary.id.to_string())
            .or_else(|| {
                self.terminals
                    .as_ref()
                    .and_then(|list| list.sessions.get(self.terminal_selected))
                    .map(|session| session.id.to_string())
            })
    }

    pub async fn on_tick(&mut self) {
        if self.current == Screen::Console {
            let _ = self.refresh_terminal_snapshot().await;
        }
    }

    pub async fn save_setup(&mut self) -> Result<(), String> {
        self.setup_form
            .replace_current(self.setup_buffer.text().to_string());
        let result = save_platform_setup_on_server(BootstrapPlatformSetupDto {
            postgres: Some(PostgresConfigUpdateDto {
                database_url: self.setup_form.database_url.clone(),
            }),
            minio: Some(MinioConfigUpdateDto {
                endpoint: self.setup_form.minio_endpoint.clone(),
                access_key: self.setup_form.minio_access_key.clone(),
                secret_key: Some(self.setup_form.minio_secret_key.clone()),
                region: Some(self.setup_form.minio_region.clone()),
            }),
        })
        .await?;
        self.platform = Some(result.config.clone());
        self.mode = Mode::Normal;
        self.set_status(result.message);
        self.refresh_bootstrap().await?;
        Ok(())
    }

    pub async fn save_note(&mut self) -> Result<(), String> {
        let body = self.note_editor.text().trim().to_string();
        if body.is_empty() {
            return Err("笔记内容不能为空。".to_string());
        }
        let tags = extract_tags(&body);
        let saved = default_knowledge_entries_api()
            .save_entry(KnowledgeEntryUpsertDto {
                source_path: String::new(),
                relative_path: String::new(),
                title: first_line(&body).to_string(),
                body,
                tags,
            })
            .await?;
        self.mode = Mode::Normal;
        self.set_status(format!("已保存笔记：{}", saved.title));
        self.refresh_notes().await?;
        Ok(())
    }

    pub async fn delete_selected_note(&mut self) -> Result<(), String> {
        let Some(note) = self.active_note() else {
            return Ok(());
        };
        let source_path = note.source_path.clone();
        let title = note.title.clone();
        default_knowledge_entries_api()
            .delete_entry(KnowledgeEntryDeleteDto { source_path })
            .await?;
        self.set_status(format!("已删除笔记：{title}"));
        self.refresh_notes().await?;
        Ok(())
    }

    pub async fn submit_command(&mut self) -> Result<(), String> {
        let command = self.command_buffer.text().trim().to_string();
        match parse_internal_command(&command)? {
            InternalCommand::Refresh => {
                self.refresh_current().await?;
                self.mode = Mode::Normal;
                self.set_status("当前模块已刷新。");
                Ok(())
            }
            InternalCommand::System(_) => {
                let output = execute_system_command(&command).await?;
                self.system_output = output;
                self.mode = Mode::Normal;
                self.refresh_system().await?;
                self.set_status(format!("已执行：{command}"));
                Ok(())
            }
        }
    }

    pub async fn submit_terminal_input(&mut self) -> Result<(), String> {
        let Some(session_id) = self.active_terminal_id() else {
            return Err("当前没有活跃终端会话。".to_string());
        };
        let data = self.terminal_input.text().to_string();
        let snapshot = default_terminal_sessions_api()
            .send_input(
                session_id,
                TerminalSessionInputDto {
                    data: format!("{data}\n"),
                },
            )
            .await?;
        self.terminal_snapshot = Some(snapshot);
        self.mode = Mode::Normal;
        self.set_status("终端输入已发送。");
        Ok(())
    }

    pub async fn create_terminal_session(&mut self) -> Result<(), String> {
        let snapshot = default_terminal_sessions_api()
            .create_session(TerminalSessionCreateDto {
                profile: TerminalProfileDto::Shell,
                cwd: None,
                title: Some("AIO Shell".to_string()),
                rows: Some(32),
                cols: Some(120),
            })
            .await?;
        self.terminal_snapshot = Some(snapshot);
        self.refresh_console().await?;
        self.set_status("已创建 Shell 会话。");
        Ok(())
    }

    pub async fn close_active_terminal(&mut self) -> Result<(), String> {
        let Some(session_id) = self.active_terminal_id() else {
            return Ok(());
        };
        default_terminal_sessions_api()
            .close_session(session_id)
            .await?;
        self.terminal_snapshot = None;
        self.refresh_console().await?;
        self.set_status("终端会话已关闭。");
        Ok(())
    }

    pub async fn sync_skills(&mut self) -> Result<(), String> {
        let report = default_skills_api()
            .sync_skills()
            .await
            .map_err(|err| err.to_string())?;
        self.skills_status = Some(report.clone());
        self.set_status(format!(
            "技能同步完成：FS+{} / PG+{} / 冲突 {}",
            report.added_to_fs.len() + report.updated_in_fs.len(),
            report.added_to_pg.len() + report.updated_in_pg.len(),
            report.conflicts.len(),
        ));
        self.refresh_skills().await?;
        Ok(())
    }

    pub async fn open_storage_selected(&mut self) -> Result<(), String> {
        let Some(storage) = &self.storage else {
            return Ok(());
        };
        if self.storage_selected < storage.folders.len() {
            let prefix = storage.folders[self.storage_selected].prefix.clone();
            self.refresh_storage(Some(prefix)).await?;
        }
        Ok(())
    }

    pub async fn storage_parent(&mut self) -> Result<(), String> {
        let prefix = self
            .storage
            .as_ref()
            .and_then(|storage| storage.parent_prefix.clone());
        self.refresh_storage(prefix).await
    }

    pub async fn refresh_bootstrap(&mut self) -> Result<(), String> {
        self.bootstrap =
            Some(crate::services::desktop_bootstrap::bootstrap_status_on_server().await?);
        self.platform = Some(load_platform_config_on_server().await?);
        if let Some(platform) = &self.platform {
            self.setup_form.database_url = platform.postgres.database_url.clone();
            self.setup_form.minio_endpoint = platform.minio.endpoint.clone();
            self.setup_form.minio_access_key = platform.minio.access_key.clone();
            self.setup_form.minio_region = platform.minio.region.clone();
        }
        self.set_status("环境状态已刷新。");
        Ok(())
    }

    pub async fn refresh_dashboard(&mut self) -> Result<(), String> {
        self.refresh_bootstrap().await?;
        self.refresh_notes().await?;
        self.refresh_system().await?;
        self.refresh_cloudflare().await?;
        self.refresh_skills().await?;
        self.refresh_cli_market().await?;
        Ok(())
    }

    pub async fn refresh_assets(&mut self) -> Result<(), String> {
        let api = default_asset_graph_api();
        self.asset_graph = Some(api.graph().await.map_err(|err| err.to_string())?);
        self.set_status("资产图已刷新。");
        Ok(())
    }

    pub async fn refresh_knowledge(&mut self) -> Result<(), String> {
        let api = default_knowledge_graph_api();
        self.knowledge_feed = Some(api.feed().await.map_err(|err| err.to_string())?);
        self.knowledge_exceptions = api.exceptions().await.map_err(|err| err.to_string())?;
        self.set_status("知识图谱摘要已刷新。");
        Ok(())
    }

    pub async fn refresh_notes(&mut self) -> Result<(), String> {
        self.notes = default_knowledge_entries_api().list_entries().await?;
        if self.notes_selected >= self.notes.len() && !self.notes.is_empty() {
            self.notes_selected = self.notes.len() - 1;
        }
        self.set_status(format!("已加载 {} 条碎片笔记。", self.notes.len()));
        Ok(())
    }

    pub async fn refresh_storage(&mut self, prefix: Option<String>) -> Result<(), String> {
        let browse = default_minio_files_api()
            .browse(StorageBrowseRequestDto {
                prefix: prefix.unwrap_or_default(),
            })
            .await
            .map_err(|err| err.to_string())?;
        self.storage = Some(browse);
        self.storage_selected = 0;
        self.set_status("对象存储目录已刷新。");
        Ok(())
    }

    pub async fn refresh_console(&mut self) -> Result<(), String> {
        self.terminals = Some(default_terminal_sessions_api().list_sessions().await?);
        if self
            .terminals
            .as_ref()
            .is_some_and(|list| !list.sessions.is_empty())
        {
            self.refresh_terminal_snapshot().await?;
        } else {
            self.terminal_snapshot = None;
        }
        self.set_status("终端会话列表已刷新。");
        Ok(())
    }

    pub async fn refresh_terminal_snapshot(&mut self) -> Result<(), String> {
        let Some(list) = &self.terminals else {
            return Ok(());
        };
        let Some(session) = list.sessions.get(self.terminal_selected) else {
            return Ok(());
        };
        self.terminal_snapshot = Some(
            default_terminal_sessions_api()
                .get_snapshot(session.id.to_string())
                .await?,
        );
        Ok(())
    }

    pub async fn refresh_skills(&mut self) -> Result<(), String> {
        let api = default_skills_api();
        self.skills = api.list_skills().await.map_err(|err| err.to_string())?;
        self.skills_status = Some(api.server_status().await.map_err(|err| err.to_string())?);
        self.set_status(format!("技能清单已刷新，共 {} 条。", self.skills.len()));
        Ok(())
    }

    pub async fn refresh_system(&mut self) -> Result<(), String> {
        let api = default_system_management_api();
        self.system.users = api.list_users().await.map_err(|err| err.to_string())?;
        self.system.roles = api.list_roles().await.map_err(|err| err.to_string())?;
        self.system.menus = api.list_menus().await.map_err(|err| err.to_string())?;
        self.system.departments = api
            .list_departments()
            .await
            .map_err(|err| err.to_string())?;
        self.system.dict_groups = api
            .list_dict_groups()
            .await
            .map_err(|err| err.to_string())?;
        self.set_status(format!(
            "系统数据已刷新：users {} / roles {} / menus {}",
            self.system.users.len(),
            self.system.roles.len(),
            self.system.menus.len()
        ));
        Ok(())
    }

    pub async fn refresh_cli_market(&mut self) -> Result<(), String> {
        self.cli_market = Some(
            default_cli_market_api()
                .catalog()
                .await
                .map_err(|err| err.to_string())?,
        );
        self.set_status("CLI 市场已刷新。");
        Ok(())
    }

    pub async fn refresh_cloudflare(&mut self) -> Result<(), String> {
        self.cloudflare = Some(cloudflare_tunnel_status_on_server().await?);
        self.set_status("Cloudflare Tunnel 状态已刷新。");
        Ok(())
    }

    pub fn dashboard_lines(&self) -> Vec<String> {
        let mut lines = Vec::new();
        if let Some(platform) = &self.platform {
            lines.push(format!(
                "PostgreSQL: {} | MinIO: {}",
                if platform.postgres.reachable {
                    "ready"
                } else {
                    "down"
                },
                if platform.minio.reachable {
                    "ready"
                } else {
                    "down"
                }
            ));
        }
        lines.push(format!("Notes: {}", self.notes.len()));
        lines.push(format!(
            "System: users {} / roles {} / menus {}",
            self.system.users.len(),
            self.system.roles.len(),
            self.system.menus.len()
        ));
        if let Some(skills) = &self.skills_status {
            lines.push(format!(
                "Skills: pg_online={} conflicts={}",
                skills.pg_online,
                skills.conflicts.len()
            ));
        }
        if let Some(catalog) = &self.cli_market {
            lines.push(format!(
                "CLI Market: total {} / published {}",
                catalog.summary.total_entries, catalog.summary.published_entries
            ));
        }
        if let Some(cloudflare) = &self.cloudflare {
            lines.push(format!(
                "Cloudflare: hosts {} / running {}",
                cloudflare.host_count, cloudflare.tunnel_running
            ));
        }
        lines
    }

    pub fn system_tab_lines(&self) -> Vec<String> {
        match self.system.tab {
            0 => self
                .system
                .users
                .iter()
                .map(|user| {
                    format!(
                        "{} [{}] roles={}",
                        user.user.username,
                        user.user.status,
                        user.role_names.join(",")
                    )
                })
                .collect(),
            1 => self
                .system
                .roles
                .iter()
                .map(|role| format!("{} menus={}", role.name, role.menu_count))
                .collect(),
            2 => self
                .system
                .menus
                .iter()
                .map(|menu| format!("{} -> {}", menu.name, menu.route))
                .collect(),
            3 => self
                .system
                .departments
                .iter()
                .map(|department| format!("{} sort={}", department.name, department.sort_order))
                .collect(),
            _ => self
                .system
                .dict_groups
                .iter()
                .map(|group| format!("{} :: {}", group.name, group.description))
                .collect(),
        }
    }

    pub fn skills_summary(&self) -> String {
        let mut text = String::new();
        if let Some(status) = &self.skills_status {
            let _ = write!(
                text,
                "pg_online={} fs_root={} conflicts={}",
                status.pg_online,
                status.fs_root,
                status.conflicts.len()
            );
        }
        text
    }

    pub fn cli_market_detail(&self) -> String {
        let Some(catalog) = &self.cli_market else {
            return "暂无 CLI 市场数据。".to_string();
        };
        let Some(entry) = catalog.entries.get(self.cli_market_selected) else {
            return "暂无条目。".to_string();
        };
        let mut text = String::new();
        let _ = writeln!(text, "{} {}", entry.slug, entry.latest_version);
        let _ = writeln!(text, "vendor: {}", entry.vendor_name);
        let _ = writeln!(text, "category: {}", entry.category_code);
        let _ = writeln!(text, "entry: {}", entry.entry_point);
        let _ = writeln!(text, "install methods:");
        for method in &entry.install_methods {
            let _ = writeln!(
                text,
                "- {} {} -> {}",
                method.platform.code(),
                method.installer_kind.code(),
                method.command_template
            );
        }
        text
    }
}

fn first_line(value: &str) -> &str {
    value.lines().next().unwrap_or("未命名碎片")
}

fn extract_tags(value: &str) -> Vec<String> {
    value
        .split_whitespace()
        .filter_map(|token| token.strip_prefix('#'))
        .map(|tag| tag.trim_matches(|ch: char| !ch.is_alphanumeric() && ch != '-' && ch != '_'))
        .filter(|tag| !tag.is_empty())
        .map(|tag| tag.to_string())
        .collect()
}
