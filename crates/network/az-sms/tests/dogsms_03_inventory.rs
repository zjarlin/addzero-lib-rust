mod dogsms_support;

use dogsms_support::{DOGSMS_TEST_COUNTRY_CODE, DOGSMS_TEST_SERVICE_CODE, live_client};

// 03. GET /api/control/inventory - 查询库存与起价。
#[tokio::test]
#[ignore = "live DogSMS test requires DOGSMS_API_KEY"]
async fn dogsms_03_inventory_gets_stock_and_pricing_from_live_api() {
    let inventory = live_client()
        .inventory(DOGSMS_TEST_SERVICE_CODE, Some(DOGSMS_TEST_COUNTRY_CODE))
        .await
        .unwrap();

    // 每条库存记录必须至少有国家编码；库存为空时服务端可以返回空数组。
    assert!(
        inventory
            .iter()
            .all(|item| !item.country_code.trim().is_empty())
    );
}
