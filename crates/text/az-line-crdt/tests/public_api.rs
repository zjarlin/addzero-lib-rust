use az_line_crdt::{LineCrdtDocument, LineCrdtError};

#[test]
fn line_operations_preserve_file_text() -> Result<(), Box<dyn std::error::Error>> {
    let document = LineCrdtDocument::from_text("alpha\nbeta")?;

    document.insert_line(1, "inserted")?;
    assert_eq!(document.text(), "alpha\ninserted\nbeta");

    document.replace_line(2, "gamma")?;
    assert_eq!(document.lines(), vec!["alpha", "inserted", "gamma"]);

    document.delete_line(1)?;
    assert_eq!(document.text(), "alpha\ngamma");

    Ok(())
}

#[test]
fn line_insert_rejects_embedded_newline() -> Result<(), Box<dyn std::error::Error>> {
    let document = LineCrdtDocument::from_text("alpha")?;
    let error = document
        .insert_line(0, "bad\nline")
        .expect_err("embedded newline should be rejected");

    assert!(matches!(error, LineCrdtError::LineContainsNewline));
    Ok(())
}

#[test]
fn snapshots_restore_exact_text() -> Result<(), Box<dyn std::error::Error>> {
    let document = LineCrdtDocument::from_text("one\ntwo\n")?;
    let snapshot = document.export_snapshot()?;
    let restored = LineCrdtDocument::from_snapshot(snapshot)?;

    assert_eq!(restored.text(), "one\ntwo\n");
    assert_eq!(restored.lines(), vec!["one", "two", ""]);

    Ok(())
}

#[test]
fn incremental_update_syncs_only_changes_after_cursor() -> Result<(), Box<dyn std::error::Error>> {
    let local = LineCrdtDocument::from_text_with_peer_id("one\ntwo", 11)?;
    let remote = LineCrdtDocument::from_snapshot_with_peer_id(local.export_snapshot()?, 12)?;
    let remote_cursor = remote.version();

    local.replace_line(1, "TWO")?;
    let update = local.export_updates_since(&remote_cursor)?;
    let report = remote.import_update(update)?;

    assert!(report.is_complete());
    assert_eq!(remote.text(), "one\nTWO");

    Ok(())
}

#[test]
fn duplicate_update_import_is_idempotent() -> Result<(), Box<dyn std::error::Error>> {
    let local = LineCrdtDocument::from_text_with_peer_id("one", 21)?;
    let remote = LineCrdtDocument::from_snapshot_with_peer_id(local.export_snapshot()?, 22)?;
    let cursor = remote.version();

    local.append_line("two")?;
    let update = local.export_updates_since(&cursor)?;

    remote.import_update(update.clone())?;
    remote.import_update(update)?;

    assert_eq!(remote.text(), "one\ntwo");
    Ok(())
}

#[test]
fn concurrent_line_insertions_converge() -> Result<(), Box<dyn std::error::Error>> {
    let seed = LineCrdtDocument::from_text_with_peer_id("top\nbottom", 31)?;
    let snapshot = seed.export_snapshot()?;
    let cursor = seed.version();
    let left = LineCrdtDocument::from_snapshot_with_peer_id(snapshot.clone(), 32)?;
    let right = LineCrdtDocument::from_snapshot_with_peer_id(snapshot, 33)?;

    left.insert_line(1, "left")?;
    right.insert_line(1, "right")?;

    let left_update = left.export_updates_since(&cursor)?;
    let right_update = right.export_updates_since(&cursor)?;
    left.import_update(right_update)?;
    right.import_update(left_update)?;

    assert_eq!(left.text(), right.text());
    assert!(left.lines().contains(&"left".to_owned()));
    assert!(left.lines().contains(&"right".to_owned()));

    Ok(())
}

#[test]
fn concurrent_character_edits_converge() -> Result<(), Box<dyn std::error::Error>> {
    let seed = LineCrdtDocument::from_text_with_peer_id("hello\nworld", 41)?;
    let snapshot = seed.export_snapshot()?;
    let cursor = seed.version();
    let left = LineCrdtDocument::from_snapshot_with_peer_id(snapshot.clone(), 42)?;
    let right = LineCrdtDocument::from_snapshot_with_peer_id(snapshot, 43)?;

    left.delete_text(2, 3)?;
    right.insert_text(5, "!")?;

    let left_update = left.export_updates_since(&cursor)?;
    let right_update = right.export_updates_since(&cursor)?;
    left.import_update(right_update)?;
    right.import_update(left_update)?;

    assert_eq!(left.text(), right.text());
    assert!(left.text().contains("he"));
    assert!(left.text().contains("world"));

    Ok(())
}

#[test]
fn apply_text_by_line_tracks_existing_file_rewrite() -> Result<(), Box<dyn std::error::Error>> {
    let local = LineCrdtDocument::from_text_with_peer_id("a\nb\nc", 51)?;
    let remote = LineCrdtDocument::from_snapshot_with_peer_id(local.export_snapshot()?, 52)?;
    let cursor = remote.version();

    local.apply_text_by_line("a\nB\nc\nd")?;
    let update = local.export_updates_since(&cursor)?;
    remote.import_update(update)?;

    assert_eq!(remote.text(), "a\nB\nc\nd");

    Ok(())
}
