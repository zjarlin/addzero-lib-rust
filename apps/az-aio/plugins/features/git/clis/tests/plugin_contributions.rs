use std::fs;

use az_aio_plugin_api::api::{AzAioPlugin, GeneratedFileStatus};
use az_aio_plugin_git_clis::plugin::GitClisPlugin;
use tempfile::TempDir;

#[test]
fn plugin_enable_scans_entries_without_rewriting_add_fn() {
    let temp = TempDir::new().expect("create temp dir");
    let root = temp.path().join("shell");
    let output = temp.path().join(".add_fn");
    fs::create_dir_all(root.join("profile.d")).expect("create profile dir");
    fs::write(root.join("profile.d/env.sh"), "export TOKEN=secret\n").expect("write shell file");
    fs::write(&output, "manual content").expect("write existing file");

    let mut plugin = GitClisPlugin::new(&root, &output, Vec::new());
    plugin.on_enable().expect("enable plugin");
    let content = fs::read_to_string(&output).expect("read generated file");
    let contributions = plugin.contributions().expect("load contributions");

    assert_eq!(content, "manual content");
    assert!(
        contributions
            .shell_entries
            .iter()
            .any(|entry| entry.name == "TOKEN")
    );
    assert_eq!(
        contributions.generated_files[0].status,
        GeneratedFileStatus::Generated
    );
    assert!(
        contributions.generated_files[0]
            .message
            .contains("可视化命令管理器")
    );
}
