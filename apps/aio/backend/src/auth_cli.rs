use std::{
    env, fs,
    io::{self, Read, Write},
    path::{Path, PathBuf},
};

#[cfg(unix)]
use std::os::unix::fs::{OpenOptionsExt, PermissionsExt};

use anyhow::{Context, bail};
use az_agent_runtime_contract::{LoginRequest, SessionUser};
use chrono::{DateTime, Duration, Utc};
use reqwest::{
    Client, StatusCode,
    header::{COOKIE, HeaderMap, SET_COOKIE},
};
use serde::{Deserialize, Serialize};

use crate::cli::{
    AuthServerArgs, KeyAddArgs, KeyCommand, KeySelectorArgs, KeyValueArgs, LoginArgs, RegArgs,
};
use crate::services::system_management::{
    ApiKeyOwnerDto, UserUpsertDto, admin_role_id_on_server, authorize_user_roles_on_server,
    create_api_key_on_server, create_user_on_server, get_user_on_server,
    owner_drive_id_for_username, resolve_api_key_on_server, revoke_api_key_on_server,
};

const COOKIE_NAME: &str = "aio_session";
const DEFAULT_SERVER_URL: &str = "http://127.0.0.1:8787";
const AUTH_FILE_NAME: &str = "auth.json";

/// Saved AIO CLI login state.
///
/// The session cookie is intentionally local-only material and is written to
/// `~/.config/aio/auth.json` with `0600` permissions on Unix platforms.
#[derive(Clone, Deserialize, Serialize)]
pub struct StoredAioAuth {
    pub server_url: String,
    pub username: String,
    pub session_cookie: String,
    pub logged_in_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
    #[serde(default)]
    pub drive_api_key: Option<StoredDriveApiKey>,
    #[serde(default)]
    pub trusted_api_keys: Vec<StoredTrustedApiKey>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct StoredDriveApiKey {
    pub api_key: String,
    pub key_prefix: String,
    pub owner_user_id: i32,
    pub owner_username: String,
    pub owner_nickname: String,
    pub owner_status: String,
    #[serde(alias = "owner_space_id")]
    pub owner_drive_id: String,
    pub label: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct StoredTrustedApiKey {
    pub api_key: String,
    pub key_prefix: String,
    pub owner_user_id: i32,
    pub owner_username: String,
    pub owner_nickname: String,
    pub owner_status: String,
    #[serde(alias = "owner_space_id")]
    pub owner_drive_id: String,
    pub label: String,
    pub added_at: DateTime<Utc>,
}

impl StoredAioAuth {
    /// Builds the HTTP Cookie header value needed by authenticated CLI calls.
    #[must_use]
    pub fn cookie_header(&self) -> String {
        format!("{COOKIE_NAME}={}", self.session_cookie)
    }

    /// Returns true when the stored session has a known expiry in the past.
    #[must_use]
    pub fn is_expired(&self) -> bool {
        self.expires_at
            .is_some_and(|expires_at| expires_at <= Utc::now())
    }
}

struct SessionCookie {
    value: String,
    max_age_seconds: Option<i64>,
}

/// Runs `aio reg`.
///
/// # Errors
/// Returns an error when PostgreSQL is not configured, the user already exists,
/// password input is invalid, or the registration cannot be persisted.
pub async fn run_reg_command(args: RegArgs) -> anyhow::Result<()> {
    let username = args.username.trim().to_owned();
    if username.is_empty() {
        bail!("用户名不能为空");
    }
    let password = resolve_required_password(args.password.as_deref(), args.password_stdin)?;
    let nickname = if args.nickname.trim().is_empty() {
        username.clone()
    } else {
        args.nickname.trim().to_owned()
    };
    let status = args.status.trim().to_owned();
    if status.is_empty() {
        bail!("用户状态不能为空");
    }

    let user = create_user_on_server(UserUpsertDto {
        username,
        password,
        nickname,
        status,
    })
    .await
    .map_err(|err| anyhow::anyhow!("注册用户失败: {err}"))?;

    let mut role_ids = args.role_ids;
    if args.admin {
        let admin_role_id = admin_role_id_on_server()
            .await
            .map_err(|err| anyhow::anyhow!("查询管理员角色失败: {err}"))?;
        if !role_ids.contains(&admin_role_id) {
            role_ids.push(admin_role_id);
        }
    }

    if !role_ids.is_empty() {
        authorize_user_roles_on_server(user.id, role_ids)
            .await
            .map_err(|err| anyhow::anyhow!("绑定用户角色失败: {err}"))?;
    }

    let registered = get_user_on_server(user.id)
        .await
        .map_err(|err| anyhow::anyhow!("读取注册用户失败: {err}"))?;
    println!("已注册");
    println!("USER    {}", registered.user.username);
    println!("ID      {}", registered.user.id);
    println!("STATUS  {}", registered.user.status);
    if registered.role_names.is_empty() {
        println!("ROLES   -");
    } else {
        println!("ROLES   {}", registered.role_names.join(","));
    }
    Ok(())
}

/// Runs `aio login`.
///
/// # Errors
/// Returns an error when credentials cannot be resolved, the server rejects
/// them, or the local auth file cannot be written.
pub async fn run_login_command(args: LoginArgs) -> anyhow::Result<()> {
    if args.use_gh {
        return run_login_with_gh(args).await;
    }

    let server_url = resolve_server_url(args.server.as_deref())?;
    let username = resolve_username(args.username.as_deref());
    let password = resolve_password(&args)?;
    let login_url = api_url(&server_url, "/api/admin/session/login");
    let client = Client::new();

    let response = client
        .post(&login_url)
        .json(&LoginRequest {
            username: username.clone(),
            password,
        })
        .send()
        .await
        .with_context(|| format!("请求登录接口失败: {login_url}"))?;

    let status = response.status();
    let headers = response.headers().clone();
    let body = response.text().await.context("读取登录响应失败")?;
    ensure_success(status, &body, "登录失败")?;

    let session: SessionUser = serde_json::from_str(&body)
        .with_context(|| format!("解析登录响应失败: {}", summarize_body(&body)))?;
    if !session.authenticated {
        bail!("登录失败: 后台没有返回已认证会话");
    }

    let cookie = extract_session_cookie(&headers).context("登录成功但后台没有返回 aio_session")?;
    let logged_in_at = Utc::now();
    let expires_at = cookie
        .max_age_seconds
        .map(|seconds| logged_in_at + Duration::seconds(seconds));
    let previous_auth = load_auth_file().ok().flatten();
    let trusted_api_keys = previous_auth
        .as_ref()
        .map(|auth| auth.trusted_api_keys.clone())
        .unwrap_or_default();
    let username = session.username.unwrap_or(username);
    let drive_api_key = ensure_drive_api_key(previous_auth.as_ref(), &username).await?;
    let auth = StoredAioAuth {
        server_url,
        username,
        session_cookie: cookie.value,
        logged_in_at,
        expires_at,
        drive_api_key,
        trusted_api_keys,
    };
    save_auth_file(&auth)?;
    let migrated = migrate_legacy_drive_after_login(&auth).await?;

    print_auth_summary("已登录", &auth);
    if migrated > 0 {
        println!("DRIVE_MIGRATED {migrated}");
    }
    Ok(())
}

async fn run_login_with_gh(args: LoginArgs) -> anyhow::Result<()> {
    let login = gh_cli_output(["api", "user", "--jq", ".login"])
        .context("读取 gh 当前登录用户失败；请先运行 gh auth login")?;
    let username = if let Some(explicit) = args.username.as_deref() {
        let explicit = explicit.trim();
        if explicit.is_empty() {
            bail!("--username 不能为空");
        }
        if explicit != login {
            bail!("--use-gh 模式下 --username 必须与 gh 当前登录用户一致: {login}");
        }
        explicit.to_owned()
    } else {
        login
    };
    let token = gh_cli_output(["auth", "token"])
        .context("读取 gh token 失败；请先确认 gh auth status 可用")?;
    if token.trim().is_empty() {
        bail!("gh 没有返回可用 token");
    }

    let owner_drive_id = owner_drive_id_for_username(&username);
    let now = Utc::now();
    let auth = StoredAioAuth {
        server_url: "gh://github.com".to_owned(),
        username: username.clone(),
        session_cookie: format!("gh:{username}"),
        logged_in_at: now,
        expires_at: None,
        drive_api_key: Some(StoredDriveApiKey {
            api_key: token.clone(),
            key_prefix: token.chars().take(18).collect(),
            owner_user_id: 0,
            owner_username: username.clone(),
            owner_nickname: username.clone(),
            owner_status: "enabled".to_owned(),
            owner_drive_id: owner_drive_id.clone(),
            label: "gh-auth".to_owned(),
            created_at: now,
        }),
        trusted_api_keys: load_auth_file()
            .ok()
            .flatten()
            .map(|existing| existing.trusted_api_keys)
            .unwrap_or_default(),
    };
    save_auth_file(&auth)?;
    let migrated = migrate_legacy_drive_after_login(&auth).await?;

    print_auth_summary("已通过 gh 登录态写入本机 Drive 登录态", &auth);
    println!("GH_USER {}", username);
    if migrated > 0 {
        println!("DRIVE_MIGRATED {migrated}");
    }
    Ok(())
}

/// Runs `aio logout`.
///
/// # Errors
/// Returns an error when the local auth path cannot be resolved or cleaned up.
pub async fn run_logout_command(args: AuthServerArgs) -> anyhow::Result<()> {
    let Some(auth) = load_or_cleanup_corrupt_auth()? else {
        println!("当前未登录");
        return Ok(());
    };

    let server_url = args
        .server
        .as_deref()
        .map(normalize_server_url)
        .transpose()?
        .unwrap_or_else(|| auth.server_url.clone());
    if let Err(err) = post_logout(&server_url, &auth).await {
        eprintln!("服务端登出请求失败，本地登录态仍会删除: {err:#}");
    }

    remove_auth_file()?;
    println!("已退出登录");
    println!("SERVER  {server_url}");
    println!("AUTH    {}", auth_file_path()?.display());
    Ok(())
}

/// Runs `aio whoami`.
///
/// # Errors
/// Returns an error when no local auth exists, the session is expired, or the
/// server no longer accepts the stored cookie.
pub async fn run_whoami_command(args: AuthServerArgs) -> anyhow::Result<()> {
    let auth = load_auth_file()?.context("未登录: 请先运行 aio login")?;
    if auth.is_expired() {
        bail!("登录态已过期: 请重新运行 aio login");
    }

    if auth.server_url.starts_with("gh://") {
        print_auth_summary("已登录", &auth);
        println!("MODE    github-cli");
        return Ok(());
    }

    let server_url = args
        .server
        .as_deref()
        .map(normalize_server_url)
        .transpose()?
        .unwrap_or_else(|| auth.server_url.clone());
    let session = get_session(&server_url, &auth).await?;
    if !session.authenticated {
        bail!("服务端会话未认证: 请重新运行 aio login");
    }

    let mut displayed = auth;
    displayed.server_url = server_url;
    if let Some(username) = session.username {
        displayed.username = username;
    }
    print_auth_summary("已登录", &displayed);
    Ok(())
}

/// Runs `aio key`.
///
/// # Errors
/// Returns an error when key creation, verification, or local trust storage
/// fails.
pub async fn run_key_command(command: KeyCommand) -> anyhow::Result<()> {
    match command {
        KeyCommand::Create(args) => run_key_create(&args.label).await,
        KeyCommand::Whoami(args) => run_key_whoami(args).await,
        KeyCommand::Add(args) => run_key_add(args).await,
        KeyCommand::Remove(args) => run_key_remove(args).await,
        KeyCommand::Revoke(args) => run_key_revoke(args).await,
        KeyCommand::List => run_key_list().await,
    }
}

async fn run_key_create(label: &str) -> anyhow::Result<()> {
    let auth = require_current_auth()?;
    let created = create_api_key_on_server(&auth.username, label)
        .await
        .map_err(|err| anyhow::anyhow!("创建 API key 失败: {err}"))?;
    println!("已创建 API key");
    println!("API_KEY {}", created.api_key);
    print_api_key_owner(&created.owner);
    Ok(())
}

async fn run_key_whoami(args: KeyValueArgs) -> anyhow::Result<()> {
    let owner = resolve_api_key_on_server(&args.api_key)
        .await
        .map_err(|err| anyhow::anyhow!("查询 API key 失败: {err}"))?;
    print_api_key_owner(&owner);
    Ok(())
}

async fn ensure_drive_api_key(
    previous_auth: Option<&StoredAioAuth>,
    username: &str,
) -> anyhow::Result<Option<StoredDriveApiKey>> {
    if let Some(key) = previous_auth
        .and_then(|auth| auth.drive_api_key.clone())
        .filter(|key| key.owner_username == username && !key.api_key.trim().is_empty())
    {
        return Ok(Some(key));
    }
    let created = create_api_key_on_server(username, "drive-default")
        .await
        .map_err(|err| anyhow::anyhow!("登录成功但准备 Drive API key 失败: {err}"))?;
    Ok(Some(stored_drive_api_key_from_created(created)))
}

async fn migrate_legacy_drive_after_login(auth: &StoredAioAuth) -> anyhow::Result<u64> {
    if auth.drive_api_key.is_none() {
        return Ok(0);
    }
    az_drive_app::migrate_legacy_main_for_current_owner()
        .await
        .context("登录成功，但迁移历史 Drive 数据失败")
}

fn stored_drive_api_key_from_created(
    created: crate::services::system_management::CreatedApiKeyDto,
) -> StoredDriveApiKey {
    StoredDriveApiKey {
        api_key: created.api_key,
        key_prefix: created.owner.key_prefix,
        owner_user_id: created.owner.user_id,
        owner_username: created.owner.username,
        owner_nickname: created.owner.nickname,
        owner_status: created.owner.status,
        owner_drive_id: created.owner.owner_drive_id,
        label: created.owner.label,
        created_at: Utc::now(),
    }
}

async fn run_key_add(args: KeyAddArgs) -> anyhow::Result<()> {
    let mut auth = require_current_auth()?;
    let owner = resolve_api_key_on_server(&args.api_key)
        .await
        .map_err(|err| anyhow::anyhow!("验证 API key 失败: {err}"))?;
    let label = if args.label.trim().is_empty() {
        owner.label.clone()
    } else {
        args.label.trim().to_owned()
    };
    auth.trusted_api_keys
        .retain(|key| key.key_prefix != owner.key_prefix);
    auth.trusted_api_keys.push(StoredTrustedApiKey {
        api_key: args.api_key,
        key_prefix: owner.key_prefix.clone(),
        owner_user_id: owner.user_id,
        owner_username: owner.username.clone(),
        owner_nickname: owner.nickname.clone(),
        owner_status: owner.status.clone(),
        owner_drive_id: owner.owner_drive_id.clone(),
        label,
        added_at: Utc::now(),
    });
    auth.trusted_api_keys
        .sort_by(|left, right| left.owner_username.cmp(&right.owner_username));
    save_auth_file(&auth)?;
    let synced_count = sync_drive_after_key_add().await?;
    println!("已添加融合源并纳入双向同步");
    print_api_key_owner(&owner);
    println!("SYNCED  {synced_count}");
    Ok(())
}

async fn sync_drive_after_key_add() -> anyhow::Result<usize> {
    let agent = az_drive_app::build_agent()
        .await
        .context("融合源已保存，但初始化 Drive 同步失败")?;
    let statuses = agent
        .sync_once()
        .await
        .context("融合源已保存，但首次双向同步失败")?;
    Ok(statuses.len())
}

async fn run_key_list() -> anyhow::Result<()> {
    let auth = require_current_auth()?;
    println!("{:<18} {:<18} {:<18} LABEL", "PREFIX", "OWNER", "DRIVE");
    for key in auth.trusted_api_keys {
        println!(
            "{:<18} {:<18} {:<18} {}",
            key.key_prefix, key.owner_username, key.owner_drive_id, key.label
        );
    }
    Ok(())
}

async fn run_key_remove(args: KeySelectorArgs) -> anyhow::Result<()> {
    let mut auth = require_current_auth()?;
    let selector = args.selector.trim();
    let before = auth.trusted_api_keys.len();
    auth.trusted_api_keys
        .retain(|key| key.key_prefix != selector && key.owner_username != selector);
    let removed = before.saturating_sub(auth.trusted_api_keys.len());
    save_auth_file(&auth)?;
    println!("已移除本机融合源: {removed}");
    Ok(())
}

async fn run_key_revoke(args: KeySelectorArgs) -> anyhow::Result<()> {
    let auth = require_current_auth()?;
    let revoked = revoke_api_key_on_server(&auth.username, &args.selector)
        .await
        .map_err(|err| anyhow::anyhow!("撤销 API key 失败: {err}"))?;
    println!("已撤销: {revoked}");
    Ok(())
}

fn require_current_auth() -> anyhow::Result<StoredAioAuth> {
    let auth = load_auth_file()?.context("未登录: 请先运行 aio login")?;
    if auth.is_expired() {
        bail!("登录态已过期: 请重新运行 aio login");
    }
    Ok(auth)
}

fn print_api_key_owner(owner: &ApiKeyOwnerDto) {
    println!("OWNER   {}", owner.username);
    println!("USER_ID {}", owner.user_id);
    println!("NICK    {}", owner.nickname);
    println!("STATUS  {}", owner.status);
    println!("PREFIX  {}", owner.key_prefix);
    println!("DRIVE   {}", owner.owner_drive_id);
    if !owner.label.is_empty() {
        println!("LABEL   {}", owner.label);
    }
}

/// Returns the default AIO CLI auth file path.
///
/// # Errors
/// Returns an error when neither `XDG_CONFIG_HOME` nor `HOME` is available.
pub fn auth_file_path() -> anyhow::Result<PathBuf> {
    aio_config_dir()
        .map(|dir| dir.join(AUTH_FILE_NAME))
        .context("无法定位 AIO 配置目录: 缺少 XDG_CONFIG_HOME/HOME")
}

/// Loads the stored AIO auth file.
///
/// # Errors
/// Returns an error when the auth file exists but cannot be read or decoded.
pub fn load_auth_file() -> anyhow::Result<Option<StoredAioAuth>> {
    let path = auth_file_path()?;
    load_auth_file_at(&path)
}

/// Resolves the AIO server URL from an explicit value or environment defaults.
///
/// # Errors
/// Returns an error when the resolved URL is empty or uses an unsupported URL
/// scheme.
pub fn resolve_server_url(explicit: Option<&str>) -> anyhow::Result<String> {
    let raw = explicit
        .map(str::to_owned)
        .or_else(|| non_empty_env("AIO_SERVER_URL"))
        .or_else(|| non_empty_env("AIO_API_URL"))
        .or_else(|| non_empty_env("AIO_API_BIND"))
        .unwrap_or_else(|| DEFAULT_SERVER_URL.to_owned());
    normalize_server_url(&raw)
}

fn load_or_cleanup_corrupt_auth() -> anyhow::Result<Option<StoredAioAuth>> {
    match load_auth_file() {
        Ok(auth) => Ok(auth),
        Err(err) => {
            remove_auth_file()?;
            println!("本地登录态已损坏，已删除: {err:#}");
            Ok(None)
        }
    }
}

fn save_auth_file(auth: &StoredAioAuth) -> anyhow::Result<()> {
    let path = auth_file_path()?;
    save_auth_file_at(&path, auth)
}

fn save_auth_file_at(path: &Path, auth: &StoredAioAuth) -> anyhow::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("创建 AIO 配置目录失败: {}", parent.display()))?;
    }

    let encoded = serde_json::to_vec_pretty(auth).context("编码登录态失败")?;
    let mut options = fs::OpenOptions::new();
    options.create(true).write(true).truncate(true);
    #[cfg(unix)]
    options.mode(0o600);
    let mut file = options
        .open(path)
        .with_context(|| format!("打开登录态文件失败: {}", path.display()))?;
    file.write_all(&encoded)
        .with_context(|| format!("写入登录态文件失败: {}", path.display()))?;
    file.write_all(b"\n")
        .with_context(|| format!("写入登录态文件失败: {}", path.display()))?;

    #[cfg(unix)]
    {
        fs::set_permissions(path, fs::Permissions::from_mode(0o600))
            .with_context(|| format!("设置登录态文件权限失败: {}", path.display()))?;
    }

    Ok(())
}

