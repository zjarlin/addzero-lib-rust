use std::sync::Arc;

use az_drive_agent::{
    agent::{
        ConflictResolution, DriveAgent, DriveAgentConfig, ListTrackedOptions, PullRemoteOptions,
        PullRemoteStatus, TrackedItemSource, TrackedItemStatus,
    },
    local_state::LocalStateStore,
};
use az_drive_store::store::{
    DriveSyncTaskKind, DriveSyncTaskStatus, InMemoryDriveMetadataStore, InMemoryDriveObjectStore,
};
use tempfile::TempDir;

fn agent(
    temp: &TempDir,
    name: &str,
    metadata: Arc<InMemoryDriveMetadataStore>,
    objects: Arc<InMemoryDriveObjectStore>,
) -> DriveAgent {
    agent_with_space(temp, name, "main", &[], metadata, objects)
}

fn agent_with_space(
    temp: &TempDir,
    name: &str,
    space_id: &str,
    fused_space_ids: &[&str],
    metadata: Arc<InMemoryDriveMetadataStore>,
    objects: Arc<InMemoryDriveObjectStore>,
) -> DriveAgent {
    DriveAgent::new(
        metadata,
        objects,
        LocalStateStore::new(temp.path().join(format!("{name}.json"))),
        DriveAgentConfig::new(space_id, format!("device-{name}"), name.to_owned())
            .with_fused_space_ids(fused_space_ids.iter().copied().map(str::to_owned))
            .with_auto_materialize_space_ids(fused_space_ids.iter().copied().map(str::to_owned)),
    )
}

#[test]
fn drive_listing_status_enums_expose_stable_codes() {
    assert_eq!(
        TrackedItemStatus::ConflictSuspended.code(),
        "conflict_suspended"
    );
    assert_eq!(TrackedItemSource::DbIgnore.code(), "db_ignore");
    assert_eq!(
        PullRemoteStatus::from_code("skipped_existing"),
        Some(PullRemoteStatus::SkippedExisting)
    );
}

#[tokio::test]
async fn host_path_tracks_relative_path_below_root() {
    let temp = TempDir::new().expect("temp dir should exist");
    let metadata = Arc::new(InMemoryDriveMetadataStore::new());
    let objects = Arc::new(InMemoryDriveObjectStore::new());
    let agent = agent(&temp, "a", metadata, objects);
    let root = temp.path().join("workspace");
    let file = root.join("docs/a.md");
    tokio::fs::create_dir_all(file.parent().expect("file should have parent"))
        .await
        .expect("parent should be created");
    tokio::fs::write(&file, b"hello")
        .await
        .expect("file should be written");
    agent
        .add_root("workspace", root.to_str().expect("utf8 path"))
        .await
        .expect("root should add");

    let statuses = agent
        .host_path(file.to_str().expect("utf8 path"), None, None)
        .await
        .expect("file should host");

    assert_eq!(statuses[0].remote_path, "main/workspace/docs/a.md");
}

#[tokio::test]
async fn unhost_path_keeps_local_file() {
    let temp = TempDir::new().expect("temp dir should exist");
    let metadata = Arc::new(InMemoryDriveMetadataStore::new());
    let objects = Arc::new(InMemoryDriveObjectStore::new());
    let agent = agent(&temp, "a", metadata, objects);
    let file = temp.path().join("a.txt");
    tokio::fs::write(&file, b"hello")
        .await
        .expect("file should be written");
    agent
        .add_root("workspace", temp.path().to_str().expect("utf8 path"))
        .await
        .expect("root should add");

    agent
        .host_path(file.to_str().expect("utf8 path"), None, Some("a.txt"))
        .await
        .expect("file should host");
    let removed = agent
        .unhost_path(file.to_str().expect("utf8 path"))
        .await
        .expect("file should unhost");

    assert_eq!(removed, 1);
    assert!(file.exists());
}

