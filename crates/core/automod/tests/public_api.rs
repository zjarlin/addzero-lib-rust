use std::fs;
use std::path::Path;
use std::process::Command;

#[test]
fn directory_modules_compile() {
    let project = TestProject::new("directory-modules");
    project.write_manifest();
    project.write(
        "src/lib.rs",
        r#"
        automod::dir!(pub "src");

        pub fn assert_modules() {
            assert_eq!(user::value(), "user");
            assert_eq!(nested::alpha::value(), "alpha");
            assert_eq!(nested::deep::beta::value(), "beta");
            assert_eq!(hyphen_name::child::value(), "hyphen-child");
        }
        "#,
    );
    project.write(
        "src/user.rs",
        r#"pub fn value() -> &'static str { "user" }"#,
    );
    project.write(
        "src/nested/alpha.rs",
        r#"pub fn value() -> &'static str { "alpha" }"#,
    );
    project.write(
        "src/nested/deep/beta.rs",
        r#"pub fn value() -> &'static str { "beta" }"#,
    );
    project.write(
        "src/hyphen-name/child.rs",
        r#"pub fn value() -> &'static str { "hyphen-child" }"#,
    );

    project.check();
}

#[test]
fn same_name_entry_file_wins() {
    let project = TestProject::new("entry-file-wins");
    project.write_manifest();
    project.write(
        "src/lib.rs",
        r#"
        automod::dir!(pub "src");

        pub fn assert_modules() {
            assert_eq!(with_entry::value(), "entry-file");
        }
        "#,
    );
    project.write(
        "src/with-entry.rs",
        r#"pub fn value() -> &'static str { "entry-file" }"#,
    );
    project.write(
        "src/with-entry/ignored_if_entry_file_wins.rs",
        r#"compile_error!("same-name entry file should own this directory");"#,
    );

    project.check();
}

#[test]
fn cargo_bin_directory_is_not_collected() {
    let project = TestProject::new("cargo-bin");
    project.write_manifest();
    project.write(
        "src/lib.rs",
        r#"
        automod::dir!(pub "src");

        pub fn assert_modules() {
            assert_eq!(sample::value(), "sample");
        }
        "#,
    );
    project.write(
        "src/sample.rs",
        r#"pub fn value() -> &'static str { "sample" }"#,
    );
    project.write(
        "src/bin/tool.rs",
        r#"compile_error!("src/bin must keep Cargo binary semantics");"#,
    );

    project.check();
}

struct TestProject {
    dir: tempfile::TempDir,
}

impl TestProject {
    fn new(name: &str) -> Self {
        let dir = tempfile::Builder::new()
            .prefix(&format!("az-automod-{name}-"))
            .tempdir()
            .expect("create temp project");

        Self { dir }
    }

    fn write_manifest(&self) {
        self.write(
            "Cargo.toml",
            &format!(
                r#"
                [package]
                name = "az-automod-fixture"
                version = "0.0.0"
                edition = "2024"

                [dependencies]
                automod = {{ package = "az-automod", path = "{}" }}
                "#,
                env!("CARGO_MANIFEST_DIR")
            ),
        );
    }

    fn write(&self, path: &str, content: &str) {
        let path = self.dir.path().join(path);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).expect("create parent directories");
        }

        fs::write(path, content).expect("write fixture file");
    }

    fn check(&self) {
        let output = Command::new("cargo")
            .arg("check")
            .arg("--lib")
            .arg("--manifest-path")
            .arg(self.dir.path().join("Cargo.toml"))
            .arg("--quiet")
            .output()
            .expect("run cargo check");

        if !output.status.success() {
            panic!(
                "cargo check failed in {}\nstdout:\n{}\nstderr:\n{}",
                display_path(self.dir.path()),
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr)
            );
        }
    }
}

fn display_path(path: &Path) -> String {
    path.to_string_lossy().into_owned()
}