fn load_auth_file_at(path: &Path) -> anyhow::Result<Option<StoredAioAuth>> {
    let content = match fs::read_to_string(path) {
        Ok(content) => content,
        Err(err) if err.kind() == io::ErrorKind::NotFound => return Ok(None),
        Err(err) => {
            return Err(err).with_context(|| format!("读取登录态文件失败: {}", path.display()));
        }
    };
    let auth = serde_json::from_str(&content)
        .with_context(|| format!("解析登录态文件失败: {}", path.display()))?;
    Ok(Some(auth))
}

fn remove_auth_file() -> anyhow::Result<()> {
    let path = auth_file_path()?;
    match fs::remove_file(&path) {
        Ok(()) => Ok(()),
        Err(err) if err.kind() == io::ErrorKind::NotFound => Ok(()),
        Err(err) => Err(err).with_context(|| format!("删除登录态文件失败: {}", path.display())),
    }
}

fn resolve_username(explicit: Option<&str>) -> String {
    explicit
        .map(str::to_owned)
        .or_else(|| non_empty_env("AIO_USERNAME"))
        .or_else(|| non_empty_env("AIO_ADMIN_USERNAME"))
        .unwrap_or_else(|| "admin".to_owned())
}

fn resolve_password(args: &LoginArgs) -> anyhow::Result<String> {
    if args.password.is_some() && args.password_stdin {
        bail!("--password 和 --password-stdin 不能同时使用");
    }

    if args.password_stdin {
        let mut input = String::new();
        io::stdin()
            .read_to_string(&mut input)
            .context("从 stdin 读取密码失败")?;
        let password = input.trim_end_matches(['\r', '\n']).to_owned();
        if password.is_empty() {
            bail!("stdin 中没有读取到密码");
        }
        return Ok(password);
    }

    Ok(args
        .password
        .clone()
        .or_else(|| non_empty_env("AIO_PASSWORD"))
        .or_else(|| non_empty_env("AIO_ADMIN_PASSWORD"))
        .unwrap_or_else(|| "admin".to_owned()))
}

