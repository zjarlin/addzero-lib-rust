use clap::{Args, Parser, Subcommand, ValueEnum};

#[derive(Debug, Parser)]
#[command(name = "aio")]
#[command(about = "AIO backend API server", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Command>,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    #[command(about = "注册 AIO 用户到本机配置的系统用户表")]
    Reg(RegArgs),
    #[command(about = "登录 AIO 后台并保存本机 CLI 登录态")]
    Login(LoginArgs),
    #[command(about = "清除本机 CLI 登录态，并尽量通知后台退出会话")]
    Logout(AuthServerArgs),
    #[command(about = "查看当前本机 CLI 登录态")]
    Whoami(AuthServerArgs),
    #[command(about = "管理 API key 和本机融合源")]
    #[command(subcommand, alias = "apikey")]
    Key(KeyCommand),
    #[command(about = "管理 AIO CLI 元数据、shell 组件、skill.sh 和外部 CLI")]
    #[command(subcommand)]
    Cli(AioCliCommand),
    #[command(about = "启动 API 后端服务")]
    Serve(ServeArgs),
    #[command(about = "AIO Drive 文件托管命令")]
    #[command(subcommand)]
    Drive(az_drive_app::cli::DriveCommand),
    #[command(about = "运行数据库迁移")]
    Migrate,
    #[command(about = "打印当前架构状态")]
    Status,
    #[command(about = "面向 agent 的系统治理 CLI")]
    System(SystemCli),
}

#[derive(Clone, Copy, Debug, ValueEnum)]
pub enum CliOutputFormat {
    Table,
    Json,
}

#[derive(Debug, Subcommand)]
pub enum AioCliCommand {
    #[command(about = "输出 AIO 内置 CLI 命令元数据")]
    Metadata(CliMetadataArgs),
    #[command(about = "安装或打印面向 agent 的 skill.sh")]
    #[command(subcommand)]
    Skill(AioCliSkillCommand),
    #[command(about = "管理 shell 组件与 ~/.add_fn 构建")]
    #[command(subcommand)]
    Component(AioCliComponentCommand),
    #[command(about = "添加一个本机外部 CLI")]
    Add(ExternalCliAddArgs),
    #[command(about = "列出本机外部 CLI")]
    List(ExternalCliListArgs),
    #[command(about = "移除一个本机外部 CLI")]
    Remove(ExternalCliRemoveArgs),
    #[command(about = "运行一个本机外部 CLI")]
    Run(ExternalCliRunArgs),
}

#[derive(Debug, Args)]
pub struct CliMetadataArgs {
    #[arg(long, value_enum, default_value_t = CliOutputFormat::Table)]
    pub format: CliOutputFormat,
}

#[derive(Debug, Subcommand)]
pub enum AioCliSkillCommand {
    #[command(about = "写入 ~/.agents/skills/aio-cli/skill.sh")]
    Install(CliSkillInstallArgs),
    #[command(about = "打印将要安装的 skill.sh")]
    Print,
}

#[derive(Clone, Copy, Debug, ValueEnum)]
pub enum ShellComponentKindArg {
    Export,
    Alias,
    Function,
    Snippet,
}

#[derive(Debug, Subcommand)]
pub enum AioCliComponentCommand {
    #[command(about = "列出 shell 组件")]
    List(ShellComponentListArgs),
    #[command(about = "查看一个 shell 组件")]
    Get(ShellComponentGetArgs),
    #[command(about = "创建或覆盖一个 shell 组件")]
    Upsert(ShellComponentUpsertArgs),
    #[command(about = "更新 shell 组件的启用和输出状态")]
    Set(ShellComponentSetArgs),
    #[command(about = "移除一个 shell 组件")]
    Remove(ShellComponentRemoveArgs),
    #[command(about = "更新 ~/.add_fn 输出配置")]
    Config(ShellComponentConfigArgs),
    #[command(about = "预览或生成 ~/.add_fn")]
    Build(ShellComponentBuildArgs),
}

#[derive(Debug, Args)]
pub struct ShellComponentListArgs {
    #[arg(long, value_enum, default_value_t = CliOutputFormat::Table)]
    pub format: CliOutputFormat,
}

#[derive(Debug, Args)]
pub struct ShellComponentGetArgs {
    pub name: String,
}

#[derive(Debug, Args)]
pub struct ShellComponentUpsertArgs {
    pub name: String,
    #[arg(long, value_enum)]
    pub kind: ShellComponentKindArg,
    #[arg(long, default_value = "")]
    pub summary: String,
    #[arg(long, default_value_t = true)]
    pub enabled: bool,
    #[arg(long = "render-to-output", default_value_t = true)]
    pub render_to_output: bool,
    #[arg(long)]
    pub value: Option<String>,
    #[arg(long)]
    pub command: Option<String>,
    #[arg(long)]
    pub body: Option<String>,
}

