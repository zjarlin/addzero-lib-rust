mod dogsms_support;

use dogsms_support::live_client;

// 02. GET /api/control/services - 查询可用服务目录。
#[tokio::test]
#[ignore = "live DogSMS test requires DOGSMS_API_KEY"]
async fn dogsms_02_services_gets_service_catalog_from_live_api() {
    let services = live_client().services().await.unwrap();

    // 服务目录至少应返回可供后续库存查询使用的服务编码字段。
    assert!(
        services
            .iter()
            .all(|service| !service.code.trim().is_empty())
    );
}