fn resolve_required_password(
    explicit: Option<&str>,
    password_stdin: bool,
) -> anyhow::Result<String> {
    if explicit.is_some() && password_stdin {
        bail!("--password 和 --password-stdin 不能同时使用");
    }
    let password = if password_stdin {
        read_password_from_stdin()?
    } else {
        explicit
            .map(str::to_owned)
            .or_else(|| non_empty_env("AIO_PASSWORD"))
            .or_else(|| non_empty_env("AIO_ADMIN_PASSWORD"))
            .context("缺少注册密码: 请传 --password、--password-stdin 或 AIO_PASSWORD")?
    };
    if password.is_empty() {
        bail!("密码不能为空");
    }
    Ok(password)
}

fn read_password_from_stdin() -> anyhow::Result<String> {
    let mut input = String::new();
    io::stdin()
        .read_to_string(&mut input)
        .context("从 stdin 读取密码失败")?;
    let password = input.trim_end_matches(['\r', '\n']).to_owned();
    if password.is_empty() {
        bail!("stdin 中没有读取到密码");
    }
    Ok(password)
}

fn non_empty_env(key: &str) -> Option<String> {
    env::var(key)
        .ok()
        .map(|value| value.trim().to_owned())
        .filter(|value| !value.is_empty())
}

fn gh_cli_output<const N: usize>(args: [&str; N]) -> anyhow::Result<String> {
    let output = std::process::Command::new("gh")
        .args(args)
        .output()
        .context("调用 gh 命令失败")?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_owned();
        if stderr.is_empty() {
            bail!("gh 命令失败: exit {}", output.status);
        }
        bail!("gh 命令失败: {stderr}");
    }
    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_owned();
    if stdout.is_empty() {
        bail!("gh 命令没有返回内容");
    }
    Ok(stdout)
}

