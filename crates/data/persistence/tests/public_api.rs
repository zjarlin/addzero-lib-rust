use az_persistence::migration::WorkspaceMigrator;
use sea_orm_migration::prelude::MigratorTrait;
use std::collections::HashSet;

#[test]
fn workspace_migration_names_are_stable_and_unique() {
    let names = WorkspaceMigrator::migrations()
        .into_iter()
        .map(|migration| migration.name().to_string())
        .collect::<Vec<_>>();
    let unique = names.iter().collect::<HashSet<_>>();

    assert_eq!(
        unique.len(),
        names.len(),
        "each migration needs its own seaql_migrations version"
    );
    assert!(
        !names.iter().any(|name| name == "lib"),
        "file-level derived migration names collapse all inline migrations to `lib`"
    );
    assert!(names.contains(&"0002_clianything_market".to_string()));
    assert!(names.contains(&"0012_unified_resource_system".to_string()));
}