#[derive(Debug, Args)]
pub struct ShellComponentSetArgs {
    pub name: String,
    #[arg(long)]
    pub summary: Option<String>,
    #[arg(long)]
    pub enabled: Option<bool>,
    #[arg(long = "render-to-output")]
    pub render_to_output: Option<bool>,
}

#[derive(Debug, Args)]
pub struct ShellComponentRemoveArgs {
    pub name: String,
}

#[derive(Debug, Args)]
pub struct ShellComponentConfigArgs {
    #[arg(long)]
    pub output: Option<String>,
}

#[derive(Debug, Args)]
pub struct ShellComponentBuildArgs {
    #[arg(long)]
    pub output: Option<String>,
    #[arg(long)]
    pub stdout: bool,
}

#[derive(Debug, Args, Clone, Default)]
pub struct ServeArgs {
    #[arg(long)]
    pub bind: Option<String>,
    #[arg(long = "desktop-token")]
    pub desktop_token: Option<String>,
}

#[derive(Debug, Args)]
pub struct CliSkillInstallArgs {
    #[arg(long)]
    pub root: Option<String>,
    #[arg(long)]
    pub path: Option<String>,
}

#[derive(Debug, Args)]
pub struct ExternalCliAddArgs {
    pub name: String,
    #[arg(long)]
    pub command: String,
    #[arg(long = "arg")]
    pub args: Vec<String>,
    #[arg(long, default_value = "")]
    pub description: String,
    #[arg(long)]
    pub working_dir: Option<String>,
    #[arg(long = "env")]
    pub env: Vec<String>,
    #[arg(long)]
    pub replace: bool,
}

#[derive(Debug, Args)]
pub struct ExternalCliListArgs {
    #[arg(long, value_enum, default_value_t = CliOutputFormat::Table)]
    pub format: CliOutputFormat,
}

#[derive(Debug, Args)]
pub struct ExternalCliRemoveArgs {
    pub name: String,
}

#[derive(Debug, Args)]
pub struct ExternalCliRunArgs {
    pub name: String,
    #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
    pub args: Vec<String>,
}

#[derive(Debug, Args)]
pub struct RegArgs {
    /// 注册用户名
    #[arg(long)]
    pub username: String,
    /// 注册密码；也可以用 --password-stdin 或 AIO_PASSWORD
    #[arg(long)]
    pub password: Option<String>,
    /// 从 stdin 读取注册密码，避免写进 shell history
    #[arg(long)]
    pub password_stdin: bool,
    /// 昵称，未传时默认等于 username
    #[arg(long, default_value = "")]
    pub nickname: String,
    /// 用户状态，默认 enabled
    #[arg(long, default_value = "enabled")]
    pub status: String,
    /// 给新用户绑定角色 id，可重复传
    #[arg(long = "role-id")]
    pub role_ids: Vec<i32>,
    /// 给新用户绑定内置“管理员”角色
    #[arg(long)]
    pub admin: bool,
}

#[derive(Debug, Args)]
pub struct LoginArgs {
    /// AIO 后台地址，默认读取 AIO_SERVER_URL/AIO_API_URL/AIO_API_BIND 或 http://127.0.0.1:8787
    #[arg(long)]
    pub server: Option<String>,
    /// 直接复用当前机器的 gh 登录态，生成本机 Drive 登录态
    #[arg(long)]
    pub use_gh: bool,
    /// 登录用户名，默认读取 AIO_USERNAME/AIO_ADMIN_USERNAME 或 admin
    #[arg(long)]
    pub username: Option<String>,
    /// 登录密码；未传时读取 AIO_PASSWORD/AIO_ADMIN_PASSWORD，最后回退到本机开发默认 admin
    #[arg(long)]
    pub password: Option<String>,
    /// 从 stdin 读取密码，避免写进 shell history
    #[arg(long)]
    pub password_stdin: bool,
}

#[derive(Debug, Args)]
pub struct AuthServerArgs {
    /// 覆盖登录态中的 AIO 后台地址
    #[arg(long)]
    pub server: Option<String>,
}

#[derive(Debug, Subcommand)]
pub enum KeyCommand {
    /// 为当前登录用户创建 API key
    Create(KeyCreateArgs),
    /// 查看 API key 对应的主人信息
    Whoami(KeyValueArgs),
    /// 添加别人的 API key 作为本机 Drive 融合源
    Add(KeyAddArgs),
    /// 从本机融合源移除 API key
    Remove(KeySelectorArgs),
    /// 撤销当前登录用户创建的 API key
    Revoke(KeySelectorArgs),
    /// 列出本机已添加的融合源
    List,
}

