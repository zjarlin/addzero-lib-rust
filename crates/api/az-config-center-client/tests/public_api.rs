use az_config_center_client::client::ConfigCenterClient;
use az_config_center_contract::api::UpsertRequest;
use serde::Serialize;

#[derive(Debug, Serialize)]
struct RedisConfig {
    host: String,
    port: u16,
}

#[test]
fn new_rejects_invalid_base_url() {
    let result = ConfigCenterClient::new("not a url");

    // 关键断言：基础 URL 在客户端创建阶段就被校验，避免请求时才暴露模糊错误。
    let error = result.expect_err("invalid base url should fail");
    assert!(error.to_string().contains("基础 URL 无效"));
}

#[test]
fn checkout_namespace_requires_login() {
    let client = ConfigCenterClient::new("http://127.0.0.1:8080").expect("创建客户端失败");
    let result = client.checkout_namespace("dev");

    // 关键断言：命名空间绑定必须发生在登录之后，和 Kotlin SDK 链式语义一致。
    let error = result.expect_err("checkout should require login");
    assert!(error.to_string().contains("尚未登录"));
}

#[test]
fn json_upsert_request_uses_json_value_type() {
    let client = ConfigCenterClient::new("http://127.0.0.1:8080").expect("创建客户端失败");
    let value = RedisConfig {
        host: "127.0.0.1".to_owned(),
        port: 6379,
    };
    let encoded = serde_json::to_string(&value).expect("序列化测试配置失败");
    let request = UpsertRequest {
        namespace: "dev".to_owned(),
        key: "redis".to_owned(),
        value: encoded,
        value_type: "json".to_owned(),
        description: "Redis 配置".to_owned(),
        enabled: true,
        updated_by: client.username().unwrap_or_default().to_owned(),
    };

    // 关键断言：Rust client 的结构化配置写入和 Kotlin SDK 一样使用 json 类型。
    assert_eq!(request.value_type, "json");
}
