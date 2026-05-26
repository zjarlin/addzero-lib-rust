//! 系统插件统一链接入口，聚合所有 addzero 系统级 starter 插件。
//!
//! 本 crate 是 addzero 系统插件体系的顶层聚合器，提供 [`link_all`] 函数
//! 将所有系统级 starter 插件一次性链接，确保链接器不会剥离任何注册入口。
//!
//! ## 包含的系统插件
//!
//! | 插件 | 职责 |
//! |------|------|
//! | `az-starter-identity` | 用户、角色与登录管理 |
//! | `az-starter-organization` | 部门结构、团队归属与责任人 |
//! | `az-starter-dictionary` | 数据字典与枚举常量管理 |
//! | `az-starter-menu` | 系统菜单配置与路由管理 |
//! | `az-starter-audit` | 操作日志与审计追踪 |
//! | `az-starter-storage` | 上传下载与插件包仓库 |
//!
//! ## 用法
//!
//! 宿主应用在 `main` 函数中调用 [`link_all`] 即可完成所有系统插件的注册：
//!
//! ```rust
//! fn main() {
//!     az_system_starters::link_all();
//!     // 此后 admin 注册中心可发现所有系统级导航节点
//! }
//! ```

const SYSTEM_DOMAIN_ID: &str = "system";

az_admin_plugin_registry::register_admin_domain! {
    id: SYSTEM_DOMAIN_ID,
    label: "系统插件",
    order: 30,
    default_href: "/system/identity/users",
}

pub fn link_all() {
    az_starter_identity::ensure_linked();
    az_starter_organization::ensure_linked();
    az_starter_dictionary::ensure_linked();
    az_starter_menu::ensure_linked();
    az_starter_audit::ensure_linked();
    az_starter_storage::ensure_linked();
}

#[cfg(test)]
mod tests {
    use az_admin_plugin_registry::{registered_domains, section_for_path};

    #[test]
    fn link_all_exposes_system_starter_navigation() {
        super::link_all();

        let system_domain = registered_domains()
            .into_iter()
            .find(|domain| domain.id == super::SYSTEM_DOMAIN_ID)
            .expect("system domain should be registered");
        assert_eq!(system_domain.label, "系统插件");
        assert_eq!(system_domain.default_href, "/system/identity/users");

        let section =
            section_for_path("/system/audit/events").expect("audit route should resolve section");
        let labels = section
            .menus
            .iter()
            .map(|node| node.label)
            .collect::<Vec<_>>();

        // Verifies the aggregate starter still exposes every system entry in menu order.
        assert_eq!(
            labels,
            vec![
                "用户管理",
                "部门管理",
                "字典管理",
                "菜单挂载",
                "审计日志",
                "包仓库"
            ]
        );
    }
}
