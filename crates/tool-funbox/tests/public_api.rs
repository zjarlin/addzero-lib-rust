use addzero_funbox::*;

#[test]
fn field_dto_defaults_match_expected_values() {
    let field = FieldDto::default();

    assert_eq!(field.rest_name, None);
    assert_eq!(field.rest_url, None);
    assert_eq!(field.model_name, None);
    assert_eq!(field.field_name, None);
    assert_eq!(field.field_eng, None);
    assert_eq!(field.field_type, None);
    assert_eq!(field.field_long, None);
    assert!(field.is_empty());
}

#[test]
fn field_dto_builder_collects_values() {
    let field = FieldDto::builder()
        .rest_name("测试接口")
        .rest_url("/api/test")
        .model_name("测试模块")
        .field_name("测试字段")
        .field_eng("testField")
        .field_type("String")
        .field_long("255")
        .build();

    assert_eq!(field.rest_name.as_deref(), Some("测试接口"));
    assert_eq!(field.rest_url.as_deref(), Some("/api/test"));
    assert_eq!(field.model_name.as_deref(), Some("测试模块"));
    assert_eq!(field.field_name.as_deref(), Some("测试字段"));
    assert_eq!(field.field_eng.as_deref(), Some("testField"));
    assert_eq!(field.field_type.as_deref(), Some("String"));
    assert_eq!(field.field_long.as_deref(), Some("255"));
}

#[test]
fn fun_box_builder_collects_parameters_and_returns() {
    let parameter = FieldDto::string_field("姓名", "name");
    let return_field = FieldDto::builder()
        .field_name("结果")
        .field_eng("result")
        .field_type("Boolean")
        .field_long("0")
        .build();

    let fun_box = FunBox::builder()
        .rest_url("/api/test")
        .method_type("POST")
        .des("测试接口")
        .fun_name("createUser")
        .parameter(parameter.clone())
        .return_field(return_field.clone())
        .build();

    assert_eq!(fun_box.rest_url.as_deref(), Some("/api/test"));
    assert_eq!(fun_box.method_type.as_deref(), Some("POST"));
    assert_eq!(fun_box.des.as_deref(), Some("测试接口"));
    assert_eq!(fun_box.fun_name.as_deref(), Some("createUser"));
    assert_eq!(fun_box.parameters(), &[parameter]);
    assert_eq!(fun_box.returns, vec![return_field]);
    assert!(fun_box.has_parameters());
    assert!(fun_box.has_returns());
    assert_eq!(fun_box.signature(), "POST /api/test createUser");
}

#[test]
fn registry_supports_manual_registration_and_lookup() {
    let get_user = FunBox::builder()
        .rest_url("/api/user")
        .method_type("GET")
        .fun_name("getUser")
        .build();
    let create_user = FunBox::builder()
        .rest_url("/api/user")
        .method_type("POST")
        .fun_name("createUser")
        .build();

    let mut registry = FunBoxRegistry::new();
    registry
        .register(get_user.clone())
        .register(create_user.clone());

    assert_eq!(registry.all().len(), 2);
    assert_eq!(registry.find_by_fun_name("getUser"), Some(&get_user));
    assert_eq!(registry.find_by_rest_url("/api/user").len(), 2);
    assert_eq!(registry.find_by_method_type("post"), vec![&create_user]);
    assert_eq!(
        AbsFunBox::get_all_fun(&registry),
        vec![get_user, create_user]
    );
}

#[test]
fn serde_roundtrip_uses_jvm_field_names() {
    let fun_box = FunBox::builder()
        .rest_url("/api/demo")
        .method_type("GET")
        .fun_name("demo")
        .parameter(FieldDto::string_field("姓名", "name"))
        .build();

    let json = serde_json::to_string(&fun_box).expect("json should serialize");
    assert!(json.contains("\"rest_url\""));
    assert!(json.contains("\"method_type\""));
    assert!(json.contains("\"paramiter\""));

    let decoded: FunBox = serde_json::from_str(&json).expect("json should deserialize");
    assert_eq!(decoded, fun_box);
}