#[derive(Debug, Args)]
pub struct KeyCreateArgs {
    /// API key 标签
    #[arg(long, default_value = "")]
    pub label: String,
}

#[derive(Debug, Args)]
pub struct KeyValueArgs {
    /// API key 原文
    pub api_key: String,
}

#[derive(Debug, Args)]
pub struct KeyAddArgs {
    /// API key 原文
    pub api_key: String,
    /// 本机备注标签
    #[arg(long, default_value = "")]
    pub label: String,
}

#[derive(Debug, Args)]
pub struct KeySelectorArgs {
    /// API key 前缀或 owner 用户名
    pub selector: String,
}

#[derive(Debug, Args)]
pub struct SystemCli {
    #[command(subcommand)]
    pub command: SystemSubcommand,
}

#[derive(Debug, Subcommand)]
pub enum SystemSubcommand {
    /// 输出系统治理模块的提示词、渐进式披露和 CLI 示例
    Docs(SystemDocsArgs),
    /// 用户管理 CRUD 与角色授权
    User(UserCli),
    /// 角色管理 CRUD 与菜单授权
    Role(RoleCli),
    /// 菜单挂载 CRUD
    Menu(MenuCli),
    /// 部门管理 CRUD
    Department(DepartmentCli),
    /// 字典组 CRUD
    DictGroup(DictGroupCli),
    /// 字典项 CRUD
    DictItem(DictItemCli),
}

#[derive(Debug, Args)]
pub struct SystemDocsArgs {
    #[arg(long)]
    pub module: Option<SystemModuleKind>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, ValueEnum)]
pub enum SystemModuleKind {
    Users,
    Roles,
    Navigation,
    Organization,
    Metadata,
    Security,
}

#[derive(Debug, Args)]
pub struct UserCli {
    #[command(subcommand)]
    pub command: UserSubcommand,
}

#[derive(Debug, Subcommand)]
pub enum UserSubcommand {
    List,
    Get(IdArgs),
    Create(UserCreateArgs),
    Update(UserUpdateArgs),
    Delete(IdArgs),
    AuthorizeRoles(UserAuthorizeRolesArgs),
    EffectiveMenus(IdArgs),
}

#[derive(Debug, Args)]
pub struct RoleCli {
    #[command(subcommand)]
    pub command: RoleSubcommand,
}

#[derive(Debug, Subcommand)]
pub enum RoleSubcommand {
    List,
    Get(IdArgs),
    Create(RoleCreateArgs),
    Update(RoleUpdateArgs),
    Delete(IdArgs),
    AuthorizeMenus(RoleAuthorizeMenusArgs),
}

#[derive(Debug, Args)]
pub struct MenuCli {
    #[command(subcommand)]
    pub command: MenuSubcommand,
}

#[derive(Debug, Subcommand)]
pub enum MenuSubcommand {
    List,
    Create(MenuCreateArgs),
    Update(MenuUpdateArgs),
    Delete(IdArgs),
}

#[derive(Debug, Args)]
pub struct DepartmentCli {
    #[command(subcommand)]
    pub command: DepartmentSubcommand,
}

#[derive(Debug, Subcommand)]
pub enum DepartmentSubcommand {
    List,
    Create(DepartmentCreateArgs),
    Update(DepartmentUpdateArgs),
    Delete(IdArgs),
}

#[derive(Debug, Args)]
pub struct DictGroupCli {
    #[command(subcommand)]
    pub command: DictGroupSubcommand,
}

#[derive(Debug, Subcommand)]
pub enum DictGroupSubcommand {
    List,
    Create(DictGroupCreateArgs),
    Update(DictGroupUpdateArgs),
    Delete(IdArgs),
}

#[derive(Debug, Args)]
pub struct DictItemCli {
    #[command(subcommand)]
    pub command: DictItemSubcommand,
}

#[derive(Debug, Subcommand)]
pub enum DictItemSubcommand {
    List(GroupIdArgs),
    Create(DictItemCreateArgs),
    Update(DictItemUpdateArgs),
    Delete(IdArgs),
}

#[derive(Debug, Args)]
pub struct IdArgs {
    #[arg(long)]
    pub id: i32,
}

#[derive(Debug, Args)]
pub struct GroupIdArgs {
    #[arg(long)]
    pub group_id: i32,
}

#[derive(Debug, Args)]
pub struct UserCreateArgs {
    #[arg(long)]
    pub username: String,
    #[arg(long)]
    pub password: String,
    #[arg(long, default_value = "")]
    pub nickname: String,
    #[arg(long, default_value = "active")]
    pub status: String,
    #[arg(long = "role-id")]
    pub role_ids: Vec<i32>,
}

