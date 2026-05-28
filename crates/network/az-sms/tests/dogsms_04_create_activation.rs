mod dogsms_support;

use az_sms::dogsms::client::DogSmsActivationRequest;
use dogsms_support::{
    DOGSMS_TEST_COUNTRY_CODE, DOGSMS_TEST_SERVICE_CODE, idempotency_key, live_client,
};

// 04. POST /api/control/activations - 创建一次性取号请求。
#[tokio::test]
#[ignore = "live DogSMS test requires DOGSMS_API_KEY and creates a paid activation order"]
async fn dogsms_04_create_activation_posts_order_request_to_live_api() {
    let request =
        DogSmsActivationRequest::new(DOGSMS_TEST_SERVICE_CODE, DOGSMS_TEST_COUNTRY_CODE).unwrap();

    let order = live_client()
        .create_activation_with_idempotency_key(request, idempotency_key("activation"))
        .await
        .unwrap();

    // 订单创建成功后，后续 05/06 接口依赖这个 request_id 查询或取消。
    assert!(!order.request_id.trim().is_empty());
    assert!(!order.number.trim().is_empty());
}
