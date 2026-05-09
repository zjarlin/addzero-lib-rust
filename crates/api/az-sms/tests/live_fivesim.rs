use az_sms::FivesimClient;

#[tokio::test]
#[ignore = "requires a real 5sim token in FIVESIM_TOKEN and calls the live 5sim API"]
async fn fivesim_profile_accepts_configured_token() {
    let token = std::env::var("FIVESIM_TOKEN")
        .expect("set FIVESIM_TOKEN to run the live 5sim profile test");

    let client = FivesimClient::from_token(token).expect("client should be created");
    let profile = client.profile().await.expect("5sim profile should load");

    // A valid authenticated profile must include a provider account id.
    assert!(profile.id > 0);
}