#[derive(Debug, Args)]
pub struct UserUpdateArgs {
    #[arg(long)]
    pub id: i32,
    #[arg(long)]
    pub username: String,
    #[arg(long, default_value = "")]
    pub nickname: String,
    #[arg(long, default_value = "active")]
    pub status: String,
    #[arg(long)]
    pub password: Option<String>,
}

#[derive(Debug, Args)]
pub struct UserAuthorizeRolesArgs {
    #[arg(long)]
    pub id: i32,
    #[arg(long = "role-id")]
    pub role_ids: Vec<i32>,
}

#[derive(Debug, Args)]
pub struct RoleCreateArgs {
    #[arg(long)]
    pub name: String,
    #[arg(long, default_value = "")]
    pub description: String,
    #[arg(long = "menu-id")]
    pub menu_ids: Vec<i32>,
}

#[derive(Debug, Args)]
pub struct RoleUpdateArgs {
    #[arg(long)]
    pub id: i32,
    #[arg(long)]
    pub name: String,
    #[arg(long, default_value = "")]
    pub description: String,
}

#[derive(Debug, Args)]
pub struct RoleAuthorizeMenusArgs {
    #[arg(long)]
    pub id: i32,
    #[arg(long = "menu-id")]
    pub menu_ids: Vec<i32>,
}

#[derive(Debug, Args)]
pub struct MenuCreateArgs {
    #[arg(long)]
    pub name: String,
    #[arg(long, default_value = "")]
    pub route: String,
    #[arg(long, default_value = "")]
    pub icon: String,
    #[arg(long)]
    pub parent_id: Option<i32>,
    #[arg(long, default_value_t = 0)]
    pub sort_order: i32,
    #[arg(long, default_value_t = true)]
    pub visible: bool,
    #[arg(long, default_value = "")]
    pub permission_code: String,
    #[arg(long, default_value = "menu")]
    pub menu_type: String,
}

#[derive(Debug, Args)]
pub struct MenuUpdateArgs {
    #[arg(long)]
    pub id: i32,
    #[arg(long)]
    pub name: String,
    #[arg(long, default_value = "")]
    pub route: String,
    #[arg(long, default_value = "")]
    pub icon: String,
    #[arg(long)]
    pub parent_id: Option<i32>,
    #[arg(long, default_value_t = 0)]
    pub sort_order: i32,
    #[arg(long, default_value_t = true)]
    pub visible: bool,
    #[arg(long, default_value = "")]
    pub permission_code: String,
    #[arg(long, default_value = "menu")]
    pub menu_type: String,
}

#[derive(Debug, Args)]
pub struct DepartmentCreateArgs {
    #[arg(long)]
    pub name: String,
    #[arg(long)]
    pub parent_id: Option<i32>,
    #[arg(long, default_value_t = 0)]
    pub sort_order: i32,
}

#[derive(Debug, Args)]
pub struct DepartmentUpdateArgs {
    #[arg(long)]
    pub id: i32,
    #[arg(long)]
    pub name: String,
    #[arg(long)]
    pub parent_id: Option<i32>,
    #[arg(long, default_value_t = 0)]
    pub sort_order: i32,
}

#[derive(Debug, Args)]
pub struct DictGroupCreateArgs {
    #[arg(long)]
    pub name: String,
    #[arg(long, default_value = "")]
    pub description: String,
}

#[derive(Debug, Args)]
pub struct DictGroupUpdateArgs {
    #[arg(long)]
    pub id: i32,
    #[arg(long)]
    pub name: String,
    #[arg(long, default_value = "")]
    pub description: String,
}

#[derive(Debug, Args)]
pub struct DictItemCreateArgs {
    #[arg(long)]
    pub group_id: i32,
    #[arg(long)]
    pub label: String,
    #[arg(long)]
    pub value: String,
    #[arg(long, default_value_t = 0)]
    pub sort_order: i32,
}

#[derive(Debug, Args)]
pub struct DictItemUpdateArgs {
    #[arg(long)]
    pub id: i32,
    #[arg(long)]
    pub group_id: i32,
    #[arg(long)]
    pub label: String,
    #[arg(long)]
    pub value: String,
    #[arg(long, default_value_t = 0)]
    pub sort_order: i32,
}

#[cfg(test)]
mod tests {
    use clap::CommandFactory;

    use super::Cli;

    #[test]
    fn root_help_should_not_expose_tui_command() {
        let help = Cli::command().render_long_help().to_string();

        assert!(!help.contains("tui"));
    }
}