fn aio_config_dir() -> Option<PathBuf> {
    env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .or_else(|| env::var_os("HOME").map(|home| PathBuf::from(home).join(".config")))
        .map(|config| config.join("aio"))
}

fn normalize_server_url(raw: &str) -> anyhow::Result<String> {
    let raw = raw.trim().trim_end_matches('/');
    if raw.is_empty() {
        bail!("AIO 后台地址不能为空");
    }
    if raw.starts_with("http://") || raw.starts_with("https://") {
        return Ok(raw.to_owned());
    }
    if raw.contains("://") {
        bail!("不支持的 AIO 后台地址协议: {raw}");
    }
    if let Some(port) = raw.strip_prefix("0.0.0.0:") {
        return Ok(format!("http://127.0.0.1:{port}"));
    }
    Ok(format!("http://{raw}"))
}

fn api_url(server_url: &str, path: &str) -> String {
    format!("{}{}", server_url.trim_end_matches('/'), path)
}

async fn get_session(server_url: &str, auth: &StoredAioAuth) -> anyhow::Result<SessionUser> {
    let url = api_url(server_url, "/api/admin/session");
    let response = Client::new()
        .get(&url)
        .header(COOKIE, auth.cookie_header())
        .send()
        .await
        .with_context(|| format!("请求会话接口失败: {url}"))?;
    let status = response.status();
    let body = response.text().await.context("读取会话响应失败")?;
    ensure_success(status, &body, "验证登录态失败")?;
    serde_json::from_str(&body)
        .with_context(|| format!("解析会话响应失败: {}", summarize_body(&body)))
}

