use std::fs;

use az_dict_spec::specification::{DictionaryItemSpec, DictionarySpec, RawValueKind};
use az_micro_dict::contribution::{
    DictBuildGenerator, DictionaryContribution, DictionaryContributor, RuoyiDictRow,
    RuoyiDictionaryContributor, StaticDictionaryContributor,
};
use tempfile::TempDir;

#[test]
fn static_contributor_writes_specs_and_enum_source() {
    let temp = TempDir::new().expect("temp dir should be available");
    let generated = DictBuildGenerator::new()
        .add_contributor(StaticDictionaryContributor::new(vec![
            DictionaryContribution::new(
                "ShellEntryKind",
                DictionarySpec {
                    code: "shell_entry_kind".to_string(),
                    name: "Shell Entry Kind".to_string(),
                    description: Some("Shell entry category".to_string()),
                    scope: "platform".to_string(),
                    raw_value_kind: RawValueKind::String,
                    open_enum: false,
                    unknown_variant: None,
                    sort_index: 0,
                    items: vec![
                        DictionaryItemSpec {
                            code: "alias".to_string(),
                            label: "别名".to_string(),
                            description: None,
                            raw_int_value: None,
                            raw_text_value: Some("alias".to_string()),
                            sort_index: 10,
                            enabled: true,
                            meta: None,
                        },
                        DictionaryItemSpec {
                            code: "export".to_string(),
                            label: "环境变量".to_string(),
                            description: None,
                            raw_int_value: None,
                            raw_text_value: Some("export".to_string()),
                            sort_index: 20,
                            enabled: true,
                            meta: None,
                        },
                    ],
                },
            ),
        ]))
        .generate_to(temp.path())
        .expect("dictionary files should be generated");

    let enum_source =
        fs::read_to_string(generated.enums_file).expect("generated enum source should exist");
    syn::parse_file(&enum_source).expect("generated enum source should parse as Rust");
    assert!(enum_source.contains("name = ShellEntryKind"));
    assert!(enum_source.contains("dict = \"shell_entry_kind\""));
    assert!(enum_source.contains("include_str!"));

    let spec_source =
        fs::read_to_string(&generated.spec_files[0]).expect("generated spec should exist");
    assert!(spec_source.contains("\"label\": \"别名\""));
}

#[test]
fn ruoyi_contributor_groups_enabled_rows_as_string_dictionary() {
    let contribution = RuoyiDictionaryContributor::new(
        vec![
            RuoyiDictRow {
                dict_type: "sys_user_sex".to_string(),
                dict_name: "用户性别".to_string(),
                dict_label: "男".to_string(),
                dict_value: "0".to_string(),
                dict_sort: 2,
                status: "0".to_string(),
                remark: Some("male".to_string()),
                css_class: None,
                list_class: Some("primary".to_string()),
            },
            RuoyiDictRow {
                dict_type: "sys_user_sex".to_string(),
                dict_name: "用户性别".to_string(),
                dict_label: "停用项".to_string(),
                dict_value: "9".to_string(),
                dict_sort: 1,
                status: "1".to_string(),
                remark: None,
                css_class: None,
                list_class: None,
            },
        ],
        "ruoyi",
    )
    .contribute()
    .expect("ruoyi rows should map to dictionaries")
    .pop()
    .expect("one dictionary should be produced");

    assert_eq!(contribution.spec.code, "sys_user_sex");
    assert_eq!(contribution.enum_name, "SysUserSex");
    assert_eq!(contribution.spec.items.len(), 1);
    assert_eq!(contribution.spec.items[0].code, "0");
    assert_eq!(
        contribution.spec.items[0].raw_text_value.as_deref(),
        Some("0")
    );
    assert_eq!(
        contribution.spec.items[0].meta.as_ref().unwrap()["listClass"],
        "primary"
    );
}

#[test]
fn generator_rejects_duplicate_dictionary_codes() {
    let duplicate = DictionaryContribution::new(
        "Dup",
        DictionarySpec {
            code: "dup".to_string(),
            name: "Duplicate".to_string(),
            description: None,
            scope: "test".to_string(),
            raw_value_kind: RawValueKind::String,
            open_enum: false,
            unknown_variant: None,
            sort_index: 0,
            items: vec![DictionaryItemSpec {
                code: "a".to_string(),
                label: "A".to_string(),
                description: None,
                raw_int_value: None,
                raw_text_value: Some("a".to_string()),
                sort_index: 0,
                enabled: true,
                meta: None,
            }],
        },
    );

    let temp = TempDir::new().expect("temp dir should be available");
    let error = DictBuildGenerator::new()
        .add_contributor(StaticDictionaryContributor::new(vec![
            duplicate.clone(),
            duplicate,
        ]))
        .generate_to(temp.path())
        .expect_err("duplicate dictionary codes should be rejected");

    assert!(error.to_string().contains("duplicate dictionary code dup"));
}
