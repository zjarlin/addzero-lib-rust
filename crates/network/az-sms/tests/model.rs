use az_sms::model::{SmsActivationRequest, SmsInbox, SmsOrder, SmsOrderStatus, WaitForSmsOptions};
use std::time::Duration;

#[test]
fn activation_request_rejects_blank_product() {
    let err = SmsActivationRequest::new("usa", "any", " ").unwrap_err();
    assert!(err.to_string().contains("product cannot be blank"));
}

#[test]
fn activation_request_keeps_optional_query_values() {
    let request = SmsActivationRequest::new("usa", "any", "telegram")
        .unwrap()
        .forwarding(true)
        .number("15551234567")
        .reuse(true)
        .voice(false)
        .ref_code("partner");

    assert_eq!(request.country, "usa");
    assert_eq!(request.forwarding, Some(true));
    assert_eq!(request.number.as_deref(), Some("15551234567"));
    assert_eq!(request.reuse, Some(true));
    assert_eq!(request.voice, Some(false));
    assert_eq!(request.ref_code.as_deref(), Some("partner"));
}

#[test]
fn order_parses_null_sms_as_empty_list() {
    let order: SmsOrder = serde_json::from_str(
        r#"{
          "id":11631253,
          "phone":"+447350690992",
          "operator":"vodafone",
          "product":"telegram",
          "price":21,
          "status":"PENDING",
          "expires":"2018-10-13T08:28:38.809469028Z",
          "sms":null,
          "created_at":"2018-10-13T08:13:38.809469028Z",
          "forwarding":false,
          "forwarding_number":"",
          "country":"england"
        }"#,
    )
    .unwrap();

    // Some providers return `sms: null` immediately after purchase; callers should see a normal empty list.
    assert!(order.sms.is_empty());
    assert_eq!(order.status, SmsOrderStatus::Pending);
}

#[test]
fn order_parses_received_sms_code() {
    let order: SmsOrder = serde_json::from_str(
        r#"{
          "id":11631253,
          "created_at":"2018-10-13T08:13:38.809469028Z",
          "phone":"+447350690992",
          "product":"telegram",
          "price":21,
          "status":"RECEIVED",
          "expires":"2018-10-13T08:28:38.809469028Z",
          "sms":[{
            "created_at":"2018-10-13T08:20:38.809469028Z",
            "date":"2018-10-13T08:19:38Z",
            "sender":"Telegram",
            "text":"Telegram code: 09363",
            "code":"09363"
          }],
          "forwarding":false,
          "forwarding_number":"",
          "country":"england"
        }"#,
    )
    .unwrap();

    // The provider-extracted code is the primary value consumers usually need.
    assert_eq!(
        order.sms.first().and_then(|sms| sms.code.as_deref()),
        Some("09363")
    );
    assert_eq!(order.status, SmsOrderStatus::Received);
}

#[test]
fn order_status_serde_uses_uppercase_provider_values() {
    assert_eq!(
        serde_json::to_string(&SmsOrderStatus::Pending).unwrap(),
        r#""PENDING""#
    );
    assert_eq!(
        serde_json::from_str::<SmsOrderStatus>(r#""RECEIVED""#).unwrap(),
        SmsOrderStatus::Received
    );
    assert_eq!(
        serde_json::from_str::<SmsOrderStatus>(r#""EXPIRED""#).unwrap(),
        SmsOrderStatus::Unknown
    );
}

#[test]
fn inbox_parses_uppercase_provider_fields() {
    let inbox: SmsInbox = serde_json::from_str(
        r#"{
          "Data":[{
            "ID":844928,
            "created_at":"2017-09-05T15:48:33.763297Z",
            "date":"2017-09-05T15:48:27Z",
            "sender":"+447350690992",
            "text":"12345",
            "code":""
          }],
          "Total":1
        }"#,
    )
    .unwrap();

    // Rented-number inbox responses use `ID`, not `id`; both should map to `SmsMessage::id`.
    assert_eq!(inbox.total, 1);
    assert_eq!(inbox.messages.first().and_then(|sms| sms.id), Some(844928));
}

#[test]
fn wait_options_reject_zero_interval() {
    let err = WaitForSmsOptions::new(Duration::from_secs(1), Duration::from_secs(0)).unwrap_err();
    assert!(err.to_string().contains("interval cannot be zero"));
}