async fn post_logout(server_url: &str, auth: &StoredAioAuth) -> anyhow::Result<()> {
    let url = api_url(server_url, "/api/admin/session/logout");
    let response = Client::new()
        .post(&url)
        .header(COOKIE, auth.cookie_header())
        .json(&serde_json::json!({}))
        .send()
        .await
        .with_context(|| format!("请求登出接口失败: {url}"))?;
    let status = response.status();
    let body = response.text().await.context("读取登出响应失败")?;
    ensure_success(status, &body, "服务端登出失败")
}

fn ensure_success(status: StatusCode, body: &str, prefix: &str) -> anyhow::Result<()> {
    if status.is_success() {
        return Ok(());
    }
    bail!("{prefix}: HTTP {status}: {}", summarize_body(body));
}

fn extract_session_cookie(headers: &HeaderMap) -> Option<SessionCookie> {
    headers
        .get_all(SET_COOKIE)
        .iter()
        .filter_map(|value| value.to_str().ok())
        .find_map(parse_session_cookie)
}

fn parse_session_cookie(raw: &str) -> Option<SessionCookie> {
    let mut parts = raw.split(';').map(str::trim);
    let first = parts.next()?;
    let (name, value) = first.split_once('=')?;
    if name != COOKIE_NAME || value.is_empty() {
        return None;
    }

    let max_age_seconds = parts
        .filter_map(|part| part.split_once('='))
        .find_map(|(key, value)| {
            key.eq_ignore_ascii_case("Max-Age")
                .then(|| value.parse::<i64>().ok())
                .flatten()
                .filter(|value| *value >= 0)
        });

    Some(SessionCookie {
        value: value.to_owned(),
        max_age_seconds,
    })
}