#[tokio::test]
async fn hosted_directory_discovers_children_and_respects_gitignore() {
    let temp = TempDir::new().expect("temp dir should exist");
    let metadata = Arc::new(InMemoryDriveMetadataStore::new());
    let objects = Arc::new(InMemoryDriveObjectStore::new());
    let agent = agent(&temp, "a", metadata, objects);
    let root = temp.path().join("workspace");
    let visible = root.join("docs/a.md");
    let ignored_dir_file = root.join("target/generated.txt");
    let ignored_glob_file = root.join("notes/debug.log");
    tokio::fs::create_dir_all(visible.parent().expect("visible parent"))
        .await
        .expect("visible parent should be created");
    tokio::fs::create_dir_all(ignored_dir_file.parent().expect("ignored dir parent"))
        .await
        .expect("ignored dir parent should be created");
    tokio::fs::create_dir_all(ignored_glob_file.parent().expect("ignored glob parent"))
        .await
        .expect("ignored glob parent should be created");
    tokio::fs::write(root.join(".gitignore"), b"target/\n*.log\n")
        .await
        .expect("gitignore should be written");
    tokio::fs::write(&visible, b"visible")
        .await
        .expect("visible file should be written");
    tokio::fs::write(&ignored_dir_file, b"ignored")
        .await
        .expect("ignored dir file should be written");
    tokio::fs::write(&ignored_glob_file, b"ignored")
        .await
        .expect("ignored glob file should be written");
    agent
        .add_root("workspace", root.to_str().expect("utf8 path"))
        .await
        .expect("root should add");

    agent
        .host_path(root.to_str().expect("utf8 path"), None, None)
        .await
        .expect("directory should host");

    let initial = agent.status(None).await.expect("status should load");
    assert!(initial.iter().any(|status| status.local_path == visible));
    assert!(
        initial
            .iter()
            .all(|status| status.local_path != ignored_dir_file)
    );
    assert!(
        initial
            .iter()
            .all(|status| status.local_path != ignored_glob_file)
    );

    let new_file = root.join("docs/new.md");
    tokio::fs::write(&new_file, b"new")
        .await
        .expect("new file should be written");
    agent
        .sync_once()
        .await
        .expect("sync should discover new file");
    let after_sync = agent.status(None).await.expect("status should reload");

    assert!(
        after_sync
            .iter()
            .any(|status| status.local_path == new_file)
    );
}

#[tokio::test]
async fn list_tracked_reports_gitignored_paths_only_when_requested() {
    let temp = TempDir::new().expect("temp dir should exist");
    let metadata = Arc::new(InMemoryDriveMetadataStore::new());
    let objects = Arc::new(InMemoryDriveObjectStore::new());
    let agent = agent(&temp, "a", metadata, objects);
    let root = temp.path().join("workspace");
    let visible = root.join("docs/a.md");
    let ignored = root.join("docs/debug.log");
    tokio::fs::create_dir_all(visible.parent().expect("visible parent"))
        .await
        .expect("visible parent should be created");
    tokio::fs::write(root.join(".gitignore"), b"*.log\n")
        .await
        .expect("gitignore should be written");
    tokio::fs::write(&visible, b"visible")
        .await
        .expect("visible file should be written");
    tokio::fs::write(&ignored, b"ignored")
        .await
        .expect("ignored file should be written");
    agent
        .add_root("workspace", root.to_str().expect("utf8 path"))
        .await
        .expect("root should add");
    agent
        .host_path(root.to_str().expect("utf8 path"), None, None)
        .await
        .expect("directory should host");

    let listed = agent
        .list_tracked(None, ListTrackedOptions::default())
        .await
        .expect("tracked list should load");
    assert!(
        listed
            .iter()
            .all(|item| item.local_path.as_ref() != Some(&ignored))
    );

    let listed_with_ignored = agent
        .list_tracked(
            None,
            ListTrackedOptions {
                include_ignored: true,
                ..ListTrackedOptions::default()
            },
        )
        .await
        .expect("tracked list with ignored should load");

    assert!(listed_with_ignored.iter().any(|item| {
        item.local_path.as_ref() == Some(&ignored)
            && item.status == TrackedItemStatus::Ignored
            && item.source == TrackedItemSource::Gitignore
    }));
}

