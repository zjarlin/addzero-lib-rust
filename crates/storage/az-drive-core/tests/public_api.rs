use az_drive_core::api::{
    ChangeDecision, LockSnapshot, RelativePath, RootAlias, RootRegistry, conflict_file_name,
    content_hash, decide_local_change, try_safe_text_merge,
};
use chrono::{TimeZone, Utc};
use std::path::Path;

#[test]
fn relative_path_rejects_parent_traversal() {
    let error = RelativePath::parse("../secret").expect_err("parent traversal must be rejected");

    assert_eq!(error.to_string(), "invalid relative path `../secret`");
}

#[test]
fn relative_path_normalizes_windows_separators() {
    let path = RelativePath::parse("notes\\today.md").expect("path should normalize");

    assert_eq!(path.as_str(), "notes/today.md");
}

#[test]
fn root_registry_maps_different_home_paths_to_same_relative_identity() {
    let alias = RootAlias::parse("workspace").expect("alias should parse");
    let mut registry = RootRegistry::default();
    registry
        .add_root(alias, "/Users/alice/workspace")
        .expect("root should be added");

    let mapping = registry
        .resolve_host_path("/Users/alice/workspace/docs/a.md", None)
        .expect("path should map to root");

    assert_eq!(mapping.relative_path.as_str(), "docs/a.md");
}

#[test]
fn change_decision_conflicts_when_remote_advanced() {
    let decision = decide_local_change(
        Some(1),
        Some(2),
        "local",
        Some("remote"),
        None,
        "device-a",
        Utc::now(),
    );

    assert_eq!(decision, ChangeDecision::Conflict);
}

#[test]
fn change_decision_blocks_other_active_lock() {
    let now = Utc.with_ymd_and_hms(2026, 5, 9, 12, 0, 0).unwrap();
    let lock = LockSnapshot {
        owner_device_id: "device-b".to_owned(),
        expires_at: now + chrono::Duration::minutes(5),
    };

    let decision = decide_local_change(
        Some(1),
        Some(1),
        "local",
        Some("remote"),
        Some(&lock),
        "device-a",
        now,
    );

    assert_eq!(
        decision,
        ChangeDecision::LockedByOther {
            owner_device_id: "device-b".to_owned()
        }
    );
}

#[test]
fn conflict_file_name_preserves_extension() {
    let timestamp = Utc.with_ymd_and_hms(2026, 5, 9, 12, 30, 0).unwrap();

    let name = conflict_file_name(Path::new("report.docx"), "mac book", timestamp);

    assert_eq!(name, "report.conflict.mac-book.20260509T123000Z.docx");
}

#[test]
fn content_hash_is_stable_for_same_bytes() {
    let left = content_hash(b"hello");
    let right = content_hash(b"hello");

    assert_eq!(left, right);
}

#[test]
fn safe_text_merge_combines_append_only_changes() {
    let merged = try_safe_text_merge(b"a\n", b"a\nb\n", b"a\nc\n")
        .expect("append-only changes should merge");

    assert_eq!(merged, b"a\nb\nc\n");
}
