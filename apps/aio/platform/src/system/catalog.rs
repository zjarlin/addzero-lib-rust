//! 系统后台管理域契约。
//!
//! 该模块把天津系统工程中的 auth、dept、dict、menu、logger 等后台能力，
//! 收敛为 admin 可以消费的双轴页面模型。这里不保存业务数据，
//! 只声明正式 PostgreSQL 边界、页面结构和 API/CLI 可共享的操作语义。

use serde::Serialize;

pub const SYSTEM_DOMAIN_ID: &str = "system";
pub const SYSTEM_DOMAIN_LABEL: &str = "管理后台";
pub const SYSTEM_DEFAULT_ROUTE: &str = "/system/identity/users";
pub const SYSTEM_RENDERER_ID: &str = "system.admin.page";
pub const SYSTEM_SIDEBAR_RENDERER_ID: &str = "system.admin.sidebar";

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum SystemFeatureStatus {
    StarterBacked,
    ReferenceOnly,
}

impl SystemFeatureStatus {
    pub fn label(self) -> &'static str {
        match self {
            Self::StarterBacked => "已接入 starter",
            Self::ReferenceOnly => "参考建模",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum SystemFieldKind {
    Text,
    Badge,
    Count,
    Time,
    Route,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
pub struct SystemTableColumn {
    pub key: &'static str,
    pub label: &'static str,
    pub kind: SystemFieldKind,
    pub width: &'static str,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
pub struct SystemTableRow {
    pub cells: &'static [SystemTableCell],
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
pub struct SystemTableCell {
    pub key: &'static str,
    pub value: &'static str,
    pub tone: Option<&'static str>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
pub struct SystemOperation {
    pub id: &'static str,
    pub label: &'static str,
    pub method: &'static str,
    pub path: &'static str,
    pub cli: &'static str,
    pub primary: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
pub struct SystemPage {
    pub id: &'static str,
    pub label: &'static str,
    pub description: &'static str,
    pub route: &'static str,
    pub icon: &'static str,
    pub order: i32,
    pub status: SystemFeatureStatus,
    pub source_modules: &'static [&'static str],
    pub pg_tables: &'static [&'static str],
    pub read_boundary: &'static str,
    pub write_boundary: &'static str,
    pub permissions_any_of: &'static [&'static str],
    pub columns: &'static [SystemTableColumn],
    pub rows: &'static [SystemTableRow],
    pub operations: &'static [SystemOperation],
}

impl SystemPage {
    pub fn is_starter_backed(self) -> bool {
        self.status == SystemFeatureStatus::StarterBacked
    }

    pub fn view(self) -> SystemPageView {
        SystemPageView {
            id: self.id.to_string(),
            label: self.label.to_string(),
            description: self.description.to_string(),
            route: self.route.to_string(),
            icon: self.icon.to_string(),
            order: self.order,
            status: self.status,
            status_label: self.status.label().to_string(),
            source_modules: strings(self.source_modules),
            pg_tables: strings(self.pg_tables),
            read_boundary: self.read_boundary.to_string(),
            write_boundary: self.write_boundary.to_string(),
            permissions_any_of: strings(self.permissions_any_of),
            columns: self.columns.to_vec(),
            rows: self.rows.to_vec(),
            operations: self.operations.to_vec(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct SystemPageView {
    pub id: String,
    pub label: String,
    pub description: String,
    pub route: String,
    pub icon: String,
    pub order: i32,
    pub status: SystemFeatureStatus,
    pub status_label: String,
    pub source_modules: Vec<String>,
    pub pg_tables: Vec<String>,
    pub read_boundary: String,
    pub write_boundary: String,
    pub permissions_any_of: Vec<String>,
    pub columns: Vec<SystemTableColumn>,
    pub rows: Vec<SystemTableRow>,
    pub operations: Vec<SystemOperation>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct SystemDashboardView {
    pub domain_id: String,
    pub label: String,
    pub default_route: String,
    pub pages: Vec<SystemPageView>,
    pub implemented_count: usize,
    pub reference_count: usize,
    pub pg_table_count: usize,
}

pub fn system_pages() -> &'static [SystemPage] {
    SYSTEM_PAGES
}

pub fn starter_backed_system_pages() -> Vec<SystemPage> {
    SYSTEM_PAGES
        .iter()
        .copied()
        .filter(|page| page.is_starter_backed())
        .collect()
}

pub fn system_page_views() -> Vec<SystemPageView> {
    SYSTEM_PAGES
        .iter()
        .copied()
        .map(SystemPage::view)
        .collect()
}

pub fn starter_backed_system_page_views() -> Vec<SystemPageView> {
    starter_backed_system_pages()
        .into_iter()
        .map(SystemPage::view)
        .collect()
}

pub fn system_page_for_route(route: &str) -> Option<SystemPage> {
    SYSTEM_PAGES
        .iter()
        .copied()
        .find(|page| route_matches(page.route, route))
}

pub fn system_dashboard_view() -> SystemDashboardView {
    let pages = system_page_views();
    let implemented_count = pages
        .iter()
        .filter(|page| page.status == SystemFeatureStatus::StarterBacked)
        .count();
    let reference_count = pages.len().saturating_sub(implemented_count);
    let pg_table_count = pages.iter().map(|page| page.pg_tables.len()).sum();

    SystemDashboardView {
        domain_id: SYSTEM_DOMAIN_ID.to_string(),
        label: SYSTEM_DOMAIN_LABEL.to_string(),
        default_route: SYSTEM_DEFAULT_ROUTE.to_string(),
        pages,
        implemented_count,
        reference_count,
        pg_table_count,
    }
}

fn strings(values: &[&str]) -> Vec<String> {
    values.iter().map(|value| (*value).to_string()).collect()
}

fn route_matches(candidate: &str, route: &str) -> bool {
    let route = route.split('?').next().unwrap_or(route);
    candidate == route
}

const USER_COLUMNS: &[SystemTableColumn] = &[
    column("account", "账号", SystemFieldKind::Text, "16rem"),
    column("dept", "部门", SystemFieldKind::Text, "12rem"),
    column("roles", "角色", SystemFieldKind::Badge, "14rem"),
    column("status", "状态", SystemFieldKind::Badge, "8rem"),
    column("login_at", "最近登录", SystemFieldKind::Time, "12rem"),
];

const USER_ROWS: &[SystemTableRow] = &[
    row(&[
        cell("account", "admin@addzero.local", None),
        cell("dept", "平台中心", None),
        cell("roles", "超级管理员", Some("accent")),
        cell("status", "启用", Some("success")),
        cell("login_at", "2026-06-20 09:18", None),
    ]),
    row(&[
        cell("account", "ops@addzero.local", None),
        cell("dept", "运维中心", None),
        cell("roles", "系统审计员", Some("neutral")),
        cell("status", "启用", Some("success")),
        cell("login_at", "2026-06-19 21:04", None),
    ]),
    row(&[
        cell("account", "disabled@addzero.local", None),
        cell("dept", "外部协作", None),
        cell("roles", "只读访客", Some("neutral")),
        cell("status", "停用", Some("warning")),
        cell("login_at", "2026-06-12 16:43", None),
    ]),
];

const ROLE_COLUMNS: &[SystemTableColumn] = &[
    column("name", "角色名称", SystemFieldKind::Text, "14rem"),
    column("code", "角色标识", SystemFieldKind::Text, "14rem"),
    column("scope", "数据权限", SystemFieldKind::Badge, "12rem"),
    column("users", "绑定用户", SystemFieldKind::Count, "8rem"),
    column("status", "状态", SystemFieldKind::Badge, "8rem"),
];

const ROLE_ROWS: &[SystemTableRow] = &[
    row(&[
        cell("name", "超级管理员", None),
        cell("code", "super_admin", None),
        cell("scope", "全部数据", Some("accent")),
        cell("users", "1", None),
        cell("status", "启用", Some("success")),
    ]),
    row(&[
        cell("name", "系统审计员", None),
        cell("code", "system_auditor", None),
        cell("scope", "审计数据", Some("neutral")),
        cell("users", "2", None),
        cell("status", "启用", Some("success")),
    ]),
    row(&[
        cell("name", "只读访客", None),
        cell("code", "readonly_guest", None),
        cell("scope", "本人数据", Some("neutral")),
        cell("users", "5", None),
        cell("status", "停用", Some("warning")),
    ]),
];

const ORGANIZATION_COLUMNS: &[SystemTableColumn] = &[
    column("name", "组织节点", SystemFieldKind::Text, "16rem"),
    column("leader", "负责人", SystemFieldKind::Text, "10rem"),
    column("posts", "岗位", SystemFieldKind::Count, "8rem"),
    column("users", "成员", SystemFieldKind::Count, "8rem"),
    column("status", "状态", SystemFieldKind::Badge, "8rem"),
];

const ORGANIZATION_ROWS: &[SystemTableRow] = &[
    row(&[
        cell("name", "AddZero / 平台中心", None),
        cell("leader", "Admin", None),
        cell("posts", "4", None),
        cell("users", "18", None),
        cell("status", "启用", Some("success")),
    ]),
    row(&[
        cell("name", "AddZero / 运维中心", None),
        cell("leader", "Ops", None),
        cell("posts", "3", None),
        cell("users", "9", None),
        cell("status", "启用", Some("success")),
    ]),
    row(&[
        cell("name", "AddZero / 外部协作", None),
        cell("leader", "Partner", None),
        cell("posts", "1", None),
        cell("users", "5", None),
        cell("status", "归档", Some("warning")),
    ]),
];

const DICTIONARY_COLUMNS: &[SystemTableColumn] = &[
    column("name", "字典类型", SystemFieldKind::Text, "16rem"),
    column("code", "编码", SystemFieldKind::Text, "16rem"),
    column("items", "条目", SystemFieldKind::Count, "8rem"),
    column("scope", "作用域", SystemFieldKind::Badge, "10rem"),
    column("updated_at", "更新时间", SystemFieldKind::Time, "12rem"),
];

const DICTIONARY_ROWS: &[SystemTableRow] = &[
    row(&[
        cell("name", "用户状态", None),
        cell("code", "system_user_status", None),
        cell("items", "3", None),
        cell("scope", "系统", Some("accent")),
        cell("updated_at", "2026-06-18 12:20", None),
    ]),
    row(&[
        cell("name", "通知级别", None),
        cell("code", "system_notice_level", None),
        cell("items", "4", None),
        cell("scope", "后台", Some("neutral")),
        cell("updated_at", "2026-06-17 19:10", None),
    ]),
    row(&[
        cell("name", "登录结果", None),
        cell("code", "system_login_result", None),
        cell("items", "5", None),
        cell("scope", "审计", Some("neutral")),
        cell("updated_at", "2026-06-16 08:42", None),
    ]),
];

const MENU_COLUMNS: &[SystemTableColumn] = &[
    column("label", "菜单节点", SystemFieldKind::Text, "18rem"),
    column("route", "路由", SystemFieldKind::Route, "18rem"),
    column("permission", "权限", SystemFieldKind::Text, "16rem"),
    column("kind", "类型", SystemFieldKind::Badge, "8rem"),
    column("status", "状态", SystemFieldKind::Badge, "8rem"),
];

const MENU_ROWS: &[SystemTableRow] = &[
    row(&[
        cell("label", "管理后台 / 用户管理", None),
        cell("route", "/system/identity/users", None),
        cell("permission", "system:user", None),
        cell("kind", "页面", Some("accent")),
        cell("status", "挂载", Some("success")),
    ]),
    row(&[
        cell("label", "管理后台 / 菜单挂载", None),
        cell("route", "/system/menu/mounting", None),
        cell("permission", "system:menu", None),
        cell("kind", "页面", Some("accent")),
        cell("status", "挂载", Some("success")),
    ]),
    row(&[
        cell("label", "低代码工作台", None),
        cell("route", "/lowcode", None),
        cell("permission", "plugin:lowcode", None),
        cell("kind", "插件", Some("neutral")),
        cell("status", "托管", Some("success")),
    ]),
];

const AUDIT_COLUMNS: &[SystemTableColumn] = &[
    column("event", "事件", SystemFieldKind::Text, "16rem"),
    column("actor", "操作者", SystemFieldKind::Text, "12rem"),
    column("target", "对象", SystemFieldKind::Text, "16rem"),
    column("result", "结果", SystemFieldKind::Badge, "8rem"),
    column("created_at", "发生时间", SystemFieldKind::Time, "12rem"),
];

const AUDIT_ROWS: &[SystemTableRow] = &[
    row(&[
        cell("event", "用户登录", None),
        cell("actor", "admin@addzero.local", None),
        cell("target", "127.0.0.1", None),
        cell("result", "成功", Some("success")),
        cell("created_at", "2026-06-20 09:18", None),
    ]),
    row(&[
        cell("event", "角色授权", None),
        cell("actor", "admin@addzero.local", None),
        cell("target", "系统审计员", None),
        cell("result", "成功", Some("success")),
        cell("created_at", "2026-06-19 20:11", None),
    ]),
    row(&[
        cell("event", "密码校验", None),
        cell("actor", "disabled@addzero.local", None),
        cell("target", "后台登录", None),
        cell("result", "拒绝", Some("warning")),
        cell("created_at", "2026-06-19 19:47", None),
    ]),
];

const AUTH_COLUMNS: &[SystemTableColumn] = &[
    column("flow", "认证流", SystemFieldKind::Text, "16rem"),
    column("entry", "入口", SystemFieldKind::Route, "16rem"),
    column("token", "令牌模型", SystemFieldKind::Text, "16rem"),
    column("status", "状态", SystemFieldKind::Badge, "8rem"),
];

const AUTH_ROWS: &[SystemTableRow] = &[
    row(&[
        cell("flow", "账号密码登录", None),
        cell("entry", "/admin-api/system/auth/login", None),
        cell("token", "OAuth2AccessTokenDO", None),
        cell("status", "待接入", Some("warning")),
    ]),
    row(&[
        cell("flow", "短信登录", None),
        cell("entry", "/admin-api/system/auth/sms-login", None),
        cell("token", "OAuth2RefreshTokenDO", None),
        cell("status", "参考", Some("neutral")),
    ]),
];

const API_KEY_COLUMNS: &[SystemTableColumn] = &[
    column("name", "密钥名称", SystemFieldKind::Text, "16rem"),
    column("prefix", "密钥前缀", SystemFieldKind::Text, "16rem"),
    column("scope", "授权范围", SystemFieldKind::Badge, "12rem"),
    column("status", "状态", SystemFieldKind::Badge, "8rem"),
    column("last_used_at", "最近使用", SystemFieldKind::Time, "12rem"),
];

const API_KEY_ROWS: &[SystemTableRow] = &[
    row(&[
        cell("name", "在线创建后实时加载", None),
        cell("prefix", "az_live_********", None),
        cell("scope", "all-services", Some("accent")),
        cell("status", "active", Some("success")),
        cell("last_used_at", "由 PostgreSQL 记录", None),
    ]),
];

const TENANT_COLUMNS: &[SystemTableColumn] = &[
    column("tenant", "租户", SystemFieldKind::Text, "16rem"),
    column("package", "套餐", SystemFieldKind::Text, "14rem"),
    column("users", "用户数", SystemFieldKind::Count, "8rem"),
    column("status", "状态", SystemFieldKind::Badge, "8rem"),
];

const TENANT_ROWS: &[SystemTableRow] = &[
    row(&[
        cell("tenant", "AddZero 默认租户", None),
        cell("package", "平台版", None),
        cell("users", "32", None),
        cell("status", "参考", Some("neutral")),
    ]),
    row(&[
        cell("tenant", "IoT 演示租户", None),
        cell("package", "设备版", None),
        cell("users", "12", None),
        cell("status", "参考", Some("neutral")),
    ]),
];

const MESSAGING_COLUMNS: &[SystemTableColumn] = &[
    column("template", "模板", SystemFieldKind::Text, "16rem"),
    column("channel", "通道", SystemFieldKind::Badge, "10rem"),
    column("sent", "发送量", SystemFieldKind::Count, "8rem"),
    column("status", "状态", SystemFieldKind::Badge, "8rem"),
];

const MESSAGING_ROWS: &[SystemTableRow] = &[
    row(&[
        cell("template", "登录验证码", None),
        cell("channel", "SMS", Some("accent")),
        cell("sent", "128", None),
        cell("status", "参考", Some("neutral")),
    ]),
    row(&[
        cell("template", "系统公告", None),
        cell("channel", "Notify", Some("accent")),
        cell("sent", "44", None),
        cell("status", "参考", Some("neutral")),
    ]),
];

const OAUTH2_COLUMNS: &[SystemTableColumn] = &[
    column("client", "客户端", SystemFieldKind::Text, "16rem"),
    column("grant", "授权模式", SystemFieldKind::Badge, "12rem"),
    column("redirect", "回调", SystemFieldKind::Route, "18rem"),
    column("status", "状态", SystemFieldKind::Badge, "8rem"),
];

const OAUTH2_ROWS: &[SystemTableRow] = &[
    row(&[
        cell("client", "az-aio-web", None),
        cell("grant", "authorization_code", Some("accent")),
        cell("redirect", "/oauth2/callback", None),
        cell("status", "参考", Some("neutral")),
    ]),
    row(&[
        cell("client", "az-cli", None),
        cell("grant", "device_code", Some("neutral")),
        cell("redirect", "urn:ietf:wg:oauth:2.0:oob", None),
        cell("status", "参考", Some("neutral")),
    ]),
];

const SOCIAL_COLUMNS: &[SystemTableColumn] = &[
    column("client", "社交客户端", SystemFieldKind::Text, "16rem"),
    column("provider", "平台", SystemFieldKind::Badge, "10rem"),
    column("bindings", "绑定数", SystemFieldKind::Count, "8rem"),
    column("status", "状态", SystemFieldKind::Badge, "8rem"),
];

const SOCIAL_ROWS: &[SystemTableRow] = &[
    row(&[
        cell("client", "WeChat JSAPI", None),
        cell("provider", "微信", Some("accent")),
        cell("bindings", "0", None),
        cell("status", "参考", Some("neutral")),
    ]),
    row(&[
        cell("client", "GitHub OAuth", None),
        cell("provider", "GitHub", Some("neutral")),
        cell("bindings", "0", None),
        cell("status", "参考", Some("neutral")),
    ]),
];

const AREA_COLUMNS: &[SystemTableColumn] = &[
    column("node", "地区节点", SystemFieldKind::Text, "16rem"),
    column("code", "编码", SystemFieldKind::Text, "10rem"),
    column("children", "子节点", SystemFieldKind::Count, "8rem"),
    column("status", "状态", SystemFieldKind::Badge, "8rem"),
];

const AREA_ROWS: &[SystemTableRow] = &[
    row(&[
        cell("node", "中国 / 天津市", None),
        cell("code", "120000", None),
        cell("children", "16", None),
        cell("status", "参考", Some("neutral")),
    ]),
    row(&[
        cell("node", "中国 / 广东省", None),
        cell("code", "440000", None),
        cell("children", "21", None),
        cell("status", "参考", Some("neutral")),
    ]),
];

const SYSTEM_PAGES: &[SystemPage] = &[
    SystemPage {
        id: "api_keys",
        label: "API 密钥",
        description: "为当前账号创建可撤销的 API Key，外部调用方可用 api_key 访问已暴露服务。",
        route: "/system/account/api-keys",
        icon: "⚿",
        order: 5,
        status: SystemFeatureStatus::StarterBacked,
        source_modules: &["auth", "api-key", "gateway"],
        pg_tables: &["biz_system_admin_system_api_key_records"],
        read_boundary: "只读取密钥元数据、前缀、状态和最近使用时间，永不读取明文密钥。",
        write_boundary: "创建时只返回一次明文 api_key，PostgreSQL 仅保存 SHA-256 哈希；撤销后立即拒绝显式 api_key 调用。",
        permissions_any_of: &["system:api-key"],
        columns: API_KEY_COLUMNS,
        rows: API_KEY_ROWS,
        operations: &[],
    },
    SystemPage {
        id: "identity",
        label: "用户管理",
        description: "后台用户、登录身份和用户角色绑定。",
        route: "/system/identity/users",
        icon: "◉",
        order: 10,
        status: SystemFeatureStatus::StarterBacked,
        source_modules: &["auth", "user"],
        pg_tables: &["sys_user", "sys_user_role"],
        read_boundary: "查询用户、登录态和角色绑定摘要，只读视图默认按租户与权限过滤。",
        write_boundary: "新增或修改用户必须同步角色绑定、密码策略和审计日志。",
        permissions_any_of: &["system:user"],
        columns: USER_COLUMNS,
        rows: USER_ROWS,
        operations: &[
            operation("system.identity.create", "新建用户", "POST", "/api/system/users", "az system user create", true),
            operation("system.identity.export", "导出用户", "GET", "/api/system/users/export", "az system user export", false),
        ],
    },
    SystemPage {
        id: "role",
        label: "角色管理",
        description: "角色、权限点、角色菜单授权和数据权限范围。",
        route: "/system/permission/roles",
        icon: "◍",
        order: 15,
        status: SystemFeatureStatus::StarterBacked,
        source_modules: &["permission", "role"],
        pg_tables: &["sys_role", "sys_role_menu", "sys_user_role"],
        read_boundary: "读取角色、角色菜单授权和用户角色绑定。",
        write_boundary: "写入角色时校验角色标识唯一性、数据范围和菜单权限影响面。",
        permissions_any_of: &["system:role"],
        columns: ROLE_COLUMNS,
        rows: ROLE_ROWS,
        operations: &[
            operation("system.role.create", "新建角色", "POST", "/api/system/roles", "az system role create", true),
            operation("system.role.grant", "分配权限", "POST", "/api/system/roles/grant", "az system role grant", false),
        ],
    },
    SystemPage {
        id: "organization",
        label: "部门管理",
        description: "部门树、岗位和用户组织归属。",
        route: "/system/organization/departments",
        icon: "◎",
        order: 20,
        status: SystemFeatureStatus::StarterBacked,
        source_modules: &["dept", "post"],
        pg_tables: &["sys_dept", "sys_post", "sys_user_post"],
        read_boundary: "读取部门树、岗位列表和用户岗位绑定。",
        write_boundary: "写入部门或岗位时校验父子关系、排序和停用影响面。",
        permissions_any_of: &["system:dept"],
        columns: ORGANIZATION_COLUMNS,
        rows: ORGANIZATION_ROWS,
        operations: &[
            operation("system.organization.create", "新建部门", "POST", "/api/system/departments", "az system dept create", true),
            operation("system.organization.sync", "同步组织树", "POST", "/api/system/departments/sync", "az system dept sync", false),
        ],
    },
    SystemPage {
        id: "dictionary",
        label: "字典管理",
        description: "后台字典类型、字典值和枚举常量。",
        route: "/system/dictionary/note-types",
        icon: "▤",
        order: 30,
        status: SystemFeatureStatus::StarterBacked,
        source_modules: &["dict"],
        pg_tables: &["sys_dict_type", "sys_dict_data"],
        read_boundary: "读取字典类型、值列表和可被前端缓存的简单字典。",
        write_boundary: "写入字典值时同步编码唯一性、状态和缓存失效事件。",
        permissions_any_of: &["system:dict"],
        columns: DICTIONARY_COLUMNS,
        rows: DICTIONARY_ROWS,
        operations: &[
            operation("system.dictionary.type.list", "查询字典类型", "GET", "/api/system/dictionary-types", "az system dictionary-type list", false),
            operation("system.dictionary.type.create", "新建字典类型", "POST", "/api/system/dictionary-types", "az system dictionary-type create", true),
            operation("system.dictionary.type.update", "更新字典类型", "PUT", "/api/system/dictionary-types/{id}", "az system dictionary-type update --id <id>", false),
            operation("system.dictionary.type.delete", "删除字典类型", "DELETE", "/api/system/dictionary-types/{id}", "az system dictionary-type delete --id <id>", false),
            operation("system.dictionary.item.create", "新建字典项", "POST", "/api/system/dictionary-items", "az system dictionary-item create", true),
            operation("system.dictionary.item.update", "更新字典项", "PUT", "/api/system/dictionary-items/{id}", "az system dictionary-item update --id <id>", false),
            operation("system.dictionary.item.delete", "删除字典项", "DELETE", "/api/system/dictionary-items/{id}", "az system dictionary-item delete --id <id>", false),
        ],
    },
    SystemPage {
        id: "menu",
        label: "菜单挂载",
        description: "后台菜单树、权限点和插件路由挂载。",
        route: "/system/menu/mounting",
        icon: "☰",
        order: 40,
        status: SystemFeatureStatus::StarterBacked,
        source_modules: &["permission", "menu"],
        pg_tables: &["sys_menu", "sys_role_menu"],
        read_boundary: "读取双轴上下文树、角色菜单授权和插件贡献路由。",
        write_boundary: "写入菜单时校验路由唯一性、权限编码和角色影响范围。",
        permissions_any_of: &["system:menu"],
        columns: MENU_COLUMNS,
        rows: MENU_ROWS,
        operations: &[
            operation("system.menu.mount", "挂载菜单", "POST", "/api/system/menus", "az system menu mount", true),
            operation("system.menu.audit", "检查权限", "GET", "/api/system/menus/audit", "az system menu audit", false),
        ],
    },
    SystemPage {
        id: "audit",
        label: "审计日志",
        description: "登录日志、操作日志和后台追踪证据。",
        route: "/system/audit/events",
        icon: "◷",
        order: 50,
        status: SystemFeatureStatus::StarterBacked,
        source_modules: &["logger"],
        pg_tables: &["sys_login_log", "sys_operate_log"],
        read_boundary: "按时间、操作者、结果和业务对象读取审计日志。",
        write_boundary: "业务操作只追加日志，不允许后台页面直接修改历史记录。",
        permissions_any_of: &["system:audit"],
        columns: AUDIT_COLUMNS,
        rows: AUDIT_ROWS,
        operations: &[
            operation("system.audit.search", "检索日志", "GET", "/api/system/audit/events", "az system audit search", true),
            operation("system.audit.export", "导出审计", "GET", "/api/system/audit/export", "az system audit export", false),
        ],
    },
    SystemPage {
        id: "auth",
        label: "认证中心",
        description: "登录、注册、重置密码、验证码和 token 生命周期。",
        route: "/system/auth/sessions",
        icon: "●",
        order: 60,
        status: SystemFeatureStatus::ReferenceOnly,
        source_modules: &["auth", "oauth2", "security"],
        pg_tables: &["sys_oauth2_access_token", "sys_oauth2_refresh_token"],
        read_boundary: "读取会话、token 和认证失败摘要。",
        write_boundary: "正式接入时所有 token 写入 PostgreSQL，验证码缓存只作为短期状态。",
        permissions_any_of: &["system:auth"],
        columns: AUTH_COLUMNS,
        rows: AUTH_ROWS,
        operations: &[
            operation("system.auth.revoke", "撤销会话", "POST", "/api/system/auth/revoke", "az system auth revoke", true),
            operation("system.auth.policy", "校验策略", "GET", "/api/system/auth/policy", "az system auth policy", false),
        ],
    },
    SystemPage {
        id: "tenant",
        label: "租户管理",
        description: "租户、租户套餐和多租户隔离策略。",
        route: "/system/tenant/tenants",
        icon: "▥",
        order: 70,
        status: SystemFeatureStatus::ReferenceOnly,
        source_modules: &["tenant"],
        pg_tables: &["sys_tenant", "sys_tenant_package"],
        read_boundary: "读取租户、套餐和容量限制。",
        write_boundary: "正式接入时租户、套餐、租户域名和过期策略都落 PostgreSQL。",
        permissions_any_of: &["system:tenant"],
        columns: TENANT_COLUMNS,
        rows: TENANT_ROWS,
        operations: &[
            operation("system.tenant.create", "新建租户", "POST", "/api/system/tenants", "az system tenant create", true),
            operation("system.tenant.package", "维护套餐", "POST", "/api/system/tenant-packages", "az system tenant package", false),
        ],
    },
    SystemPage {
        id: "messaging",
        label: "消息中心",
        description: "站内信、邮件、短信模板、发送日志和通知触达。",
        route: "/system/messaging/templates",
        icon: "✉",
        order: 80,
        status: SystemFeatureStatus::ReferenceOnly,
        source_modules: &["mail", "notify", "sms", "notice"],
        pg_tables: &[
            "sys_mail_account",
            "sys_mail_template",
            "sys_mail_log",
            "sys_notify_template",
            "sys_notify_message",
            "sys_sms_channel",
            "sys_sms_template",
            "sys_sms_log",
        ],
        read_boundary: "读取模板、通道、通知记录和发送日志。",
        write_boundary: "模板、通道和发送记录必须进入 PostgreSQL，队列只保留投递态。",
        permissions_any_of: &["system:message"],
        columns: MESSAGING_COLUMNS,
        rows: MESSAGING_ROWS,
        operations: &[
            operation("system.messaging.create", "新建模板", "POST", "/api/system/messages/templates", "az system message template", true),
            operation("system.messaging.test", "测试发送", "POST", "/api/system/messages/test", "az system message test", false),
        ],
    },
    SystemPage {
        id: "oauth2",
        label: "OAuth2",
        description: "OAuth2 客户端、授权码、批准记录和开放用户信息。",
        route: "/system/oauth2/clients",
        icon: "◇",
        order: 90,
        status: SystemFeatureStatus::ReferenceOnly,
        source_modules: &["oauth2"],
        pg_tables: &[
            "sys_oauth2_client",
            "sys_oauth2_approve",
            "sys_oauth2_code",
            "sys_oauth2_access_token",
            "sys_oauth2_refresh_token",
        ],
        read_boundary: "读取客户端、授权记录、授权码和 token 摘要。",
        write_boundary: "客户端密钥、授权码和 token 写入 PostgreSQL 并补审计。",
        permissions_any_of: &["system:oauth2"],
        columns: OAUTH2_COLUMNS,
        rows: OAUTH2_ROWS,
        operations: &[
            operation("system.oauth2.create", "新建客户端", "POST", "/api/system/oauth2/clients", "az system oauth2 create", true),
            operation("system.oauth2.rotate", "轮换密钥", "POST", "/api/system/oauth2/clients/rotate", "az system oauth2 rotate", false),
        ],
    },
    SystemPage {
        id: "social",
        label: "社交集成",
        description: "社交客户端、用户绑定、微信 JSAPI 和二维码入口。",
        route: "/system/social/clients",
        icon: "◌",
        order: 100,
        status: SystemFeatureStatus::ReferenceOnly,
        source_modules: &["social"],
        pg_tables: &["sys_social_client", "sys_social_user", "sys_social_user_bind"],
        read_boundary: "读取社交客户端、外部用户和本地用户绑定。",
        write_boundary: "绑定关系、客户端配置和授权回调记录正式落 PostgreSQL。",
        permissions_any_of: &["system:social"],
        columns: SOCIAL_COLUMNS,
        rows: SOCIAL_ROWS,
        operations: &[
            operation("system.social.create", "新建客户端", "POST", "/api/system/social/clients", "az system social create", true),
            operation("system.social.rebind", "重新绑定", "POST", "/api/system/social/rebind", "az system social rebind", false),
        ],
    },
    SystemPage {
        id: "area",
        label: "地区数据",
        description: "行政区划树和 IP 归属地查询。",
        route: "/system/area/tree",
        icon: "⌖",
        order: 110,
        status: SystemFeatureStatus::ReferenceOnly,
        source_modules: &["ip", "area"],
        pg_tables: &["sys_area", "sys_ip_location"],
        read_boundary: "读取地区树、地区编码和 IP 归属地缓存。",
        write_boundary: "地区数据导入和 IP 库更新进入 PostgreSQL，文件只作为导入源。",
        permissions_any_of: &["system:area"],
        columns: AREA_COLUMNS,
        rows: AREA_ROWS,
        operations: &[
            operation("system.area.import", "导入地区", "POST", "/api/system/areas/import", "az system area import", true),
            operation("system.area.lookup", "查询 IP", "GET", "/api/system/areas/ip", "az system area lookup", false),
        ],
    },
];

const fn column(
    key: &'static str,
    label: &'static str,
    kind: SystemFieldKind,
    width: &'static str,
) -> SystemTableColumn {
    SystemTableColumn {
        key,
        label,
        kind,
        width,
    }
}

const fn row(cells: &'static [SystemTableCell]) -> SystemTableRow {
    SystemTableRow { cells }
}

const fn cell(
    key: &'static str,
    value: &'static str,
    tone: Option<&'static str>,
) -> SystemTableCell {
    SystemTableCell { key, value, tone }
}

const fn operation(
    id: &'static str,
    label: &'static str,
    method: &'static str,
    path: &'static str,
    cli: &'static str,
    primary: bool,
) -> SystemOperation {
    SystemOperation {
        id,
        label,
        method,
        path,
        cli,
        primary,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn catalog_keeps_tianjin_system_slices_visible_as_reference() {
        let ids = system_pages().iter().map(|page| page.id).collect::<Vec<_>>();

        assert!(ids.contains(&"identity"));
        assert!(ids.contains(&"api_keys"));
        assert!(ids.contains(&"role"));
        assert!(ids.contains(&"organization"));
        assert!(ids.contains(&"dictionary"));
        assert!(ids.contains(&"menu"));
        assert!(ids.contains(&"audit"));
        assert!(ids.contains(&"auth"));
        assert!(ids.contains(&"messaging"));
        assert!(ids.contains(&"oauth2"));
        assert!(ids.contains(&"tenant"));
        assert!(ids.contains(&"social"));
    }

    #[test]
    fn visible_pages_are_limited_to_current_starter_backed_set() {
        let visible_ids = starter_backed_system_pages()
            .iter()
            .map(|page| page.id)
            .collect::<Vec<_>>();

        assert_eq!(
            visible_ids,
            vec![
                "api_keys",
                "identity",
                "role",
                "organization",
                "dictionary",
                "menu",
                "audit"
            ]
        );
    }

    #[test]
    fn every_system_page_declares_pg_boundary_and_operation_contract() {
        for page in system_pages() {
            assert!(!page.pg_tables.is_empty());
            assert!(!page.read_boundary.is_empty());
            assert!(!page.write_boundary.is_empty());

            // 关键断言：API 与 CLI 必须来自同一套操作定义。
            assert!(page.operations.iter().all(|operation| {
                !operation.path.is_empty() && !operation.cli.is_empty()
            }));
        }
    }

    #[test]
    fn route_lookup_ignores_query_string() {
        let page = system_page_for_route("/system/identity/users?tab=roles");

        assert_eq!(page.map(|page| page.id), Some("identity"));
    }
}
