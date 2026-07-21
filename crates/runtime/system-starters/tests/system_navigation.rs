use az_admin_plugin_registry::navigation::{registered_domains, section_for_path};

#[test]
fn link_all_exposes_system_starter_navigation() {
    az_system_starters::linking::link_all();

    let system_domain = registered_domains()
        .into_iter()
        .find(|domain| domain.id == "system")
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
