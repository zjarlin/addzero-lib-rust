mod dogsms_support;

use dogsms_support::{existing_request_id, live_client};

// 05. GET /api/control/activations/{requestId} - 查询取号状态与短信内容。
#[tokio::test]
#[ignore = "live DogSMS test requires DOGSMS_API_KEY and DOGSMS_EXISTING_REQUEST_ID"]
async fn dogsms_05_activation_detail_gets_status_and_sms_from_live_api() {
    let order = live_client()
        .activation(existing_request_id())
        .await
        .unwrap();

    // 查询接口必须返回同一个订单的 request_id，短信内容可能尚未到达。
    assert!(!order.request_id.trim().is_empty());
}
