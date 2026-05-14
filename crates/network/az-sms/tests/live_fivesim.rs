use az_sms::FivesimClient;

#[tokio::test]
#[ignore = "requires a real 5sim token in FIVESIM_TOKEN and calls the live 5sim API"]
async fn fivesim_profile_accepts_configured_token() {
    let token = "eyJhbGciOiJSUzUxMiIsInR5cCI6IkpXVCJ9.eyJleHAiOjE4MDk4NDYzMzcsImlhdCI6MTc3ODMxMDMzNywicmF5IjoiMjg4ZTZkOGM4Y2I4MTEzMmFlNDNkNmYzYjI0ODM2MzUiLCJzdWIiOjQwNTM0MzB9.Wh6Rf9o8wlbED8AKxkaMDbiUpNytosKCnFua34_iifSR_RDp5OGeozx3490zJhpwAn-_V6qPuiAQJ8lxZ_NcbLazR3PNbf-ofzcN5bE0dASeKLjD5-dgFzsET4QHwVxdvb8TW5S4kmyMVH4N_WCYY8RUIzA_OQV-JV3XCBDm6r_NOt_bGHbNfCipzY9hJA1LDOnRHMsJ_nu1TAXYnHm1JSvEjfSNvJf5HsOmgfBjnoIRYu_zqYjC6MbFER2e56ZJ5KxdJklPo22wxWHAOBGfEnSX-D3Xz8u_i7tTGtq-yYEa2sEukIByeCQHpFif2c7CwOtit299C-u8qSgMrtqTGA";

    // let token = std::env::var("FIVESIM_TOKEN")
    //     .expect("set FIVESIM_TOKEN to run the live 5sim profile test");

    let client = FivesimClient::from_token(token).expect("client should be created");
    let profile = client.profile().await.expect("5sim profile should load");

    // A valid authenticated profile must include a provider account id.
    assert!(profile.id > 0);
}