#[cfg(unix)]
#[tokio::test]
async fn hosted_directory_skips_unreadable_paths() {
    let temp = TempDir::new().expect("temp dir should exist");
    let metadata = Arc::new(InMemoryDriveMetadataStore::new());
    let objects = Arc::new(InMemoryDriveObjectStore::new());
    let agent = agent(&temp, "a", metadata, objects);
    let root = temp.path().join("workspace");
    let visible = root.join("docs/a.md");
    let unreadable = root.join("docs/private.md");
    let blocked_dir = root.join("Library/Blocked");
    tokio::fs::create_dir_all(visible.parent().expect("visible parent"))
        .await
        .expect("visible parent should be created");
    tokio::fs::create_dir_all(&blocked_dir)
        .await
        .expect("blocked dir should be created");
    tokio::fs::write(&visible, b"visible")
        .await
        .expect("visible file should be written");
    tokio::fs::write(&unreadable, b"private")
        .await
        .expect("unreadable file should be written");
    tokio::fs::write(blocked_dir.join("secret.md"), b"secret")
        .await
        .expect("blocked file should be written");
    std::fs::set_permissions(
        &unreadable,
        std::os::unix::fs::PermissionsExt::from_mode(0o000),
    )
    .expect("unreadable file permissions should change");
    std::fs::set_permissions(
        &blocked_dir,
        std::os::unix::fs::PermissionsExt::from_mode(0o000),
    )
    .expect("blocked dir permissions should change");
    agent
        .add_root("workspace", root.to_str().expect("utf8 path"))
        .await
        .expect("root should add");

    let statuses = agent
        .host_path(root.to_str().expect("utf8 path"), None, None)
        .await
        .expect("directory should host readable descendants");

    std::fs::set_permissions(
        &unreadable,
        std::os::unix::fs::PermissionsExt::from_mode(0o600),
    )
    .expect("unreadable file permissions should restore");
    std::fs::set_permissions(
        &blocked_dir,
        std::os::unix::fs::PermissionsExt::from_mode(0o700),
    )
    .expect("blocked dir permissions should restore");
    assert!(statuses.iter().any(|status| status.local_path == visible));
}

#[tokio::test]
async fn unhost_child_under_hosted_root_creates_shared_ignore_rule() {
    let temp = TempDir::new().expect("temp dir should exist");
    let metadata = Arc::new(InMemoryDriveMetadataStore::new());
    let objects = Arc::new(InMemoryDriveObjectStore::new());
    let agent = agent(&temp, "a", metadata, objects);
    let root = temp.path().join("workspace");
    let visible = root.join("docs/a.md");
    let ignored_dir = root.join("private");
    let ignored_file = ignored_dir.join("secret.md");
    tokio::fs::create_dir_all(visible.parent().expect("visible parent"))
        .await
        .expect("visible parent should be created");
    tokio::fs::create_dir_all(&ignored_dir)
        .await
        .expect("ignored dir should be created");
    tokio::fs::write(&visible, b"visible")
        .await
        .expect("visible file should be written");
    tokio::fs::write(&ignored_file, b"secret")
        .await
        .expect("ignored file should be written");
    agent
        .add_root("workspace", root.to_str().expect("utf8 path"))
        .await
        .expect("root should add");
    agent
        .host_path(root.to_str().expect("utf8 path"), None, None)
        .await
        .expect("directory should host");

    let removed = agent
        .unhost_path(ignored_dir.to_str().expect("utf8 path"))
        .await
        .expect("child should unhost");

    assert!(removed > 0);
    let listed = agent
        .list_tracked(None, ListTrackedOptions::default())
        .await
        .expect("tracked list should load");
    assert!(
        listed
            .iter()
            .all(|item| item.local_path.as_ref() != Some(&ignored_file))
    );

    let ignored = agent
        .list_tracked(
            None,
            ListTrackedOptions {
                include_ignored: true,
                ..ListTrackedOptions::default()
            },
        )
        .await
        .expect("ignored list should load");

    assert!(ignored.iter().any(|item| {
        item.canonical_path == "workspace/private"
            && item.status == TrackedItemStatus::Ignored
            && item.source == TrackedItemSource::DbIgnore
    }));
}