fn summarize_body(body: &str) -> String {
    let normalized = body.split_whitespace().collect::<Vec<_>>().join(" ");
    if normalized.is_empty() {
        return "<empty response>".to_owned();
    }
    const MAX_LEN: usize = 240;
    if normalized.chars().count() <= MAX_LEN {
        normalized
    } else {
        format!(
            "{}...",
            normalized.chars().take(MAX_LEN).collect::<String>()
        )
    }
}

fn print_auth_summary(title: &str, auth: &StoredAioAuth) {
    println!("{title}");
    println!("USER    {}", auth.username);
    println!("SERVER  {}", auth.server_url);
    println!(
        "AUTH    {}",
        auth_file_path().map_or_else(|_| "-".to_owned(), |path| path.display().to_string())
    );
    match auth.expires_at {
        Some(expires_at) => println!("EXPIRES {}", expires_at.to_rfc3339()),
        None => println!("EXPIRES unknown"),
    }
    if let Some(key) = &auth.drive_api_key {
        println!("DRIVE   {} ({})", key.owner_drive_id, key.key_prefix);
    } else {
        println!("DRIVE   {}", owner_drive_id_for_username(&auth.username));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use reqwest::header::HeaderValue;
    use uuid::Uuid;

    #[test]
    fn parse_session_cookie_should_extract_value_and_max_age() {
        let mut headers = HeaderMap::new();
        headers.append(
            SET_COOKIE,
            HeaderValue::from_static("aio_session=abc.def; Path=/; HttpOnly; Max-Age=3600"),
        );

        let cookie = extract_session_cookie(&headers).expect("session cookie should parse");

        assert_eq!(cookie.value, "abc.def");
        assert_eq!(cookie.max_age_seconds, Some(3600));
    }

    #[test]
    fn normalize_server_url_should_accept_plain_bind_address() -> anyhow::Result<()> {
        assert_eq!(
            normalize_server_url("127.0.0.1:8787")?,
            "http://127.0.0.1:8787"
        );
        assert_eq!(
            normalize_server_url("0.0.0.0:8787")?,
            "http://127.0.0.1:8787"
        );
        Ok(())
    }

    #[test]
    fn auth_file_should_accept_legacy_owner_space_id_fields() -> anyhow::Result<()> {
        let auth: StoredAioAuth = serde_json::from_str(
            r#"
            {
              "server_url": "http://127.0.0.1:8787",
              "username": "zjarlin",
              "session_cookie": "cookie-value",
              "logged_in_at": "2026-05-10T00:00:00Z",
              "expires_at": null,
              "drive_api_key": {
                "api_key": "aio_live_self",
                "key_prefix": "self",
                "owner_user_id": 1,
                "owner_username": "zjarlin",
                "owner_nickname": "zjarlin",
                "owner_status": "enabled",
                "owner_space_id": "user-zjarlin",
                "label": "drive-default",
                "created_at": "2026-05-10T00:00:00Z"
              },
              "trusted_api_keys": [
                {
                  "api_key": "aio_live_other",
                  "key_prefix": "other",
                  "owner_user_id": 2,
                  "owner_username": "lisi",
                  "owner_nickname": "lisi",
                  "owner_status": "enabled",
                  "owner_space_id": "user-lisi",
                  "label": "shared",
                  "added_at": "2026-05-10T00:00:00Z"
                }
              ]
            }
            "#,
        )?;

        assert_eq!(
            auth.drive_api_key
                .as_ref()
                .expect("drive api key should load")
                .owner_drive_id,
            "user-zjarlin"
        );
        assert_eq!(auth.trusted_api_keys[0].owner_drive_id, "user-lisi");
        Ok(())
    }

    #[test]
    fn save_auth_file_should_round_trip_without_relaxing_permissions() -> anyhow::Result<()> {
        let root = env::temp_dir().join(format!("aio-auth-test-{}", Uuid::new_v4()));
        let path = root.join("auth.json");
        let auth = StoredAioAuth {
            server_url: DEFAULT_SERVER_URL.to_owned(),
            username: "admin".to_owned(),
            session_cookie: "cookie-value".to_owned(),
            logged_in_at: Utc::now(),
            expires_at: None,
            drive_api_key: None,
            trusted_api_keys: Vec::new(),
        };

        save_auth_file_at(&path, &auth)?;
        let loaded = load_auth_file_at(&path)?.expect("auth file should exist");

        assert_eq!(loaded.server_url, DEFAULT_SERVER_URL);
        assert_eq!(loaded.username, "admin");
        assert_eq!(loaded.session_cookie, "cookie-value");
        #[cfg(unix)]
        assert_eq!(fs::metadata(&path)?.permissions().mode() & 0o777, 0o600);

        fs::remove_dir_all(root)?;
        Ok(())
    }
}
