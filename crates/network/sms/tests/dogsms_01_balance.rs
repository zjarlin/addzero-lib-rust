mod dogsms_support;

use dogsms_support::live_client;

// 01. GET /api/control/balance - 查询账户余额。
#[tokio::test]
#[ignore = "live DogSMS test requires DOGSMS_API_KEY"]
async fn dogsms_01_balance_gets_account_balance_from_live_api() {
    let balance = live_client().balance().await.unwrap();
    println!(
        "DogSMS balance: {}, frozen: {:?}, currency: {:?}",
        balance.balance, balance.frozen_balance, balance.currency
    );

    // 余额接口必须返回可用余额；币种字段由 DogSMS 服务端决定是否返回。
    assert!(balance.balance >= 0.0);
}
