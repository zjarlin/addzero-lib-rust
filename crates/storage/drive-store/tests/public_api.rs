use az_drive_core::model::{EntryKey, RelativePath, RootAlias, content_hash, object_key_for_hash};
use az_drive_store::store::{
    DriveEntryKind, DriveMetadataStore, DriveObjectStore, DriveVersion, InMemoryDriveMetadataStore,
    InMemoryDriveObjectStore,
};
use chrono::Utc;
use uuid::Uuid;

fn key() -> EntryKey {
    EntryKey::new(
        "main",
        RootAlias::parse("workspace").expect("alias should parse"),
        RelativePath::parse("docs/a.md").expect("path should parse"),
    )
}

#[tokio::test]
async fn in_memory_store_tracks_latest_version() {
    let store = InMemoryDriveMetadataStore::new();
    let entry = store
        .upsert_entry(&key(), DriveEntryKind::File)
        .await
        .expect("entry should upsert");
    let hash = content_hash(b"hello");
    let version = DriveVersion {
        id: Uuid::new_v4(),
        entry_id: entry.id,
        version: 1,
        content_hash: hash.clone(),
        object_key: object_key_for_hash(&hash),
        size_bytes: 5,
        device_id: "device-a".to_owned(),
        modified_at: Utc::now(),
    };

    store
        .insert_version(version)
        .await
        .expect("version should insert");
    let latest = store
        .latest_version(entry.id)
        .await
        .expect("latest version query should work")
        .expect("latest version should exist");

    assert_eq!(latest.content_hash, hash);
}

#[tokio::test]
async fn in_memory_store_lists_entries_by_space() {
    let store = InMemoryDriveMetadataStore::new();
    store
        .upsert_entry(&key(), DriveEntryKind::File)
        .await
        .expect("main entry should upsert");
    store
        .upsert_entry(
            &EntryKey::new(
                "other",
                RootAlias::parse("workspace").expect("alias should parse"),
                RelativePath::parse("docs/b.md").expect("path should parse"),
            ),
            DriveEntryKind::File,
        )
        .await
        .expect("other entry should upsert");

    let entries = store
        .list_entries_by_space("main")
        .await
        .expect("space entries should list");

    assert_eq!(entries.len(), 1);
}

#[tokio::test]
async fn in_memory_store_migrates_legacy_main_namespace_to_owner_drive() {
    let store = InMemoryDriveMetadataStore::new();
    let entry = store
        .upsert_entry(&key(), DriveEntryKind::File)
        .await
        .expect("entry should upsert");
    let hash = content_hash(b"hello");
    store
        .insert_version(DriveVersion {
            id: Uuid::new_v4(),
            entry_id: entry.id,
            version: 1,
            content_hash: hash.clone(),
            object_key: object_key_for_hash(&hash),
            size_bytes: 5,
            device_id: "device-a".to_owned(),
            modified_at: Utc::now(),
        })
        .await
        .expect("version should insert");
    store
        .upsert_ignored_path(&key(), "device-a")
        .await
        .expect("ignore should upsert");

    let migrated = store
        .migrate_owner_drive_namespace("main", "user-zjarlin")
        .await
        .expect("namespace should migrate");

    assert_eq!(migrated, 2);
    assert!(
        store
            .list_entries_by_space("main")
            .await
            .expect("main entries should list")
            .is_empty()
    );
    assert_eq!(
        store
            .list_entries_by_space("user-zjarlin")
            .await
            .expect("owner entries should list")
            .len(),
        1
    );
    assert_eq!(
        store
            .list_ignored_paths("user-zjarlin", None, None)
            .await
            .expect("owner ignored paths should list")
            .len(),
        1
    );
}

#[tokio::test]
async fn in_memory_store_lists_ignored_paths_by_prefix() {
    let store = InMemoryDriveMetadataStore::new();
    store
        .upsert_ignored_path(&key(), "device-a")
        .await
        .expect("ignore should upsert");

    let ignored = store
        .list_ignored_paths(
            "main",
            Some(&RootAlias::parse("workspace").expect("alias should parse")),
            Some(&RelativePath::parse("docs").expect("prefix should parse")),
        )
        .await
        .expect("ignored paths should list");

    assert_eq!(ignored[0].relative_path.as_str(), "docs/a.md");
}

#[tokio::test]
async fn in_memory_object_store_round_trips_bytes() {
    let store = InMemoryDriveObjectStore::new();

    store
        .put_object("objects/demo", b"hello")
        .await
        .expect("object should store");
    let bytes = store
        .get_object("objects/demo")
        .await
        .expect("object should load");

    assert_eq!(bytes, b"hello");
}