#[tokio::test]
async fn list_tracked_uses_home_canonical_path_across_devices() {
    let temp = TempDir::new().expect("temp dir should exist");
    let metadata = Arc::new(InMemoryDriveMetadataStore::new());
    let objects = Arc::new(InMemoryDriveObjectStore::new());
    let agent_a = agent(&temp, "a", Arc::clone(&metadata), Arc::clone(&objects));
    let agent_b = agent(&temp, "b", metadata, objects);
    let home_a = temp.path().join("home-a");
    let home_b = temp.path().join("home-b");
    let file_a = home_a.join(".agents/skills/demo/SKILL.md");
    let file_b = home_b.join(".agents/skills/demo/SKILL.md");
    tokio::fs::create_dir_all(file_a.parent().expect("file a parent"))
        .await
        .expect("file a parent should be created");
    tokio::fs::create_dir_all(file_b.parent().expect("file b parent"))
        .await
        .expect("file b parent should be created");
    tokio::fs::write(&file_a, b"skill")
        .await
        .expect("file a should be written");
    tokio::fs::write(&file_b, b"skill")
        .await
        .expect("file b should be written");
    agent_a
        .add_root("home", home_a.to_str().expect("utf8 path"))
        .await
        .expect("home a should add");
    agent_b
        .add_root("home", home_b.to_str().expect("utf8 path"))
        .await
        .expect("home b should add");
    agent_a
        .host_path(file_a.to_str().expect("utf8 path"), None, None)
        .await
        .expect("file a should host");
    agent_b
        .host_path(file_b.to_str().expect("utf8 path"), None, None)
        .await
        .expect("file b should host");

    let listed_a = agent_a
        .list_tracked(None, ListTrackedOptions::default())
        .await
        .expect("agent a list should load");
    let listed_b = agent_b
        .list_tracked(None, ListTrackedOptions::default())
        .await
        .expect("agent b list should load");

    assert_eq!(listed_a[0].canonical_path, listed_b[0].canonical_path);
}

#[tokio::test]
async fn list_tracked_displays_absolute_root_paths() {
    let temp = TempDir::new().expect("temp dir should exist");
    let metadata = Arc::new(InMemoryDriveMetadataStore::new());
    let objects = Arc::new(InMemoryDriveObjectStore::new());
    let agent = agent(&temp, "a", metadata, objects);
    let root = temp.path().join("macos-root");
    let file = root.join("Library/Application Support/demo");
    tokio::fs::create_dir_all(file.parent().expect("file should have parent"))
        .await
        .expect("parent should be created");
    tokio::fs::write(&file, b"demo")
        .await
        .expect("file should be written");
    agent
        .add_root("macos", root.to_str().expect("utf8 path"))
        .await
        .expect("macos root should add");
    agent
        .host_path(file.to_str().expect("utf8 path"), None, None)
        .await
        .expect("file should host");

    let listed = agent
        .list_tracked(None, ListTrackedOptions::default())
        .await
        .expect("tracked list should load");

    assert!(
        listed[0]
            .display_path
            .ends_with("Library/Application Support/demo")
    );
}

