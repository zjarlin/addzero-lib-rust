use az_persistence::migration::workspace_sql_migrations;
use std::collections::HashSet;

#[test]
fn workspace_migration_names_are_stable_and_unique() {
    let names = workspace_sql_migrations()
        .iter()
        .map(|migration| migration.name.to_string())
        .collect::<Vec<_>>();
    let unique = names.iter().collect::<HashSet<_>>();

    assert_eq!(
        unique.len(),
        names.len(),
        "每个 Toasty SQL 迁移都必须使用独立名称"
    );
    assert!(
        !names.iter().any(|name| name == "lib"),
        "迁移名称不能退化为模块文件名"
    );
    assert!(names.contains(&"0002_clianything_market.sql".to_string()));
    assert!(names.contains(&"0012_unified_resource_system.sql".to_string()));
}
