use addzero_io::*;
use std::fs;
use tempfile::TempDir;

#[test]
fn ensure_file_and_directory_behave_explicitly() {
    let temp = TempDir::new().expect("temp dir should be created");
    let file_path = temp.path().join("nested/output.txt");
    let dir_path = temp.path().join("logs");

    file_path
        .as_path()
        .ensure_file()
        .expect("file should be created");
    dir_path
        .as_path()
        .ensure_dir()
        .expect("dir should be created");

    assert!(file_path.is_file());
    assert!(dir_path.is_dir());
}

#[test]
fn remove_if_exists_handles_files_and_directories() {
    let temp = TempDir::new().expect("temp dir should be created");
    let file_path = temp.path().join("artifact.txt");
    let dir_path = temp.path().join("build/cache");
    file_path
        .as_path()
        .ensure_file()
        .expect("file should be created");
    dir_path
        .as_path()
        .ensure_dir()
        .expect("dir should be created");

    file_path
        .as_path()
        .remove_if_exists()
        .expect("file should be removed");
    temp.path()
        .join("build")
        .as_path()
        .remove_if_exists()
        .expect("dir should be removed");

    assert!(!file_path.exists());
    assert!(!temp.path().join("build").exists());
}

#[test]
fn mvln_returns_noop_when_paths_match() {
    let temp = TempDir::new().expect("temp dir should be created");
    let path = temp.path().join("same.txt");
    fs::write(&path, "hello").expect("file should be written");

    let result = mvln(&path, &path).expect("same path should be a noop");

    assert_eq!(result, path);
}

#[cfg(unix)]
#[test]
fn mvln_moves_file_and_undo_restores_it() {
    let temp = TempDir::new().expect("temp dir should be created");
    let source = temp.path().join("report.txt");
    fs::write(&source, "hello world").expect("file should be written");

    let moved = MoveLink::new(&source)
        .to(temp.path().join("archive"))
        .move_and_link()
        .expect("move-and-link should succeed");

    assert_eq!(moved, temp.path().join("archive/report.txt"));
    assert!(
        fs::symlink_metadata(&source)
            .expect("metadata should exist")
            .file_type()
            .is_symlink()
    );
    assert_eq!(
        fs::read_to_string(&source).expect("link should resolve"),
        "hello world"
    );

    let restored = undo_mvln(&source).expect("undo should succeed");

    assert_eq!(restored, source);
    assert!(source.is_file());
    assert_eq!(
        fs::read_to_string(&source).expect("file should be restored"),
        "hello world"
    );
    assert!(!moved.exists());
}

#[cfg(unix)]
#[test]
fn mvln_moves_directory_and_undo_restores_it() {
    let temp = TempDir::new().expect("temp dir should be created");
    let source = temp.path().join("docs");
    source
        .as_path()
        .ensure_dir()
        .expect("dir should be created");
    fs::write(source.join("guide.md"), "# guide").expect("nested file should be written");

    let moved = mvln(&source, temp.path().join("backup")).expect("directory move should work");

    assert_eq!(moved, temp.path().join("backup/docs"));
    assert!(
        fs::symlink_metadata(&source)
            .expect("metadata should exist")
            .file_type()
            .is_symlink()
    );
    assert_eq!(
        fs::read_to_string(source.join("guide.md")).expect("link should resolve"),
        "# guide"
    );

    undo_mvln(&source).expect("undo should restore directory");

    assert!(source.is_dir());
    assert_eq!(
        fs::read_to_string(source.join("guide.md")).expect("directory should be restored"),
        "# guide"
    );
    assert!(!moved.exists());
}

#[cfg(unix)]
#[test]
fn undo_mvln_reports_non_symlink_and_broken_symlink_errors() {
    let temp = TempDir::new().expect("temp dir should be created");
    let regular_file = temp.path().join("plain.txt");
    fs::write(&regular_file, "hello").expect("file should be written");

    let regular_error = undo_mvln(&regular_file).expect_err("regular file should fail");
    assert!(matches!(regular_error, IoError::NotSymlink(_)));

    let broken_link = temp.path().join("dangling.txt");
    std::os::unix::fs::symlink(temp.path().join("missing.txt"), &broken_link)
        .expect("broken symlink should be created");

    let broken_error = undo_mvln(&broken_link).expect_err("broken symlink should fail");
    assert!(matches!(broken_error, IoError::BrokenSymlink(_)));
}