#[tokio::test]
async fn sync_once_creates_conflict_copy_for_stale_local_change() {
    let temp = TempDir::new().expect("temp dir should exist");
    let metadata = Arc::new(InMemoryDriveMetadataStore::new());
    let objects = Arc::new(InMemoryDriveObjectStore::new());
    let agent_a = agent(&temp, "a", Arc::clone(&metadata), Arc::clone(&objects));
    let agent_b = agent(&temp, "b", metadata, objects);
    let root_a = temp.path().join("a-root");
    let root_b = temp.path().join("b-root");
    let file_a = root_a.join("same.txt");
    let file_b = root_b.join("same.txt");
    tokio::fs::create_dir_all(&root_a).await.expect("root a");
    tokio::fs::create_dir_all(&root_b).await.expect("root b");
    tokio::fs::write(&file_a, b"base").await.expect("file a");
    tokio::fs::write(&file_b, b"base").await.expect("file b");
    agent_a
        .add_root("workspace", root_a.to_str().expect("utf8 path"))
        .await
        .expect("root a should add");
    agent_b
        .add_root("workspace", root_b.to_str().expect("utf8 path"))
        .await
        .expect("root b should add");
    agent_a
        .host_path(file_a.to_str().expect("utf8 path"), None, None)
        .await
        .expect("a should host");
    agent_b
        .host_path(file_b.to_str().expect("utf8 path"), None, None)
        .await
        .expect("b should host");

    tokio::fs::write(&file_a, b"from-a").await.expect("edit a");
    agent_a.sync_once().await.expect("a sync");
    tokio::fs::write(&file_b, b"from-b").await.expect("edit b");
    agent_b.sync_once().await.expect("b sync");
    let conflicts = agent_b.conflicts().await.expect("conflicts should load");
    let listed = agent_b
        .list_tracked(None, ListTrackedOptions::default())
        .await
        .expect("tracked items should load");
    let queue = agent_b
        .sync_queue(Some(DriveSyncTaskStatus::Done))
        .await
        .expect("queue should load");

    assert_eq!(conflicts.len(), 1);
    assert!(
        listed
            .iter()
            .any(|item| item.status == TrackedItemStatus::ConflictSuspended)
    );
    assert!(
        queue
            .iter()
            .any(|item| item.kind == DriveSyncTaskKind::Conflict)
    );

    agent_b
        .resolve_conflict(conflicts[0].id, ConflictResolution::KeepRemote)
        .await
        .expect("conflict should resolve");
    let listed_after_resolve = agent_b
        .list_tracked(None, ListTrackedOptions::default())
        .await
        .expect("tracked items should reload");
    assert!(
        listed_after_resolve
            .iter()
            .all(|item| item.status != TrackedItemStatus::ConflictSuspended)
    );
}

#[tokio::test]
async fn pull_remote_materializes_entries_under_device_root() {
    let temp = TempDir::new().expect("temp dir should exist");
    let metadata = Arc::new(InMemoryDriveMetadataStore::new());
    let objects = Arc::new(InMemoryDriveObjectStore::new());
    let agent_a = agent(&temp, "a", Arc::clone(&metadata), Arc::clone(&objects));
    let agent_b = agent(&temp, "b", metadata, objects);
    let root_a = temp.path().join("a-root");
    let root_b = temp.path().join("b-root");
    let file_a = root_a.join("skills/demo/SKILL.md");
    tokio::fs::create_dir_all(file_a.parent().expect("file should have parent"))
        .await
        .expect("parent should be created");
    tokio::fs::write(&file_a, b"skill from a")
        .await
        .expect("source file should be written");
    agent_a
        .add_root("workspace", root_a.to_str().expect("utf8 path"))
        .await
        .expect("root a should add");
    agent_b
        .add_root("workspace", root_b.to_str().expect("utf8 path"))
        .await
        .expect("root b should add");
    agent_a
        .host_path(file_a.to_str().expect("utf8 path"), None, None)
        .await
        .expect("source file should host");

    let pulled = agent_b
        .pull_remote(None, PullRemoteOptions::default())
        .await
        .expect("remote should pull");

    let file_b = root_b.join("skills/demo/SKILL.md");
    assert_eq!(pulled[0].status, PullRemoteStatus::Pulled);
    assert_eq!(
        tokio::fs::read_to_string(&file_b)
            .await
            .expect("pulled file should exist"),
        "skill from a"
    );
    assert!(
        agent_b
            .status(None)
            .await
            .expect("status should load")
            .iter()
            .any(|status| status.local_path == file_b)
    );
}

#[tokio::test]
async fn sync_once_materializes_primary_owner_remote_on_new_device() {
    let temp = TempDir::new().expect("temp dir should exist");
    let metadata = Arc::new(InMemoryDriveMetadataStore::new());
    let objects = Arc::new(InMemoryDriveObjectStore::new());
    let agent_a = agent(&temp, "a", Arc::clone(&metadata), Arc::clone(&objects));
    let agent_b = agent(&temp, "b", metadata, objects);
    let root_a = temp.path().join("a-root");
    let root_b = temp.path().join("b-root");
    let file_a = root_a.join("docs/a.txt");
    tokio::fs::create_dir_all(file_a.parent().expect("file should have parent"))
        .await
        .expect("parent should be created");
    tokio::fs::write(&file_a, b"from primary owner")
        .await
        .expect("source file should be written");
    agent_a
        .add_root("workspace", root_a.to_str().expect("utf8 path"))
        .await
        .expect("root a should add");
    agent_b
        .add_root("workspace", root_b.to_str().expect("utf8 path"))
        .await
        .expect("root b should add");
    agent_a
        .host_path(file_a.to_str().expect("utf8 path"), None, None)
        .await
        .expect("source file should host");

    agent_b
        .sync_once()
        .await
        .expect("new device sync should materialize primary owner file");

    let file_b = root_b.join("docs/a.txt");
    assert_eq!(
        tokio::fs::read_to_string(&file_b)
            .await
            .expect("synced file should exist"),
        "from primary owner"
    );
}

