use std::{fs, io::Write, path::Path};

use az_plugin_contract::{
    HostCapability, MarkdownSchema, PageSchema, PluginDescriptor, PluginKind,
    PluginMenuContribution, PluginPackageManifest, PluginPage, PluginStatus, RuntimeBinding,
};
use az_plugin_runtime::{PluginRuntime, RuntimeError, create_package_from_dir, unpack_package};
use sha2::{Digest, Sha256};
use tempfile::TempDir;
use zip::{ZipWriter, write::SimpleFileOptions};

#[test]
fn installs_packaged_plugin_and_creates_instances() {
    let temp = TempDir::new().expect("temp dir should exist");
    let source = temp.path().join("memory-manager");
    let catalog = temp.path().join("catalog");
    let install = temp.path().join("install");
    fs::create_dir_all(source.join("backend")).expect("backend dir should exist");
    fs::create_dir_all(&catalog).expect("catalog dir should exist");

    let manifest = PluginPackageManifest {
        descriptor: PluginDescriptor {
            id: "memory-manager".to_string(),
            name: "记忆管理系统".to_string(),
            version: "0.1.0".to_string(),
            kind: PluginKind::Business,
            summary: "面向资料沉淀与回忆检索的业务插件".to_string(),
            tags: vec!["memory".to_string(), "knowledge".to_string()],
            icon: Some("brain".to_string()),
            compatibility: vec!["web".to_string(), "desktop".to_string()],
            capabilities: vec![HostCapability::Dictionary, HostCapability::Audit],
            menus: vec![PluginMenuContribution {
                section: "业务应用".to_string(),
                label: "工作台".to_string(),
                page_id: "overview".to_string(),
                order: 10,
                icon: None,
            }],
            pages: vec![PluginPage {
                id: "overview".to_string(),
                title: "工作台".to_string(),
                subtitle: "插件首页".to_string(),
                schema: PageSchema::Markdown(MarkdownSchema {
                    body: "hello".to_string(),
                }),
            }],
            metadata: Default::default(),
            cli_commands: vec![],
        },
        runtime: RuntimeBinding {
            binary_path: "backend/plugin.wasm".to_string(),
            checksum_path: "checksums.sha256".to_string(),
            assets_dir: None,
        },
        default_instance_label: Some("默认记忆空间".to_string()),
    };
    let manifest_toml =
        toml_edit::ser::to_string_pretty(&manifest).expect("manifest should serialize");
    fs::write(source.join("plugin.toml"), manifest_toml).expect("manifest should write");

    let wasm_bytes = [0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00];
    fs::write(source.join("backend/plugin.wasm"), wasm_bytes).expect("wasm should write");
    write_checksums(&source);

    let package_path = catalog.join("memory-manager.azplugin");
    create_package_from_dir(&source, &package_path).expect("package should be created");

    let mut runtime = PluginRuntime::new(&catalog, &install).expect("runtime should init");
    let snapshot = runtime.marketplace_snapshot();
    assert_eq!(snapshot.entries.len(), 1);
    assert_eq!(snapshot.entries[0].status, PluginStatus::Available);

    let installed = runtime
        .install_from_catalog("memory-manager")
        .expect("plugin should install");
    assert_eq!(installed.id, "memory-manager");

    let instance = runtime
        .create_instance("memory-manager", "资料员管理系统")
        .expect("instance should create");
    assert_eq!(instance.plugin_name, "记忆管理系统");
    assert!(
        instance.slug.starts_with("plugin-instance")
            || instance.slug.starts_with("memory")
            || instance.slug.starts_with("plugin")
    );
}

#[test]
fn unpack_should_reject_path_traversal_entries() {
    let temp = TempDir::new().expect("temp dir should exist");
    let package_path = temp.path().join("unsafe.azplugin");
    let target = temp.path().join("target");

    let package_file = fs::File::create(&package_path).expect("package should create");
    let mut writer = ZipWriter::new(package_file);
    writer
        .start_file("../escape.txt", SimpleFileOptions::default())
        .expect("unsafe entry should write");
    writer
        .write_all(b"escape")
        .expect("unsafe entry bytes should write");
    writer.finish().expect("package should finish");

    let err = unpack_package(&package_path, &target).expect_err("unsafe path must be rejected");
    match err {
        RuntimeError::InvalidPackage(message) => {
            // The extractor must reject traversal before touching the output directory.
            assert!(message.contains("escapes target directory"));
        }
        other => panic!("expected invalid package error, got {other:?}"),
    }
    assert!(!temp.path().join("escape.txt").exists());
}

fn write_checksums(source: &Path) {
    let manifest_hash = sha256_hex(&fs::read(source.join("plugin.toml")).expect("manifest bytes"));
    let wasm_hash = sha256_hex(&fs::read(source.join("backend/plugin.wasm")).expect("wasm bytes"));
    let content = format!("{manifest_hash}  plugin.toml\n{wasm_hash}  backend/plugin.wasm\n");
    fs::write(source.join("checksums.sha256"), content).expect("checksums should write");
}

fn sha256_hex(bytes: &[u8]) -> String {
    let mut digest = Sha256::new();
    digest.update(bytes);
    format!("{:x}", digest.finalize())
}
