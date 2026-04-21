use addzero_toml::*;
use tempfile::TempDir;

#[test]
fn parses_and_serializes_version_catalog() {
    let source = r#"[versions]
kotlin = "2.1.0"

[libraries]
hutool = { group = "cn.hutool", name = "hutool-all", version.ref = "kotlin" }

[plugins]
kotlin = { id = "org.jetbrains.kotlin.jvm", version.ref = "kotlin" }

[bundles]
spring = ["spring-boot", "spring-core"]
"#;

    let catalog = VersionCatalog::from_str(source).expect("catalog should parse");
    let expected = catalog! {
        versions {
            kotlin = "2.1.0",
        }
        libraries {
            hutool = { group: "cn.hutool", name: "hutool-all", version_ref: "kotlin" },
        }
        plugins {
            kotlin = { id: "org.jetbrains.kotlin.jvm", version_ref: "kotlin" },
        }
        bundles {
            spring = ["spring-boot", "spring-core"],
        }
    };

    assert_eq!(catalog, expected);

    let rendered = catalog.to_string_pretty();
    assert!(rendered.contains("version.ref = \"kotlin\""));
    let reparsed = VersionCatalog::from_str(&rendered).expect("rendered catalog should parse");
    assert_eq!(reparsed, expected);
}

#[test]
fn load_or_init_creates_default_catalog_only_when_requested() {
    let temp = TempDir::new().expect("temp dir should be created");
    let path = temp.path().join("gradle/libs.versions.toml");

    let catalog = VersionCatalog::load_or_init(&path).expect("catalog should load");

    assert!(path.exists());
    assert_eq!(
        catalog,
        VersionCatalog::from_str(DEFAULT_VERSION_CATALOG_TEMPLATE)
            .expect("default template should parse")
    );
}

#[test]
fn merge_many_uses_jvm_compatibility_rules() {
    let left = catalog! {
        versions { kotlin = "2.1.0" }
        libraries {
            hutool = { group: "cn.hutool", name: "hutool-all", version_ref: "kotlin" },
        }
        plugins {
            kotlin = { id: "org.jetbrains.kotlin.jvm", version_ref: "kotlin" },
        }
        bundles {
            spring = ["spring-boot"],
        }
    };
    let right = catalog! {
        versions { kotlin = "2.2.0", serde = "1.0.0" }
        libraries {
            hutool = { group: "cn.hutool", name: "hutool-all", version: "6.0.0" },
            serde = { group: "serde", name: "serde", version_ref: "serde" },
        }
        plugins {
            kotlin_android = { id: "org.jetbrains.kotlin.jvm", version: "2.2.0" },
            android = { id: "com.android.application", version: "8.8.0" },
        }
        bundles {
            spring = ["spring-boot", "spring-core"],
            common = ["serde"],
        }
    };

    let merged = VersionCatalog::merge_many(vec![left, right]);

    assert_eq!(
        merged.versions,
        vec![
            VersionEntry {
                version_ref: "kotlin".to_owned(),
                version: "2.1.0".to_owned(),
            },
            VersionEntry {
                version_ref: "serde".to_owned(),
                version: "1.0.0".to_owned(),
            },
        ]
    );
    assert_eq!(merged.libraries.len(), 2);
    assert_eq!(merged.libraries[0].group, "cn.hutool");
    assert_eq!(merged.libraries[0].version.as_deref(), Some("6.0.0"));
    assert_eq!(merged.plugins.len(), 2);
    assert_eq!(merged.plugins[0].id, "com.android.application");
    assert_eq!(merged.plugins[1].id, "org.jetbrains.kotlin.jvm");
    assert_eq!(merged.bundles.len(), 2);
    assert_eq!(merged.bundles[1].key, "spring");
    assert_eq!(merged.bundles[1].libraries, vec!["spring-boot".to_owned()]);
}

#[test]
fn insert_after_table_supports_bare_and_bracket_tags() {
    let source = "[plugins]\nkotlin = { id = \"org.jetbrains.kotlin.jvm\" }\n";

    let inserted_bare = insert_after_table(
        source,
        "plugins",
        "android = { id = \"com.android.application\" }",
    );
    let inserted_bracket = insert_after_table(
        source,
        "[plugins]",
        "android = { id = \"com.android.application\" }",
    );

    assert_eq!(inserted_bare, inserted_bracket);
    assert!(inserted_bare.contains("android = { id = \"com.android.application\" }"));
}

#[test]
fn catalog_macro_matches_parser_output() {
    let macro_catalog = catalog! {
        versions {
            kotlin = "2.1.0",
        }
        libraries {
            hutool = { group: "cn.hutool", name: "hutool-all", version_ref: "kotlin" },
        }
        plugins {
            kotlin = { id: "org.jetbrains.kotlin.jvm", version_ref: "kotlin" },
        }
        bundles {
            spring = ["spring-boot", "spring-core"],
        }
    };

    let parsed = VersionCatalog::from_str(&macro_catalog.to_string_pretty())
        .expect("macro catalog should round-trip");

    assert_eq!(parsed, macro_catalog);
}
