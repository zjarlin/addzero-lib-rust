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
    /// 启动 API 后端服务
    Serve,
    /// 运行数据库迁移
    Migrate,
    /// 打印当前架构状态
    Status,
    /// 面向 agent 的系统治理 CLI
    System(SystemCli),
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