#[tokio::test]
async fn migrate_legacy_owner_drive_updates_metadata_and_local_state() {
    let temp = TempDir::new().expect("temp dir should exist");
    let metadata = Arc::new(InMemoryDriveMetadataStore::new());
    let objects = Arc::new(InMemoryDriveObjectStore::new());
    let agent = agent(&temp, "a", metadata, objects);
    let root = temp.path().join("workspace");
    let file = root.join("docs/a.md");
    tokio::fs::create_dir_all(file.parent().expect("file should have parent"))
        .await
        .expect("parent should be created");
    tokio::fs::write(&file, b"hello")
        .await
        .expect("file should be written");
    agent
        .add_root("workspace", root.to_str().expect("utf8 path"))
        .await
        .expect("root should add");
    agent
        .host_path(file.to_str().expect("utf8 path"), None, None)
        .await
        .expect("file should host");

    let migrated = agent
        .migrate_legacy_owner_drive("main", "user-zjarlin")
        .await
        .expect("legacy owner drive should migrate");

    assert!(migrated >= 2);
    assert!(
        agent
            .list_tracked(None, ListTrackedOptions::default())
            .await
            .expect("tracked files should list")
            .iter()
            .any(|item| item.remote_path == "user-zjarlin/workspace/docs/a.md")
    );
}

#[tokio::test]
async fn sync_once_materializes_fused_remote_and_uploads_local_edit() {
    let temp = TempDir::new().expect("temp dir should exist");
    let metadata = Arc::new(InMemoryDriveMetadataStore::new());
    let objects = Arc::new(InMemoryDriveObjectStore::new());
    let owner_space = "user-owner";
    let agent_owner = agent_with_space(
        &temp,
        "owner",
        owner_space,
        &[],
        Arc::clone(&metadata),
        Arc::clone(&objects),
    );
    let agent_guest = agent_with_space(
        &temp,
        "guest",
        "user-guest",
        &[owner_space],
        metadata,
        objects,
    );
    let owner_root = temp.path().join("owner-root");
    let guest_root = temp.path().join("guest-root");
    let owner_file = owner_root.join("skills/demo/SKILL.md");
    tokio::fs::create_dir_all(owner_file.parent().expect("file should have parent"))
        .await
        .expect("parent should be created");
    tokio::fs::write(&owner_file, b"from owner")
        .await
        .expect("owner file should be written");
    agent_owner
        .add_root("workspace", owner_root.to_str().expect("utf8 path"))
        .await
        .expect("owner root should add");
    agent_guest
        .add_root("workspace", guest_root.to_str().expect("utf8 path"))
        .await
        .expect("guest root should add");
    agent_owner
        .host_path(owner_file.to_str().expect("utf8 path"), None, None)
        .await
        .expect("owner file should host");

    agent_guest
        .sync_once()
        .await
        .expect("guest sync should materialize fused file");

    let guest_file = guest_root.join("skills/demo/SKILL.md");
    assert_eq!(
        tokio::fs::read_to_string(&guest_file)
            .await
            .expect("fused file should exist on guest"),
        "from owner"
    );

    tokio::fs::write(&guest_file, b"from guest")
        .await
        .expect("guest edit should be written");
    agent_guest
        .sync_once()
        .await
        .expect("guest edit should upload to owner space");
    agent_owner
        .sync_once()
        .await
        .expect("owner sync should receive guest edit");

    assert_eq!(
        tokio::fs::read_to_string(&owner_file)
            .await
            .expect("owner file should receive remote update"),
        "from guest"
    );
}
