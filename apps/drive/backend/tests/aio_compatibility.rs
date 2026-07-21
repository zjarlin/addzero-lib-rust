use az_drive_agent::{agent::DriveAgentConfig, local_state::LocalState};
use az_drive_core::model::{RelativePath, RootAlias};
use az_drive_store::store::{DriveEntryKind, InMemoryDriveMetadataStore};

#[test]
fn public_drive_crates_are_importable_for_future_aio_integration() {
    let _metadata = InMemoryDriveMetadataStore::new();
    let _kind = DriveEntryKind::File;
    let alias = RootAlias::parse("workspace").expect("alias should parse");
    let relative = RelativePath::parse("notes/a.md").expect("path should parse");
    let local_state = LocalState::new("aio-compat".to_owned());
    let config = DriveAgentConfig::new("main", local_state.device_id, local_state.device_name);

    assert_eq!(alias.as_str(), "workspace");
    assert_eq!(relative.as_str(), "notes/a.md");
    assert_eq!(config.space_id, "main");
}

#[test]
fn owner_drive_id_is_stable_for_user_api_key_ownership() {
    assert_eq!(
        az_drive_app::owner_drive_id_for_username("zjarlin"),
        "user-zjarlin"
    );
    assert_eq!(az_drive_app::owner_drive_id_for_username("张三"), "user-__");
}
