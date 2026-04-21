use addzero_yml::*;
use serde::Deserialize;
use serde_yaml::Value;
use std::fs;
use tempfile::TempDir;

#[derive(Debug, Deserialize, PartialEq, Eq)]
struct AppConfig {
    spring: SpringSection,
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
struct SpringSection {
    application: ApplicationSection,
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
struct ApplicationSection {
    name: String,
}

#[test]
fn load_yaml_reads_typed_value() {
    let temp = TempDir::new().expect("temp dir should be created");
    let path = temp.path().join("application.yml");
    fs::write(&path, "spring:\n  application:\n    name: addzero-lib\n")
        .expect("yaml should be written");

    let parsed = load_yaml::<AppConfig, _>(&path).expect("yaml should deserialize");

    assert_eq!(
        parsed,
        AppConfig {
            spring: SpringSection {
                application: ApplicationSection {
                    name: "addzero-lib".to_owned(),
                },
            },
        }
    );
}

#[test]
fn yaml_path_macro_supports_bracket_segments() {
    let doc = YamlDoc::from_value(
        serde_yaml::from_str(
            "spring:\n  datasource:\n    jdbc-url: jdbc:mysql://localhost:3306/app\n",
        )
        .expect("yaml should parse"),
    );

    let path = yaml_path!(spring.datasource["jdbc-url"]);
    let value = doc
        .get_path(&path)
        .and_then(Value::as_str)
        .map(ToOwned::to_owned);
    let macro_value = yaml_get!(doc, spring.datasource["jdbc-url"])
        .and_then(Value::as_str)
        .map(ToOwned::to_owned);

    assert_eq!(value.as_deref(), Some("jdbc:mysql://localhost:3306/app"));
    assert_eq!(macro_value, value);
}

#[test]
fn env_subst_uses_env_and_defaults() {
    let expanded = env_subst("${CARGO_MANIFEST_DIR}");
    let defaulted = env_subst("jdbc:${ADDZERO_MISSING_TEST_VAR:sqlite}");
    let empty = env_subst("${ADDZERO_MISSING_TEST_VAR}");

    assert!(!expanded.is_empty());
    assert_eq!(defaulted, "jdbc:sqlite");
    assert_eq!(empty, "");
}

#[test]
fn resolve_resource_prefers_yml_and_falls_back_to_requested_name() {
    let temp = TempDir::new().expect("temp dir should be created");
    fs::write(temp.path().join("config.yml"), "yml: content\n").expect("yml should exist");
    fs::write(temp.path().join("config.yaml"), "yaml: content\n").expect("yaml should exist");
    fs::write(temp.path().join("other.yaml"), "yaml: content\n").expect("yaml should exist");

    let spring_yaml = SpringYaml::from_dir(temp.path());

    assert_eq!(
        spring_yaml.resolve_resource("config"),
        temp.path().join("config.yml")
    );
    assert_eq!(
        spring_yaml.resolve_resource("other.yaml"),
        temp.path().join("other.yaml")
    );
    assert_eq!(
        spring_yaml.resolve_resource("missing.txt"),
        temp.path().join("missing.txt")
    );
}

#[test]
fn load_active_uses_profile_file_and_falls_back_to_primary() {
    let temp = TempDir::new().expect("temp dir should be created");
    fs::write(
        temp.path().join("application.yml"),
        "spring:\n  profiles:\n    active: dev\nfeature:\n  enabled: false\n",
    )
    .expect("application.yml should be written");
    fs::write(
        temp.path().join("application-dev.yaml"),
        "feature:\n  enabled: true\n",
    )
    .expect("profile file should be written");

    let active = SpringYaml::from_dir(temp.path())
        .load_active()
        .expect("active file should load");
    let enabled = active
        .get("feature.enabled")
        .expect("path should parse")
        .and_then(Value::as_bool);

    assert_eq!(enabled, Some(true));

    fs::remove_file(temp.path().join("application-dev.yaml"))
        .expect("profile file should be removed");
    let fallback = SpringYaml::from_dir(temp.path())
        .load_active()
        .expect("fallback file should load");
    let fallback_enabled = fallback
        .get("feature.enabled")
        .expect("path should parse")
        .and_then(Value::as_bool);
    assert_eq!(fallback_enabled, Some(false));
}

#[test]
fn database_config_reader_handles_supported_layouts() {
    let temp = TempDir::new().expect("temp dir should be created");
    fs::write(
        temp.path().join("application.yml"),
        "spring:\n  datasource:\n    url: jdbc:mysql://localhost:3306/app\n    username: root\n    password: secret\n",
    )
    .expect("application should be written");

    let single = DatabaseConfigReader::read(temp.path(), None)
        .expect("config should load")
        .expect("config should exist");
    assert_eq!(single.jdbc_url, "jdbc:mysql://localhost:3306/app");
    assert_eq!(single.jdbc_username.as_deref(), Some("root"));
    assert_eq!(single.jdbc_password.as_deref(), Some("secret"));

    fs::write(
        temp.path().join("application.yml"),
        "spring:\n  datasource:\n    master:\n      url: jdbc:mysql://localhost:3306/master\n      username: root\n      password: master-pass\n    slave:\n      url: jdbc:mysql://localhost:3306/slave\n      username: slave-user\n      password: slave-pass\n",
    )
    .expect("application should be overwritten");

    let preferred = DatabaseConfigReader::read(temp.path(), Some("slave"))
        .expect("config should load")
        .expect("preferred config should exist");
    assert_eq!(preferred.jdbc_url, "jdbc:mysql://localhost:3306/slave");
    assert_eq!(preferred.jdbc_username.as_deref(), Some("slave-user"));
    assert_eq!(preferred.jdbc_password.as_deref(), Some("slave-pass"));

    let fallback = DatabaseConfigReader::read(temp.path(), None)
        .expect("config should load")
        .expect("fallback config should exist");
    assert_eq!(fallback.jdbc_url, "jdbc:mysql://localhost:3306/master");
}

#[test]
fn database_config_reader_returns_none_when_missing() {
    let temp = TempDir::new().expect("temp dir should be created");
    fs::write(temp.path().join("application.yml"), "app:\n  name: demo\n")
        .expect("application should be written");

    let config = DatabaseConfigReader::read(temp.path(), None).expect("config should load");

    assert_eq!(config, None);
}
