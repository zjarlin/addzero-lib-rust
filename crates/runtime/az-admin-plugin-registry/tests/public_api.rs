use az_admin_plugin_registry::api::{
    AdminNavigationKind, path_matches_patterns, registered_domains, section_for_path,
};

const OVERVIEW_DOMAIN_ID: &str = "test-overview";
const SYSTEM_DOMAIN_ID: &str = "test-system";
const KNOWLEDGE_DOMAIN_ID: &str = "test-knowledge";
const CLI_MARKET_NODE_ID: &str = "test-cli-market";
const SETTINGS_NODE_ID: &str = "test-system-settings";

az_admin_plugin_registry::register_admin_domain! {
    id: KNOWLEDGE_DOMAIN_ID,
    label: "Knowledge",
    order: 20,
    default_href: "/knowledge/notes",
}

az_admin_plugin_registry::register_admin_domain! {
    id: OVERVIEW_DOMAIN_ID,
    label: "Overview",
    order: 10,
    default_href: "/",
}

az_admin_plugin_registry::register_admin_domain! {
    id: SYSTEM_DOMAIN_ID,
    label: "System",
    order: 30,
    default_href: "/system/users",
}

az_admin_plugin_registry::register_admin_page! {
    id: "test-home",
    domain: OVERVIEW_DOMAIN_ID,
    parent: None,
    label: "Home",
    order: 10,
    href: "/",
    active_patterns: &["/", "/dashboard"],
    permissions_any_of: &[],
}

az_admin_plugin_registry::register_admin_page! {
    id: "test-notes",
    domain: KNOWLEDGE_DOMAIN_ID,
    parent: None,
    label: "Notes",
    order: 10,
    href: "/knowledge/notes",
    active_patterns: &["/knowledge/notes"],
    permissions_any_of: &["knowledge:note"],
}

az_admin_plugin_registry::register_admin_branch! {
    id: CLI_MARKET_NODE_ID,
    domain: KNOWLEDGE_DOMAIN_ID,
    parent: None,
    label: "CLI Market",
    order: 20,
    href: "/knowledge/cli-market",
    active_patterns: &[
        "/knowledge/cli-market",
        "/knowledge/cli-market/imports",
        "/knowledge/cli-market/docs",
    ],
    permissions_any_of: &["knowledge:cli"],
}

az_admin_plugin_registry::register_admin_page! {
    id: "test-cli-market-imports",
    domain: KNOWLEDGE_DOMAIN_ID,
    parent: Some(CLI_MARKET_NODE_ID),
    label: "Imports",
    order: 10,
    href: "/knowledge/cli-market/imports",
    active_patterns: &["/knowledge/cli-market/imports"],
    permissions_any_of: &["knowledge:cli"],
}

az_admin_plugin_registry::register_admin_page! {
    id: "test-cli-market-docs",
    domain: KNOWLEDGE_DOMAIN_ID,
    parent: Some(CLI_MARKET_NODE_ID),
    label: "Docs",
    order: 20,
    href: "/knowledge/cli-market/docs",
    active_patterns: &["/knowledge/cli-market/docs"],
    permissions_any_of: &["knowledge:cli"],
}

az_admin_plugin_registry::register_admin_page! {
    id: "test-system-users",
    domain: SYSTEM_DOMAIN_ID,
    parent: None,
    label: "Users",
    order: 10,
    href: "/system/users",
    active_patterns: &["/system/users/:id", "/system/users"],
    permissions_any_of: &["system:user"],
}

az_admin_plugin_registry::declare_admin_root_page_plugin! {
    id: SETTINGS_NODE_ID,
    domain: SYSTEM_DOMAIN_ID,
    label: "Settings",
    order: 20,
    href: "/system/settings",
}

#[test]
fn path_matching_should_support_dynamic_segments() {
    assert!(path_matches_patterns("/agents/demo", &["/agents/:name"]));
    assert!(path_matches_patterns(
        "/api/admin/system/users/42",
        &["/api/admin/system/users/:id"]
    ));
    assert!(!path_matches_patterns(
        "/agents/demo/edit",
        &["/agents/:name"]
    ));
}

#[test]
fn path_matching_should_strip_query_and_trailing_slash() {
    assert!(path_matches_patterns("/files/?tab=recent", &["/files"]));
    assert!(path_matches_patterns(" / ", &["/"]));
}

#[test]
fn admin_navigation_kind_should_expose_stable_codes() {
    assert_eq!(
        AdminNavigationKind::ALL,
        &[AdminNavigationKind::Branch, AdminNavigationKind::Page]
    );
    assert_eq!(AdminNavigationKind::Branch.code(), "branch");
    assert_eq!(
        AdminNavigationKind::from_code("page"),
        Some(AdminNavigationKind::Page)
    );
}

#[test]
fn registered_domains_should_follow_order_and_drop_empty_domains() {
    let ids: Vec<_> = registered_domains()
        .into_iter()
        .map(|domain| domain.id)
        .collect();

    assert_eq!(
        ids,
        vec![OVERVIEW_DOMAIN_ID, KNOWLEDGE_DOMAIN_ID, SYSTEM_DOMAIN_ID]
    );
}

#[test]
fn section_for_path_should_build_tree_and_preserve_permissions() {
    let section =
        section_for_path("/knowledge/cli-market/imports").expect("cli market section should exist");
    let labels: Vec<_> = section.menus.iter().map(|menu| menu.label).collect();
    let cli_market = section
        .menus
        .iter()
        .find(|menu| menu.id == CLI_MARKET_NODE_ID)
        .expect("cli market branch");
    let child_labels: Vec<_> = cli_market.children.iter().map(|menu| menu.label).collect();

    assert_eq!(section.label, "Knowledge");
    assert_eq!(section.default_href, "/knowledge/notes");
    assert_eq!(labels, vec!["Notes", "CLI Market"]);
    assert_eq!(cli_market.kind, AdminNavigationKind::Branch);
    assert_eq!(cli_market.permissions_any_of, &["knowledge:cli"]);
    assert_eq!(child_labels, vec!["Imports", "Docs"]);
    assert_eq!(cli_market.children[0].kind, AdminNavigationKind::Page);
    assert_eq!(
        cli_market.children[0].permissions_any_of,
        &["knowledge:cli"]
    );
}

#[test]
fn root_page_macro_should_register_top_level_page_defaults() {
    ensure_linked();
    let section =
        section_for_path("/system/settings").expect("settings route should resolve section");
    let settings = section
        .menus
        .iter()
        .find(|menu| menu.id == SETTINGS_NODE_ID)
        .expect("settings root page");

    assert_eq!(settings.kind, AdminNavigationKind::Page);
    assert_eq!(settings.href, "/system/settings");
    assert_eq!(settings.active_patterns, &["/system/settings"]);
    assert!(settings.permissions_any_of.is_empty());
    assert!(settings.children.is_empty());
}
