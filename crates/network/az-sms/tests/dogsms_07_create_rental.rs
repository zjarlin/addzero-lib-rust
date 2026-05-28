mod dogsms_support;

use az_sms::dogsms::client::DogSmsRentalRequest;
use dogsms_support::{
    DOGSMS_TEST_RENTAL_COUNTRY_CODE, DOGSMS_TEST_RENTAL_MONTHS, idempotency_key, live_client,
};

// 07. POST /api/control/rentals - 创建长期租号订单。
#[tokio::test]
#[ignore = "live DogSMS test requires DOGSMS_API_KEY and creates a paid rental order"]
async fn dogsms_07_create_rental_posts_rental_request_to_live_api() {
    let request =
        DogSmsRentalRequest::new(DOGSMS_TEST_RENTAL_COUNTRY_CODE, DOGSMS_TEST_RENTAL_MONTHS)
            .unwrap();

    let rental = live_client()
        .create_rental_with_idempotency_key(request, idempotency_key("rental"))
        .await
        .unwrap();

    // 租号创建成功后必须返回租号订单 ID 和号码。
    assert!(!rental.rental_id.trim().is_empty());
    assert!(!rental.number.trim().is_empty());
}
