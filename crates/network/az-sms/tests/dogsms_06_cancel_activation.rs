mod dogsms_support;

use dogsms_support::{cancel_request_id, idempotency_key, live_client};

// 06. PATCH /api/control/activations/{requestId}/cancel - 取消等待中的取号请求。
#[tokio::test]
#[ignore = "live DogSMS test requires DOGSMS_API_KEY and mutates an existing activation order"]
async fn dogsms_06_cancel_activation_patches_cancel_endpoint_on_live_api() {
    let order = live_client()
        .cancel_activation_with_idempotency_key(cancel_request_id(), idempotency_key("cancel"))
        .await
        .unwrap();

    // 取消接口成功时必须仍能识别被取消的订单。
    assert!(!order.request_id.trim().is_empty());
}
